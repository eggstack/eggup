# Verified Update Core Roadmap

Status: M001-M006 closed; M007 post-commit policy ready for handoff

Long-term references:

- `plans/000-long-term-specification.md#3-mechanism-versus-policy`
- `plans/000-long-term-specification.md#5-transaction-model`
- `plans/000-long-term-specification.md#6-multi-artifact-installation-is-fundamental`
- `plans/000-long-term-specification.md#9-filesystem-safety`
- `plans/000-long-term-specification.md#10-ownership-model`
- `plans/000-long-term-specification.md#11-locking-and-contention`
- `plans/000-long-term-specification.md#12-rollback-model`

Related ADRs:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`
- `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md`
- `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md`

## 1. Purpose and ownership boundary

This subsystem owns the security-sensitive local mechanics that turn already resolved/acquired artifacts into a coherent installed deployment.

It owns:

- core domain types;
- staging;
- integrity primitives that operate on local bytes/files;
- candidate execution/validation;
- destination ownership/preflight;
- mutation locking;
- backup/commit/rollback;
- transaction receipts and recovery outcomes;
- generic uninstall-safe path primitives.

It does not own:

- network transport;
- release discovery/version ordering;
- service managers;
- consumer CLI;
- bootstrap installer generation;
- application migrations.

## 2. Work classification

### Invariants

- unverified bytes never execute;
- no live mutation occurs before all required candidates validate;
- one mutation lock owns an installation domain;
- successful multi-member commit is generation-consistent;
- pre-commit failure leaves live state unchanged;
- rollback/recovery outcome is explicit;
- post-commit failure policy remains rollback-capable until explicit finalization;
- no implicit privilege escalation;
- eggup-core carries no HTTP/TLS/service-manager dependency.

### Capabilities

- prepare and commit a one-member deployment;
- prepare and commit a multi-member deployment;
- recover the previous deployment after injected commit failures;
- retain rollback capability through one caller-supplied post-commit verification step;
- validate executable identity/version through bounded commands;
- expose structured terminal receipts/errors.

### Infrastructure

- validated identifiers;
- ArtifactSet/InstallPlan;
- private stage;
- MutationLock;
- BackupSet;
- transaction state machine;
- failure injection/test support.

### Polish

- human-readable diagnostics;
- event/progress hooks;
- dependency/footprint tuning;
- documentation examples.

## 3. Non-goals

- GitHub/crates.io API clients;
- Cargo fallback policy;
- service restart;
- archive format selection;
- package-manager transitions;
- persistent application update history.

## 4. Current state

M001-M004 are historically closed and the repository now contains a functional local transaction core: validated plans/private staging, multi-artifact commit/rollback, native SHA-256 integrity checks, bounded candidate execution, and typed terminal disposition.

A post-closure review at `9f527be` found pre-adoption contract defects that must be corrected before package qualification:

- existing-destination ownership is not independently proven;
- destination-parent creation occurs before final containment revalidation;
- staged bytes are not re-hashed immediately before commit;
- an unenforced authenticity-required state is public;
- cleanup disposition is misnamed as `PostCommitFailurePolicy`;
- rollback/recovery receipts discard the triggering phase/cause;
- cleanup failure can report a synthetic non-existent recovery path;
- transaction-owned Unix permissions are not explicitly private;
- stale-lock handling is fail-closed but planning language implied stronger recovery;
- root/crate architecture documentation still contains foundation-era capability statements.

These were tracked and closed in M005, followed by M006 package qualification. A later planning review identified one accepted ADR-0002 capability that was intentionally reserved but still unimplemented: the current successful `ValidatedTransaction::commit()` finalizes backup state before a caller can run post-install verification and choose `KeepInstalled | RollBack`. M007 owns that missing deferred-finalization boundary. M001-M006 closure records remain historical evidence and are not rewritten.

## 5. Target architecture

Initial package:

```text
crates/eggup-core/
  src/
    artifact.rs
    identity.rs
    integrity.rs
    ownership.rs
    stage.rs
    candidate.rs
    lock.rs
    transaction.rs
    rollback.rs
    receipt.rs
    error.rs
    lib.rs
  tests/
    transaction_faults.rs
    ownership.rs
    candidate.rs
