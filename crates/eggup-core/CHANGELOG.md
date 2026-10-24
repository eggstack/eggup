# Changelog

## 0.2.0 — 2026-10-23

Published to crates.io (`plans/closure/verified-update-core/019-status.md`) as
the first crate of the lockstep 0.2.0 train: `eggup-archive`,
`eggup-acquisition`, `eggup-curl`, `eggup-eggfetch`, `eggup-service`, and
`eggup-eggpack` are published from the same source commit, after this crate.
Dependency surface remains `sha2` plus a **Windows-only** `self-replace`; the
Unix/macOS package graph is unchanged and no new dependency was added.
Integrity evidence is SHA-256 checksum only — no authenticity or signature
claims. Self-update selects no release and infers no version.

This version carries the bounded stdin execution corrective (M017), the
bounded-stdin outcome-priority corrective (M018), the public-API contract
truthfulness correction (M015), and the bounded candidate deadline correction
(M014). None of them was published before: `0.1.3` still carries every one of
them. Current-executable transaction parity (M010), proof-authorized stale-lock
recovery (M011), the earlier Core audit fixes, and the M012 public-error
compatibility correction were published at `0.1.3`; their entries are carried
below unchanged in substance, and they are not new in this version.

**Compatibility, stated precisely.** `Error` keeps exactly the seven variants
`0.1.3` published, so an exhaustive `match` on it compiles unchanged against
both versions, and `Error::Injected` remains `#[cfg(test)]`-only.
`RecoveryError` keeps its two variants and its `#[non_exhaustive]` attribute,
and `CleanupDisposition` keeps the three members `0.1.3` published, including
`DeferredToProcessExit`. The transaction, staging, locking, commit, rollback,
and ownership machinery are untouched. `FailureCategory` keeps
`#[non_exhaustive]` and the `as_str()` strings of the seven categories that
remain; the one removal is `FailureCategory::RetainedEvidence` with its
`retained-evidence` string, and that removal is the only source break this
version introduces.

**Additive.** `MAX_STDIN_BYTES` and `CommandSpec::stdin_data` are new public
items, and the `operation` string of `Error::Io` is now a published
classification surface for `run_bounded` — `"spawning bounded candidate"`,
`"polling bounded candidate"`, and the new `"writing bounded stdin"`. Beyond
the removal named above, nothing else in the public surface moved, and the new
builder is optional.

**Behaviour a consumer can observe.** The deadline is now an unconditional
maximum wall-clock bound on the whole bounded execution, so a call that could
run past its own documented bound now returns at the deadline with `timed_out`
set. A bounded stdin write failure observed at or after the deadline is now the
deadline outcome — `Ok` with `timed_out` set — where `0.1.3` reported it as an
`Err` in some runs and as the deadline outcome in others, so the same input and
the same timeout no longer produce two different public classifications. A
write failure observed before the deadline is still an `Err`, and an undelivered
payload is never reported as a success.

**One consumer action is required.** `eggup-core 0.1.3` on crates.io does publish
`FailureCategory::RetainedEvidence`, so a downstream crate that named it
explicitly — a match arm, an equality assertion, or a constant `as_str()`
call — fails to compile against `0.2.0`. That is a compile-time break, not a
silent behaviour change, and deleting the name reference is the whole of the
required work. Beyond that, no consumer migration is required: no data format
changed, no configuration changed, and no other consumer action is needed beyond
picking up the new version.

**Bounded-stdin outcome-priority corrective (Verified Update Core M018).** The
published rule for `run_bounded`'s bounded stdin write was not the rule the
implementation applied, and the failure taxonomy's producer count was stated
for a test build. This corrective states one rule, implements it so the
rustdoc is literally true, and corrects the statements that were not.

- **Changed: the deadline has priority over the stdin writer's verdict, and the
  verdict is classified by its own observation instant.** The writer now stamps
  its verdict with the instant at which it observed the write's completion, so
  "before or after the deadline" is a fact about the write and never a fact
  about when `run_bounded` happened to poll for the verdict. A write failure
  observed **before** the deadline is an `Err`
  (`Error::Io { operation: "writing bounded stdin", source }`), as before. A
  write failure observed **at or after** the deadline no longer turns the
  deadline outcome into an `Err`: the deadline already decided that outcome,
  and the call returns `Ok` with `timed_out` set. A writer that had not
  delivered its verdict when the deadline was reached is abandoned, exactly
  like a reader blocked on a pipe a surviving descendant holds.
