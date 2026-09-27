# Service Lifecycle M008 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/service-lifecycle/008-operation-deadline-test-determinism-corrective.md`

Source roadmap: `plans/subsystems/service-lifecycle-roadmap.md#M008--OperationDeadline-test-determinism-corrective`

Original closed work affected:

- `plans/implementation/service-lifecycle/003-unix-adapter-correctness-security-corrective.md`
- `plans/closure/service-lifecycle/003-status.md`

Related later closure evidence:

- `plans/closure/service-lifecycle/007-status.md` (line 68 already records this same test flaking on a prior macOS attempt with a 1 ms deadline while the other 100 service tests passed)

Reviewed source baseline: `510111fa67bafd0e188dad744eed4d224a015b76` (C006 closure head; source-plan baseline). Work began from the C007 closure head `d7bd539a80b665531d4fe0223fa34a03819fa21f`; every commit between the plan baseline and the work head is planning/registry/roadmap text only.

Implementation commits:

- `0b0cdaafb308e4b27b4ecfa6b7144e90c84a488b` — deterministic injected-`Instant` deadline seam (`new_at`, `remaining_at`, `command_timeout_at`) plus exact-boundary unit proof; production wrappers unchanged. Single-file change: `crates/eggup-service/src/lib.rs` (+46/−9).
- This closure batch (docs/planning only: closure record + M008 plan status + service roadmap + M001d rebase/unblock + registry; head recorded in registry as the M008 closure head)

Original failure (superseded): hosted run `36261671380` — macOS job failed only `unix_tests::transition_deadline_rejects_zero_and_only_shrinks` with `Manager("transition deadline exhausted before manager command")`. Stable Linux, Rust 1.89 MSRV, and Windows passed. The 80 ms real deadline plus 10 ms sleep plus unconditional second `unwrap()` allowed hosted scheduler delay to consume the full budget before the second read; the production helper correctly returned the exhaustion error.

