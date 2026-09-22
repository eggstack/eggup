# Verified Update Core M003 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/verified-update-core/003-transaction-commit-rollback.md`

Source roadmap: `plans/subsystems/verified-update-core-roadmap.md#M003--mutation-lock-commit-rollback-and-recovery`

Reviewed repository baseline: `6b1484d`

## Implementation commits/PRs

- `f5765a5` (`feat: add transaction commit rollback and recovery`)
- No PR was required for this local implementation pass.

## Executive finding

M003 is complete. A prepared transaction can now synchronously acquire an
exclusive installation lock, revalidate destinations, back up existing
members, commit a complete one- or multi-member set, restore old state after a
partial failure, and return a typed terminal receipt. Rollback failure and
post-commit cleanup failure retain recovery evidence instead of silently
discarding the only known backup.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| One writer per installation domain | create-new `MutationLock` and contention test | passed |
| Ambiguous lock records fail closed | malformed lock test and bounded create-new record | passed |
| Destination ownership revalidated before mutation | `revalidate_destination` and link/hard-link tests | passed |
| Backup before replacing existing members | sibling backup set and old-byte assertions | passed |
| One- and multi-member success | complete two-member commit test | passed |
| Partial commit rollback | injected member failure restores both old members | passed |
| Absent old member handling | injected failure removes newly created absent member | passed |
| Backup/pre-commit failure handling | injected matrix test | passed |
| Rollback failure distinction | recovery-required test with retained lock/evidence | passed |
| Structured terminal receipt | `TransactionReceipt` and disposition assertions | passed |
| No privilege escalation | synchronous local filesystem operations only | passed |

## Production implementation evidence

`MutationLock` uses create-new semantics, bounded self-authored records, and
token-checked cleanup. `PreparedTransaction::commit` keeps all destructive work
synchronous. Existing regular destinations are renamed into a unique sibling
backup set before staged members are renamed into place. The same engine handles
single members and bundles. `RecoveryRequired` preserves the lock and backup
path for manual recovery.

## Fault matrix evidence

| Injected phase | Expected terminal state | Evidence |
|---|---|---|
| lock creation | no mutation / error | `injected_lock_creation_failure_performs_no_mutation` |
| each backup/member pre-commit | old bytes restored | `precommit_and_backup_failures_restore_old_state` |
| before first commit | old bytes restored | same test |
| member commit | old bytes restored | `partial_commit_failure_restores_every_old_member` |
| absent-member commit | new member removed | `rollback_removes_new_members_that_were_absent_before_commit` |
| rollback of a member | recovery required and evidence retained | `rollback_failure_returns_recovery_required_and_retains_evidence` |
| finalization cleanup | committed state with retained evidence | `Finalize` path in transaction engine |

## Exact commands run and results

Environment: `Darwin 25.6.0 x86_64`, Rust 1.89.0.

```text
rustc +1.89.0 --version
cargo +1.89.0 fmt --all -- --check
cargo +1.89.0 check --workspace --all-targets --locked
cargo +1.89.0 clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo +1.89.0 test --workspace --all-targets --all-features --locked
cargo +1.89.0 doc --workspace --no-deps --locked
cargo +1.89.0 tree --workspace --locked
./scripts/check-local.sh
```

The final suite passed 16/16 tests. Native Windows and Linux evidence was not
run in this environment; no result is inferred for those platforms.

## Invariant review

- All candidates are prepared before lock-induced destructive work.
- Destination type, hard-link count on Unix, and canonical ancestor are
  rechecked before backup and again before replacement.
- Existing members are retained in a uniquely named backup set before rename.
- Successful commit reaches every member; partial failure returns either a
  verified rollback or `RecoveryRequired`.
- No service restart, network access, or privilege escalation is present.

## Failure/rollback/recovery review

The fault matrix covers lock, backup, pre-commit, per-member commit, rollback,
and finalization paths. A rollback failure leaves the backup directory and lock
record. The implementation does not claim crash-safe journaling: process-crash
recovery beyond filesystem backup evidence remains a documented limitation.

## Compatibility and migration review

No existing consumer migration is required. The receipt and disposition types
are transport- and consumer-policy-neutral. No service lifecycle or release
ordering API was introduced.

## Security review

Symlink, non-regular, hard-linked, escaped, and ambiguous-lock cases fail
closed. Lock cleanup verifies ownership before unlinking. Backup paths are
unique per transaction and live under the explicit installation root. There is
no automatic privilege escalation.

## Documentation/operations evidence

`crates/eggup-core/docs/transaction.md` documents lock, backup, commit,
rollback, recovery, and atomicity limits. Recovery receipts retain an operator
path. The local verification script and ordinary CI remain active.

## Unresolved findings with severity

- Medium / accepted limitation: stale or malformed lock records are never
  auto-removed; an operator must establish ownership and remove them manually.
  This is fail-closed behavior and avoids unsafe PID-reuse assumptions.
- Medium / accepted limitation: no crash journal or automatic process-restart
  recovery is claimed. A future durability contract requires a separate ADR or
  corrective plan.
- Informational: Windows native replacement behavior was not exercised and
  remains outside this macOS evidence set.

## Disposition and roadmap transition

M003 is closed. M004 can proceed against implementation baseline `f5765a5` and
remains dependency-ready. M005 remains blocked on both M003 and M004.