```

Exact file names may evolve. The ownership split is normative; layout is not.

Core public API should be transaction-oriented rather than a single `self_update()` convenience function.

## 6. Dependency graph

```text
M001 repository/workspace foundation
        |
        v
M002 core domain + prepared transaction
        |
        +-------------------+
        |                   |
        v                   v
M003 commit/rollback      M004 integrity/candidate validation
        |                   |
        +---------+---------+
                  |
                  v
M005 pre-qualification safety/API corrective
                  |
                  v
M006 core package qualification
        |
        v
M007 deferred finalization + post-commit policy
```

- M001 -> M002: hard.
- M002 -> M003: hard.
- M002 -> M004: hard.
- M003 + M004 -> M005: hard.
- M005 -> M006: hard.
- M005 + M006 -> M007: hard and satisfied.
- Acquisition consumers may continue using the qualified immediate-commit path.
- Service Lifecycle M005 is hard-blocked on M007 because rollback after service restart/health failure requires backup retention beyond the initial live commit.

## 7. Milestones

### M001 — Repository/workspace foundation and contract harness

Class: infrastructure/invariant.

Objective: establish the Rust workspace, package boundary, safety linting, deterministic test support, and verification baseline without implementing updater behavior.

Exit conditions:

- Rust 1.89 workspace builds/tests;
- eggup-core exists with no network/service dependency;
- package metadata and architecture docs exist;
- deterministic temp-root/failure-injection support has a stable internal contract.

### M002 — Core domain and prepared-transaction contract

Class: infrastructure/invariant.

Objective: represent single/multi-artifact deployment plans, ownership, staging, and prepared state without live mutation.

Exit conditions:

- duplicate/escaping destination plans fail;
- private stage exists;
- exact installation root and destinations are explicit;
- candidate requirements can be associated with members;
- no commit API can be reached before preparation.

### M003 — Mutation lock, commit, rollback, and recovery

Class: capability/invariant.

Objective: safely mutate one or many live members.

Exit conditions:

- lock contention/stale handling covered;
- backup-before-commit behavior proven;
- per-phase failure injection restores old state when possible;
- rollback failure yields RecoveryRequired;
- success never leaves mixed generation.

### M004 — Integrity and candidate validation

Class: capability/invariant.

Objective: centralize local verification.

Exit conditions:

- native SHA-256;
- strict sidecar/manifest parsing;
- bounded executable validator;
- exact identity/version helper;
- custom validator composition;
- unverified bytes cannot execute.

### M005 — Pre-qualification safety and API corrective

Class: invariant/corrective.

Plan: `plans/implementation/verified-update-core/005-prequalification-safety-and-api-corrective.md`.

Objective: correct ownership proof, path-parent mutation ordering, verification-to-commit continuity, authenticity API truthfulness, transaction-result semantics, recovery evidence, transaction-owned permissions, lock-contract documentation, and stale capability docs before any consumer freezes the API.

Exit conditions:

- destructive replacement requires explicit `Owned` classification and foreign/unknown fail closed;
- absent-destination creation is explicit;
- no unchecked `create_dir_all` occurs on a live destination path;
- all staged members are revalidated/re-hashed under lock before backup;
- no unenforced authenticity-required state is commit-capable;
- cleanup disposition is no longer named as post-commit policy;
- rolled-back/recovery outcomes preserve phase/cause;
- recovery paths identify real retained evidence;
- Unix transaction state is explicitly private;
- stale-lock behavior is implemented or documented truthfully;
- docs match current capability.

### M006 — Core package qualification

Class: polish/infrastructure.

Plan: `plans/implementation/verified-update-core/006-core-package-qualification.md`.

Objective: qualify the corrected eggup-core as an independently consumable package.

Exit conditions:

- rustdoc complete for public API;
- cargo package/publish dry-run passes;
- MSRV evidence;
- dependency tree reviewed;
- no consumer-specific constants;
- no HTTP/TLS/service-manager dependency;
- representative single/bundle examples compile;
- platform-support claims match actual CI/native evidence.

### M007 — Deferred finalization and post-commit failure policy

Class: invariant/capability.

Plan: `plans/implementation/verified-update-core/007-post-commit-policy-and-deferred-finalization.md`.

Objective: implement ADR-0002's reserved `KeepInstalled | RollBack` boundary after a coherent new artifact generation becomes live but before backup finalization.

Exit conditions:

- one caller-supplied post-commit operation runs while the mutation lock and rollback evidence remain owned by the transaction;
- post-commit success finalizes normally;
- `KeepInstalled` retains the new generation while recording the failed post-commit check truthfully;
- `RollBack` restores and verifies the old generation;
- rollback failure yields `RecoveryRequired` with both causes and real retained evidence;
- existing immediate `commit()` behavior remains available;
- no service/network/product policy enters `eggup-core`;
- drop/interruption semantics for the chosen API shape are fail-safe and documented.

## 8. Cross-cutting requirements

### Storage and migration

Core should not require persistent state. Lock/rollback files are ephemeral transaction state with explicit cleanup/recovery rules.

### Protocol and compatibility

No network protocol. Public Rust API compatibility matters once first consumers adopt.

### Security

Path validation, permissions, link handling, bounded execution, lock ownership, and rollback are first-class.

### Concurrency and recovery

One writer per installation domain. Fault injection is mandatory.

### Observability

Typed events/errors; bounded strings; no secret-bearing URLs expected in core.

### Performance/resource use

Do not buffer large artifact bytes unnecessarily when file-backed staging suffices.

### Documentation

Every safety-sensitive public type documents what it proves and what it does not prove.

## 9. Verification strategy

- unit tests for pure validation;
- integration tests over temporary filesystem roots;
- exhaustive/fault-injected commit phase tests;
- OS-specific replacement tests;
- MSRV build/check;
- package boundary/dependency inspection;
- later consumer qualification.

## 10. Risks and decision points

- Windows running-image semantics may require a platform-specific commit primitive.
- Crash durability beyond process-level rollback may require a journal/receipt design; if persistent recovery semantics become public, open an ADR.
- Hard-link/symlink policy must be conservative until cross-platform evidence exists.
- Archive extraction belongs outside core unless a generic staged-member contract proves insufficient.

## 11. Completion definition

The roadmap closes when eggup-core safely supports one- and multi-member verified local transactions, fault-injected rollback/recovery is closed, the package is independently consumable, and ADR-0002 post-commit failure policy can retain or roll back a coherent newly installed generation before backup finalization.

## 12. Milestone status

| Milestone | Status | Implementation plan | Closure record | Blockers |
|---|---|---|---|---|
| M001 | closed | `plans/implementation/verified-update-core/001-repository-workspace-foundation.md` | `plans/closure/verified-update-core/001-status.md` | — |
| M002 | closed | `plans/implementation/verified-update-core/002-domain-and-prepared-transaction.md` | `plans/closure/verified-update-core/002-status.md` | — |
| M003 | closed | `plans/implementation/verified-update-core/003-transaction-commit-rollback.md` | `plans/closure/verified-update-core/003-status.md` | — |
| M004 | closed; post-closure findings feed M005 | `plans/implementation/verified-update-core/004-integrity-and-candidate-validation.md` | `plans/closure/verified-update-core/004-status.md` | — |
| M005 | closed | `plans/implementation/verified-update-core/005-prequalification-safety-and-api-corrective.md` | `plans/closure/verified-update-core/005-status.md` | — |
| M006 | closed | `plans/implementation/verified-update-core/006-core-package-qualification.md` | `plans/closure/verified-update-core/006-status.md` | — |
| M007 | ready | `plans/implementation/verified-update-core/007-post-commit-policy-and-deferred-finalization.md` | — | M005 + M006 closed |