Hosted final qualification: CI run [36332823865](https://github.com/eggstack/eggup/actions/runs/36332823865) on `0b0cdaa`; Stable Linux, Rust 1.89 MSRV, macOS, and Windows all passed (conclusion `success` on all four jobs). macOS log confirms `test unix_tests::transition_deadline_rejects_zero_and_only_shrinks ... ok` inside the green 101/101 service suite.

## Executive finding

`OperationDeadline` production semantics are byte-for-byte unchanged (same validation, same monotonic-clock wrappers, same bounds, same error text); only the unit proof changed from wall-clock sleep to injected-`Instant` arithmetic. The red baseline is restored to green with no deadline widening, no tolerance, no retry, and no API change.

M008 is closed. No medium-or-higher service finding remains open. Archive M001d is rebased to the new green head and restored to executable-ready (see Roadmap disposition).

## Requirement-to-evidence matrix

| Requirement (source plan Section 10) | Evidence | Result |
|---|---|---|
| zero transition budget rejected | `OperationDeadline::new(Duration::ZERO).is_err()` retained in rewritten test | passed |
| budget above `MAX_TRANSITION_TIMEOUT` rejected | new assertion: `new(MAX + 1s).is_err()` | passed |
| deterministic exact remaining at creation | `remaining_at(start) == Some(80 ms)` | passed |
| deterministic exact shrink at supplied instants | `remaining_at(start+10 ms) == Some(70 ms)`; `command_timeout_at` equals `70 ms` and is `< 80 ms` | passed |
| 1 ms boundary before expiry | `remaining_at(start+79 ms) == Some(1 ms)` | passed |
| exactly-at-deadline exhaustion | `command_timeout_at(start+80 ms).is_err()` | passed |
| after-deadline exhaustion | `command_timeout_at(start+81 ms).is_err()` | passed |
| no sleep in the arithmetic proof | rewritten test contains no `thread::sleep`; 10/10 local repeat runs green | passed |
| existing manager deadline tests green | full `eggup-service` suite 101/101 (all pre-existing transition/polling tests untouched) | passed |
| full service + workspace suites green | `cargo test -p eggup-service --all-targets` 101/101; `scripts/check-local.sh` exit 0 (18 `ok` suites, zero failures) | passed |
| Rust 1.89 holds | `cargo +1.89.0 check --workspace --all-targets --locked` passed; no new language features or dependencies | passed |
| macOS hosted lane passes formerly flaky test | run `36332823865` macOS job `success`; log shows the test `... ok` | passed |
| Windows service tests green | run `36332823865` Windows job `success` | passed |
| Stable Linux + docs green | run `36332823865` Stable job `success`; local `cargo doc` passed | passed |
| public API/dependencies unchanged | no new `pub` item in diff; no `Cargo.toml`/`Cargo.lock` delta; `cargo tree` review clean | passed |

## Production implementation and audit evidence

- `crates/eggup-service/src/lib.rs` (`OperationDeadline`, still private `struct OperationDeadline(Instant)`):
  - `new(timeout)` delegates to `new_at(Instant::now(), timeout)`; validation (`zero` / `> MAX_TRANSITION_TIMEOUT` → `invalid("transition timeout out of range")`) and `now + timeout` construction are unchanged in behavior.
  - `remaining()` delegates to `remaining_at(Instant::now())`; `checked_duration_since` + nonzero filter unchanged.
  - `command_timeout()` delegates to `command_timeout_at(Instant::now())`; exhaustion error text `manager("transition deadline exhausted before manager command")` unchanged.
  - New private helpers `new_at` / `remaining_at` / `command_timeout_at` take an explicit `Instant`; all production callers use the `Instant::now()` wrappers. No fake-clock framework or dependency.
- Exact private helper diff: `git diff d7bd539..0b0cdaa -- crates/eggup-service/src/lib.rs` (one file, +46/−9; see matrix for semantics).
- Public API proof: `git diff` `+pub` grep empty; no `Cargo.toml`/`Cargo.lock`/workflow delta in the implementation commit.
- No production scheduling slack, tolerance, retry, renewal, default, cadence, error-category, lifecycle, or disposition change. Remaining `thread::sleep` sites are the pre-existing bounded manager-poll loops and fake-executor simulation, untouched per source plan Section 6.3.

## Exact commands and results

Local Darwin arm64 (on `0b0cdaa`):

```text
cargo fmt --all -- --check                                      passed
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings  passed
cargo test -p eggup-service --lib transition_deadline_rejects_zero_and_only_shrinks --locked  passed 10/10 repeat runs
cargo test -p eggup-service --all-targets --all-features --locked  passed (101/101)
cargo test --workspace --all-targets --all-features --locked    passed (all suites green)
cargo test -p eggup-curl --all-targets --all-features --locked  passed 5/5 repeat runs (22/22; see flakes note)
cargo doc --workspace --no-deps --locked                        passed
cargo +1.89.0 check --workspace --all-targets --locked           passed
./scripts/check-local.sh                                         exit 0 (18 ok suites, zero failures/errors)
git diff --check                                                passed
```

Hosted final run 36332823865 (on `0b0cdaa`):

- Stable checks: success (fmt, clippy, full workspace tests, docs).
- MSRV check: success (Rust 1.89 workspace all-target check).
- macOS tests: success (full workspace tests; formerly flaky test `... ok`; service 101/101).
- Windows archive, acquisition, and service tests and check: success.

Flakes note: the first local full-workspace run after the fix showed one transient `eggup-curl` failure (21/22) that passed in isolation immediately after and in all 5 subsequent repeat runs plus 2 full-workspace reruns. The M008 diff touches only `crates/eggup-service/src/lib.rs` and shares no code path with curl; the curl suite is timing-sensitive (sub-second child-process tests) and unrelated to this corrective. No repeated failure was observed. M008 MUST NOT have closed on a rerun of the old flaky test — it did not: the sleep-based assertion is removed and replaced by the deterministic proof above.

## Invariant review

- One absolute operation deadline still governs the whole manager transition; every manager command still receives only the remaining duration.
- Manager command timeouts only shrink (proven by exact arithmetic: 80 ms → 70 ms → 1 ms → error).
- Expired deadlines still fail before a new manager command starts (`command_timeout_at` errors at/after the deadline; no retry/renew/clamp).
- Zero timeout remains invalid; `MAX_TRANSITION_TIMEOUT` remains enforced.
- No production scheduling slack or tolerance added; no exhausted deadline renewed or extended.
- systemd/launchd/cron/SCM lifecycle behavior unchanged; public API unchanged; service error text/bounds unchanged.
- Rust 1.89 supported; no new dependency; no first-party unsafe.

## Failure and recovery review

No production failure semantics changed. An exhausted deadline remains a hard pre-command manager error. Test infrastructure adds no sleeps, polling, or retries that could hide a real deadline failure — the new test fails deterministically if arithmetic ever returns a non-shrinking or non-exhausted value.

Stop-condition check (source plan Section 14): deterministic injected-time tests confirm production arithmetic is correct; no manager transition can start after the absolute deadline; no deadline widening was required; no platform-specific `Instant` anomaly appeared (macOS green); hosted macOS passes with the sleep removed. No broader service corrective is needed. A genuine runtime failure was not misclassified as flakiness: the failure mode (valid exhaustion error under descheduling) proves the test construction, not the deadline model, was at fault.

## Compatibility and migration review

No public API, dependency, feature, binary, package, or consumer behavior change. All new helpers are private to `eggup-service`. No consumer migration. No changelog entry: runtime behavior and public API are unchanged, so per source plan Section 12 this is recorded as test/qualification hardening, not a service behavior change.

## Security review

The change narrows no security boundary and widens none: deadline enforcement is identical, exhaustion still fails closed before any manager mutation, and the deterministic test cannot mask a real expiry (at/after-deadline assertions require the error). No new attack surface, secret flow, or privilege path. No medium-or-higher finding remains open.

## Unresolved findings

- None: no high- or medium-severity service issue remains. Low/informational: one transient local `eggup-curl` timing flake (see Flakes note), unrelated to this corrective and green on all reruns plus hosted lanes.

## Roadmap disposition and downstream unblock

Service Lifecycle M008 moves from ready to closed. Future-plan triage (unblocking review):

- **Archive M001d**: contract-ready / execution-blocked on M008 → **ready for handoff** (unblocked by this closure). Rebases from planning-analysis baseline `f463aa7` to the proven-green head `0b0cdaafb308e4b27b4ecfa6b7144e90c84a488b` (hosted run `36332823865` green on all lanes); header, Sections 1–2, and roadmap/registry blockers updated here. Docs-only C007/M008 closure commits between the carried-qualification heads preserve runtime qualification (no `.rs`/`Cargo.toml`/`Cargo.lock`/workflow delta — verified by `git diff --name-only`).
- **Consumer Adoption M006 Egress**: **remains blocked** on M001d closure + green hosted qualification. No implementation plan exists yet; authoring it now would violate the M001d gate, so no status change.
- **Eggpack Interoperability M002**: **remains blocked** on M001d closure + green hosted qualification. No implementation plan exists yet; same gate applies, so no status change.
- **Gregg M004**: writable but intentionally unwritten per separate authoring decision; M008 does not change its prerequisites or authorize migration, so no status change.
- **Eggpack M003/M004**: independently blocked on producer-owned conventions; unaffected by M008, so no status change.
- No other plan becomes dependency-ready as a result of M008. M001d is the single unblocked plan; both requested plans (C007, M008) are now closed, so no further eligible-plan continuation is required.

## Registry updates

- Service Lifecycle M008: ready → closed (hosted run `36332823865` green; supersedes `36261671380`).
- Archive M001d: contract-ready / execution-blocked → ready for handoff (rebased to `0b0cdaa` + run `36332823865`).
- Service lifecycle roadmap: M001-M008 closed; no next implementation milestone registered.
- Top metadata: current head green (`36332823865` all lanes); `36261671380` preserved only as superseded context.
- Execution graph, subsystem table, current-state, and next-handoff rows reconciled to the closed-M008 / ready-M001d graph. Egress M006 and Eggpack M002 stay blocked on M001d.
