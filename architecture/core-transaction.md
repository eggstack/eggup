# eggup-core — Core Transaction Deep Dive

Source: `crates/eggup-core` (`src: lib.rs, domain.rs, stage.rs, integrity.rs, candidate.rs, transaction.rs, lock.rs, error.rs`; `docs/domain.md, docs/transaction.md, docs/verification.md`; `README.md`; `Cargo.toml`).
Crate description (`Cargo.toml:10`): “Policy-neutral local mechanics for verified multi-artifact updates (checksum integrity only, no transport or service manager)”. Sole dependency: `sha2 = "0.10"`. `#![forbid(unsafe_code)]`, `#![deny(missing_docs)]`.

## 1. Purpose and non-goals / boundary

Purpose (`lib.rs:3-10`, `README.md:3-8`): policy-neutral local substrate for verified multi-artifact updates. Caller owns acquisition, destination-ownership proof, release/service policy. Core provides:

- Bounded local validation (`InstallPlan::new`).
- Private staging (`Stage::prepare`).
- SHA-256 integrity verification (`verify_integrity`).
- Bounded candidate validation (`validate` + `run_bounded`).
- Locked ownership + staged-digest revalidation.
- Synchronous commit/rollback with structured `TransactionReceipt`.
- Post-commit boundary (`commit_with_post_commit`, ADR-0002).

Explicit non-goals (all stated in code/docs):

- No transport/fetch: sources must already be acquired absolute local paths; `ArtifactMember::new(id, source, destination)` never fetches (`domain.rs:292-306`, `domain.md:5-8`).
- No service manager: nothing starts/stops/supervises services.
- No release/fallback/ordering policy: `AllValidators` is empty-by-default composition; `ExactIdentityValidator` / `CrossMemberAgreementValidator` are opt-in helpers; “No fallback or release ordering is selected by core” (`verification.md:37`).
- No authenticity: “Integrity here is checksum evidence only. No authenticity or signature claim is made” (`lib.rs:11-12`, `README.md:20-21`, `domain.rs:123-127`, `integrity.rs` sidecar docs). There is no signature verifier type or API.
- No journaling / crash-safe atomicity: “does not claim crash-safe journaling or literal filesystem-wide atomicity” (`transaction.md:37-39`). Mutation unit is per-artifact `fs::rename` + backup restore, not a filesystem journal. Process termination / power loss during post-commit check is outside the guarantee (`candidate.rs:464`, `transaction.md:36`).
- No automatic stale-lock recovery: locks are fail-closed, inspection-only, manual operator removal (`lock.rs:34-42`, `transaction.md:43-46`).
- No destination-parent creation: missing parents fail; `require_ready_parent` never calls `create_dir_all` on live paths (`transaction.rs:782-788`).
- No unbounded outputs in receipts: `FailureReport` detail and post-commit callback error display are truncated to 512 bytes/chars (`transaction.rs:141-158`, `candidate.rs:516-532`).
- No invented post-commit timeout: “The callback must itself bound its work; core invents no timeout” (`candidate.rs:464`).

## 2. Public API surface and state machine

Re-exports (`lib.rs:25-44`):

- `candidate`: `run_bounded`, `AllValidators`, `CandidateValidator`, `CommandOutput`, `CommandSpec`, `CrossMemberAgreementValidator`, `ExactIdentityValidator`, `ValidatedTransaction`.
- `domain`: `AbsentOnlyVerifier`, `AbsentPolicy`, `ArtifactMember`, `ArtifactSet`, `CommitOwnership`, `ExactDigestVerifier`, `ExistingAsOwnedVerifier`, `FileKind`, `InstallPlan`, `IntegrityRequirement`, `MemberId`, `Ownership`, `OwnershipVerifier`, `PermissionsIntent`, `ProductId`, `ReleaseId`.
- `error`: `Error`, `Result`.
- `integrity`: `hash_file`, `parse_sha256_sidecar`, `verify_file`, `IntegrityResult`, `IntegrityStatus`, `Sha256Manifest`, `VerifiedTransaction`.
- `lock`: `LockStatus`, `MutationLock`.
- `stage`: `PreparedTransaction`.
- `transaction`: `CleanupDisposition`, `FailureCategory`, `FailurePhase`, `FailureReport`, `PostCommitFailurePolicy`, `TransactionDisposition`, `TransactionReceipt`.

