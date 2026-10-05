# Commit, rollback, and recovery contract

`ValidatedTransaction::commit(ownership)` is synchronous and is the only
production commit path. It classifies every destination before locking
(preflight), acquires a create-new lock, re-proves ownership under lock,
re-hashes every staged member against its verified digest, and only then moves
existing members into an owned sibling backup set and renames staged members
into their exact destinations. The artifact set is the mutation unit;
a single-member update uses the same engine as a bundle.

`InstallPlan::prepare_with_bound_sources(bound)` is the additive staging
seam used by archive callers. Members present in `bound` are staged by
reading the moved open object from byte zero, so a member-entry replacement
or root rename after the handoff boundary cannot redirect staged bytes.
Members absent from `bound` stage from their recorded source paths exactly
as in `prepare`. Every bound identity must name a member of this plan;
leftover handles fail closed and no staged byte is ever obtained by
reopening a member name as fallback.

Ownership uses `Absent | Owned | Foreign | Unknown`. `Owned` allows
replacement; `Absent` allows installation only with
`AbsentPolicy::AllowCreate`; `Foreign` and `Unknown` fail closed, as does any
change between preflight and locked classification. Destination parents must
already exist beneath the installation root; missing parents fail without
creation, and symlinked or otherwise ambiguous parents fail closed. Staged
bytes that changed after verification fail with `StageRevalidation` before any
live mutation.

If a backup or commit step fails, the engine restores every moved old member
and removes newly installed members that were absent before the attempt. The
receipt distinguishes `Committed`, `RolledBack`, and `RecoveryRequired`,
reports whether restoration was verified, and always carries the triggering
`FailureReport` (phase, member, category, bounded detail). Rollback failure
additionally carries `rollback_failure` while preserving the original cause.
`CleanupDisposition::{Cleaned, RetainedForRecovery, DeferredToProcessExit}`
describes temporary evidence only. `DeferredToProcessExit` is reported only by a
current-executable self-update kept installed on Windows: the previous generation
is still mapped by this process, so its deletion is scheduled rather than
performed, and nothing is stranded for an operator. `commit_with_post_commit` is the ADR-0002 boundary: after all
members are live, its one caller check runs while the mutation lock and backup
remain held. Success finalizes normally. On failure, `KeepInstalled` finalizes
the backup and records `post_commit_failure` while retaining `Committed`;
`RollBack` restores and verifies the previous generation. A rollback failure
returns `RecoveryRequired`, preserving the post-commit cause, rollback cause,
real backup path, and lock record. Callback errors are copied into bounded
core-owned reports; callback panics are treated as failed checks and resolved
by the selected policy. The callback owns its own time bound. Process
termination and power loss are outside this guarantee. Cleanup failure keeps
the real backup root and lock record; no synthetic path is returned. The
implementation does not claim crash-safe journaling or literal filesystem-wide
atomicity.

Existing symlink, non-regular, hard-linked, or escaped destinations fail closed
before backup.

A program may replace the executable it is currently running through this same
transaction model. `InstallPlan::for_current_executable` binds the canonical
live image and plans `StagePlacement::InsideInstallationRoot`, so the private
stage and the backup set stay inside the executable's own directory and no write
authority above it is ever required. An invocation symlink is followed to its
real target and never overwritten; a destination whose exact identity cannot be
proven is refused. That identity is re-proved under the mutation lock and again
immediately before the first live rename. The old generation is renamed aside
rather than deleted, so it stays rollback-addressable until the caller's policy
resolves.

Lock records are bounded (4 KiB), owner-private (0600), and malformed, oversized,
symlinked, non-regular, non-UTF-8, or otherwise ambiguous records are never
auto-removed. `MutationLock::inspect` reports `Available | Held | Malformed`
without deleting anything, and `MutationLock::acquire` never recovers a record.
Stale recovery is opt-in per commit through `acquire_with_recovery` and a
caller-supplied `StaleLockVerifier`: Core exposes a bounded `LockObservation` of
one exact record and displaces it only on `StaleLockDecision::ProvenStale`,
binding the authorization to that observation's exact bytes. Core decides no
staleness from PID liveness, record age, executable name, or service state. A
record replaced between observation and claim is never deleted, and a writer that
creates the lock after the claim always wins.
