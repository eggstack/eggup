# Service Lifecycle Roadmap

Status: M001-M004 closed; M005 prepared-transaction integration ready for plan authoring

Long-term references:

- `plans/000-long-term-specification.md#10-ownership-model`
- `plans/000-long-term-specification.md#13-service-lifecycle`
- `plans/000-long-term-specification.md#20-platform-scope`

Related ADR:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`

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
- represent manager conflicts explicitly.

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

M002 implemented systemd/launchd/cron adapters. M003 closed the follow-up restart truthfulness, end-to-end deadline, trusted execution environment, mutation-status, and config-identity findings. Unix service mechanics are therefore ready for consumer adoption.

Windows SCM is the remaining native manager family. Eggsearch provides concrete evidence for SCM create/query/start/stop/delete behavior without requiring its product policy to move into Eggup.

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
       +--> M004 Windows SCM
       |
       `-------------------+
                           v
                 M005 update-lifecycle integration
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

Compose a verified prepared transaction with quiesce/commit/restart using explicit post-commit failure policy.

## 8. Cross-cutting requirements

No destructive action on Foreign/Unknown. Permission errors return remediation rather than escalating. Service definition rendering must reject unsafe control characters/path ambiguity.

## 9. Verification strategy

Deterministic adapter tests plus native smoke lanes for Linux systemd, macOS launchd, and Windows SCM when available. Cron tests preserve unrelated crontab bytes.

## 10. Risks and decision points

System-level versus user-level service registration differs across consumers. Do not over-generalize before eggsearch/gregg migration evidence.

## 11. Completion definition

At least two service-bearing consumers share the manager mechanics without losing their application-specific service policy.

## 12. Milestone status

| Milestone | Status | Implementation plan | Closure record | Blockers |
|---|---|---|---|---|
| M001 | closed | `plans/implementation/service-lifecycle/001-manager-neutral-state-and-ownership.md` | `plans/closure/service-lifecycle/001-status.md` | — |
| M002 | closed; post-closure findings feed M003 | `plans/implementation/service-lifecycle/002-unix-manager-adapters.md` | `plans/closure/service-lifecycle/002-status.md` | — |
| M003 | closed | `plans/implementation/service-lifecycle/003-unix-adapter-correctness-security-corrective.md` | `plans/closure/service-lifecycle/003-status.md` | — |
| M004 | closed | `plans/implementation/service-lifecycle/004-windows-scm-adapter.md` | `plans/closure/service-lifecycle/004-status.md` | — |
| M005 | ready for plan authoring | — | — | service M004 + verified-update-core M005 closed |