State machine (`docs/verification.md:13-22`):

```text
InstallPlan --prepare()--> PreparedTransaction
  --verify_integrity()--> VerifiedTransaction
    --validate(&dyn CandidateValidator)--> ValidatedTransaction
      --commit(CommitOwnership)--> Result<TransactionReceipt>
      --commit_with_post_commit(CommitOwnership, PostCommitFailurePolicy, FnOnce()->Result<(),E>)--> Result<TransactionReceipt>
```

Key methods:

- `InstallPlan::new(product, release, installation_root, artifacts) -> Result<Self>` (`domain.rs:415-441`); `plan.prepare() -> Result<PreparedTransaction>` (`domain.rs:473-476`); accessors `product/release/installation_root/artifacts/destination(member)`.
- `PreparedTransaction::{product, release, artifacts, destination, staged_path, stage_root}` (`stage.rs:80-115`); `verify_integrity(self) -> Result<VerifiedTransaction>` (`integrity.rs:171-208`).
- `VerifiedTransaction::{artifacts, product, release, stage_root, staged_path, integrity(member)}` (`integrity.rs:210-248`); `validate(self, validator) -> Result<ValidatedTransaction>` (`candidate.rs:411-424`).
- `ValidatedTransaction::{artifacts, staged_path}` + `commit(self, ownership)` + `commit_with_post_commit(self, ownership, policy, check)` (`candidate.rs:426-480`). Test-only `commit_with_fault` / `commit_with_post_commit_fault` (`candidate.rs:485-513`, `transaction.rs:294-304` `CommitFault`).
- `TransactionReceipt::{product, release, disposition, rollback_performed, rollback_verified, cleanup, recovery_path, failure, rollback_failure, post_commit_failure}` (`transaction.rs:229-292`).

`Result<T> = Result<T, Error>` is terminal-receipt vs hard-error split: pre-lock injected lock-creation failure returns `Err`; once the lock path is entered, ownership/stage/backup/commit failures return `Ok(receipt)` with `RolledBack` / `RecoveryRequired` and structured reports. Only unrecoverable setup errors (`LockCreation` fault, lock `acquire` contention) surface as `Err(Error)`.

## 3. Per-module breakdown

### domain.rs — identities, members, plan, ownership contract