- **Behaviour change.** Before this corrective, a write failure observed after
  the deadline was reached was reported as an `Err` in some runs and as the
  deadline outcome in others, depending on the poll phase, so the same input
  and the same timeout produced two different public classifications. The
  classification is now a function of the write's own observation instant. An
  undelivered payload is never reported as a success: the outcome is either an
  `Err` or an `Ok` whose `timed_out` flag is set.
- **Fixed: the deadline's effect on the writer is stated truthfully.** Killing
  the direct child breaks the child's stdin pipe only for a child that is still
  running and whose stdin pipe no descendant inherited. A descendant that
  inherited the pipe keeps it open, so the writer stays blocked until that
  descendant exits or closes the pipe; the earlier wording overstated this.
- **Fixed: the failure taxonomy's producer count.** `FailureCategory`'s rustdoc
  said that six categories are produced by the crate's internal
  error-to-category mapping. Five are produced in a shipped build; the sixth,
  `FailureCategory::Injected`, is produced by that same mapping only in this
  crate's own test builds.
- **Unchanged: the public surface.** `Error` keeps exactly the seven published
  variants (plus the `#[cfg(test)]`-only `Error::Injected`), `FailureCategory`
  keeps its members and its published `as_str()` strings, and no new public
  item was added. The dependency surface is unchanged: `sha2` plus the
  Windows-only `self-replace`. No new dependency and no `unsafe`.
- **Published in `0.2.0`, and no migration required.** The published
  `eggup-core 0.1.3` still carries the pre-correction behaviour; this
  correction reaches consumers in `0.2.0`. No consumer needs to change
  anything.

**Bounded stdin execution corrective (Verified Update Core M017).** `run_bounded`
gave every child a **null** stdin, so a downstream adapter whose workflow needs a
child to read a payload — a service manager installing a unit or a crontab from
standard input — could not delegate process execution to core without losing
that capability, and had to keep a runner of its own. Core now owns the bounded
stdin capability.

- **Added: `CommandSpec::stdin_data`.** One infallible builder attaches a
  bounded stdin payload. Without a payload the child's stdin stays null and
  `run_bounded` behaves exactly as before.
- **Added: `MAX_STDIN_BYTES`.** The inclusive bound on one payload (1 MiB). A
  larger payload is rejected with
  `Error::InvalidInput("stdin data exceeds the bounded stdin bound")` **before
  any child is spawned**, so the bound fails closed and no process is created.
- **Added: a bounded stdin write.** An attached payload is written by a
  dedicated writer thread that starts only after the output readers are
  attached, so a child that fills stdout or stderr before draining its stdin
  cannot deadlock both sides.
- **The deadline now bounds the writer too.** The wall-clock deadline remains an
  unconditional maximum bound on the whole bounded execution. At the deadline
  the direct child is killed, which breaks the child's stdin pipe for a child
  that is still running and whose stdin pipe no descendant inherited; a
  descendant that inherited the pipe keeps it open, so a writer still blocked
  in `write` is abandoned explicitly and ends only when that descendant exits
  or closes the pipe, exactly like a reader blocked on a pipe a surviving
  descendant holds. A bounded execution is not reported as complete until the
  writer has delivered its verdict, so a write failure is never lost behind a
  "the child exited" observation.
- **Added: a typed write failure.** A stdin payload that cannot be written —
  including the broken pipe a child produces when it exits without reading its
  stdin — kills and reaps the child and then fails closed with
  `Error::Io { operation: "writing bounded stdin", source }`, when the failure
  was observed before the deadline. A failure observed at or after the
  deadline does not become an error: the deadline already decided that
  outcome, and the call returns `Ok` with `timed_out` set (see the M018 entry
  above). When a failed write and an output overflow are live at the same
  time, the write failure wins.
- **Added: a stable error-classification surface.** The `operation` string of
  `Error::Io` is now a published contract for `run_bounded`: `"spawning
  bounded candidate"`, `"polling bounded candidate"`, and `"writing bounded
  stdin"`. A downstream adapter may branch on them, and the `run_bounded`
  rustdoc states what each one means and what an adapter is expected to do
  with it.
