# eggup-curl

External-`curl` acquisition adapter for Eggup.

Implements `eggup-acquisition::AcquisitionTransport` without an embedded
HTTP/TLS stack. A caller supplies an explicit `curl` executable path (or opts
in to `PATH` discovery), and the adapter invokes it directly with no shell,
no `sudo`, and no release/version/mirror policy.

- Exact caller-selected URLs only; no discovery, no fallback inside the adapter.
- Explicit connect/total ceilings (`min(request, adapter)`), serialized as
  locale-independent decimal-second `--connect-timeout` / `--max-time` plus
  an independent parent wall deadline. Sub-second caller deadlines are
  honored truthfully (truncated to whole microseconds); sub-microsecond
  positive durations are rejected at validation rather than widened.
- Timeout classification labels the connect phase using only the parent
  scheduling tolerance (`POLL_INTERVAL + 5 ms`); there is no whole-second
  truthfulness allowance.
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
- An optional outbound `User-Agent` is caller-settable per adapter. The value is
  validated before it can reach the wire and is passed to `curl` as the literal
  argv element following `--user-agent`, before the `--` separator and the URL.
  By default the adapter sends **no** `User-Agent`, so `curl` applies its own
  default and the published wire behavior is unchanged.
- Diagnostics use category text plus the redacted URL; raw curl output,
  command lines, and proxy credentials are never embedded.

Transport fallback (`curl <-> Eggfetch` for the same exact URL) lives in
`eggup-acquisition::ComposedTransport` and is never release/source fallback.

## Outbound `User-Agent`

`CurlTransport::with_executable_and_user_agent(executable, config, user_agent)`
installs a `User-Agent` used for **every** request that adapter makes.
`CurlTransport::user_agent()` returns the accepted value, or `None` when none
was installed (the default).

The accepted grammar is:

```text
user-agent = 1*( VCHAR / SP / obs-text )
VCHAR      = %x21-7E          ; printable ASCII, excluding SP
SP         = %x20            ; permitted only between two non-SP characters
obs-text   = %xA0-10FFFF     ; any non-ASCII scalar value (C1 excluded)
```

A value is rejected with `AcquisitionError::InvalidInput` when it is empty;
when its UTF-8 length exceeds `MAX_USER_AGENT_BYTES` (512 bytes); when it
contains any C0 control character (including `CR`, `LF`, `NUL`, and `HTAB`),
`DEL` (`U+007F`), or any C1 control character (`U+0080`-`U+009F`); or when it
begins or ends with a space.

`InvalidInput` is terminal: unlike `Unavailable` it never makes a composed
transport fall back to another adapter, so a malformed value cannot silently
change which transport serves the request. The control characters are refused
because `curl` does not sanitize `--user-agent` — a value containing `CR LF`
would be written into the request head verbatim and inject a header.
