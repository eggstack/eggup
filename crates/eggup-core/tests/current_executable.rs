//! Current-executable transaction parity: the self-update path is an ordinary
//! one-member transaction, and these tests prove it end to end on the running
//! platform.
//!
//! The interesting cases need a process that actually replaces the image it is
//! executing, so the suite drives a child process that re-executes this very
//! test binary from a private "installed" location. The parent asserts on real
//! filesystem state after the child exits, which is the only evidence that
//! holds equally on Unix and on Windows.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use eggup_core::{
    hash_file, AbsentPolicy, AllValidators, CommitOwnership, CurrentExecutable, Error,
    ExistingAsOwnedVerifier, InstallPlan, IntegrityRequirement, Ownership, OwnershipVerifier,
    PostCommitFailurePolicy, ProductId, ReleaseId, StagePlacement,
};

const CHILD_MODE: &str = "EGGUP_CORE_SELF_UPDATE_CHILD";
const CHILD_POLICY: &str = "EGGUP_CORE_SELF_UPDATE_POLICY";
const CHILD_CANDIDATE: &str = "EGGUP_CORE_SELF_UPDATE_CANDIDATE";
const CHILD_REPORT: &str = "EGGUP_CORE_SELF_UPDATE_REPORT";

static NEXT_DEPLOYMENT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// A private tree holding the "installed" image, its candidates, and reports.
struct Deployment {
    root: PathBuf,
}

impl Deployment {
    fn new() -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        // Tests run in parallel threads of one process, so the pid alone is not
        // unique; the sequence makes the name collision-free.
        let sequence = NEXT_DEPLOYMENT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "eggup-selfupdate-{nanos}-{sequence}-{}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("bin")).expect("create bin");
        fs::create_dir_all(root.join("candidates")).expect("create candidates");
        Self { root }
    }

    fn bin(&self) -> PathBuf {
        self.root.join("bin")
    }

    /// Copies this test binary to `bin/app` and marks it executable.
    fn install_self(&self) -> PathBuf {
        let target = self.bin().join("app");
        fs::copy(std::env::current_exe().expect("current exe"), &target).expect("copy self");
        make_executable(&target);
        target
    }

    /// Writes a candidate image that is byte-different from the installed one.
    fn candidate(&self, tag: &str) -> PathBuf {
        let path = self.root.join("candidates").join(format!("app-{tag}"));
        // A candidate that is still a runnable ELF/Mach-O/PE would be ideal, but
        // the transaction never executes the candidate: it only stages, hashes,
        // and renames it. Distinct trailing bytes are enough to prove
        // replacement happened.
        fs::write(&path, format!("candidate generation {tag}")).expect("write candidate");
        path
    }
}

impl Drop for Deployment {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).expect("chmod");
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) {}

#[derive(Debug, PartialEq, Eq)]
struct Report {
    disposition: String,
    cleanup: String,
    rollback_performed: bool,
    rollback_verified: bool,
    recovery_path_present: bool,
    failure_detail: Option<String>,
}

impl Report {
    fn parse(raw: &str) -> Self {
        let mut disposition = None;
        let mut cleanup = None;
        let mut rollback_performed = None;
        let mut rollback_verified = None;
        let mut recovery_path_present = None;
        let mut failure_detail = None;
        for line in raw.lines() {
            let (key, value) = line.split_once('=').expect("report line");
            match key {
                "disposition" => disposition = Some(value.to_string()),
                "cleanup" => cleanup = Some(value.to_string()),
                "rollback_performed" => rollback_performed = Some(value == "true"),
                "rollback_verified" => rollback_verified = Some(value == "true"),
                "recovery_path_present" => recovery_path_present = Some(value == "true"),
                "failure" => failure_detail = Some(value.to_string()),
                other => panic!("unexpected report key {other}"),
            }
        }
        Self {
            disposition: disposition.expect("disposition"),
            cleanup: cleanup.expect("cleanup"),
            rollback_performed: rollback_performed.expect("rollback_performed"),
            rollback_verified: rollback_verified.expect("rollback_verified"),
            recovery_path_present: recovery_path_present.expect("recovery_path_present"),
            failure_detail,
        }
    }
}

