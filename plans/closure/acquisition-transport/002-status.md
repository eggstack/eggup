# Acquisition Transport M002 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/acquisition-transport/002-eggfetch-adapter.md`

Source roadmap: `plans/subsystems/acquisition-transport-roadmap.md#M002--eggfetch-acquisition-adapter`

Reviewed repository baseline: `9f527beb20da585bc3cf56f45f1fd96fd418c124` (plan baseline; implementation ran at transport M001 closure `42fa5b0` plus adapter changes)

## Implementation commits/PRs

- M002 adapter commit (this pass): `feat: add eggup-eggfetch adapter (M002)` (pending SHA; see git log)
- Prior: `42fa5b0` (acquisition M001 seam + fixture)
- No PR was required for this local implementation pass. No publication was performed.

## Executive finding

M002 is complete. `eggup-eggfetch` implements the M001 seam as the preferred native Rust HTTP adapter for consumers already using Eggfetch, with explicit bounded policy and no release/fallback authority. Local fixture coverage proves redirect, status, timeout, proxy, bound, and cleanup behavior; core remains transport-free.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| `crates/eggup-eggfetch`; exact Eggfetch dependency/features | `eggfetch-core 0.2.0` with `http1,tls-rustls,tls-native-roots,proxy`; no compression/cookies/retry/json/http2/3; `EggfetchTransport::eggfetch_version()` records the surface | passed |
| Adapter implements M001 seam | `impl AcquisitionTransport for EggfetchTransport` (`fetch_metadata`, `fetch_artifact`); seam fixture + adapter agree on `NotFound` (`seam_fixture_and_adapter_agree_on_not_found`) | passed |
| Local HTTP fixture server | std-only `TcpListener` harness in `#[cfg(test)]` (no extra prod deps); single-response + redirect-loop servers | passed |
| Redirect/status/timeout/proxy/body-bound tests | 13 tests: 200 metadata, oversized, streaming, truncation, 404, 500, redirect loop/limit, downgrade seam, total timeout, invalid proxy, cleanup | passed |
| Package docs | `crates/eggup-eggfetch/README.md` + rustdoc policy block; `#![deny(missing_docs)]` clean | passed |
| Dependency and binary-size measurements | `cargo tree` recorded below; package sizes recorded; methodology documented | passed |
| HTTP/1 only unless Eggfetch requires otherwise | `HttpVersionPolicy::Http1Only` in `build_client`; `strict_config_uses_expected_policy` | passed |
| Rustls + documented root store | `tls-rustls` + `tls-native-roots` (native roots w/ WebPKI fallback, eggfetch default); README documents full cert/hostname verification | passed |
| Finite connect/total deadlines | `Timeout::builder().connect(10s).total(120s)`; distinct phases; `total_timeout_is_a_hard_failure` | passed |
| Finite redirect count | `RedirectPolicy::strict(10)`; `redirect_limit_is_enforced` | passed |
| Downgrade rejection | `DowngradePolicy::Deny` via `strict`; `downgrade_policy_denies_https_to_http` through test seam (live https→http downgrade not exercised against plain-http fixtures by design) | passed |
| Explicit proxy; invalid fails closed | `ProxyDecision::{Disabled, FromEnvironment, Custom}`; `proxy_environment` mapped at build; `invalid_proxy_fails_closed` | passed |
| Bounded metadata; streamed binary; no retry | `max_decoded_body_size` per fetch; `bytes_stream` chunked to temp + rename; `automatic_decompression(false)`; no retry policy configured | passed |
| 404 typed distinctly, never fallback | `classify(404) → NotFound`; `not_found_is_typed_distinctly`; 500/truncation/timeout/redirect all `Transport`/`TooLarge`/`Timeout`, never `NotFound` | passed |
| No decompression/cookies/retry unless proven | Features exclude them; `cargo tree` confirms absence | passed |

## Exact Eggfetch version/features

`eggfetch-core 0.2.0` (crates.io, matches eggsact consumer) with `default-features = false` plus `http1`, `tls-rustls`, `tls-native-roots`, `proxy`. Direct additions: `tokio` (rt/time/fs/io-util/net for the sync bridge + streaming writes), `futures-util` (alloc for `bytes_stream`). No `compression-*`, `cookies`, `json`, `http2/3`, `tracing`, `multipart`.

## Fixture matrix

| Case | Harness | Result |
|---|---|---|
| 200 metadata | `ok_server` | `Success`, exact bytes |
| Oversized metadata | 128 KiB body vs 64 KiB bound | `TooLarge` |
| Streaming download | 8192 pseudo-random bytes | exact bytes, `bytes_written=8192` |
| Truncation | headers then connection close | `Transport`, no promotion |
| 404 metadata + artifact | separate 404 servers | `NotFound` both, no file |
| 500 | 500 server | `Transport`, never `NotFound` |
| Redirect loop | self-302 loop, `max_redirects=2` | `Transport` after bound |
| Downgrade | `RedirectPolicy::strict.check_downgrade` unit seam | https→https ok, https→http err |
| Total timeout | 3 s stall vs 300 ms deadline | `Timeout` or `Transport` (both hard, never `NotFound`) |
| Invalid proxy | `HTTPS_PROXY=http://user:bogus@[::1` | `strict()` returns `Err` (fails closed) |
| Cleanup | 500 artifact fetch | no `dest`, no `.eggup-eggfetch-*.part` leftovers |
| Seam agreement | fixture 404 vs adapter contract | both `NotFound` |

