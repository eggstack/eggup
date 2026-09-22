# Service Lifecycle Milestone 001 — Manager-Neutral State and Ownership Contract

Status: blocked on verified-update-core M005 closure

Repository baseline for planning: `9f527beb20da585bc3cf56f45f1fd96fd418c124`

Source roadmap:

- `plans/subsystems/service-lifecycle-roadmap.md`

Long-term requirements:

- `plans/000-long-term-specification.md#10-ownership-model`
- `plans/000-long-term-specification.md#13-service-lifecycle`

Primary class: infrastructure / invariant

## 1. Objective

Define the manager-neutral service-registration, ownership, state, and lifecycle-snapshot model before implementing systemd, launchd, cron, or Windows SCM adapters.

## 2. Why this milestone is blocked

The service subsystem must reuse the corrected canonical ownership vocabulary from core M005 instead of inventing another `Managed`/boolean ownership model.

## 3. Invariants

- service ownership uses `Absent | Owned | Foreign | Unknown`;
- destructive manager operations are authorized only for Owned;
- a stopped service remains stopped unless caller policy explicitly changes that;
- manager state and application health are separate;
- no updater/release/network policy enters service crate;
- no automatic privilege elevation;
- exact executable/config arguments participate in ownership evidence.

## 4. Scope

### In scope

- `ServiceId` / `ServiceSpec`;
- manager-neutral registration snapshot;
- lifecycle state vocabulary;
- ownership evidence/result;
- lifecycle snapshot;
- desired restoration intent;
- manager trait/test double;
- health-probe seam;
- bounded transition result model;
- conflict diagnostics.

### Explicitly out of scope

- real systemd/launchd/cron/SCM calls;
- service-file rendering;
- privilege escalation;
- update transaction orchestration;
- application-specific health payloads.

## 5. Required design

A registration snapshot should be capable of expressing:

- absent;
- present and exactly owned;
- present but foreign;
- present but ownership cannot be proven.

Running state should distinguish at least stopped, running, transitioning, and unknown/error without conflating health.

The model must preserve pre-update intent so later orchestration can restore "was running" versus "was registered but stopped."

## 6. Required tests

- exact owned registration;
- same service name/different executable -> Foreign;
- same executable/different critical args -> Foreign or Unknown according to documented rule;
- malformed registration -> Unknown;
- absent;
- stopped/running snapshot;
- health failure does not mutate manager state;
- foreign/unknown destructive operation denied by manager-neutral guard.

## 7. Acceptance criteria

Platform adapters can implement the contract without changing its ownership vocabulary or embedding consumer-specific service definitions.

## 8. Stop conditions

Stop if systemd/launchd/SCM differences require incompatible public ownership/state semantics; resolve via ADR before adapter implementation.

## 9. Closure evidence required

- public model;
- ownership matrix;
- test-double lifecycle tests;
- dependency tree proving no transport/update-policy coupling.
