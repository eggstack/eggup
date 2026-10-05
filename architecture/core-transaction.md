# `eggup-core` — Verified Update Transaction Deep Dive

Status: current as of `eggup-core` 0.1.2 (`M009` closure, published to crates.io).

This is a review reference. Every behavioral claim below is grounded in source, and
`file:line` references are relative to the workspace root. Where code and crate
documentation disagree, the code is treated as the behavior and the discrepancy is
listed in [Known doc/code drift](#known-doccode-drift).

## 1. Purpose and ownership boundary

`eggup-core` implements the policy-neutral local mechanics of a verified multi-artifact
update: it turns a caller-supplied set of already-resolved local artifact files into a
coherent installed deployment, and it either commits that whole generation or leaves the
live state as it found it. It owns the domain types, the private stage, local integrity
primitives, candidate execution, destination ownership preflight, the mutation lock,
backup/commit/rollback, and the terminal receipt.

It deliberately does **not**:

| Not owned | Where the boundary sits instead |
| --- | --- |
| Network transport, HTTP/TLS, download | [acquisition](acquisition.md) seam, `eggup-eggfetch`, `eggup-curl` |
| Release discovery, version ordering, channel policy | caller (see [Eggpack adapter](eggpack-adapter.md) for the one release-manifest shape) |
| Authenticity, signatures, publisher identity | caller. Core has no authenticity verifier and makes no such claim; see [integrity is checksum-only](#5-core-concepts) |
| Service-manager lifecycle | [eggup-service](service-lifecycle.md) |
| Archive extraction | [eggup-archive](archive-extraction.md) |
| CLI, bootstrap installer, migrations, uninstall orchestration | consumer crates |
| Whether a given release may be installed at all | caller. Core answers "are these local bytes the bytes you asked for, and is the destination mine to replace" |

Dependency rule: the crate's only dependency is `sha2` (`crates/eggup-core/Cargo.toml`).
`#![forbid(unsafe_code)]` and `#![deny(missing_docs)]` are declared at `lib.rs:1-2`.

Two further self-imposed constraints are worth stating because they are easy to violate
in review:

- Core never creates a live destination parent. `require_ready_parent`
  (`transaction.rs:791`) only *checks* that the parent exists, is a real directory, and
  is not a symlink. Directory creation happens exclusively inside the stage
  (`stage.rs:72-82`) and the backup root (`transaction.rs:718`).
- Core never makes a timeout, retry, or crash-durability promise of its own. The only
  timeout is the caller-supplied one in `CommandSpec::timeout`
  (`candidate.rs:64`, default 5s at `candidate.rs:14`).

## 2. Position in the workspace

```text
eggup-acquisition ──▶ eggup-eggfetch        (transport adapters, no core dep)
        │
        └──────────▶ eggup-curl

eggup-core (sha2 only)
   ├──▶ eggup-service      (manager-neutral lifecycle, unpublished)
   ├──▶ eggup-archive      (dev-dependency only, for qualification tests)
   └──▶ eggup-eggpack      (published leaf adapter; producer types live there)
```

Core is at the bottom of the graph. It imports no workspace crate, so no cycle is
possible and no adapter can leak policy into it. Consumers that name core in their
manifest: `eggup-service`, `eggup-archive` (dev), `eggup-eggpack`.

See [overview.md](overview.md) for the module map, the cross-cutting invariants, and the
terminology contract this document follows.

## 3. Public surface

Everything below is re-exported from `crates/eggup-core/src/lib.rs:25-44`.

### Domain (`domain.rs`)

| Export | Role | Ref |
| --- | --- | --- |
| `ProductId` | Validated product identity, `1..=64` chars, no `/` `\` or control chars | `domain.rs:11`, `domain.rs:15` |
| `ReleaseId` | Validated release identity, same character rules | `domain.rs:33`, `domain.rs:37` |
| `MemberId` | Validated per-artifact identity inside a set | `domain.rs:55`, `domain.rs:59` |
| `FileKind` | Source file classification; only `Regular` today, `#[non_exhaustive]` | `domain.rs:87` |
| `PermissionsIntent` | `Private` / `Executable` / `Preserve` staged-file mode intent | `domain.rs:99` |
| `Ownership` | `Absent` / `Owned` / `Foreign` / `Unknown` | `domain.rs:112` |
| `IntegrityRequirement` | `Sha256(Digest)` (only variant today), `NotRequired` marker used by absence of a member requirement | `domain.rs:130` |
| `OwnershipVerifier` | Caller-implemented ownership proof trait | `domain.rs:147` |
| `AbsentPolicy` | `AllowCreate` / `DenyCreate` for `Ownership::Absent` destinations | `domain.rs:158` |
| `CommitOwnership<'a>` | Verifier + absent policy pair passed to `commit` | `domain.rs:168`, `domain.rs:177` |
| `ArtifactMember` | One `(id, source, relative destination)` triple plus mode/digest intent | `domain.rs:281` |
| `ArtifactSet` | Ordered, duplicate-free member collection; the mutation unit | `domain.rs:357` |
| `BoundSources` | Pre-opened `MemberId -> File` map for the object-bound prepare path | `domain.rs:418` |
| `InstallPlan` | Product + release + root + set; entry point | `domain.rs:447` |
| `Result<T>` | `core::Result<T> = Result<T, Error>` | `error.rs:5` |
| `Error` | The only non-receipt failure channel | `error.rs:9` |
| `AbsentOnlyVerifier` | Verifier that only ever returns `Absent`; cannot authorize replacement | `domain.rs:194` |
| `ExistingAsOwnedVerifier` | Verifier that returns `Owned` for any existing path (for fixtures) | `domain.rs:213` |
| `ExactDigestVerifier` | Verifier that proves `Owned` by exact prior content digest | `domain.rs:232` |

### Integrity (`integrity.rs`)

| Export | Role | Ref |
| --- | --- | --- |
| `hash_file` | Streaming SHA-256 of a local file | `integrity.rs:65` |
| `parse_sha256_sidecar` | Strict `<64 hex> [ filename]` sidecar parser | `integrity.rs:83` |
| `verify_file` | Digest + `FileKind` check over local bytes | `integrity.rs:117` |
| `Sha256Manifest` | Digest + optional declared filename | `integrity.rs:13` |
| `IntegrityStatus` | `Verified` / `Failed` | `integrity.rs:32` |
| `IntegrityResult` | Status + observed digest | `integrity.rs:41` |
| `VerifiedTransaction` | Prepared state with member integrity resolved | `integrity.rs:164` |

### Staging (`stage.rs`)

| Export | Role | Ref |
| --- | --- | --- |
| `PreparedTransaction` | Stage-backed plan; exposes `staged_path` for external inspection | `stage.rs:147` |

### Candidate execution (`candidate.rs`)

| Export | Role | Ref |
| --- | --- | --- |
| `CommandSpec` | Bounded external-command request (program, args, cwd, timeout, output cap, explicit env only) | `candidate.rs:19` |
| `CommandOutput` | Exit code, captured stdout/stderr, `timed_out`, `output_limited`, `success` | `candidate.rs:84` |
| `run_bounded` | Spawn/wait/poll with kill-on-timeout and output truncation | `candidate.rs:125` |
| `CandidateValidator` | Trait: `validate(&VerifiedTransaction) -> Result<()>` | `candidate.rs:228` |
| `ExactIdentityValidator` | Runs a member's staged file and requires exact stdout | `candidate.rs:235` |
| `CrossMemberAgreementValidator` | Requires every member to print the same identity | `candidate.rs:336` |
| `AllValidators<'a>` | Fan-out over validators; empty by default | `candidate.rs:370` |
| `ValidatedTransaction` | Terminal pre-commit state; exposes `commit` / `commit_with_post_commit` | `candidate.rs:406` |

### Locking (`lock.rs`)

| Export | Role | Ref |
| --- | --- | --- |
| `MutationLock` | Create-new lock record owning one installation domain | `lock.rs:44`, `lock.rs:53` |
| `LockStatus` | `Available` / `Held` / `Malformed`, from the read-only inspector | `lock.rs:15` |

### Transaction and receipts (`transaction.rs`)

| Export | Role | Ref |
| --- | --- | --- |
| `TransactionDisposition` | `Committed` / `RolledBack` / `RecoveryRequired` | `transaction.rs:19` |
| `CleanupDisposition` | `Cleaned` / `RetainedForRecovery` | `transaction.rs:34` |
| `PostCommitFailurePolicy` | `KeepInstalled` / `RollBack` | `transaction.rs:43` |
| `FailurePhase` | `Ownership`, `StageRevalidation`, `Backup`, `Commit`, `Rollback`, `Finalize`, `PostCommit` | `transaction.rs:55` |
| `FailureCategory` | `Ownership`, `Lock`, `Destination`, `Stage`, `Backup`, `Commit`, `Rollback`, `Finalize`, `PostCommitCheck` | `transaction.rs:93` |
| `FailureReport` | Phase + category + optional member + bounded (≤512 B) core-authored detail | `transaction.rs:127` |
| `TransactionReceipt` | The terminal artifact: disposition, rollback facts, cleanup, recovery path, up to three reports | `transaction.rs:210` |

`TransactionReceipt` accessors are all read-only: `disposition` (`transaction.rs:235`),
`rollback_performed` (`transaction.rs:240`), `rollback_verified` (`transaction.rs:252`),
`cleanup` (`transaction.rs:257`), `recovery_path` (`transaction.rs:268`), `failure`
(`transaction.rs:279`), `rollback_failure` (`transaction.rs:287`), `post_commit_failure`
(`transaction.rs:292`).

Note on `Error`: it is a public enum **without** `#[non_exhaustive]`
(`error.rs:8-9`). Downstream matches are therefore exhaustive and adding a variant — as
`Injected` (`error.rs:40`) recently was — is a breaking change. `Error::injected` itself
is `pub(crate)` (`error.rs:50-52`), so downstream code cannot construct that variant but
must still handle it.

## 4. The state machine

```text
                          prepare()                      verify_integrity()
  InstallPlan  ──────────────────────────▶ PreparedTransaction ─────────────────────▶ VerifiedTransaction
  domain.rs:447                              stage.rs:147                           integrity.rs:164
                                                │                                       │
                                                │                              validate(&dyn CandidateValidator)
                                                ▼                                       ▼
                                        (stage dir, 0700)                      ValidatedTransaction
                                        sibling of root                        candidate.rs:406
                                                                                    │
                          ┌─────────────────────────────────────────────────────────┤
                          │ commit(CommitOwnership)                                 │ commit_with_post_commit(
                          │  (candidate.rs:449)                                     │   ownership, policy, FnOnce)
                          └────────────────────────────┬────────────────────────────┘
                                                       ▼
                                                TransactionReceipt
                                             Committed | RolledBack | RecoveryRequired

  internal commit phases (transaction.rs:322 commit_inner), all under MutationLock:
    preflight classify (ownership, first call)  ──▶ under-lock revalidate
      (ownership 2nd call + staged digests)     ──▶ backup (per member)
      ──▶ commit loop (per member rename)       ──▶ [optional post-commit check]
      ──▶ finish: finalize or rollback          ──▶ receipt
```

### `InstallPlan` — `domain.rs:447`

- **Proves**: nothing about the files. It is a validated declaration: identifier
  character rules, relative destination normalization, duplicate normalized-destination
  rejection, source `FileKind::Regular` and not-a-symlink, destination containment
  inside the root (`domain.rs:456`, `domain.rs:545`, `domain.rs:557`,
  `domain.rs:594`).
- **Durable**: nothing. No filesystem mutation has occurred.
- **Can still fail**: everything downstream.
- **Advances via**: `prepare()` (`domain.rs:515`) or `prepare_with_bound_sources()`
  (`domain.rs:532`).

### `PreparedTransaction` — `stage.rs:147`

- **Proves**: that every member was copied from a proven source into an
  owner-private stage owned by this process. `DestinationConflict` from a foreign
  file in the stage fails closed (`stage.rs:196-208` on `Drop`, and the create-new
  directory at `stage.rs:232`). No destination has been read, written, or even stat'ed
  for existence beyond what the plan already validated.
- **Durable**: a private stage directory in the **parent** of the installation root
  (`stage.rs:214-231`), named `.eggup-stage-{root}-{pid}-{seq}-{nanos}`, one
  subdirectory per member destination, each file narrowed to `0600`/`0700` on Unix
  (`stage.rs:294-305`).
- **Can still fail**: copy, digest mismatch, candidate execution, ownership,
  contention, and every commit-phase failure.
- **Advances via**: `PreparedTransaction::verify_integrity` (`integrity.rs:171`),
  which consumes `self`; or `commit_inner` (`transaction.rs:322`) only as an internal
  fast path exercised through the test-only fault seams.

### `VerifiedTransaction` — `integrity.rs:164`

- **Proves**: each member's staged bytes hash to its `IntegrityRequirement::Sha256`
  value and the staged object is still a regular file. Records the observed digests in
  `verified_digests` (`integrity.rs:241`).
- **Durable**: same as `PreparedTransaction`, plus the digest evidence.
- **Can still fail**: the *same* member can fail again later — `revalidate_staged_under_lock`
  (`transaction.rs:659`) re-reads and re-hashes each staged file under the lock, so
  staged-byte mutation between verify and commit is caught before any live mutation.
- **Advances via**: `ValidatedTransaction::validate` (`candidate.rs:412`).

### `ValidatedTransaction` — `candidate.rs:406`

- **Proves**: (a) at least one validator was supplied and ran, unless every member is
  `NotRequired` (`candidate.rs:413-420`); (b) every validator returned `Ok`. Because
  the crate ships no default validators, "validate succeeded" is a claim about the
  caller's validators plus the `NotRequired` gate, nothing more.
- **Durable**: same stage, same digests.
- **Can still fail**: any ownership, revalidation, backup, commit, rollback, or
  finalization failure. The staged bytes can still change after `validate` returns;
  that is exactly what the under-lock revalidation exists to catch.
- **Advances via**: `commit` (`candidate.rs:449`) or `commit_with_post_commit`
  (`candidate.rs:465`).

### `TransactionReceipt` — `transaction.rs:210`

- **Proves**: the terminal disposition plus what happened to the live generation,
  the backup, the lock, and the failure. There is no further mutation; receipts are
  inert data.
- **Durable**: whatever the filesystem now holds, described by the receipt.
- **Can still fail**: nothing. A receipt is never an `Err`.
- **Advances via**: nothing — it is terminal. `RecoveryRequired` is an operator handoff,
  not a retryable state, and core offers no resume path.

## 5. Core concepts

### `ArtifactSet` and `Member`

An `ArtifactSet` (`domain.rs:357`) is an ordered, duplicate-keyed collection of
`ArtifactMember` (`domain.rs:281`). Each member carries an id, an absolute or relative
**source** path, and a **relative destination**. `InstallPlan::new` normalizes every
destination through `normalize_relative_path` (`domain.rs:594`) and rejects duplicates
*after* normalization, so `bin/app` and `./bin/app` cannot both be members.

The set is the mutation unit. Multi-member commit is all-or-nothing: `backup_members`
(`transaction.rs:555`) records one `BackupEntry` per member before any rename, and the
commit loop (`transaction.rs:407-444`) only begins after every entry exists. There is
no partial-success state, and `multi_member.rs` exists to demonstrate the generation
property rather than a per-file outcome.

### Digest and size evidence

`IntegrityRequirement` has one variant today, `Sha256(Digest)` (`domain.rs:130`).
`verify_file` (`integrity.rs:117`) hashes local bytes and checks `FileKind::Regular`; it
returns `IntegrityResult` with status and observed digest. `parse_sha256_sidecar`
(`integrity.rs:83`) is a strict two-token parser, not a general `sha256sum` dialect
reader.

Integrity here is **checksum evidence only**. A matching SHA-256 proves the staged bytes
equal the bytes the caller described. It does not prove who produced them, that they
came from a particular release channel, or that the digest itself was not substituted
upstream. Core ships no authenticity verifier and takes no position on one; the caller
owns release resolution and any signature or provenance check. The test
`checksum_only_state_is_explicit_in_docs` (`lib.rs:1658`) pins this in place so it
cannot be quietly upgraded.

There is no size requirement in the domain today, so no size evidence is recorded.
`CommandOutput::output_limited` (`candidate.rs:114`) is an execution bound, not an
artifact-integrity claim.

### `OwnershipVerifier`

`OwnershipVerifier` (`domain.rs:147`) is a single-method trait,
`verify(&MemberId, &Path) -> Ownership`. Core never infers `Owned`: it is whatever the
caller returns. The shipped verifiers are deliberately unequal in power —
`ExistingAsOwnedVerifier` (`domain.rs:213`) is a fixture-grade trust-everything
verifier, `ExactDigestVerifier` (`domain.rs:232`) proves ownership by prior content, and
`AbsentOnlyVerifier` (`domain.rs:194`) can never authorize replacement.

The trait doc (`domain.rs:139-142`) requires a deterministic answer for fixed filesystem
state. That is load-bearing: `classify_all` (`transaction.rs:612`) and
`revalidate_ownership_locked` (`transaction.rs:627`) each call the verifier, and any
change in the answer between the two calls fails the transaction closed.

### Candidate execution and bounded validators

`run_bounded` (`candidate.rs:125`) is the only place core spawns a process. It clears the
environment (`candidate.rs:129`), passes only explicitly added variables, applies a
caller timeout, kills the child on expiry, and caps captured output. The bounds are
observables on the result, not silent truncations: `timed_out` and `output_limited`
(`candidate.rs:109-120`).

`ExactIdentityValidator` (`candidate.rs:235`) requires exact stdout equality for one
member; `CrossMemberAgreementValidator` (`candidate.rs:336`) requires every member in a
bundle to print the same identity. Timeout and output-overflow are converted into
`Error::CandidateExecution` at `candidate.rs:320-327`, so a validator cannot report
success by being silent. `AllValidators::new()` is empty by default
(`candidate.rs:376`, `Default` at `candidate.rs:390`), which means `AllValidators` is a
convenience fan-out, not a policy.

`CandidateValidator` receives `&VerifiedTransaction`, so a custom validator sees staged
paths and digests but has no mutation authority — the `custom_validator.rs` example
shows the intended shape.

### `MutationLock` and `LockStatus`

`MutationLock::acquire` (`lock.rs:53`) creates `<installation_root>/.eggup-mutation.lock`
with `create_new` (`lock.rs:66-69`). That single syscall is the entire mutual-exclusion
mechanism: no advisory flock, no PID liveness probe, no heartbeat, no expiry. The record
is a bounded text token `pid=… nonce=… product=… release=…` capped at 4096 bytes
(`lock.rs:11`, `lock.rs:63`).

`MutationLock::inspect` (`lock.rs:100`) is read-only and returns `LockStatus::Available`,
`Held`, or `Malformed`. It never removes anything, and neither does anything else in the
crate: a stale lock blocks forever until an operator removes it, by design.

`Drop` (`lock.rs:141`) removes the record only if it is still a regular file whose bytes
equal this process's token, so a lock that was replaced or rewritten by someone else is
left alone.

### Receipts and the receipt status variants

A receipt combines three orthogonal status axes plus up to three reports:

| Axis | Variants | Meaning |
| --- | --- | --- |
| `TransactionDisposition` | `Committed` | The new generation is in place. With `KeepInstalled` after a failed post-commit check, the *check* failed but the disposition is still `Committed` — read `post_commit_failure` to tell. |
| | `RolledBack` | Live state is coherent and is not the new generation. Never equivalent to success. |
| | `RecoveryRequired` | Core cannot prove either generation is coherent. Operator action required. |
| `CleanupDisposition` | `Cleaned` | Stage and backup directories are gone. |
| | `RetainedForRecovery` | Backup and/or lock preserved as evidence; `recovery_path` points at the real backup root. |
| rollback facts | `rollback_performed` | Whether live destinations were rewritten from backup. |
| | `rollback_verified` | Whether every member was re-checked after restoration, including a second containment pass. |

`recovery_path` (`transaction.rs:268`) is always the actual backup root returned by
`create_backup_directory` (`transaction.rs:718`), never a constructed string; the test
`cleanup_failure_reports_real_backup_root` (`lib.rs:1510`) pins this.

## 6. Control flow, step by step

### prepare — `stage.rs:20` / `stage.rs:24`

1. `create_stage_directory` (`stage.rs:212`) requires a parent of the installation root
   and creates `.eggup-stage-{root}-{pid}-{seq}-{nanos}` there with `create_dir`
   (`stage.rs:232`), then chmods `0700` on Unix. Up to 32 name collisions are retried.
2. For each member, `copy_members` (`stage.rs:63`) creates the member's subdirectory
   under the stage and chmods it `0700`. This is the only place directories are created.
3. The source is either an already-open `BoundSources` handle (`stage_bound_source`,
   `stage.rs:120`) or a pathname copy (`stage.rs:96`).
4. Executable intent is read **before** the staged file exists — from the handle's own
   `fstat` for bound sources (`stage.rs:253`), or from the just-copied file for
   pathname sources (`stage.rs:269`). `apply_permissions` (`stage.rs:281`) then narrows
   the staged file to `0600`, or `0700` for `Executable`/`Preserve`-executable.
5. On any failure the stage is dropped and removed (`stage.rs:52-55`).
6. A leftover `BoundSources` entry for a non-member fails closed
   (`stage.rs:56-59`).

Nothing here reads a live destination. The `NotRequired`/ownership questions are all
deferred to commit.

### verify — `integrity.rs:171`

For each member, `verify_file` re-hashes the **staged** file and compares against
`IntegrityRequirement`. Failures return `Err(Error::VerificationFailed)` — not a receipt,
because no live mutation and no backup exist yet. The observed digests are retained in
`VerifiedTransaction::verified_digests` for later comparison.

### validate — `candidate.rs:412`

1. If not all members are `NotRequired` and the validator set is empty, return
   `Err(VerificationFailed)`. This is the gate that stops a caller from getting an
   unvalidated commit by passing an empty `AllValidators`.
2. Otherwise run the validator. Any error propagates unchanged, so validators keep full
   control of their error variants.

### commit — `candidate.rs:449` → `transaction.rs:322`

`commit_inner` runs strictly in this order:

1. **Lock** — `MutationLock::acquire` (`transaction.rs:335`). Contention and lock setup
   failure return `Err` here, before any mutation.
2. **Preflight classification** — `classify_all` (`transaction.rs:612`) calls the
   verifier once per member and checks parent readiness (`require_ready_parent`,
   `transaction.rs:791`: parent must exist, be a real directory, and not be a symlink).
   Failure here produces a `RolledBack` receipt with `rollback_performed == false`.
3. **Under-lock revalidation** — `revalidate_ownership_locked` (`transaction.rs:627`)
   calls the verifier a **second** time and compares with the preflight answer, then
   runs `revalidate_destination` (`transaction.rs:746`) per member, which fails closed
   on a non-regular or hard-linked (`nlink > 1`) destination. Any difference, or a
   `Foreign`/`Unknown` answer, or `Absent` under `DenyCreate`, yields a `RolledBack`
   receipt with no mutation performed.
4. **Staged revalidation** — `revalidate_staged_under_lock` (`transaction.rs:659`)
   re-resolves each staged path, requires a regular non-hard-linked file, requires a
   recorded verified digest, and **re-hashes the bytes**, comparing against that
   recorded digest. This is what catches a staged file replaced between `validate`
   and commit.
5. **Backup** — `create_backup_directory` (`transaction.rs:718`) creates
   `.eggup-backup-{root}-{pid}-{nonce}-{nanos}` **inside the installation root** and
   chmods the backup root `0700` on Unix (`transaction.rs:735`), retrying up to 32 name
   collisions. `backup_members` (`transaction.rs:555`) then, per member, re-runs
   `revalidate_destination` (`transaction.rs:573`) and — if a live file exists —
   **`fs::rename`s it into the backup set** (`transaction.rs:586`). This is a move, not
   a copy: the live inode, its ownership, and its mode travel into the backup, and the
   live path becomes briefly absent. Intermediate backup subdirectories are created with
   `fs::create_dir_all` and no explicit mode (`transaction.rs:579`); they inherit the
   umask but sit inside the `0700` backup root. Only after every member has a
   `BackupEntry` does the commit loop proceed.
6. **Commit loop** (`transaction.rs:407-444`) — per member: resolve the destination,
   `require_ready_parent` again, resolve the staged path, then a single
   `fs::rename(staged, destination)`. The first error triggers `finish_failure`.
7. **Finish** — success: `remove_dir_all` of stage and backup, then `Cleaned`. Failure:
   `finish_failure` (`transaction.rs:939`) rolls back.

A reviewer should note what "generation-consistent" means here. Because the backup is a
`rename` and the commit is a `rename`, a destination that has already been committed is
live while a *later* member's destination is momentarily empty — moved to backup but not
yet replaced. The all-or-nothing property holds at transaction boundaries (on
`Committed`, or on a fully verified `RolledBack`), not at every instant during the
commit loop.

Note the asymmetry in step 5 versus step 6: `revalidate_destination` runs per member
during backup, but the commit loop re-checks only parent readiness
(`transaction.rs:411`), not destination type or link count. See
[Concurrency](#9-concurrency) for why that still fails safe.

### rollback — `transaction.rs:850` / `transaction.rs:939`

`finish_failure` restores in two passes. Because both backup and restore are `rename`s,
the original inode — and therefore its mode and ownership — comes back unchanged; no
separate mode restoration is needed or performed.

1. Members are walked in reverse order (`transaction.rs:857`). If the destination exists
   and is not a regular file, it is **not** removed — `verified` is set to false, a
   `Rollback`/`Filesystem` report is recorded, and the pass continues, leaving the backup
   in place (`transaction.rs:873-886`). Otherwise an existing partial new file is
   removed (`transaction.rs:874`).
2. Each backup is then `fs::rename`d back onto the destination
   (`transaction.rs:888-899`); a failure marks the transaction unverified and records the
   first `Rollback`/`Filesystem` cause.
3. A second pass (`transaction.rs:902-913`) re-checks every member: a member with a
   backup must have *something* at its destination, and a member with no backup that was
   committed must have *nothing* there.

Then `rollback_verified` is recomputed:

- If `verified` and the backup directory was removed, the receipt is `RolledBack` with
  `rollback_verified == true` and `cleanup == Cleaned`.
- If `verified` but backup removal failed, the receipt is `RolledBack` with
  `rollback_verified == true` and `cleanup == RetainedForRecovery` (`transaction.rs:969-983`).
- If not `verified`, the receipt is `RecoveryRequired` with `rollback_performed == true`,
  a `rollback_failure` report when one was produced, the **real** `recovery_path`, and
  the lock **preserved** by `MutationLock::preserve` (`lock.rs:136`) so evidence cannot
  be swept by a later run.

Pre-mutation failures skip all of this via `finish_failure_no_mutation`
(`transaction.rs:917`), which reports `rollback_performed == false,
rollback_verified == true`.

### `commit_with_post_commit` — `candidate.rs:465`

1. Steps 1-6 run exactly as above, but on success the lock and backup are **retained**
   rather than finalized.
2. The caller's check runs **once**, with the mutation lock still held and the backup
   set still on disk, so a concurrent Eggup commit is still excluded
   (`post_commit_check_sees_complete_generation_and_holds_mutation_lock`,
   `lib.rs:655`). The callback receives no transaction handle and therefore no mutation
   authority.
3. The check's error text is reduced to bounded core-owned text of at most 512 bytes
   (`bounded_display`, `candidate.rs:516`); a panic is caught and recorded as a
   `PostCommitCheck` failure (`candidate.rs:448-453`).
4. `PostCommitFailurePolicy::KeepInstalled` finalizes and returns `Committed` with
   `post_commit_failure` set. A finalize failure upgrades cleanup to
   `RetainedForRecovery` and sets `failure` to a `Finalize` report, keeping the
   post-commit report too (`lib.rs:807`).
5. `PostCommitFailurePolicy::RollBack` runs the same `finish_failure` path. The returned
   receipt carries the post-commit report in `post_commit_failure` **and** in `failure`
   when the rollback itself succeeded (`transaction.rs:496`); a failed rollback
   yields `RecoveryRequired` retaining both reports and the backup evidence
   (`lib.rs:774`).

## 7. Invariants and enforcement

| # | Invariant | Enforced at | On violation |
| --- | --- | --- | --- |
| 1 | Unverified bytes never execute | `candidate.rs:413-420` (non-`NotRequired` set requires a non-empty validator set) | `Err(VerificationFailed)`; no candidate is spawned |
| 2 | No live mutation before every member is backed up | `transaction.rs:555-609` must complete for all members before `transaction.rs:407` starts | `Err`/`Receipt` from backup; rollback restores untouched state |
| 3 | `Owned` is never inferred; ownership is caller-proven and checked twice | `classify_all` (`transaction.rs:612`) and `revalidate_ownership_locked` (`transaction.rs:627`) | Flap, `Foreign`, `Unknown`, or `Absent`+`DenyCreate` → `RolledBack` receipt, zero mutation |
| 4 | Staged bytes are re-verified under the lock | `revalidate_staged_under_lock` (`transaction.rs:659`) re-hashes against the digest recorded at verify time | `RolledBack` receipt, `StageRevalidation`, zero live mutation |
| 5 | Destinations are never symlinks or hard links, and live parents are never created | `revalidate_destination` (`transaction.rs:746-789`, canonicalizes the root and the nearest existing ancestor), `require_ready_parent` (`transaction.rs:791-830`) | `RolledBack` receipt, `Destination`; **no** directory is created |
| 6 | Destinations stay inside the installation root | `normalize_relative_path` + `InstallPlan::new` (`domain.rs:545`, `domain.rs:594`); re-checked per member during backup and per member in `restore_entries` | `Err(InvalidInput)` at plan time; `rollback_verified == false` → `RecoveryRequired` |
| 7 | Transaction-owned state is owner-private on Unix | `0700` on the stage dir (`stage.rs:237`), stage subdirs (`stage.rs:80`), and the backup root (`transaction.rs:735`); `0600` on staged files (`stage.rs:300`) and the lock record (`lock.rs:84`) | Permissions are best-effort `set_permissions` calls whose errors are discarded with `let _ =`; a failure is not surfaced |
| 7a | The backup set is not widened | The backup is a `rename` of the live file (`transaction.rs:586`), so a backed-up file keeps its **original** mode inside the `0700` backup root. Core does not re-narrow it, and the intermediate backup subdirectories are not chmod'd (`transaction.rs:579`) | No error path; the `0700` root is the only containment |
| 8 | Core never removes a lock it does not own, and never inspects destructively | `lock.rs:141` (`Drop` compares token bytes), `lock.rs:100` (`inspect` is read-only) | A foreign lock is left in place; a stale lock blocks all later commits with `Err(UpdateInProgress)` |
| 9 | Rollback is verified, not assumed | Two-pass `restore_entries` + second containment pass (`transaction.rs:902-913`) | `RecoveryRequired` receipt, real `recovery_path`, lock preserved |
| 10 | Crashes are never claimed to be recovered | Nothing — no journal, no replay | Process death leaves stage, backup, and possibly the lock on disk; recovery is operator work via the receipt and the backup |

### Platform-conditional behavior

Invariants 7 and part of 5 are **Unix-only in implementation**:

- Every permission operation is `#[cfg(unix)]`. On non-Unix, `apply_permissions`
  (`stage.rs:307-308`) is a no-op, `bound_handle_is_executable` returns `false`
  (`stage.rs:262`), and `staged_is_executable` returns `false` (`stage.rs:277`).
  Consequence: on Windows, `PermissionsIntent` has no observable effect, `Preserve`
  cannot recover source executable intent, and staged files, the stage directory, the
  backup directory, and the lock record all inherit the parent ACL. Core makes no
  Windows hardening claim, and `crates/eggup-core/docs/domain.md:11` states the
  permission behavior without this qualification.
- The `0600`/`0700` mode is also applied **after** the object is created — the stage
  directory at `stage.rs:232-238`, the backup root at `transaction.rs:730-736`, the
  lock record after the token is written at `lock.rs:77-85`. There is therefore a brief
  window in which the object carries umask-derived permissions rather than the private
  mode. For the stage this window opens before any staged bytes are copied, which limits
  exposure to an empty directory; for the lock it exposes the token line, which is
  diagnostic data, not a secret.
- Symlink-dependent enforcement is also Unix-shaped. `symlink_metadata` distinguishes
  symlinks from regular files on both platforms, but the 9 `#[cfg(unix)]` tests
  (see [Testing approach](#10-testing-approach)) mean the link and permission
  invariants are only exercised on Unix.

## 8. Failure semantics

This is the central review artifact. Two rules govern the whole table:

1. **`Err` is for setup, not for outcome.** Setup failures — plan validation, stage
   creation/copy, integrity, candidate execution, lock acquisition, lock-record
   corruption — return `Err` because nothing live has changed.
2. **Anything after the lock is taken is reported by receipt.** Once
   `MutationLock::acquire` succeeds, ownership/staging/backup/commit/rollback/finalize
   problems are all reported as `Ok(receipt)` with `Committed`, `RolledBack`, or
   `RecoveryRequired`. The lock is dropped on the way out unless the receipt requires
   `RecoveryRequired` or `RetainedForRecovery`.

| Condition | Outcome | Caller obligation | Ref |
| --- | --- | --- | --- |
| Invalid identifier, escaping or duplicate normalized destination, bad installation root, non-regular/symlinked source | `Err(InvalidInput)` | Fix the declaration; nothing was touched | `domain.rs:456`, `domain.rs:545`, `domain.rs:557` |
| Stage directory cannot be created or 32 names collide | `Err(Io)` / `Err(InvalidInput)` | Ensure the **parent** of the install root is writable | `stage.rs:242`, `stage.rs:245` |
| Stage member copy fails, or injected `StageCreate`/`StageCopy` | `Err(Io)` / `Err(Injected)` | Stage is removed; safe to retry | `stage.rs:52-55`, `stage.rs:318` |
| `BoundSources` entry for a non-member | `Err(InvalidInput)` | Producer/consumer key mismatch; fix the map | `stage.rs:56-59` |
| Staged digest mismatch or non-regular staged file at verify | `Err(VerificationFailed)` | Re-resolve artifacts; no receipt exists yet | `integrity.rs:179`, `integrity.rs:191` |
| `NotRequired` set validated with an empty validator set | `Err(VerificationFailed)` | Supply a validator; the gate will not be bypassed | `candidate.rs:413-420` |
| Validator returns an error, or `run_bounded` times out / truncates output | `Err(<validator's variant>)`, typically `CandidateExecution` | Handle validator error variants | `candidate.rs:421`, `candidate.rs:320` |
| `run_bounded` spawn/poll failure | `Err(Io)` | Environment problem; nothing was committed | `candidate.rs:141`, `candidate.rs:182` |
| Lock record already present (including stale or malformed) | `Err(UpdateInProgress { lock })` | Inspect with `MutationLock::inspect`; a stale lock needs manual operator removal | `lock.rs:70-72` |
| Lock token exceeds 4096 bytes, or lock write fails | `Err(InvalidInput)` / `Err(Io)` (lock file removed on write failure) | Shorten identifiers | `lock.rs:63`, `lock.rs:77-80` |
| Injected `LockCreation` (test seam) | `Err(Injected)` | Proves the pre-lock path performs zero mutation | `transaction.rs:329` || Destination resolution fails during preflight | `Err(UnknownMember)` | Unreachable for plan-built members; treat as a core bug if seen | `transaction.rs:334` |
| Ownership `Foreign` or `Unknown` | **Receipt** `RolledBack`, `rollback_performed=false`, `rollback_verified=true`, `Cleaned`, `failure` phase `Ownership` | Treat as refused; do not retry without new ownership evidence | `transaction.rs:342-346` |
| Ownership `Absent` with `DenyCreate` | **Receipt** `RolledBack`, same shape | Authorize creation explicitly or do not retry | `transaction.rs:642-648` |
| Ownership answer differs between preflight and under-lock (flap) | **Receipt** `RolledBack`, zero mutation | Make the verifier deterministic; investigate the writer | `transaction.rs:637-641` |
| Missing or symlinked live parent | **Receipt** `RolledBack`, zero mutation; **no directory created** | Create the parent yourself, then retry | `transaction.rs:791-815` |
| Destination is a symlink or hard link (`nlink > 1`) at preflight or at backup | **Receipt** `RolledBack`, zero live mutation | Resolve the link; core will not follow or clobber it | `transaction.rs:746-786` |
| Staged member missing, non-regular, hard-linked, lacks a verified digest, or digest changed under the lock | **Receipt** `RolledBack`, `StageRevalidation`, zero live mutation | Re-prepare; the stage was tampered with | `transaction.rs:349-352`, `transaction.rs:659` |
| Backup directory creation fails | **Receipt** `RolledBack`, phase `Backup`, `rollback_performed=false` | Fix install-root writability | `transaction.rs:353-359` |
| Per-member backup rename fails (injected `CommitFault::Backup` or real I/O) | **Receipt** `RolledBack` if restore verified, else `RecoveryRequired` | Read `rollback_failure` and `recovery_path` | `transaction.rs:361-368`, `transaction.rs:563` |
| Any commit-loop error: destination resolution, `require_ready_parent`, staged resolution, or `fs::rename` | **Receipt** `RolledBack` (all old members restored, members that were absent removed) or `RecoveryRequired` | Never assume partial success; branch on disposition | `transaction.rs:369-444` |
| Injected `BeforeFirstCommit` / `Commit(MemberId)` / `CommitThenRollback` (test seams) | **Receipt** `RolledBack` with per-member restore, including removal of newly created members; `RecoveryRequired` for `CommitThenRollback` | Proves the multi-member restore contract and the `RecoveryRequired` transition | `transaction.rs:369`, `transaction.rs:381`, `transaction.rs:859-871`; `lib.rs:867`, `lib.rs:896` |
| Post-commit check returns `Err` or panics, `KeepInstalled` | **Receipt** `Committed` with `post_commit_failure` set (≤512 B detail), cleanup `Cleaned` | The generation is installed **and** the check failed; surface both facts | `candidate.rs:462-464` |
| Post-commit check returns `Err` or panics, `RollBack` | **Receipt** `RolledBack` with `post_commit_failure` **and** `failure`; `RecoveryRequired` if restore fails | Same, plus inspect `rollback_failure` if present | `transaction.rs:496` |
| Post-commit `KeepInstalled` then finalize removal fails | **Receipt** `Committed`, `RetainedForRecovery`, `failure` phase `Finalize` **and** `post_commit_failure` both set | Treat as installed-with-uncertain-cleanup; do not assume the backup is gone | `transaction.rs:524-540`, `lib.rs:807` |
| Restore cannot restore a member, or a post-rollback re-check finds an inconsistency | **Receipt** `RecoveryRequired`, `rollback_performed=true`, real `recovery_path`, **lock preserved** | Operator intervention; core will not retry or clean up | `transaction.rs:986-998` |
| Restore verified but backup removal fails | **Receipt** `RolledBack`, `rollback_verified=true`, `RetainedForRecovery` | Old generation is live; backup remains as evidence | `transaction.rs:969-983` |
| Process death, power loss, or kill at any point | No receipt; stage/backup/lock may remain | Out of scope by design. Use receipts and backups; there is no crash journal | `crates/eggup-core/docs/transaction.md:44-45` |
| Stale lock from a dead process | `Err(UpdateInProgress)` forever | Manual removal only; `inspect` will not delete it | `lock.rs:70`, `lock.rs:100` |

### Reading a receipt safely

- `disposition() == Committed` does **not** mean the caller's post-commit check passed.
  Check `post_commit_failure().is_some()`.
- `disposition() == RecoveryRequired` is never retryable through this API. It means
  core cannot prove either generation is coherent.
- `cleanup() == RetainedForRecovery` always comes with a real `recovery_path`.
- `rollback_performed() == false` with `RolledBack` means nothing live was touched —
  the safe common case for ownership refusals.

## 9. Concurrency

`MutationLock` provides exactly one guarantee: mutual exclusion between Eggup
transactions on the same installation root, via `create_new` on
`<root>/.eggup-mutation.lock` (`lock.rs:66`). One lock per root, not per product or
release, so two `InstallPlan`s for the same root serialize.

What concurrent callers observe:

| Scenario | Observable |
| --- | --- |
| Two Eggup commits, same root, no stale lock | First commits; second gets `Err(UpdateInProgress { lock })` at `transaction.rs:335`, before any mutation |
| Stale lock from a crashed process | Every later commit gets `Err(UpdateInProgress)` indefinitely. `MutationLock::inspect` (`lock.rs:100`) reports `Malformed`/`Held` but never removes it |
| Lock record replaced by a third party mid-transaction | `Drop` (`lock.rs:141`) refuses to remove it because the token bytes differ; the foreign file survives |
| Two Eggup commits, different roots | Fully independent; stage and backup names embed pid, an atomic counter, and a timestamp (`stage.rs:223-231`, `transaction.rs:12`) |
| Two transactions in one process, same root | Serialized the same way; the in-process atomic nonce prevents lock-token collision |

What core does **not** protect against, and a reviewer should keep in mind:

- **Non-Eggup writers.** The lock is advisory to other software. The revalidations
  narrow the window but do not eliminate it. In particular, `revalidate_destination`
  runs per member at backup time (`transaction.rs:573`) and per member under
  preflight (`transaction.rs:654`), but the **commit loop re-checks only parent
  readiness** (`transaction.rs:411`), not destination type or link count. A
  non-Eggup writer that swaps a destination for a symlink between backup and rename
  will have the rename replace the directory entry. This fails safe in the sense that
  `fs::rename` over a symlink replaces the symlink rather than following it, and the
  rollback path refuses to remove a non-regular destination
  (`transaction.rs:873-886`) and marks the transaction `RecoveryRequired`. But it is a
  clobber, not a refusal.
- **A non-Eggup reader can observe a partially swapped generation.** Because backup and
  commit are both `rename`s, a member's live path is empty between the two. There is no
  cross-process reader lock, so a consumer that reads the installation root without
  taking `MutationLock` can observe a mixed old/new generation mid-commit. Generation
  consistency is a property of the receipt, not of the filesystem at every instant.
- **PID reuse and clock skew.** Stage and backup uniqueness relies on
  pid + counter + `nanos`. Cross-process collisions are theoretically possible if pid
  and nanosecond coincide; `create_dir` retry (32 attempts) covers the realistic case.
- **Lock-record confidentiality.** The `0600` mode is applied after creation and the
  `set_permissions` result is discarded (`let _ =` at `lock.rs:84`).
- **Read-only stale handling.** By design, nothing in the crate removes a lock it did
  not create, and nothing ever treats a lock as expired.

## 10. Testing approach

The crate carries 50 `#[test]` functions in `lib.rs` and 5 examples. Nine tests are
`#[cfg(unix)]`-gated (`lib.rs:406`, `461`, `517`, `1064`, `1212`, `1263`, `1290`,
`1454`, `1534`), so a non-Unix run executes 41. The suite is entirely in-crate
(`mod tests` in `lib.rs`), which is why the fault seams
(`commit_with_fault` at `candidate.rs:485`, `commit_with_post_commit_fault` at
`candidate.rs:497`, `prepare_with_failure` at `stage.rs:32`, `CommitFault` at
`transaction.rs:299-307`) are `pub(crate)` rather than public API. `CommitFault` offers
`LockCreation`, `Backup(MemberId)`, `BeforeFirstCommit`, `Commit(MemberId)`,
`CommitThenRollback(MemberId, MemberId)`, `Finalize`, and `PostCommitRollback`, which is
what makes every receipt-producing branch reachable from a test.

What the tests actually demonstrate, by area:

| Area | Representative tests | Demonstrated |
| --- | --- | --- |
| Plan validation | `rejects_duplicate_normalized_destinations` (`lib.rs:222`), `rejects_empty_sets_and_escaping_destinations` (`lib.rs:254`), `rejects_symlink_sources_and_preserves_executable_intent_in_stage` (`lib.rs:408`) | Normalization-driven duplicate detection, `..` escape rejection, symlinked sources rejected |
| Stage isolation | `prepares_a_multi_member_bundle_without_touching_destinations` (`lib.rs:135`), `preparation_failure_cleans_private_stage` (`lib.rs:176`) | Destinations untouched by prepare; a failed prepare leaves no private state |
| Object-bound staging | `bound_source_stages_open_object_despite_foreign_replacement` (`lib.rs:271`), `…_after_root_rename` (`lib.rs:308`), `…_from_nonzero_cursor` (`lib.rs:346`), `bound_source_for_unknown_member_fails_closed` (`lib.rs:383`) | Staging follows the open handle, not the pathname; rewind semantics; unknown-member keys fail closed |
| Permission handling | `bound_source_preserves_executable_intent_that_the_fresh_stage_masks` (`lib.rs:463`), `bound_source_does_not_widen_a_non_executable_source` (`lib.rs:519`), `transaction_owned_state_is_owner_private` (`lib.rs:1536`) | Executable intent survives `0600` staging without widening a non-executable source; stage, backup, and lock modes |
| Structural error classification | `fault_category_is_structural_not_message_substring` (`lib.rs:553`) | `Injected` is a real variant, not a substring match on message text |
| Integrity ordering | `integrity_verification_precedes_candidate_validation` (`lib.rs:1143`), `unverified_integrity_none_cannot_commit` (`lib.rs:1490`), `hashes_known_bytes_and_parses_strict_sidecars` (`lib.rs:1121`) | No candidate runs before integrity; `NotRequired` cannot be committed through a real path; strict sidecar parsing |
| Bounded execution | `bounded_candidates_are_exact_and_environment_is_cleared` (`lib.rs:1214`), `bounded_runner_kills_timeouts_and_limits_output` (`lib.rs:1265`), `cross_member_identity_requires_bundle_agreement` (`lib.rs:1292`) | Exact stdout matching, environment clearing, kill-on-timeout, output cap, cross-member agreement |
| Ownership | `ownership_owned_allows_replacement_and_reports_no_failure` (`lib.rs:1374`), `ownership_foreign_and_unknown_fail_closed_with_zero_mutation` (`lib.rs:1384`), `ownership_absent_create_policy_is_enforced` (`lib.rs:1399`), `ownership_flapping_between_preflight_and_lock_fails` (`lib.rs:1413`), `exact_digest_verifier_proves_ownership` (`lib.rs:1615`), `absent_only_verifier_never_authorizes_replacement` (`lib.rs:1632`) | All four ownership values, absent-policy enforcement, the double-call flap contract, and the relative power of the three verifiers |
| Filesystem safety | `missing_parent_fails_without_creating_directories` (`lib.rs:1444`), `symlinked_parent_fails_closed` (`lib.rs:1456`), `destination_links_fail_immediately_before_backup` (`lib.rs:1066`), `staged_mutation_after_validation_fails_before_live_mutation` (`lib.rs:1468`) | Live parents are never created; symlinked parents and hard-linked/symlinked destinations fail closed; staged tampering is caught under the lock |
| Commit and rollback | `commits_a_complete_multi_member_generation_and_releases_lock` (`lib.rs:624`), `partial_commit_failure_restores_every_old_member` (`lib.rs:867`), `rollback_removes_new_members_that_were_absent_before_commit` (`lib.rs:896`), `precommit_and_backup_failures_restore_old_state` (`lib.rs:1007`) | Generation-consistent commit, lock release on success, per-member restore, removal of members that were absent pre-commit |
| Recovery | `rollback_failure_returns_recovery_required_and_retains_evidence` (`lib.rs:968`), `injected_lock_creation_failure_performs_no_mutation` (`lib.rs:1044`), `cleanup_failure_reports_real_backup_root` (`lib.rs:1510`) | `RecoveryRequired` retains backup evidence and the lock; pre-lock failure mutates nothing; the real backup root is reported |
| Lock inspection | `lock_contention_and_malformed_lock_fail_closed` (`lib.rs:948`), `lock_inspect_never_deletes_and_reports_status` (`lib.rs:1574`), `oversized_lock_record_is_malformed_not_deleted` (`lib.rs:1603`) | Contention, malformed and oversized records all fail closed and are never deleted |
| Post-commit | `post_commit_check_sees_complete_generation_and_holds_mutation_lock` (`lib.rs:655`), `post_commit_keep_installed_records_bounded_failure` (`lib.rs:691`), `post_commit_rollback_restores_old_generation_and_absent_members` (`lib.rs:721`), `post_commit_rollback_failure_retains_both_reports_and_evidence` (`lib.rs:774`), `keep_installed_finalize_failure_preserves_both_failure_facts` (`lib.rs:807`), `post_commit_panic_is_reported_and_rolls_back` (`lib.rs:841`) | Lock retention across the check, bounded detail, panic capture, both policies, and report/evidence retention |
| Receipt shape | `failure_reports_carry_phase_category_and_member` (`lib.rs:1647`), `checksum_only_state_is_explicit_in_docs` (`lib.rs:1658`) | Report structure; checksum-only is pinned as an explicit contract |

The 5 examples are runnable, asserted programs rather than unit tests, and each one
demonstrates a distinct caller obligation: `one_member.rs` (the supported happy path),
`multi_member.rs` (generation-consistent bundle commit), `ownership.rs` (prove-by-content
plus `DenyCreate`), `custom_validator.rs` (a caller-supplied `CandidateValidator`), and
`receipts.rs` (interpreting `Committed` / `RolledBack` / `RecoveryRequired`, including a
failed post-commit check that keeps the generation installed).

Gaps a reviewer should note:

- `test_support::FailureInjector` and `FailurePoint::Prepare` / `FailurePoint::Commit`
  (`test_support.rs:10-27`) are exercised only by
  `failure_injector_defaults_to_no_failure` (`lib.rs:127`). Real fault injection runs
  through `Stage::prepare_with_failure` (`stage.rs:32`), which only maps
  `FailurePoint::StageCreate` and `FailurePoint::StageCopy` (`stage.rs:37-38`), and
  through `CommitFault`. The `FailureInjector` struct carries no production coverage.
- The commit-fault seams (`CommitFault::BeforeFirstCommit`, `Commit`,
  `CommitThenRollback`) and `FailurePoint` are `pub(crate)` and test-only; there is no
  way for a downstream consumer to inject a commit fault, so rollback-path behavior is
  only testable in-crate.
- The backup-removal and rollback-restore failure paths that produce
  `RecoveryRequired` are reached via injected faults, not by real filesystem
  failures.
- `#[cfg(unix)]` gating means the timeout/kill, environment-clearing,
  symlinked-parent, destination-link, and permission-mode assertions are all absent from
  Windows and macOS-non-unix runs. `cargo test -p eggup-core` on Windows exercises 41
  tests.

Focused verification commands:

- `cargo test -p eggup-core` — the full crate suite.
- `cargo test -p eggup-core <name>` — a single test by name substring, e.g.
  `cargo test -p eggup-core post_commit_rollback`.
- `cargo clippy -p eggup-core --all-targets --locked -- -D warnings` — lint gate, which
  also covers the examples and the test module.
- `cargo run -p eggup-core --example receipts` — the receipt-interpretation example,
  which asserts its own expectations.

The workspace gate is `scripts/check-local.sh`; see
[tooling governance](tooling-governance.md). CI additionally runs MSRV `cargo check`
on 1.89.0, `cargo test` on macOS, and `cargo check` on Windows.

## Known doc/code drift

1. **`docs/transaction.md:31-34` overstates report retention.** It states that a
   rolled-back receipt "always carries the triggering `FailureReport`". On the
   `RolledBack` + backup-cleanup-failure path (`transaction.rs:963-983`), `failure()` is
   reassigned to a `Finalize` report and the original `Backup`- or `Commit`-phase cause
   is dropped. The post-commit case is the one exception that survives, because the
   caller re-attaches it as `post_commit_failure` at `transaction.rs:496`.

2. **`TransactionReceipt::rollback_verified` rustdoc is imprecise.**
   `transaction.rs:244-251` claims `false` "is never reported without a
   `rollback_failure` or a recorded cleanup failure to explain it". Two paths violate
   this: a clean `Committed` receipt reports `rollback_verified() == false` with no
   failure at all (`transaction.rs:544-552`), and the second verification pass in
   `restore_entries` (`transaction.rs:902-913`) can set `verified = false` **without**
   producing any `FailureReport`, yielding a `RecoveryRequired` receipt whose
   `rollback_failure` is `None`. That second case is the one a reviewer should know
   about, because the receipt then lacks a per-member explanation of why recovery is
   required.

3. **`docs/domain.md:11` states the permission behavior unconditionally.** The
   `0700`/`0600` values and the executable-intent mapping are `#[cfg(unix)]` only
   (`stage.rs:281-310`, `stage.rs:252-279`); on non-Unix `apply_permissions` is a no-op
   and executable intent is not observable. The same sentence also implies the whole
   transaction-owned tree is narrowed, whereas a backed-up file keeps its original
   mode. See [Platform-conditional behavior](#platform-conditional-behavior) and
   invariant 7a.

4. **The previous revision of this deep dive was stale on the public surface.** It
   omitted `BoundSources` and `InstallPlan::prepare_with_bound_sources` (the
   object-bound prepare seam now used by `eggup-archive` and `eggup-eggpack`), it
   described backup as a copy rather than the `rename` it is
   (`transaction.rs:586`), and it described the `Injected` fault category as being
   derived from an `InvalidInput` message containing `"injected"`. Since the changelog
   entry for the structural variant, classification is structural (`error.rs:40`,
   `error.rs:50-52`; `transaction.rs:181`).

## 11. Cross-references

Sibling deep dives:

- [overview.md](overview.md) — module map, cross-cutting invariants, terminology
- [acquisition.md](acquisition.md) — transport-neutral fetch seam
- [eggfetch-adapter.md](eggfetch-adapter.md), [curl-adapter.md](curl-adapter.md)
- [service-lifecycle.md](service-lifecycle.md)
- [archive-extraction.md](archive-extraction.md)
- [eggpack-adapter.md](eggpack-adapter.md) — the only producer-type consumer
- [transport-footprint.md](transport-footprint.md)
- [tooling-governance.md](tooling-governance.md) — workspace, CI, plans/process

Normative contracts in this crate:

- [domain.md](../crates/eggup-core/docs/domain.md) — identities, ownership, filesystem
  safety rules
- [verification.md](../crates/eggup-core/docs/verification.md) — integrity and
  candidate-validation contract
- [transaction.md](../crates/eggup-core/docs/transaction.md) — states, receipts,
  failure semantics, and the explicit no-crash-journal claim
- [README.md](../crates/eggup-core/README.md),
  [CHANGELOG.md](../crates/eggup-core/CHANGELOG.md)

Decisions:

- [ADR-0001](../plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md) —
  layered mechanism and policy ownership
- [ADR-0002](../plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md) —
  multi-artifact transaction and explicit rollback semantics
- [ADR-0003](../plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md) —
  verification layers and transport neutrality

Planning and closure:

- [verified-update-core-roadmap.md](../plans/subsystems/verified-update-core-roadmap.md)
  — invariants, capability list, M001-M009 status
- [001](../plans/closure/verified-update-core/001-status.md) workspace foundation
- [002](../plans/closure/verified-update-core/002-status.md) domain and prepared
  transaction
- [003](../plans/closure/verified-update-core/003-status.md) mutation lock, commit,
  rollback, recovery
- [004](../plans/closure/verified-update-core/004-status.md) integrity and candidate
  validation
- [005](../plans/closure/verified-update-core/005-status.md) prequalification safety and
  API corrective
- [006](../plans/closure/verified-update-core/006-status.md) core package qualification
- [007](../plans/closure/verified-update-core/007-status.md) post-commit failure policy
  and deferred finalization
- [008](../plans/closure/verified-update-core/008-status.md) core/archive consumer
  package qualification
- [009](../plans/closure/verified-update-core/009-status.md) core/archive 0.1.2
  publication
