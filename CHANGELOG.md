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

The entries below are **unpublished** work. The current published baselines are
`0.1.3` for `eggup-core`, `eggup-acquisition`, `eggup-eggfetch`,
`eggup-archive`, `eggup-eggpack` and `eggup-service`; `0.1.2` for
`eggup-curl`; `eggup-transport-footprint` is `publish = false`. Nothing here
has been published: each entry names the published version that still carries
the defect it corrects, and republication is a separate milestone.

The whole 2026-10-23 corrective batch is hosted-qualified. CI run
`40008039843` is green on all five lanes (`stable`, `msrv`, `service-systemd`,
`macos`, `windows-check`) for the first eleven milestones, and CI run
`40062299630` is green on all five lanes for the five that followed it
(`eggup-curl` User-Agent parity, `eggup-core` bounded stdin, `eggup-service`
bounded-executor consolidation, the `eggup-transport-footprint` isolated
configuration coverage, and the architecture-anchor repair). The first of those
two runs, `40061134915`, was red in four lanes and is the evidence behind the
packaged-crate deferral recorded below.

Work published by the 0.1.3 train is recorded in the `0.1.3` section above,
not here. Per-crate release notes live in `crates/*/CHANGELOG.md`; this
section aggregates them, it does not replace them.

- `eggup-core` corrects the bounded candidate deadline (Verified Update Core
  M014): `run_bounded` evaluated its documented wall-clock deadline only while
  the direct child was still alive, so a descendant that inherited the output
  pipe and held it open pinned the call indefinitely, and the timeout path left
  both reader threads unjoined. The deadline is now an unconditional maximum
  bound on the whole bounded execution and `run_bounded` always returns: a
  still-running direct child is killed and reaped exactly as before, an
  already-exited child is not killed again, and the call no longer waits for
  pipe EOF. The descendant case is bounded and fail-closed — `timed_out` is
  set, `output_limited` is clear, `success()` is false, and the direct child's
  exit code stays observable through `exit_code`. Reader-thread lifecycle is
  explicit: a reader that delivered its result is joined, and a reader still
  blocked at the deadline is abandoned, because core has no `libc`, `nix`, or
  `windows-sys` dependency and `unsafe` is denied. The public surface is
  unchanged: `Error` keeps exactly the seven published `0.1.2` variants, the
  output bound stays per stream, the ownership model, transaction state
  machine, lock, receipts, and failure taxonomy are untouched, and the
  dependency surface remains `sha2` only. **Unpublished** — `eggup-core 0.1.3`
  on crates.io still carries the defect, and `eggup-eggpack`'s `=0.1.3` pin
  blocks propagation until a separate adapter republication. No consumer
  migration is required.

- `eggup-core` corrects the published failure taxonomy and documents its
  public failure surface (Verified Update Core M015): the unreachable
  `FailureCategory::RetainedEvidence` variant and its `retained-evidence`
  string are removed, so every remaining category describes a failure that can
  actually occur; the remaining seven `as_str()` strings are byte-identical to
  the published ones. Every public fallible item now carries an `# Errors`
  section stating which `Error` variants it can return, that a receipt — not
  an `Err` — is how a post-lock outcome is reported, that `Owned` is never
  inferred, and that a member without verified integrity evidence is rejected
  with no escape hatch. `clippy::missing_errors_doc` is enabled crate-locally so
  the existing gate keeps that documentation honest; the workspace lint table
  is unchanged and no other crate is affected. `Error` keeps exactly the seven
  published variants and `FailureCategory` stays `#[non_exhaustive]`, so
  downstream matchers keep compiling; the only downstream break is an explicit
  reference to the removed variant name. **Unpublished** — `eggup-core 0.1.3`
  still carries the defect, and `eggup-eggpack`'s `=0.1.3` pin blocks
  propagation. No consumer migration is required beyond deleting such a name
  reference.

- `eggup-archive` corrects its public cleanup contract and removes a duplicated
  member-write path (Archive Extraction M005): the four `cleanup()` entry
  points documented that they "remove" the extraction directory, but the shared
  implementation emptied the owned tree through the retained handle and then
  unconditionally returned `Err(ExtractionErrorKind::CleanupFailed)`, leaving
  the `Ok` path unreachable in a published crate. All four now share one
  truthful contract — `Ok(())` means the owned root held no entries at the
  final handle-relative listing, `Err(..)` with `kind() == CleanupFailed` means
  it could not be fully emptied, and `residue_path()` names the recorded root
  path — while the crate still never removes the root directory itself and
  never traverses the root pathname; the residual empty directory remains the
  caller's to remove. The tar.gz and zip member-write bodies are now one shared
  helper, behaviour-preserving. **Behaviour change:** the success path of
  `cleanup()` is now reachable, so a caller that wrote `cleanup().unwrap_err()`
  will panic and must match on the result instead; capture the residual path
  from `root()` before the consuming call. No new dependency and no public
  signature change. **Unpublished** — `eggup-archive 0.1.3` on crates.io still
  carries the defect, and `eggup-eggpack`'s `=0.1.3` archive pin blocks
  propagation. Integrity remains SHA-256 checksum evidence only.

