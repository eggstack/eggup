# Verified Update Core Roadmap

Status: active; M001-M011 closed and hosted-green (run `37376971555`); M012 0.1.x API-compatibility corrective ready; M013 eggup-core 0.1.3 publication blocked on M012 + Acquisition M010

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

These were tracked and closed in M005, followed by M006 package qualification. A later planning review identified one accepted ADR-0002 capability that was intentionally reserved: the current successful `ValidatedTransaction::commit()` finalized backup state before caller post-install verification. M007 implemented and qualified the deferred-finalization boundary; see `plans/closure/verified-update-core/007-status.md`. M001-M006 closure records remain historical evidence and are not rewritten.

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
M007 deferred finalization + post-commit policy [closed]
        |
        +--> Archive M001d object-bound handoff [closed]
                    |
                    v
M008 core/archive consumer package qualification [CLOSED via M008a]
                    |
                    v
M009 core/archive 0.1.2 publication [CLOSED 2026-09-28]
        |
        +--> M010 current-executable transaction parity [CLOSED]
        |
        `--> M011 proof-authorized stale-lock recovery [CLOSED]

M010 + M011 closures
        |
        v
M012 0.1.x public-error compatibility corrective [READY]
        |
        +-------------------------+
        |                         |
        |                 Acquisition M010
        |                 workspace 0.1.3 + acquisition publish
        |                         |
        +------------+------------+
                     |
                     v
M013 eggup-core 0.1.3 publication [BLOCKED]
```

- M001 -> M002: hard.
- M002 -> M003: hard.
- M002 -> M004: hard.
- M003 + M004 -> M005: hard.
- M005 -> M006: hard.
- M005 + M006 -> M007: hard and satisfied.
- M007 + Archive M001d -> M008 package qualification: hard and satisfied.
- M008 + M008a -> M009 publication: hard and satisfied; `eggup-core 0.1.2`
  then `eggup-archive 0.1.2` published 2026-09-28, `v0.1.2` + GitHub Release
  `0.1.2` recorded in `plans/closure/verified-update-core/009-status.md`.
- M009 -> Consumer M006 / Egress Delivery M003: hard Eggup-side gate
  satisfied; consumer implementation is Egress-owned and now executable.
- Acquisition consumers may continue using the qualified immediate-commit path.
- Service Lifecycle M005/M006 are closed. Current Gregg/EggPool migration research exposed two remaining Core substrate gaps: current-executable parity (M010) and caller-authorized stale-lock recovery (M011). Both are additive correctives against already-normative requirements.

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

### M008 — Core/archive consumer package qualification

Class: infrastructure/polish.

Plan: `plans/implementation/verified-update-core/008-core-archive-consumer-package-qualification.md`.

Objective: qualify a versioned crates.io-usable package boundary containing the post-M001d `eggup-core` bound-source API plus `eggup-archive`, without automatic publication.

Exit conditions:

- versioning/pin decision is explicit and compatible with existing 0.1.1 consumers;
- `cargo package` and `cargo publish --dry-run` pass for core/archive;
- a clean external fixture consumer compiles the object-bound archive -> core staging flow;
- package contents/dependency trees/MSRV/platform evidence are recorded;
- manual publish order is explicit; no publication is automatic.

### M009 — eggup-core / eggup-archive 0.1.2 publication

Class: infrastructure/release operations.

Plan: `plans/implementation/verified-update-core/009-core-archive-0.1.2-publication.md`.

Objective: manually publish only `eggup-core 0.1.2` and `eggup-archive 0.1.2` from one clean, green release-prep commit; verify registry-only consumption; then create `v0.1.2`/GitHub Release and unblock Egress Delivery M003/M006.

Exit conditions:

- archive crate name/ownership and both exact version absences are proven before core upload;
- release changelogs distinguish the two published crates from unrelated workspace 0.1.2 source versions;
- release-prep commit is fully requalified;
- core publishes and is verified before archive publication;
- archive publishes from the same exact commit;
- a clean registry-only external fixture resolves both `=0.1.2` crates and exercises the M001d flow;
- `v0.1.2` and GitHub Release point to the exact publication commit;
- no other Eggup crate is published;
- Egress publication gate is reconciled to ready after closure.

### M010 — Current-executable transaction parity corrective

Class: invariant/capability corrective.

Plan: `plans/implementation/verified-update-core/010-current-executable-transaction-parity-corrective.md`.

Objective: make ADR-0002's one-member transaction model true for an executable that is currently running, without requiring write authority above the executable directory and with native Windows running-image rollback/finalization semantics.

Reference evidence: Gregg `1aac89f1` and EggPool `fe3c308d`.

Exit conditions:

- current-executable binding reuses the normal validated transaction/receipt model;
- executable-local staging requires no broader write authority than the executable directory;
- exact target/symlink identity is revalidated immediately before mutation;
- Windows native evidence proves running-image replacement;
- old-image rollback evidence remains addressable until `KeepInstalled | RollBack` resolves;
- ordinary bundle/multi-member behavior does not change.

### M011 — Proof-authorized stale-lock recovery

Class: invariant/capability corrective.

Plan: `plans/implementation/verified-update-core/011-proof-authorized-stale-lock-recovery.md`.

Objective: implement the specification's stale-lock recovery requirement without teaching Core consumer process policy.

