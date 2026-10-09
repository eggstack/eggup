# Service Lifecycle M010 — Owned Failed-systemd Service Quiescence Corrective

Status: ready for implementation.
Repository baseline: `eggstack/eggup@70ec4e63c52988bc6c82bea30d14f72eefda920a` (2026-10-08).
Source roadmap: `plans/subsystems/service-lifecycle-roadmap.md`.
Primary class: invariant / corrective.
Consumer blocker: `dbowm91/wg-basic`, Distribution M004 C001 at `plans/m004-update-correctives`, closure disposition `plans/closure/distribution/004-c001-status.md`.

## 1. Objective

Supply a product-neutral, ownership-safe systemd service-quiescence operation that handles a known *failed* owned service during update rollback/recovery. A service reported as `failed` by systemd is not by itself a proof that no processes or cgroup tasks are running. The API must distinguish (a) recognized failure diagnosis, (b) authority over the exact owned registration, and (c) demonstrably completed quiescence.

Enable a consumer to stop a failed candidate and restore a compatible application binary/database without adding a product-specific raw `systemctl` fallback. Preserve fail-closed behavior for foreign, malformed, unidentifiable, transitioning, and unverifiably quiesced registrations.

## 2. Readiness, historical reference, and detection gap

- Service M001–M009 are closed, with `eggup-service 0.1.2` published and the workspace at `0.1.3`. See `plans/implementation/service-lifecycle/005-prepared-transaction-lifecycle-integration.md`, `plans/closure/service-lifecycle/005-status.md`, `plans/implementation/service-lifecycle/006-daemon-update-disposition-and-reference-qualification.md`, `plans/closure/service-lifecycle/006-status.md`, and `plans/closure/service-lifecycle/009-status.md`.
- In `crates/eggup-service/src/lib.rs`, `systemd_ownership()` maps `ActiveState=active` to Running, `inactive` to Stopped, `activating|deactivating|reloading` to Transitioning, and all other values (including `failed`) to Unknown.
- `SystemdManager::stop()` checks ownership via the parsed `ExecStart` and systemd unit, issues an owned `systemctl stop`, then `poll_active(false)` accepts only `is-active=inactive` with status 3. A failed unit may remain reported as failed unless an explicit stop changes the manager state; exact behavior requires a real-systemd fixture.
- `wg-basic` refuses Unknown/Transitioning before its service-stop path and cannot prove safety or recover a candidate after such a failed start. Its C001 implementation therefore remains `corrective required`, not closed, despite earlier synthetic tests and unrelated Phase 9 rootful upgrade rehearsal.
- Earlier service M005/M006 closure tests exercised ordinary owned Running/Stopped, foreign preservation, and selected lifecycle failures, but did not discriminate an exact owned `failed` systemd unit from a genuinely unknown/unowned unit after candidate start failure.
- systemd documents `failed` as an error/diagnostic state; `reset-failed` clears failure metadata and restart limits. Resetting failure history is neither necessary nor sufficient to prove quiescence and must not be treated as an automatic repair action.

Hard dependency: historical service M001–M009 closure only; this is ready independently of wg-basic implementation. Release/publication M011 is blocked on this plan's strict closure.

## 3. Discriminating research fixture first (mandatory)

Build a Linux systemd VM fixture with a real service definition and exact owned `ServiceSpec`, with cases:
1. normal running -> `stop` -> inactive;
2. immediate exit nonzero -> `ActiveState=failed`, `is-active=failed`, no remaining tasks;
3. `Restart=on-failure` repeated failure/start-limit hit;
4. failed `ExecStartPre` or failing startup with surviving `ExecStopPost`/cgroup tasks if reproducible;
5. stop racing failed -> activating/auto-restart;
6. control case: exact unit file but changed `ExecStart`, foreign unit, missing unit, malformed show output, and inaccessible manager;
7. user/system scope as supported by the adapter.

Record before/after `systemctl show -p LoadState,ActiveState,SubState,ExecStart,MainPID,ControlPID,ControlGroup,Job,Result`, `is-active` output and exit code, `stop` exit code, and independently established process/cgroup quiescence. Include host systemd version. **Determine whether existing Eggup stop already converges failed -> inactive after a legal stop.** If so, the minimum correction may be to improve explicitly typed classification/diagnostics and tests, while the wg-basic call-site correction belongs downstream; do not invent a new public mechanism without a demonstrated need.

