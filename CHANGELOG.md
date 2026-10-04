# Changelog

## 0.1.2 — 2026-09-28, completed 2026-10-04

The 0.1.2 source version was published to crates.io in three steps. On 2026-09-28
(Verified Update Core M009) `eggup-core 0.1.2` and `eggup-archive 0.1.2` were
published. On 2026-10-02 (Eggpack Manifest Interoperability M004) the
producer-schema-dependent half of the same 0.1.2 source version was published:
`eggup-acquisition 0.1.2`, `eggup-eggfetch 0.1.2`, and `eggup-eggpack 0.1.2`.
On 2026-10-04 (Acquisition Transport M009) `eggup-curl 0.1.2` was published.

**Complete crates.io 0.1.2 publication set:** `eggup-core`, `eggup-archive`,
`eggup-acquisition`, `eggup-eggfetch`, `eggup-eggpack`, `eggup-curl`.

`eggup-service` and `eggup-transport-footprint` share the 0.1.2 source version
and are still **not** published. `eggup-transport-footprint` is
`publish = false` by design and has no edge in the published adapter graph;
`eggup-service` remains outside every publication authorization to date.

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

## Unreleased

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
