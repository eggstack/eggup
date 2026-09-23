# Commit, rollback, and recovery contract

`ValidatedTransaction::commit(ownership)` is synchronous and is the only
production commit path. It classifies every destination before locking
(preflight), acquires a create-new lock, re-proves ownership under lock,
re-hashes every staged member against its verified digest, and only then moves
existing members into an owned sibling backup set and renames staged members
into their exact destinations. The artifact set is the mutation unit;
a single-member update uses the same engine as a bundle.

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
`CleanupDisposition::{Cleaned, RetainedForRecovery}` describes temporary
evidence only. `commit_with_post_commit` is the ADR-0002 boundary: after all
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
before backup. Lock records are bounded (4 KiB), owner-private (0600), and
malformed, oversized, or ambiguous records are never auto-removed.
`MutationLock::inspect` reports `Available | Held | Malformed` without
deleting anything; stale removal requires manual operator action with
deployment-specific process evidence.