- `eggup-service` collapses two divergent copies of the destructive
  commit/rollback engine into one (Service Lifecycle M012): `disposition.rs`
  and `lifecycle_update.rs` each carried a full orchestrator that had already
  drifted apart in safety-relevant ways — the disposition copy did not
  re-inspect after a transition, tolerated an ownership that was no longer
  `Owned`, swallowed a rollback-quiesce `inspect()` error and could stop an
  unproven target, and left its rollback restore unpanic-guarded, while the
  lifecycle copy could panic on an `unreachable!()` for a desired state outside
  `{Running, Stopped}`. There is now one engine (`commit_engine`) reached by
  both public orchestrators, and the stricter behaviour wins everywhere: a
  manager's `completed` flag is a self-report and is re-verified by
  observation, an ownership that is no longer `Owned` is always a typed
  failure, an unobservable or non-owned service is never stopped before a
  rollback, and an invalid desired state produces a typed bounded failure
  instead of a panic. The two former behavioural differences survive as named,
  separately testable policy inputs (`CommitEnginePolicy::post_transition_verification`,
  `CommitEnginePolicy::ownership_loss_tolerance`), both defaulting to the
  stricter behaviour, with the selection observable in the returned receipt.
  Three structurally identical ownership enums become one type:
  `eggup_service::Ownership` re-exports `eggup_core::Ownership` and
  `eggup_service::CronOwnership` is a public alias of it. The manager command
  output type is renamed `ManagerCommandOutput`, with the old spelling kept as
  a non-deprecated type alias, and manager-output diagnostic text is now
  bounded by one declared ceiling, `MAX_DIAGNOSTIC_BYTES` (512 bytes).
  **Unpublished** — `eggup-service 0.1.3` remains the published version and
  the `v0.1.3` tag is unmoved; republication is a separate milestone. No
  consumer migration is required: every affected spelling still resolves.

- `eggup-service` corrects its manager failure taxonomy, its timeout model,
  and its cron ownership rule (Service Lifecycle M013): `ServiceError::Manager`
  now carries a machine-readable `ManagerFailureKind` (`Spawn`, `Timeout`,
  `Permission`, `NotFound`, `Protocol`, `LocalFilesystem`) alongside the
  bounded detail and the platform OS code, and the crate no longer classifies
  its own failures by substring-matching its own error text — `permission_hint`
  and its call sites are gone. Every adapter now creates exactly one
  `OperationDeadline` per operation, so a single systemd `install` no longer
  spends several times its declared budget across chained operations.
  `CronManager` lists the crontab once per `inspect` instead of twice, states
  its ownership rule once instead of applying two disagreeing rules, and
  refuses inconsistent request material with `InvalidInput` before any listing
  or mutation. Manager-derived free text has one 512-byte ceiling instead of
  two competing ones. The public surface gains an `# Errors`/`# Panics`
  documentation sweep, with the Windows SCM surface excluded because it is
  reserved by Service M014. **Behaviour change:** a published-behaviour
  tightening applies to cron consumers whose block body omits the spec
  executable, which now fail closed instead of proceeding. **Not published by
  this milestone** — `eggup-service 0.1.3` remains the published version, the
  `v0.1.3` tag is unmoved, and no consumer migration is required until a
  republication milestone ships it; at that point a consumer matching
  `ServiceError::Manager` positionally must match
  `ServiceError::Manager { kind, detail, os_code }` and branch on `kind`.

