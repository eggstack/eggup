# Changelog

## 0.2.0 — 2026-10-23, in progress

The 0.2.0 source version is being published to crates.io as one lockstep train.
`version.workspace = true` makes every crate in the tree claim the same
version, and `eggup-acquisition`'s next release is source-breaking —
`AcquisitionError::Io(String)` became `Io { operation, source }`, and because
`std::io::Error` implements neither `Clone` nor `PartialEq`/`Eq` the enum now
derives `Debug` only — so the seam's version had to advance to `0.2.0` and the
lockstep workspace carried every crate with it.

The crates publish in dependency order, because `cargo package` resolves a
dependency from the registry rather than from the in-tree path dependency:
`eggup-core`, `eggup-archive`, `eggup-acquisition`, `eggup-curl`,
`eggup-eggfetch`, `eggup-service`, `eggup-eggpack`. All seven publish from one
shared source commit, so the `v0.2.0` tag points at that commit.

The published `0.1.x` versions are not replaced or withdrawn. `eggup-curl` is on
crates.io through `0.1.2` and the other six through `0.1.3`; `eggup-curl` never
had a `0.1.3` at all. The `v0.1.1`, `v0.1.2`, and `v0.1.3` tags are not moved.

Integrity remains SHA-256 checksum evidence only; no authenticity or signature
claims are made by any of these crates. Per-crate release notes live in
`crates/*/CHANGELOG.md`; this section aggregates them, it does not replace
them, and the milestone-by-milestone detail of this release is in the five
publication closure records under `plans/closure/`.

- `eggup-core 0.2.0` (published 2026-10-23, Verified Update Core M019): the
  `Error` enum keeps exactly the seven variants `0.1.3` published, so an
  exhaustive `match` on it compiles unchanged, and `FailureCategory` stays
  `#[non_exhaustive]`. Four corrections ship together. **Fixed:** `run_bounded`
  evaluated its documented wall-clock deadline only while the direct child was
  still alive, so a descendant that inherited the output pipe and held it open
  pinned the call indefinitely, and the timeout path left both reader threads
  unjoined; the deadline is now an unconditional maximum bound on the whole
  bounded execution (M014). **Fixed:** the unreachable
  `FailureCategory::RetainedEvidence` variant and its `retained-evidence`
  string are removed, so every remaining category describes a failure that can
  actually occur, and every public fallible item now carries an `# Errors`
  section (M015). **Added:** `CommandSpec::stdin_data` attaches a payload to the
  bounded execution and `MAX_STDIN_BYTES` (1 MiB, inclusive) bounds it; an
  over-bound payload is rejected with `Error::InvalidInput` before any child is
  spawned, and the payload is written on a dedicated thread whose failure —
  including the broken pipe from a child that exits without reading — kills and
  reaps the child and returns a typed `Error::Io` rather than hanging (M017).
  **Changed:** the deadline now has priority over the stdin writer's verdict,
  classified by the instant the writer observed the write: a failure observed
  before the deadline is an `Err`, one observed at or after it is the deadline
  outcome (`Ok`, `timed_out` set), and an undelivered payload is never reported
  as success (M018). **One consumer action:** `eggup-core 0.1.3` does publish
  `FailureCategory::RetainedEvidence`, so a downstream crate that named it
  explicitly — a match arm, an equality assertion, a constant `as_str()` call
  — stops compiling against `0.2.0`; deleting the name reference is the whole
  of the required work. The dependency surface remains `sha2` plus a
  **Windows-only** `self-replace`, so the Unix/macOS graph is unchanged.

- `eggup-archive 0.2.0` (published 2026-10-23, Archive Extraction M006): no
  public API change and no new dependency. The `cleanup()` success path is now
  reachable — the four entry points documented that they "remove" the
  extraction directory, but the shared implementation emptied the owned tree
  through the retained handle and then unconditionally returned
  `Err(ExtractionErrorKind::CleanupFailed)`, leaving the `Ok` path unreachable
  in a published crate. All four now share one truthful contract: `Ok(())`
  means the owned root held no entries at the final handle-relative listing,
  `Err(..)` with `kind() == CleanupFailed` means it could not be fully emptied,
  and `residue_path()` names the recorded root path. The crate still never
  removes the root directory itself and never traverses the root pathname; the
  residual empty directory remains the caller's to remove. **Behaviour change:**
  a caller that wrote `cleanup().unwrap_err()` will panic and must match on the
  result instead; capture the residual path from `root()` before the consuming
  call. The tar.gz and zip member-write bodies are now one shared helper,
  behaviour-preserving.