- `ProductId / ReleaseId / MemberId`: opaque validated strings; reject empty or control-character input (`validate_identifier`, `domain.rs:75-82`). `Display` + `as_str`.
- `FileKind::Regular` (non-exhaustive): only regular files accepted; directories, links, devices, sockets, FIFOs rejected (`domain.rs:84-90`, check in `validate_source` via `symlink_metadata.is_file()`).
- `PermissionsIntent::{Preserve, Executable}` (non-exhaustive): staged output always owner-private (`0600`, `0700` if executable). `Preserve` maps source `+x` to `0700` else `0600`; broad modes never inherited (`domain.rs:92-104`, `stage.rs:175-204`).
- `Ownership::{Absent, Owned, Foreign, Unknown}`: `Owned` never inferred from plan; only returned by consumer `OwnershipVerifier` (`domain.rs:106-121`).
- `IntegrityRequirement::{None, Sha256([u8;32])}` (non-exhaustive): `None` visible at prepare but never commit-capable (`domain.rs:123-135`).
- `OwnershipVerifier` trait: synchronous, deterministic for fixed filesystem state, errors/ambiguity must map to `Unknown`/hard failure, never `Owned`; re-run under lock; flap fails transaction (`domain.rs:137-150`, `transaction.rs:624-654`).
- `AbsentPolicy::{AllowCreate, DenyCreate}` + `CommitOwnership{verifier, absent}` (`domain.rs:152-188`): creation vs replacement are distinct authorizations.
- Helpers: `AbsentOnlyVerifier` (`Absent` if missing else `Foreign`; `Unknown` on other IO), `ExistingAsOwnedVerifier` (test/examples only; `Owned` for existing regular file — docs warn production should use `ExactDigestVerifier`), `ExactDigestVerifier{expected: HashMap<MemberId,[u8;32]>}` (`Absent`/`Owned` on hash match/`Foreign` on mismatch incl. `nlink!=1` on unix/`Unknown` on unreadable or unknown member) (`domain.rs:190-273`).
- `ArtifactMember{id, source, destination, file_kind, permissions, integrity}`: `new` normalizes destination via `normalize_relative_path`; `with_permissions/with_integrity` builders; accessors (`domain.rs:275-353`).
- `ArtifactSet(Vec<ArtifactMember>)`: rejects empty, duplicate member ids (`domain.rs:359-402`); `single` helper.
- `InstallPlan{product, release, installation_root, artifacts}`: `new` requires absolute installation root that `symlink_metadata` says is a dir; canonicalizes root; rejects duplicate normalized destinations; per-member `validate_source`: source absolute, `symlink_metadata` regular file (so source symlinks rejected), canonical source not under canonical root, canonical source != joined destination (`domain.rs:413-534`). `destination(member)` joins exact root (`domain.rs:464-471`). `normalize_relative_path` rejects empty/absolute, strips `CurDir`, rejects `ParentDir/RootDir/Prefix`, rejects control chars, rejects empty normalization (`domain.rs:536-561`).

### stage.rs — private staging

- `Stage{path, parent}` (crate-private) + `NEXT_STAGE_ID` atomic. `prepare(plan)` / test `prepare_with_failure` map `StageCreate/StageCopy` to `FailureAt::{Create, Copy}` (`stage.rs:17-43`).
- `prepare_inner`: `check_failure(Create)`; `create_stage_directory`; on `copy_members` error, `drop(stage)` cleans up then returns `Err`.
- `copy_members`: per member `check_failure(Copy)`; `create_dir_all` staged parent + `0700` (unix); `fs::copy(source, staged)`; `apply_permissions` (`stage.rs:45-65`).
- `create_stage_directory(root)`: parent = `root.parent()`; name `.eggup-stage-{rootname}-{pid}-{seq}-{nanos}`; `fs::create_dir` retry up to 32 on `AlreadyExists`; `0700` (unix) (`stage.rs:139-173`). Sibling of install root, not inside it.
- `apply_permissions`: `Executable => 0700`; `Preserve => 0700` if staged mode has any `0o111` else `0600` (unix); no-op otherwise (`stage.rs:175-204`).
- `PreparedTransaction{plan, stage}`: read-only accessors; `staged_path` resolves via member id or `UnknownMember`; `stage_root` for diagnostics/validation (`stage.rs:72-115`). No live mutation here (test asserts destinations untouched, `lib.rs:133-172`).
- `Drop for Stage`: removes only what it owns — filename prefix `.eggup-stage-`, `symlink_metadata` is real dir (not symlink), parent equals recorded parent — then `remove_dir_all` (`stage.rs:117-137`).

### integrity.rs — checksum evidence

- `hash_file(path)`: `File::open` + `Sha256` streaming with 64 KiB buffer (`integrity.rs:65-80`).
- `Sha256Manifest{digest, filename}` + `parse_sha256_sidecar`: trims, keeps non-empty lines, requires exactly 1; `splitn(2, whitespace)`; `decode_digest` requires 64 hex chars; optional filename strips leading `*`, rejects empty/control/multi-component (`file_name != value`) (`integrity.rs:83-114`).
- `verify_file(path, manifest)`: optional exact filename binding (`actual != expected` fails); `hash_file` compare; mismatch fails (`integrity.rs:117-137`).
- `IntegrityStatus::{Verified, NotRequired}`; `IntegrityResult{member, digest, status}` with accessors (`integrity.rs:30-62`).
- `PreparedTransaction::verify_integrity`: per member `staged_path`, `symlink_metadata.is_file` else `VerificationFailed`, `hash_file`, match `None => NotRequired`, `Sha256(expected==digest) => Verified` else `VerificationFailed` mismatch (`integrity.rs:169-208`).
- `VerifiedTransaction{prepared, results}`: accessors + `integrity(member)` lookup + crate `verified_digests()` filtering `Verified` only (`integrity.rs:210-248`).

