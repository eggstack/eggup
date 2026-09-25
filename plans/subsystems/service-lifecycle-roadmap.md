# Service Lifecycle Roadmap

Status: M001-M005 closed; M006 daemon-update disposition/reference qualification ready for handoff

Long-term references:

- `plans/000-long-term-specification.md#10-ownership-model`
- `plans/000-long-term-specification.md#12-rollback-model`
- `plans/000-long-term-specification.md#13-service-lifecycle`
- `plans/000-long-term-specification.md#20-platform-scope`

Related ADRs:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`
- `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md`

## 1. Purpose and ownership boundary

This subsystem owns reusable, ownership-safe interaction with host service managers.

It does not own application health semantics, service hardening content, artifact acquisition, release selection, or update policy.

## 2. Work classification

### Invariants

- foreign/unknown registrations are never destructively mutated;
- exact executable identity participates in ownership decisions;
- stopped services remain stopped by default;
- manager calls are bounded;
- service operations do not imply artifact update.

### Capabilities

- inspect/install/uninstall/start/stop/restart owned registrations;
- preserve lifecycle state around a consumer transaction;
- represent manager conflicts explicitly;
- classify daemon-update runtime disposition separately from artifact mutation authority.

### Infrastructure

- ServiceSpec;
- ServiceRegistration/ServiceState;
- Ownership;
- systemd/launchd/cron/Windows SCM adapters;
- transition wait helpers;
- health-probe seam.

### Polish

- instructions/rendering;
- manager diagnostics;
- platform test harnesses.

## 3. Non-goals

- replacing native service managers;
- generic daemon supervision;
- application-specific health JSON;
- automatic root elevation;
- forcing all consumers to use the same service-installation path.

## 4. Current state evidence

M001 is closed and `eggup-service 0.1.0` is published with the manager-neutral ownership/lifecycle contract and deterministic test double.

M002 implemented systemd/launchd/cron adapters. M003 closed the follow-up restart truthfulness, end-to-end deadline, trusted execution environment, mutation-status, and config-identity findings. Unix service mechanics are qualified and already used by eggsearch.

M004 closed Windows SCM registration, ownership, start/stop/restart, and uninstall behavior. All planned manager families therefore have a reusable adapter surface.

The owned-manager orchestration milestone, M005, is closed. It consumes Core M007's ADR-0002 `KeepInstalled | RollBack` choice while a coherent new artifact generation remains live and rollback evidence is retained. See `plans/closure/service-lifecycle/005-status.md` and `plans/closure/verified-update-core/007-status.md` for qualification evidence and limits.

Read-only review of `eggstack/gregg@8b18f9ee16461e3fa0ef0d804ed39ebb9183b727` exposes a mature daemon-update distinction not yet represented by M005: managed-running, managed-stopped, direct-running, stopped, and foreign-manager-preserved states, with exact executable/config identity and a post-preparation revalidation barrier. M006 is ready to generalize and test those semantics in Eggup without modifying or depending on Gregg.

## 5. Target architecture

The service crate exposes manager-neutral types plus platform adapters. Consumer-supplied specifications identify exact executable/config arguments and optional health behavior.

Update orchestration composes service lifecycle with a prepared core transaction; the service crate does not depend on release transport.

## 6. Dependency graph

```text
core M005 corrected ownership/transaction semantics
       |
       v
M001 lifecycle model + ownership contract
       |
       v
M002 systemd/launchd/cron [closed]
       |
       v
M003 Unix adapter correctness/security corrective
       |
       +--> M004 Windows SCM [closed]
       |
       +---------------------------+
                                   |
core M007 deferred finalization ---+
                    |
                    v
                    M005 update-lifecycle integration [closed]
                              |
                              v
                    M006 daemon disposition + revalidation [ready]
                    (Gregg reference only; no migration)
