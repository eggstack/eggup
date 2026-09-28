//! Interpreting terminal receipts.
//!
//! A rolled-back or recovery-required result is never equivalent to success.
//! This example shows the three dispositions and how to read the structured
//! failure report. Crash durability beyond process-level rollback is not
//! claimed; stale locks require manual operator action. Run with
//! `cargo run -p eggup-core --example receipts`.

use eggup_core::{
    AbsentPolicy, AllValidators, ArtifactMember, ArtifactSet, CommitOwnership,
    ExistingAsOwnedVerifier, InstallPlan, IntegrityRequirement, MemberId, Ownership,
    OwnershipVerifier, PostCommitFailurePolicy, ProductId, ReleaseId, TransactionDisposition,
};
use std::fmt::Debug;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
struct DenyAll;
impl OwnershipVerifier for DenyAll {
    fn verify(&self, _member: &MemberId, _destination: &Path) -> Ownership {
        Ownership::Foreign
    }
}

fn temp_root(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()))
}

fn build(
    new: &[u8],
    old: Option<&[u8]>,
) -> Result<(PathBuf, InstallPlan), Box<dyn std::error::Error>> {
    let base = temp_root("eggup-receipts");
    let inputs = base.join("inputs");
    let root = base.join("install");
    fs::create_dir_all(&inputs)?;
    fs::create_dir_all(root.join("bin"))?;
    fs::write(inputs.join("app"), new)?;
    if let Some(old) = old {
        fs::write(root.join("bin/app"), old)?;
    }
    let digest = eggup_core::hash_file(&inputs.join("app"))?;
    let member = ArtifactMember::new(MemberId::new("main")?, inputs.join("app"), "bin/app")?
        .with_integrity(IntegrityRequirement::Sha256(digest));
    let plan = InstallPlan::new(
        ProductId::new("demo")?,
        ReleaseId::new("5.0.0")?,
        &root,
        ArtifactSet::single(member)?,
    )?;
    // Leak the base via the plan's root; caller cleans `root.parent()`.
    Ok((base, plan))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Committed: success.
    let (base, plan) = build(b"new", Some(b"old"))?;
    let receipt = plan
        .prepare()?
        .verify_integrity()?
        .validate(&AllValidators::new())?
        .commit(CommitOwnership::new(
            &ExistingAsOwnedVerifier,
            AbsentPolicy::AllowCreate,
        ))?;
    assert_eq!(receipt.disposition(), TransactionDisposition::Committed);
    assert!(receipt.failure().is_none());
    println!("committed: {:?}", receipt.disposition());
    let _ = fs::remove_dir_all(&base);

    // 2. RolledBack: ownership denied, live state unchanged, failure explains why.
    let (base, plan) = build(b"new", Some(b"old"))?;
    let root = base.join("install");
    let receipt = plan
        .prepare()?
        .verify_integrity()?
        .validate(&AllValidators::new())?
        .commit(CommitOwnership::new(&DenyAll, AbsentPolicy::AllowCreate))?;
    assert_eq!(receipt.disposition(), TransactionDisposition::RolledBack);
    assert!(receipt.rollback_verified());
    let failure = receipt.failure().expect("rolled back carries a cause");
    println!(
        "rolled back: phase={} category={} detail={}",
        failure.phase().as_str(),
        failure.category().as_str(),
        failure.detail()
    );
    assert_eq!(fs::read(root.join("bin/app"))?, b"old");
    assert_ne!(
        receipt.disposition(),
        TransactionDisposition::Committed,
        "rollback is not success"
    );
    let _ = fs::remove_dir_all(&base);

    // 3. RecoveryRequired is distinct and high severity: it means Eggup cannot
    // prove either generation is coherent. It is produced by rollback failure
    // or unclean finalization; operators must inspect `recovery_path` and the
    // preserved lock. See `docs/transaction.md` for the full contract.
    println!("recovery-required: inspect recovery_path + lock; do not retry blindly");

    // 4. A failed post-commit check can keep the coherent new generation,
    // while remaining machine-readable as a failed check.
    let (base, plan) = build(b"new", Some(b"old"))?;
    let receipt = plan
        .prepare()?
        .verify_integrity()?
        .validate(&AllValidators::new())?
        .commit_with_post_commit(
            CommitOwnership::new(&ExistingAsOwnedVerifier, AbsentPolicy::AllowCreate),
            PostCommitFailurePolicy::KeepInstalled,
            || Err("example health check failed"),
        )?;
    assert_eq!(receipt.disposition(), TransactionDisposition::Committed);
    assert!(receipt.post_commit_failure().is_some());
    println!(
        "kept installed after failed check: {:?}",
        receipt.disposition()
    );
    let _ = fs::remove_dir_all(base);
    Ok(())
}
