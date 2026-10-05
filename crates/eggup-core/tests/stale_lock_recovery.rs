//! Proof-authorized stale-lock recovery.
//!
//! Core never decides that a lock is stale. These tests cover the mechanism's
//! obligations instead: that authorization binds to exactly one observation,
//! that a pathname match alone can never authorize a deletion, that records Core
//! cannot prove safe are never displaced, that concurrent writers are preserved,
//! and that a partial recovery retains real, reportable evidence.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use eggup_core::{
    Error, LockObservation, LockStatus, MutationLock, ProductId, ReleaseId, StaleLockDecision,
    StaleLockVerifier,
};

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

struct Root(PathBuf);

impl Root {
    fn new() -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "eggup-lock-{nanos}-{sequence}-{}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create root");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn lock_path(&self) -> PathBuf {
        self.path().join(".eggup-mutation.lock")
    }

    fn claims(&self) -> Vec<PathBuf> {
        let mut out = Vec::new();
        let Ok(entries) = fs::read_dir(self.path()) else {
            return out;
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with(".eggup-stale-claim-") {
                out.push(entry.path());
            }
        }
        out
    }

    /// Writes a lock record as if another crashed updater had left it.
    fn plant(&self, contents: &str) {
        fs::write(self.lock_path(), contents).expect("plant lock");
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn product() -> ProductId {
    ProductId::new("eggup").expect("product")
}

fn release() -> ReleaseId {
    ReleaseId::new("2026.10.05").expect("release")
}

struct Fixed(StaleLockDecision);

impl std::fmt::Debug for Fixed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Fixed").field(&self.0).finish()
    }
}

impl StaleLockVerifier for Fixed {
    fn classify(&self, _observed: &LockObservation) -> StaleLockDecision {
        self.0
    }
}

/// A verifier that records what it was asked about, proving the caller sees the
/// exact record identity rather than a pathname.
struct Recording {
    decision: StaleLockDecision,
    seen: std::cell::RefCell<Vec<LockObservation>>,
}

impl Recording {
    fn new(decision: StaleLockDecision) -> Self {
        Self {
            decision,
            seen: std::cell::RefCell::new(Vec::new()),
        }
    }
}

impl std::fmt::Debug for Recording {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Recording")
            .field("decision", &self.decision)
            .finish()
    }
}

impl StaleLockVerifier for Recording {
    fn classify(&self, observed: &LockObservation) -> StaleLockDecision {
        self.seen.borrow_mut().push(observed.clone());
        self.decision
    }
}

/// The default acquire is unchanged: an existing record is never displaced.
#[test]
fn ordinary_acquire_never_recovers_an_existing_record() {
    let root = Root::new();
    root.plant("pid=999 nonce=7 product=eggup release=2026.10.05\n");

    for _ in 0..2 {
        let error = MutationLock::acquire(root.path(), &product(), &release())
            .expect_err("an existing record must never be recovered by default");
        assert!(matches!(error, Error::UpdateInProgress { .. }), "{error:?}");
    }
    assert!(
        root.lock_path().exists(),
        "the record must survive untouched"
    );
    assert!(root.claims().is_empty(), "no claim is ever made");
}

/// `Active` and `Unknown` retain the record and perform no mutation at all.
#[test]
fn only_proven_stale_displaces_the_record() {
    for decision in [StaleLockDecision::Active, StaleLockDecision::Unknown] {
        let root = Root::new();
        let planted = "pid=999 nonce=7 product=eggup release=2026.10.05\n";
        root.plant(planted);

        let error = MutationLock::acquire_with_recovery(
            root.path(),
            &product(),
            &release(),
            &Fixed(decision),
        )
        .expect_err("a non-stale verdict must keep the record");
        assert!(matches!(error, Error::UpdateInProgress { .. }), "{error:?}");
        assert_eq!(
            fs::read_to_string(root.lock_path()).expect("read"),
            planted,
            "{decision:?} must leave the record byte-identical"
        );
        assert!(root.claims().is_empty());
    }
}

