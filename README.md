# Eggup

Eggup is a set of Rust crates for applying local software updates. `eggup-core`
stages files, checks SHA-256 integrity, validates candidates, verifies
destination ownership, and commits with rollback and receipts. SHA-256 is
integrity evidence only. Eggup does not authenticate releases or verify
signatures.

Requires Rust 1.89 or newer and edition 2021.

```toml
[dependencies]
eggup-core = "0.2.0"
```

## Start here

- [Quickstart](docs/quickstart.md): run a verified first-install example.
- [Choose a crate](docs/crates.md): find the crates for your use case.
- [Versions and upgrades](docs/releases.md): registry status and 0.2.0 migration notes.

For the transaction contract, see
[`eggup-core` documentation](crates/eggup-core/docs/transaction.md). For the
workspace design, see [architecture overview](architecture/overview.md).