- **Unchanged: the public error surface.** No `Error` variant was added,
  removed, or reshaped. `Error` keeps exactly the seven published variants
  (plus the `#[cfg(test)]`-only `Error::Injected`), so an exhaustive `match` on
  it compiles unchanged. `FailureCategory` is untouched.
- **Unchanged: the dependency surface.** `sha2` plus the Windows-only
  `self-replace`. No new dependency, no `unsafe`, no process-group or
  job-object mechanism, and no change to the transaction, staging, locking,
  commit, rollback, or ownership machinery. SHA-256 remains integrity evidence
  only; no authenticity or signature claim was introduced.
- **Published in `0.2.0`, and no migration required.** The published
  `eggup-core 0.1.3` still gives every child a null stdin and still lacks
  `stdin_data` and `MAX_STDIN_BYTES`; the capability reaches consumers in
  `0.2.0`. The addition is source-compatible: no consumer needs to change
  anything, and the new builder is optional.

**Public-API contract truthfulness correction (Verified Update Core M015).** The
published failure taxonomy contained a category that could never be produced:
`FailureCategory::RetainedEvidence` was never constructed by any producer, yet
`as_str()` published the name `retained-evidence` to consumers. Separately, the
crate's public fallible surface documented **no** failure modes at all — it
carried zero `# Errors` rustdoc sections across all of `src/`.

- **Removed: `FailureCategory::RetainedEvidence`.** The variant and its
  `as_str` arm are gone. The variant was residue of M012's relocation of
  retained-evidence reporting into `RecoveryError`; that meaning is carried by
  `TransactionDisposition::RecoveryRequired`, by
  `CleanupDisposition::RetainedForRecovery`, and by the real path in
  `TransactionReceipt::recovery_path` — never by a failure category. The
  published taxonomy is now exactly seven categories, and **every one of them
  describes a failure that can actually occur**.
- **Unchanged: the remaining `as_str()` strings.** `lock-contention`,
  `ownership-conflict`, `verification`, `filesystem`, `invalid-input`,
  `injected`, and `post-commit-check` are byte-identical to what `0.1.3`
  published. The only string that disappears is `retained-evidence`, and no
  runtime failure could ever carry it — no producer existed — so no consumer
  could have observed it in a `FailureReport`, in a receipt, or in a
  persisted category.
- **Unchanged: the failure space.** No produced `FailureReport`, no
  `TransactionReceipt`, no `Error`, and no `RecoveryError` differs before and
  after. `Error` keeps **exactly** the seven variants `0.1.3` published (plus
  the `#[cfg(test)]`-only `Error::Injected`), and `RecoveryError` keeps its
  two variants and its `#[non_exhaustive]` attribute.
- **Added: failure-mode documentation on the public fallible surface.** Every
  public fallible item now carries an `# Errors` section that states which
  `Error` variants it can return and under what condition, that a receipt —
  not an `Err` — is how a post-lock outcome is reported, that the ownership
  direction stays fail-closed (`Owned` is never inferred; a verifier's
  ambiguity is `Unknown`), and that a member without verified integrity
  evidence is rejected with no escape hatch. The wording points at
  `crates/eggup-core/docs/{domain,verification,transaction}.md` and
  `architecture/core-transaction.md` §8 rather than restating them. SHA-256
  remains integrity evidence only: no authenticity or signature claim was
  introduced anywhere.
- **Added: a crate-local doc-accuracy check.** `clippy::missing_errors_doc` is
  now enabled for `eggup-core` only, so the existing gate's `-D warnings` makes
  a missing `# Errors` section a hard error for this crate. The workspace lint
  table is unchanged and no other crate is affected.
- **Documented, not removed: `FailureCategory::Injected` is test-only.** It is
  produced only by this crate's own test fault harness, because its only
  producer is the `#[cfg(test)]`-only `Error::Injected`; a shipped
  `eggup-core` build never produces the `injected` category. This is the same
  class of unreachable branch as the removed variant, so it is now stated
  explicitly in the rustdoc rather than left implicit.
