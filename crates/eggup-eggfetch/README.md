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
