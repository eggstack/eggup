# eggfetch Adapter Deep-Dive (`crates/eggup-eggfetch`)

> Source: `crates/eggup-eggfetch/src/lib.rs` (1180 lines), `README.md`, `Cargo.toml`.
> Dependency pin: `eggfetch-core 0.2.0` with `http1,tls-rustls,tls-native-roots,proxy; no compression/cookies/retry/json`
> (reported by `EggfetchTransport::eggfetch_version()`, `lib.rs:210-214`).

## 1. Purpose

Native HTTP acquisition adapter implementing `eggup-acquisition::AcquisitionTransport`
(`lib.rs:9-12,231`).

Explicit non-goals stated in code/docs (`lib.rs:186-190`, `README.md:3-5`):

- No release, version, fallback, destination, or service policy.
- No automatic retry, decompression, cookies, JSON handling.
- 404 is typed distinctly as `FetchOutcome::NotFound` but never triggers
  fallback inside the adapter.
- Checksum mismatch, TLS errors, malformed metadata, timeouts, 5xx, and
  redirect-policy violations are hard failures.

Crate attributes: `#![forbid(unsafe_code)]`, `#![deny(missing_docs)]` (`lib.rs:1-2`).

## 2. Single Policy Point: `EggfetchConfig`

All transport policy lives in `EggfetchConfig` (`lib.rs:50-63`):

```rust
pub struct EggfetchConfig {
    pub user_agent: String,
    pub connect_timeout: Duration,
    pub total_timeout: Duration,
    pub max_redirects: usize,
    pub proxy: ProxyDecision,
}
```

Builders: `user_agent()`, `timeouts()`, `max_redirects()`, `proxy()` (`lib.rs:84-112`).
`EggfetchConfig::strict()` == `Default` (`lib.rs:79-81`).
`EggfetchTransport::strict(config)` is the only constructor; it validates
timeouts and proxy eagerly and fails closed (`lib.rs:199-202`).
`transport.config()` exposes the active config (`lib.rs:205-207`).

## 3. Defaults

Defined as constants (`lib.rs:18-24`):

| Field | Default | Note |
|---|---|---|
| `user_agent` | `"eggup-eggfetch"` (`DEFAULT_USER_AGENT`) | Sent via `Client::builder().user_agent()` (`lib.rs:167-168`) |
| `connect_timeout` | 10 s (`DEFAULT_CONNECT_TIMEOUT`) | Matches eggsact/stegoeggo posture |
| `total_timeout` | 120 s (`DEFAULT_TOTAL_TIMEOUT`) | Wall-clock, headers + body streaming |
| `max_redirects` | 10 (`DEFAULT_MAX_REDIRECTS`) | Asset-chain bound |
| `proxy` | `ProxyDecision::Disabled` (default variant) | Never inherited accidentally |

README compatibility note (`README.md:59-61`): eggsact (10 s/120 s) unchanged;
stegoeggo (10 s/60 s) tightens from prior adapter-enforced 120 s to requested
60 s (stricter wins, never extended).

## 4. Strict Network Posture

Built in `EggfetchConfig::build_client()` (`lib.rs:155-183`):

- `HttpVersionPolicy::Http1Only` (`lib.rs:169`).
- Rustls with native roots + packaged WebPKI fallback (eggfetch default),
  full certificate/hostname verification (`README.md:11-13`).
- `RedirectPolicy::strict(max_redirects)` (`lib.rs:151-153`).
- `automatic_decompression(false)` (`lib.rs:172`).
- No retry / cookies / JSON features enabled (see Cargo features below).
- Downgrade policy: strict HTTPS→HTTP downgrade rejection; tested explicitly
  in `downgrade_policy_denies_https_to_http` (`lib.rs:612-623`).

Cargo features (`Cargo.toml:25`):

```toml
eggfetch-core = { version = "0.2.0", default-features = false,
  features = ["http1", "tls-rustls", "tls-native-roots", "proxy"] }
tokio = { version = "1", default-features = false,
  features = ["rt", "time", "fs", "io-util", "net"] }
futures-util = { version = "0.3", default-features = false,
  features = ["alloc"] }
```