- **Compatibility.** Removing a variant of the `#[non_exhaustive]`
  `FailureCategory` does not break a downstream matcher: every downstream
  matcher must already carry a wildcard arm, and that arm now covers one
  fewer variant. **The one caveat**: `eggup-core 0.1.3` on crates.io *does*
  publish `FailureCategory::RetainedEvidence`, so a downstream crate that
  named it explicitly — a match arm, an equality assertion, or a constant
  `as_str()` call — fails to compile against `0.2.0`. That is a compile-time
  break, not a silent behavior change, and it is the only downstream break
  this correction introduces. **No consumer migration is required** beyond
  deleting such a name reference.
- **Published in `0.2.0`.** `eggup-core 0.1.3` on crates.io still carries the
  defect. The fix reaches consumers in `0.2.0`. The lockstep 0.2.0 train
  repins `eggup-eggpack` to `eggup-core = "=0.2.0"` and publishes that adapter
  after this crate, so the correction reaches it in the same train. The
  published `0.1.3` release and its `v0.1.3` tag are not moved.
- **No migration path** is required: no API changed shape beyond the removal
  above, no data format changed, and no consumer action is needed beyond
  picking up the fixed version.

**Bounded candidate deadline correction (Verified Update Core M014).** `run_bounded`
documented `CommandSpec::timeout` as the maximum wall-clock execution time of a
candidate command, but the implementation evaluated the deadline only while the direct
child was still alive and joined its reader threads only on the success path. A
descendant that inherited the stdout or stderr pipe and held it open after the direct
child exited pinned the call indefinitely, and the timeout path additionally left both
reader threads unjoined. A published library could therefore hang past its own
documented safety bound.

- **Fixed: the deadline is now an unconditional maximum wall-clock bound** on the whole
  bounded execution. It is evaluated on every poll whether or not the direct child has
  already exited, so `run_bounded` always returns. When the deadline elapses, a still
  running direct child is killed and reaped exactly as before; an already exited child
  is not killed and not reaped again, and the call no longer waits for pipe EOF.
- **Changed: the descendant case is now bounded and fail-closed.** When the direct
  child exited but a descendant still held the output pipe open, the call returns at the
  deadline with `timed_out` set, `output_limited` clear, `success()` false, and the
  direct child's exit code still observable through `exit_code`. The stream the
  descendant still holds contributes no bytes, because they were never captured. Core
  terminates only the direct child it spawned; a descendant is never killed.
- **Fixed: reader-thread lifecycle is now explicit.** A reader that delivered its
  result is joined, which is bounded because a reader only has to return after its
  send. A reader still blocked in `read()` at the deadline cannot be joined without
  waiting past the deadline, and core cannot terminate a descendant it does not own —
  it has no `libc`, `nix`, or `windows-sys` dependency and `unsafe` is denied — so such
  a reader is abandoned explicitly and ends when the pipe reaches EOF.
- **Unchanged: the public surface.** `CommandSpec`, `CommandOutput` and its accessors
  keep their published shape, `Error` keeps exactly the seven published `0.1.2`
  variants, the output bound stays per stream at the published defaults (64 KiB, 5 s),
  and the overflow semantics stay as published: an overflowing stream is reported as
  empty. The ownership model, the transaction state machine, the lock, the receipts,
  and the failure taxonomy are untouched.
- **Compatibility.** Source compatibility is unchanged. The behavior change is
  confined to calls that never returned under `0.1.3`: such a call now returns at the deadline
  with `timed_out == true`. No caller can depend on a hang, so this is a defect
  correction rather than a semantic break.
- **Published in `0.2.0`.** `eggup-core 0.1.3` on crates.io still carries the
  defect; the fix reaches consumers in `0.2.0`. The lockstep 0.2.0 train
  repins `eggup-eggpack` to `eggup-core = "=0.2.0"` and publishes that adapter
  after this crate, so the correction reaches it in the same train.
- **No migration path** is required: no API changed, no data format changed, and no
  consumer action is needed beyond picking up the fixed version.

**Compatibility correction (Verified Update Core M012).** An earlier revision of
this file claimed that adding variants here was additive because `Error`,
`CleanupDisposition`, and `FailureCategory` were all `#[non_exhaustive]`. Only
`FailureCategory` was. `Error` and `CleanupDisposition` are closed enums, so an
added variant breaks an exhaustive downstream `match`. That claim was wrong and
is corrected here rather than left standing.

