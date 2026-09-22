# Verified Update Core Milestone 003 — Transaction Commit, Rollback, and Recovery

Status: implemented

Repository baseline for planning: `adaada4`

Source roadmap:

- `plans/subsystems/verified-update-core-roadmap.md#M003--mutation-lock-commit-rollback-and-recovery`

Long-term requirements:

- `plans/000-long-term-specification.md#11-locking-and-contention`
- `plans/000-long-term-specification.md#12-rollback-model`
- `plans/001-terminology-and-domain-model.md#19-transaction`

Applicable ADR:

- `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md`

Primary class: capability / invariant

## 1. Objective

Implement the first live-mutation state machine: exclusive mutation lock, destination revalidation, backup set, multi-member commit, rollback, recovery-required outcome, and structured receipt.

## 2. Why this milestone is not yet ready

Hard dependency: M002 closure defining PreparedTransaction and destination ownership is closed at `adaada4`.

Before handoff, update the repository baseline and reconcile the fault-injection surface created by M001/M002.

## 3. Current implementation evidence

EggPool provides strong prior art for stale update locks, immediate pre-rename ownership revalidation, backup image, post-install self-check, and rollback. Egress provides prior art for treating two binaries as one release unit.

Eggup must generalize those mechanics without copying EggPool's release/provenance policy.

## 4. Invariants that must not regress

- one writer per installation domain;
- ambiguous stale lock fails closed;
- all candidates were prepared before lock-induced destructive work;
- destination ownership is revalidated immediately before first mutation;
- backup state exists before replacing any pre-existing member required for rollback;
- success means every member reached the intended generation;
- rollback failure is distinct from ordinary update failure;
- no automatic privilege escalation.

## 5. Scope

### In scope

- MutationLock and record;
- stale-lock proof;
- backup naming/storage inside safe same-filesystem scope;
- commit ordering;
- one- and multi-member commit;
- copy/rename handling across filesystem boundaries where explicitly supported;
- rollback state machine;
- RecoveryRequired;
- cleanup/finalization;
- TransactionReceipt;
- PostCommitFailurePolicy representation;
- deterministic failure injection at every mutation phase.

### Explicitly out of scope

- HTTP;
- checksums/candidate execution if M004 not yet closed;
- service restart;
- package-manager fallback;
- archive extraction;
- crash-restart persistence stronger than the implemented lock/backup contract unless separately planned.

## 6. Required production changes

### Mutation lock

Record bounded installation/product identity and process evidence. Use create-new/exclusive semantics.

Stale removal requires proof. A malformed, oversized, live, mismatched, or ambiguous record is not stale.

### Destination revalidation

Re-check file type, ownership/link policy, and authorized path immediately before destructive work.

### Backup set

Back up all existing members that must be restorable before installing new members.

Avoid reusable predictable backup paths that can be precreated by another principal.

### Commit

Commit the entire ArtifactSet.

Prefer same-filesystem rename. If cross-device transfer is supported, stage the copy into the destination filesystem before the final commit point so copy is not mistaken for atomic replacement.

### Rollback

On partial commit failure, restore all old members and remove newly committed members that did not exist before.

Record whether restoration was fully verified.

### Receipt

Return typed terminal disposition, including whether rollback occurred and whether manual recovery is required.

## 7. Ordered work packages

A. MutationLock and contention tests.

B. BackupSet and owned backup lifecycle.

C. Single-member commit.

D. Multi-member commit generalized from the same engine.

E. Fault-injected rollback matrix.

F. RecoveryRequired and receipt/finalization.

G. Platform-specific replacement seam, especially Windows running-image behavior.

## 8. Failure, cancellation, restart, and contention semantics

- pre-lock/pre-commit failure: no live mutation;
- second concurrent mutator: deterministic UpdateInProgress/locked result;
- partial backup failure: restore any moved members before returning;
- partial commit failure: rollback all affected members;
- rollback failure: preserve recovery evidence and return RecoveryRequired;
- caller cancellation during an in-process critical section must not intentionally abandon a half-mutated set; if async is not needed, keep core commit synchronous to simplify this guarantee;
- process crash durability is only what the documented filesystem backup contract proves. Do not claim automatic crash recovery unless implemented/tested.

## 9. Compatibility and migration

No consumer migration yet. Public receipt/error shapes may become consumer-facing, so document them carefully.

## 10. Required tests

A matrix must inject failure:

- lock creation;
- each backup operation;
- after all backups before first commit;
- each member commit;
- after final member before finalization;
- rollback of each member;
- backup cleanup.

Also test:

- concurrent lock attempts;
- stale live PID/foreign executable ambiguity;
- one-, two-, and three-member success;
- pre-existing absent member;
- destination replaced/raced between preparation and commit;
- symlink/hard-link negative cases;
- permission denied;
- Windows-specific path when environment available.

## 11. Required verification commands

Focused fault tests plus the complete M001 verification suite. Native platform evidence is recorded separately.

## 12. Documentation updates

Document exact atomicity limits, rollback guarantees, and RecoveryRequired operator meaning. Avoid the word "atomic" where the implementation is recoverable but not literally one filesystem atomic operation.

## 13. Acceptance criteria

- success implies coherent complete artifact set;
- all deterministic injected failures produce a known old/new/recovery state;
- no known failure path silently deletes the only old copy;
- lock contention is safe;
- no privilege escalation.

## 14. Stop conditions

Stop if:

- M002 cannot prove exact destinations;
- Windows replacement semantics require a materially different public transaction model;
- crash-safe journaling becomes necessary to meet claimed guarantees;
- cross-filesystem behavior cannot be made truthful under the documented contract.

## 15. Closure evidence required

- fault matrix table;
- exact old/new bytes after each injected failure;
- lock contention/stale evidence;
- platform results;
- transaction receipt examples;
- unresolved durability limitations.

## 16. Handoff notes

Do not integrate service restart into this state machine. Service lifecycle composes around a prepared transaction later.
