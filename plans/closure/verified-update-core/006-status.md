# Verified Update Core M006 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/verified-update-core/006-core-package-qualification.md`

Source roadmap: `plans/subsystems/verified-update-core-roadmap.md#M006--core-package-qualification`

Reviewed repository baseline: `9f527beb20da585bc3cf56f45f1fd96fd418c124` (plan baseline; implementation ran at M005 closure `811d44e` plus qualification changes)

## Implementation commits/PRs

- M006 qualification commit (this pass): `feat: qualify eggup-core package (M006)` (pending SHA; see git log)
- Prior: `811d44e` (`feat: close core M005 safety/API corrective`)
- No PR was required for this local implementation pass. No publication was performed.

## Executive finding

M006 is complete. The corrected `eggup-core` is an independently consumable pre-1.0 Rust library: public API reviewed, rustdoc clean, five runnable examples, package metadata qualified, `cargo package` and `publish --dry-run` green, dependency surface minimal, MSRV proven, platform claims match evidence. First-consumer adoption may proceed against this versioned API without using private modules.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Public API/rustdoc review: stable terminology, ownership/precondition docs, failure semantics, no consumer policy, `non_exhaustive` where appropriate, no test surface, no misleading atomic claims | `cargo doc` clean with `#![deny(missing_docs)]`; `FileKind`, `PermissionsIntent`, `IntegrityRequirement`, `FailurePhase`, `FailureCategory` marked `#[non_exhaustive]`; canonical `Ownership`, `TransactionDisposition`, `CleanupDisposition`, `AbsentPolicy` stay exhaustive; `CommitFault`/`commit_with_fault` remain `pub(crate)` + `#[cfg(test)]`; only `atomic-feeling` qualified use + explicit non-atomicity in `docs/transaction.md` | passed |
| Examples for one-member, multi-member, custom validator, ownership verifier, receipt interpretation; only verified/validated commit path; rollback/recovery not success; crash/stale limits stated | `examples/{one_member,multi_member,custom_validator,ownership,receipts}.rs` all run green; each uses `prepare → verify_integrity → validate → commit(ownership)`; `receipts.rs` asserts rollback ≠ success and documents recovery-required severity + manual lock action | passed |
| Package manifest metadata | `description`, `readme`, `license`, `repository`, `homepage`, `documentation`, `keywords`, `categories`, `exclude = ["docs/","tests/","benches/"]` in `crates/eggup-core/Cargo.toml` | passed |
| Crates.io name availability at execution time | `GET https://crates.io/api/v1/crates/eggup-core` → 404 (available); no publication performed | passed |
| `cargo package` + `publish --dry-run`; content inspection | `cargo +1.89.0 package -p eggup-core --locked --allow-dirty`: 19 files, 155.5 KiB (31.3 KiB compressed), verification tests passed; `publish --dry-run --allow-dirty`: upload dry-run ok; `.crate` listing contains `Cargo.toml`, `README.md`, `src/*`, `examples/*` only — no planning archives, target outputs, secrets, or CI assets | passed |
| Dependency tree + duplicate/security review + rationale | `cargo +1.89.0 tree --workspace --locked`: `eggup-core → sha2 0.10.9 → digest/block-buffer/cpufeatures/...` only; no HTTP/TLS/service-manager; no duplicates beyond `generic-array`/`typenum` via `sha2` DAG (expected); `sha2` rationale: native local SHA-256 for integrity without transport | passed |
| Representative examples compile | `cargo check --workspace --all-targets` covers all five examples; `cargo run -p eggup-core --example …` green for each | passed |
| Compile/check lanes for desktop/server targets; MSRV proof | `cargo +1.89.0 check/test/clippy/doc` green (37/37 tests); stable toolchain green; `ci.yml` defines stable, MSRV 1.89, macOS, Windows-check lanes | passed |
| README/changelog version-state reconciliation | `README.md`, `crates/eggup-core/README.md`, `CHANGELOG.md` describe corrected transaction + qualification state; no stale foundation-only wording | passed |
| No consumer-specific constants; no HTTP/TLS/service; no unsafe; no auto-publication; package metadata implies no authenticity | `cargo tree` + `grep` clean; `#![forbid(unsafe_code)]`; checksum-only docs; no publish workflow | passed |

## Exact tests/commands actually run

Environment: `Darwin 25.6.0 arm64`, stable `1.98.1`, MSRV `1.89.0`.

