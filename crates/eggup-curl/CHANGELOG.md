# Changelog

## Unreleased

Nothing in this section has been published. `0.1.2` is the published baseline on
crates.io, and no publication milestone authorizes these entries.

**Spawn-failure diagnosability (Acquisition Transport M015).**

- **Fixed: a spawn failure reported no cause.** When `Command::spawn()` failed,
  the adapter discarded the returned `io::Error` and reported the same
  cause-free string every time: `curl spawn failed for {redacted}`. A missing
  executable (`ENOENT`), a non-executable file (`EACCES`), and a transient
  resource failure (`EAGAIN`/`ENOMEM`) were therefore **indistinguishable** in
  the only record a consumer has of the failure. A hosted CI run failed with
  exactly that message and could not be attributed, because the diagnostic
  carried no way to tell the causes apart. The detail now names the cause:
  `curl spawn failed for {redacted}: {kind}[ (os error {code})]`, where
  `{kind}` is the `std::io::ErrorKind` (`NotFound`, `PermissionDenied`,
  `WouldBlock`, …) and the ` (os error {code})` suffix is present whenever the
  error carries a raw OS code.
- **Unchanged: the classification.** A spawn failure is still
  `AcquisitionError::Unavailable`, and `Unavailable` still falls back under
  `CompositionPolicy::UnavailableOnly`. The cause is **evidence inside the
  detail**, never a change of variant: `Io` is terminal under every
  composition policy and is not used for a spawn failure.
- **Unchanged: the redaction.** The URL is still redacted exactly as before —
  no userinfo, no query string, no fragment, and the seam's 256-byte
  truncation. The added cause segment is derived **only** from the
  `ErrorKind` discriminant and the numeric OS code; the OS-provided message
  text is never rendered, so no path or credential material can enter the
  diagnostic through it. The detail stays bounded by the seam's shared
  512-byte UTF-8-safe rule.

**Compatibility, stated precisely.** This crate's **public API is unchanged**
and its observable behavior is unchanged: no public signature, field,
constructor, or error variant was added, removed, or retyped, and no wire
request changes. The `Unavailable` / `Transport` classification of every
failure is unchanged, and the fallback contract is unchanged. The only
consumer-visible difference is the **text** of the spawn-failure detail, which
now carries the OS cause after the established
`curl spawn failed for {redacted}` prefix. **No consumer migration is required
in this crate**, and no publication or version bump is authorized by this
milestone: the published `0.1.2` artifact keeps the cause-free detail until a
separate republication milestone ships this change.

**Adapter capability parity: a caller-settable `User-Agent` (Acquisition Transport
M014).**

- **Added: a caller-settable outbound `User-Agent`.**
  `CurlTransport::with_executable_and_user_agent(executable, config, user_agent)`
  installs a `User-Agent` used for **every** request that adapter makes, and
  `CurlTransport::user_agent()` returns the accepted value (`None` when none was
  installed). The value reaches `curl` as the literal argv element following
  `--user-agent`, positioned before the `--` separator and the URL. No shell is
  involved, so the value is never re-parsed or interpolated.
- **Added: validation before the wire.** The accepted grammar is
  `1*( VCHAR / SP / obs-text )` — `VCHAR` is `%x21-7E`, `SP` (`%x20`) is
  permitted only between two non-`SP` characters, and `obs-text` is any
  non-ASCII scalar value (`%xA0-10FFFF`, C1 excluded). A value is rejected with
  `AcquisitionError::InvalidInput` when it is empty, when its UTF-8 length
  exceeds `MAX_USER_AGENT_BYTES` (512 bytes), when it contains any C0 control
  character (including `CR`, `LF`, `NUL`, and `HTAB`), `DEL` (`U+007F`), or any
  C1 control character (`U+0080`-`U+009F`), or when it begins or ends with a
  space. A rejected value produces **no** argv. `InvalidInput` is terminal:
  unlike `Unavailable` it never makes a composed transport fall back to another
  adapter, so a malformed value cannot silently change which transport serves
  the request. The control characters are refused because `curl` does not
  sanitize `--user-agent` — a value containing `CR LF` would be written into the
  request head verbatim and inject an additional header.
- **Unchanged: the default wire behavior.**
  `CurlTransport::with_executable` installs no `User-Agent`; the adapter's argv
  contains no `--user-agent` element, so `curl` applies its own default. The
  bytes a consumer of the published `0.1.2` sends are unchanged.
- **Compatibility, stated precisely.** The change is **additive**. `CurlConfig`
  is untouched — its fields are public and it is not `#[non_exhaustive]`, so
  adding a field there would be source-breaking for every struct-literal
  construction — and `CurlTransport` gains only a **private** field, a new
  constructor, and a new accessor. No existing public signature was added,
  removed, or retyped, so **no consumer migration is required**, and a future
  republication would be a non-breaking `0.1.3`. **No publication or version
  bump is authorized by this milestone**: the published `0.1.2` artifact keeps
  its current shape and behavior.

