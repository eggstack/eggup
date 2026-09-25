# Service Lifecycle M006 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/service-lifecycle/006-daemon-update-disposition-and-reference-qualification.md`

Source roadmap: `plans/subsystems/service-lifecycle-roadmap.md#M006--daemon-update-disposition-and-reference-qualification`

Reviewed repository baseline: `942a977a7ef86fd82105503cfc1802d22754c07f` (M005 closure; plan-stated baseline `881c95ff069d3d465a282cb6a495ba6fcb70cb6f` is an ancestor)

Implementation commit: `7627494088ae11f98d652a12899a8b575b1ef578` (`feat(service): add daemon update disposition and reference qualification (M006)`).

Reference implementation reviewed (read-only): `eggstack/gregg@8b18f9ee16461e3fa0ef0d804ed39ebb9183b727` (`crates/greggd/src/update.rs`, `crates/greggd/src/startup/*`, `crates/greggd/src/service/*`). Gregg was neither modified, migrated, nor depended on (no `gregg` path/git dependency; `cargo tree` contains no gregg node; no Gregg product strings in public API).

## Executive finding

M006 is complete. `eggup-service` now exposes a product-neutral five-disposition
update runtime model (`ManagedRunning`, `ManagedStopped`, `DirectRunning`,
`Stopped`, `ForeignPreserved`) with pure `plan_unix`/`plan_windows` planners
reproducing the reviewed reference matrix, a minimal caller-owned
`DirectRuntimeControl` seam, and `commit_with_disposition` orchestration that
reuses Core M007 `KeepInstalled | RollBack` / `RecoveryRequired` semantics.
Artifact authority and manager mutation authority are separate;
`ForeignPreserved` performs zero manager mutation and `DirectRunning` uses only
exact direct control. Preparation precedes quiescence and authority is
revalidated immediately before mutation. Existing M001-M005 behavior is
preserved. No medium-or-higher correctness/security finding remains open.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Product-neutral five-disposition model | `UpdateRuntimeDisposition` with 5 variants; `no_gregg_strings_in_public_types` | passed |
| Pure planner reproduces reference matrix | `unix_reference_matrix` (14 cases) + `windows_reference_matrix` (7 cases) | passed |
| Artifact vs manager authority separate | `DispositionBaseline` + per-disposition mutation rules; `ForeignPreserved` zero-mutation test | passed |
| ForeignPreserved performs no manager mutation | `foreign_preserved_performs_zero_manager_mutation` (registration unchanged, artifacts committed) | passed |
| DirectRunning uses only exact caller-owned control | `direct_running_uses_only_direct_seam` (direct stop/start events, manager still Foreign) | passed |
| Preparation precedes quiescence | `preparation_completes_before_first_quiesce` (inspect before stop; invalid policy zero mutation) | passed |
| Revalidation before mutation | `owned_to_foreign_revalidation_fails_before_stop`, `direct_identity_change_fails_before_stop` (Preflight, zero further mutation) | passed |
| Owned-to-foreign/unknown fails before destructive action | Revalidation compares fresh vs baseline ownership/state; drift returns Preflight | passed |
| Stopped preservation, KeepInstalled, RollBack, RecoveryRequired truthful | `managed_stopped_stays_stopped_under_preserve`, `stopped_fabricates_no_restart`, `keepinstalled_reports_committed_plus_lifecycle_failure`, `rollback_quiesces_new_generation_and_restores_old`, `recovery_required_suppresses_all_automatic_starts` | passed |
| M001-M005 tests remain green | `cargo test -p eggup-service` 98 passed (83 pre-existing + 15 new) | passed |
| No Gregg dependency/migration | `cargo tree -p eggup-service` has no gregg; grep for gregg in `crates/eggup-service/src` empty | passed |
| Rust 1.89, package/docs, hosted platform | `rustc 1.89.0`; `cargo doc` green; `cargo package -p eggup-service` verify green; hosted CI pending push | passed with CI pending |
| Ordering: prepare/verify before stopping | ValidatedTransaction input + inspect-before-stop event evidence | passed |

## Production implementation evidence

- `crates/eggup-service/src/disposition.rs` (new, `forbid(unsafe)` + `deny(missing_docs)`):
  `UpdateRuntimeDisposition`, `KnownConfig::{Absent, Present, Unknown}`,
  `DirectState::{Running, Stopped, Unknown}`, `DirectObservation`
  (state + exact ownership + executable/config identity),
  `UnixPlannerInput`/`WindowsPlannerInput`/`WindowsServiceState`,
  pure `plan_unix`/`plan_windows`, `DirectRuntimeControl` trait
  (`inspect`/`stop`/`start` with bounded deadlines), `DispositionBaseline`,
  `commit_with_disposition` + manager/direct commit paths reusing
  `commit_with_post_commit` (Core backup/rollback, not duplicated).