- `eggup-acquisition 0.2.0` (published 2026-10-23, Acquisition Transport M017):
  **source-breaking**, and in exactly two ways. `AcquisitionError::Io(String)`
  became `AcquisitionError::Io { operation, source }` with a live `source()`
  chain, so a consumer that matched `AcquisitionError::Io(_)` must match the
  struct-shaped variant; and the enum now derives `Debug` only, because
  `std::io::Error` implements neither `Clone` nor `PartialEq`/`Eq`, so a
  consumer that cloned or compared whole error values must match on the variant
  and its fields instead. The `Display` rendering stays bounded at 512 bytes
  and is truncated only at a UTF-8 character boundary, `Io` stays terminal under
  every `CompositionPolicy`, and the crate keeps **zero** `[dependencies]`.

- `eggup-curl 0.2.0` (published 2026-10-23, Acquisition Transport M017): no
  source-breaking change of its own, but it is **transitively source-breaking**
  because it depends on `eggup-acquisition ^0.2` and the seam's `0.2.0` is
  source-breaking — the break is inherited from the seam, not introduced here,
  so a consumer must make the same two changes a consumer of the seam makes.
  The crate's own public API is additive: the caller-settable `User-Agent`
  control that the other adapter already had is now present
  (`CurlTransport::with_executable_and_user_agent` and
  `CurlTransport::user_agent`), validated as an HTTP field value and emitted as
  a literal `--user-agent` argv element ahead of the `--` separator that
  isolates the URL; a rejected value is a terminal
  `AcquisitionError::InvalidInput` and never `Unavailable`, so a malformed
  value can never trigger a composed-transport fallback. `CurlConfig` is
  untouched, so the observable wire behaviour of the published adapter does not
  change. The transport semantics qualified under Acquisition M005-M008 were
  published at `0.1.2` and are unchanged in substance.

- `eggup-eggfetch 0.2.0` (published 2026-10-23, Acquisition Transport M017): no
  source-breaking change of its own, but it is **transitively source-breaking**
  through the same seam requirement. Its own public API is unchanged and
  `EggfetchTransport` remains shareable across threads. No new runtime or API
  feature; the M011-era behaviour — a runtime-build failure returning
  `AcquisitionError::Unavailable` instead of panicking, the current-thread
  runtime cached per calling thread without making the transport `!Sync`, and
  `TooLarge` reporting the enforced bound instead of `0` — was published at
  `0.1.3` and is unchanged in substance.

- `eggup-service 0.2.0` (published 2026-10-23, Service Lifecycle M017): the
  first publication that carries the fix for the stdin-verdict defect the
  published `0.1.3` still has — the crate's bounded executor consumed the
  stdin writer's verdict with `try_recv`, discarding a successful write and
  then reading the now-empty channel as a failure, so a consumer of `0.1.3`
  receives a typed `Spawn` failure where a successful `crontab -` install was
  intended. **Two source breaks.** `ServiceError::Manager(detail)` is now
  `ServiceError::Manager { kind, detail, os_code }` carrying a
  `ManagerFailureKind` (`Spawn | Timeout | Permission | NotFound | Protocol |
  LocalFilesystem`); a consumer must match the struct form and branch on
  `kind`, not on the message text. And `LifecycleUpdateReceipt` gained the
  public field `engine_policy` (`CommitEnginePolicy`); the struct has no
  `Default` impl, so a consumer that constructs it with a **struct literal**
  must add the field and stops compiling until it does, while a consumer that
  only reads the receipt's fields is unaffected. **Behaviour change:** the cron
  managed block body must embed the absolute executable path of the spec, so a
  consumer whose `desired_block` omits `spec.executable()` now receives
  `ServiceError::InvalidInput` from `inspect`, `install`, and `uninstall`.
  **Behaviour change, fail-closed:** every adapter now spends exactly one
  `OperationDeadline` per public operation, so an operation that previously
  overran its declared budget now fails with a `Timeout` kind instead of
  overrunning. The three structurally identical ownership enums became one type
  (`eggup_service::Ownership` re-exports `eggup_core::Ownership`,
  `eggup_service::CronOwnership` is a public alias), `CommandOutput` survives
  as a non-deprecated alias of `ManagerCommandOutput`, and manager-output
  diagnostics have one declared ceiling, `MAX_DIAGNOSTIC_BYTES` (512 bytes) —
  none of which requires a consumer change. The `eggup-core` requirement
  widened to `^0.2`, and that is load-bearing rather than cosmetic:
  `eggup-core 0.1.3` has neither `CommandSpec::stdin_data` nor core's
  unconditional execution deadline and cannot compile this release.

