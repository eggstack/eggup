# eggup-core

`eggup-core` is the policy-neutral local substrate for verified, multi-artifact
updates. It is intentionally transport-neutral: callers acquire artifacts,
prove destination ownership, and choose release and service policies; the core
provides bounded local validation, private staging, SHA-256 integrity
verification, bounded candidate validation, locked ownership and staged-digest
revalidation, and synchronous commit/rollback with structured receipts.

Ownership uses `Absent | Owned | Foreign | Unknown`. Destructive replacement
requires `Owned`, or `Absent` with explicit creation authorization;
`Foreign` and `Unknown` fail closed. No destination parent is created
automatically. Staged bytes are re-hashed under lock before any live mutation.
Terminal receipts preserve failure phase, category, member, post-commit check
failure, and real recovery evidence. `commit_with_post_commit` retains the
mutation lock and rollback set until one caller check succeeds or its explicit
`KeepInstalled | RollBack` policy is resolved. Locks are fail-closed with
inspection only and no automatic stale removal.

Integrity is checksum evidence only. No authenticity or signature verification
exists in this crate.
