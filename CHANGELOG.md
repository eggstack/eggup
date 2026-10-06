# Changelog

## 0.1.2 — 2026-09-28, completed 2026-10-04

The 0.1.2 source version was published to crates.io in four steps. On 2026-09-28
(Verified Update Core M009) `eggup-core 0.1.2` and `eggup-archive 0.1.2` were
published. On 2026-10-02 (Eggpack Manifest Interoperability M004) the
producer-schema-dependent half of the same 0.1.2 source version was published:
`eggup-acquisition 0.1.2`, `eggup-eggfetch 0.1.2`, and `eggup-eggpack 0.1.2`.
On 2026-10-04 (Acquisition Transport M009) `eggup-curl 0.1.2` was published. On
2026-10-05 (Service Lifecycle M009) `eggup-service 0.1.2` was published.

**Complete crates.io 0.1.2 publication set:** `eggup-core`, `eggup-archive`,
`eggup-acquisition`, `eggup-eggfetch`, `eggup-eggpack`, `eggup-curl`,
`eggup-service`.

`eggup-service` was the last crate still lagging the workspace. Its `0.1.2` was
published **later, from a different source commit** than the shared `v0.1.2` tag:
the tag still denotes the `eggup-core`/`eggup-archive` publication source and was
not moved or recreated. `eggup-transport-footprint` is `publish = false` by design
and has no edge in the published adapter graph.

Integrity remains SHA-256 checksum evidence only; no authenticity or signature
claims are made by any of these crates.

- `eggup-core 0.1.2`: workspace version bumps to 0.1.2 to carry the M001d
  additive bound-source API (`BoundSources` +
  `InstallPlan::prepare_with_bound_sources`). Dependency surface remains
  `sha2` only. Published 2026-09-28.

- `eggup-archive 0.1.2` (first crates.io publication): bounded allowlisted
  extraction for verified local tar.gz and zip archives; object-bound
  handoff (`PersistedExtraction::into_bound_sources`,
  `BoundExtraction::into_members`, `BoundMember`); retain
  extraction-root authority through `fs_at 0.2.1` (`mkdir_at`,
  `open_dir_at`, `unlink_at`, `rmdir_at`) for creation, replace-race
  isolation, and recursive cleanup; no runtime dependency on `eggup-core`;
  dev-only `eggup-core` path dependency preserved so the package can
  verify before publication. Package metadata declares explicit
  `homepage` and `documentation` fields and records the absence of
  authenticity or signature support. Published 2026-09-28.

- `eggup-acquisition 0.1.2` (first publication of this version): the
  transport-neutral acquisition seam and its deterministic fixture transport.
  The `FetchLimits` finite-bounds corrective is carried here: `max_artifact_bytes`
  is a plain `u64` and `None` is no longer representable. No Eggup or producer
  dependency. Published 2026-10-02.

- `eggup-eggfetch 0.1.2`: native `eggfetch-core` HTTP adapter carrying the
  matching `FetchLimits` migration (`max_artifact_bytes: u64`, UTF-8-safe
  `bound()`), plus the `Unavailable`/composition API qualified against the
  0.1.2 acquisition seam. This publication is required for registry coherence:
  published `eggup-eggfetch 0.1.1` declares `eggup-acquisition ^0.1.0` but its
  source uses `Option<u64>`, so a fresh resolve after `eggup-acquisition 0.1.2`
  publishes would otherwise select an incompatible pair. Published 2026-10-02.

- `eggup-eggpack 0.1.2` (first crates.io publication): optional Eggpack
  ReleaseManifest adapter. This version replaces the immutable-Git
  `eggpack-manifest` dependency with the registry dependency
  `eggpack-manifest = "=0.1.0"` (published 2026-10-02 from
  `eggstack/eggpack@8d661e4`, checksum `2a08f24b…b629`, whose `src/lib.rs` is
  byte-identical to the previously pinned `678bbf04` source), and removes
  `publish = false`. It carries the M003a caller-bound destination seam
  (`materialize_artifact_set_with_destinations`,
  `core_plan_for_archive_with_destinations`, `ManifestProjection::default_destinations`)
  alongside the M002 archive extraction handoff. Package metadata gains
  `homepage`, `documentation`, `authors`, `keywords`, and `categories`. The
  adapter still selects no release, verifies no authenticity, and authorizes
  no filesystem destination. Published 2026-10-02.

