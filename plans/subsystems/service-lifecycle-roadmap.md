# Service Lifecycle Roadmap

Status: active; M001-M002 closed, M003 Windows SCM planning-ready

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

Eggsearch and greggd each contain mature but separate Unix manager logic. Both demonstrate the rules M002 should extract: native manager selection, exact ownership checks, bounded manager commands/transitions, health distinct from manager state, cron fallback, and no hidden elevation. Windows SCM remains a separate M003 concern.

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
       +--> M002 systemd/launchd/cron
       |
       `--> M003 Windows SCM
                 |
                 v
           M004 update-lifecycle integration
```

## 7. Milestones

### M001 — Manager-neutral state and ownership

Plan: `plans/implementation/service-lifecycle/001-manager-neutral-state-and-ownership.md`.

Hard dependency: verified-update-core M005 closure.

Define ServiceSpec, registration/state, canonical ownership classification, conflict semantics, lifecycle snapshot, health seam, and test adapters.

### M002 — Unix manager adapters

Plan: `plans/implementation/service-lifecycle/002-unix-manager-adapters.md`.

Systemd, launchd, and cron/watchdog mechanics with bounded execution, exact ownership, caller-owned definitions, and no implicit elevation.

### M003 — Windows SCM adapter

Native SCM registration/state/transition behavior and running-image ownership checks.

### M004 — Prepared-transaction lifecycle integration

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
| M002 | closed | `plans/implementation/service-lifecycle/002-unix-manager-adapters.md` | `plans/closure/service-lifecycle/002-status.md` | — |
| M003 | planned (planning-ready) | — | — | service M002 closed; detailed plan may now be authored |
| M004 | planned | — | — | service M002/M003 + corrected core |
