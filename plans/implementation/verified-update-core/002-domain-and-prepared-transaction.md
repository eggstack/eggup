# Verified Update Core Milestone 002 — Domain and Prepared Transaction Contract

Status: blocked on M001 closure

Repository baseline for planning: `5b6cf13ed7e6e2bd40951216f29ea68d7333aadf`

Source roadmap:

- `plans/subsystems/verified-update-core-roadmap.md#M002--core-domain-and-prepared-transaction-contract`

Long-term requirements:

- `plans/000-long-term-specification.md#5-transaction-model`
- `plans/000-long-term-specification.md#6-multi-artifact-installation-is-fundamental`
- `plans/000-long-term-specification.md#9-filesystem-safety`
- `plans/001-terminology-and-domain-model.md`

Applicable ADRs:

- ADR-0001
- ADR-0002

Primary class: infrastructure / invariant

## 1. Objective

Define the policy-neutral Rust domain and a preparation phase that can validate and privately stage a one- or multi-member deployment without mutating live destinations.

## 2. Why this milestone is not yet ready

Hard dependency: M001 accepted closure.

At handoff time, update this plan's repository baseline to the M001 closure commit and reconcile file/module names with the actual workspace.

## 3. Current implementation evidence

The planning baseline has no Rust production implementation. M001 is expected to establish only the workspace and test harness.

Existing consumer evidence requires the model to support both one executable and bundles such as Egress/CodeGG.

## 4. Invariants that must not regress

- no network or service manager in core;
- no live destination mutation during preparation;
- installation root and each destination are explicit;
- staged state is private;
- duplicate destination/member ambiguity fails;
- path traversal/escape fails;
- policy values such as ReleaseId remain consumer-owned opaque values;
- preparation cannot silently infer members from PATH.

## 5. Scope

### In scope

- validated ProductId/ReleaseId or equivalent wrappers;
- ArtifactMember and ArtifactSet;
- InstallPlan;
- exact destination model;
- expected file kind/permission intent;
- integrity/authenticity requirement descriptors without implementing all verification;
- CandidateValidator trait/interface shell;
- Ownership enum;
- transaction phase/state types;
- private Stage type;
- prepare API that copies/accepts already-acquired local inputs into owned staging and validates plan coherence;
- deterministic plan/stage tests.

### Explicitly out of scope

- digest computation if M004 owns it;
- executing candidates;
- network download;
- locking;
- backup/live replacement;
- rollback;
- services;
- archives;
- Cargo fallback.

## 6. Required production changes

Design APIs so invalid phase transitions are difficult to express.

Prefer distinct types such as `InstallPlan` -> `PreparedTransaction` over one mutable transaction with public booleans.

Artifact member identity must be independent from destination basename.

Plan validation should reject:

- empty artifact sets;
- duplicate member IDs;
- duplicate destinations;
- destination outside authorized installation root unless a future explicit external-destination type is introduced;
- source/staged path aliasing that defeats ownership;
- directory/device/socket/FIFO inputs where a regular file is required;
- control characters or invalid platform path cases where later command rendering would be unsafe.

Stage creation should use owner-private temporary state and explicit cleanup ownership.

## 7. Ordered work packages

### A — Domain identities and artifact-set model

Define minimal validated IDs and artifact/member set.

### B — InstallPlan validation

Enforce path/member uniqueness and installation-root containment.

### C — Stage ownership

Create private stage abstraction and copy/attach acquired local files without live mutation.

### D — PreparedTransaction type

Consume a valid plan + staged members into a non-committed prepared state with explicit remaining validation requirements.

### E — Fixture matrix

Represent at least:

- eggsact/stegoeggo single binary;
- Egress two-binary sibling pair;
- CodeGG three-runfile bundle.

## 8. Failure, cancellation, restart, and contention semantics

Preparation failure deletes only Eggup-owned staging and never touches destinations.

No lock is required yet because no live state changes. If preparation opens paths that could race, record the evidence required for M003 revalidation rather than claiming race safety prematurely.

## 9. Compatibility and migration

No consumer migration yet. Public domain names should match canonical terminology.

## 10. Required tests

- empty/duplicate set rejection;
- destination escape rejection;
- relative/absolute normalization cases;
- symlink source behavior explicitly tested;
- private stage permissions on Unix;
- cleanup after injected prepare failure;
- three representative consumer layouts;
- compile-fail or API tests if useful to prove prepared-vs-unprepared phase separation.

## 11. Required verification commands

Use M001 broad commands plus focused package tests.

## 12. Documentation updates

Add core domain/state-machine docs and examples that stop before commit.

## 13. Acceptance criteria

A caller can prepare a valid one- or multi-member transaction from local inputs; preparation proves plan/stage coherence but performs zero live mutation.

## 14. Stop conditions

Stop if implementation requires:

- network transport;
- service knowledge;
- consumer-specific version parsing;
- live replacement;
- a persistent crash journal not already justified by M003 design.

## 15. Closure evidence required

- API/domain inventory;
- representative layout tests;
- staging permission evidence;
- negative path tests;
- confirmation of zero live-destination mutation;
- broad verification.

## 16. Handoff notes

Before execution, replace the planning baseline with the M001 closure SHA and adjust only file-level mechanics, not the ownership boundary.