```text
cargo fmt --all
cargo +1.89.0 fmt --all -- --check
cargo +1.89.0 check --workspace --all-targets --locked
cargo +1.89.0 clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo +1.89.0 test --workspace --all-targets --all-features --locked
cargo +1.89.0 test --doc -p eggup-core --locked
cargo +1.89.0 doc --workspace --no-deps --locked
cargo +1.89.0 tree --workspace --locked
cargo +1.89.0 package -p eggup-core --locked --allow-dirty
cargo +1.89.0 publish -p eggup-core --dry-run --locked --allow-dirty
cargo run -p eggup-core --example one_member
cargo run -p eggup-core --example multi_member
cargo run -p eggup-core --example custom_validator
cargo run -p eggup-core --example ownership
cargo run -p eggup-core --example receipts
./scripts/check-local.sh
```

Results: all passed. `test`: 37/37. `test --doc`: 0 doctests (examples cover the five flows; no inline doctests claimed). `package`: 19 files. `publish --dry-run`: dry-run upload ok. Examples: each prints `Committed` (and `receipts` demonstrates `RolledBack` + failure report).

## Public API review notes

- All exported types carry ownership, precondition, and failure docs.
- `#[non_exhaustive]` added to `FileKind`, `PermissionsIntent`, `IntegrityRequirement`, `FailurePhase`, `FailureCategory` to permit future hash kinds, permission modes, phases, and categories without breaking downstream matches.
- `Ownership::{Absent, Owned, Foreign, Unknown}`, `AbsentPolicy::{AllowCreate, DenyCreate}`, `TransactionDisposition`, `CleanupDisposition` intentionally exhaustive (closed canonical sets).
- No public test/fault-injection surface: `CommitFault`, `commit_with_fault`, `prepare_with_failure`, `test_support` are `pub(crate)` and/or `#[cfg(test)]`.
- `ExistingAsOwnedVerifier` is public but explicitly documented as test/example-oriented; production consumers should use `ExactDigestVerifier` or deployment-specific identity.
- No `unsafe`, no `atomic` claims beyond qualified `atomic-feeling`, no authenticity implications in metadata (`description` states checksum integrity only).

## Direct/transitive dependency summary

```text
eggup-core v0.1.0
└── sha2 v0.10.9
    ├── cfg-if, cpufeatures (+libc), digest (+block-buffer, crypto-common, generic-array, typenum)
```

Runtime surface: one direct dependency (`sha2`), justified for native local hashing. No HTTP/TLS (`eggfetch`, `reqwest`, `hyper`, `rustls`, `native-tls`), no service manager, no async runtime, no serialization. No advisory scanner is adopted as durable tooling (per plan, none added solely for this milestone).

## Platform matrix

| Platform | Evidence | Result |
|---|---|---|
| Linux x86_64 | Hosted CI lane defined (`ubuntu-latest` stable + MSRV); not run locally here | lane defined, no local inference |
| macOS (Darwin arm64, local) | `cargo +1.89.0 test` 37/37, examples green, `package`/`publish --dry-run` green | passed |
| Windows | `windows-check` compile lane defined; not run locally | compile lane defined, live replacement explicitly unclaimed |

Supported-platform claims match evidence: local verified transaction behavior is proven on macOS; Windows live replacement remains unproven and is documented as such. No Windows/macOS behavior is inferred from Linux-only or macOS-only runs.

## MSRV result

`rust-version = "1.89"` in workspace and package. `cargo +1.89.0 check/test/clippy/doc/package` all green. `rustc 1.89.0 (29483883e 2025-08-04)` used for proof.

## Crate-name availability result

`eggup-core` is available on crates.io at execution time (API 404). No reservation or publication was made. If unavailable at future publication time, stop rather than publishing under an accidental name (per plan).

## Unresolved limitations

- Low / accepted: Windows live replacement unproven (compile lane only).
- Low / accepted: No crash journal; stale locks manual (carried from M005).
- Informational: Hosted CI results not available locally; lanes defined.
- None: No known high/medium safety contract defect remains.

## Disposition and recommendation

M006 is closed. Publication is **not** performed by this milestone (by design). Whether to publish `eggup-core 0.1.0` is a separate release decision; the package is qualified for first-consumer adoption via versioned path/API dependency without private-module use. Proceed to acquisition M001/M002 and then eggsact adoption M001 per the registry order.