> **Published-content caveat for `eggup-acquisition 0.1.2`.** That version was
> published from `02a1d32` on 2026-10-02 and therefore does **not** contain the
> two workspace bug-audit fixes that landed afterwards in `0b8cb98` (2026-10-04):
> the owned `.part` file leak when permission hardening fails, and the
> `ComposedTransport` fallback receiving the caller's full budget instead of the
> remaining one. The second one is reachable today by any consumer that composes
> two transports from the registry, so those consumers should either pin a
> later `eggup-acquisition` or set a total deadline that tolerates the doubled
> composed window until the fix is republished. `eggup-curl 0.1.2` keeps a
> caret-compatible `eggup-acquisition` requirement, so it inherits an
> `eggup-acquisition 0.1.3` automatically once one is published.
>
> **Status update (2026-10-06):** `eggup-acquisition 0.1.3` is published, so
> this caveat is **resolved for every `^0.1.0` consumer** (`eggup-curl`,
> `eggup-eggfetch`) automatically on their next resolve. It **remains open for
> `eggup-eggpack 0.1.2`**, which exact-pins `=0.1.2` and therefore still composes
> against the defective seam until `eggup-eggpack 0.1.3` is published.

- `eggup-curl 0.1.2` (first crates.io publication): the lightweight
  external-`curl` acquisition adapter, promoted as a package only. It fetches
  exact caller-selected URLs by invoking a caller-supplied `curl` executable
  with no shell, no internal `sudo`, and no release/version/mirror policy, so a
  curl-only binary carries no embedded HTTP/TLS stack. Carries the Acquisition
  M005-M008 qualified behavior: explicit connect/total ceilings plus an
  independent parent wall deadline serialized truthfully at sub-second
  precision, exact 404 classified as `FetchOutcome::NotFound`, bounded streaming
  into Eggup-owned staging, cancellation/timeout child kill and reap,
  race-safe no-clobber promotion, credential-bearing diagnostic redaction, and
  explicit redirect/protocol/proxy policy. Package metadata is behavior-neutral
  and the existing `eggup-acquisition` requirement stays caret-compatible. The
  adapter selects no release, verifies no authenticity, and authorizes no
  destination. No Windows live-loopback curl result is claimed: hosted Windows
  runners refused the historical spawned-curl loopback case, so the Windows lane
  carries the portable adapter, process, and error-path fixtures only. Published
  2026-10-04.

- `eggup-service 0.1.2` (first publication of this version): the
  manager-neutral service lifecycle crate, promoted as a package only. It carries
  the already-qualified M001-M008 substrate — systemd/launchd/cron/Windows SCM
  adapters, transaction/lifecycle composition with `KeepInstalled | RollBack`,
  the M006 managed-running/managed-stopped/direct-running/stopped/foreign-preserved
  disposition and revalidation semantics, UTF-8-safe bounded diagnostics, and
  deterministic deadline proof — plus the post-closure bug-audit fixes: a
  panicking service transition is now recorded under the phase that actually
  executes instead of as a post-install check failure, an unobservable target
  before a destructive `stop` is `Unknown` and fails closed, and the
  `SystemExecutor::run` stdin write is bounded by the caller's deadline. No
  service runtime behavior changed for the publication. The
  `eggup-core = ^0.1.0` requirement is unchanged and resolves from the registry.
  No migration is required for consumers of `0.1.1`. Published 2026-10-05.

## 0.1.3 — 2026-10-06, in progress

The 0.1.3 source version is being published to crates.io crate-by-crate, because
`version.workspace = true` forces the local tree to claim `0.1.3` for all seven
published crates while each crate is published only when its own milestone
closes. Every entry below states which crate is actually on the registry; no
crate is implied to be published by the mere presence of this section.

