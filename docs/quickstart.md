# Quickstart

This walkthrough runs the checked-in first-install example. It introduces the
core transaction flow without duplicating Rust code that can drift from the
compiled example.

## Requirements

- Rust 1.89 or newer, edition 2021.
- No extra features or build flags for `eggup-core`.

## Add the dependency

```toml
[dependencies]
eggup-core = "0.2.0"
```

## Run the example

From the Eggup workspace, run:

```sh
cargo run -p eggup-core --example one_member
```

The example prints `committed: Committed`. Its source is
[`one_member.rs`](../crates/eggup-core/examples/one_member.rs).

## What it does

The example creates a source file and an install directory under a temporary
root, then:

1. Calculates the source file's SHA-256 digest and declares it as the expected
   integrity value.
2. Describes the product, release, install root, and destination path.
3. Stages the bytes, verifies their digest, and runs candidate validation.
4. Commits with an ownership policy and checks the returned receipt.

The destination's parent directory must exist before commit. The core does not
create it for you. SHA-256 checks integrity only; it does not establish who
published the file or authenticate a release.

## Choose an ownership policy

The first-install example uses `ExistingAsOwnedVerifier` for convenience. It
reports any existing regular file as owned, so it is suitable only for examples
and tests. Do not use it as a production ownership check.

For replacement, use `ExactDigestVerifier` or a deployment-specific verifier to
prove that the existing file belongs to your installation. For first installs,
`AbsentOnlyVerifier` treats any existing destination as foreign. Pair it with
`AbsentPolicy::AllowCreate` to permit a new destination.
See the [`ownership` example](../crates/eggup-core/examples/ownership.rs) for
the exact-content approach.

## Check the receipt

A commit returns a receipt. Only `TransactionDisposition::Committed` means the
install succeeded. Handle `RolledBack` and `RecoveryRequired` explicitly. The
[`receipts` example](../crates/eggup-core/examples/receipts.rs) shows rollback
and post-commit check handling. Its recovery-required line is illustrative; the
example does not induce a real recovery failure.

Other examples cover multiple artifacts, custom validation, exact-content
ownership, and caller-proven stale-lock recovery:

- [`multi_member.rs`](../crates/eggup-core/examples/multi_member.rs)
- [`custom_validator.rs`](../crates/eggup-core/examples/custom_validator.rs)
- [`ownership.rs`](../crates/eggup-core/examples/ownership.rs)
- [`stale_lock_recovery.rs`](../crates/eggup-core/examples/stale_lock_recovery.rs)

Run any example from the workspace with
`cargo run -p eggup-core --example <name>`.

## Next steps

- [Choose a crate](crates.md) for transport, archive extraction, services, or
  Eggpack support.
- [Versions and upgrades](releases.md) for published version and migration notes.
- [`eggup-core` transaction contract](../crates/eggup-core/docs/transaction.md)
  for authoritative behavior.