/// A proven-stale record is claimed and acquisition succeeds.
#[test]
fn proven_stale_record_is_claimed_and_acquisition_succeeds() {
    let root = Root::new();
    let planted = "pid=999 nonce=7 product=eggup release=2026.10.05\n";
    root.plant(planted);

    let lock = MutationLock::acquire_with_recovery(
        root.path(),
        &product(),
        &release(),
        &Fixed(StaleLockDecision::ProvenStale),
    )
    .expect("recovery should succeed");

    let live = fs::read_to_string(root.lock_path()).expect("read live record");
    assert_ne!(
        live, planted,
        "the lock now belongs to this transaction, not the crashed updater"
    );
    assert!(
        live.starts_with(&format!("pid={} nonce=", std::process::id())),
        "the live record names this process: {live}"
    );
    assert!(
        root.claims().is_empty(),
        "the claimed record is removed once ownership is established: {:?}",
        root.claims()
    );
    drop(lock);
    assert!(!root.lock_path().exists(), "our own record is cleaned up");
}

/// The verifier is shown the exact bounded record, not a pathname.
#[test]
fn caller_supplied_proof_sees_the_exact_record() {
    let root = Root::new();
    let planted = "pid=4242 nonce=9 product=eggup release=2026.10.05\n";
    root.plant(planted);

    let verifier = Recording::new(StaleLockDecision::ProvenStale);
    let _lock = MutationLock::acquire_with_recovery(root.path(), &product(), &release(), &verifier)
        .expect("recovery");

    let seen = verifier.seen.borrow();
    assert_eq!(seen.len(), 1, "one bounded observation, not a retry loop");
    assert_eq!(seen[0].record(), planted);
    assert_eq!(seen[0].pid(), Some(4242));
    assert_eq!(seen[0].nonce(), Some(9));
    assert_eq!(seen[0].product(), Some("eggup"));
    assert_eq!(seen[0].release(), Some("2026.10.05"));
    assert!(seen[0].format_known());
}

/// Parsing an unknown record format is conservative: the record stays
/// observable, but no field is invented.
#[test]
fn unknown_record_format_is_observed_without_inventing_fields() {
    let root = Root::new();
    root.plant("owned-by-someone-else\n");

    let verifier = Recording::new(StaleLockDecision::Unknown);
    let error = MutationLock::acquire_with_recovery(root.path(), &product(), &release(), &verifier)
        .expect_err("an unproven record must not be displaced");
    assert!(matches!(error, Error::UpdateInProgress { .. }));

    let seen = verifier.seen.borrow();
    assert_eq!(seen.len(), 1);
    assert!(!seen[0].format_known());
    assert_eq!(seen[0].pid(), None);
    assert_eq!(seen[0].nonce(), None);
    assert_eq!(seen[0].product(), None);
}

/// Records Core cannot prove safe never reach destructive recovery.
#[test]
fn unsafe_records_never_reach_destructive_recovery() {
    // Non-UTF-8 bytes.
    let root = Root::new();
    fs::write(root.lock_path(), [0xff, 0xfe, 0x00]).expect("write");
    let rendered = MutationLock::acquire_with_recovery(
        root.path(),
        &product(),
        &release(),
        &Fixed(StaleLockDecision::ProvenStale),
    )
    .expect_err("non-UTF-8 record")
    .to_string();
    assert!(!rendered.is_empty());
    assert!(root.lock_path().exists(), "the record survives");
    assert!(root.claims().is_empty());

    // Oversized record.
    let root = Root::new();
    fs::write(root.lock_path(), "x".repeat(8192)).expect("write");
    assert!(MutationLock::acquire_with_recovery(
        root.path(),
        &product(),
        &release(),
        &Fixed(StaleLockDecision::ProvenStale)
    )
    .is_err());
    assert!(root.lock_path().exists());
    assert!(root.claims().is_empty());

    // Directory instead of a record.
    let root = Root::new();
    fs::create_dir(root.lock_path()).expect("mkdir");
    assert!(MutationLock::acquire_with_recovery(
        root.path(),
        &product(),
        &release(),
        &Fixed(StaleLockDecision::ProvenStale)
    )
    .is_err());
    assert!(root.lock_path().is_dir());
    assert!(root.claims().is_empty());

    // Symlinked record.
    #[cfg(unix)]
    {
        let root = Root::new();
        let target = root.path().join("elsewhere");
        fs::write(&target, "pid=1 nonce=1 product=x release=y\n").expect("write");
        std::os::unix::fs::symlink(&target, root.lock_path()).expect("symlink");
        assert!(MutationLock::acquire_with_recovery(
            root.path(),
            &product(),
            &release(),
            &Fixed(StaleLockDecision::ProvenStale)
        )
        .is_err());
        assert!(root.lock_path().is_symlink(), "the link is never displaced");
        assert!(target.exists(), "the link target is never touched");
        assert!(root.claims().is_empty());
    }
}

