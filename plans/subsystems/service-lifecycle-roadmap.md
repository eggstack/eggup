# Service Lifecycle Roadmap

Status: active; M001–M010 closed; M011 service 0.1.3 release qualification complete, with publication pending explicit maintainer authorization; `eggup-service 0.1.2` published 2026-10-05

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

Read-only review of `eggstack/gregg@8b18f9ee16461e3fa0ef0d804ed39ebb9183b727` exposed a mature daemon-update distinction not represented by M005. M006 is closed at `plans/closure/service-lifecycle/006-status.md`, generalizing managed-running, managed-stopped, direct-running, stopped, and foreign-manager-preserved states with exact executable/config identity and a post-preparation revalidation barrier, without modifying or depending on Gregg.

A 2026-09-26 post-closure audit opened narrow corrective M007: several production bounded-diagnostic helpers use fixed-byte `String::truncate` and can panic when a multibyte UTF-8 code point crosses the 256/512-byte limit. M007 changes no lifecycle state-machine semantics.

After C006 closed on head `510111f`, hosted run `36261671380` failed only the macOS `unix_tests::transition_deadline_rejects_zero_and_only_shrinks` test. Stable Linux, Rust 1.89 MSRV, and Windows all passed. The test used an 80 ms real deadline plus a 10 ms sleep and unconditionally expected a second positive timeout; on the hosted runner the valid production exhaustion error arrived instead. M008 closed this as a deterministic-test corrective (injected-`Instant` arithmetic, no production deadline change) with fresh green hosted run `36332823865` on `0b0cdaa`; see `plans/closure/service-lifecycle/008-status.md`.

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
M006 daemon disposition + revalidation [closed; evidence reconciled via acquisition M006]
                      (Gregg reference only; no migration)
                                |
                                v
                      M007 UTF-8 bounded diagnostics [CLOSED]
                                |
                                v
                      M008 deadline-test determinism [CLOSED via run 36332823865]
                                |
                                v
                      M009 eggup-service 0.1.2 publication [CLOSED]
                         |                   |
                         |                   `--> Acquisition M010 workspace 0.1.3 bump
                         v
                      M010 failed-systemd quiescence [CLOSED; runtime change]
                         |
                         v
                      M011 service 0.1.3 release [QUALIFIED; publish authorization pending]
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

Status: closed; see `plans/closure/service-lifecycle/006-status.md` (evidence reconciled via acquisition M006, no service code change).

Hard dependencies: service M005 and verified-update-core M007 closure (both closed).

Generalized the daemon update decision model using greggd only as a read-only behavioral oracle. Artifact commit authority and service-manager mutation authority are separate; product-neutral managed-running / managed-stopped / direct-running / stopped / foreign-preserved dispositions; exact executable/config/runtime authority revalidated after preparation and immediately before mutation; M005 `KeepInstalled | RollBack` / RecoveryRequired behavior preserved. No Gregg migration or dependency.

### M007 — UTF-8-safe bounded diagnostics corrective

Plan: `plans/implementation/service-lifecycle/007-utf8-safe-bounded-diagnostics-corrective.md`.

Status: closed; see `plans/closure/service-lifecycle/007-status.md`.

Replace panic-capable fixed-byte string truncation in production service diagnostics with UTF-8-boundary-safe byte bounding while preserving existing limits and all M005/M006 lifecycle behavior. Closed with the portable diagnostic boundary matrix, full Linux/macOS service suites, and Windows diagnostic/SCM runtime qualification.

### M008 — OperationDeadline test determinism corrective

Plan: `plans/implementation/service-lifecycle/008-operation-deadline-test-determinism-corrective.md`.

Status: closed; see `plans/closure/service-lifecycle/008-status.md`.

Replaced the scheduler-sensitive 80 ms/sleep-based unit proof with deterministic injected-`Instant` arithmetic while preserving the existing absolute-deadline runtime contract. No production tolerance, deadline extension, retry, API, or dependency change. Closed with fresh green Stable/MSRV/macOS/Windows hosted run `36332823865`, which supersedes failed run `36261671380`.

### M009 — eggup-service 0.1.2 publication

Plan: `plans/implementation/service-lifecycle/009-eggup-service-0.1.2-publication.md`.

Status: closed; see `plans/closure/service-lifecycle/009-status.md`.

Publish the already-qualified M001-M008 service substrate without runtime/source change. This is a release-order prerequisite for Acquisition M010 because that milestone moves the shared workspace version to 0.1.3.

M009 must preserve the existing `v0.1.2` tag as the historical core/archive publication source; service 0.1.2 is a later package publication from its own recorded commit and is documented additively.

### M010 — Owned failed-systemd service quiescence corrective

Plan: `plans/implementation/service-lifecycle/010-owned-failed-systemd-service-quiescence-corrective.md`.

Status: **closed**; see `plans/closure/service-lifecycle/010-status.md`.

A real wg-basic failed-candidate recovery identified that systemd `ActiveState=failed` is mapped to generic `LifecycleState::Unknown`; the wg-basic adapter refuses this state before calling Eggup stop and Eggup's stop completion accepts only `inactive`. First build a discriminating real-systemd fixture: a failed unit may or may not converge to inactive after stop, and failure is not proof of quiescence. Implement the minimum portable source-compatible ownership-safe quiescence operation actually needed; do not add a public closed-enum `Failed` variant or treat all Unknown as Stopped. Recheck exact service owner, inspect manager/process state and preserve restart/deadline/foreign refusal invariants.

M010 found and fixed an upstream runtime defect. On systemd 255, a successful stop left an exact-owned failed service reported as `failed`, so the former inactive-only poll could not complete. The implementation now proves post-stop ownership, manager state, job/process evidence, cgroup quiescence when present, and consistent `is-active` output. A live child in the cgroup remains incomplete, and a changed-ExecStart control is denied. The hosted Linux systemd lane and Stable/MSRV/macOS/Windows matrix are green; see the M010 closure for exact evidence.

### M011 — Conditional published service patch for wg-basic

Plan: `plans/implementation/service-lifecycle/011-verified-service-patch-publication.md`.

Status: **release qualification complete; publication blocked on explicit maintainer authorization**. M010 closed with a required runtime change, so this plan is applicable. Exact candidate head `f65496f` passed hosted run `37876276212` on Stable/MSRV/macOS/Windows and real Linux systemd.

The exact corrected `eggup-service 0.1.3` candidate is built and checked against registry-only consumers, with Stable/MSRV/macOS/Windows and current real-systemd evidence. Package checksum is `4de199de96dc24a8af5f524db069f5b8c47a8d599b0d624be78539bc9e05b9c2` at source `f65496f`; `0.1.3` remains absent from crates.io. Preserve historical shared tags and all other crate identities. Crates.io publication remains a distinct explicit authorization boundary. After publication, supply the immutable version/checksum to wg-basic; do not publish wg-basic itself.

## 8. Cross-cutting requirements

No destructive action on Foreign/Unknown. Permission errors return remediation rather than escalating. Service definition rendering must reject unsafe control characters/path ambiguity.

## 9. Verification strategy

Deterministic adapter tests plus native smoke lanes for Linux systemd, macOS launchd, and Windows SCM when available. Cron tests preserve unrelated crontab bytes.

## 10. Risks and decision points

System-level versus user-level service registration differs across consumers. Use Gregg's concrete decision matrix as reference evidence, but keep application health/control protocols caller-owned and do not require a Gregg migration to qualify the generic mechanism.

## 11. Completion definition

Manager mechanics remain shared by service-bearing consumers without losing application-specific policy. M006 additionally requires product-neutral reference parity for mature daemon-update dispositions without downstream migration. M007 closed the bounded-diagnostic panic gap. M008 closed the deadline-arithmetic determinism gap with a fresh green hosted matrix. M009 published the 0.1.2 package; M010 closed the owned failed-systemd quiescence defect with a source-compatible runtime correction. M011's 0.1.3 release qualification is complete; publication and downstream wg-basic handoff await explicit registry-publication authorization.

## 12. Milestone status

| Milestone | Status | Implementation plan | Closure record | Blockers |
|---|---|---|---|---|
| M001 | closed | `plans/implementation/service-lifecycle/001-manager-neutral-state-and-ownership.md` | `plans/closure/service-lifecycle/001-status.md` | — |
| M002 | closed; post-closure findings feed M003 | `plans/implementation/service-lifecycle/002-unix-manager-adapters.md` | `plans/closure/service-lifecycle/002-status.md` | — |
| M003 | closed | `plans/implementation/service-lifecycle/003-unix-adapter-correctness-security-corrective.md` | `plans/closure/service-lifecycle/003-status.md` | — |
| M004 | closed | `plans/implementation/service-lifecycle/004-windows-scm-adapter.md` | `plans/closure/service-lifecycle/004-status.md` | — |
| M005 | closed | `plans/implementation/service-lifecycle/005-prepared-transaction-lifecycle-integration.md` | `plans/closure/service-lifecycle/005-status.md` | — |
| M006 | closed; hosted evidence reconciled via acquisition M006 | `plans/implementation/service-lifecycle/006-daemon-update-disposition-and-reference-qualification.md` | `plans/closure/service-lifecycle/006-status.md` | — |
| M007 | closed | `plans/implementation/service-lifecycle/007-utf8-safe-bounded-diagnostics-corrective.md` | `plans/closure/service-lifecycle/007-status.md` | — |
| M008 | closed | `plans/implementation/service-lifecycle/008-operation-deadline-test-determinism-corrective.md` | `plans/closure/service-lifecycle/008-status.md` | — |
| M009 | closed | `plans/implementation/service-lifecycle/009-eggup-service-0.1.2-publication.md` | `plans/closure/service-lifecycle/009-status.md` | — (published 2026-10-05 from `7fb84bc`; registry-only fixture 10/10; `v0.1.2` unmoved; no `src/` change). Unblocks the Acquisition M010 workspace bump |
| M010 | closed — runtime corrective | `plans/implementation/service-lifecycle/010-owned-failed-systemd-service-quiescence-corrective.md` | `plans/closure/service-lifecycle/010-status.md` | run `37875012280` green on Stable/MSRV/macOS/Windows and real Linux systemd including auto-restart race; runtime change required |
| M011 | release qualification complete; publication blocked on explicit authorization | `plans/implementation/service-lifecycle/011-verified-service-patch-publication.md` | not yet written | Hosted run `37876276212` green at `f65496f`; package SHA-256 recorded in plan; registry-only 0.1.2 negative control passes; no publish without maintainer authorization |