Dev-dependencies add `url = "2"` and fuller tokio features for tests
(`Cargo.toml:29-31`).

## 5. Timeouts as Ceilings: `min(request, adapter)`

Documented in `timeouts()` (`lib.rs:89-100`), `effective_timeouts()` (`lib.rs:114-123`),
and `README.md:29-45`.

Formula:

```text
effective connect = min(request connect, adapter connect ceiling)
effective total   = min(request total,   adapter total ceiling)
```

- Delegates to `FetchLimits::effective(connect_ceiling, total_ceiling)`.
- A stricter adapter tightens a deadline; it never extends one.
- Both `fetch_metadata` and `fetch_artifact` derive `effective_request_timeout()`
  (`lib.rs:132-135`) and pass it as a real Eggfetch request-level `.timeout()`
  override (`lib.rs:254`, `lib.rs:337`), never a post-hoc elapsed check.
- Effective total covers headers plus body streaming.
- `effective_timeouts()` assumes `limits.validate()` succeeded; entry points
  call `limits.validate()?` first (`lib.rs:238`, `lib.rs:302`).

Validation fails closed (`lib.rs:137-149`):

- Zero connect or total → `InvalidInput("adapter timeouts must be positive")`.
- `connect > total` → `InvalidInput("adapter connect timeout must not exceed total timeout")`.
- Enforced in `EggfetchTransport::strict` via `build_client()`.

Client-level `config.timeout()` (`lib.rs:125-130`) is still set on the builder,
but per-fetch `effective_request_timeout` is authoritative per request.

## 6. Redirect Bound

- `redirect_policy() = RedirectPolicy::strict(max_redirects)` (`lib.rs:151-153`).
- Default bound 10; strict HTTPS→HTTP downgrade denial.
- Violation surfaces as Eggfetch error → mapped to `AcquisitionError::Transport`
  (hard failure, never `NotFound`).
- Test `redirect_limit_is_enforced` uses a self-redirect loop with
  `max_redirects = 2` and asserts `Transport` (`lib.rs:777-807`).

## 7. `ProxyDecision`: Explicit Routing

```rust
pub enum ProxyDecision {
    #[default] Disabled,
    FromEnvironment,
    Custom(Vec<(String, String)>),
}
```

(`lib.rs:27-36`)

- `environment()` maps to `ProxyEnvironment::new()` / `from_env()` / `from_map()`
  (`lib.rs:39-48`).
- Passed via `builder.proxy_environment(&env)` (`lib.rs:173-182`).
- Construction is treated as infallible for these decisions; any error maps to
  category-only `Transport("proxy configuration/routing failure")` without
  echoing environment content.
- Invalid proxy (e.g. `http://user:bogus@[::1`) fails closed in `strict()`,
  never silently direct — tested in `invalid_proxy_fails_closed` (`lib.rs:843-849`).
- `InvalidProxyUrl` at request/fetch time also maps to the same generic
  `Transport` message (`lib.rs:449-451`, `lib.rs:489-491`).

## 8. User-Agent

- Default `"eggup-eggfetch"` (`lib.rs:18`).
- Overridable via `EggfetchConfig::user_agent()` (`lib.rs:84-87`).
- Forwarded to `Client::builder().user_agent(&self.user_agent)` (`lib.rs:168`).

## 9. `fetch_metadata` Flow (`lib.rs:232-293`)

1. `limits.validate()?`; early `Cancelled` if flag set (`lib.rs:238-241`).
2. `max = limits.max_metadata_bytes` is authoritative; Content-Length advisory only.
3. Derive `effective = effective_request_timeout(limits)`.
4. `client.get(url).timeout(effective).max_decoded_body_size(max).send().await`
   (`lib.rs:249-258`).
   - Build errors → `map_request_error`; send/body errors → `map_fetch_error`
     with `Some(max)`.
5. `classify(status)` (see §11).
   - `NotFound` → `Ok(FetchOutcome::NotFound)`.
   - `HardFailure` → `Transport("HTTP {status} from {redacted} while fetching metadata")`.
   - `Success`:
     - Re-check cancel.
     - If `content_length() > max` → `TooLarge { limit: max }` (`lib.rs:270-274`).
     - `response.bytes().await` → if `len > max` → `TooLarge` (`lib.rs:275-281`).
     - Wrap via `__adapter_metadata(bytes)` → `FetchOutcome::Success`.