The state after M012:

- `Error` keeps **exactly** the seven variants `0.1.2` published, so an
  exhaustive `match` on `Error` compiles unchanged against both versions.
  `Error::Injected` no longer exists in any packaged build; the fault harness
  classifies injected failures structurally through a `#[cfg(test)]`-only
  variant. `Error::RecoveryRequired` moved to a new `RecoveryError` type.
- `FailureCategory` **is** `#[non_exhaustive]`, so
  `FailureCategory::RetainedEvidence` is genuinely additive. (M015 later
  removed that variant, because no producer ever existed for it; see the M015
  entry above. The `#[non_exhaustive]` attribute itself is unchanged.)
- `CleanupDisposition` is **not** `#[non_exhaustive]`, so
  `CleanupDisposition::DeferredToProcessExit` is a source-visible addition. It
  is required by M010: a kept-installed Windows self-update cannot unlink the
  previous generation while this very process still maps it, and no truthful
  value exists in the two-variant 0.1.2 set (`Cleaned` would claim the old image
  is gone; `RetainedForRecovery` would wrongly page an operator for a state that
  needs no operator action). It was **not** marked `#[non_exhaustive]` to paper
  over this: M012 deliberately avoids that attribute on already-published enums
  because adding it breaks the same matches a new variant breaks, and future
  extensibility belongs to an intentional 0.2/1.0 boundary. The affected
  surface is a single accessor on `TransactionReceipt`; no `eggstack` consumer
  references `CleanupDisposition` at all.

Integrity evidence is still SHA-256 checksum only — no authenticity or
signature claims. Self-update selects no release and infers no version.

### Current-executable transaction parity

- **Added: `CurrentExecutable` and `InstallPlan::for_current_executable`.** A
  program can now replace the executable it is currently running through the
  ordinary one-member transaction model, rather than needing a bespoke updater.
  This is not a second state machine: preparation, integrity verification,
  candidate validation, locked ownership revalidation, staged-digest
  revalidation, commit, rollback, and the receipt are the existing ones.
  `CurrentExecutable::resolve` / `bind` canonicalize the running image, so an
  invocation through a symlink updates the real target and never overwrites the
  link object. A destination whose exact identity cannot be proven — a symlink, a
  directory, or a hard-linked image — is refused rather than replaced.
- **Added: `StagePlacement`.** A self-update plans
  `StagePlacement::InsideInstallationRoot`, so the private stage and the backup
  set both live inside the executable's own directory. A self-updater therefore
  needs write authority where its executable lives and never in that directory's
  parent. Ordinary plans keep `StagePlacement::SiblingOfInstallationRoot` and
  are unchanged; `InstallPlan::stage_placement` exposes which one applies.
- **Added: `CleanupDisposition::DeferredToProcessExit`.** After a `KeepInstalled`
  self-update on Windows, the previous generation is still mapped by this very
  process and cannot be unlinked. It has already been renamed to a
  transaction-owned path, so it stays rollback-addressable until the caller's
  policy resolves; only then is its deletion scheduled for process exit. The
  receipt reports this distinctly rather than calling it either cleaned or
  stranded, and never reports it for an ordinary multi-member transaction.
- **Added: retained-evidence reporting.** The real retained evidence path is
  carried when a transaction-owned artifact cannot be resolved automatically,
  plus `FailureCategory::RetainedEvidence`. This was originally added as
  `Error::RecoveryRequired`, which would have been a **breaking** change for an
  exhaustive `match` on the closed `Error` enum; M012 moved it to
  `RecoveryError` instead, so the published `Error` variant set is preserved.
  Receipt-level recovery semantics are unchanged: transaction rollback
  uncertainty is still reported through `TransactionDisposition::RecoveryRequired`
  and `CleanupDisposition::RetainedForRecovery`.
- The bound executable identity is re-proved under the mutation lock and again
  immediately before the first live rename, so an image swapped in between
  fails closed before anything is moved.
- Dependency impact: `eggup-core` gains exactly one dependency,
  `self-replace`, under `[target.'cfg(windows)'.dependencies]`. The Unix and
  macOS dependency graphs are unchanged (`sha2` only). The helper is called only
  after `KeepInstalled` has resolved and rollback is no longer possible; a
  direct `self_replace()` of the running image is never used, because that would
  hide the old generation before Eggup's own `RollBack` decision.