## 4. Invariants

- Only exact `Ownership::Owned` units may be stopped. Inspect ownership again immediately before manager mutation, and do not broaden ownership by unit name alone, PID alone, `LoadState` alone, or a caller's assertion.
- A reported `failed` or zero `MainPID` alone is **not** proof of no surviving unit processes. Successful quiescence requires actual stop/completion evidence and conservative post-stop active/substate and, where necessary, cgroup/process presence evidence. A partially failing stop never yields Completed.
- An unknown manager state, foreign or changed `ExecStart`, unknown/suspicious control-group identity, inconsistent observations, active manager job, restart race, permission failure, or timed-out call must produce a bounded, classified failure and retain recovery authority with the caller (no implicit restart).
- `systemctl reset-failed` is not a substitute for `stop`, and must not be invoked merely to normalize `is-active`. If test evidence requires it, stop for a new reviewed design because it clears manager diagnostics and rate-limit counters.
- Preserve absolute `OperationDeadline` budgets, process-output byte limits, bounded subprocess cleanup, exact service-registration matching, private data redaction and no automatic privilege escalation.
- Preserve the existing `LifecycleState` variants and meanings. The enum is already `#[non_exhaustive]`, so external consumers must include a wildcard arm; do not rely on the original plan's incorrect claim that it is closed. Keep failure diagnosis internal to the systemd adapter and do not add a public variant for this correction.
- No application-specific notion of database compatibility, service health HTTP, unit string, or VPN state enters Eggup.

## 5. Scope and non-scope

In scope: systemd adapter `inspect/stop` classification and quiescence proof, narrowly necessary typed API additions if documented, deterministic fake-executor fixtures with negative controls, real systemd integration qualification, service docs/roadmap/closure handoff.

Out of scope: Core artifact transaction semantics, non-systemd daemon manager changes, any cross-platform API break, arbitrary systemctl string passthrough, blanket `Unknown -> Stopped` mapping, default `reset-failed`, raw kill/cgroup manipulation as a shortcut, changing service configuration/hardening of consumers, registry publication (M011), wg-basic consumer code (separate corrective).

## 6. Ordered work packages

### WP1 — Classify and reproduce

Run the real fixture matrix in §3. Produce a state transition table for `Loaded+Owned` in `active/inactive/failed/activating/deactivating`, for `Foreign`, and for malformed show/is-active output. Identify whether the failure is in the Eggup adapter, its observation schema, the downstream preflight, or a combination. Avoid speculative API additions before this finding.

### WP2 — Minimal safe implementation

- Ensure known `failed` can lead to an owned stop attempt without granting stop authority to truly unknown states. Separate *failure classification* from *ownership proof* even if public `LifecycleState` stays Unknown.
- Prefer preserving `ServiceManager::stop` and its existing public signatures. If the generic enum cannot express the evidence, add a documented additive systemd-specific operation or auxiliary typed observation/proof method; document its exact preconditions and postconditions. Do not add an enum variant or silently treat all Unknown as Stopped.
- Consider final quiescence evidence from successful `systemctl stop`, post-stop `ActiveState=inactive`, lack of an in-flight job, and cgroup/process absence where a failed state can persist. Establish what is actually needed with real-systemd evidence; do not build an unchecked `PID=0` heuristic or hide a failed polling result.
- Re-inspect exact ownership after the mutation and at proof consumption, reject changed unit executable/config and symlink/foreign unit state, and retain an observable recovery-required error on ambiguity.
- Preserve former success semantics for already stopped units, for normal Running -> Stopped, and for dry/foreign/malformed refusals.

### WP3 — Regressions, edge cases, and consumer fixture

Add deterministic FakeExecutor/fake systemctl tests for `failed` with/without safe post-stop result, failed->inactive, failed->still-failed, failed->active restart, changed ExecStart, loaded unknown, missing fields, failed manager process, arbitrary nonzero is-active status, job stuck, timeout, foreign ownership, and untrusted diagnostics. Include a test where a process/cgroup remains after a nominal success, which MUST fail if the evidence protocol requires process proof.

Run at least one real systemd executable fixture: owned candidate fails startup, expected stop/quiescence result, then restart of a known-good unit after a caller-controlled repair. Demonstrate that the new behavior is not a semantic relaxation of Foreign/Unknown mutation.

