# Verified Update Core M005 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/verified-update-core/005-prequalification-safety-and-api-corrective.md`

Source roadmap: `plans/subsystems/verified-update-core-roadmap.md#M005--pre-qualification-safety-and-api-corrective`

Corrects post-closure findings from:

- `plans/closure/verified-update-core/002-status.md`
- `plans/closure/verified-update-core/003-status.md`
- `plans/closure/verified-update-core/004-status.md`

Reviewed repository baseline: `9f527beb20da585bc3cf56f45f1fd96fd418c124`
Implementation baseline: working tree at closure (see Implementation commits)

## Implementation commits/PRs

- M005 implementation commit (this pass): `feat: close core M005 safety/API corrective` (pending SHA at closure; see git log)
- No PR was required for this local implementation pass.

Breaking pre-1.0 API changes were explicitly authorized by the plan (no consumers yet).

## Requirement-to-evidence matrix

| M005 requirement | Evidence | Result |
|---|---|---|
| Replace `Ownership::Managed` with `Absent \| Owned \| Foreign \| Unknown` | `domain.rs` enum, `OwnershipVerifier` trait, no stored ownership in `ArtifactMember` | passed |
| Consumer-supplied ownership verification, sync/testable, errors→`Unknown`, re-run under lock, absent-create vs replace distinct | `CommitOwnership{verifier, absent}`, `AbsentPolicy::{AllowCreate, DenyCreate}`, `AbsentOnlyVerifier`, `ExactDigestVerifier`, `ExistingAsOwnedVerifier` (test-oriented), preflight + locked revalidation with flap detection | passed |
| No unchecked destination-parent creation; parents must exist, canonicalize beneath root, symlink fails closed, missing parent errors | `require_ready_parent` (no `create_dir_all` on live path), `revalidate_destination`, `missing_parent_fails_without_creating_directories`, `symlinked_parent_fails_closed` | passed |
| Harden stage/backup/lock permissions and naming; ownership/token before deletion | stage/backup dirs `0700`, lock file `0600`, staged files `0600` / `0700` (private executable mapping), collision-resistant `pid+nonce+nanos` names with create-new retry, `Stage::drop` ownership checks, `MutationLock` token-checked cleanup | passed |
| Retain digest; re-read/re-hash every staged member after lock before backup; reject `NotRequired` | `VerifiedTransaction::verified_digests`, `revalidate_staged_under_lock`, `ValidatedTransaction::commit` only path, `validate` rejects non-`Verified`, `staged_mutation_after_validation_fails_before_live_mutation`, `unverified_integrity_none_cannot_commit` | passed |
| Eliminate unenforced authenticity-required state | `AuthenticityRequirement` removed entirely; `ArtifactMember` has no authenticity field; docs state checksum-only; `checksum_only_state_is_explicit_in_docs` + `unverified_integrity_none_cannot_commit` | passed |
| Rename cleanup disposition; reserve `PostCommitFailurePolicy` for ADR-0002 `KeepInstalled \| RollBack` | `CleanupDisposition::{Cleaned, RetainedForRecovery}`, no `PostCommitFailurePolicy` type remains, `cargo grep` clean | passed |
| Structured failure phase/cause/member | `FailurePhase::{Lock, Ownership, StageRevalidation, Backup, Commit, Rollback, Finalize}`, `FailureCategory`, `FailureReport` (512-char bound), `TransactionReceipt::{failure, rollback_failure}`, every injected failure asserts phase/member | passed |
| Fix retained-recovery path reporting; never synthetic | finalize path returns real `backup_root`, `cleanup_failure_reports_real_backup_root` asserts `exists()` and no `cleanup-failed` synthetic suffix | passed |
| Truthful stale-lock support/documentation | fail-closed preserved; `MutationLock::inspect()->Available \| Held \| Malformed`, 4 KiB bound, never deletes, no PID-liveness auto-removal, documented in `transaction.md` + rustdoc | passed |
| Current capability documentation | root `README.md`, `crates/eggup-core/README.md`, `lib.rs` docs, `architecture/overview.md`, `docs/{domain,transaction,verification}.md`, `CHANGELOG.md` updated; stale foundation wording removed | passed |
| Expand tests and CI where practical | 37 tests (was 21), macOS + Windows-check lanes added to `ci.yml`, MSRV 1.89 lanes retained | passed |