- `crates/eggup-service/src/lib.rs`: `mod disposition` + public re-exports;
  existing `commit_with_lifecycle` untouched and source-compatible.
- Planners are pure/deterministic (no manager I/O); orchestration observes
  after preparation (Validated input), validates baseline consistency, then
  revalidates fresh manager (+ direct for DirectRunning) immediately before
  the first stop. Owned→Foreign/Unknown or direct identity/ownership loss
  returns Preflight with zero further mutation.
- Per-disposition behavior: ManagedRunning (revalidate Owned+Running →
  quiesce → commit → restore/start per policy → check); ManagedStopped
  (commit, preserve stopped unless EnsureRunning/EnsureStopped explicitly);
  DirectRunning (revalidate exact direct → quiesce via seam → commit →
  restore direct → check; zero manager mutation); Stopped (commit, no
  fabricated restart; EnsureRunning on Owned may start explicitly, otherwise
  no mutation); ForeignPreserved (zero manager stop/start/install/uninstall;
  artifact commit under Core ownership only; check still runs read-only).
- RollBack quiesces the new generation before artifact rollback (manager and
  direct paths); successful rollback restores pre-update state (manager paths
  only; ForeignPreserved/Stopped never fabricate starts); RecoveryRequired
  maps to `NotAttemptedRecoveryRequired` with no automatic starts.

## Exact commands and results

Environment: Darwin 25.6.0 arm64; `rustc 1.89.0`.

