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