## Public API before/after summary

Before:

- `Ownership::Managed` stored on `ArtifactMember` via `with_ownership`
- `AuthenticityRequirement::{None, Required}` + `with_authenticity`
- `PostCommitFailurePolicy::{Cleaned, RetainForRecovery}` via `receipt.cleanup()`
- `PreparedTransaction::commit()` / `commit_with_fault()` reachable without integrity/ownership
- `ValidatedTransaction::commit()` with no arguments
- `TransactionReceipt` without failure reports; cleanup failure returned synthetic `backup_root.join("cleanup-failed-…")`
- `MutationLock::{acquire, path}` only; no inspection API

After:

- `Ownership::{Absent, Owned, Foreign, Unknown}` (verifier return only, never stored)
- `OwnershipVerifier` trait + `CommitOwnership{verifier, absent}` + `AbsentPolicy::{AllowCreate, DenyCreate}`
- Helpers: `AbsentOnlyVerifier`, `ExactDigestVerifier`, `ExistingAsOwnedVerifier` (test-oriented, documented)
- No authenticity type; checksum-only contract
- `CleanupDisposition::{Cleaned, RetainedForRecovery}` via `receipt.cleanup()`
- `FailurePhase`, `FailureCategory`, `FailureReport`, `receipt.failure()`, `receipt.rollback_failure()`
- Only `ValidatedTransaction::commit(ownership)` is the production commit path (plus `#[cfg(test)] commit_with_fault`)
- `MutationLock::inspect()->LockStatus::{Available, Held, Malformed}`
- `ValidatedTransaction::{artifacts, staged_path}` added for inspection

## Exact tests/commands actually run

Environment: `Darwin 25.6.0 arm64`, stable `rustc 1.98.1`, MSRV `rustc 1.89.0`.

```text
rustc +1.89.0 --version
cargo +1.89.0 fmt --all -- --check
cargo +1.89.0 check --workspace --all-targets --locked
cargo +1.89.0 clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo +1.89.0 test --workspace --all-targets --all-features --locked
cargo +1.89.0 doc --workspace --no-deps --locked
cargo +1.89.0 tree --workspace --locked
cargo +1.89.0 package --workspace --locked --allow-dirty
./scripts/check-local.sh
```

Results:

- `fmt --check`: passed
- `check`: passed
- `clippy -D warnings`: passed on stable and 1.89 (one `overly_complex_bool_expr` found and fixed during pass)
- `test`: 37/37 passed on stable and 1.89
- `doc`: passed
- `tree`: `eggup-core` runtime deps remain `sha2` only; no HTTP/TLS/service-manager
- `package --allow-dirty`: 17 files, 146.9 KiB (30.7 KiB compressed), verification tests passed
- `./scripts/check-local.sh` (fmt + clippy + test + doc + tree): passed

Hosted CI: not run in this environment; `.github/workflows/ci.yml` now defines stable, MSRV 1.89, macOS, and Windows-check lanes. Do not infer hosted results.

## Invariant review

- No unverified or changed-after-verification bytes committed: `validate` requires all-`Verified`; `commit` re-hashes under lock; mutation test fails with zero live mutation.
- No candidate executes before required integrity: unchanged from M004, plus `NotRequired` can never reach `commit`.
- Destructive ops require `Owned` or explicitly authorized `Absent`: enforced in `revalidate_ownership_locked`; `Foreign`/`Unknown`/flapped fail closed.
- Destination containment proven before filesystem creation/mutation outside transaction state: `require_ready_parent` + `revalidate_destination` under lock; no `create_dir_all` on live paths.
- Rollback/recovery evidence never silently discarded: failure reports preserved; real backup roots retained; lock preserved on recovery.
- Transaction results preserve phase/cause: every terminal failure carries `FailureReport`; rollback failure preserves both causes.
- Type names match semantics: `CleanupDisposition` vs reserved ADR-0002 `PostCommitFailurePolicy`.
- No HTTP/TLS/service-manager in `eggup-core`: `cargo tree` confirms `sha2` only.
- No privilege escalation; no authenticity claim without verifier.

