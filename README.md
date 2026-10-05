# Eggup

Eggup is a Rust library for safe, verified local updates of one or more
application artifacts. Its core owns staging, SHA-256 integrity verification,
bounded candidate validation, destination-ownership revalidation, locking,
replacement, rollback, and recovery evidence — as a set of composable crates
that stay out of each other's way.

Integrity is checksum evidence only. Eggup makes **no authenticity or signature
claim** anywhere.

## Quickstart

Requires Rust **1.89**+ (edition 2021). The only dependency is `sha2`.

```toml
[dependencies]
eggup-core = "0.1.2"
```

```rust

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
```

Each step returns the next type, so a committed transaction is only reachable by
passing all five:

| Step | Guarantees |
|---|---|
| `.prepare()` | Bytes staged in an owner-private directory (`0700`/`0600` on Unix) |
| `.verify_integrity()` | Staged bytes match the declared SHA-256 |
| `.validate(&v)` | Your candidate check passed — **every** member must be `Verified` |
| `.commit(ownership)` | Ownership re-proven and digests re-hashed **under the lock** |

Run it: `cargo run -p eggup-core --example one_member` → `committed: Committed`

The core never creates the destination's parent directory, and a commit is only
a success once the receipt says `Committed` — `RolledBack` and
`RecoveryRequired` are real outcomes, not errors.

## The crates

| Crate | Role |
|---|---|
| [`eggup-core`](crates/eggup-core) | Policy-neutral local transaction mechanics. Sole dep: `sha2` |
| [`eggup-acquisition`](crates/eggup-acquisition) | Transport-neutral seam, bounds, fixtures, composition. **Zero dependencies** |
| [`eggup-eggfetch`](crates/eggup-eggfetch) | Native HTTP adapter (in-process Rustls) |
| [`eggup-curl`](crates/eggup-curl) | External-`curl` adapter, no embedded HTTP/TLS stack |
| [`eggup-archive`](crates/eggup-archive) | Bounded allowlisted extraction of already-verified tar.gz/zip |
| [`eggup-eggpack`](crates/eggup-eggpack) | Optional Eggpack ReleaseManifest v1 adapter (leaf) |
| [`eggup-service`](crates/eggup-service) | Manager-neutral service lifecycle: systemd, launchd, cron, Windows SCM |
| `eggup-transport-footprint` | `publish = false` fixtures measuring which transport stack you link |

`eggup-curl` and `eggup-eggfetch` are interchangeable: both implement
`AcquisitionTransport`, neither may know the other exists, and composition lives
in the seam. Choosing between them is a footprint and trust decision.

## Documentation

| Document | Use it for |
|---|---|
| [docs/quickstart.md](docs/quickstart.md) | The walkthrough above, expanded: all five runnable examples, receipts, post-commit checks |
| [docs/crates.md](docs/crates.md) | Which crate(s) to depend on, and what each will not do |
| [docs/releases.md](docs/releases.md) | Published versions, the exact-pin cascade, and known limitations in published releases |
| [architecture/overview.md](architecture/overview.md) | Dependency graph, module map, cross-cutting invariants |
| [crates/eggup-core/docs/](crates/eggup-core/docs/transaction.md) | The authoritative contracts — domain, verification, transaction |
| [AGENTS.md](AGENTS.md) | Index for coding agents working in this workspace |

`crates/eggup-core/docs/` and `architecture/` are normative. `docs/` is
user-facing guidance and never overrides them.

## Publication state

`eggup-core`, `eggup-archive`, `eggup-acquisition`, `eggup-eggfetch`,
`eggup-eggpack`, and `eggup-curl` are published at `0.1.2`. `eggup-service` is
published but lags at `0.1.1`, so its `0.1.2` has never been published.
`eggup-transport-footprint` is `publish = false` by design. CI never publishes;
releases are manual and milestone-gated.

Note that published releases trail the working tree — see
[docs/releases.md](docs/releases.md) for the limitations you inherit at `0.1.2`.