Add a product-neutral consumer fixture or a short pinned external wg-basic smoke (read-only version/compile + controlled unit) to prove service M010 can be used for failed-candidate rollback; do not block Eggup on completing the whole wg-basic application update.

### WP4 — Security/API review and native portability

Diff public `LifecycleState`, `Ownership`, `ServiceManager` and `TransitionResult` against published service 0.1.2; compile a registry-only external consumer using the required wildcard enum match and reject removed or newly required API. Confirm no service change on macOS launchd, cron, Windows SCM or non-systemd Linux. Keep the dependency graph and Rust 1.89 intact, `unsafe_code = deny`, and no new shell-based command construction.

## 7. Failure, restart, contention and cancellation

A bounded stop which times out or loses ownership must return a refusal/uncertain-state result, never a success. A candidate with a restart loop may change state between observation and mutation; require final revalidation and fail closed if convergence is not proven. Do not kill unrelated processes or alter host-global service-manager state. If observed states are contradictory, report precise safe metadata without invoking an unreviewed new transition. This is a local manager operation, not a binary/DB rollback transaction.

## 8. Compatibility and migrations

If public additions are necessary, add them in a source-compatible fashion with explicit docs and caller migration examples. No modification to `eggup-core` or other crate APIs, no new protocol/schema/lock migrations. A consumer may need to change its own pre-stop guard to permit the newly qualified *specific* Failed case; downstream wg-basic C001a owns that adoption. Do not conflate `LifecycleState::Unknown` with `Ownership::Unknown` in docs or tests.

## 9. Verification

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p eggup-service --all-targets --all-features --locked
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo package -p eggup-service --locked
cargo publish -p eggup-service --dry-run --locked
git diff --check
```

Hosted evidence: Linux Stable, Rust 1.89, macOS, Windows; real disposable systemd fixture plus negative tests. Explicitly record package diff and portable compile. A simulated `systemctl` transcript alone is not sufficient to claim that `failed` is quiescent.

## 10. Documentation

Update `architecture/service-lifecycle.md`, `crates/eggup-service/README.md`, `crates/eggup-service/CHANGELOG.md` Unreleased section, `plans/subsystems/service-lifecycle-roadmap.md` and `plans/registry.md` after implementation. Cross-link `dbowm91/wg-basic` C001a handoff. Preserve historical service M005/M006/M009 closure records.

## 11. Acceptance criteria

1. Exact Owned failed candidates can be stopped/proven quiescent through Eggup without generic downstream systemctl fallback, **or** the documented discriminating fixture proves existing Eggup API is fully sufficient and identifies only the downstream guard as defective; record the narrower conclusion honestly.
2. A failed/foreign/malformed/ambiguous state cannot yield a false completion or mutate a foreign service.
3. Real systemd and deterministic negative-control fixtures pass, including failed->inactive or failed-state persistence as actually observed, and deadline/restart races.
4. An external consumer matching the published non-exhaustive `LifecycleState` with a wildcard still compiles; no platform regressions, new dependency, or privilege escalation.
5. All required current-head CI and Rust 1.89 gates pass, and an exact source/behavior/consumer handoff is recorded.

If M010 closes as `existing API sufficient, no service runtime change`, **do not publish a gratuitous service patch**; mark M011 not needed and unblock the downstream corrective using proven 0.1.2. Otherwise M011 owns the registry release.

## 12. Stop conditions

Stop for design review if proving quiescence requires clearing systemd failure history/rate limit, changing published public enum variants, treating any Unknown as stopped, assuming absent PID proves an empty cgroup, accepting stale ownership after a restart race, breaking service M005/M006 semantics, adding a service-manager escape hatch, or silently skipping a native CI lane.

## 13. Closure evidence

Write `plans/closure/service-lifecycle/010-status.md` only after real implementation evidence. Record baseline/implementation SHAs, exact observed systemd state/transitions, failure detection evidence, old/new behavior with negative controls, service API diff and external consumer compile, real systemd runner/kernel, full hosted run links, package dry-run, unresolved severity, and explicit disposition on the M011 publication requirement.

## 14. Handoff

Eggup Service M011 is operationally blocked on M010's closed, **runtime-changed** disposition. The wg-basic C001a handoff can qualify against a published patched service version after M011, or against the already published 0.1.2 only if M010's evidence proves no Eggup production change is required.