### candidate.rs — bounded execution and validation gate

- `CommandSpec{program, args, current_dir, timeout, max_output_bytes, environment}`: `new(program)` never invokes shell; builders `arg/args/current_dir/timeout/max_output_bytes/environment`. Defaults `5s`, `64 KiB` (`candidate.rs:14-80`).
- `CommandOutput{exit_code, stdout, stderr, timed_out, output_limited}` + `success() = !timed_out && !output_limited && exit==Some(0)` (`candidate.rs:82-122`).
- `run_bounded(spec)`: `Command` with `env_clear`, `stdin null`, piped out/err, optional `current_dir`, explicit envs; `spawn`; reader threads `read_limited` (4096-byte chunks, `> limit` => `Err`, surfaced as `output_limited`); poll loop: `try_recv` stdout/stderr, `try_wait`, `kill` on `output_limited` or deadline expiry (`timed_out=true`), `wait` reap; sleep 2 ms; join threads when clean (`candidate.rs:125-210`). Env-clear proven by test (`EGGUP_SECRET` => `unset`, `lib.rs:1004-1014`).
- `CandidateValidator` trait: `validate(&VerifiedTransaction) -> Result<()>` (`candidate.rs:227-231`).
- `ExactIdentityValidator{checks, timeout, max_output_bytes}`: `new(member, expected)` / `for_members` / `args/timeout/max_output_bytes`; `validate` runs `run_bounded(CommandSpec::new(staged_path).args(...).current_dir(stage_root)...)` per check; requires `success && stderr.empty && stdout==expected` else `CandidateExecution` (`candidate.rs:233-332`).
- `CrossMemberAgreementValidator(ExactIdentityValidator)`: same-output-across-members helper (`candidate.rs:334-367`).
- `AllValidators<'a>{validators: Vec<&'a dyn CandidateValidator>}`: empty by default; `push`; sequential `validate` (`candidate.rs:369-402`).
- `VerifiedTransaction::validate`: rejects if any member missing result or `status != Verified` (“candidate execution requires verified integrity evidence for every member”), then `validator.validate` (`candidate.rs:410-424`). This is the `NotRequired`-cannot-commit gate (first of two; second is staged revalidation requiring a verified digest).
- `ValidatedTransaction{verified}`: `artifacts/staged_path` (staged mutability intentionally visible so tamper test proves revalidation catches it), `commit`, `commit_with_post_commit`, test faults (`candidate.rs:426-513`). `bounded_display` char-boundary truncates callback error to 512 bytes (`candidate.rs:516-532`).

### transaction.rs — locked commit, rollback, receipts, post-commit