Integrity remains SHA-256 checksum evidence only; no authenticity or signature
claims are made by any of these crates.

- `eggup-acquisition 0.1.3` (published 2026-10-06, Acquisition Transport M010):
  carries the two workspace bug-audit fixes from `0b8cb98` that are absent from
  the published `0.1.2` source. A `ComposedTransport` fallback previously
  received the caller's full `FetchLimits` instead of the remaining budget, so
  one composed fetch could run for roughly twice the documented total deadline;
  it now receives only the remaining budget with `connect_timeout` clamped to
  preserve `connect_timeout <= total_timeout`, and an already-exhausted budget
  reports `Timeout` without attempting the secondary transport. Separately, a
  part file could be left on disk when its permission hardening failed; both
  failure paths now drop the handle and remove the candidate. No public API
  change — `FetchLimits` field types, defaults, and validation are untouched,
  so this is semver-compatible within `0.1.x` and requires no migration. The
  crate keeps **zero** `[dependencies]`.

> **The `eggup-eggpack 0.1.2` caveat is still open at this point.** The
> published `eggup-eggpack 0.1.2` exact-pins `eggup-acquisition = "=0.1.2"`, so
> it does **not** receive the seam fixes above: adapter consumers that compose
> two transports must pin `eggup-acquisition 0.1.3` themselves, or tolerate the
> doubled composed window, until `eggup-eggpack 0.1.3` is published.
> `eggup-curl 0.1.2` and `eggup-eggfetch 0.1.2` keep caret-compatible
> requirements and inherit `eggup-acquisition 0.1.3` automatically on the next
> resolve.

- `eggup-eggfetch 0.1.3` (Acquisition Transport M011): carries the three
  workspace bug-audit fixes absent from the published `0.1.2`. A `Result` API
  could panic because the tokio runtime was built with `.expect()`, so a
  runtime-build failure (fd exhaustion, driver creation failure) aborted the
  process from inside a fallible fetch; runtime-build and thread-teardown
  failures now return `AcquisitionError::Unavailable`, which composition already
  knows how to handle. The runtime is now built once per calling thread rather
  than once per fetch: a `current_thread` runtime is `Send` but not `Sync`, so
  caching one on the transport would have made `EggfetchTransport` `!Sync` and
  broken callers sharing one transport across threads — a thread-local keeps
  the auto-traits intact. And `TooLarge` could report `limit: 0`, contradicting
  the documented meaning of that field; the mapper now takes a mandatory `u64`
  and every call site passes the bound it actually enforces. No public API
  change, so no consumer migration is required. `eggup-acquisition` is a caret
  requirement, so this package resolves the `0.1.3` seam above.

- `eggup-archive 0.1.3` (Archive Extraction M004): carries the workspace
  bug-audit fixes absent from the published `0.1.2`. `validate_output_name`
  rejected `COM1`..`COM9`/`LPT1`..`LPT9` but accepted `COM0`/`LPT0` and the
  reserved console names `CONIN$`, `CONOUT$`, `CLOCK$`, all of which Windows
  also maps to a device; all are now rejected as `InvalidPath`, while near
  misses such as `com10` and `console` stay valid. `with_empty_residue`
  recorded a residue path without verifying the private root was actually
  empty, so a partially failed cleanup was reported under the original error
  kind; the root is now verified and a root that still holds evidence is
  promoted to `CleanupFailed`, making `residue_path()`'s documented "the
  directory is empty" guarantee true. Hostile traversal names are now tested as
  real tar and zip *entries* on the untrusted path rather than only as
  caller-declared plan input. No API change, no new dependency.

- `eggup-core 0.1.3` (Core M012/M013): current-executable transaction parity
  (M010), proof-authorized stale-lock recovery (M011), the earlier Core audit
  fixes, and the M012 public-error compatibility correction. M010/M011 were
  never published, so this is their first appearance on the registry. The
  `Error` enum keeps exactly the seven variants `0.1.2` published, proven by a
  byte-identical external exhaustive-match fixture that compiles against both
  versions and fails against the pre-M012 tree; retained evidence moved to a
  new `#[non_exhaustive] RecoveryError`. The published `eggup-service 0.1.2`
  resolves and runs against this Core. `self-replace` remains Windows-only, so
  the Unix/macOS graph is still `sha2` only.