- `eggup-curl` makes its two scheduler-sensitive deadline assertions
  deterministic (Acquisition Transport M012): the connect deadline stays
  sub-second at 250 ms and is still proven end-to-end through a real spawn,
  but the total budget moves from 750 ms to 30 s, so the assertion can no
  longer expire for scheduling reasons. The sub-second `--connect-timeout` /
  `--max-time` serialization is now proven at the layer where the rule lives,
  with no subprocess, no network, and no wall clock, and the prompt-kill test
  asserts the elapsed time against the short total timeout instead of an
  absolute ceiling. The change is test-only: no production code, no
  dependency, and no public surface changed. **Unpublished** — `eggup-curl
  0.1.2` remains the published version and still carries the exposure, and
  the `eggup-acquisition 0.1.3` seam it resolves against is unchanged. The
  milestone is conditionally closed: the hosted lane re-observed registry
  finding 1 in the spawn mode, so the finding stays open and narrowed.

- `eggup-acquisition` types its I/O failure surface, and `eggup-curl` stops
  reporting a pass it did not earn (Acquisition Transport M013):
  `AcquisitionError::Io(String)` becomes `Io { operation, source }` with a
  live `source()` chain, and the nine `eggup-curl` tests that returned early
  when their fixture was unavailable now fail loudly with an actionable
  message instead of asserting nothing. The UTF-8-safe truncation helper is
  consolidated inside each reachable dependency group, the curl adapter's
  bounded subprocess runner keeps its own boundary-constrained copy, and the
  adapter-capability-parity decision is recorded rather than silently
  resolved. **Unpublished and source-breaking** — `eggup-acquisition 0.1.3`
  still carries the flat `Io(String)` shape, and because `std::io::Error`
  implements neither `Clone` nor `PartialEq`/`Eq` the enum now derives only
  `Debug`; a consumer matching `AcquisitionError::Io(_)` must match the
  struct-shaped variant, which makes `0.2.0` the consequence.

- `eggup-curl` makes a failed spawn attributable (Acquisition Transport
  M015): `Command::spawn()`'s `io::Error` was discarded, so a missing
  executable, a non-executable file, and a transient resource exhaustion were
  indistinguishable in the diagnostic. The `Unavailable` detail now names the
  `ErrorKind` and the raw OS code, rendered only from the discriminant and the
  numeric code so no path, URL, or credential material can leak. The variant
  stays `Unavailable` and the `ComposedTransport` fallback contract is
  unchanged. **Unpublished** — `eggup-curl 0.1.2` remains the published
  version and still carries the cause-free detail, and the `eggup-acquisition
  0.1.3` seam is unchanged. This narrows registry finding 1 rather than
  closing it: the cause is attributable, not prevented.

- `eggup-service` executes its Windows SCM path and pins its public surface
  (Service Lifecycle M014): the five former `unreachable!()` sites in the SCM
  adapter become typed errors built from the crate's existing constructors, so
  the `1060`/`1072` mapping stays lossless and a service that disappears
  before an operation is a reported failure rather than a panic. A Windows
  fixture registers a disposable service through the crate's public API and
  drives install, inspect, ownership, start, stop, and uninstall against the
  genuine Service Control Manager, self-cleaning on every exit path and
  skippable only with a recorded reason. The public surface is pinned by a
  dependency-free compile-only signature guard plus a committed snapshot, and
  the per-variant dead-code gating replaces the two enum-wide suppressions.
  **Unpublished** — `eggup-service 0.1.3` still carries the panic sites, and
  the `v0.1.3` tag is unmoved; republication is a separate milestone.

- `eggup-eggpack` leaves exactly one file-digest implementation in the
  workspace (Eggpack Manifest Interoperability M006): the adapter delegates
  to `eggup_core::hash_file` instead of running its own inline SHA-256 loop,
  the symlink rejection and the exact-size check still run and are still
  ordered before the digest computation, and the direct `sha2` runtime
  dependency that existed only for that one call site is removed. The digest
  value, the error variant, and the error payload are unchanged, and no pin
  cascade is required because the pinned `eggup-core 0.1.3` already exports
  the capability. **Unpublished** — `eggup-eggpack 0.1.3` still carries the
  duplicate implementation and the redundant dependency edge.

- `eggup-curl` gains the caller-settable `User-Agent` control the other
  adapter already had (Acquisition Transport M014): the curl adapter now
  accepts an outbound `User-Agent`, validates it as an HTTP field value (no
  C0 or C1 control character, no leading or trailing space, a 512-byte
  bound), and emits it as a literal `--user-agent` argv element ahead of the
  `--` separator that isolates the URL. The control lives on `CurlTransport`,
  not on `CurlConfig`, because `CurlConfig`'s public fields would make any
  added field a source-breaking change for a consumer of the published
  crate; the default stays unset, so the observable wire behavior of the
  published adapter does not change. A rejected value is a terminal
  `AcquisitionError::InvalidInput` and never `Unavailable`, so a malformed
  value can never trigger a composed-transport fallback. The parity this
  closes is capability parity, not placement parity and not default-value
  parity. **Unpublished** — `eggup-curl 0.1.2` remains the published version
  and still carries the missing control, the `eggup-acquisition 0.1.3` seam
  it resolves against is unchanged, and the `v0.1.2` tag is unmoved;
  republication is a separate milestone.