- Enums: `TransactionDisposition::{Committed, RolledBack, RecoveryRequired}`; `CleanupDisposition::{Cleaned, RetainedForRecovery}` (temp evidence only); `PostCommitFailurePolicy::{KeepInstalled, RollBack}`; `FailurePhase::{Lock, Ownership, StageRevalidation, Backup, Commit, PostCommit, Rollback, Finalize}` with `as_str`; `FailureCategory::{LockContention, OwnershipConflict, Verification, Filesystem, InvalidInput, Injected, PostCommitCheck}` with `as_str` (`transaction.rs:14-123`).
- `FailureReport{phase, member, category, detail}`: `new` truncates `detail` at char boundary `<=512`; accessors (`transaction.rs:125-179`). `report_for_error(phase, member, error)` maps `UpdateInProgress=>LockContention`, `DestinationConflict=>OwnershipConflict`, `VerificationFailed/CandidateExecution=>Verification`, `InvalidInput` containing “injected”`=>Injected` else `InvalidInput`, `UnknownMember=>InvalidInput`, `Io=>Filesystem` (`transaction.rs:181-212`).
- `TransactionReceipt` fields + accessors; `recovery_path` always real retained evidence, never synthetic (`transaction.rs:214-292`).
- `CommitFault` (test-only): `LockCreation | Backup(MemberId) | BeforeFirstCommit | Commit(MemberId) | CommitThenRollback(MemberId, MemberId) | Finalize | PostCommitRollback` (`transaction.rs:294-304`). `BackupEntry{member, destination, backup: Option<PathBuf>}`.
- `PreparedTransaction::commit_inner(ownership, verified_digests, fault, post_commit)` (`transaction.rs:318-550`): ordered steps — (1) `LockCreation` fault => `Err(InvalidInput)` no mutation; (2) `classify_all` preflight; (3) `MutationLock::acquire`; (4) `revalidate_ownership_locked` fail => `finish_failure_no_mutation` (`RolledBack`, `performed=false`, `verified=true`); (5) `revalidate_staged_under_lock` fail => same; (6) `create_backup_directory` fail => same; (7) `backup_members` fail => `restore_entries(committed=∅)` + `finish_failure`; (8) `BeforeFirstCommit` fault => same; (9) per-member commit loop (`Commit` fault injection, `destination()`, `require_ready_parent`, `staged_path`, `fs::rename(staged, destination)`, record `committed`); any error => restore + `finish_failure`; (10) optional post-commit check (`catch_unwind`; `Ok(Ok)` continue; `Ok(Err)/Err` => `FailureReport PostCommit/PostCommitCheck`; `KeepInstalled` stores `post_commit_failure` and continues to finalize; `RollBack` maps `PostCommitRollback` fault to `CommitThenRollback(first,first)`, restores, returns receipt with both `failure` (post-commit cause) and `post_commit_failure` set); (11) `Finalize` fault => `preserve()` lock, `Committed/RetainedForRecovery/recovery_path=backup_root/failure=Finalize`; (12) `remove_dir_all(backup_root)` failure => same with real IO report; (13) success => `Committed/Cleaned/no recovery/no failures` (+ `post_commit_failure` if `KeepInstalled` had a failed check).
- `backup_members`: per member fault check; `destination()`; `revalidate_destination`; `symlink_metadata`: exists => `create_dir_all(backup_parent)` + `rename(destination, backup/member-path)`; `NotFound => None`; other IO => error (`transaction.rs:552-607`).
- Helpers: `classify_all` (verifier per destination), `revalidate_ownership_locked` (locked==preflight else `DestinationConflict`; `Owned` or `Absent+AllowCreate` else conflict; `require_ready_parent` + `revalidate_destination`), `revalidate_staged_under_lock` (verified digest must exist; `symlink_metadata.is_file`; unix `nlink==1`; re-hash equals), `create_backup_directory` (`.eggup-backup-{pid}-{nonce}-{nanos}` under root, 32 retries, `0700`), `revalidate_destination` (canonical root; walk up to existing ancestor, canonical must `starts_with` root; existing must be regular file, `nlink==1` unix), `require_ready_parent` (parent `symlink_metadata` exists, not symlink, is dir; canonical parent under canonical root; walk ancestors reject any symlink), `restore_entries` (reverse; skip injected `CommitThenRollback(_,id)` member; remove partial file if `backup.is_some || committed`; `rename(backup,destination)`; second pass sets `verified=false` if expected state missing), `finish_failure_no_mutation` vs `finish_failure` (latter cleans backup on full success else `preserve()` + `RecoveryRequired/RetainedForRecovery`) (`transaction.rs:609-971`).

### lock.rs — fail-closed mutation lock

