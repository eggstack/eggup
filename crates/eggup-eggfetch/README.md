# eggup-eggfetch

Native Rust HTTP acquisition adapter for Eggup consumers already using
Eggfetch, with explicit bounded network policy and no release/fallback
authority.

Policy (single configuration point in `EggfetchTransport::strict`):

- HTTP/1 only (`HttpVersionPolicy::Http1Only`);
- Rustls with native roots plus packaged WebPKI fallback (eggfetch default)
  and full certificate/hostname verification;
- finite connect timeout (default 10 s) and total timeout (default 120 s) as
  distinct phases;
- finite redirect count (default 10) with strict HTTPS → HTTP downgrade
  rejection;
- explicit environment-proxy decision; invalid proxy configuration fails
  closed, never silently direct;
- bounded metadata in memory; artifact bodies stream to disk chunk-by-chunk,
  never buffered as one allocation;
- no automatic retry; no decompression/cookies/retry features;
- 404 is typed distinctly (`NotFound`) but never triggers fallback inside the
  adapter; checksum mismatch, TLS errors, malformed metadata, timeouts, 5xx,
  and redirect-policy violations are hard failures.

The adapter implements `eggup-acquisition::AcquisitionTransport` (sync seam)
by bridging async Eggfetch on a private current-thread runtime. URLs and
secrets are redacted in diagnostics.

Effective timeouts (M003 corrective):

```text
effective connect = min(request connect, adapter connect ceiling)
effective total   = min(request total, adapter total ceiling)
```

Both operations apply the effective deadlines via a real Eggfetch
request-level timeout override (never a post-hoc elapsed check).
`EggfetchConfig::timeouts` are ceilings; invalid values (zero or
`connect > total`) fail closed at `EggfetchTransport::strict`.
Because `FetchLimits` fields remain public for 0.1.x compatibility, each
metadata and artifact call also validates request limits before network or
filesystem work. `EggfetchConfig::effective_timeouts` assumes validated input;
the transport entry points enforce that precondition.
A caller deadline surfaces as `AcquisitionError::Timeout`
(including established-transport inactivity mapped to `total`).

Staging matches the seam contract: exclusively-created owner-private
(`0600` on Unix) temp siblings in the destination parent, race-safe
no-clobber promotion (existing or raced-in `dest` fails explicitly and is
preserved), owned-temp-only cleanup. Successful no-clobber hard-link creation
commits the complete destination; failure to remove the redundant temp link is
best-effort cleanup and cannot turn the committed fetch into ordinary failure.

Redaction audit (M003): upstream `eggfetch_core::Error` display and proxy
details are never embedded. All transport failures use `Error::kind()`
category text plus the redacted URL, so userinfo, query tokens, proxy
passwords, and redirect targets cannot leak.

Compatibility: eggsact (10 s/120 s) is unchanged by the correction;
stegoeggo (10 s/60 s) tightens from the prior adapter-enforced 120 s total
to its requested 60 s total (stricter, never extended).
