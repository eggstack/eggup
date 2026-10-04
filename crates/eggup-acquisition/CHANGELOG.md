# Changelog

## Unreleased

Acquisition M007 (unpublished corrective): diagnostics now truncate only at
  UTF-8 character boundaries. `FetchLimits::max_artifact_bytes` is a mandatory
  positive finite `u64` (migration for direct field writers: `Some(n)` → `n`,
  and every `None` case must choose a positive bound). Composition, fallback,
  release, service, and core policy are unchanged.

Additional fixes from the workspace bug audit:

- **Fixed: a fallback adapter restarted the caller's total time budget.**
  `ComposedTransport` passed the caller's original `FetchLimits` to the secondary
  adapter, so one composed fetch could run for roughly twice the caller's
  deadline. `FetchLimits::total_timeout` is documented as the *total* deadline,
  so the fallback now receives only the remaining budget, with `connect_timeout`
  clamped alongside it to preserve `connect_timeout <= total_timeout`. When the
  budget is already exhausted the fallback is not attempted at all and the
  result is a `Timeout`, rather than starting a doomed attempt.

## 0.1.2 — 2026-10-02

Published to crates.io (`plans/closure/eggpack-manifest-interoperability/004-status.md`).
First publication of this version, and the version that retired the unbounded
artifact cap. No Eggup or producer dependency; the crate has zero `[dependencies]`.

- Finite artifact bound: `FetchLimits::max_artifact_bytes` is a plain `u64`
  and `None` is no longer representable. `validate()` requires a strictly
  positive value, so every fetch is finitely bounded.
- Adapter-only constructors, exclusive owner-private (`0600` on Unix) temp
  siblings, race-safe no-clobber promotion, and owned-temp-only cleanup.

## 0.1.1

- M004 acquisition corrective: `FetchLimits::validate` is shared by its
  constructor and re-run at every fixture/Eggfetch transport boundary while
  public fields remain source-compatible for 0.1.x. Invalid direct literals fail
  before transport work. No-clobber hard-link creation is the promotion commit
  point; redundant temp-link cleanup is best-effort and cannot return ordinary
  failure after a complete destination exists.
- M003 acquisition corrective (patch, no signature break): authoritative
  `min(request, adapter ceiling)` effective timeouts; `FetchLimits::new` and
  adapter construction require `connect <= total`; exclusive owner-private
  (`0600` Unix) temp siblings with bounded collision retry and no symlink
  following; race-safe no-clobber artifact promotion (an existing or raced
  `dest` fails explicitly and is never overwritten); owned-temp-only cleanup;
  category-only transport diagnostics (no upstream/proxy error echo, userinfo /
  query / fragment redaction). Version decision: workspace lockstep `0.1.1`
  patch, publish order seam-then-adapter.

## 0.1.0

First crates.io publication (`plans/closure/consumer-adoption/001-status.md`):
the transport-neutral acquisition seam and its deterministic
`FixtureTransport`. `NotFound` is data; TLS/proxy/timeout/5xx/malformed
responses are hard failures that never trigger fallback inside a transport.
