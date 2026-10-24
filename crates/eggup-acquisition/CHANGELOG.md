# Changelog

## Unreleased

Nothing in this section has been published. `0.1.3` is the published baseline on
crates.io, and no publication milestone authorizes these entries.

**Declared next release: `0.2.0`** — source-breaking. No publication, no tag
movement, and no version bump is authorized by this milestone; `0.1.3` remains
the published baseline on crates.io.

**Publication coupling with the seam's `0.2.0` release (Acquisition Transport
M016).** The seam's next release is `0.2.0`, and `0.2.0` does **not** satisfy
the `^0.1.0` requirement the two adapters declare today, nor the `=0.1.3` pin
the leaf adapter declares. Every in-workspace dependent's requirement must
therefore widen, in the same publication milestone, to:

- `eggup-curl`: `eggup-acquisition` = "0.2" (widened from "0.1.0").
- `eggup-eggfetch`: `eggup-acquisition` = "0.2" (widened from "0.1.0").
- `eggup-eggpack`: `eggup-acquisition` = "=0.2.0" (re-pinned from "=0.1.3").
- `eggup-transport-footprint`: `eggup-acquisition` = "0.2" (widened from "0.1.0").

**This entry records the coupling; it does not perform it.** No manifest is
changed by this entry, no version is bumped, and nothing is published. The
publication milestone must move the workspace version to `0.2.0` (the workspace
is version-lockstep, so all eight crates move with it), apply the widenings
above, re-pin `eggup-eggpack`'s other exact pins (`eggup-core`,
`eggup-archive`) to the new version, and publish in dependency order: the
seam, then the adapters, then the leaf. `eggup-transport-footprint` is
`publish = false` and is never published; its requirement widens only so the
workspace still resolves.

**Typed I/O failure surface (Acquisition Transport M013).** `AcquisitionError::Io`
no longer flattens the underlying `std::io::Error` into a string. The variant is
retyped in place as `Io { operation: &'static str, source: std::io::Error }`,
mirroring `eggup-core`'s published `Error::Io { operation, source }`, and
`impl std::error::Error for AcquisitionError` now overrides `source()` to return
the underlying `io::Error`. A consumer can match on `std::io::ErrorKind`
(`NotFound` vs `PermissionDenied` vs `AlreadyExists`) and walk the `source()`
chain instead of parsing `Display` text.

**Compatibility, stated precisely.** This is a **source-breaking** change:

- `AcquisitionError::Io(_)` no longer compiles against the struct-shaped
  variant; consumers must write `AcquisitionError::Io { .. }` (or
  `AcquisitionError::Io { operation, source }`).
- `AcquisitionError` now derives **only** `Debug`. `Clone`, `PartialEq`, and
  `Eq` are **removed**, because `std::io::Error` implements none of them — the
  same trade-off `eggup-core`'s published `Error` makes for the same reason.
  Consumers that cloned or compared whole `AcquisitionError` values must match
  on the variant and its fields instead.
- The **next** publication of this crate is therefore a source-breaking release
  (`0.2.0`, or the 0.2/1.0 boundary release), **not** a `0.1.4`. This milestone
  performs no publication, moves no tag, and bumps no published version.

Unchanged by this change: `Io` remains **terminal** under every
`CompositionPolicy` (it is never reclassified as `Unavailable` or `Transport`,
so it never triggers transport fallback); the rendered `Display` keeps the
seam's bounded form (at most 512 bytes, truncated only at a UTF-8 character
boundary); and the typed path carries **no** URL, credential, or proxy material
in the variant, in `source()`, or in `Display`.

**Public API addition (adapter-facing, source-compatible).** Two `#[doc(hidden)]`
adapter-facing entry points are added. They are not part of the consumer
contract and exist only so the verified adapters can build seam values:

- `__adapter_io(operation, source)` — builds the typed `Io` variant.
- `__adapter_bound_detail(detail)` — applies the seam's 512-byte
  UTF-8-safe bounding rule, so adapters no longer keep their own truncation
  copy.

**Truncation consolidation.** The two inlined UTF-8-safe truncation walks inside
`redact_url` (256-byte cap plus the trailing `…`) and `__scrub_upstream_text`
(512-byte cap, no ellipsis) are replaced by this crate's own
`truncate_utf8_bytes` primitive. Output is **byte-identical**; the redaction
tests that pin the 256/512-byte edges remain unchanged and green.

The crate still has **zero `[dependencies]`**, before and after.

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
