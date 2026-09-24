---
name: verify-workflow
description: Run Eggup verification gate locally and interpret CI lanes — command order, focused runs, MSRV and platform evidence
---

# Eggup verification workflow

## Local gate (in this order)

```sh
./scripts/check-local.sh
# = cargo fmt --all -- --check
# + cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
# + cargo test --workspace --all-targets --all-features --locked
# + cargo doc --workspace --no-deps --locked
# + cargo tree --workspace --locked  # dependency-surface review, no gate
```

## Focused runs

- `cargo test -p eggup-core`, `cargo test -p eggup-core <name>`
- `cargo clippy -p <crate> --all-targets --locked -- -D warnings`

## CI lanes (`.github/workflows/ci.yml`) — what local does NOT cover

| Job | What it adds |
|---|---|
| `msrv` | `cargo check --workspace --all-targets --locked` on pinned 1.89.0 |
| `macos` | full `cargo test` on `macos-latest` |
| `windows-check` | `cargo check` on `windows-latest` (compile-only) |

CI never runs `cargo tree`; CI never publishes — releases are manual.
Closure records require per-platform reporting, so platform-gated changes
need CI (or manual matrix) evidence, not just the local script.
Public API changes also need rustdoc; keep crate `README.md` and
`CHANGELOG.md` (`Unreleased`, no-publication/no-migration disclaimer) current.