6. No extra wall-clock wrapper: inner Eggfetch `Timeout` covers connect/total;
   comment at `lib.rs:288-292` states `block_on` inherits client deadlines.

## 10. `fetch_artifact` Flow (`lib.rs:295-419`)

1. `limits.validate()?`; early `Cancelled` (`lib.rs:302-305`).
2. Destination parent checks (`lib.rs:306-315`):
   - `dest.parent()` must exist; `symlink_metadata` must be a real directory
     (not symlink) else `InvalidInput`/`Io`.
3. Fast-fail if `dest` already exists via `symlink_metadata(dest).is_ok()` →
   `InvalidInput("artifact destination already exists; refusing to overwrite")`
   (`lib.rs:318-322`). Race-safe guarantee comes later from
   `__promote_no_clobber`.
4. Derive `effective` timeout; `client.get(url).timeout(effective).send().await`
   (no `max_decoded_body_size`; bound enforced during streaming)
   (`lib.rs:333-340`).
5. `classify(status)`: `NotFound` → `Ok(NotFound)` with no file created;
   `HardFailure` → `Transport("HTTP {status} from {redacted} while fetching artifact")`.
6. Exclusive staging (`lib.rs:351-370`):
   - `__acquire_exclusive_temp(&parent, "eggup-eggfetch")` — owner-private
     (`0600` on Unix), sibling in destination parent (same filesystem).
   - `Guard` drops → `__remove_owned_temp` unless disarmed.
7. Streaming (`lib.rs:372-395`):
   - `response.bytes_stream()`; per-chunk cancel check; skip empty chunks.
   - `written.saturating_add(len)`; if `max_artifact = Some(max)` and
     `written > max` → `TooLarge { limit: max }`. Unbounded (`None`) still counts.
   - `tokio::fs::File::from_std`, `write_all`, `flush`; I/O errors →
     `Io("writing/flushing part file: ...")`.
8. Post-stream cancel check, then `__promote_no_clobber(&tmp, &dest)` (`lib.rs:400-415`):
   - Success → disarm guard, `__adapter_artifact(written)`.
   - Failure (existing or raced-in dest) → remove owned temp, disarm, return error.
   - Per README (`README.md:47-52`): successful hard-link creation commits the
     destination; failure to remove the redundant temp link is best-effort and
     cannot turn commit into failure.

Truncation (e.g. `Content-Length: 4096` with early close) surfaces as stream
error → `Transport`, dest never promoted (`truncation_is_a_hard_failure_without_promotion`,
`lib.rs:712-736`).

## 11. Status Classification

```rust
enum StatusClass { Success, NotFound, HardFailure }
fn classify(status: u16) -> StatusClass {
    match status {
        200..=299 => Success,
        404 => NotFound,
        _ => HardFailure,
    }
}
```

(`lib.rs:429-442`)

- Only exact `404` → `FetchOutcome::NotFound` (both operations).
- All other non-2xx (including 5xx, 302-over-bound, 3xx-observed as failure)
  → `Transport` hard failure.
- Tests: `not_found_is_typed_distinctly` (metadata + artifact, dest untouched),
  `server_error_never_becomes_not_found` (500 → `Transport`),
  `seam_fixture_and_adapter_agree_on_not_found` (cross-check vs `FixtureTransport`).

## 12. `TooLarge` / `Timeout` Mapping

`map_fetch_error(e, request, max)` (`lib.rs:467-498`):

| Eggfetch error | Acquisition error |
|---|---|
| `DecodedBodyTooLarge` | `TooLarge { limit: max.unwrap_or(0) }` |
| `Timeout { phase: Connect }` | `Timeout { phase: "connect" }` |
| `Timeout { phase: Total }` | `Timeout { phase: "total" }` |
| `TransportIoTimeout` (established-connection inactivity) | `Timeout { phase: "total" }` |
| `InvalidProxyUrl` | `Transport("proxy configuration/routing failure")` |
| other | `Transport("fetch failed for {redacted}: {kind}")` |