- `LockStatus::{Available, Held{lock, contents}, Malformed{lock}}`; `MAX_LOCK_BYTES=4096`; `NEXT_LOCK_NONCE` (`lock.rs:11-32`).
- `MutationLock{path, token, cleanup, _file}`: `acquire(root, product, release)` writes `pid/nonce/product/release` token; rejects token `>4096`; `create_new(true)` so `AlreadyExists => UpdateInProgress{lock}`; write failure removes file; `0600` (unix) (`lock.rs:52-92`).
- `inspect(root)`: `Available` on `NotFound`; `Malformed` on non-file/symlink/oversized/unreadable/non-UTF8; `Held` with bounded contents otherwise; never deletes (`lock.rs:94-128`).
- `preserve()` (crate) disables cleanup; `Drop` removes file only if `is_file` and contents equal own token (`lock.rs:136-159`). Oversized/malformed records thus block `acquire` until manual operator removal with out-of-band process evidence.

### error.rs — error taxonomy

- `Error::{InvalidInput(String), UnknownMember(String), Io{operation:&'static str, source:io::Error}, UpdateInProgress{lock:PathBuf}, DestinationConflict{destination:PathBuf}, VerificationFailed(String), CandidateExecution(String)}` (`error.rs:9-35`); `Result<T>` alias; crate helpers `invalid/io`; `Display`; `source()` only for `Io`.

## 4. Key invariants / fail-closed rules

- Ownership is caller-proven: `Owned` only from `OwnershipVerifier`; `Foreign`/`Unknown` fail closed; `Absent` requires `AllowCreate`; replacement of `Owned` never needs the flag (`domain.rs:152-188`, `transaction.rs:624-654`). Flap between preflight and locked classification fails (`DestinationConflict`).
- Staged-digest revalidation: `commit` captures `verified_digests()` (Verified-only map); `revalidate_staged_under_lock` re-hashes every staged member under lock before any backup/mutation; tamper/unreadable/hard-link/missing-digest => `StageRevalidation` receipt, zero live mutation (tests: `staged_mutation_after_validation_fails_before_live_mutation`).
- No `NotRequired` commit: `validate` rejects any non-`Verified` member; revalidation requires a verified digest per member (`candidate.rs:412-420`, `transaction.rs:656-713`).
- No parent creation: `require_ready_parent` fails on missing parent without creating; symlinked/non-dir/escaped parents fail closed (`transaction.rs:788-845`; tests `missing_parent_fails_without_creating_directories`, `symlinked_parent_fails_closed`).
- Link/containment: sources must be regular files (symlinks rejected at plan); staged must be regular files with `nlink==1` (unix); destinations must be regular files with `nlink==1`, no symlink, canonical ancestor under canonical root (`domain.rs:499-534`, `transaction.rs:743-845`; tests `rejects_symlink_sources…`, `destination_links_fail_immediately_before_backup`).
- Duplicate/traversal rejection: duplicate member ids, duplicate normalized destinations, absolute/escaping/control-char destinations, empty sets, sources inside install root or aliasing destinations all rejected at `ArtifactSet::new` / `InstallPlan::new` / `normalize_relative_path`.
- `0700`/`0600`: stage dirs + backup dir `0700`; staged files `0600` (`0700` for executable intent); lock file `0600` (all unix-gated via `PermissionsExt`; tests assert `0700` stage, `0600` staged, `0600` lock).
- Bounded outputs: `DEFAULT_TIMEOUT 5s`, `DEFAULT_OUTPUT_LIMIT 64 KiB` per stream; `run_bounded` kills on timeout/overflow; `success` false on either; `FailureReport` and post-commit error display bounded to 512 (`candidate.rs:14-16,119-122`, `transaction.rs:141-158`).
- Fail-closed lock: `create_new`, `UpdateInProgress` on contention/malformed, `inspect` never deletes, no PID-liveness auto-recovery, oversized `>4096` is `Malformed` not deleted, `Drop` token-equality removal only.
- No authenticity: SHA-256 + sidecar filename binding are integrity evidence only; `ExactIdentityValidator` checks stdout identity of staged bytes, not publisher trust.

## 5. Data flow sequences

Success (multi-member):