- `eggup-eggpack 0.2.0` (published 2026-10-23, Eggpack Manifest Interoperability
  M007): its three exact pins move from `=0.1.3` to `=0.2.0`, which is the
  mechanism by which the seam's `0.2.0` and this release's fixes reach adapter
  consumers at all. The adapter delegates to `eggup_core::hash_file` instead of
  running its own inline SHA-256 loop, so the workspace holds exactly one
  file-digest implementation; the symlink rejection and the exact-size check
  still run and are still ordered before the digest computation, and the direct
  `sha2` runtime dependency that existed only for that one call site is
  removed. The digest value, the error variant, and the error payload are
  unchanged. The adapter still stops at extraction-required evidence: it
  selects no release, verifies no authenticity, and authorizes no filesystem
  destination. `eggpack-manifest` is an external crate and is not republished
  here.

- `eggup-transport-footprint` remains `publish = false` and has no registry
  version; it is never published. Its three requirements widened to `0.2` only
  so the workspace still resolves. Its three isolated transport configurations
  are built by the `stable` and `macos` lanes and checked at the 1.89 MSRV
  floor by the `msrv` lane, with a source-level linkage-discipline guard that
  rejects a sibling-adapter reference in every configuration.

- Two repository-level milestones close with this release and are not part of
  any published crate. The documentation-truth gate now reads the crate
  manifests and the per-crate changelogs, covers the architecture deep dives,
  and reads three-cell registry rows, each change red-first proven against a
  documented false negative (Planning / Closure Hygiene C014). The architecture
  deep dives are re-anchored: 179 stale `path:line` citations were re-derived
  across the five deep dives whose source crates were stable, and eleven
  substantive claims were corrected (Planning / Closure Hygiene C015). The
  gate's published-version check was generalized in this release so that it is
  not pinned to a single version number and keeps policing the crates that are
  actually on the registry.


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

## 0.1.3 — 2026-10-06, completed 2026-10-09

The 0.1.3 source version was published to crates.io crate-by-crate, because
`version.workspace = true` forces the local tree to claim `0.1.3` for all seven
published crates while each crate is published only when its own milestone
closes. Every entry below states which crate is actually on the registry; no
crate is implied to be published by the mere presence of this section.

**`eggup-curl` was never published at `0.1.3`.** Its `0.1.3` publication was
not attempted, because the seam's next release had to be the source-breaking
`0.2.0` and the workspace is version-lockstep; the adapter's next release after
`0.1.2` is therefore `0.2.0`. The six crates below are all on crates.io at
`0.1.3`.

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

- `eggup-service 0.1.3` (Service M011; published 2026-10-09):
  contains the Service M010 source-compatible correction for stopping an
  exact-owned failed systemd unit only after identity, manager state, process,
  and cgroup quiescence evidence agree. Unknown state, changed identity,
  restart races, and a remaining cgroup task fail closed. Runtime source is
  `0bde3fefbda07019529ad7566c02e6e4ec141fd6`; the package source commit is
  `feb6ae5aea4c9b61c4051957f3662ca49d845f9e` and the registry checksum is
  `9f7f7ea854577158e66aa202709ab1c97a3aedcf00b06c1ab914d25b132124dc`.
  M011's registry-only post-publication and hosted systemd evidence is recorded
  in `plans/closure/service-lifecycle/011-status.md`.

## Unreleased

Nothing in this section has been published. The current published baselines
are `0.2.0` for all seven published crates; `eggup-transport-footprint` is
`publish = false` and has no registry version.

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
