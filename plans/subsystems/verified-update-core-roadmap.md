# Verified Update Core Roadmap

Status: active

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
- no implicit privilege escalation;
- eggup-core carries no HTTP/TLS/service-manager dependency.

### Capabilities

- prepare and commit a one-member deployment;
- prepare and commit a multi-member deployment;
- recover the previous deployment after injected commit failures;
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

The Eggup repository starts without production code.

Relevant proven source patterns exist across Eggstack:

- Gregg's `gregg-update` demonstrates a useful shared updater boundary, private staging, checksum verification, candidate identity checks, and self replacement.
- EggPool has stronger mutation-lock, ownership revalidation, rollback, and post-install self-check behavior.
- Egress already treats two binaries as one logical transaction with backup/restore.
- eggsact, stegoeggo, and eggsearch duplicate local integrity/candidate/staging behaviors.

The subsystem should extract the common mechanics rather than copy any one consumer wholesale.

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
M005 core package qualification
```

- M001 -> M002: hard.
- M002 -> M003: hard.
- M002 -> M004: hard.
- M003 + M004 -> M005: hard.
- Acquisition adapter can begin against M002's stable local-artifact interface as an interface dependency.

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

### M005 — Core package qualification

Class: polish/infrastructure.

Objective: qualify eggup-core as an independently consumable package.

Exit conditions:

- rustdoc complete for public API;
- cargo package/publish dry-run passes;
- MSRV evidence;
- dependency tree reviewed;
- no consumer-specific constants;
- no HTTP/TLS/service-manager dependency;
- representative single/bundle examples compile.

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

The roadmap closes when eggup-core safely supports one- and multi-member verified local transactions, fault-injected rollback/recovery is closed, and the package is independently consumable.

## 12. Milestone status

| Milestone | Status | Implementation plan | Closure record | Blockers |
|---|---|---|---|---|
| M001 | closed | `plans/implementation/verified-update-core/001-repository-workspace-foundation.md` | `plans/closure/verified-update-core/001-status.md` | — |
| M002 | closed | `plans/implementation/verified-update-core/002-domain-and-prepared-transaction.md` | `plans/closure/verified-update-core/002-status.md` | — |
| M003 | ready | `plans/implementation/verified-update-core/003-transaction-commit-rollback.md` | — | — |
| M004 | ready | `plans/implementation/verified-update-core/004-integrity-and-candidate-validation.md` | — | — |
| M005 | planned | — | — | M003, M004 |
