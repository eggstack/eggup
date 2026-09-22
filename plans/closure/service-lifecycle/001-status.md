# Service Lifecycle M001 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/service-lifecycle/001-manager-neutral-state-and-ownership.md`

Source roadmap: `plans/subsystems/service-lifecycle-roadmap.md#M001--manager-neutral-state-and-ownership`

Reviewed repository baseline: `9f527beb20da585bc3cf56f45f1fd96fd418c124` (plan baseline; implementation ran at transport M002 closure `2147a72` plus service changes)

## Implementation commits/PRs

- M001 service commit (this pass): `feat: add manager-neutral service contract (M001)` (pending SHA; see git log)
- Prior: `2147a72` (acquisition M002 Eggfetch adapter)
- No PR was required for this local implementation pass. No publication was performed.

## Executive finding

M001 is complete. The manager-neutral registration, ownership, state, and lifecycle-snapshot model is defined before any systemd/launchd/cron/SCM adapter. Platform adapters can implement the contract without changing its ownership vocabulary or embedding consumer-specific service definitions.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| `ServiceId` / `ServiceSpec` | Validated id + absolute executable + critical args + optional config; control-char/relative rejection | passed |
| Manager-neutral registration snapshot | `RegistrationSnapshot{present, executable, args, config, malformed}` | passed |
| Lifecycle state vocabulary | `LifecycleState::{Stopped, Running, Transitioning, Unknown}` (`#[non_exhaustive]`), health separate | passed |
| Ownership evidence/result | `Ownership::{Absent, Owned, Foreign, Unknown}` (canonical, shared with core); `snapshot.ownership(spec)` | passed |
| Lifecycle snapshot | `LifecycleSnapshot{id, ownership, state, health, was_registered, was_running}` + `restore_running()` | passed |
| Desired restoration intent | `RestoreIntent::{Preserve, EnsureRunning, EnsureStopped}`; stopped stays stopped by default | passed |
| Manager trait/test double | `ServiceManager{inspect, install, start, stop, restart, uninstall}` + `TestDoubleManager` | passed |
| Health-probe seam | `HealthProbe` (read-only) + `NoProbe`; health never mutates manager state | passed |
| Bounded transition result | `TransitionResult{operation, completed, detail(512)}` | passed |
| Conflict diagnostics | `ServiceError::{InvalidInput, OwnershipDenied, Conflict, Manager}`; `require_owned` guard | passed |
| Exact owned registration | `exact_owned_registration_allows_lifecycle` | passed |
| Same name/different executable → Foreign | `same_name_different_executable_is_foreign` + destructive denied | passed |
| Same executable/different critical args → Foreign | `same_executable_different_critical_args_is_foreign` (documented rule) | passed |
| Malformed → Unknown | `malformed_registration_is_unknown_and_denies_mutation` | passed |
| Absent | `absent_install_then_lifecycle` | passed |
| Stopped/running snapshot | `stopped_and_running_snapshots_preserve_intent` | passed |
| Health failure doesn't mutate manager | `health_failure_does_not_mutate_manager_state` | passed |
| Foreign/unknown destructive denied | `foreign_and_unknown_destructive_operations_are_denied` | passed |
| No transport/update-policy coupling | `cargo tree -p eggup-service` → zero dependencies | passed |

## Public model

`ServiceId`, `ServiceSpec`, `Ownership`, `LifecycleState`, `HealthState`, `RegistrationSnapshot`, `LifecycleSnapshot`, `RestoreIntent`, `ServiceOperation`, `TransitionResult`, `ServiceError`, `HealthProbe`/`NoProbe`, `ServiceManager`, `TestDoubleManager`, `require_owned`.

Ownership rule (documented in `ServiceSpec`): same name + different executable → `Foreign`; same executable + different critical args or config → `Foreign`; malformed/unparseable → `Unknown`; absent → `Absent`; exact match → `Owned`. Only `Owned` authorizes `Start/Stop/Restart/Uninstall`; `Install` allows `Absent` creation or `Owned` refresh; `Foreign`/`Unknown` deny all destructive ops including `Install`.

## Ownership matrix

| Observation | Result |
|---|---|
| Absent | `Absent` |
| Exact executable + args + config | `Owned` |
| Same id, different executable | `Foreign` |
| Same executable, different critical args/config | `Foreign` |
| Malformed record | `Unknown` |
| Missing executable field | `Unknown` |

## Exact tests/commands actually run

Environment: `Darwin 25.6.0 arm64`, stable `1.98.1`, MSRV `1.89.0`.

```text
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo tree -p eggup-service --locked
cargo package -p eggup-service --locked --allow-dirty
./scripts/check-local.sh
```

MSRV variants green. Results: 10/10 `eggup-service` + 13/13 `eggup-eggfetch` + 12/12 `eggup-acquisition` + 37/37 `eggup-core` (72 total). `cargo package -p eggup-service`: 6 files, verification passed. Hosted CI not run locally.

## Invariant review

- Canonical `Absent | Owned | Foreign | Unknown` reused; no new `Managed`/boolean model.
- Destructive ops only for `Owned`; `Foreign`/`Unknown` fail closed (including `Install` on foreign).
- Stopped remains stopped: `RestoreIntent::Preserve` default; no auto-start on inspect/install.
- Manager state vs health separate: `LifecycleState` vs `HealthState`; probe is read-only.
- No updater/release/network policy in crate (zero deps).
- No automatic privilege elevation (no escalation APIs; operations return typed errors).
- Exact executable/config args participate in ownership (spec fields compared).

## Failure/recovery review

All destructive denials return `ServiceError::OwnershipDenied{id, ownership}` without mutation. `Conflict`/`Manager`/`InvalidInput` are bounded (512 chars). Test double preserves `was_running`/`was_registered` intent for later orchestration (M004).

## Compatibility/migration review

New crate; no consumers migrate. Later adapters (M002 systemd/launchd/cron, M003 SCM, M004 orchestration) implement `ServiceManager` without changing ownership vocabulary. If platform differences require incompatible semantics, an ADR is required per the plan's stop condition (not triggered).

## Security review

- No symlink/privilege/escalation surface in M001 (no filesystem mutation beyond in-memory test double).
- Input validation: id length/controls, absolute executable/config, arg controls.
- Bounded diagnostics; no secrets in service specs (executable/args/config only).
- `#![forbid(unsafe_code)]`.

## Docs/operations evidence

- `crates/eggup-service/README.md` + rustdoc contract.
- `cargo doc` clean with `#![deny(missing_docs)]`.
- `cargo tree -p eggup-service`: zero dependencies (no transport/update-policy coupling).

## Unresolved findings with severity

- None (M001 scope). Real adapters, rendering, escalation, and orchestration explicitly deferred to M002-M004.
- Informational: Hosted CI not run locally.

## Disposition and roadmap transition

M001 is closed. Unblocks service M002/M003 adapter work (no detailed plans yet per registry discipline). Consumer adoption is unaffected (service composition awaits M004 orchestration). No ADR was required: systemd/launchd/SCM differences did not force incompatible public semantics at the neutral-model layer.
