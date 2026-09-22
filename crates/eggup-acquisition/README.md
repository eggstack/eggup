# eggup-acquisition

Transport-neutral acquisition seam for Eggup consumers.

The crate defines the smallest contract that can stream an artifact into
Eggup-owned staging without selecting release, fallback, version,
destination, or service policy:

- `AcquisitionRequest`: an exact caller-selected URL (no discovery, no fallback).
- `FetchLimits`: caller-provided byte and time bounds.
- `FetchOutcome::{Success, NotFound}` vs hard `AcquisitionError`: 404-style
  absence is data; TLS/proxy/timeout/5xx/malformed responses are hard failures
  that never trigger fallback inside a transport.
- `AcquisitionTransport::fetch_metadata` (bounded small body) and
  `fetch_artifact` (streamed to a file).
- `FixtureTransport`: deterministic in-memory transport for correctness tests;
  real network policy lives in `eggup-eggfetch`.

Transports never execute downloaded content, never choose fallback, and redact
credential-bearing URL material in diagnostics.

Effective time bounds (M003 corrective):

```text
effective connect timeout = min(request connect, adapter connect ceiling)
effective total timeout   = min(request total, adapter total ceiling)
```

A stricter adapter may tighten a deadline but never extends one.
`FetchLimits::new` requires non-zero timeouts with `connect <= total`.
Both metadata and artifact operations use the same derivation; the total
covers headers plus body streaming. A caller deadline produces
`AcquisitionError::Timeout`, never `NotFound`/fallback.

Artifact staging (M003 corrective):

- Temporary siblings are exclusively created (`create_new`, bounded retry,
  never follows symlinks), owner-private `0600` on Unix, in the exact
  destination parent (same filesystem).
- Promotion requires `dest` to be absent and uses race-safe no-replace
  semantics (`hard_link` fails with `AlreadyExists` without overwriting).
  An existing or raced-in destination fails explicitly and is preserved.
- Cleanup removes only the owned temp; foreign files are preserved.
  No prefix-based scavenging.

Diagnostics (M003 corrective):

- `redact_url` strips userinfo, query, and fragment.
- Transport failures use structured category text plus the redacted URL.
  Raw upstream/proxy error display is never embedded, so URL credentials,
  query tokens, and proxy secrets cannot leak.