- `eggup-eggpack 0.1.3` (Eggpack M005): closes a security defect present in
  published `0.1.2`. `bind_archive_members` paired bound archive members to
  declared members **positionally**, never cross-checking a bound member's
  recorded `source_path`/`output_name` against the declaration it was about to
  represent. A mismatched or reordered `BoundExtraction` could therefore bind
  bytes under a different member identity whenever content digests happened to
  match. Each bound member is now checked against its declared counterpart and
  fails closed with `AdapterError::MapMismatch`. Identical-content members still
  cannot substitute for one another. The exact Eggup dependency set moves to
  `=0.1.3`, which also delivers the acquisition composed-budget fix to adapter
  consumers for the first time. The adapter still selects no release, verifies no
  authenticity, and authorizes no filesystem destination; caller-bound
  destination APIs are unchanged.

## Unreleased

The workspace-wide bug-audit fixes below are recorded here as well as in the
per-crate changelogs, because they are **not** on the registry. Published
baselines are `0.1.2` for all seven published crates. No publication milestone
authorizes these entries yet.

The two `0b8cb98` acquisition fixes that previously lived in this section are no
longer here: they were published as `eggup-acquisition 0.1.3` on 2026-10-06
(Acquisition Transport M010) and are recorded in the `0.1.3` section above.

- `eggup-core` gains current-executable transaction parity (Verified Update Core
  M010): `CurrentExecutable` and `InstallPlan::for_current_executable` let a
  program replace the executable it is running through the ordinary one-member
  transaction model, with `StagePlacement::InsideInstallationRoot` so no write
  authority above the executable's own directory is ever required. Exact image
  identity is canonicalized and re-proved under the lock immediately before the
  first live rename. New `CleanupDisposition::DeferredToProcessExit` reports
  truthfully that a kept-installed Windows self-update still holds a mapped old
  generation whose deletion is scheduled for process exit. `self-replace` is
  added as a **Windows-only** dependency; the Unix/macOS graph is unchanged
  (`sha2` only). `CleanupDisposition` is not `#[non_exhaustive]`, so this new
  variant is a source-visible addition to that one enum; no `eggstack` consumer
  references the type. Retained evidence is reported through a new
  `RecoveryError` rather than a new `Error` variant, so an exhaustive `match` on
  `Error` compiles unchanged (Verified Update Core M012).

- `eggup-core` gains proof-authorized stale-lock recovery (Verified Update Core
  M011): `LockObservation`, `StaleLockDecision`, `StaleLockVerifier`, and
  `MutationLock::acquire_with_recovery`. Core exposes a bounded observation of
  one exact record and displaces it only when the caller's verifier returns
  `ProvenStale`; PID liveness, age, and service state remain consumer policy.
  Claiming re-reads, renames into a same-directory Eggup-owned claim path, and
  re-verifies the claimed object before anything is created, so a replaced
  record is never deleted and a competing writer always wins. The default
  `MutationLock::acquire` remains fail-closed and never recovers.

- Eggpack Interop audit fix — **security, in a published crate**
  (`eggup-eggpack`, unpublished here): archive members were bound
  positionally, with no identity cross-check. A `BoundExtraction` from a
  different `ArchivePlan`, or reused after member reordering, could commit each
  member's bytes under another member's declared identity — digest-invisible
  when two declared members share identical content. Bound members are now
  cross-checked against the declared member they would bind to and a mismatch
  fails closed with `AdapterError::MapMismatch`. No API change. Consumers
  resolving `eggup-eggpack 0.1.2` from crates.io do **not** have this fix until
  a new version is published. See `crates/eggup-eggpack/CHANGELOG.md`.