`map_request_error` (`lib.rs:444-465`): same timeout/proxy mapping for
build-phase errors; other → `Transport("request build failed for {redacted}: {kind}")`.

Metadata additionally enforces `TooLarge` via declared `content_length()` and
actual `bytes.len()` checks (advisory + authoritative). Artifact enforces via
streaming `written > max` counter.

Timeout tests: `total_timeout_is_a_hard_failure`, `request_stricter_than_adapter_wins`,
`adapter_stricter_than_request_wins`, `metadata_body_stall_times_out_without_fallback`,
`artifact_body_stall_after_partial_data_times_out` (partial artifact not promoted,
no `.part` leftovers), `timeout_never_becomes_not_found`.

## 13. Redaction

- All diagnostics use `request.redacted()` (userinfo/query stripped) plus
  `e.kind()` category text — never upstream `Error` display which may contain
  proxy URLs with credentials (`lib.rs:446-448, 487-488`, `README.md:54-57`).
- `bound()` truncates messages to 512 chars (`lib.rs:421-427`).
- Proxy failure text is fixed category-only: `"proxy configuration/routing failure"`.
- Test `error_diagnostics_never_expose_credential_sentinels` (`lib.rs:1122-1169`)
  asserts userinfo/query/proxy-password sentinels never appear in `Display`/`Debug`,
  that raw `InvalidProxyUrl` display *does* contain the password (proving the
  mapper is load-bearing), and that bad proxy config fails without echo.

## 14. Exclusive Temp + No-Clobber

- Helpers from `eggup-acquisition` (seam-owned): `__acquire_exclusive_temp`,
  `__remove_owned_temp`, `__promote_no_clobber`, `__adapter_metadata`,
  `__adapter_artifact`.
- Temp prefix `"eggup-eggfetch"`; cleanup tests filter that prefix
  (`lib.rs:867-871`) and `.part` suffix for stall/no-clobber paths.
- `0600` Unix mode asserted in
  `exclusive_temp_helper_is_owner_private_for_eggfetch_prefix` (`lib.rs:1107-1118`).
- `artifact_refuses_existing_destination_without_overwrite` asserts foreign
  content preserved, no `.part` leftovers (`lib.rs:1077-1105`).
- `output_file_is_cleaned_on_failure` asserts 500 leaves no dest and no
  `.eggup-eggfetch-*` temp (`lib.rs:851-875`).

## 15. Cancellation

`CancelFlag` checked:

- Before any I/O (both ops).
- After headers / before body (metadata success path, `lib.rs:267-269`).
- Per stream chunk (artifact, `lib.rs:377-379`).
- After streaming / before promotion (artifact, `lib.rs:400-402`).

On cancel → `Err(AcquisitionError::Cancelled)`; artifact `Guard` cleans the temp.

## 16. Sync Bridge

`block_on` (`lib.rs:215-228`): private `tokio::runtime::Builder::new_current_thread().enable_all()`
built per call; `expect("eggfetch current-thread runtime")` on failure.
Rationale in comment: sync seam never forces an async runtime into eggup-core
or lightweight consumers; async consumers should call via `spawn_blocking` per
their own policy.

## 17. Error Mapping / Downgrade Discipline

- Transport surfaces only seam errors: `InvalidInput`, `Transport`, `TooLarge`,
  `Timeout`, `Io`, `Cancelled`. No fallback, no downgrade of hard failures to
  `NotFound` except exact 404.
- `bound()` caps at 512 chars to prevent unbounded upstream/detail growth.
- Invalid request limits fail before any Eggfetch I/O or filesystem staging:
  `invalid_public_limits_fail_before_eggfetch_io` covers zero/oversize metadata
  cap, zero connect/total, `connect > total`; asserts dest absent and temp dir
  empty (`lib.rs:636-678`).
- `connect_exceeding_total_is_rejected_in_both_layers` covers seam
  `FetchLimits::new` and adapter `strict()` plus zero-deadline rejection
  (`lib.rs:961-976`).
