//! One-member verified transaction.
//!
//! Demonstrates the supported path:
//! `InstallPlan -> prepare -> verify_integrity -> validate -> commit(ownership)`.
//!
//! This file is the source of the quickstart snippet in `README.md` and
//! `docs/quickstart.md`, which reproduce it verbatim. Keep the three in sync:
//! the gate builds and runs this example, so a divergence is a wrong doc.
//!
//! Run with `cargo run -p eggup-core --example one_member`.
//! Output: `committed: Committed`

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
    let base = temp_root("eggup-quickstart");
    let (inputs, root) = (base.join("inputs"), base.join("install"));
    fs::create_dir_all(inputs.join("bin"))?;
    fs::create_dir_all(root.join("bin"))?;
    fs::write(inputs.join("app"), b"new-app")?;

    // 1. Describe one member, with the SHA-256 you expect to verify.
    let source = inputs.join("app");
    let digest = eggup_core::hash_file(&source)?;
    let member = ArtifactMember::new(MemberId::new("main")?, source, "bin/app")?
        .with_integrity(IntegrityRequirement::Sha256(digest));

    // 2. Name the product, release, install root, and artifact set.
    let plan = InstallPlan::new(
        ProductId::new("demo")?,
        ReleaseId::new("1.0.0")?,
        &root,
        ArtifactSet::single(member)?,
    )?;

    // 3. Stage, verify, validate, then commit with proven ownership.
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
    println!("committed: {:?}", receipt.disposition());
    Ok(())
}