- Core audit fixes (unpublished): bound-source staging dropped the executable
  bit for every `PermissionsIntent::Preserve` member; a fully successful
  rollback could be reported as `RecoveryRequired` with
  `rollback_verified: false`; injected fault failures are now classified
  structurally rather than by substring, without adding a public `Error` variant,
  so the published `Error` surface stays compatible (Verified Update Core M012).
  See `crates/eggup-core/CHANGELOG.md`.

- Archive audit fixes (unpublished): Windows device-name validation accepted
  `COM0`/`LPT0` and the reserved console names `CONIN$`, `CONOUT$`, `CLOCK$`;
  `residue_path()` could report a non-empty root as cleaned. See
  `crates/eggup-archive/CHANGELOG.md`.

- Eggfetch audit fixes (unpublished): `block_on` could panic via `.expect()`
  when a tokio runtime failed to build, and `TooLarge` could report
  `limit: 0`. Both now surface as typed errors. See
  `crates/eggup-eggfetch/CHANGELOG.md`.

- Acquisition audit fixes (unpublished; **these two are M010's publication
  payload**, confirmed by diffing the published `0.1.2` source against the
  working tree): a fallback adapter restarted the caller's total time budget, so
  one composed fetch could run for roughly twice the documented `total_timeout`
  — the fallback now receives only the remaining budget; and a part file was
  left on disk when securing it to `0600` failed, which now drops the handle
  and removes the candidate. Note that `FetchLimits` validation at every
  transport boundary, the retirement of `Option` from `max_artifact_bytes`, and
  UTF-8-safe bounded diagnostics all shipped in the published `0.1.2` — a
  consumer resolving `eggup-acquisition 0.1.2` already has those.

- Eggpack Interop M002 (published in `eggup-eggpack 0.1.2` on 2026-10-02; see the 0.1.2 section above): `eggup-eggpack` gains the archive
  extraction handoff (`archive_format_for_name`, `validate_acquired_archive`,
  `archive_plan_for`, `core_plan_for_archive`, `bind_archive_members`) that
  translates `ManifestProjection::Archive` into a validated
  `eggup_archive::ArchivePlan` plus a core `InstallPlan` and `BoundSources`
  for `prepare_with_bound_sources`, with exact member size/digest facts and
  fail-closed format/count/permission mapping. New `AdapterError::
  UnsupportedArchiveFormat` variant (the error enum is `non_exhaustive`, so
  this is additive). The adapter performs no extraction, commit, or cleanup
  itself; orchestration and deferred cleanup stay caller-owned. Direct/bundle
  APIs are unchanged and `materialize_artifact_set` still returns
  `ArchiveExtractionRequired` for archive projections. New runtime
  dependencies are `eggup-archive =0.1.2` and `sha2 0.10.9`; `eggup-core`
  remains archive/Eggpack independent. The later M003a caller-bound
  destination seam and the M004 registry promotion shipped in the same
  `0.1.2` version; Eggsact's own Git-to-registry consumer migration is still
  separately authorized in `eggstack/eggsact` and is not claimed here.

- Acquisition M008 (published in `eggup-curl 0.1.2` on 2026-10-04; see the 0.1.2 section above): `eggup-curl` deadline
  arguments now serialize at microsecond precision with a `.` decimal
  separator, so sub-second connect/total ceilings (`100 ms -> "0.1"`,
  `250 ms -> "0.25"`, `1.5 s -> "1.5"`, `2 s -> "2"`) are passed to curl
  truthfully instead of being widened to whole seconds. Truncation to
  whole microseconds never widens the input. Sub-microsecond positive
  durations are rejected at validation rather than silently extended.
  Connect-phase timeout attribution uses only the parent scheduling
  tolerance (`POLL_INTERVAL + 5 ms`); the previous one-second
  truthfulness allowance is removed. No public `FetchLimits` API or
  default values changed; no fallback, release, service, or core policy
  changed. No consumer migration performed.

- Service M007 (unpublished corrective): bounded service errors and manager
  output excerpts now truncate at UTF-8 character boundaries within their
  existing 512-byte and 256-byte limits. Long permission errors retain their
  bounded remediation hint. No lifecycle behavior or consumer migration
  changed; no publication performed.