Exit conditions:

- ordinary `MutationLock::acquire` remains fail-closed;
- an opt-in caller verifier can authorize recovery only for one exact observed lock;
- malformed/unknown records remain non-destructible;
- stale claiming is race-safe against pathname replacement and concurrent new writers;
- partial recovery preserves real evidence;
- EggPool's stronger stale-lock behavior is representable without importing its provenance/package-manager policy.

### M012 — 0.1.x public-error compatibility corrective

Class: API compatibility / release corrective.

Plan: `plans/implementation/verified-update-core/012-0.1x-public-error-compatibility-corrective.md`.

Status: ready.

Restore the published `eggup-core 0.1.2` `Error` variant set while retaining
M010/M011 semantics. Keep fault injection structural but out of the packaged
public error surface, and move stale-lock retained-evidence failures into a new
recovery-specific non-exhaustive error/result used only by the still-unpublished
M011 APIs. Prove compatibility with an unchanged exhaustive-match fixture and
with published `eggup-service 0.1.2`.

### M013 — eggup-core 0.1.3 publication and registry handoff

Class: package promotion / downstream handoff.

Plan: `plans/implementation/verified-update-core/013-eggup-core-0.1.3-publication.md`.

Status: closed 2026-10-06; `eggup-core 0.1.3` published, `v0.1.3` and release created.

Publish the hosted-qualified M010/M011 substrate plus the M012 compatibility
correction and earlier Core audit fixes. Registry-only direct and
`eggup-service 0.1.2 -> eggup-core 0.1.3` proofs are mandatory. M013 closure
is the versioned Core gate for authoring Gregg M004 and EggPool M007.

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

M010/M011 are implemented and hosted-green, but their current unpublished public
Error additions are not patch-compatible with the published 0.1.2 exhaustive enum.
M012 is the compatibility gate; M013 is the explicit registry handoff. The
already-qualified 0.1.2 core/archive pair remains published and no prior
publication/tag is reopened.

## 12. Milestone status

| Milestone | Status | Implementation plan | Closure record | Blockers |
|---|---|---|---|---|
| M001 | closed | `plans/implementation/verified-update-core/001-repository-workspace-foundation.md` | `plans/closure/verified-update-core/001-status.md` | — |
| M002 | closed | `plans/implementation/verified-update-core/002-domain-and-prepared-transaction.md` | `plans/closure/verified-update-core/002-status.md` | — |
| M003 | closed | `plans/implementation/verified-update-core/003-transaction-commit-rollback.md` | `plans/closure/verified-update-core/003-status.md` | — |
| M004 | closed; post-closure findings feed M005 | `plans/implementation/verified-update-core/004-integrity-and-candidate-validation.md` | `plans/closure/verified-update-core/004-status.md` | — |
| M005 | closed | `plans/implementation/verified-update-core/005-prequalification-safety-and-api-corrective.md` | `plans/closure/verified-update-core/005-status.md` | — |
| M006 | closed | `plans/implementation/verified-update-core/006-core-package-qualification.md` | `plans/closure/verified-update-core/006-status.md` | — |
| M007 | closed | `plans/implementation/verified-update-core/007-post-commit-policy-and-deferred-finalization.md` | `plans/closure/verified-update-core/007-status.md` | — |
| M008 | closed with clean package evidence (M008a); publication gate satisfied by M009 | `plans/implementation/verified-update-core/008-core-archive-consumer-package-qualification.md` | `plans/closure/verified-update-core/008-status.md` (M008a addendum) | — |
| M008a | closed | `plans/implementation/verified-update-core/008a-clean-package-evidence-corrective.md` | `plans/closure/verified-update-core/008a-status.md` | — (was: hard dependency on M002a green head; satisfied by run `36477024102`) |
| M009 | closed | `plans/implementation/verified-update-core/009-core-archive-0.1.2-publication.md` | `plans/closure/verified-update-core/009-status.md` | — (pair published 2026-09-28; Consumer M006 gate satisfied) |
| M010 | closed | `plans/implementation/verified-update-core/010-current-executable-transaction-parity-corrective.md` | `plans/closure/verified-update-core/010-status.md` | — (`CurrentExecutable` + `InstallPlan::for_current_executable`; `StagePlacement::InsideInstallationRoot`; Windows-native fixtures in CI) |
| M011 | closed | `plans/implementation/verified-update-core/011-proof-authorized-stale-lock-recovery.md` | `plans/closure/verified-update-core/011-status.md` | — (`LockObservation` + `StaleLockVerifier`; default `acquire` still fail-closed; claim/race fixtures run natively on Windows in CI) |
| M012 | closed | `plans/implementation/verified-update-core/012-0.1x-public-error-compatibility-corrective.md` | `plans/closure/verified-update-core/012-status.md` | `Error` keeps the seven published 0.1.2 variants; retained evidence moved to `RecoveryError`; byte-identical exhaustive-match fixture fails against pre-M012 main |
| M013 | closed | `plans/implementation/verified-update-core/013-eggup-core-0.1.3-publication.md` | `plans/closure/verified-update-core/013-status.md` | published from `bd43683` (run `37527706900` green); `v0.1.3` + release created; registry-only direct and `eggup-service 0.1.2 -> eggup-core 0.1.3` graphs green |