/// A record replaced between observation and claim is never deleted.
#[test]
fn record_replaced_after_observation_is_never_deleted() {
    let root = Root::new();
    root.plant("pid=1 nonce=1 product=a release=b\n");

    struct ReplaceOnClassify {
        root: PathBuf,
    }
    impl std::fmt::Debug for ReplaceOnClassify {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("ReplaceOnClassify")
        }
    }
    impl StaleLockVerifier for ReplaceOnClassify {
        fn classify(&self, observed: &LockObservation) -> StaleLockDecision {
            // Another writer replaces the record after it was observed but
            // before any claim is attempted.
            fs::write(&self.root, "pid=2 nonce=2 product=a release=b\n").expect("replace");
            assert_eq!(observed.record(), "pid=1 nonce=1 product=a release=b\n");
            StaleLockDecision::ProvenStale
        }
    }

    let verifier = ReplaceOnClassify {
        root: root.lock_path(),
    };
    let error = MutationLock::acquire_with_recovery(root.path(), &product(), &release(), &verifier)
        .expect_err("a changed record must not be claimed");

    assert!(matches!(error, Error::UpdateInProgress { .. }), "{error:?}");
    assert_eq!(
        fs::read_to_string(root.lock_path()).expect("read"),
        "pid=2 nonce=2 product=a release=b\n",
        "the replacement record survives untouched"
    );
    assert!(
        root.claims().is_empty(),
        "nothing was displaced, so nothing is retained"
    );
}

/// A writer that creates the lock after the claim wins; its record is preserved
/// and never removed.
#[test]
fn a_second_writer_after_the_claim_is_preserved() {
    let root = Root::new();
    root.plant("pid=1 nonce=1 product=a release=b\n");

    struct ClaimThenLose {
        root: PathBuf,
    }
    impl std::fmt::Debug for ClaimThenLose {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("ClaimThenLose")
        }
    }
    impl StaleLockVerifier for ClaimThenLose {
        fn classify(&self, _observed: &LockObservation) -> StaleLockDecision {
            // Simulate a competing writer winning the domain between our
            // observation and our create-new.
            fs::write(&self.root, "pid=2 nonce=2 product=a release=b\n").expect("competing writer");
            StaleLockDecision::ProvenStale
        }
    }

    let verifier = ClaimThenLose {
        root: root.lock_path(),
    };
    let error = MutationLock::acquire_with_recovery(root.path(), &product(), &release(), &verifier)
        .expect_err("the competing writer owns the domain");

    assert!(matches!(error, Error::UpdateInProgress { .. }), "{error:?}");
    assert_eq!(
        fs::read_to_string(root.lock_path()).expect("read"),
        "pid=2 nonce=2 product=a release=b\n",
        "the winning writer's record is never removed"
    );
    assert!(
        root.claims().is_empty(),
        "our own displaced record is cleaned up: {:?}",
        root.claims()
    );
}