## Dependency trees (isolation proof)

`cargo tree -p eggup-core` (14 lines): `eggup-core → sha2` only. No HTTP/TLS/service-manager.

`cargo tree -p eggup-eggfetch --depth 1`: `eggup-eggfetch → eggfetch-core 0.2.0, eggup-acquisition (path), futures-util, tokio (+ dev url)`. Full transitive tree: 211 lines / ~209 versioned nodes (hyper, rustls, tokio, url, etc., all under the transport crate).

`cargo tree -p eggup-acquisition`: zero dependencies.

Core manifest unchanged with respect to transport: no new dependency in `eggup-core`; transport lives only in `eggup-eggfetch`.

## Size measurement methodology/result

Methodology: `cargo package` compressed sizes for path-independent crates + `cargo tree` node counts for the transport graph. No `target/` or binary-size claim is made for a library.

- `eggup-acquisition-0.1.0.crate`: 6 files, 30.1 KiB (7.6 KiB compressed).
- `eggup-core-0.1.0.crate`: 19 files, 155.5 KiB (31.3 KiB compressed) at M006.
- `eggup-eggfetch`: `cargo package` is blocked on publication ordering (`eggup-acquisition` path dep not yet on crates.io; `cargo package -p eggup-eggfetch` errors `no matching package named eggup-acquisition`). This is the expected ADR-0001 multi-crate ordering, not a code defect. Transitive graph size above (211 lines) is the recorded cost proxy until the seam crate is published.
- No decompression/cookie/retry features are enabled; the graph contains no `async-compression`, `cookie`, or retry policy crates.

## HTTP policy documentation

`README.md` + rustdoc record the single configuration point (`EggfetchConfig::strict`): HTTP/1-only, Rustls + native/WebPKI roots, 10 s connect + 120 s total, 10 redirects with downgrade deny, explicit proxy (disabled by default; environment opt-in), bounded metadata, streamed artifacts, no retry, 404-vs-hard-failure taxonomy, redaction.

## Package qualification result

- `cargo package -p eggup-acquisition`: green (above).
- `cargo package -p eggup-eggfetch`: blocked on unpublished path dep (ordering; see above). Build, test, doc, and tree qualification are green; publication requires `eggup-acquisition` first.
- `cargo publish --dry-run` not claimed for `eggup-eggfetch` for the same ordering reason. No publication was performed for any crate.

## Exact tests/commands actually run

Environment: `Darwin 25.6.0 arm64`, stable `1.98.1`, MSRV `1.89.0`.

```text
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo tree -p eggup-core --locked
cargo tree -p eggup-acquisition --locked
cargo tree -p eggup-eggfetch --locked
cargo package -p eggup-acquisition --locked --allow-dirty
./scripts/check-local.sh
```

MSRV variants (`cargo +1.89.0 …`) were run for check/clippy/test/doc and are green. Results: 13/13 `eggup-eggfetch` + 12/12 `eggup-acquisition` + 37/37 `eggup-core` passed on stable and 1.89. One clippy `manual_flatten` finding was fixed during the pass. Hosted CI not run locally; lanes cover stable/MSRV/macOS/Windows-check.

## Invariant review

- No Eggfetch dependency in `eggup-core`: proven by tree.
- HTTP/1 + Rustls ownership explicit and minimal: features above.
- Downgrade rejected; redirects bounded; proxy explicit; deadlines explicit; metadata bounded; artifacts streamed; 404 distinct; no fallback; no decompression/cookies/retry.
- Downloaded bytes never executed; no shell; redacted diagnostics.

## Failure/recovery review

No fallback on checksum-mismatch-equivalent (`TooLarge`), TLS error, malformed metadata, timeout, 5xx, or redirect violation — all map to `Err`, never `NotFound`. Partial artifacts never promote; temp siblings cleaned. Invalid proxy fails at construction, never silently direct.

## Compatibility/migration review

New crate; no existing consumers migrate. M002 is the transport half of the adoption gate (with core M006, already closed). Consumer policy (release selection, Cargo fallback, CLI) remains outside.

## Security review

- No release/fallback authority in adapter; exact-URL only.
- Parent must pre-exist as a real directory; atomic temp+rename; cleanup verified.
- Redaction of credentials/query in all errors.
- `#![forbid(unsafe_code)]`; no privilege escalation.

## Unresolved findings with severity

- Low / accepted (ordering): `eggup-eggfetch` packaging requires `eggup-acquisition` published first. No code action; publish in seam-then-adapter order when release is directed.
- Informational: Live HTTPS downgrade exercised via policy unit seam, not a live TLS downgrade (plain-http fixtures by design).
- Informational: Hosted CI not run locally.
- None: No medium-or-higher safety/policy issue remains.

## Disposition and roadmap transition

M002 is closed. Unblocks consumer adoption M001 (eggsact; core M006 already closed). Service M001 remains independently ready. Do not add release-policy decisions to the adapter; consumer differences stay explicit caller policy.