- The two transport adapters record their publication coupling with the seam
  (Acquisition Transport M016): the seam's next release is
  `eggup-acquisition 0.2.0` and is source-breaking, because
  `AcquisitionError::Io(String)` became `Io { operation, source }` and the enum
  derives `Debug` only. Both adapters call seam capabilities —
  `__adapter_io` and `__adapter_bound_detail` — that exist only in the
  unpublished seam, and the `^0.1.0` requirement they declare does not accept
  `0.2.0`, so neither adapter can be packaged against the registry today. The
  adapters' changelogs had stated that a future republication would keep them
  at `0.1.3` and require no consumer migration; both halves are false once the
  seam goes to `0.2.0`, and they are now corrected. The seam's changelog
  declares its next release and records the widening each dependent needs —
  the two adapters to `0.2`, `eggup-eggpack` to `=0.2.0`, and
  `eggup-transport-footprint` to `0.2` — and a guard fails when a dependent
  neither accepts that version nor records the widening. **Unpublished** — no
  manifest, version, or publication changed; the widening belongs to the
  publication milestone, which needs explicit maintainer authorization. The
  published `eggup-acquisition 0.1.3` still carries the pre-coupling seam
  surface, and the published `eggup-curl 0.1.2` and `eggup-eggfetch 0.1.3`
  still carry the pre-coupling adapter state — the seam's `Io(String)` variant
  and the adapters' `eggup-acquisition = "0.1.0"` requirement — and the tags are
  unmoved. The
  `User-Agent` grammar in the curl adapter's public documentation is also
  corrected: interior spaces may repeat, as RFC 7230's `field-content`
  permits, which is what the implementation has always accepted.

- `eggup-core` bounds the stdin a child may receive (Verified Update Core
  M017): `CommandSpec::stdin_data` attaches a payload to the bounded
  execution and `MAX_STDIN_BYTES` (1 MiB, inclusive) bounds it, a payload
  over that bound is rejected with `Error::InvalidInput` before any child is
  spawned, and the payload is written on a dedicated thread whose failure —
  including the broken pipe from a child that exits without reading — kills
  and reaps the child and returns a typed `Error::Io` rather than hanging.
  The unconditional deadline bounds that writer exactly as it bounds the
  output readers, and the three `Error::Io` operation strings (`spawning
  bounded candidate`, `polling bounded candidate`, `writing bounded stdin`)
  are documented as a stable classification surface, so a downstream adapter
  can re-classify a core failure without matching on error text. `Error`
  keeps exactly the seven published variants, no dependency was added, and
  the dependency surface remains `sha2` only. **Unpublished** — `eggup-core
  0.1.3` still carries the null stdin and lacks `stdin_data` and
  `MAX_STDIN_BYTES`, and `eggup-eggpack`'s `=0.1.3` pin blocks propagation
  until a separate adapter republication. No consumer migration is required.

- `eggup-transport-footprint` gains CI coverage for the configurations it
  exists to represent (Transport Footprint M001): the crate's central claim
  — that a consumer can link exactly one transport stack — was verified by
  no machine, because every lane built only the dual configuration. The
  curl-only, eggfetch-only, and both-adapters configurations are now built by
  the `stable` and `macos` lanes and checked at the 1.89 MSRV floor, and a
  linkage-discipline guard rejects a sibling-adapter reference in every
  configuration, with a negative control that shows the guard discriminates.
  The recorded dependency counts 3 / 92 / 93 still reproduce exactly. The
  crate remains `publish = false` with no registry version; no published
  crate's dependency graph changed, and no consumer migration is required.

- `eggup-service` runs manager commands through `eggup_core::run_bounded`, so
  the workspace holds one bounded-process-runner implementation instead of two
  (Service Lifecycle M015). The crate's own safety policy — allowlisted
  absolute program paths, a cleared child environment with only the explicit
  user-scoped systemd session-bus variables, bounded stdin, and the timeout
  range check — is unchanged, as are the `ManagerFailureKind` taxonomy, the
  public API, and every manager's argv. The consolidation also fixes a
  defect the replaced runner had: it consumed the stdin writer's verdict with
  `try_recv`, discarding a successful write, and then read the now-empty
  channel as a failure. The affected path is `crontab -`, the crate's only
  production stdin caller. **Unpublished** — `eggup-service 0.1.3` still
  carries that defect and the previous runner until a republication; the
  `v0.1.3` tag is unmoved. The crate's packaged-source verification against
  the registry is deferred for the same reason and is restored by the
  `eggup-core` republication. No consumer migration is required.