/// Runs the child once and returns its report plus the post-exit filesystem state.
fn run_child(
    deployment: &Deployment,
    policy: PostCommitFailurePolicy,
    launch_via: Option<&Path>,
) -> (Report, std::process::Output) {
    let candidate = deployment.candidate(if policy == PostCommitFailurePolicy::KeepInstalled {
        "next"
    } else {
        "rollback"
    });
    // The report lives in the executable's own directory: the authority test
    // seals everything above it, so nothing else would be writable.
    let report = deployment.bin().join("child-report.txt");
    let installed = deployment.bin().join("app");

    let invoked = launch_via.unwrap_or(&installed);
    let mut command = Command::new(invoked);
    command
        .arg("--exact")
        .arg("self_update_child_replaces_its_own_running_image")
        .arg("--nocapture")
        .env(CHILD_MODE, "1")
        .env(CHILD_POLICY, format!("{policy:?}"))
        .env(CHILD_CANDIDATE, &candidate)
        .env(CHILD_REPORT, &report);
    let output = command.output().expect("spawn installed child");
    let raw = fs::read_to_string(&report).unwrap_or_else(|error| {
        panic!(
            "child produced no report at {}: {error}\nstdout: {}\nstderr: {}",
            report.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (Report::parse(&raw), output)
}

struct FixedVerifier(Ownership);
impl std::fmt::Debug for FixedVerifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FixedVerifier").field(&self.0).finish()
    }
}
impl OwnershipVerifier for FixedVerifier {
    fn verify(&self, _member: &eggup_core::MemberId, _destination: &Path) -> Ownership {
        self.0
    }
}

/// The child half: it is running from `bin/app` and replaces that very file.
#[test]
fn self_update_child_replaces_its_own_running_image() {
    if std::env::var(CHILD_MODE).is_err() {
        // Invoked by the parent tests below.
        return;
    }
    let candidate = PathBuf::from(std::env::var(CHILD_CANDIDATE).expect("candidate env"));
    let report_path = PathBuf::from(std::env::var(CHILD_REPORT).expect("report env"));
    let policy = match std::env::var(CHILD_POLICY).expect("policy env").as_str() {
        "KeepInstalled" => PostCommitFailurePolicy::KeepInstalled,
        "RollBack" => PostCommitFailurePolicy::RollBack,
        other => panic!("unknown policy {other}"),
    };

    let current = CurrentExecutable::resolve().expect("resolve current executable");
    assert!(
        current.target().is_absolute(),
        "the bound target must be absolute"
    );
    let digest = hash_file(&candidate).expect("hash candidate");
    let plan = InstallPlan::for_current_executable(
        ProductId::new("eggup-self-update").expect("product"),
        ReleaseId::new("child").expect("release"),
        &candidate,
        &current,
        IntegrityRequirement::Sha256(digest),
    )
    .expect("current-executable plan");

    assert_eq!(
        plan.stage_placement(),
        StagePlacement::InsideInstallationRoot,
        "a self-update must never need authority above the executable's directory"
    );
    assert_eq!(
        plan.installation_root(),
        current.installation_root(),
        "the installation root is the executable's own directory"
    );

    let receipt = plan
        .prepare()
        .expect("prepare")
        .verify_integrity()
        .expect("verify")
        .validate(&AllValidators::new())
        .expect("validate")
        .commit_with_post_commit(
            CommitOwnership::new(&ExistingAsOwnedVerifier, AbsentPolicy::DenyCreate),
            policy,
            || -> std::result::Result<(), String> {
                if policy == PostCommitFailurePolicy::RollBack {
                    Err("child forced rollback".to_string())
                } else {
                    Ok(())
                }
            },
        )
        .expect("commit");

    let rendered = format!(
        "disposition={:?}\ncleanup={:?}\nrollback_performed={}\nrollback_verified={}\nrecovery_path_present={}\nfailure={}\n",
        receipt.disposition(),
        receipt.cleanup(),
        receipt.rollback_performed(),
        receipt.rollback_verified(),
        receipt.recovery_path().is_some(),
        receipt
            .failure()
            .map(|report| report.detail().to_string())
            .unwrap_or_else(|| "<none>".to_string()),
    );
    fs::write(&report_path, rendered).expect("write report");
}

/// The whole transaction succeeds with write authority limited to the
/// executable's own directory.
///
/// This is the central claim of the corrective: a self-updater needs to write
/// where its executable lives, never in that directory's parent.
#[cfg(unix)]
#[test]
fn self_update_needs_no_authority_above_the_executable_directory() {
    let deployment = Deployment::new();
    let installed = deployment.install_self();
    let original = fs::read(&installed).expect("read installed");
    let parent = deployment.bin().parent().expect("bin parent").to_path_buf();

    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&parent, fs::Permissions::from_mode(0o500)).expect("seal parent");

    // A sealed parent must genuinely block writes before the assertion below
    // means anything; a privileged test runner cannot express the constraint.
    if fs::create_dir(parent.join("probe")).is_ok() {
        fs::remove_dir(parent.join("probe")).ok();
        fs::set_permissions(&parent, fs::Permissions::from_mode(0o700)).expect("restore parent");
        eprintln!("skipping: this runner can write a 0500 directory");
        return;
    }

    let (report, output) = run_child(&deployment, PostCommitFailurePolicy::KeepInstalled, None);
    fs::set_permissions(&parent, fs::Permissions::from_mode(0o700)).expect("restore parent");

    assert!(
        output.status.success(),
        "child failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(report.disposition, "Committed");
    assert!(!report.rollback_performed);
    assert_ne!(
        fs::read(&installed).expect("read installed"),
        original,
        "the installed image must have been replaced"
    );
    assert!(
        leftover_transaction_state(&deployment.bin()).is_empty(),
        "a kept-installed self-update leaves no transaction state behind on this platform: {:?}",
        leftover_transaction_state(&deployment.bin())
    );
}

/// `KeepInstalled` leaves the new generation live and records a truthful cleanup
/// state, including the Windows case where the old image cannot be unlinked
/// while this process is still executing it.
#[test]
fn keep_installed_leaves_the_new_generation_live() {
    let deployment = Deployment::new();
    let installed = deployment.install_self();

    let (report, output) = run_child(&deployment, PostCommitFailurePolicy::KeepInstalled, None);
    assert!(
        output.status.success(),
        "child failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(report.disposition, "Committed");
    assert!(!report.rollback_performed);
    assert!(
        matches!(report.cleanup.as_str(), "Cleaned" | "DeferredToProcessExit"),
        "unexpected cleanup state {}",
        report.cleanup
    );
    assert_eq!(
        fs::read(&installed).expect("read installed"),
        fs::read(deployment.candidate("next")).expect("read candidate"),
        "the original path must hold the new generation after the child exits"
    );
}

/// `RollBack` restores a byte-identical old generation after the child exits.
#[test]
fn rollback_restores_a_byte_identical_old_generation() {
    let deployment = Deployment::new();
    let installed = deployment.install_self();
    let original = fs::read(&installed).expect("read installed");

    let (report, output) = run_child(&deployment, PostCommitFailurePolicy::RollBack, None);
    assert!(
        output.status.success(),
        "child failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(report.disposition, "RolledBack");
    assert!(report.rollback_performed);
    assert!(report.rollback_verified);
    assert_eq!(
        report.failure_detail.as_deref(),
        Some("child forced rollback"),
        "the receipt must name the post-commit failure that triggered rollback"
    );
    assert_eq!(
        fs::read(&installed).expect("read installed"),
        original,
        "rollback must restore the previous generation byte for byte"
    );
}

/// An invocation symlink is followed to its real target; the link object itself
/// is never overwritten.
#[cfg(unix)]
#[test]
fn symlink_invocation_updates_the_real_image_and_preserves_the_link() {
    let deployment = Deployment::new();
    let installed = deployment.install_self();
    let link = deployment.bin().join("app-link");
    std::os::unix::fs::symlink(&installed, &link).expect("symlink");

    let (report, output) = run_child(
        &deployment,
        PostCommitFailurePolicy::KeepInstalled,
        Some(&link),
    );
    assert!(
        output.status.success(),
        "child failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(report.disposition, "Committed");

    let link_metadata = fs::symlink_metadata(&link).expect("stat link");
    assert!(
        link_metadata.file_type().is_symlink(),
        "the invocation symlink must still be a symlink, not a replaced regular file"
    );
    assert_eq!(
        fs::read_link(&link).expect("read link"),
        installed,
        "the invocation symlink must still point at the same real image"
    );
    assert_eq!(
        fs::read(&installed).expect("read installed"),
        fs::read(deployment.candidate("next")).expect("read candidate"),
        "the real image, not the link, is what got replaced"
    );
}

/// A hard-linked image has other names whose contents would change with it, so
/// exact identity is not provable and the binding refuses it.
#[cfg(unix)]
#[test]
fn hard_linked_current_executable_is_refused() {
    let deployment = Deployment::new();
    let installed = deployment.install_self();
    let alias = deployment.bin().join("app-alias");
    fs::hard_link(&installed, &alias).expect("hard link");

    let error = CurrentExecutable::bind(&installed).expect_err("hard-linked image must be refused");
    assert!(
        matches!(error, Error::DestinationConflict { .. }),
        "expected a destination conflict, got {error:?}"
    );
    fs::remove_file(&alias).expect("remove alias");
    CurrentExecutable::bind(&installed).expect("single-linked image binds");
}

/// A destination replaced between binding and commit is refused before any live
/// mutation: the new object is not the executable that was proven.
#[test]
fn destination_replaced_after_binding_fails_closed() {
    let deployment = Deployment::new();
    let installed = deployment.install_self();
    let candidate = deployment.candidate("swap");
    let current = CurrentExecutable::bind(&installed).expect("bind");
    let digest = hash_file(&candidate).expect("hash");
    let plan = InstallPlan::for_current_executable(
        ProductId::new("eggup").expect("product"),
        ReleaseId::new("r1").expect("release"),
        &candidate,
        &current,
        IntegrityRequirement::Sha256(digest),
    )
    .expect("plan");

    // Swap the object behind the exact path after the identity was proven.
    fs::remove_file(&installed).expect("remove");
    fs::write(&installed, b"an impostor at the same path").expect("write impostor");

    let receipt = plan
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
        .expect("an ownership failure is a receipt, never an Err");

    assert_eq!(
        receipt.disposition(),
        eggup_core::TransactionDisposition::RolledBack,
        "a swapped image must fail closed before any live mutation"
    );
    assert!(!receipt.rollback_performed());
    assert_eq!(
        fs::read(&installed).expect("read installed"),
        b"an impostor at the same path",
        "the impostor is not this transaction's image and must be left alone"
    );
}

/// Ownership the caller cannot prove fails closed before any live mutation,
/// exactly as for an ordinary destination.
#[test]
fn foreign_current_executable_is_never_replaced() {
    let deployment = Deployment::new();
    let installed = deployment.install_self();
    let original = fs::read(&installed).expect("read");
    let candidate = deployment.candidate("foreign");
    let current = CurrentExecutable::bind(&installed).expect("bind");
    let digest = hash_file(&candidate).expect("hash");

    let receipt = InstallPlan::for_current_executable(
        ProductId::new("eggup").expect("product"),
        ReleaseId::new("r1").expect("release"),
        &candidate,
        &current,
        IntegrityRequirement::Sha256(digest),
    )
    .expect("plan")
    .prepare()
    .expect("prepare")
    .verify_integrity()
    .expect("verify")
    .validate(&AllValidators::new())
    .expect("validate")
    .commit(CommitOwnership::new(
        &FixedVerifier(Ownership::Foreign),
        AbsentPolicy::DenyCreate,
    ))
    .expect("typed receipt, never an Err");

    assert_eq!(
        receipt.disposition(),
        eggup_core::TransactionDisposition::RolledBack,
        "a foreign destination fails closed with a receipt"
    );
    assert_eq!(fs::read(&installed).expect("read"), original);
}

/// Integrity is proved before anything is live: a wrong candidate digest never
/// reaches the commit path.
#[test]
fn wrong_candidate_digest_fails_before_mutation() {
    let deployment = Deployment::new();
    let installed = deployment.install_self();
    let original = fs::read(&installed).expect("read");
    let candidate = deployment.candidate("wrong-digest");
    let current = CurrentExecutable::bind(&installed).expect("bind");

    let error = InstallPlan::for_current_executable(
        ProductId::new("eggup").expect("product"),
        ReleaseId::new("r1").expect("release"),
        &candidate,
        &current,
        IntegrityRequirement::Sha256([0u8; 32]),
    )
    .expect("plan")
    .prepare()
    .expect("prepare")
    .verify_integrity()
    .expect_err("a mismatched digest must not verify");

    assert!(matches!(error, Error::VerificationFailed(_)), "{error:?}");
    assert_eq!(fs::read(&installed).expect("read"), original);
}

/// Ordinary one-member and multi-member transactions keep their existing stage
/// placement: nothing about the generic path changes to support self-update.
#[test]
fn ordinary_plans_keep_sibling_stage_placement() {
    let inputs = std::env::temp_dir().join(format!("eggup-placement-{}", std::process::id()));
    let install = inputs.join("install");
    fs::create_dir_all(&install).expect("install dir");
    fs::write(inputs.join("candidate"), b"bytes").expect("candidate");

    let plan = InstallPlan::new(
        ProductId::new("eggup").expect("product"),
        ReleaseId::new("r1").expect("release"),
        &install,
        eggup_core::ArtifactSet::single(
            eggup_core::ArtifactMember::new(
                eggup_core::MemberId::new("main").expect("member"),
                inputs.join("candidate"),
                "bin/eggup",
            )
            .expect("artifact"),
        )
        .expect("members"),
    )
    .expect("plan");

    assert_eq!(
        plan.stage_placement(),
        StagePlacement::SiblingOfInstallationRoot
    );
    assert!(plan.current_executable().is_none());
    let _ = fs::remove_dir_all(&inputs);
}

/// Names any transaction-owned state still sitting in `bin` after a completed
/// self-update.
fn leftover_transaction_state(bin: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(bin) else {
        return out;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with(".eggup-") {
            out.push(name);
        }
    }
    out
}
