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
`KeepInstalled | RollBack` policy is resolved.

A program can replace the executable it is currently running through the same
transaction model: `InstallPlan::for_current_executable` binds the canonical
live image, plans `StagePlacement::InsideInstallationRoot` so no write authority
above that directory is ever needed, and re-proves the image's identity under
the lock immediately before the first live rename. An invocation symlink is
followed to its real target and never overwritten; a hard-linked or non-regular
destination is refused. On Windows the previous generation stays mapped until the
process exits, so `KeepInstalled` reports
`CleanupDisposition::DeferredToProcessExit` and schedules its removal, rather
than calling it cleaned or stranded.

Locks are fail-closed by default: `MutationLock::acquire` never recovers a stale
record and `inspect` is read-only. Recovery is opt-in through
`MutationLock::acquire_with_recovery`, which displaces a record only when a
caller-supplied `StaleLockVerifier` returns `StaleLockDecision::ProvenStale`
for the exact observation Core reported. That opt-in API returns
`RecoveryResult<T>`: ordinary failures arrive as `RecoveryError::Core(Error)`,
and retained evidence as `RecoveryError::RecoveryRequired`, which carries the
real path via `RecoveryError::evidence()`. Default `acquire` keeps returning
`Result<T, Error>`. Core never decides staleness from PID
liveness, record age, executable name, or service state; see
`examples/stale_lock_recovery.rs`.

Integrity is checksum evidence only. No authenticity or signature verification
exists in this crate.
