# Eggup Planning and Agent-Handoff Process

Status: normative planning governance

This document defines how Eggup's durable architecture becomes bounded implementation work.

The keywords MUST, MUST NOT, REQUIRED, SHOULD, SHOULD NOT, and MAY are normative.

## 1. Planning horizons

Eggup maintains two horizons.

**Long-term planning** defines ownership boundaries, safety invariants, public abstractions, dependency order, and end-state capability.

**Interim planning** defines bounded work against a specific repository baseline and is the primary coding-agent handoff.

Interim implementation MUST NOT silently redefine the long-term safety model.

## 2. Canonical documents

Canonical documents are:

- `plans/000-long-term-specification.md`;
- `plans/001-terminology-and-domain-model.md`;
- `plans/002-long-term-roadmap.md`;
- this document.

Ordinary coding passes do not rewrite them.

Changes require:

- explicit project-direction change;
- a material contradiction or omission;
- an accepted ADR that changes the intended architecture;
- explicit maintainer instruction.

## 3. ADRs

Use an ADR when a decision establishes a durable cross-consumer contract.

Typical Eggup ADR subjects include:

- crate/layer ownership;
- transaction atomicity and rollback guarantees;
- authenticity trust standard;
- archive extraction safety contract;
- service ownership semantics;
- persistent receipt/lock format compatibility;
- public MSRV policy.

Accepted ADRs are historical. Supersede rather than rewrite.

Before authoring any cross-repository release/update milestone, plan authors MUST classify the work against ADR-0004. Producer release contracts, conformance, packaging, bootstrap generation, release manifests, generated release CI, publication, and producer provenance belong in Eggpack. Consumer acquisition, verification, local mutation/recovery, service lifecycle, and optional manifest-to-deployment translation belong in Eggup. If a proposed Eggup plan crosses that boundary, stop and register the work in Eggpack instead.

## 4. Subsystem roadmaps

A subsystem roadmap MUST define:

- purpose and ownership;
- invariants/capabilities/infrastructure/polish;
- non-goals;
- current evidence from Eggstack consumers where relevant;
- target architecture;
- dependency graph;
- ordered milestones;
- cross-cutting security/recovery/platform concerns;
- verification strategy;
- risks/decision points;
- completion definition;
- milestone status table.

Roadmaps should survive several repository revisions. Exact file edits belong in implementation plans.

## 5. Implementation plans

A milestone implementation plan MUST be:

- independently executable;
- tied to a repository baseline;
- dependency-ready;
- narrow enough for one coherent coding pass;
- explicit about non-goals;
- explicit about failure and rollback semantics;
- explicit about compatibility/consumer effects;
- specific about tests and closure evidence.

Each plan MUST contain:

1. objective;
2. readiness/dependencies;
3. current evidence;
4. invariants;
5. scope/non-scope;
6. required production changes;
7. ordered work packages;
8. failure/restart/contention semantics;
9. compatibility/migration;
10. tests;
11. verification commands;
12. docs;
13. acceptance;
14. stop conditions;
15. closure evidence;
16. handoff notes.

## 6. Work classification

Every item receives one primary class.

### Invariant

Safety or architecture property that must always remain true.

Examples:

- unverified bytes never execute;
- core owns no HTTP stack;
- destructive operations require ownership evidence.

### Capability

Consumer/operator-visible behavior.

Examples:

- a consumer can update a three-binary release atomically;
- a daemon can preserve registered/running state across update.

### Infrastructure

Reusable machinery enabling capabilities.

Examples:

- MutationLock;
- ArtifactSet;
- Eggfetch Fetcher;
- Windows SCM adapter.

### Polish

Diagnostics, ergonomics, performance, documentation, cleanup, and optimization.

## 7. Dependency classes

Each milestone declares dependencies as:

- **hard** — cannot correctly begin before closure;
- **interface** — may proceed against a stable written contract/test double;
- **soft** — may proceed independently but integration depends on another milestone;
- **operational** — code can land, but deployment/release needs external evidence.

A milestone is ready only when hard dependencies are closed and interface dependencies are sufficiently specified.

## 8. Milestone sizing

Prefer vertical contracts over broad refactors.

A good milestone establishes one meaningful reusable boundary with its failure tests.

Examples:

- good: multi-artifact commit + rollback with fault injection;
- too broad: core + HTTP + services + installer generation;
- too small: rename Artifact to Package with no contract change.

## 9. Consumer evidence rule

Eggup is a shared library. Genericity must be demonstrated.

A new abstraction SHOULD be grounded in at least two existing consumer patterns or one existing pattern plus a clearly required second consumer.

Do not create speculative extension points merely because they might be useful.

When consumer implementations conflict, preserve the distinction as explicit caller policy rather than choosing one silently.

## 10. Security-sensitive implementation rule

For code handling downloads, staged executables, filesystem mutation, locks, rollback, archives, or services:

- inspect current platform behavior before editing;
- prefer typed state over boolean flags;
- bound all external I/O;
- add negative tests before claiming closure;
- test partial failure, not only success;
- preserve evidence for recovery-required states;
- never add hidden fallback or privilege escalation.

## 11. Corrective passes

A corrective pass is a new plan.

It MUST:

- reference the original plan and closure record;
- enumerate each unclosed requirement;
- explain the detection gap;
- add regression evidence;
- avoid reopening unrelated closed scope.

Repeated corrective passes should trigger roadmap/milestone decomposition review.

## 12. Closure records

A closure record MUST contain:

- implementation commits/PRs;
- requirement-to-evidence matrix;
- exact tests/commands actually run;
- invariant review;
- failure/recovery review;
- compatibility/migration review;
- security review;
- docs/operations evidence;
- unresolved findings with severity;
- disposition.

Compilation alone is never closure.

## 13. Registry

`plans/registry.md` is the active control surface.

It should contain only:

- active subsystem roadmaps;
- ready/active implementation plans;
- recently closed work;
- blocked work and blockers;
- next dependency transitions.

Do not duplicate plan detail into the registry.

## 14. Baseline handling

Before handoff, record the current repository SHA.

For the initial empty-repository planning bootstrap, the first implementation milestone uses the planning baseline commit created by these documents.

After implementation begins, plans MUST use exact SHAs rather than vague "main" references.

## 15. Verification honesty

Closure records MUST distinguish:

- passed;
- failed;
- timed out;
- blocked by environment;
- not run;
- not applicable.

Do not infer Windows/macOS behavior from Linux-only tests. Platform-sensitive closure may be conditional when implementation is complete but native evidence is unavailable.

## 16. Public API discipline

Because Eggup is intended as a cross-repository library:

- exported types require docs;
- security-sensitive defaults require explicit documentation;
- consumer-specific names must not leak into generic modules;
- feature flags require dependency-surface review;
- package dry-run is part of boundary qualification;
- breaking API changes before 1.0 still require migration notes for existing Eggstack consumers.

## 17. Subsystem decomposition

Active Eggup roadmaps are:

1. verified update core;
2. acquisition transport;
3. service lifecycle;
4. consumer adoption and compatibility.

The historical distribution/bootstrap roadmap is migration-only. Its sole remaining milestone is M004 retirement of `eggup-dist`, blocked on Eggpack Contract M002 closure. It MUST NOT be extended with producer capabilities.

Future Eggpack-manifest interoperability, if justified, is a consumer adapter concern and receives its own narrow Eggup roadmap/plan only after Eggpack ReleaseManifest v1 is stable.

Only dependency-ready implementation plans should be created. Future roadmap milestones need not have detailed implementation plans until preceding evidence stabilizes the contract.
