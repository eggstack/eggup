# Changelog

## Unreleased

Nothing in this section has been published. `0.1.2` remains the published
baseline on crates.io, and no publication milestone authorizes these entries.
The `Error::Injected` variant added below is a **breaking** change for pre-1.0
consumers: an exhaustive `match` on the non-`#[non_exhaustive]` `Error` enum
needs a new arm. No other consumer migration is required.

Fixes from the workspace bug audit. No new features; dependency surface remains
`sha2` only. Integrity evidence is still SHA-256 checksum only — no authenticity
or signature claims.

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
  Fault injection now uses a dedicated `Error::Injected` variant. This adds a
  variant to the public `Error` enum, which is not `#[non_exhaustive]` — an
  exhaustive downstream match on `Error` needs a new arm. `Error::injected` is
  `pub(crate)`; production code cannot construct the variant.

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
