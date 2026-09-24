# Service Lifecycle Milestone 005 — Prepared-Transaction Lifecycle Integration

Status: closed; see `plans/closure/service-lifecycle/005-status.md`

Repository baseline reviewed: `bfa01c05e0108eccba9d7d939a19517ef366f1ef`

Source roadmap:

- `plans/subsystems/service-lifecycle-roadmap.md`

Long-term requirements:

- `plans/000-long-term-specification.md#5-transaction-model`
- `plans/000-long-term-specification.md#12-rollback-model`
- `plans/000-long-term-specification.md#13-service-lifecycle`

Applicable ADRs:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`
- `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md`

Hard-dependency evidence:

- Service M004 closure: `plans/closure/service-lifecycle/004-status.md`
- Verified Update Core M007 closure: `plans/closure/verified-update-core/007-status.md`
- Core M007 implementation: `8d5fc12f7224145285f22d0975a7bb91e1e363ea`

Related planning-only cleanup:

- C002: `plans/implementation/planning-closure-hygiene-corrective/002-c001-commit-and-registry-baseline-reconciliation.md` — docs-only, not a runtime/API blocker.

Primary class: capability / infrastructure

## 1. Objective

Add one reusable service-aware update orchestration boundary that composes:

- an already validated `eggup-core::ValidatedTransaction`;
- existing ownership-safe `ServiceManager` mechanics;
- pre-update lifecycle snapshot and safe quiescence;
- Core M007 `ValidatedTransaction::commit_with_post_commit`;
- bounded post-commit lifecycle restoration;
- a caller-owned bounded post-install check;
- explicit `PostCommitFailurePolicy::{KeepInstalled, RollBack}`;
- truthful post-rollback service restoration and composite evidence.

M005 must make the safe cross-platform path reusable without moving release selection, artifact acquisition, application health semantics, service-definition migration, or privilege policy into Eggup.

The core transaction receipt remains authoritative for artifact disposition. The service layer must never flatten `Committed`, `RolledBack`, or `RecoveryRequired` into a generic success/failure boolean.

## 2. Readiness and dependencies

Hard dependencies are closed:

- M004 qualified `ServiceManager` for systemd, launchd, cron/watchdog, and Windows SCM;
- M003 established exact ownership, truthful transitions, and one-deadline-per-manager-operation semantics;
- Core M007 added `commit_with_post_commit`, retains the mutation lock and backup set through one caller callback, and supports explicit keep-installed versus rollback behavior.

The reviewed public Core M007 contract is:

```rust
ValidatedTransaction::commit_with_post_commit(
    ownership,
    PostCommitFailurePolicy,
    check,
) -> eggup_core::Result<TransactionReceipt>
```

The callback runs after the entire new artifact generation is live and while the core mutation lock plus rollback evidence are retained. Core does not impose a timeout on that callback. M005 must therefore provide its own bounded lifecycle/check contract.

C002 is a documentation-only bookkeeping corrective. It should execute before or alongside M005 implementation, but it does not block coding or API design.

## 3. Current evidence

At the reviewed baseline, `eggup-service` already provides:

- `ServiceSpec`;
- `Ownership::{Absent, Owned, Foreign, Unknown}`;
- `LifecycleState::{Stopped, Running, Transitioning, Unknown}`;
- `HealthState::{Healthy, Degraded, Unknown}`;
- `LifecycleSnapshot` with `was_registered` / `was_running`;
- `RestoreIntent::{Preserve, EnsureRunning, EnsureStopped}`;
- `TransitionResult`;
- `ServiceManager::{inspect, install, start, stop, restart, uninstall}`;
- deterministic `TestDoubleManager`;
- qualified systemd, launchd, cron, and Windows SCM adapters;
- private monotonic deadline machinery used by platform transitions.

The current `HealthProbe` is read-only but has no timeout parameter:

```rust
fn check(&self, spec: &ServiceSpec) -> HealthState
```

It is suitable for passive observation but cannot, by itself, satisfy M005's bounded post-install requirement. M005 must not place an arbitrary unbounded `HealthProbe::check` inside Core M007's callback and then claim the callback is bounded.

The reviewed `eggup-service` manifest has no dependency on `eggup-core`. M005 is the first service/core composition point.

## 4. Invariants

### Artifact/transaction invariants

- `TransactionReceipt` remains the source of truth for artifact disposition.
- Service orchestration never reimplements backup, artifact rollback, recovery paths, or mutation locking.
- Core M007 `commit_with_post_commit` is the only post-commit rollback mechanism used.
- `RecoveryRequired` is never converted to an ordinary service error or success.
- A service failure and an artifact rollback failure remain separately observable.

### Ownership/lifecycle invariants

- M005 mutates only an `Owned` registration.
- `Absent`, `Foreign`, `Unknown`, `Transitioning`, and manager-unknown preconditions fail before artifact mutation.
- Same service name/path existence alone never grants ownership.
- A service that was running is quiesced before artifact replacement.
- A service that was stopped is not started merely because an update succeeded when `RestoreIntent::Preserve` is selected.
- On a failed/rolled-back update, restore the **pre-update lifecycle state**, not a new success-only `RestoreIntent`.
- No automatic registration create/refresh occurs in M005.
- No hidden elevation.

### Post-commit/rollback invariants

- Success-only restoration policy is explicit through `RestoreIntent`.
- Caller-selected `PostCommitFailurePolicy` is fixed before mutation begins.
- Under `RollBack`, a failed post-install check must attempt to quiesce any newly started service before returning failure to Core M007, so artifact rollback is not knowingly attempted under a running new generation.
- If rollback succeeds, M005 attempts to restore the old service's pre-update running/stopped state.
- If Core reports `RecoveryRequired`, M005 does not automatically start/restart the service against an uncertain artifact generation.
- If post-rollback lifecycle restoration fails, the artifact receipt remains `RolledBack`; the separate lifecycle failure is preserved.
- If `KeepInstalled` is selected and post-commit work fails, the verified new generation remains installed exactly as Core reports; M005 must not silently roll it back afterward.

### Boundedness/panic invariants

- Manager transitions use bounded deadlines.
- Post-install checks receive an explicit remaining-time budget and are contractually required to self-bound.
- M005 must not use an unbounded worker thread as fake cancellation.
- Caller-check panics inside the post-commit orchestration are caught early enough that `RollBack` can still attempt service quiescence before control returns to Core M007.
- All service-owned diagnostics are bounded and contain no secrets.

## 5. Scope and non-scope

### In scope

- direct `eggup-core` composition from `eggup-service`;
- manager-neutral update orchestration API;
- pre-update ownership/state snapshot;
- safe quiescence of a running owned service;
- success-only `RestoreIntent`;
- bounded post-install check contract;
- Core M007 post-commit policy wiring;
- rollback-time service quiescence;
- post-artifact-rollback restoration of the pre-update service state;
- composite lifecycle/update receipt;
- structured lifecycle failure evidence;
- deterministic fault matrix;
- existing-platform CI/package/dependency qualification;
- docs/roadmap/registry/closure.

### Out of scope

- release/version selection;
- network acquisition;
- archive extraction;
- candidate validation policy;
- authenticity/signatures;
- installing a service when registration is absent;
- automatic `ServiceManager::install` refresh after an update;
- changing service definitions/argv/config as part of the transaction;
- consumer-specific migrations;
- application-specific health payload parsing;
- bootstrap installers;
- background auto-update;
- privilege escalation;
- persistent crash journal or process/power-loss recovery beyond Core M007's existing claim;
- native privileged mutation of real system service managers solely for M005 tests;
- publication.

If a consumer needs service-definition migration together with artifact update, it remains explicit consumer policy until separate evidence justifies a generic contract.

## 6. Required production changes

### A. Add the Core composition dependency

Add `eggup-core` to `eggup-service` using the workspace path/version convention, for example:

```toml
eggup-core = { version = "0.1.0", path = "../eggup-core" }
```

Before closure:

- inspect `cargo tree -p eggup-service --locked` on the host and Windows target;
- confirm no HTTP/TLS/acquisition dependency enters `eggup-service`;
- record the dependency delta;
- confirm `eggup-core` remains independent of `eggup-service` (no cycle).

Do not introduce acquisition or Eggpack dependencies.

### B. Introduce a manager-neutral lifecycle update policy

Add a small policy object, naming may vary, that contains only mechanism-level choices needed before mutation.

Conceptually:

```rust
pub struct LifecycleUpdatePolicy {
    pub restore: RestoreIntent,
    pub post_commit_failure: PostCommitFailurePolicy,
    pub quiesce_timeout: Duration,
    pub post_commit_timeout: Duration,
    pub rollback_restore_timeout: Duration,
}
```

Exact field structure may differ, but the public contract must make these facts explicit:

- successful-update desired lifecycle state;
- keep-installed versus rollback choice;
- bounded pre-commit quiescence;
- bounded post-commit restore/check budget;
- bounded restoration after successful artifact rollback.

Validate zero/overlong durations consistently with existing service limits.

Do not add product names, release IDs, URLs, or health endpoint policy.

### C. Add a bounded caller-owned post-install check seam

Do not reuse the existing unbounded `HealthProbe` as the sole M005 contract.

Add a bounded check seam, either a trait or generic callback. Preferred shape:

```rust
pub trait PostInstallCheck: fmt::Debug {
    fn check(
        &self,
        spec: &ServiceSpec,
        snapshot: &LifecycleSnapshot,
        remaining: Duration,
    ) -> Result<(), PostInstallCheckError>;
}
```

Requirements:

- receives a remaining-time budget;
- returns bounded, non-secret failure evidence;
- application decides what "healthy enough" means;
- may internally use HTTP, IPC, a process probe, or existing `HealthProbe` semantics supplied by the consumer;
- `eggup-service` itself gains no network stack;
- include a no-op implementation for consumers that need only lifecycle restoration.

The trait/callback contract must explicitly state that the implementation must honor the supplied budget. M005 cannot forcibly cancel arbitrary synchronous user code without unsafe/unbounded worker semantics.

Retain `HealthProbe` for its existing read-only observation role unless a separate compatibility review justifies changing it.

### D. Add lifecycle orchestration phases/evidence

Add a service-owned phase vocabulary sufficient to explain failures without scraping strings.

Conceptually:

```rust
pub enum LifecycleUpdatePhase {
    Inspect,
    Quiesce,
    Commit,
    RestoreNew,
    PostInstallCheck,
    QuiesceForRollback,
    RestoreOld,
    FinalInspect,
}
```

Add a bounded structured failure report containing:

- phase;
- service id;
- bounded detail;
- optional observed ownership/state where relevant.

Do not duplicate Core `FailureReport`; service and artifact failure domains remain distinct.

### E. Add a composite lifecycle update receipt

Return a receipt that preserves Core evidence intact.

Conceptually it must expose:

- pre-update `LifecycleSnapshot`;
- optional/final `LifecycleSnapshot`;
- exact `TransactionReceipt`;
- whether pre-commit quiescence occurred;
- post-commit lifecycle/check failure, if any;
- rollback-time quiescence failure, if any;
- post-artifact-rollback lifecycle restoration result/failure, if any.

Do not invent an overall boolean that hides the transaction disposition.

Useful query methods may answer:

- artifact disposition;
- whether service state was restored;
- whether manual artifact recovery is required;
- whether the new generation was retained despite a service/check failure.

### F. Add one non-escaping orchestration entrypoint

Prefer one function/method that owns the full service/transaction sequence so a stopped pre-update service is not accidentally leaked through a public half-orchestrated handle.

Conceptual shape:

```rust
pub fn commit_with_lifecycle<M, C>(
    manager: &mut M,
    spec: &ServiceSpec,
    transaction: ValidatedTransaction,
    ownership: CommitOwnership<'_>,
    policy: LifecycleUpdatePolicy,
    check: &C,
) -> Result<LifecycleUpdateReceipt, LifecycleUpdateError>
where
    M: ServiceManager,
    C: PostInstallCheck;