## Failure/recovery review

- Pre-mutation failures (ownership, stage revalidation): no live mutation, lock released, `RolledBack` with `failure` phase `Ownership`/`StageRevalidation`, `rollback_performed=false`, `rollback_verified=true`.
- Backup/commit failures: rollback attempted, `RolledBack` with original phase report when verified, else `RecoveryRequired` with both `failure` and `rollback_failure`.
- Rollback failure (`CommitThenRollback`): `RecoveryRequired`, lock + backup retained, both failure reports present.
- Finalize cleanup failure: `Committed` with `RetainedForRecovery`, real `backup_root`, `Finalize` phase report.
- Lock contention/malformed/oversized: `Err(UpdateInProgress)`, no deletion, `inspect` never deletes.
- Crash durability beyond process-level rollback remains out of scope (no journal claimed).

## Compatibility/migration review

No external consumers exist, so breaking changes required no migration. No deprecated aliases were added. `grep` confirms no `PostCommitFailurePolicy` or `AuthenticityRequirement` remains in code. Future consumers (M006 qualification, acquisition, adoption) must use `ValidatedTransaction::commit(ownership)` with an explicit verifier and `AbsentPolicy`.

## Security review

- Symlink, non-regular, hard-linked (Unix `nlink != 1`), escaped, missing-parent, and ambiguous-parent destinations fail closed under lock.
- Stage/backup `0700`, lock `0600`, staged files `0600`/`0700` (private executable mapping, never inherits broad source modes).
- Transaction names use `pid + atomic nonce + nanos` with create-new retry; cleanup verifies token/prefix/directory/symlink/parent before deletion.
- Bounded lock records (4 KiB), bounded failure detail (512 chars), bounded candidate execution (unchanged), no secret-bearing URLs in core.
- No hidden fallback or privilege escalation added.

## Docs/operations evidence

- `README.md`: verified-transaction capability, ownership, checksum-only
- `crates/eggup-core/README.md`: ownership, permissions, receipts, lock, checksum-only
- `lib.rs`: transaction-oriented docs, checksum-only notice
- `architecture/overview.md`: corrected capability prose + domain contract link
- `docs/transaction.md`: full corrected commit/ownership/receipt/lock contract
- `docs/verification.md`: `NotRequired` rejection + revalidation + checksum-only
- `docs/domain.md`: preparation, private perms, no stored ownership, helper policy
- `CHANGELOG.md`: M005 breaking summary
- `ci.yml`: macOS + Windows-check lanes added

## Unresolved findings with severity

- Low / accepted: Windows running-image replacement semantics remain unproven; `windows-check` lane is compile-only. Native Windows transaction evidence required before claiming full Windows support (feeds M006 platform matrix).
- Low / accepted: No persistent crash journal; process-crash recovery beyond backup evidence remains out of scope by design.
- Informational: Hosted Linux/macOS/Windows CI results were not available in this environment; lanes are defined but not evidenced here.
- None: No medium-or-higher safety/API issue remains open against the M005 scope.

## Disposition

M005 is closed. The corrected `Absent | Owned | Foreign | Unknown` ownership, parent safety, staged revalidation, truthful authenticity/cleanup/failure/lock semantics, private permissions, and current docs are implemented and verified (37/37 tests, full 1.89 + stable verification green).

Unblocks (per registry rule, upon committing this record + roadmap/registry updates):

- verified-update-core M006 package qualification
- acquisition-transport M001 seam
- service-lifecycle M001 ownership contract

Do not begin package publication or consumer migration until M006 qualifies the corrected API.
