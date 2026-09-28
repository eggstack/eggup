//! Multi-member verified transaction (bundle).
//!
//! The artifact set is the mutation unit: either every member commits or the
//! transaction rolls back. Run with
//! `cargo run -p eggup-core --example multi_member`.

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
    let base = temp_root("eggup-multi-member");
    let inputs = base.join("inputs");
    let root = base.join("install");
    fs::create_dir_all(&inputs)?;
    fs::create_dir_all(root.join("bin"))?;
    fs::write(inputs.join("main"), b"new-main")?;
    fs::write(inputs.join("helper"), b"new-helper")?;
    fs::write(root.join("bin/main"), b"old-main")?;
    fs::write(root.join("bin/helper"), b"old-helper")?;
    let main_digest = eggup_core::hash_file(&inputs.join("main"))?;
    let helper_digest = eggup_core::hash_file(&inputs.join("helper"))?;
    let members = ArtifactSet::new(vec![
        ArtifactMember::new(MemberId::new("main")?, inputs.join("main"), "bin/main")?
            .with_integrity(IntegrityRequirement::Sha256(main_digest)),
        ArtifactMember::new(
            MemberId::new("helper")?,
            inputs.join("helper"),
            "bin/helper",
        )?
        .with_integrity(IntegrityRequirement::Sha256(helper_digest)),
    ])?;
    let plan = InstallPlan::new(
        ProductId::new("bundle")?,
        ReleaseId::new("2.0.0")?,
        &root,
        members,
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
    assert_eq!(fs::read(root.join("bin/main"))?, b"new-main");
    assert_eq!(fs::read(root.join("bin/helper"))?, b"new-helper");
    let _ = fs::remove_dir_all(&base);
    println!("multi-member commit: {:?}", receipt.disposition());
    Ok(())
}
