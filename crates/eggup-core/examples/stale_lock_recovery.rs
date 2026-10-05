//! Proof-authorized stale-lock recovery.
//!
//! Core owns safe lock mutation and refuses to decide staleness on its own. This
//! example shows the shape a consumer implements: it inspects what Core can
//! prove about the record, combines that with deployment evidence only it has,
//! and returns `ProvenStale` only when the combination is conclusive.
//!
//! The evidence is deliberately deployment-shaped rather than universal. A real
//! consumer would additionally consult its own service manager, its own lock
//! semantics, or an operator-provisioned deadline. What matters for this crate's
//! contract is the shape:
//!
//! - `Active` and `Unknown` retain the record and fail closed.
//! - Only `ProvenStale` authorizes recovery, and only for the exact observed
//!   record.
//! - Core never consults process liveness, record age, executable name, or
//!   service state on the verifier's behalf.
//!
//! Run with: `cargo run -p eggup-core --example stale_lock_recovery`

use eggup_core::{
    LockObservation, MutationLock, ProductId, ReleaseId, StaleLockDecision, StaleLockVerifier,
};

/// Evidence a consumer gathers from its own deployment.
///
/// `supervised` is what this consumer is *responsible for`; `alive` is what it
/// can currently *see*. Keeping them apart is the point: a pid the consumer
/// never supervised cannot be reasoned about at all, and a supervised pid it can
/// no longer see is merely suggestive on its own.
#[derive(Debug)]
struct DeploymentEvidence {
    product: ProductId,
    supervised: Vec<u32>,
    alive: Vec<u32>,
    /// Whether the deployment has an out-of-band statement that it is idle.
    declared_idle: bool,
}

/// A consumer verifier that combines a record with its own evidence.
///
/// Note what it does *not* do: it does not treat "the pid is not alive" as proof
/// by itself. Pids are reused, so liveness is corroboration, never authority.
struct DeploymentVerifier {
    evidence: DeploymentEvidence,
}

impl std::fmt::Debug for DeploymentVerifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeploymentVerifier")
            .field("evidence", &self.evidence)
            .finish()
    }
}

impl StaleLockVerifier for DeploymentVerifier {
    fn classify(&self, observed: &LockObservation) -> StaleLockDecision {
        // The record must belong to this consumer's domain before its fields
        // mean anything. Core observed the bytes; attributing them is the
        // caller's job.
        if observed.product() != Some(self.evidence.product.as_str()) {
            return StaleLockDecision::Unknown;
        }
        // An unrecognised record format carries no attributable evidence.
        if !observed.format_known() {
            return StaleLockDecision::Unknown;
        }
        let Some(pid) = observed.pid() else {
            return StaleLockDecision::Unknown;
        };
        // A supervised process this consumer can still see is plainly active.
        if self.evidence.alive.contains(&pid) {
            return StaleLockDecision::Active;
        }
        // A pid this consumer never supervised is ambiguous: the number may now
        // belong to an unrelated process entirely.
        if !self.evidence.supervised.contains(&pid) {
            return StaleLockDecision::Unknown;
        }
        // Conclusive only with the deployment's own out-of-band statement.
        if self.evidence.declared_idle {
            StaleLockDecision::ProvenStale
        } else {
            StaleLockDecision::Unknown
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root =
        std::env::temp_dir().join(format!("eggup-stale-lock-example-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root)?;

    let product = ProductId::new("example-daemon")?;
    let release = ReleaseId::new("2026.10.05")?;
    let lock_path = root.join(".eggup-mutation.lock");

    // A crashed updater left this record behind.
    std::fs::write(
        &lock_path,
        "pid=4242 nonce=9 product=example-daemon release=2026.10.05\n",
    )?;

    println!("default acquisition never recovers:");
    let error = MutationLock::acquire(&root, &product, &release).expect_err("record exists");
    println!("  {error}\n");

    // The consumer supervises pid 4242, can no longer see it, and the deployment
    // is declared idle. That combination is its proof.
    let conclusive = DeploymentVerifier {
        evidence: DeploymentEvidence {
            product: product.clone(),
            supervised: vec![4242],
            alive: Vec::new(),
            declared_idle: true,
        },
    };
    match MutationLock::acquire_with_recovery(&root, &product, &release, &conclusive) {
        Ok(lock) => {
            println!("recovered after caller proof:");
            println!("  lock record: {}", lock.path().display());
            println!("  our own record is removed when it is dropped");
        }
        Err(error) => println!("retained the record: {error}"),
    }

    // Contrast: the same record without the deployment's out-of-band statement is
    // ambiguous, so it is left exactly where it is.
    let _ = std::fs::remove_file(&lock_path);
    std::fs::write(
        &lock_path,
        "pid=5150 nonce=1 product=example-daemon release=2026.10.05\n",
    )?;
    let cautious = DeploymentVerifier {
        evidence: DeploymentEvidence {
            product: product.clone(),
            supervised: vec![5150],
            alive: Vec::new(),
            declared_idle: false,
        },
    };
    let error = MutationLock::acquire_with_recovery(&root, &product, &release, &cautious)
        .expect_err("no out-of-band proof");
    println!("\nwithout out-of-band proof:");
    println!("  {error}");
    println!("  record still present: {}", lock_path.exists());

    // And a record belonging to a different deployment is never claimed.
    let _ = std::fs::remove_file(&lock_path);
    std::fs::write(
        &lock_path,
        "pid=6060 nonce=1 product=somebody-else release=2026.10.05\n",
    )?;
    let foreign = DeploymentVerifier {
        evidence: DeploymentEvidence {
            product: ProductId::new("example-daemon")?,
            supervised: vec![6060],
            alive: Vec::new(),
            declared_idle: true,
        },
    };
    let error = MutationLock::acquire_with_recovery(&root, &product, &release, &foreign)
        .expect_err("foreign record");
    println!("\nfor another deployment's record:");
    println!("  {error}");
    println!("  record still present: {}", lock_path.exists());

    let _ = std::fs::remove_dir_all(&root);
    Ok(())
}
