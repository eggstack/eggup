# Acquisition Transport M006 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/acquisition-transport/006-m005-windows-portability-and-cross-closure-qualification-corrective.md`

Source roadmap: `plans/subsystems/acquisition-transport-roadmap.md#M006--m005-windows-portability-and-cross-closure-qualification-corrective`

Reviewed repository baseline: `759f175828b0be9b462fdd5336b9c3115ebb3534` (plan handoff; plan-stated baseline `eb989659feabd44e3c1441bb8eb522614ce96a31` is the M005/M006 implementation + closure head where the Windows lane failed)

Implementation commit: `1c601f29a16c952e90feaeebd0fa654401b54a86` (`fix(acquisition): gate Unix-only curl test support and core imports for Windows (M006)`).

Reference implementation reviewed (read-only): none re-read in this pass. Gregg remains at the previously reviewed `eggstack/gregg@8b18f9ee16461e3fa0ef0d804ed39ebb9183b727`; it was neither modified nor depended on (no `gregg` path/git dependency; `cargo tree --workspace --locked` contains no gregg node).

## Executive finding

M006 is complete. The post-M005/M006 hosted-qualification defect was a narrow
test/support portability failure: `eggup-curl` test code unconditionally
imported Unix-only `PermissionsExt` and called `set_mode`, so the Windows
`cargo check --workspace --all-targets --locked` lane could not compile the
test target. The same lane reported an `eggup-core::stage` unused-import
warning for `PermissionsIntent`. This pass target-gates that support code
without weakening Unix permission assertions, removes the Windows-only
warnings, reruns the full hosted matrix green, and reconciles M005/M006 closure
evidence plus stale roadmap/registry status. No transport, composition,
disposition, or Gregg semantics changed. No medium-or-higher finding remains open.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Windows compiles `eggup-curl` all targets without Unix-only API errors | `cargo check -p eggup-curl --all-targets --locked --target x86_64-pc-windows-msvc` green locally; hosted Windows job `108206656342` green | passed |
| Unix permission-hardening coverage remains present | `artifact_temp_is_owner_private_and_no_clobber` passes on Unix and still asserts promoted `0600`; `PermissionsExt` import + `fake_curl_script` + 6 shell tests `#[cfg(unix)]`-gated, module itself still compiled on Windows | passed |
| Windows-only `PermissionsIntent` warning gone | `PermissionsIntent` import `#[cfg(unix)]`-gated in `stage.rs`; same-class test-only `Duration` import gated in `core/lib.rs`; per-package Windows-target checks for core/curl/acquisition/service report zero warnings | passed |
| Stable, MSRV, macOS, Windows hosted jobs all pass on corrective head | Workflow `36176009068` success: Stable `108206656486`, MSRV `108206656574`, macOS `108206656474`, Windows `108206656342` all success | passed |
| M005/M006 closures truthfully record pending/failure/correction | Addenda appended to `005-status.md` and `service-lifecycle/006-status.md` with failed runs `36169295410`/`36175333145`, diagnostics, corrective SHA, and succeeding run `36176009068` | passed |
| Stale service/consumer/registry status reconciled | Acquisition/service/consumer roadmaps + registry updated (see below); plan status → implemented | passed |
| Gregg M004 remains unwritten, Gregg unmodified | No gregg paths in `git status`; `grep` for gregg in crate manifests empty; `cargo tree` has no gregg node | passed |
| No public API/semantic regression | `git show --stat` shows only test/support `cfg` gating + import gating; transport/composition/disposition tests green (acquisition 34, curl 16, service 98, core 43) | passed |

## Production implementation evidence

- `crates/eggup-curl/src/lib.rs` (tests only): `use std::os::unix::fs::PermissionsExt` now `#[cfg(unix)]`; `fake_curl_script` (shell + `xxd` + `set_mode(0o700)`) now `#[cfg(unix)]` with a doc note explaining Windows exercises the same paths via real-`curl.exe` integration + platform-neutral unit tests; six shell-dependent fake-child tests `#[cfg(unix)]`-gated (`tls_like_process_failure`, `proxy_disabled`, `proxy_custom`, `artifact_temp_is_owner_private`, `credential_material_is_redacted`, `invalid_limits_fail_before_spawn`). The module itself is never disabled on Windows: discovery, missing-executable `Unavailable`, real-curl metadata/artifact/redirect/timeout/truncation/cancellation, and fixture-composition tests still compile and run on all platforms.
- `crates/eggup-core/src/stage.rs`: `PermissionsIntent` import split to `#[cfg(unix)]`; `ArtifactMember`/`InstallPlan` remain unconditional. Permission semantics unchanged (`Executable → 0700`, `Preserve → 0700/0600` on Unix; no-op on non-Unix).
- `crates/eggup-core/src/lib.rs` (tests only): same-class `Duration` import (used only by a `#[cfg(unix)]` bounded-runner test) gated `#[cfg(unix)]` so the Windows test target reports zero warnings.
- No production `CurlTransport` change: direct-executable/no-shell invariant, timeout/redirect/protocol/proxy bounds, kill/reap, redaction, and `Unavailable`/`NotFound` semantics untouched. No service disposition change.

## Exact commands and results

Environment (local): Darwin arm64; `rustc 1.89.0`; `cargo fmt`/`clippy`/`test` per `scripts/check-local.sh`.

