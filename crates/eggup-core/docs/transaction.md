# Commit, rollback, and recovery contract

`PreparedTransaction::commit` is synchronous. It acquires a create-new lock in
the installation root, revalidates every destination immediately before
destructive work, moves existing members into an owned sibling backup set, and
renames staged members into their exact destinations. The artifact set is the
mutation unit; a single-member update uses the same engine as a bundle.

If a backup or commit step fails, the engine restores every moved old member
and removes newly installed members that were absent before the attempt. The
receipt distinguishes `Committed`, `RolledBack`, and `RecoveryRequired` and
reports whether restoration was verified. A rollback or cleanup failure keeps
the backup evidence and lock record for manual recovery. The implementation
does not claim crash-safe journaling or literal filesystem-wide atomicity.

Existing symlink, non-regular, hard-linked, or escaped destinations fail closed
before backup. Lock records are bounded and malformed or ambiguous records are
never treated as stale automatically.

