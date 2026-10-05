# Changelog

## Unreleased

Nothing in this section has been published. `0.1.2` remains the published
baseline on crates.io, and no publication milestone authorizes these entries.
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