Passed:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo check --workspace --all-targets --locked
cargo check -p eggup-curl --all-targets --locked
cargo check -p eggup-core --all-targets --locked
cargo test -p eggup-acquisition --locked
cargo test -p eggup-curl --locked
cargo test -p eggup-service --locked
cargo tree --workspace --locked
```

Focused results (local): acquisition 34 passed, curl 16 passed (including Unix `artifact_temp_is_owner_private_and_no_clobber`), core 43 passed, service 98 passed, eggfetch 24 passed; workspace `cargo test` reports 0 failed across all suites. `cargo tree` contains no gregg node.

Focused Windows-target evidence (local cross-check from macOS; full workspace cross-compile is limited by `ring` C toolchain, so per-package lanes are reported):

```text
cargo check -p eggup-curl --all-targets --locked --target x86_64-pc-windows-msvc  # pass, zero warnings
cargo check -p eggup-core --all-targets --locked --target x86_64-pc-windows-msvc  # pass, zero warnings
cargo check -p eggup-acquisition --all-targets --locked --target x86_64-pc-windows-msvc  # pass
cargo check -p eggup-service --all-targets --locked --target x86_64-pc-windows-msvc  # pass
```

Original failure (preserved history):

```text
error[E0433]: cannot find `unix` in `os`
 --> crates/eggup-curl/src/lib.rs:662:18
error[E0599]: no method named `set_mode` found for struct `Permissions`
 --> crates/eggup-curl/src/lib.rs:787:15
warning: unused import: `PermissionsIntent`
 --> crates/eggup-core/src/stage.rs:5:50
```

Hosted evidence:

- Failed (original): workflow `36169295410` failure — Stable `108184625133` success, MSRV `108184625222` success, macOS `108184625296` success, Windows `108184625301` failure.
- Failed (plan-only head `759f175`): workflow `36175333145` failure — Windows `108204439265` failure (Stable/MSRV/macOS green).
- Succeeding (corrective head `1c601f2`): workflow `36176009068` success — Stable `108206656486` success, MSRV `108206656574` success, macOS `108206656474` success, Windows `108206656342` success (`cargo check --workspace --all-targets --locked`).

## Invariant review

- M005 transport semantics unchanged (exact `NotFound` terminal; default fallback unavailability-only; curl-only avoids Eggfetch/TLS).
- Unix 0700/0600 permission assertions remain meaningful and green on Unix.
- Windows never compiles Unix-only APIs (gated imports/helpers/tests).
- M006 disposition/revalidation semantics unchanged; service production code untouched.
- Gregg untouched and unlinked; consumer M004 still unwritten.
- Closure evidence records pending, failed, and corrected hosted results truthfully; red lane gated closure until green.

## Failure and recovery review

This corrective changes no runtime transport cancellation, lifecycle restart,
rollback, or mutation semantics. No production-path semantic change occurred,
so no new failure-mode table is required beyond the preserved M005/M006
matrices. The only failure addressed is compile-time test-target portability;
its recovery is the `#[cfg(unix)]` gating above. No stop condition from the
plan was triggered: no production curl behavior was exposed, no no-shell
weakening was needed, service code needed no Windows fix, no public API change
was required, and hosted Windows turned green after the identified test-code
correction.

## Compatibility and migration review

No public API change. No downstream migration required. No crate published.
Gregg remains untouched and M004 remains unwritten. Additive `cfg` gating only.

## Security review

- No executable-path, proxy, redaction, temp-ownership, or no-clobber behavior changed.
- Fake-curl shell scripts remain test-only Unix doubles; production still uses direct-executable invocation with cleared env and `--` URL terminator.
- Staged-file modes unchanged on Unix; Windows behavior unchanged (no-op permission path as before).
- No medium-or-higher finding open.

## Documentation and operations evidence

- M005 closure addendum: `plans/closure/acquisition-transport/005-status.md#post-closure-corrective-addendum--acquisition-m006-2026-09-25`.
- M006 service closure addendum: `plans/closure/service-lifecycle/006-status.md#post-closure-corrective-addendum--acquisition-m006-2026-09-25`.
- Roadmaps reconciled: acquisition M005 → closed/qualified, M006 → closed; service M006 → closed (evidence reconciled); consumer M004 → prerequisites satisfied, plan still intentionally unwritten.
- Registry reconciled (see below).
- Plan `006-*.md` status → implemented.
- No crate `README.md`/`CHANGELOG.md` change required (no public API/behavior change).

## Unresolved findings

- None: no medium-or-higher correctness/security finding remains open.
- Informational: full-workspace Windows cross-compile from macOS remains limited by third-party `ring` C toolchain (`assert.h` absent for the Windows target from Darwin); hosted native Windows `cargo check --workspace --all-targets` is the authoritative lane and is green. Per-package Windows-target checks are green locally.
- Informational: shell-dependent fake-child tests run on Unix only by design; Windows coverage comes from real-`curl.exe` integration + platform-neutral unit/composition tests. No whole-module disable occurred.

## Roadmap disposition

Update `plans/subsystems/acquisition-transport-roadmap.md` M005 to closed/qualified and M006 to closed; update `plans/subsystems/service-lifecycle-roadmap.md` M006 evidence-reconciled/closed and its dependency graph; update `plans/subsystems/consumer-adoption-roadmap.md` M004 blocker to prerequisites-satisfied/plan-unwritten; update `plans/registry.md` accordingly.

## Registry updates

- Acquisition M005: implemented → closed/qualified (with M006 corrective).
- Acquisition M006: ready → closed (`plans/closure/acquisition-transport/006-status.md`).
- Service M006: implemented → closed/evidence-reconciled (no service code change).
- Consumer adoption Gregg M004: deferred → writable prerequisite satisfied (acquisition M005 + service M006 closed with green hosted matrix); plan still intentionally unwritten pending a separate authoring decision; no Gregg migration authorized by this closure.
- Current state: acquisition primary path fully cross-platform qualified (native + curl + composition); service disposition/reference qualification reconciled.
