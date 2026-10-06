# Changelog

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

## Unreleased

Nothing in this section has been published. `0.1.3` is the published baseline on
crates.io, and no publication milestone authorizes these entries.

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
  `FailureCategory::RetainedEvidence` is genuinely additive.
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
  Only the three still-unpublished M011 entry points
  (`MutationLock::acquire_with_recovery`,
  `ValidatedTransaction::commit_with_stale_lock_recovery`, and
  `commit_with_post_commit_and_stale_lock_recovery`) return it; every published
  signature, including `MutationLock::acquire` and all `commit` variants, keeps
  `Result<T, Error>`.

The `eggup-archive` and `eggup-service` crates consume this crate; see their
changelogs for changes that cross the boundary.

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