```text
caller acquires bytes + digests
  -> InstallPlan::new (validate root/dests/sources)
  -> prepare() (sibling .eggup-stage-*, copy, 0600/0700)
  -> verify_integrity() (hash staged, compare declared Sha256)
  -> validate(AllValidators/ExactIdentity/…) (require all Verified, run_bounded staged programs)
  -> commit(CommitOwnership{verifier, AllowCreate|DenyCreate})
     preflight classify -> acquire .eggup-mutation.lock (0600, create_new)
     -> revalidate ownership under lock + require_ready_parent + revalidate_destination
     -> re-hash staged under lock vs verified_digests
     -> create .eggup-backup-* (0700) -> rename live->backup per member
     -> rename staged->live per member (track committed)
     -> [optional post-commit check with lock+backup held]
     -> remove_dir_all(backup) -> drop lock (token match) -> Receipt{Committed, Cleaned}
```

Rollback table (all `commit_inner` failure exits):

| Phase / trigger | Live mutation yet? | Restore action | Receipt |
|---|---|---|---|
| `Ownership` (locked mismatch, `Foreign`/`Unknown`, `Absent+DenyCreate`, bad parent/link/containment) | No | none; drop lock | `RolledBack`, `performed=false`, `verified=true`, `Cleaned`, `failure=Ownership/OwnershipConflict or Filesystem` |
| `StageRevalidation` (tampered/unreadable/hard-linked/missing digest) | No | none; drop lock | `RolledBack`, `performed=false`, `verified=true`, `Cleaned`, `failure=StageRevalidation/Verification` |
| `Backup` (incl. injected `Backup(id)`, backup-dir create fail) | No (backup loop partial state restored) | `restore_entries(entries, ∅)` | `RolledBack` if verified+cleanup ok else `RecoveryRequired` + `recovery_path=real backup` + `preserve()` lock |
| `Commit` (incl. `BeforeFirstCommit`, `Commit(id)`, rename/parent/staged errors) | Partial (`committed` set) | `restore_entries(entries, committed)`: remove partials, rename backups back, remove absent-members | Same split; `failure.member=Some(failing)`; `failure.phase=Commit` |
| `PostCommit RollBack` (check `Err`/panic + `RollBack`) | Full new generation live | `restore_entries(entries, committed)` | `RolledBack` + `failure=PostCommit/PostCommitCheck` + `post_commit_failure` set; on restore fault => `RecoveryRequired` + `rollback_failure=Rollback` + real path + lock preserved |
| `PostCommit KeepInstalled` (check fail + `KeepInstalled`) | Full new generation live | none (finalize backup) | `Committed` + `post_commit_failure=PostCommit` (+ `failure=Finalize` if finalize also fails) |
| `Rollback` (restore itself fails, incl. `CommitThenRollback`) | Partial unrestored | preserve lock + backup | `RecoveryRequired`, `performed=true`, `verified=false`, `RetainedForRecovery`, `failure=original`, `rollback_failure=Rollback`, `recovery_path=real backup` |
| `Finalize` (injected or `remove_dir_all` fails) | Committed | preserve lock + real backup | `Committed`, `RetainedForRecovery`, `failure=Finalize`, `recovery_path=real backup` (never synthetic) |
| `Lock` (`acquire` contention/malformed) | No | n/a | `Err(UpdateInProgress)` (not a receipt); `LockCreation` fault => `Err(InvalidInput)` |

## 6. Test / failure-injection strategy

