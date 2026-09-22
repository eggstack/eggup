# Service Lifecycle Roadmap

Status: M001-M003 closed; Windows SCM and lifecycle integration remain planned

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

M002 implemented systemd/launchd/cron adapters, but post-closure review found four correctness/security gaps tracked by M003: launchd restart can report complete after an incomplete subtransition, caller timeouts are not consistently end-to-end budgets, production manager execution trusts ambient PATH/full inherited environment, and `ServiceSpec.config` is not faithfully represented by systemd/launchd observations.

Eggsearch and greggd remain the primary consumer evidence. No service-bearing consumer should migrate until M003 closes. Windows SCM moves to M004 so it can reuse the corrected shared executor/deadline/identity semantics.

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

Native SCM registration/state/transition behavior and running-image ownership checks, built on the corrected M003 shared execution/identity semantics.

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
| M004 | dependency-ready / plan needed | — | — | M003 closed |
| M005 | planned / blocked | — | — | service M004 + corrected core |