- `effective_formula_is_minimum_per_phase` pins `min()` per phase
  (adapter 1 s/10 s vs request 5 s/5 s → 1 s/5 s) (`lib.rs:1171-1179`).

## 18. Tests (local TCP harness, no external network)

Harness: `serve_once` minimal single-response HTTP/1.1 server on 127.0.0.1
with configurable status/headers/body/split/stall/abort (`lib.rs:537-583`);
`ok_server`, `serve_partial_then_stall` variants.

| Test | Covers |
|---|---|
| `strict_config_uses_expected_policy` | Defaults + version string pin |
| `downgrade_policy_denies_https_to_http` | Redirect downgrade denial |
| `metadata_200_is_success` | Happy-path metadata bytes |
| `invalid_public_limits_fail_before_eggfetch_io` | Pre-I/O validation, no staging |
| `oversized_metadata_is_too_large` | 128 KiB > 64 KiB cap → `TooLarge` |
| `streaming_download_writes_exact_bytes` | 8192-byte chunked artifact, byte-exact |
| `truncation_is_a_hard_failure_without_promotion` | Early close → `Transport`, no dest |
| `not_found_is_typed_distinctly` | 404 → `NotFound` both ops |
| `server_error_never_becomes_not_found` | 500 → `Transport` |
| `redirect_limit_is_enforced` | Self-loop + `max=2` → `Transport` |
| `total_timeout_is_a_hard_failure` | 3 s stall vs 300 ms → `Timeout`/`Transport` |
| `invalid_proxy_fails_closed` | Bad proxy URL → `strict` errs |
| `output_file_is_cleaned_on_failure` | 500 → no dest, no temp |
| `seam_fixture_and_adapter_agree_on_not_found` | Fixture cross-check |
| `request_stricter_than_adapter_wins` | min() direction 1 (M003) |
| `adapter_stricter_than_request_wins` | min() direction 2 (M003) |
| `connect_exceeding_total_is_rejected_in_both_layers` | Fail-closed validation |
| `metadata_body_stall_times_out_without_fallback` | Header+stall → `Timeout` |
| `artifact_body_stall_after_partial_data_times_out` | Mid-body stall → `Timeout`, no promote |
| `timeout_never_becomes_not_found` | Timeout ≠ `NotFound`/`InvalidInput` |
| `artifact_refuses_existing_destination_without_overwrite` | No-clobber fast-fail |
| `exclusive_temp_helper_is_owner_private_for_eggfetch_prefix` (unix) | 0600 temp |
| `error_diagnostics_never_expose_credential_sentinels` | Redaction audit |
| `effective_formula_is_minimum_per_phase` | min() formula pin |

## 19. Review Checklist

- [ ] `EggfetchConfig` remains the only policy knob; no hidden defaults added.
- [ ] `eggfetch-core` version/features still match `eggfetch_version()` string
      when bumped (`Cargo.toml:25` vs `lib.rs:210-214`).
- [ ] HTTP/1-only, Rustls, no decompression/cookies/retry preserved.
- [ ] Both ops still call `limits.validate()` before I/O and derive effective
      timeouts via `min(request, adapter)` with a real request-level override.
- [ ] `connect > total` and zero deadlines still fail closed in `strict()`.
- [ ] Only exact 404 → `NotFound`; 5xx/redirect/timeout/TLS/checksum stay hard.
- [ ] `TooLarge` enforced three ways: decoded-body cap (metadata), declared +
      actual length (metadata), streaming counter (artifact).
- [ ] `TransportIoTimeout` still maps to `Timeout{total}`, not `Transport`.
- [ ] Diagnostics still use `redacted()` + `kind()` only, `bound()` at 512;
      no upstream `Display` or proxy URL embedded.
- [ ] Artifact staging still: real-dir parent, pre-existing dest fast-fail,
      exclusive 0600 temp in dest parent, `Guard` cleanup,
      `__promote_no_clobber` commit, owned-temp-only removal.
- [ ] Cancel checked pre-I/O, per-chunk, and pre-promotion.
- [ ] Sync bridge still private current-thread runtime; no global/executor leak.
- [ ] New Eggfetch error variants mapped explicitly (no silent `Transport`
      downgrade of timeouts, no leak of new detail strings).
