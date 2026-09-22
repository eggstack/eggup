//! Custom candidate validator.
//!
//! Shows how a consumer supplies its own identity check without changing
//! Eggup. The validator below requires the staged file to contain an exact
//! marker. Run with `cargo run -p eggup-core --example custom_validator`.

use eggup_core::{
    AbsentPolicy, ArtifactMember, ArtifactSet, CandidateValidator, CommitOwnership,
    ExistingAsOwnedVerifier, InstallPlan, IntegrityRequirement, MemberId, ProductId, ReleaseId,
    TransactionDisposition, VerifiedTransaction,
};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
struct ContainsMarker {
    member: MemberId,
    marker: Vec<u8>,
}

impl CandidateValidator for ContainsMarker {
    fn validate(&self, transaction: &VerifiedTransaction) -> eggup_core::Result<()> {
        let path = transaction.staged_path(&self.member)?;
        let bytes = fs::read(&path).map_err(|e| {
            eggup_core::Error::CandidateExecution(format!("reading candidate: {e}"))
        })?;
        if bytes.windows(self.marker.len()).any(|w| w == self.marker) {
            Ok(())
        } else {
            Err(eggup_core::Error::CandidateExecution(
                "marker not found in candidate".into(),
            ))
        }
    }
}

fn temp_root(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base = temp_root("eggup-custom-validator");
    let inputs = base.join("inputs");
    let root = base.join("install");
    fs::create_dir_all(&inputs)?;
    fs::create_dir_all(root.join("bin"))?;
    fs::write(inputs.join("app"), b"release-marker-42")?;
    let digest = eggup_core::hash_file(&inputs.join("app"))?;
    let member = ArtifactMember::new(MemberId::new("main")?, inputs.join("app"), "bin/app")?
        .with_integrity(IntegrityRequirement::Sha256(digest));
    let plan = InstallPlan::new(
        ProductId::new("demo")?,
        ReleaseId::new("3.0.0")?,
        &root,
        ArtifactSet::single(member)?,
    )?;
    let validator = ContainsMarker {
        member: MemberId::new("main")?,
        marker: b"marker-42".to_vec(),
    };
    let receipt = plan
        .prepare()?
        .verify_integrity()?
        .validate(&validator)?
        .commit(CommitOwnership::new(
            &ExistingAsOwnedVerifier,
            AbsentPolicy::AllowCreate,
        ))?;
    assert_eq!(receipt.disposition(), TransactionDisposition::Committed);
    let _ = fs::remove_dir_all(&base);
    println!("custom validator commit: {:?}", receipt.disposition());
    Ok(())
}