- `eggup-service` states its own published-change record accurately (Service
  Lifecycle M016). The crate's `CHANGELOG.md` omitted the release note for the
  **eighth** behaviour difference of the consolidation — the one that is a
  **defect fix, not a cost**: the replaced runner consumed the stdin writer's
  verdict with `try_recv`, so a successful `crontab -` write was reported as a
  failure. The difference count was corrected from "seven" to eight and the
  missing release note added. The changelog's compatibility call-out named only
  one source-breaking change; the unflagged `LifecycleUpdateReceipt::engine_policy`
  field addition — which stops a consumer that constructs the receipt with a
  struct literal from compiling — is now stated in the entry that introduces the
  field. The service M015 closure's test inventory is corrected: **14** tests in
  `bounded_executor_consolidation.rs` (not 12), the three current argv guards
  replace the deleted `the_managers_argv_is_byte_identical`, the total is **16**
  new tests (not 15), and the hosted `windows-check` lane runs **ten**
  `cargo test` steps (not nine). **Unpublished** — documentation only; no
  production code, test code, manifest, lockfile, CI file, or architecture
  document changed, nothing published, and the `v0.1.3` tag is unmoved.
  `eggup-service 0.1.3` still carries the stdin-verdict defect until a
  republication.
- `eggup-core` gives the bounded-stdin deadline priority over the writer's
  verdict, and classifies that verdict by its own observation instant (Verified
  Update Core M018). The M017 entry promised that "a write failure detected
  after the deadline was reached does not retroactively turn the deadline
  outcome into an `Err`", but the implementation evaluated the stdin arm
  before the deadline arm, so the same input at the same timeout could be
  classified either way depending on the runner's poll phase. The deadline now
  has priority: the writer stamps its verdict with the instant it observed the
  write, a failure observed **before** the deadline is an `Err`, and one
  observed **at or after** it is the deadline outcome (`Ok`, `timed_out` set).
  An undelivered payload is never reported as a success, and the unconditional
  deadline guarantee is unchanged. Two overstated statements are corrected: the
  child's stdin pipe is only broken by the kill **unless a surviving descendant
  inherited it**, and the `FailureCategory` mapping has five producers in a
  shipped build, not six — `Injected` is test-only. The M015 closure record's
  **U1** is closed on the evidence of commit `ab90688`, which did add the four
  type-level rustdocs the finding said were missing. **Unpublished** — `Error`
  keeps exactly its seven published variants, no new dependency was added, and
  `eggup-core 0.1.3` still carries the old behaviour until a republication; the
  `v0.1.3` tag is unmoved.
- The documentation-truth gate now checks what it claims to check (Planning /
  Closure Hygiene C014). `scripts/check-docs-truth.sh` previously opened only
  `docs/`, `README.md`, `AGENTS.md`, the root `CHANGELOG.md`, and
  `plans/registry.md` — it never read a `Cargo.toml`, did not cover
  `architecture/*.md` at all, and skipped every registry table row with fewer
  than four cells, which hid the stale "Recently closed foundation" table
  entirely. It now reads the crate manifests and the per-crate changelogs,
  covers the architecture deep dives, and reads three-cell rows; each change is
  red-first proven against a documented false negative. The
  `architecture/transport-footprint.md` deep dive is corrected: it still said
  that no CI job builds the isolated single-adapter configurations, that the gap
  the crate exists to close "is still open", that the crate has "no subsystem
  roadmap", and that it has "zero `#[test]` functions" — all four contradicted
  the branch's own work. The dependency counts 3 / 92 / 93 are unchanged and
  still reproduce exactly.
- The architecture deep dives are re-anchored and their claims re-checked
  (Planning / Closure Hygiene C015). 179 stale `path:line` citations were
  re-derived across the five deep dives whose source crates were stable, and
  eleven substantive claims were corrected — among them a table row that still
  documented `permission_hint`, an API service M013 removed from the crate, a
  citation to a line past the end of the file, and three stale `=0.1.2` pins
  where the manifest says `=0.1.3`. **Unpublished** — documentation only; no
  production code, test code, manifest, lockfile, CI file, or script changed,
  and no tag moved.

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
