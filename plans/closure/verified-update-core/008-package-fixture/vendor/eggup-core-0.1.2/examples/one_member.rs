//! One-member verified transaction.
//!
//! Demonstrates the supported path:
//! `InstallPlan -> prepare -> verify_integrity -> validate -> commit(ownership)`.
//!
//! Run with `cargo run -p eggup-core --example one_member`.

use eggup_core::{
    AbsentPolicy, AllValidators, ArtifactMember, ArtifactSet, CommitOwnership,
    ExistingAsOwnedVerifier, InstallPlan, IntegrityRequirement, MemberId, ProductId, ReleaseId,
    TransactionDisposition,
};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_root(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base = temp_root("eggup-one-member");
    let inputs = base.join("inputs");
    let root = base.join("install");
    fs::create_dir_all(inputs.join("bin"))?;
    fs::create_dir_all(root.join("bin"))?;
    fs::write(inputs.join("app"), b"new-app")?;
    let digest = eggup_core::hash_file(&inputs.join("app"))?;
    let member = ArtifactMember::new(MemberId::new("main")?, inputs.join("app"), "bin/app")?
        .with_integrity(IntegrityRequirement::Sha256(digest));
    let plan = InstallPlan::new(
        ProductId::new("demo")?,
        ReleaseId::new("1.0.0")?,
        &root,
        ArtifactSet::single(member)?,
    )?;
    let receipt = plan
        .prepare()?
        .verify_integrity()?
        .validate(&AllValidators::new())?
        .commit(CommitOwnership::new(
            &ExistingAsOwnedVerifier,
            AbsentPolicy::AllowCreate,
        ))?;
    assert_eq!(receipt.disposition(), TransactionDisposition::Committed);
    assert_eq!(fs::read(root.join("bin/app"))?, b"new-app");
    let _ = fs::remove_dir_all(&base);
    println!("one-member commit: {:?}", receipt.disposition());
    Ok(())
}