- Harness (`test_support.rs`, `lib.rs` tests): `InstallationRoot::new()` temp dirs (`eggup-test-{nanos}-{seq}-{pid}`) with `safe_path` rejecting escapes (`PermissionDenied`); `write_file/read_file`; `Drop` removes root. `FailurePoint::{Prepare, StageCreate, StageCopy, Commit}` + `FailureInjector::check`; `Stage::prepare_with_failure` maps `StageCreate/StageCopy`; `CommitFault::{LockCreation, Backup, BeforeFirstCommit, Commit, CommitThenRollback, Finalize, PostCommitRollback}` threads through `commit_inner` (`candidate.rs:485-513`, `transaction.rs:294-304`).
- Coverage buckets (in `lib.rs` tests): fixture exact-bytes/escape rejection; prepare without touching destinations + stage-cleanup-on-failure (asserts no `.eggup-stage-*` remains); duplicate normalized destinations / empty sets / escaping destinations; symlink sources + executable-intent `0700`; full multi-member commit + lock release + backup cleanup; post-commit sees complete generation + holds lock (`acquire` inside check fails); `KeepInstalled` bounded failure (`é`*400 => `len<=512`); post-commit rollback restores old + removes absent-members; post-commit rollback fault => `RecoveryRequired` + both reports + real recovery path; finalize fault preserves both facts; panic => `RolledBack` + “post-commit check panicked”; partial commit fault restores every old member (`rollback_performed/verified`); absent-member removal on rollback; lock contention + malformed lock fail closed; rollback failure => `RecoveryRequired` + retained lock/evidence; precommit/backup faults restore; lock-creation fault performs zero mutation; link tests (symlink dest => `Ownership` rolled back; hard-linked live => rolled back); sidecar strict parsing (known `abc` hash `ba7816…15ad`, filename binding, lowercase/uppercase tolerance, malformed rejections); integrity-before-validation ordering + `NotRequired` cannot validate; bounded candidates exact + env-cleared; timeout kill + output-limit kill; cross-member agreement; ownership matrix (`Owned` allows, `Foreign`/`Unknown` zero-mutation rollback, `Absent` policy enforced, flap fails, missing parent creates nothing, symlinked parent fails, staged tamper => `StageRevalidation`, `None` integrity cannot commit, cleanup failure reports real path, `0700`/`0600` modes, `inspect` never deletes, oversized lock malformed).
- Examples (referenced `lib.rs:14-15`): one-member, multi-member, custom validator, ownership verifier, receipt interpretation.

## 7. Review checklist

- [ ] Plan validation: root absolute + dir + canonicalizable; destinations normalized relative, no traversal/absolute/controls/duplicates; sources absolute regular files (no symlinks), outside root, not aliasing destination.
- [ ] Staging: sibling stage, unique name, `0700` dirs, `0600`/`0700` files, no live mutation, `Drop` only removes owned prefix/real-dir/recorded-parent.
- [ ] Integrity: every member has `Sha256`; `verify_integrity` hashes staged bytes; sidecar (if used) single-entry strict with filename binding; no `None` reaches `validate`.
- [ ] Candidate validation: runs on staged paths with `run_bounded` (argv, `env_clear`, null stdin, cwd=`stage_root`, deadline, per-stream cap, kill/reap); exact-output + empty-stderr where identity matters; custom validators implement `CandidateValidator`.
- [ ] Commit: single production path `commit` (+ ADR-0002 `commit_with_post_commit`); preflight classify, `create_new` lock, locked ownership revalidation (equality + `Owned`/`Absent+AllowCreate`), staged re-hash under lock, backup-then-rename unit = full artifact set.
- [ ] Parents/links/containment: `require_ready_parent` + `revalidate_destination` under lock and per-member at backup/commit; no `create_dir_all` on live paths; symlinks/hard-links/non-regular/escaped fail closed.
- [ ] Receipts: `RolledBack` never treated as success; `failure` always present on non-clean commit; `rollback_failure` preserves original cause; `post_commit_failure` distinct; `recovery_path` real only; `cleanup` describes temp evidence, not post-commit policy.
- [ ] Post-commit: one caller check, lock+backup held, `catch_unwind` => failed check, bounded error copy, explicit `KeepInstalled|RollBack`, no core timeout, no crash/power-loss claim.
- [ ] Locking: `0600`, `4096`-bounded, `create_new`, `UpdateInProgress` on held/malformed, `inspect` read-only, manual stale removal with out-of-band evidence, `Drop` token-match only.
- [ ] Bounds/permissions: timeouts/output caps set on every `CommandSpec`; receipt details `<=512`; stage/backup/lock modes asserted on unix.
- [ ] No overclaims: docs/code never promise transport, service management, release ordering, signatures/authenticity, or journaled atomicity.
