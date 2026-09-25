# Service Lifecycle Milestone 006 — Daemon Update Disposition and Reference Qualification

Status: implemented (closed; see `plans/closure/service-lifecycle/006-status.md`)

Repository baseline: `881c95ff069d3d465a282cb6a495ba6fcb70cb6f`

Reference implementation reviewed: `eggstack/gregg@8b18f9ee16461e3fa0ef0d804ed39ebb9183b727` (`crates/greggd/src/update.rs`, `crates/greggd/src/startup/*`, and `crates/greggd/src/service/*`). Gregg is reference/test evidence only; this milestone MUST NOT modify, migrate, or depend on Gregg.

Source roadmap:

- `plans/subsystems/service-lifecycle-roadmap.md`

Long-term requirements:

- `plans/000-long-term-specification.md#10-ownership-model`
- `plans/000-long-term-specification.md#12-rollback-model`
- `plans/000-long-term-specification.md#13-service-lifecycle`
- `plans/000-long-term-specification.md#20-platform-scope`
- `plans/001-terminology-and-domain-model.md`

Applicable ADRs:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`
- `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md`

Primary class: capability

## 1. Objective

Generalize Eggup's service-aware update orchestration so daemon update decisions distinguish artifact mutation authority from service-manager mutation authority and can represent the mature runtime dispositions demonstrated by greggd:

- managed and running;
- managed and stopped;
- directly/externally running outside the owned manager path;
- stopped;
- foreign-manager-preserved.

Add a pure, testable disposition/revalidation boundary plus the minimum generic orchestration needed to preserve these states around an already validated Eggup transaction.

Use greggd's decision matrices as reference fixtures. Do not change Gregg and do not create a compile-time/runtime dependency on it.

## 2. Why this milestone is ready

Service M001-M005 and Verified Update Core M007 are closed.

Eggup already provides exact service identity and ownership, systemd/launchd/cron/watchdog/Windows SCM adapters, bounded manager operations, `commit_with_lifecycle`, explicit `KeepInstalled | RollBack`, and post-commit restoration/check/RecoveryRequired handling.

Current `commit_with_lifecycle` intentionally accepts only an `Owned` registration in `Running | Stopped`. That is safe but narrower than mature daemon deployments.

Greggd at the reviewed reference commit supplies concrete evidence for the missing distinction: an artifact may be safely replaceable even when a manager registration is foreign/preserved, and a selected direct daemon may be the runtime generation requiring quiescence/restart while a different manager registration exists.

This milestone is grounded in a real production pattern, not a speculative extension point.

## 3. Current implementation evidence

At the Eggup baseline, `commit_with_lifecycle`:

- inspects the manager;
- requires `Ownership::Owned`;
- requires `Running | Stopped`;
- quiesces owned running services before commit;
- restores success state according to `RestoreIntent`;
- coordinates post-commit check/rollback with Core M007.

At the Gregg reference baseline, `greggd/src/update.rs` has a pure `UpdateLifecycle` decision model with:

- `ManagedRunning`;
- `ManagedStopped`;
- `DirectRunning`;
- `Stopped`;
- `ForeignPreserved`.

Its Unix decision combines manager executable ownership, manager activity, registered config identity, selected config identity, and selected-endpoint runtime state. Its Windows decision combines SCM registration state with exact executable-path equivalence.

Greggd also enforces two important ordering rules:

1. fully prepare/verify the candidate before stopping a running daemon;
2. re-observe/revalidate lifecycle ownership immediately before mutation so an owned-to-foreign transition cannot authorize a stale stop/restart.

Those rules should become generic Eggup behavior where applicable.

## 4. Invariants that must not regress

- Foreign or unknown service-manager registrations are never destructively mutated.
- Artifact mutation authority and manager mutation authority are separate facts.
- A foreign-preserved disposition may authorize zero manager mutation; it does not convert the manager to Eggup ownership.
- Unknown contradictory active states fail closed.
- Candidate bytes are already acquired/verified/validated before any daemon quiescence.
- Runtime/manager ownership is revalidated after preparation and immediately before destructive lifecycle mutation.
- Stopped remains stopped under preserve semantics.
- RecoveryRequired never triggers automatic service/direct-runtime start.
- Rollback never restarts against uncertain artifact state.
- A direct-runtime seam must be caller-owned and exact-instance/config scoped; Eggup does not invent application health semantics.
- All manager/direct-runtime transitions are bounded.
- No implicit privilege escalation.
- Existing M005 `commit_with_lifecycle` behavior remains available and source-compatible unless an explicit migration note justifies a narrow change.
- No public Eggup type contains Gregg/greggd product terminology.

## 5. Scope

### In scope

- a generic update-runtime disposition type equivalent in semantics to managed-running / managed-stopped / direct-running / stopped / foreign-preserved;
- pure decision functions/planner inputs separating manager registration observation from selected direct-runtime observation;
- explicit representation of config/executable identity evidence needed to make those decisions;
- a small caller-owned direct-runtime control seam if required for bounded stop/start/re-observe of the selected direct instance;
- disposition-aware transaction/lifecycle orchestration over an existing `ValidatedTransaction`;
- post-preparation, pre-mutation revalidation;
- Windows transitional-state handling equivalent to Running/StartPending and Stopped/StopPending classes;
- differential/reference tests derived from greggd's pure decision matrix;
- deterministic scripted manager/direct-runtime tests;
- documentation of authority separation.

### Explicitly out of scope

- modifying, migrating, or adding a dependency on `eggstack/gregg`;
- replacing `greggd` startup/service modules;
- release/version selection;
- curl/Eggfetch acquisition;
- application-specific HTTP health JSON;
- generic daemon supervision;
- process discovery by name or broad host scanning;
- service-definition migration/refresh;
- package-manager/Cargo fallback;
- an `update [VERSION]` CLI contract;
- changing producer packaging/release behavior.

## 6. Required production changes

### 6.1 Generic disposition model

Add a product-neutral type, naming to be chosen during implementation, with semantics equivalent to:

```rust
ManagedRunning
ManagedStopped
DirectRunning
Stopped
ForeignPreserved
```

This type describes what lifecycle authority Eggup may exercise around an artifact transaction. It is not an installation registry and must not imply release-selection policy.

### 6.2 Planner inputs and pure decisions

Create explicit typed inputs for manager registration ownership, manager running/transitional state, manager executable identity, known registered config identity when relevant, selected artifact executable identity, selected config identity when relevant, and caller-supplied selected direct-runtime state.

The planner must be pure/deterministic so the reference matrix can be exhaustively unit-tested without native managers.

Do not infer direct-runtime ownership from a port/process name globally. The application supplies exact selected-runtime evidence through a narrow seam.

### 6.3 Direct-runtime control seam

If `DirectRunning` is executable by the generic orchestrator, introduce the minimum trait required to inspect the selected runtime, stop/quiesce it within a supplied deadline, and start/restore it within a supplied deadline.

The seam must be config/instance scoped and caller implemented. It MUST NOT encode greggd endpoints, control sockets, PID-file formats, or health JSON.

A consumer that cannot prove exact direct-runtime ownership must return Unknown and fail closed where mutation would otherwise be required.

### 6.4 Disposition-aware orchestration

Add an additive advanced orchestration entrypoint or generalize M005 without breaking its existing safe path.

Required behavior:

- `ManagedRunning`: revalidate owned manager -> quiesce -> commit -> restore/start according to policy -> post-install check;
- `ManagedStopped`: commit and preserve stopped state unless caller explicitly requests otherwise;
- `DirectRunning`: revalidate exact direct runtime -> quiesce through caller seam -> commit -> restore direct runtime -> post-install check;
- `Stopped`: commit with no fabricated restart;
- `ForeignPreserved`: perform no service-manager mutation; artifact commit may proceed only under independently valid Core commit ownership and caller policy.

Post-commit `KeepInstalled | RollBack` and RecoveryRequired semantics remain those established by M005/Core M007.

### 6.5 Revalidation barrier

The disposition used for mutation must be observed after candidate/transaction preparation and revalidated immediately before the first stop/quiesce or other destructive lifecycle action.

At minimum, an owned-to-foreign/unknown manager change must fail before manager mutation. If direct-runtime identity/state changes such that exact ownership is no longer proven, fail before direct mutation.

## 7. Ordered work packages

1. Port the Gregg decision matrix into product-neutral failing unit tests, recording the reference commit but copying no product API.
2. Define generic disposition and typed planner observations.
3. Implement pure Unix-like manager/direct decision logic.
4. Implement pure Windows SCM/executable decision logic including pending states.
5. Add revalidation-barrier tests, especially owned-to-foreign and exact-runtime identity changes.
6. Introduce the minimal direct-runtime control seam only if required by orchestration.
7. Add disposition-aware orchestration around `ValidatedTransaction`/Core M007.
8. Reuse M005 restoration/rollback machinery rather than duplicating transaction backup/restore.
9. Add scripted failure matrices for prepare-before-quiesce, rollback, KeepInstalled, RecoveryRequired, and final observation.
10. Run native manager regression lanes and package/docs qualification.
11. Record reference-parity evidence and update closure/status.

## 8. Failure, cancellation, restart, and contention semantics

Preparation failure performs zero lifecycle mutation.

A revalidation failure performs zero lifecycle mutation after the failed observation and returns bounded evidence describing changed authority/state.

For `ManagedRunning`, manager stop/restart is allowed only while ownership remains exact/Owned.

For `ForeignPreserved`, Eggup performs zero manager stop/start/install/uninstall regardless of whether the foreign registration is active.

For `DirectRunning`, only the caller-provided exact direct-runtime control may stop/start; manager adapters remain untouched unless separately owned and required by the disposition.

A post-commit restore/check failure follows `KeepInstalled | RollBack`. Before artifact rollback, a newly started managed/direct generation must be quiesced when safely possible, matching M005 ordering.

If Core returns RecoveryRequired, suppress automatic restart of either managed or direct runtime.

All waits use explicit bounded deadlines; do not turn synchronous caller code into a pretend forcibly cancellable API.

## 9. Compatibility and migration

No downstream migration occurs.

`commit_with_lifecycle` must continue to provide the existing owned-manager-only path. New behavior should be additive unless a small internal refactor preserves its public semantics exactly.

Gregg remains unchanged. Do not add a path/git/dev dependency on Gregg and do not alter Gregg planning files.

An eventual Gregg consumer-adoption M004 remains a separate future milestone after Acquisition M005 and Service M006 close.

## 10. Required tests

The product-neutral reference matrix must include at least:

Unix-like planning:

- Owned + active -> ManagedRunning;
- Owned + inactive -> ManagedStopped;
- Foreign + active + registered config equals selected config -> ForeignPreserved;
- Foreign + active + different registered config + selected direct running -> DirectRunning;
- Foreign + active + different registered config + selected direct stopped -> Stopped;
- Foreign + active + unknown registered config -> error;
- Foreign + inactive + selected direct running -> DirectRunning;
- Foreign + inactive + selected direct stopped -> Stopped;
- Absent + manager active -> error;
- Absent + manager inactive + selected direct running -> DirectRunning;
- Absent + manager inactive + selected direct stopped -> Stopped;
- Unknown + manager active -> error;
- Unknown + manager inactive + selected direct running -> error;
- Unknown + manager inactive + selected direct stopped -> Stopped.

Windows-like planning:

- not installed -> Stopped;
- registration executable absent/unparseable -> error;
- foreign executable path -> ForeignPreserved;
- owned Running -> ManagedRunning;
- owned StartPending -> ManagedRunning;
- owned Stopped -> ManagedStopped;
- owned StopPending -> ManagedStopped.

Orchestration/failure tests:

- candidate/transaction preparation completes before first quiesce call;
- owned-to-foreign revalidation change fails before stop;
- direct exact-instance revalidation change fails before stop;
- ManagedStopped stays stopped under Preserve;
- Stopped fabricates no restart;
- ForeignPreserved performs zero manager mutation;
- DirectRunning performs zero manager mutation unless separately authorized;
- ManagedRunning stop/commit/restart ordering;
- DirectRunning stop/commit/restart ordering;
- KeepInstalled activation failure reports committed artifacts plus lifecycle failure;
- RollBack quiesces new generation before artifact rollback and restores old lifecycle;
- RecoveryRequired suppresses all automatic starts;
- panic/error in caller post-install check follows existing M005 policy;
- no Gregg-specific public strings/types.

## 11. Required verification commands

Run and record at least:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo doc --workspace --no-deps
cargo check --workspace --all-targets --all-features
cargo package -p eggup-service --allow-dirty
```

