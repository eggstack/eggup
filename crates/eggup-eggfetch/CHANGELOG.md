# Changelog

## 0.1.3 — 2026-10-06

Published to crates.io (`plans/closure/acquisition-transport/011-status.md`).
No public API change, no transport-policy change, and no new runtime or API
feature. Composition and dual-transport behavior live in `eggup-acquisition`
(`ComposedTransport`, `CompositionPolicy`) and `eggup-curl`; see those
changelogs. `EggfetchTransport` remains shareable across threads.

`eggup-acquisition` is a caret requirement, so this package resolves the
published `eggup-acquisition 0.1.3` and inherits its composed-budget fix with
no change on the consumer side.

No consumer migration is required: no public API changed, and the
previously-panicking `block_on` path now returns
`AcquisitionError::Unavailable`, which is the error variant composition
already had to handle.

Fixes from the workspace bug audit; no API change. Composition and
dual-transport behavior live in `eggup-acquisition` (`ComposedTransport`,
`CompositionPolicy`) and `eggup-curl`; see those changelogs.

- **Fixed: a `Result` API could panic.** `block_on` built the tokio runtime with
  `.expect()`, so a runtime-build failure (fd exhaustion, driver creation
  failure) aborted the process from inside a fallible fetch. Runtime-build and
  thread-teardown failures are now returned as `AcquisitionError::Unavailable`,
  which also lets `ComposedTransport` fall back safely.
- **The runtime is now built once per calling thread instead of once per fetch.**
  A `current_thread` runtime is `Send` but not `Sync`, so caching one on the
  transport would have made `EggfetchTransport` `!Sync` and broken callers that
  share one transport across threads. A thread-local keeps the type's auto-traits
  intact while removing the per-fetch reactor and driver registration cost.
- **Fixed: `TooLarge` could report a limit of 0.** The artifact path passed no
  bound to the error mapper, so an eggfetch-side decoded-body limit produced
  `TooLarge { limit: 0 }` — a documented field meaning "the bound that was
  exceeded". The mapper now takes a mandatory `u64` and every call site passes
  the bound it actually enforces.

## Unreleased

Nothing in this section has been published. `0.1.3` is the published baseline on
crates.io, and no publication milestone authorizes these entries.

**Typed I/O failure surface and internal consolidation (Acquisition Transport
M013).**

- The adapter's filesystem failures are now reported through the seam's typed
  I/O variant, `AcquisitionError::Io { operation, source }`, so the underlying
  `std::io::Error` survives: a consumer can match on `std::io::ErrorKind` and
  walk the `source()` chain instead of parsing `Display` text. The `Io`
  variant stays **terminal** under every composition policy, and the typed path
  carries no URL, credential, or proxy material.
- The adapter's private truncation copy is gone. Its `bound` helper now
  delegates to the seam's shared 512-byte UTF-8-safe bounding rule
  (`eggup_acquisition::__adapter_bound_detail`).
- A vacuity regression guard (pattern guard, ignore-reason guard, and behavior
  witness) was added. It reads the crate's own sources at compile time, so it
  needs no filesystem discovery, no network, and no wall clock.

**Compatibility, stated precisely.** This crate's **public API is unchanged**:
no public signature, field, or constructor was added, removed, or retyped, and
its observable behavior — status classification, streaming, timeout handling,
redaction, and the `User-Agent` it already exposes through
`EggfetchConfig::user_agent` — is unchanged. The only consumer-visible
consequence is the source break in `eggup-acquisition`'s `AcquisitionError::Io`
variant, which this crate now builds through the seam's typed constructor
(`eggup-acquisition` M013). **No consumer migration is required in this
crate**, and no publication or version bump is authorized by this milestone.

The adapter still exposes a caller-settable `User-Agent` while `eggup-curl`
exposes none; that asymmetry is a known, documented difference in the *adapter
configuration surface*, not in the seam contract, and it is owned by the named
follow-up milestone "Acquisition M014 — adapter capability parity
(`User-Agent` and the `CurlConfig` public-field API boundary)".

## 0.1.2 — 2026-10-02

Published to crates.io (`plans/closure/eggpack-manifest-interoperability/004-status.md`).
This publication was required for registry coherence, not just for the feature:
published `eggup-eggfetch 0.1.1` declares `eggup-acquisition ^0.1.0` but its
source uses `Option<u64>`, so a fresh resolve after `eggup-acquisition 0.1.2`
published would otherwise select an incompatible pair. The `E0308` mismatch was
proven by a compile failure before this version was cut.

- Migrated to the `0.1.2` finite acquisition seam: `max_artifact_bytes: u64`
  and a UTF-8-safe `bound()`.
- `Unavailable` / composition API qualified against the `0.1.2` acquisition seam.

## 0.1.1

- M004 acquisition corrective: `FetchLimits::validate` re-run at the transport
  boundary while public fields stay source-compatible for 0.1.x; no-clobber
  hard-link creation is the artifact promotion commit point, and redundant
  temp-link cleanup is best-effort and cannot fail an already-committed
  destination.
- M003 acquisition corrective (patch, no signature break): authoritative
  `min(request, adapter ceiling)` effective timeouts applied as real Eggfetch
  request-level overrides (never a post-hoc elapsed check); `FetchLimits::new`
  and `EggfetchTransport::strict` require `connect <= total`; exclusive
  owner-private (`0600` Unix) temp siblings with bounded collision retry;
  race-safe no-clobber promotion; owned-temp-only cleanup; category-only
  transport diagnostics with userinfo/query/fragment redaction. Eggsact
  (10 s/120 s) unchanged; stegoeggo's total tightens from the adapter-enforced
  120 s to the requested 60 s (a stricter request wins and is never extended).

## 0.1.0

First crates.io publication (`plans/closure/consumer-adoption/001-status.md`):
the native `eggfetch-core` HTTP adapter with explicit bounded network policy
(HTTP/1 only, rustls with native roots plus packaged WebPKI fallback, bounded
connect/total phases, bounded redirects with strict HTTPS→HTTP downgrade
rejection, explicit proxy decision, no automatic retry, no decompression) and no
release/fallback authority. 404 is typed as `NotFound` but never triggers
fallback inside the adapter.