```

## 7. Milestones

### M001 — Manager-neutral state and ownership

Plan: `plans/implementation/service-lifecycle/001-manager-neutral-state-and-ownership.md`.

Hard dependency: verified-update-core M005 closure.

Define ServiceSpec, registration/state, canonical ownership classification, conflict semantics, lifecycle snapshot, health seam, and test adapters.

### M002 — Unix manager adapters

Plan: `plans/implementation/service-lifecycle/002-unix-manager-adapters.md`.

Systemd, launchd, and cron/watchdog mechanics with bounded execution, exact ownership, caller-owned definitions, and no implicit elevation.

### M003 — Unix adapter correctness and execution hardening corrective

Plan: `plans/implementation/service-lifecycle/003-unix-adapter-correctness-security-corrective.md`.

Correct composed transition truthfulness, end-to-end deadline semantics, trusted manager executable/environment handling, and neutral config identity before consumer adoption.

### M004 — Windows SCM adapter

Plan: `plans/implementation/service-lifecycle/004-windows-scm-adapter.md`.

Native SCM registration/state/transition behavior and exact executable/argv/config ownership, built on the corrected M003 deadline/identity semantics. Use a typed SCM API rather than `sc.exe` parsing and keep product service policy caller-owned.

### M005 — Prepared-transaction lifecycle integration

Plan: `plans/implementation/service-lifecycle/005-prepared-transaction-lifecycle-integration.md`.

Status: closed; see `plans/closure/service-lifecycle/005-status.md`.

Hard dependencies:

- service M004 closure;
- verified-update-core M007 deferred-finalization/post-commit policy closure (closed; `plans/closure/verified-update-core/007-status.md`).

Compose a validated transaction with lifecycle snapshot, ownership-safe quiescence, coherent artifact commit, bounded post-commit lifecycle/check work, and explicit `KeepInstalled | RollBack` policy. M005 consumes the qualified `commit_with_post_commit` boundary rather than recreating artifact backup/restore machinery. The implementation plan also makes the rollback/service-state boundary explicit: RollBack quiesces a newly started service before artifact rollback when possible, successful artifact rollback restores the pre-update service state, and RecoveryRequired does not auto-start against uncertain artifacts. Application health semantics remain caller-owned through a bounded post-install check seam.

Implemented by `plans/closure/service-lifecycle/005-status.md`. Eggsearch M003 remains closed.

### M006 — Daemon update disposition and reference qualification

Plan: `plans/implementation/service-lifecycle/006-daemon-update-disposition-and-reference-qualification.md`.

Status: ready for handoff.

Hard dependencies: service M005 and verified-update-core M007 closure (both closed).

Generalize the daemon update decision model using greggd only as a read-only behavioral oracle. Separate artifact commit authority from service-manager mutation authority; add product-neutral managed-running / managed-stopped / direct-running / stopped / foreign-preserved dispositions; revalidate exact executable/config/runtime authority after preparation and immediately before mutation; and preserve M005 `KeepInstalled | RollBack` / RecoveryRequired behavior. No Gregg migration or dependency is allowed in M006.

## 8. Cross-cutting requirements

No destructive action on Foreign/Unknown. Permission errors return remediation rather than escalating. Service definition rendering must reject unsafe control characters/path ambiguity.

## 9. Verification strategy

Deterministic adapter tests plus native smoke lanes for Linux systemd, macOS launchd, and Windows SCM when available. Cron tests preserve unrelated crontab bytes.

## 10. Risks and decision points

System-level versus user-level service registration differs across consumers. Use Gregg's concrete decision matrix as reference evidence, but keep application health/control protocols caller-owned and do not require a Gregg migration to qualify the generic mechanism.

## 11. Completion definition

Manager mechanics remain shared by service-bearing consumers without losing application-specific policy. M006 additionally requires product-neutral reference parity for mature daemon-update dispositions without downstream migration.

## 12. Milestone status

| Milestone | Status | Implementation plan | Closure record | Blockers |
|---|---|---|---|---|
| M001 | closed | `plans/implementation/service-lifecycle/001-manager-neutral-state-and-ownership.md` | `plans/closure/service-lifecycle/001-status.md` | — |
| M002 | closed; post-closure findings feed M003 | `plans/implementation/service-lifecycle/002-unix-manager-adapters.md` | `plans/closure/service-lifecycle/002-status.md` | — |
| M003 | closed | `plans/implementation/service-lifecycle/003-unix-adapter-correctness-security-corrective.md` | `plans/closure/service-lifecycle/003-status.md` | — |
| M004 | closed | `plans/implementation/service-lifecycle/004-windows-scm-adapter.md` | `plans/closure/service-lifecycle/004-status.md` | — |
| M005 | closed | `plans/implementation/service-lifecycle/005-prepared-transaction-lifecycle-integration.md` | `plans/closure/service-lifecycle/005-status.md` | — |
| M006 | ready for handoff | `plans/implementation/service-lifecycle/006-daemon-update-disposition-and-reference-qualification.md` | — | M005 + Core M007 closed; Gregg reference only |
