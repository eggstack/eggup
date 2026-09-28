# Eggpack Manifest Interoperability M002a — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/eggpack-manifest-interoperability/002a-hosted-qualification-and-clippy-corrective.md`

Source roadmap: `plans/subsystems/eggpack-manifest-interoperability-roadmap.md`

Original plan: `plans/implementation/eggpack-manifest-interoperability/002-archive-extraction-handoff.md`

Original closure: `plans/closure/eggpack-manifest-interoperability/002-status.md` (amended with the M002a addendum; LOW Windows-evidence finding resolved)

Reviewed repository baseline: `9a5500e221e3eec24773a6517ca61a90e1b5afd9`

Implementation commits:

- `9a5500e` — fix(eggpack): remove redundant into_iter in bind_archive_members; run eggup-eggpack tests on Windows CI (source correction + workflow change; pushed 2026-09-28)

Original failed qualification: hosted run `36463041223` — Stable Linux failed at `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` (`clippy::useless_conversion` at `crates/eggup-eggpack/src/lib.rs:559`); macOS, Rust 1.89 MSRV, and Windows passed. Stable checks stopped at clippy, so full Stable tests/docs did not execute on that run.

## Executive finding

M002a restores truthful hosted qualification for Eggpack Interop M002 with a minimal two-line delta and no architecture or public API change. Current Stable clippy is green, full Stable tests/docs run after clippy, direct hosted Windows adapter tests are green, and macOS + MSRV remain green. The M002 closure no longer carries the LOW "Windows adapter tests not run" finding.

## Requirement-to-evidence matrix

| Requirement (source plan Section 10/13) | Evidence | Result |
|---|---|---|
| current Stable clippy green, no lint allow | hosted run `36477024102` Stable job: `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` success; local clippy clean; source uses `members.iter().zip(bound_members)`, no `#[allow]` | passed |
| full Stable tests/docs run after clippy | same run: `cargo test --workspace --all-targets --all-features --locked` + `cargo doc --workspace --no-deps --locked` success after clippy | passed |
| direct hosted Windows adapter tests green | Windows job step `Run cargo test -p eggup-eggpack --all-targets --all-features --locked`: success (covers unit 7 + archive_handoff 11 + interoperability 28) | passed |
| macOS remains green | macOS job `cargo test --workspace --all-targets --all-features --locked`: success | passed |
| MSRV 1.89 remains green | MSRV job `cargo check --workspace --all-targets --locked` on 1.89.0: success; local `cargo +1.89.0 check` clean | passed |
| M002 closure LOW finding removed | `002-status.md` M002a addendum marks it resolved with run + command evidence | passed |
| no runtime/public API behavior changed | diff is one iterator expression + one CI step; focused adapter suites 7/11/28 green locally; full workspace green | passed |

## Production implementation and audit evidence

Source diff (`crates/eggup-eggpack/src/lib.rs:559`):

```diff
-    for (declared, bound) in members.iter().zip(bound_members.into_iter()) {
+    for (declared, bound) in members.iter().zip(bound_members) {
```

Workflow diff (`.github/workflows/ci.yml`, Windows lane):

```diff
       - run: cargo test -p eggup-acquisition -p eggup-archive -p eggup-curl --locked
+      - run: cargo test -p eggup-eggpack --all-targets --all-features --locked
       - run: cargo test -p eggup-service --lib diagnostic_byte_bounds_preserve_utf8_at_256_and_512_edges --locked
```

The broader `--all-targets --all-features` form was chosen per the plan's preference: the adapter suites are portable (no platform-gated logic; the one symlink test degrades to a logged skip on privilege-constrained Windows hosts) and runtime cost is seconds.

## Exact verification commands and results

Local environment: Darwin arm64; stable `rustc 1.89.0` / Cargo 1.89.0 (MSRV toolchain itself); head `9a5500e` (C009 docs edits were uncommitted working-tree state in `plans/` only and do not affect Rust/CI lanes).

Passed:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p eggup-eggpack --all-targets --all-features --locked
  lib unit tests:                7 passed
  tests/archive_handoff.rs:     11 passed
  tests/interoperability.rs:    28 passed
cargo test --workspace --all-targets --all-features --locked   (all suites green, zero failures)
cargo +1.89.0 check --workspace --all-targets --locked
cargo doc --workspace --no-deps --locked
git diff --check
```

Hosted run `36477024102` (head `9a5500e`, branch `main`, push):

```text
Stable checks:                                        success (fmt + clippy + workspace tests + doc)
MSRV check (1.89.0):                                  success
macOS tests:                                          success
Windows archive, acquisition, and service tests
  + eggup-eggpack tests and check:                    success
  - cargo test -p eggup-acquisition -p eggup-archive -p eggup-curl --locked: success
  - cargo test -p eggup-eggpack --all-targets --all-features --locked:       success
  - eggup-service targeted lib tests + windows_scm::tests:                   success (all four)
  - cargo check --workspace --all-targets --locked:                           success
```

## Invariant review

- no public API change: diff touches one iterator expression and CI YAML only.
- five M002 adapter helpers retain semantics: focused suites green unchanged.
- declaration-order binding remains fail-closed: `bind_archive_members_fails_closed_on_count_mismatch` green.
- direct/bundle adapter APIs unchanged: `tests/interoperability.rs` 28/28 green.
- no producer authority enters Eggup: no Eggpack-side file touched.
- `eggup-core` remains archive/Eggpack independent: untouched by this pass.
- no platform-specific behavior added to satisfy CI: the iterator fix is platform-neutral; Windows only gains test execution.
- Rust 1.89 supported: MSRV lane green.
- first-party unsafe forbidden: no unsafe added (`unsafe_code = "deny"` holds via clippy/workspace lints).

## Failure/rollback/recovery review

No runtime failure semantics change. No Windows adapter defect surfaced (the stop-condition trigger did not fire), so no scope expansion was needed. Removing `.into_iter()` did not change type inference or ownership behavior: `Vec<BoundMember>` iterated by value either way; all 11 handoff tests green.

## Compatibility and migration review

No compatibility or consumer migration effect. Additive-only M002 surface untouched.

## Security review

No new trust boundary: one iterator-expression simplification plus CI test execution. No secret, URL, digest, or file-content flow changed.

## Documentation/operations evidence

- `plans/closure/eggpack-manifest-interoperability/002-status.md` amended (LOW finding resolved + M002a addendum with run/command evidence).
- this closure record.
- roadmap + registry updates recorded in the M002a/M008a/C009 reconciliation commit (see Registry updates below).

No changelog entry: lint-only source correction with no behavior change, per plan Section 12.

## Unresolved findings

None. No medium-or-higher finding; the single LOW finding is resolved.

## Roadmap disposition

Eggpack Interop M002a is closed. M002's qualification is now fully hosted on all four lanes. Core M008a is unblocked: current head `9a5500e` is green (`36477024102`), satisfying its hard dependency on M002a closure + green current head. Verified Update Core roadmap and registry M008a rows move from blocked to ready as part of this batch's reconciliation.

## Registry updates

- M002a row: ready → closed (run `36477024102`).
- M002 row: qualification corrective open → closed with hosted evidence.
- M008a row: blocked on M002a → ready (dependency satisfied).
- Execution graph: M002a → M008a edge now traversable; C009 independent docs-only path unaffected.
