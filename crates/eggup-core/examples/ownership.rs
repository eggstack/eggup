//! Ownership verifier.
//!
//! Destructive replacement requires `Owned`, or `Absent` with explicit
//! creation authorization. This example proves ownership by exact prior
//! content and forbids creation of new paths. Run with
//! `cargo run -p eggup-core --example ownership`.

use eggup_core::{
    AbsentPolicy, AllValidators, ArtifactMember, ArtifactSet, CommitOwnership, ExactDigestVerifier,
    InstallPlan, IntegrityRequirement, MemberId, ProductId, ReleaseId, TransactionDisposition,
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
    let base = temp_root("eggup-ownership");
    let inputs = base.join("inputs");
    let root = base.join("install");
    fs::create_dir_all(&inputs)?;
    fs::create_dir_all(root.join("bin"))?;
    fs::write(inputs.join("app"), b"new-app")?;
    fs::write(root.join("bin/app"), b"old-app")?;
    let new_digest = eggup_core::hash_file(&inputs.join("app"))?;
    let old_digest = eggup_core::hash_file(&root.join("bin/app"))?;
    let member = ArtifactMember::new(MemberId::new("main")?, inputs.join("app"), "bin/app")?
        .with_integrity(IntegrityRequirement::Sha256(new_digest));
    let plan = InstallPlan::new(
        ProductId::new("demo")?,
        ReleaseId::new("4.0.0")?,
        &root,
        ArtifactSet::single(member)?,
    )?;
    // Prove the live file is the expected old deployment; deny creation of
    // absent destinations (replacement-only policy).
    let verifier = ExactDigestVerifier::new(vec![(MemberId::new("main")?, old_digest)]);
    let receipt = plan
        .prepare()?
        .verify_integrity()?
        .validate(&AllValidators::new())?
        .commit(CommitOwnership::new(&verifier, AbsentPolicy::DenyCreate))?;
    assert_eq!(receipt.disposition(), TransactionDisposition::Committed);
    assert_eq!(fs::read(root.join("bin/app"))?, b"new-app");
    let _ = fs::remove_dir_all(&base);
    println!("ownership-verified commit: {:?}", receipt.disposition());
    Ok(())
}
