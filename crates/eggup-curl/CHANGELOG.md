# Changelog

## Unreleased

Nothing in this section has been published. `0.1.2` is the published baseline on
crates.io, and no publication milestone authorizes these entries.

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
the adapter never gains a `User-Agent` surface. The only consumer-visible
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