Passed:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo tree --workspace --locked
cargo check --workspace --all-targets --locked
cargo package -p eggup-service --allow-dirty
cargo package -p eggup-service --allow-dirty --no-verify
```

Service suite: `cargo test -p eggup-service` 98 passed, 0 failed (83
pre-existing M001-M005 + Windows SCM + lifecycle_update, 15 new disposition:
2 planner matrices + 13 orchestration/failure). Workspace `cargo test` green
across all suites (acquisition 34, eggfetch 43, curl 16, core 24+7+28,
service 98).

Native lanes (local Darwin): `native_launchd_smoke_on_macos_only` and
`native_crontab_list_only_never_mutates` pass as part of the service suite.
Windows SCM and Linux systemd lanes rely on hosted CI (compile-only Windows
check + hosted tests) pending push; deterministic planner/orchestration tests
do not require native managers and are reported as such (no cross-platform
inference).

Package: `cargo package -p eggup-service --allow-dirty` verifies cleanly
(only `eggup-core` path dependency, already published).

Hosted Linux/macOS/Windows CI pending push; local evidence is Darwin only.

## Invariant review

- Foreign/unknown registrations never destructively mutated (require Owned
  for all manager stop/start; ForeignPreserved path contains no mutating
  calls; revalidation denies on drift).
- Artifact vs manager authority separate (`DispositionBaseline` + planned
  disposition; Core `CommitOwnership` independent of service `Ownership`).
- ForeignPreserved authorizes zero manager mutation and never converts the
  manager to Eggup ownership.
- Unknown/contradictory active states fail closed (planners return
  `InvalidInput`; orchestration returns Preflight).
- Validated candidate exists before any quiesce (ValidatedTransaction input +
  inspect-before-stop events + invalid-policy zero-mutation test).
- Ownership revalidated after preparation, immediately before mutation.
- Stopped remains stopped under Preserve; RecoveryRequired never auto-starts;
  rollback never restarts against uncertain artifacts (new generation quiesced
  first).
- Direct seam caller-owned, exact-instance/config scoped; no health semantics,
  no endpoint/socket/PID/health-JSON content in `eggup-service`.
- All manager/direct transitions bounded (policy timeouts validated, zero
  rejected, `>300 s` rejected; remaining-budget checks before check/start).
- No implicit privilege escalation (no sudo/elevation text; adapters unchanged).
- M005 `commit_with_lifecycle` available and source-compatible (untouched;
  full M005 suite green).
- No Gregg/greggd strings in public types (test asserts).

## Failure and recovery review

| State/event | Result |
|---|---|
| Preparation failure (invalid policy) | Zero lifecycle mutation; `InvalidPolicy` |
| Baseline/disposition mismatch | Preflight, zero mutation |
| Revalidation drift (manager Owned→Foreign/Unknown/state change) | Preflight, zero further mutation |
| Direct identity/ownership change | Preflight, zero further mutation |
| ManagedRunning stop failure | Quiesce + best-effort restore to Running |
| Direct stop failure | Quiesce + best-effort direct restart |
| Post-commit restore/check failure, KeepInstalled | Committed + `post_commit_failure`; `Failed` when restore failed |
| Post-commit failure, RollBack | New generation quiesced, artifact rollback, old lifecycle restored |
| Rollback with new-generation quiesce failure | `rollback_quiesce_failure` retained separately from Core receipt |
| Core Err before receipt | Restore pre-update state when quiesced; `CoreBeforeReceipt` |
| RecoveryRequired | `NotAttemptedRecoveryRequired`; no automatic starts |
| Final inspect failure | `observation_failure` with `final_snapshot: None`; Core disposition unchanged |
| Caller check panic | Caught, converted to check failure per M005 policy |

## Compatibility and migration review

Additive only. New module, types, planners, seam, and orchestration entrypoint;
no existing public signature changed. `commit_with_lifecycle` semantics
preserved exactly. No downstream migration (Gregg unmigrated by design).
No version rewritten; no crate published (CHANGELOG `Unreleased`).

## Security review

- No process discovery by name or host scanning; direct evidence is
  caller-supplied exact instance/config.
- No service-definition migration/refresh in the disposition path;
  `commit_with_disposition` never installs/refreshes registrations.
- Parsers fail closed: ambiguous/unknown configs, unparseable executables,
  transitioning/unknown states map to `Unknown`/error, never to Owned.
- Config identity uses exact path equality; missing is Foreign, ambiguous is
  Unknown (inherited M003 semantics).
- Timeouts bounded; panics in manager/direct/check caught and converted to
  bounded failures (no secret content; 512-char bound, control chars stripped).
- No medium-or-higher finding open.

## Documentation and operations evidence

- `crates/eggup-service/README.md`: disposition model, authority separation,
  seam safety, planner/orchestration entrypoints.
- `architecture/overview.md`: service row lists M006 types.
- Rustdoc on all new public items (`deny(missing_docs)` green).
- `CHANGELOG.md`: `Unreleased` M006 entry with no-publication/no-migration disclaimer.
- Consumer adoption roadmap: Gregg remains unmigrated (see registry).
- Gregg referenced only in plan/closure evidence, never in public API examples.

## Unresolved findings

- Informational: hosted CI (Linux/macOS/Windows + MSRV lane) pending push;
  local evidence is Darwin arm64 + `rustc 1.89.0` only.
- None: no medium-or-higher correctness/security finding remains open.

## Roadmap disposition

Update `plans/subsystems/service-lifecycle-roadmap.md` M006 to closed and
`plans/registry.md` to move M006 from ready to closed. With Acquisition M005
and Service M006 both closed, Gregg consumer M004 becomes writable (plan
intentionally unwritten until now); authoring it remains a separate decision
and no Gregg migration is authorized by this closure.

## Registry updates

- Service M006: ready → closed (`plans/closure/service-lifecycle/006-status.md`).
- Consumer adoption Gregg M004: deferred → writable prerequisite satisfied
  (acquisition M005 + service M006 closed); plan still intentionally unwritten.
- Current state: service disposition/reference qualification complete.

## Post-closure corrective addendum — acquisition M006 (2026-09-25)

This record was originally written with hosted CI pending. The subsequent push
proved the shared Windows workspace lane was red for a defect outside service
production code. History is preserved; nothing above is rewritten.

Original failure is the same acquisition-head event recorded in
`plans/closure/acquisition-transport/005-status.md`: workflow `36169295410`,
Windows job `108184625301` (`cargo check --workspace --all-targets --locked`)
failed on `eggup-curl` Unix-only test support plus an `eggup-core::stage`
unused-import warning. Service M006 production code required no change; no
service stop condition from the corrective plan was triggered.

Corrective implementation `1c601f29a16c952e90feaeebd0fa654401b54a86` leaves all
M006 disposition/revalidation semantics unchanged. Succeeding hosted matrix on
that head: workflow `36176009068` success with Stable `108206656486`, MSRV
`108206656574`, macOS `108206656474`, and Windows `108206656342` all green.

Final disposition: M006 implementation stands; hosted workspace qualification is
reconciled through acquisition M006
(`plans/closure/acquisition-transport/006-status.md`). No service production
change was required.