```

Naming may differ.

Do not expose a public "quiesced but not committed" object unless its Drop/cancellation semantics can provably restore state. The preferred design is non-escaping orchestration.

### G. Preflight ownership and state

Before stopping anything or invoking Core:

1. `manager.inspect(spec)`;
2. require `Ownership::Owned`;
3. require `LifecycleState::Running` or `Stopped`;
4. reject `Transitioning` and `Unknown`;
5. retain the snapshot as immutable pre-update evidence.

`Absent` is intentionally not auto-installed by M005.

If future install-time service creation needs transaction composition, plan it separately.

### H. Quiesce before artifact commit

If the pre-update state is `Running`:

- call `stop` with the configured quiesce budget;
- require `TransitionResult.completed == true`;
- re-inspect;
- require ownership still `Owned`;
- require state `Stopped`.

If the pre-update state is already `Stopped`, do not issue a redundant stop.

If quiescence fails or ownership changes, do not invoke Core and return structured lifecycle evidence.

### I. Compose Core M007

Invoke `ValidatedTransaction::commit_with_post_commit` with the exact caller-selected `PostCommitFailurePolicy`.

The callback owns only post-commit service work. It does not gain artifact mutation authority.

Inside the callback, create one monotonic deadline from `post_commit_timeout` and consume remaining budget across:

1. success-only lifecycle restoration;
2. final manager observation;
3. caller-owned post-install check;
4. if needed under `RollBack`, service quiescence before returning failure to Core.

Do not reset the full timeout for each substep.

### J. Successful-update lifecycle restoration

Derive the desired successful state:

- `RestoreIntent::Preserve`:
  - pre-running -> Running;
  - pre-stopped -> Stopped;
- `EnsureRunning` -> Running;
- `EnsureStopped` -> Stopped.

Because M005 quiesces a pre-running service before commit:

- desired Running -> call `start`, not `restart`;
- desired Stopped -> confirm stopped and do not start.

After transition:

- re-inspect;
- require `Owned`;
- require desired state;
- then run the caller post-install check, if configured.

Do not automatically call `install`/refresh.

### K. Post-install failure under KeepInstalled

On manager restoration or caller-check failure with `KeepInstalled`:

- record structured service failure;
- return failure from the Core callback;
- do not perform artifact rollback;
- do not attempt a second hidden service policy after Core returns;
- preserve the Core receipt showing `Committed` plus `post_commit_failure`.

A final read-only inspect may be recorded if it fits the existing bounded budget or occurs after Core under an explicit small bound; do not claim service state if it cannot be observed.

### L. Post-install failure under RollBack

Before returning failure to Core:

- if the newly installed service is running or may be running, attempt to stop it using the **remaining post-commit deadline**;
- confirm stopped when possible;
- record any quiesce-for-rollback failure separately;
- return the original post-install failure to Core so Core applies artifact rollback.

If the stop fails, still return failure to Core; Core's resulting artifact disposition may become `RecoveryRequired`, especially on platforms that cannot replace a running executable. Preserve both service and Core recovery evidence.

Do not silently switch a caller's `RollBack` policy to `KeepInstalled`.

### M. Panic handling inside post-commit service work

Core M007 catches panics in its callback, but a raw panic would bypass M005's stop-before-rollback preparation.

Therefore M005 must catch unwind around caller-controlled post-install check execution (and, if practical, the service post-commit composition block) before handing failure back to Core.

On panic:

- convert to bounded lifecycle failure;
- under `RollBack`, attempt the same quiesce-for-rollback path;
- then return ordinary callback failure so Core resolves the selected policy.

Core's own panic catch remains the final safety net, not the primary M005 control path.

No panic payload contents are exposed.

### N. Core pre-receipt error after service quiescence

If Core returns `Err` rather than a `TransactionReceipt` after M005 has stopped a previously running service:

- treat artifact mutation as not authoritatively committed;
- attempt to restore the **pre-update lifecycle state** within `rollback_restore_timeout`;
- return a structured `LifecycleUpdateError` containing the Core error plus any service-restoration failure.

Do not apply success-only `RestoreIntent` in this path.

Before implementation, audit Core M007 to document which commit paths can return `Err` versus a terminal receipt; tests must cover the reachable pre-receipt cases.

### O. Core RolledBack result

Whenever Core returns `TransactionDisposition::RolledBack`:

- artifacts are back to the old generation according to the Core receipt;
- restore the **pre-update service state**, regardless of success-only `RestoreIntent`;
- pre-running -> start old generation and confirm Running;
- pre-stopped -> confirm/remain Stopped;
- record restoration result separately.

If old-service restoration fails:

- keep Core receipt as `RolledBack`;
- report lifecycle restoration failure;
- do not relabel artifact disposition as `RecoveryRequired` unless Core did so.

### P. Core RecoveryRequired result

If Core returns `RecoveryRequired`:

- do not automatically start/restart a service against uncertain artifact state;
- preserve the Core recovery path/failure evidence;
- optionally perform read-only `inspect`;
- mark lifecycle restoration incomplete/not attempted due artifact recovery requirement.

This is a high-severity composite terminal state.

### Q. Final observation

For ordinary committed success and successful rolled-back restoration, perform a final read-only `inspect` and preserve it in the composite receipt.

Do not turn inability to perform a nonessential final read-only observation into false artifact rollback.

Classify it separately as lifecycle evidence degradation/failure.

## 7. Ordered work packages

A. Add `eggup-core` dependency to `eggup-service`; qualify dependency direction/tree.

B. Define lifecycle-update policy, phase/failure evidence, bounded post-install-check seam, and composite receipt.

C. Implement preflight ownership/state validation and running-service quiescence.

D. Implement non-escaping Core M007 composition with shared post-commit deadline.

E. Implement success-only `RestoreIntent` restoration and bounded caller check.

F. Implement `KeepInstalled` failure semantics.

G. Implement `RollBack` stop-before-artifact-rollback behavior and post-rollback old-service restoration.

H. Implement Core `Err` and `RecoveryRequired` service-state semantics.

I. Add panic/fault coverage and deterministic scripted/faulting manager support in tests.

J. Run package/workspace/MSRV/platform qualification and document dependency impact.

K. Write closure; update roadmap/registry; identify whether a real service-bearing consumer adoption is newly ready.

## 8. Failure/restart/contention matrix

At minimum, implement and test this matrix.

| Pre-state / event | Artifact result | Required service behavior |
|---|---|---|
| Owned + stopped, Preserve, success | Committed | remains stopped |
| Owned + running, Preserve, success | Committed | stop old -> install -> start new -> confirmed running |
| Owned + stopped, EnsureRunning, success | Committed | start new -> confirmed running |
| Owned + running, EnsureStopped, success | Committed | stop old -> install -> remains stopped |
| Foreign/Unknown/Absent preflight | no Core mutation | fail closed; no service mutation |
| Transitioning/Unknown state | no Core mutation | fail closed |
| pre-quiesce incomplete/error | no Core mutation | structured lifecycle failure |
| Core pre-receipt error after quiesce | no authoritative commit | restore pre-update service state |
| Core returns RolledBack before post-check | RolledBack | restore pre-update service state |
| post-restore failure + KeepInstalled | Committed + post-commit failure | retain new artifacts; preserve service failure |
| health/check failure + KeepInstalled | Committed + post-commit failure | retain new artifacts; no hidden rollback |
| health/check failure + RollBack; stop succeeds | RolledBack | core restores old artifacts; then restore pre-update service state |
| check panic + RollBack | RolledBack or RecoveryRequired | panic bounded; stop-before-rollback attempted |
| stop-before-rollback fails | RolledBack or RecoveryRequired per Core | preserve stop failure + Core evidence |
| artifact rollback succeeds, old-service start fails | RolledBack | report separate lifecycle restore failure |
| artifact rollback fails | RecoveryRequired | do not auto-start; preserve recovery evidence |
| final inspect fails after committed success | Committed | artifact success retained; report observation/lifecycle evidence failure separately |

Manager contention/ownership change at any reinspection fails closed.

## 9. Compatibility and migration

M005 is additive at the Eggup API level except for the new normal `eggup-core` dependency of `eggup-service`.

Do not break existing:

- `ServiceManager`;
- platform adapters;
- `HealthProbe`;
- `RestoreIntent`;
- existing service registration/install behavior.

If an implementation discovers that a breaking change to those contracts is unavoidable, stop and write a corrective/ADR rather than silently changing qualified M001-M004 semantics.

No consumer is migrated in M005 itself.

The lockstep 0.1.1 publication remains a separate operational qualification/publication task.

## 10. Tests

### API/policy

- zero/overlong lifecycle budgets rejected;
- `RestoreIntent` mapping for all pre-state combinations;
- absent/foreign/unknown/transitioning preconditions fail before transaction;
- no automatic `install` call.

### Quiescence

- running owned service stopped once and confirmed before Core commit;
- stopped owned service not redundantly stopped;
- stop error/incomplete prevents Core invocation;
- ownership drift after stop prevents Core invocation.

### Successful commit

- Preserve running -> running;
- Preserve stopped -> stopped;
- EnsureRunning -> running;
- EnsureStopped -> stopped;
- bounded post-install check sees complete new multi-member generation;
- final snapshot recorded.

### KeepInstalled

- start failure retained as committed transaction + separate lifecycle failure;
- caller-check failure retained as committed + Core post-commit report;
- check panic converted to bounded failure;
- no artifact rollback attempted by service layer.

### RollBack

- failed check while new service running causes stop before callback failure returns to Core;
- successful artifact rollback restores old files;
- pre-running old service restarted after rollback;
- pre-stopped old service stays stopped;
- post-rollback service start failure remains separate from Core RolledBack receipt;
- injected artifact rollback failure -> RecoveryRequired and no automatic restart;
- stop-before-rollback failure evidence preserved;
- caller-check panic still attempts rollback quiescence.

### Core error boundary

- lock/preflight Core error after service quiescence restores old lifecycle state;
- restoration failure preserves both causes;
- no success-only `EnsureRunning`/EnsureStopped policy applied after failed transaction.

### Deadlines

- one post-commit deadline shrinks across start + inspect + check + rollback-quiesce;
- exhausted budget does not reset for a later substep;
- caller check receives remaining budget;
- tests prove no hidden second full timeout.

### Platform regression

Existing:

- systemd;
- launchd;
- cron/watchdog;
- Windows SCM fake/backend;
- Windows cross-check;
- macOS tests

must remain green.

M005 orchestration correctness is primarily manager-neutral and should be exhaustively covered with deterministic test managers; do not require privileged real-service mutation merely to close this composition milestone.

## 11. Verification commands

Adapt to the repository's current scripts, at minimum:

```text
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p eggup-service --all-targets --all-features --locked
cargo test -p eggup-core --all-targets --all-features --locked
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --all-features --no-deps
cargo +1.89.0 check --workspace --all-targets --locked
cargo +1.89.0 test -p eggup-service --all-targets --locked
cargo tree -p eggup-service --locked
cargo tree -p eggup-core --locked
cargo check -p eggup-service --all-targets --locked --target x86_64-pc-windows-msvc
git diff --check
scripts/check-local.sh
```

Hosted CI must record separately:

- stable Linux/default checks;
- Rust 1.89;
- macOS workspace tests;
- Windows workspace check.

Do not infer native Windows service-runtime behavior from compile-only CI.

## 12. Documentation updates

Update:

- `crates/eggup-service/README.md`;
- public rustdoc for orchestration policy/check/receipt;
- `plans/subsystems/service-lifecycle-roadmap.md`;
- `plans/registry.md`;
- matching M005 closure record.

Document prominently:

- lifecycle orchestration requires an existing Owned registration;
- stopped-stays-stopped under Preserve;
- `RestoreIntent` applies to successful new-generation restoration only;
- rollback restores pre-update lifecycle state;
- RecoveryRequired suppresses automatic restart;
- bounded post-install checks must honor supplied remaining budget;
- existing `HealthProbe` alone is not a hard timeout mechanism;
- no registration migration/refresh occurs automatically.

## 13. Acceptance criteria

M005 closes only when:

- `eggup-service` composes directly with qualified Core M007 without copying transaction rollback;
- a running owned service is quiesced before artifact mutation;
- stopped Preserve remains stopped;
- successful updates restore the caller-selected success state;
- post-install manager/check failure follows explicit KeepInstalled/RollBack policy;
- RollBack failure path attempts service quiescence before Core artifact rollback;
- successful artifact rollback restores the old pre-update lifecycle state;
- RecoveryRequired never auto-starts a service;
- service failure evidence remains distinct from Core transaction evidence;
- post-install check contract is explicitly bounded by caller-supplied remaining time;
- panic handling does not bypass rollback quiescence for caller-controlled checks;
- no automatic service install/refresh, acquisition, release, or privilege policy enters the layer;
- existing M001-M004 platform adapter behavior remains qualified;
- Rust 1.89, package/docs, and hosted CI pass;
- no medium-or-higher correctness/security finding remains open.

## 14. Stop conditions

Stop and write a corrective/ADR if:

- Core M007 cannot provide enough information to restore lifecycle truthfully after its terminal receipts;
- a public half-orchestrated handle is required and safe Drop/cancellation semantics cannot be proven;
- service rollback requires duplicating artifact backup/restore in `eggup-service`;
- RollBack cannot quiesce the new process before artifact restore on a supported manager without a new core handshake;
- adding `eggup-core` to `eggup-service` creates a dependency cycle or unacceptable feature/dependency contamination;
- bounded post-install work would require pretending an arbitrary synchronous callback can be forcibly cancelled;
- service-definition migration proves necessary for correctness rather than caller policy;
- a qualified M001-M004 public contract must be broken.

## 15. Closure evidence required

Record:

- exact implementation SHA(s);
- exact `eggup-core` / `eggup-service` dependency relationship and tree delta;
- public API added;
- preflight ownership/state matrix;
- quiesce matrix;
- success RestoreIntent matrix;
- KeepInstalled matrix;
- RollBack + stop-before-rollback matrix;
- Core `Err`, RolledBack, RecoveryRequired behavior;
- post-rollback lifecycle restoration evidence;
- bounded-check/deadline evidence;
- panic evidence;
- test counts;
- Rust 1.89 results;
- Linux/macOS/Windows hosted results with native-vs-compile limits stated honestly;
- package/docs results;
- security review;
- unresolved findings with severity;
- exact downstream transition: which consumer adoption, if any, becomes ready.

## 16. Handoff notes

One implementation plan is sufficient for M005.

The platform adapters are already separate qualified milestones; splitting M005 by operating system would duplicate orchestration semantics and risk drift. Implement the manager-neutral orchestration once, reuse the existing adapters, and qualify it with deterministic fault matrices plus the existing platform CI lanes.

Do not migrate eggsearch/Gregg or publish crates in this milestone. Consumer adoption is a separate next dependency transition after M005 closes.