Also run the repository's Rust 1.89/MSRV checks and hosted Linux/macOS/Windows lanes used for existing manager adapters. Native-manager limitations must be reported truthfully rather than inferred from deterministic tests.

## 12. Documentation updates

Update the service roadmap, `eggup-service` docs, authority/ownership documentation distinguishing artifact commit ownership from manager mutation ownership, lifecycle examples for the five dispositions, direct-runtime seam safety requirements, and consumer adoption roadmap noting Gregg remains unmigrated and blocked on upstream qualification only.

Reference Gregg only in implementation/closure evidence, not public API examples.

## 13. Acceptance criteria

M006 closes only when:

- Eggup has a product-neutral five-disposition update runtime model;
- the pure planner reproduces the reviewed greggd reference matrix where generic inputs are equivalent;
- artifact authority and manager mutation authority are represented separately;
- ForeignPreserved performs no manager mutation;
- DirectRunning uses only exact caller-owned direct-runtime control;
- candidate/transaction preparation precedes quiescence;
- authority/state is revalidated immediately before mutation;
- owned-to-foreign/unknown changes fail before destructive manager action;
- stopped-preservation, KeepInstalled, RollBack, and RecoveryRequired remain truthful;
- existing M001-M005 adapter/orchestration tests remain green;
- no Gregg dependency or migration is introduced;
- Rust 1.89, package/docs, and hosted platform checks pass;
- no medium-or-higher correctness/security finding remains open.