### Proof-authorized stale-lock recovery

- **Added: `LockObservation`, `StaleLockDecision`, `StaleLockVerifier`, and
  `MutationLock::acquire_with_recovery`.** Stale-lock recovery is now possible,
  but Core never decides staleness: it exposes a bounded, typed observation of
  one exact record and performs the mutation only when the caller's verifier
  returns `ProvenStale`. `Active` and `Unknown` retain the record and return
  contention. PID liveness, record age, executable name, and service state stay
  consumer policy; Core consults none of them.
- **Unchanged: `MutationLock::acquire` remains fail-closed** and never recovers a
  record, and `MutationLock::inspect` remains read-only and unchanged.
  Recovery is opt-in through `ValidatedTransaction::commit_with_stale_lock_recovery`
  or `commit_with_post_commit_and_stale_lock_recovery`; existing commit methods
  keep their exact signatures and semantics.
- Claiming is race-safe without unsafe code: the record is re-read, renamed into
  a unique Eggup-owned same-directory claim path, and the claimed object is
  re-read and required to still equal the authorized observation before
  anything is created. A record that changed in between is never deleted. If
  another writer creates the lock after the claim, that writer wins and its
  record is never removed.
- Malformed, oversized, symlinked, non-regular, non-UTF-8, and unreadable
  records never reach destructive recovery. An unrecognised record format stays
  observable and byte-identifiable but reports no parsed fields, so a caller
  needing fields cannot prove staleness from it.
- No process-enumeration dependency enters Core, and a crash after moving a
  record to a claim path is never treated as permission to delete it later.

### Workspace bug audit fixes

- **Fixed (breaking, pre-1.0): bound-source staging dropped the executable bit.**
  `stage_bound_source` creates a fresh owner-private staged file and hardcodes
  `0600`; `apply_permissions` then read the executable intent back *from that
  staged file*. Every bound member therefore staged and installed non-executable
  under `PermissionsIntent::Preserve`, even when the source was `0755`.
  Executable intent is now read from the bound handle's own metadata (an `fstat`
  on the already-open object) before the stage exists. The `0600` stage hardening
  is unchanged. Non-executable bound sources still land `0600`, and broad source
  modes are still never inherited.
- **Fixed: a fully successful rollback was reported as unverified.** When
  restoration succeeded but the backup set could not be removed, `finish_failure`
  returned `RecoveryRequired` with `rollback_verified: false`, a real backup
  `recovery_path`, the original failure report, and no `rollback_failure` — a
  combination that implied a failed restoration that never happened. The
  committed path already handled this case; the rollback path now mirrors it:
  `RolledBack` with `rollback_verified: true`, the retained real backup root, and
  the cleanup problem reported as a `Finalize`-phase failure. `RecoveryRequired`
  is still reserved for genuinely unverified restoration.
- **Changed: `FailureCategory::Injected` is now structural.** It was derived by
  checking whether an `InvalidInput` message contained the substring
  `"injected"`, so caller-supplied text could claim the test-harness category.
  Fault injection now uses a dedicated internal marker that is compiled only for
  this crate's own tests, converted straight into
  `FailureCategory::Injected`. Message-substring classification is gone and
  cannot come back. The packaged public `Error` enum is untouched, so there is no
  downstream arm to add and no way for a production caller to claim the
  test-harness category.

- **Added: `RecoveryError` and `RecoveryResult<T>`.** A `#[non_exhaustive]` error
  type separating ordinary core failures (`RecoveryError::Core`) from retained
  evidence (`RecoveryError::RecoveryRequired { evidence, detail }`). It exposes
  the real retained path through `evidence()` without parsing `Display`, carries
  bounded detail through `detail()`, and chains as a `std::error::Error` whose
  `source()` is the inner `Error`. `From<Error>` keeps propagation ergonomic.
  Only the three M011 entry points
  (`MutationLock::acquire_with_recovery`,
  `ValidatedTransaction::commit_with_stale_lock_recovery`, and
  `commit_with_post_commit_and_stale_lock_recovery`) return it; every published
  signature, including `MutationLock::acquire` and all `commit` variants, keeps
  `Result<T, Error>`.

