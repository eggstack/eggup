# Quickstart

A complete, runnable first transaction. Everything below was executed against
this workspace; the program is a verbatim copy of
[`crates/eggup-core/examples/one_member.rs`](../crates/eggup-core/examples/one_member.rs),
and the printed line is its real output.

## Requirements

- Rust **1.89** or newer (the declared `rust-version`; verified by CI's `msrv`
  lane). Edition 2021.
- No extra features or build flags. `eggup-core`'s only dependency is `sha2`.

## 1. Add the dependency

```toml
[dependencies]
eggup-core = "0.1.2"
```

Inside this workspace, use a path dependency instead:

```toml
eggup-core = { path = "../crates/eggup-core" }
```

## 2. Run the example

The fastest way to see a real transaction is to run the checked-in example:

```sh
cargo run -p eggup-core --example one_member
```

```
committed: Committed
```

## 3. What that example does

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

The program is also verified from *outside* the workspace, which proves every
name above is part of the public API rather than an internal detail.

## 4. The five steps

The type chain is the API's spine: each step returns the next type, so a
committed transaction is only reachable by passing all five.

| Step | Returns | What it guarantees |
|---|---|---|
| `InstallPlan::new(..)` | `InstallPlan` | A described, not-yet-touched install |
| `.prepare()` | `PreparedTransaction` | Bytes staged in an owner-private directory (`0700`/`0600` on Unix) |
| `.verify_integrity()` | `VerifiedTransaction` | Each member's staged bytes match the declared SHA-256 |
| `.validate(&validator)` | `ValidatedTransaction` | Your candidate check passed |
| `.commit(ownership)` | `TransactionReceipt` | Ownership re-proven and staged digests re-hashed **under the lock** |

Three rules are worth internalising before you build on this:

1. **`validate` requires every member to be `Verified`.** A member declared
   `IntegrityRequirement::None` always fails here — there is no escape hatch
   for skipping verification.
2. **The core never creates the destination's parent directory.** Create it
   yourself; a missing parent is an error, not something Eggup will add.
3. **A commit is not a success until you check the receipt.**
   `TransactionDisposition` is `Committed`, `RolledBack`, or `RecoveryRequired`,
   and only the first means the install happened.

## 5. Ownership is caller-proven

`commit` takes a `CommitOwnership`, because the core will not guess who owns a
destination. `ExistingAsOwnedVerifier` (above) reports `Owned` for an existing
regular file and `Foreign` otherwise. Pair it with `AbsentPolicy::AllowCreate`
to permit creating a new path, or `DenyCreate` to forbid it.

`Foreign` and `Unknown` **fail closed with a receipt**, not an `Err` — see
[Reading receipts](#6-reading-receipts). `Err` is reserved for lock contention
and setup failure.

## 6. Reading receipts

`receipts.rs` demonstrates all three dispositions, including the failure paths:

```sh
cargo run -p eggup-core --example receipts
```

```
committed: Committed
rolled back: phase=ownership category=ownership-conflict detail=destination conflict at <TMPDIR>/eggup-receipts-<pid>-<nanos>/install/bin/app
recovery-required: inspect recovery_path + lock; do not retry blindly
kept installed after failed check: Committed
```

(The temp path is elided; the real one is platform-specific.)

`RecoveryRequired` is the one that needs operator attention: a real recovery
path exists and a stale lock may remain. Eggup does not clear stale locks for
you — `MutationLock::inspect` is read-only by design.

## 7. After a post-install check

If you need to verify the install *after* the files land, use
`commit_with_post_commit`. It keeps the lock and the rollback set alive across
one caller-supplied check, then applies your `PostCommitFailurePolicy`:

- `KeepInstalled` — the check failed; leave the install in place.
- `RollBack` — the check failed; restore the backup set.

The check is a `FnOnce() -> Result<(), E>` closure, not a trait object or a
borrowed reference. Passing anything that returns `Ok` leaves the install
committed and `post_commit_failure()` as `None`:

```rust
let receipt = plan
    .prepare()?
    .verify_integrity()?
    .validate(&AllValidators::new())?
    .commit_with_post_commit(
        CommitOwnership::new(&ExistingAsOwnedVerifier, AbsentPolicy::AllowCreate),
        PostCommitFailurePolicy::KeepInstalled,
        || Err("health check failed"),
    )?;
assert!(receipt.post_commit_failure().is_some());
```

That is the closing block of `receipts.rs`, so it is compiled and run by the
gate like every other snippet here.

## More examples

All five are runnable and currently pass:

| Example | Shows |
|---|---|
| `one_member` | The minimal single-member transaction (above) |
| `multi_member` | A bundle that commits or rolls back as one unit |
| `custom_validator` | Supplying your own `CandidateValidator` |
| `ownership` | Proving ownership by exact prior content with `ExactDigestVerifier` |
| `receipts` | Interpreting `Committed` / `RolledBack` / `RecoveryRequired` |

```sh
cargo run -p eggup-core --example multi_member
```

## Where to go next

- [crates.md](crates.md) — picking the right crate.
- [releases.md](releases.md) — versions, pinning, and upgrading.
- [`crates/eggup-core/docs/transaction.md`](../crates/eggup-core/docs/transaction.md) —
  the authoritative transaction contract.
- [`architecture/core-transaction.md`](../architecture/core-transaction.md) — the
  deep dive on why the core is shaped this way.
