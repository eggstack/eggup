# Changelog

## 0.1.3 — 2026-10-06

Published to crates.io from the workspace `0.1.3` source version
(`plans/closure/acquisition-transport/010-status.md`). The crate has **zero
`[dependencies]`**, before and after. No public API change; `FetchLimits` field
types, defaults, and validation messages are untouched, so this is
semver-compatible within `0.1.x` and no migration is required.

These two bug-audit fixes were already implemented and green in the local tree
(commit `0b8cb98`, 2026-10-04) but were **absent from the published `0.1.2`**,
which was published from `02a1d32` on 2026-10-02.

- **Fixed: a fallback adapter restarted the caller's total time budget.**
  `ComposedTransport` passed the caller's original `FetchLimits` to the secondary
  adapter, so one composed fetch could run for roughly twice the caller's
  deadline — the connect phase is part of the total wall-clock budget.
  `FetchLimits::total_timeout` is documented as the *total* deadline, so the
  fallback now receives only the remaining budget, with `connect_timeout`
  clamped alongside it to preserve `connect_timeout <= total_timeout`. When the
  budget is already exhausted the fallback is not attempted at all and the
  result is a `Timeout`, rather than starting a doomed attempt.
- **Fixed: a part file was left on disk when securing it failed.** If reading the
  part's permissions or applying the `0600` mode failed, the candidate file was
  abandoned in the private temp directory instead of being removed. Both
  failure paths now drop the handle and remove the candidate before returning.
  The `0600` hardening itself is unchanged — this only fixes the cleanup on
  the error path.

`FetchLimits::validate` is **not** part of this section: it is already enforced
at every transport boundary in the published `0.1.2`, as is the retirement of
`Option` from `max_artifact_bytes` (see the `0.1.2` section below). An earlier
revision of this file filed both under `Unreleased` as an "M007 unpublished
corrective", which was misleading for anyone reading the published crate; that
wording is corrected here rather than left standing.

Transport fallback remains *not* release/source fallback: an exact `NotFound`
stays terminal for the requested URL. Diagnostic redaction is unchanged, and no
raw upstream, command-line, or credential material is ever embedded. The part
file is still created exclusively, owner-private (`0600` on Unix), never
symlink-followed, and promoted with race-safe no-clobber semantics.

## Unreleased

Nothing in this section has been published. `0.1.3` is the published baseline on
crates.io, and no publication milestone authorizes these entries.

## 0.1.2 — 2026-10-02

Published to crates.io (`plans/closure/eggpack-manifest-interoperability/004-status.md`).
First publication of this version, and the version that retired the unbounded
artifact cap. No Eggup or producer dependency; the crate has zero `[dependencies]`.

- Finite artifact bound: `FetchLimits::max_artifact_bytes` is a plain `u64`
  and `None` is no longer representable. `validate()` requires a strictly
  positive value, so every fetch is finitely bounded.
- UTF-8-safe bounded diagnostics (Acquisition M007): truncation
  (`truncate_utf8_bytes`, URL redaction) lands only on `char` boundaries within
  the existing 512-byte bound, so a truncated diagnostic is always valid UTF-8.
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