**Test-strengthening and internal consolidation (Acquisition Transport M013).**

- **Fixed: nine tests could report a pass while asserting nothing.** Each of the
  nine `eggup-curl` tests that needs a system `curl` obtained its fixture
  through a helper that returned `None` and exited early when no `curl` was
  discoverable on `PATH`. On a host without `curl` they were reported as
  **passed** while asserting nothing. All nine now obtain their fixture through
  a helper that **fails loudly** with an actionable message naming the missing
  prerequisite and the remedy (install `curl`, or put a curl executable on
  `PATH`). The `#[cfg(not(windows))]` gate on the loopback-server tests is
  **unchanged** — it is a known, deliberate platform difference, now recorded
  in the test module — and the Windows lane still runs a smaller test set.
- **Split: the connect/total timeout test.** The total-phase branch stays an
  end-to-end assertion of `Timeout { phase: "total" }`. The connect-phase
  branch needs a host that blackholes the route to the probe address; most
  hosts and CI runners fail the connect immediately with `ENETUNREACH` /
  `EHOSTUNREACH`, which is a transport failure rather than a timeout. That
  branch is therefore **explicitly marked as ignored with a recorded reason**
  and is reported as *ignored*, never as *passed*. The connect/total **label**
  is proven deterministically, with no subprocess, no network, and no
  sub-second wall-clock budget, by a new unit proof at the `classify_curl_result`
  layer that injects `start`.
- **Added: a vacuity regression guard.** A pattern guard, an ignore-reason
  guard, and a behavior witness now make this class of defect impossible to
  reintroduce silently. The guards read the crate's own sources at compile
  time, so they need no filesystem discovery, no network, and no wall clock.
- **Internal: the local truncation copy is gone.** The adapter's private
  `bound` helper now delegates to the seam's shared 512-byte UTF-8-safe
  bounding rule (`eggup_acquisition::__adapter_bound_detail`) instead of
  keeping its own implementation.

**Compatibility, stated precisely.** This crate's **public API is unchanged**
and its observable behavior is unchanged: no public signature, field, or
constructor was added, removed, or retyped, and no wire request changes. The
`Unavailable` / `Transport` classification of every failure is unchanged, and
the adapter gains no `User-Agent` surface **in M013** (M014 later added one;
see the M014 entry above). The only consumer-visible
consequence is the source break in `eggup-acquisition`'s `AcquisitionError::Io`
variant, which this crate now builds through the seam's typed constructor
(`eggup-acquisition` M013). **No consumer migration is required in this
crate**, and no publication or version bump is authorized by this milestone.

**Test determinism hardening (Acquisition Transport M012).**

- **Fixed: a sub-second deadline test asserted the host's scheduler, not the
  transport.** `build_curl_args_passes_sub_second_deadlines_to_fake_curl` used
  one pair of sub-second `FetchLimits` values for two incompatible purposes at
  once: the values whose serialization it asserted, *and* the live wall-clock
  budget the parent enforced against the spawned child. The fake child ignores
  `--connect-timeout` and `--max-time`, so the parent deadline was the only
  enforcement and the test could fail by overrunning its own 750 ms
  `total_timeout` while trying to succeed — reported as
  `Timeout { phase: "total" }` against a correct transport. The connect
  deadline stays sub-second (250 ms) and is still proven end-to-end through a
  real spawn; the total budget is now generous, so the assertion no longer
  depends on host scheduling.
- **Fixed: an absolute sub-second `elapsed` ceiling.** `sub_second_total_timeout_kills_child_promptly`
  no longer asserts `elapsed < 750 ms`, a bound that charged the test for
  process spawn, kill, reap, and two reader-thread joins that the transport's
  deadline contract does not govern. It now proves promptness structurally:
  `elapsed >= total_timeout` (the deadline was actually reached — the fetch did
  not succeed early) and `elapsed < 2 s` (the child was killed rather than
  left to stall, against a 4000 ms server stall). The residue assertions
  (`!dest.exists()`, empty directory) are unchanged and remain the
  deterministic core of the test.
- **Added: deterministic sub-second serialization coverage.** A new
  platform-neutral proof, with no subprocess, no network, and no wall clock,
  asserts that a sub-second connect **and** total pair (250 ms / 750 ms)
  serializes to `--connect-timeout 0.25` and `--max-time 0.75`, that
  `--max-time` is the *effective* total ceiling and never the raw request when
  the adapter ceiling is lower, and that sub-microsecond positive durations are
  still rejected as `InvalidInput`. This retains the sub-second `--max-time`
  proof that the end-to-end test had to give up to become deterministic.

**Compatibility, stated precisely.** This entry is **test-only**: no public
API, no public field, no error variant, no dependency, no feature, and no
observable behaviour changed. No production timeout policy, tolerance,
`POLL_INTERVAL`, or classification rule was touched, and no deadline was
widened. **No publication or version bump is authorized or implied by this
milestone**, and no consumer migration is required.

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