The `eggup-archive` and `eggup-service` crates consume this crate; see their
changelogs for changes that cross the boundary.

## Unreleased

Nothing in this section has been published. `0.2.0` is the published baseline on
crates.io, and no publication milestone authorizes these entries.

## 0.1.3 — 2026-10-06

Published to crates.io (`plans/closure/verified-update-core/013-status.md`).
Dependency surface remains `sha2` plus a **Windows-only** `self-replace`; the
Unix/macOS package graph is unchanged. Integrity evidence is SHA-256 checksum
only — no authenticity or signature claims. Self-update selects no release and
infers no version.

This version carries current-executable transaction parity (M010),
proof-authorized stale-lock recovery (M011), the earlier Core audit fixes, and
the M012 public-error compatibility correction. Because M010/M011 were never
published, this is their first appearance on the registry.

**Compatibility, stated precisely.** `Error` keeps exactly the seven variants
`0.1.2` published, so an exhaustive `match` on it compiles unchanged against
both versions — proven by an external fixture that is byte-identical between
the two and fails against the pre-M012 tree. Retained evidence moved to a new
`#[non_exhaustive] RecoveryError`. The one intentional source-visible addition
is `CleanupDisposition::DeferredToProcessExit`, on a closed enum, required by
M010 so a kept-installed Windows self-update does not report its still-mapped
old generation as either cleaned or stranded. The compatibility section below
explains why that addition is not removable.

## 0.1.2 — 2026-09-28

Published to crates.io (`plans/closure/consumer-adoption/006-status.md`; the
registry audit in `plans/closure/verified-update-core/009-status.md` observed
`0.1.0` and `0.1.1` as the prior published versions). The workspace version bump
to `0.1.2` carries one additive API. Dependency surface remains `sha2` only.

- Bound-source staging seam (M001d, additive): `BoundSources` and
  `InstallPlan::prepare_with_bound_sources` let a caller supply already-open
  member objects instead of pathname sources. The matching `eggup-archive` side
  is `PersistedExtraction::into_bound_sources()` and
  `BoundExtraction::into_members()`. With a bound handle supplied, staging never
  reopens the recorded path, so a member-entry replacement or root rename cannot
  redirect staged bytes and a foreign replacement stays untouched. The recorded
  path becomes advisory diagnostics only. The ordinary path-source
  `ArtifactMember::new` API and its semantics are unchanged for non-archive
  callers. Integrity remains SHA-256 checksum evidence only — no authenticity or
  signature claims.

## 0.1.1

- ADR-0002 post-commit policy via `ValidatedTransaction::commit_with_post_commit`.
  The transaction retains its mutation lock and rollback set through one caller
  check, records bounded check-failure evidence, and either keeps the complete
  new generation or restores the old one. Existing immediate `commit()` is
  unchanged. Callback panics are converted into failed-check evidence;
  process-crash durability is not claimed.
- Workspace foundation: Rust 1.89 `eggup-core` plus deterministic test fixture
  support, validated private staging, synchronous commit/rollback receipts,
  native SHA-256 integrity checks, and bounded candidate validation phases.
- M005 corrective (breaking pre-1.0): canonical
  `Absent | Owned | Foreign | Unknown` ownership with a consumer-provided
  verifier and an absent-create policy; no automatic destination-parent creation;
  staged-digest revalidation under lock; removed the unenforced authenticity API;
  renamed cleanup disposition to `CleanupDisposition`; structured failure
  phase/category/member reports with real recovery paths; owner-private
  stage/backup/lock permissions with collision-resistant names and
  ownership-checked cleanup; truthful fail-closed lock inspection with no
  automatic stale removal; checksum-only documentation.
- M006 qualification: package metadata (keywords, categories, homepage, docs
  URL, exclude rules); `#[non_exhaustive]` on extensible enums; five runnable
  examples (one-member, multi-member, custom validator, ownership verifier,
  receipt interpretation); `cargo package` and `cargo publish --dry-run` green;
  crates.io name verified available; dependency surface remains `sha2` only;
  MSRV 1.89 and platform lanes documented.

## 0.1.0

First crates.io publication (`plans/closure/consumer-adoption/001-status.md`).