- Acquisition M007: the `eggup-acquisition` half shipped in `eggup-acquisition
  0.1.2` on 2026-10-02; the `eggup-curl` half shipped in `eggup-curl 0.1.2` on
  2026-10-04 (see the 0.1.2 section above). Diagnostics truncate only at
  UTF-8 boundaries; `FetchLimits::max_artifact_bytes` is a mandatory positive
  finite `u64` (migration: `Some(n)` → `n`, remove `None`); curl streams body
  bytes into Eggup's retained exclusive file handle and captures bounded HTTP
  status separately. No fallback, release, service, or core policy changed.
  No consumer migration performed.

- Acquisition M005 (published: `eggup-curl` 0.1.2 on 2026-10-04 and
  `eggup-acquisition` 0.1.2 on 2026-10-02; see the 0.1.2 section above): the
  `eggup-curl` external-curl adapter plus
  `AcquisitionError::Unavailable` and `eggup-acquisition::ComposedTransport`
  with `CompositionPolicy::{UnavailableOnly (default), UnavailableOrTransport}`.
  Curl-only binaries avoid an embedded HTTP/TLS stack; Eggfetch-only binaries
  avoid curl; dual binaries compose both for the same exact URL. Exact 404
  remains terminal `NotFound`; default fallback occurs only on unavailability.
  No `eggup-core` change; no Gregg modification or dependency. No consumer
  migration performed.

- Service M006 (unpublished): product-neutral `UpdateRuntimeDisposition`
  (`ManagedRunning`, `ManagedStopped`, `DirectRunning`, `Stopped`,
  `ForeignPreserved`) with pure `plan_unix`/`plan_windows` reference planners,
  caller-owned `DirectRuntimeControl` seam, and `commit_with_disposition`
  orchestration reusing Core M007 `KeepInstalled | RollBack` / `RecoveryRequired`
  semantics. Artifact authority and manager mutation authority are separate;
  `ForeignPreserved` performs zero manager mutation and `DirectRunning` uses
  only exact direct control. Preparation precedes quiescence and authority is
  revalidated immediately before mutation. Existing `commit_with_lifecycle`
  behavior is preserved. No Gregg modification, migration, or dependency. No
  publication or consumer migration performed.

## 0.1.1

- `eggup-core`: added ADR-0002 post-commit policy via
  `ValidatedTransaction::commit_with_post_commit`. The transaction retains
  its lock and backup through one caller check, records bounded check-failure
  evidence, and either keeps the complete new generation or restores the old
  one. Existing immediate `commit()` remains unchanged. Callback panics are
  converted into failed-check evidence; process-crash durability is not
  claimed.

- M004 Windows SCM adapter for `eggup-service`: safe typed SCM query,
  create/refresh, start/stop/restart, and delete; exact parsed executable and
  argument ownership; fail-closed handling of ambiguous command lines; shared
  transition deadlines; and truthful marked-for-delete completion. SCM
  settings remain caller-owned, unmanaged registration fields are preserved,
  and no shell, `sc.exe`, ambient PATH, or automatic elevation is used.
  `windows-service` is target-specific; custom passwords, descriptions,
  recovery actions, and service entrypoints remain out of scope. No consumer
  migration or publication performed.

- M003 distribution conformance library: derive stable release asset/sidecar
  inventories; validate caller-supplied release and archive-member names with
  explicit allow/exact extras policy; compare typed runtime/bootstrap target
  mappings against schema v1. Inventories and reports are bounded and
  deterministic. No network client, archive reader, source parser, CLI,
  generator, or runtime dependency added.

- M003 service corrective: launchd restart now propagates incomplete stop/start
  results; systemd/launchd transitions share one end-to-end monotonic deadline;
  production manager commands use trusted absolute paths and a cleared,
  documented environment; config identity is reconciled against exact argv;
  manager mutation exit statuses are checked. No consumer migration or
  publication performed.