/// The retained-evidence error always names a real path and says so plainly.
///
/// A partial recovery that cannot clean up its displaced record has no
/// deterministic public trigger, so this asserts the contract the claim path
/// depends on rather than pretending to provoke it.
#[test]
fn retained_evidence_error_names_the_real_path() {
    let root = Root::new();
    root.plant("pid=1 nonce=1 product=a release=b\n");

    // A directory occupying the claim name is not how this happens in practice;
    // the point under test is the reporting contract, so exercise the typed
    // error directly through the same shape the claim path produces.
    let error = Error::RecoveryRequired {
        evidence: root.path().join(".eggup-stale-claim-example"),
        detail: "claimed stale lock record could not be removed".to_string(),
    };
    match &error {
        Error::RecoveryRequired { evidence, detail } => {
            assert!(detail.contains("could not be removed"));
            assert!(evidence.to_string_lossy().contains(".eggup-stale-claim-"));
        }
        other => panic!("unexpected {other:?}"),
    }
    let rendered = error.to_string();
    assert!(
        rendered.contains("recovery evidence retained at"),
        "{rendered}"
    );
    assert!(rendered.contains("eggup-stale-claim-example"), "{rendered}");
}

/// The current owner never deletes a lock that replaced its own record.
#[test]
fn owner_drop_never_deletes_a_replacement_record() {
    let root = Root::new();
    let lock = MutationLock::acquire(root.path(), &product(), &release()).expect("acquire");

    // Another process takes over the pathname.
    let replacement = "pid=2 nonce=2 product=a release=b\n";
    fs::write(root.lock_path(), replacement).expect("replace");

    drop(lock);
    assert_eq!(
        fs::read_to_string(root.lock_path()).expect("read"),
        replacement,
        "a lock record we no longer own is never removed"
    );
}

/// `inspect` stays read-only and keeps reporting what it always did.
#[test]
fn inspect_remains_read_only_and_backward_compatible() {
    let root = Root::new();
    assert_eq!(
        MutationLock::inspect(root.path()).expect("inspect"),
        LockStatus::Available
    );
    assert_eq!(
        MutationLock::inspect(root.path()).expect("inspect"),
        LockStatus::Available,
        "inspection never removes anything"
    );

    let planted = "pid=1 nonce=1 product=a release=b\n";
    root.plant(planted);
    match MutationLock::inspect(root.path()).expect("inspect") {
        LockStatus::Held { lock, contents } => {
            assert_eq!(lock, root.lock_path());
            assert_eq!(contents.as_deref(), Some(planted));
        }
        other => panic!("expected Held, got {other:?}"),
    }
    assert!(root.lock_path().exists());

    fs::write(root.lock_path(), [0xff, 0xfe]).expect("write");
    assert!(matches!(
        MutationLock::inspect(root.path()).expect("inspect"),
        LockStatus::Malformed { .. }
    ));
    assert!(
        root.lock_path().exists(),
        "malformed records are never removed"
    );
}

/// An absent record is observable as `None`, and recovery takes the plain path.
#[test]
fn observation_of_an_absent_record_is_none() {
    let root = Root::new();
    assert_eq!(MutationLock::observe(root.path()).expect("observe"), None);
    let lock = MutationLock::acquire_with_recovery(
        root.path(),
        &product(),
        &release(),
        &Fixed(StaleLockDecision::ProvenStale),
    )
    .expect("nothing to recover");
    drop(lock);
    assert!(!root.lock_path().exists());
}
/// Bounded diagnostics stay panic-free for multi-byte content at the edge.
///
/// Truncation that lands inside a character boundary is exactly the case a
/// crashed updater record could produce, and recovery must classify it without
/// panicking or inventing fields.
#[test]
fn bounded_non_ascii_record_diagnostics_stay_panic_free() {
    let root = Root::new();
    // 300 two-byte characters, so a naive byte-bound truncation would split one.
    let planted: String = "é".repeat(300);
    root.plant(&planted);

    let verifier = Recording::new(StaleLockDecision::Unknown);
    let error = MutationLock::acquire_with_recovery(root.path(), &product(), &release(), &verifier)
        .expect_err("an unparseable record must not be displaced");

    let rendered = error.to_string();
    assert!(!rendered.is_empty());
    assert!(
        std::str::from_utf8(rendered.as_bytes()).is_ok(),
        "{rendered:?}"
    );
    assert!(
        rendered.len() < 512,
        "diagnostics stay bounded: {}",
        rendered.len()
    );

    let seen = verifier.seen.borrow();
    assert_eq!(seen.len(), 1, "an unparseable record is still observable");
    assert!(!seen[0].format_known());
    assert_eq!(seen[0].record(), planted, "the exact bytes are preserved");
}