## 14. Stop conditions

Stop and write a corrective/ADR if:

- generic support requires Eggup to discover arbitrary processes/endpoints globally;
- ForeignPreserved cannot be represented without weakening the “never mutate foreign registration” invariant;
- direct-runtime control requires application-specific health/control protocol in `eggup-service`;
- revalidation cannot prevent stale manager ownership from authorizing stop/restart;
- the orchestrator would duplicate Core backup/rollback machinery;
- Windows running-image executable replacement is required for this service milestone;
- Gregg would need to be modified to prove the Eggup contract.

## 15. Closure evidence required

The closure record must include implementation commit(s), generic public types/traits added, Gregg reference commit `8b18f9ee16461e3fa0ef0d804ed39ebb9183b727`, reference decision matrix mapped to Eggup tests, prepare-before-quiesce ordering evidence, revalidation-race evidence, zero-mutation evidence for ForeignPreserved, direct-runtime exact-instance evidence, KeepInstalled/RollBack/RecoveryRequired fault matrix, native platform results and limitations, package/doc/MSRV evidence, security findings, and explicit confirmation that Gregg was neither modified nor depended on.

## 16. Handoff notes

Treat greggd as a behavioral oracle, not code to integrate.

The core distinction M006 must preserve is:

```text
permission to replace verified application artifacts
                    !=
permission to mutate a service manager registration/runtime
```

The existing owned-manager M005 path remains the simple case. M006 adds a truthful model for real daemon deployments without importing application-specific daemon policy into Eggup.
