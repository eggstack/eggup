# eggup-curl

External-`curl` acquisition adapter for Eggup.

Implements `eggup-acquisition::AcquisitionTransport` without an embedded
HTTP/TLS stack. A caller supplies an explicit `curl` executable path (or opts
in to `PATH` discovery), and the adapter invokes it directly with no shell,
no `sudo`, and no release/version/mirror policy.

- Exact caller-selected URLs only; no discovery, no fallback inside the adapter.
- Explicit connect/total ceilings (`min(request, adapter)`), passed as
  `--connect-timeout` / `--max-time` plus an independent parent wall deadline.
- Cancellation and timeout kill and reap the owned child before return.
- HTTP status is captured from the same transfer (`-w "%{stderr}%{http_code}"`); no
  second probe. Exact 404 becomes `FetchOutcome::NotFound` and never triggers
  fallback inside the transport.
- Missing executable / discovery failure / spawn failure becomes typed
  `AcquisitionError::Unavailable` for safe composition. TLS/5xx/timeout are
  ordinary hard failures.
- Response bodies stream from curl stdout into Eggup's retained exclusive
  output handle; status is drained separately from stderr. The pathname is
  never handed back to curl for reopening. Artifacts use a private temp sibling
  (`0600` Unix) and race-safe no-clobber promotion; partial output cleans only
  the owned temp.
- Redirect, protocol, and proxy policy are explicit. `~/.curlrc` is ignored
  via `--disable`. Disabled proxy uses a cleared environment plus
  `--noproxy "*"`.
- Diagnostics use category text plus the redacted URL; raw curl output,
  command lines, and proxy credentials are never embedded.

Transport fallback (`curl <-> Eggfetch` for the same exact URL) lives in
`eggup-acquisition::ComposedTransport` and is never release/source fallback.
