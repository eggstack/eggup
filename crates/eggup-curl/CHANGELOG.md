# Changelog

## Unreleased

Nothing is pending beyond the published `0.1.2` release below.

## 0.1.2 — 2026-10-04

First crates.io publication of `eggup-curl`
(`plans/closure/acquisition-transport/009-status.md`). This is a package
promotion only: the transport semantics qualified under Acquisition M005-M008
are unchanged, and the crate was not re-architected for publication.

Integrity remains SHA-256 checksum evidence only; no authenticity or signature
claims. The adapter selects no release, authorizes no install destination, and
claims no Windows live-loopback evidence.

- **Fixed: an `https` request could follow a redirect down to `http`.**
  `CurlConfig::allowed_protocols` was forwarded as both `--proto` and
  `--proto-redir`, and the default list contains `http`. A `302 Location:
  http://…` was therefore followed in cleartext with no error and no
  diagnostic, and the cleartext body was promoted to the destination as an
  ordinary success. `--proto-redir` is now narrowed per request so an `https`
  URL never lists `http` as a permitted redirect target, matching the strict
  downgrade denial the native `eggup-eggfetch` adapter enforces. A cleartext
  request may still be redirected to either scheme, and a configuration that
  allows only `http` still emits a well-formed list.
- **Fixed: a mislabelled timeout phase.** For curl exit 28 the connect phase was
  inferred from elapsed time against the effective connect ceiling. When the
  effective connect and total ceilings are equal — as they are under a
  one-second caller limit, which the adapter's minimums never shrink — that test
  held for every exit 28, so a total-deadline timeout was reported as a connect
  timeout. Coincident ceilings now report the weaker, always-true `total` phase.

- Acquisition M005 (published in this version): the `eggup-curl` external-curl
  adapter plus `AcquisitionError::Unavailable` and
  `eggup-acquisition::ComposedTransport` with
  `CompositionPolicy::{UnavailableOnly (default), UnavailableOrTransport}`.
  Curl-only binaries avoid an embedded HTTP/TLS stack; Eggfetch-only binaries
  avoid curl; dual binaries compose both for the same exact URL. Exact 404
  remains terminal `NotFound`; default fallback occurs only on unavailability.
  Transport fallback is never release/source fallback. No `eggup-core` change;
  no Gregg modification or dependency; no consumer migration performed.

- Acquisition M007 (published in this version): diagnostics truncate only at
  UTF-8 boundaries; `FetchLimits::max_artifact_bytes` is a mandatory positive
  finite `u64` (migration: `Some(n)` → `n`, remove `None`); curl streams body
  bytes into Eggup's retained exclusive file handle and captures bounded HTTP
  status separately, so the temp pathname is never handed back to curl for
  reopening. No fallback, release, service, or core policy changed.

- Acquisition M008 (published in this version): deadline arguments serialize at
  microsecond precision with a `.` decimal separator, so sub-second
  connect/total ceilings (`100 ms -> "0.1"`, `250 ms -> "0.25"`, `1.5 s -> "1.5"`,
  `2 s -> "2"`) are passed to curl truthfully instead of being widened to whole
  seconds. Truncation to whole microseconds never widens the input, and
  sub-microsecond positive durations are rejected at validation rather than
  silently extended. Connect-phase timeout attribution uses only the parent
  scheduling tolerance (`POLL_INTERVAL + 5 ms`); the previous one-second
  truthfulness allowance is removed. No public `FetchLimits` API or default
  values changed.

- Platform note: Windows hosted runners refused the historical spawned-curl
  loopback case, so the Windows lane carries the portable adapter, process, and
  error-path fixtures only. No Windows live HTTP/curl result is claimed. Live
  local HTTP/curl evidence is carried by the Linux and macOS lanes.