- M002 distribution schema corrective: TOML v1 templates now use a strict
  lexer that rejects malformed/stray/nested braces and unsupported syntax at
  parse time. Expanded release assets and checksum sidecars share a unique
  namespace, and expanded install names are unique; exact and ASCII-case-only
  collisions fail with a bounded typed error. No publication or consumer
  migration performed.

- M004 acquisition corrective: `FetchLimits::validate` is shared by its
  constructor and re-run at every fixture/Eggfetch transport boundary while
  public fields remain source-compatible for 0.1.x. Invalid direct literals
  fail before transport work. No-clobber hard-link creation is now the
  promotion commit point; redundant temp-link cleanup is best-effort and
  cannot return ordinary failure after a complete destination exists. No
  publication performed.

- M002 Unix service adapters (`eggup-service` capability): `SystemdManager`,
  `LaunchdManager`, and `CronManager` on the closed M001 contract with
  bounded literal-argv command execution (no shell, kill/reap, output
  bounds, fake executor), exact ownership (ambiguous outputs yield
  `Unknown`, foreign never mutated), atomic definition writes, cron
  byte-preservation, explicit privilege/scope handling (no sudo, no EUID
  guessing), and host fact vs candidate-policy separation. No consumer
  migrated; manager-specific types are new 0.1.x API. No publication
  performed.

- M001 distribution schema (new `eggup-dist` release-time crate, TOML v1):
  versioned `DistributionContract` with direct/bundle/archive asset forms,
  unambiguous target/alias mappings, small `{product}/{version}/{target}/
  {alias}(/{asset} for sidecars)` template expansion, explicit SHA-256
  sidecar naming (integrity only), and traversal-free archive member
  validation. Fixtures prove simple, CodeGG-like bundle, and Egress-like
  archive layouts. No runtime crate depends on `eggup-dist`; no network,
  extraction, or installer generation. Serialization choice: TOML (readable
  in review, clean for tagged `direct|bundle|archive` via explicit `kind`
  tables). No publication performed.

- M003 acquisition corrective (patch, no signature break): authoritative
  `min(request, adapter ceiling)` effective timeouts via Eggfetch
  request-level overrides; `FetchLimits::new` and adapter construction now
  require `connect <= total`; exclusive owner-private (`0600` Unix)
  temp siblings with bounded collision retry and no symlink following;
  race-safe no-clobber artifact promotion (existing/raced `dest` fails
  explicitly, never overwritten); owned-temp-only cleanup; category-only
  transport diagnostics (no upstream/proxy error echo, userinfo/query/
  fragment redaction). Eggsact (10s/120s) unchanged; stegoeggo total
  tightens 120s -> 60s (request wins). Version decision: next release is
  workspace lockstep 0.1.1 patch (acquisition + eggfetch fix, no API break);
  publish order seam-then-adapter when directed. No publication performed.

- Added the Rust 1.89 `eggup-core` workspace foundation and deterministic test
  fixture support.
- Documented the transport-neutral ownership boundary. No production updater
  behavior is included yet.
- Added validated private staging, synchronous commit/rollback receipts, native
  SHA-256 integrity checks, and bounded candidate validation phases.
- M005 corrective (breaking pre-1.0): canonical `Absent | Owned | Foreign |
  Unknown` ownership with consumer verifier and absent-create policy; no
  automatic destination-parent creation; staged-digest revalidation under lock;
  removed unenforced authenticity API; renamed cleanup disposition to
  `CleanupDisposition`; structured failure phase/category/member reports with
  real recovery paths; owner-private stage/backup/lock permissions with
  collision-resistant names and ownership-checked cleanup; truthful
  fail-closed lock inspection with no automatic stale removal; checksum-only
  docs.
- M006 qualification: `eggup-core` package metadata (keywords, categories,
  homepage, docs URL, exclude rules); `#[non_exhaustive]` on extensible enums;
  five runnable examples (one-member, multi-member, custom validator,
  ownership verifier, receipt interpretation); `cargo package` and `publish
  --dry-run` green; `eggup-core` crates.io name verified available;
  dependency surface remains `sha2` only; MSRV 1.89 and platform lanes
  documented. No publication performed.