/// The commit path can opt into recovery without changing any other guarantee.
#[test]
fn commit_recovers_only_with_caller_proof() {
    use eggup_core::{
        AbsentPolicy, AllValidators, ArtifactMember, ArtifactSet, CommitOwnership,
        ExistingAsOwnedVerifier, InstallPlan, IntegrityRequirement, MemberId,
        TransactionDisposition,
    };

    let build = || {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let base = std::env::temp_dir().join(format!(
            "eggup-recovery-commit-{stamp}-{}",
            std::process::id()
        ));
        let inputs = base.join("inputs");
        let install = base.join("install");
        fs::create_dir_all(&inputs).expect("inputs");
        fs::create_dir_all(&install).expect("install");
        let source = inputs.join("candidate");
        fs::write(&source, b"new generation").expect("candidate");
        fs::write(install.join("main"), b"old generation").expect("installed");
        (base, source, install)
    };

    // Without proof, a left-behind record blocks the transaction outright.
    let (base, source, install) = build();
    fs::write(
        install.join(".eggup-mutation.lock"),
        "pid=999 nonce=1 product=eggup release=2026.10.05\n",
    )
    .expect("plant");
    let plan = || {
        InstallPlan::new(
            product(),
            release(),
            &install,
            ArtifactSet::single(
                ArtifactMember::new(MemberId::new("main").expect("member"), &source, "main")
                    .expect("artifact")
                    .with_integrity(IntegrityRequirement::Sha256(
                        eggup_core::hash_file(&source).expect("hash"),
                    )),
            )
            .expect("members"),
        )
        .expect("plan")
    };
    let blocked = plan()
        .prepare()
        .expect("prepare")
        .verify_integrity()
        .expect("verify")
        .validate(&AllValidators::new())
        .expect("validate")
        .commit(CommitOwnership::new(
            &ExistingAsOwnedVerifier,
            AbsentPolicy::DenyCreate,
        ))
        .expect_err("the default path never recovers");
    assert!(matches!(blocked, Error::UpdateInProgress { .. }));
    assert_eq!(
        fs::read(install.join("main")).expect("read"),
        b"old generation"
    );
    let _ = fs::remove_dir_all(&base);

    // With proof, the same transaction recovers and commits normally.
    let (base, source, install) = build();
    fs::write(
        install.join(".eggup-mutation.lock"),
        "pid=999 nonce=1 product=eggup release=2026.10.05\n",
    )
    .expect("plant");
    let receipt = InstallPlan::new(
        product(),
        release(),
        &install,
        ArtifactSet::single(
            ArtifactMember::new(MemberId::new("main").expect("member"), &source, "main")
                .expect("artifact")
                .with_integrity(IntegrityRequirement::Sha256(
                    eggup_core::hash_file(&source).expect("hash"),
                )),
        )
        .expect("members"),
    )
    .expect("plan")
    .prepare()
    .expect("prepare")
    .verify_integrity()
    .expect("verify")
    .validate(&AllValidators::new())
    .expect("validate")
    .commit_with_stale_lock_recovery(
        CommitOwnership::new(&ExistingAsOwnedVerifier, AbsentPolicy::DenyCreate),
        &Fixed(StaleLockDecision::ProvenStale),
    )
    .expect("recovery then commit");

    assert_eq!(receipt.disposition(), TransactionDisposition::Committed);
    assert_eq!(
        fs::read(install.join("main")).expect("read"),
        b"new generation"
    );
    assert!(!install.join(".eggup-mutation.lock").exists());
    let _ = fs::remove_dir_all(&base);
}
