# Deep dive: `eggup-eggfetch` (native HTTP adapter)

Review entry point for the native Rust transport adapter. Authoritative
contracts remain the [seam deep dive](acquisition.md),
[ADR-0003](../plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md),
and the closure records under
[`plans/closure/acquisition-transport/`](../plans/closure/README.md). Where this
document and code disagree, code wins — see
[Known doc/code drift](#known-doccode-drift).

## 1. Purpose and ownership boundary

`eggup-eggfetch` turns an `AcquisitionRequest` into bytes: a bounded small body
in memory (`fetch_metadata`) or a streamed, no-clobber-promoted file
(`fetch_artifact`). All of its network policy is declared in
`EggfetchConfig`, and its only outputs are `FetchOutcome` values and
`AcquisitionError` values borrowed from the seam.

What it owns: HTTP/1 request issuance, redirect bound, proxy routing decision,
connect/total deadline **ceilings**, status classification, body-size
enforcement, streaming-to-temp, and the promotion commit point.

What it never decides, and must never grow the ability to decide:

| Never decided here | Owner |
|---|---|
| Which release, version, or URL to fetch | caller; the exact URL arrives in the request (`lib.rs:289`, `lib.rs:365`) |
| Mirror or source fallback | caller's `ComposedTransport` + `CompositionPolicy` in the seam ([acquisition.md](acquisition.md)); the adapter emits `Unavailable` only for its own runtime (`lib.rs:250`, `lib.rs:259`, `lib.rs:266`) |
| Authenticity of the artifact | nobody in this workspace; TLS here is transport confidentiality only (§8) |
| Checksum verification | `eggup-core`, after bytes land ([core-transaction.md](core-transaction.md)) |
| Install destination or parent creation | caller; the adapter requires an existing real parent directory and never creates one (`lib.rs:348-357`) |
| Service lifecycle / execution of bytes | `eggup-service`; downloaded bytes are data ([overview.md](overview.md)) |
| Protocol version, proxy, redirects | caller via `EggfetchConfig`, or the default |

Crate attributes: `#![forbid(unsafe_code)]`, `#![deny(missing_docs)]`
(`lib.rs:1-2`).

## 2. Position in the workspace

```text
eggup-eggfetch ──> eggup-acquisition (zero deps)   +   eggfetch-core 0.2 + tokio + futures-util
```

Manifest: `crates/eggup-eggfetch/Cargo.toml`. `eggfetch-core` is pinned to
`0.2.0` with `default-features = false` and exactly four features:
`http1`, `tls-rustls`, `tls-native-roots`, `proxy` — no compression, cookies,
retry, or JSON. `tokio` is likewise narrowed to `rt`, `time`, `fs`, `io-util`,
`net`. `futures-util` is narrowed to `alloc`. Dev-dependencies add `url = "2"`
and fuller `tokio` features.

This is one of **two interchangeable adapters**; the other is
`eggup-curl` ([curl-adapter.md](curl-adapter.md)). Neither may know the other
exists, and `eggup-acquisition` is the only place composition happens. The
choice is a footprint/trust decision by the caller, measured by
[`eggup-transport-footprint`](transport-footprint.md).

`examples/eggfetch_fetch.rs` is the footprint fixture: it constructs
`EggfetchTransport::strict(EggfetchConfig::strict())` and prints the redacted
request and default limits, performing **no** network I/O.

## 3. Public surface

| Item | Role | Where |
|---|---|---|
| `DEFAULT_USER_AGENT: &str` | `"eggup-eggfetch"`; header value default | `lib.rs:18` |
| `DEFAULT_CONNECT_TIMEOUT: Duration` | 10 s connect ceiling | `lib.rs:20` |
| `DEFAULT_TOTAL_TIMEOUT: Duration` | 120 s wall-clock ceiling | `lib.rs:22` |
| `DEFAULT_MAX_REDIRECTS: usize` | 10 redirect bound | `lib.rs:24` |
| `ProxyDecision` | Explicit routing: `Disabled` (default), `FromEnvironment`, `Custom(Vec<(String,String)>)` | `lib.rs:27-36` |
| `EggfetchConfig` | The policy struct: five public fields | `lib.rs:51-63` |
| `impl Default for EggfetchConfig` | All fields from the four constants + `ProxyDecision::Disabled` | `lib.rs:65-75` |
| `EggfetchConfig::strict()` | Conservative constructor; identical to `default()` | `lib.rs:79-81` |
| `EggfetchConfig::user_agent(impl Into<String>)` | Builder: user agent | `lib.rs:84-87` |
| `EggfetchConfig::timeouts(Duration, Duration)` | Builder: connect/total **ceilings** | `lib.rs:96-100` |
| `EggfetchConfig::max_redirects(usize)` | Builder: redirect bound | `lib.rs:103-106` |
| `EggfetchConfig::proxy(ProxyDecision)` | Builder: routing decision | `lib.rs:109-112` |
| `EggfetchConfig::effective_timeouts(&self, FetchLimits) -> (Duration, Duration)` | `min(request, adapter)` per phase | `lib.rs:121-123` |
| `EggfetchTransport` | The adapter; private `client` + `config` fields | `lib.rs:191-194` |
| `EggfetchTransport::strict(EggfetchConfig) -> Result<Self, _>` | Only constructor; validates and fails closed | `lib.rs:205-208` |
| `EggfetchTransport::config(&self) -> &EggfetchConfig` | Read-back of active policy | `lib.rs:211-213` |
| `EggfetchTransport::eggfetch_version() -> &'static str` | Qualification-time pin string of the Eggfetch version/features | `lib.rs:216-219` |
| `impl AcquisitionTransport for EggfetchTransport` | `fetch_metadata`, `fetch_artifact` | `lib.rs:273-458` |

Private, but load-bearing for review: `ProxyDecision::environment`
(`lib.rs:39-47`), `EggfetchConfig::{timeout, effective_request_timeout,
validate_timeouts, redirect_policy, build_client}` (`lib.rs:125-183`),
`block_on` (`lib.rs:221-270`), `bound` (`lib.rs:460-470`), `StatusClass` /
`classify` (`lib.rs:472-485`), `map_request_error` (`lib.rs:487-508`),
`map_fetch_error` (`lib.rs:510-543`).

The crate imports `eggfetch_core::{Client, HttpVersionPolicy, ProxyEnvironment,
RedirectPolicy, Timeout}` (`lib.rs:8`) — the only upstream items it depends on.

## 4. The policy point: `EggfetchConfig`

Every field is public, so the struct is also a valid struct literal; there is no
other policy surface.

| Field | Type | Default | Meaning |
|---|---|---|---|
| `user_agent` | `String` | `DEFAULT_USER_AGENT` = `"eggup-eggfetch"` | Sent via `Client::builder().user_agent()` (`lib.rs:168`) |
| `connect_timeout` | `Duration` | `DEFAULT_CONNECT_TIMEOUT` = 10 s | **Ceiling** for the connect phase, not a default |
| `total_timeout` | `Duration` | `DEFAULT_TOTAL_TIMEOUT` = 120 s | **Ceiling** for the whole fetch, headers plus body streaming |
| `max_redirects` | `usize` | `DEFAULT_MAX_REDIRECTS` = 10 | Bound handed to `RedirectPolicy::strict` (`lib.rs:151-153`) |
| `proxy` | `ProxyDecision` | `ProxyDecision::Disabled` | `Disabled` never consults environment variables, so routing is never inherited by accident |

`strict()` returns `Self::default()` verbatim (`lib.rs:79-81`) — the strictness
comes from the *values* (finite deadlines, bounded redirects, proxy off), not
from a separate stricter branch. `README.md:7` describes the policy as living in
`EggfetchTransport::strict`; the values are actually chosen by
`EggfetchConfig::strict` and only *validated* by the transport.

Fail-closed validation, reached only through `EggfetchTransport::strict` →
`build_client` → `validate_timeouts` (`lib.rs:137-149`, `lib.rs:156`):

- zero connect or zero total → `InvalidInput("adapter timeouts must be positive")`
- `connect_timeout > total_timeout` → `InvalidInput("adapter connect timeout must not exceed total timeout")`

An invalid proxy never silently degrades to a direct connection: it becomes
`Transport("proxy configuration/routing failure")` at construction
(`lib.rs:176-182`), and `InvalidProxyUrl` at request time maps to the same fixed
string (`lib.rs:492-494`, `lib.rs:534-536`). The upstream error display is
deliberately dropped because it can contain credential-bearing proxy URLs
(`lib.rs:177-181`).

### `ProxyDecision`

| Variant | Effect | Where |
|---|---|---|
| `Disabled` (default) | `ProxyEnvironment::new()` — empty | `lib.rs:41` |
| `FromEnvironment` | `ProxyEnvironment::from_env()` snapshot | `lib.rs:42` |
| `Custom(pairs)` | `ProxyEnvironment::from_map(pairs)` — e.g. `HTTPS_PROXY`, `NO_PROXY` | `lib.rs:43-45` |

### `effective_timeouts` and the `min(request, adapter)` clamp

`effective_timeouts` delegates to the seam's
`FetchLimits::effective` (`crates/eggup-acquisition/src/lib.rs:143-151`):

```text
effective connect = min(request.connect_timeout, config.connect_timeout)
effective total   = min(request.total_timeout,   config.total_timeout)
```

Worked example, matching `effective_formula_is_minimum_per_phase`
(`lib.rs:1222-1230`): adapter `timeouts(1s, 10s)`, request `5s/5s` → effective
`(1s, 5s)`. Connect is clamped by the adapter ceiling, total by the request.

The clamp is applied per fetch, not just documented: both trait methods build an
`effective_request_timeout(limits)` and pass it as a **real Eggfetch
request-level `.timeout(...)` override** (`lib.rs:132-135`, `lib.rs:296`,
`lib.rs:378`). There is no post-hoc elapsed-time check. The client-level
`timeout()` (`lib.rs:125-130`, `lib.rs:171`) is the same ceiling set at build
time, so the per-request override is never looser.

`effective_timeouts` is `pub` and assumes `limits.validate()` already
succeeded (`lib.rs:114-120`). Both transport entry points call
`limits.validate()?` as their first statement (`lib.rs:280`, `lib.rs:344`), so
the precondition holds for every fetch the adapter performs.

## 5. Seam implementation

`impl AcquisitionTransport for EggfetchTransport` (`lib.rs:273-458`).

| Seam guarantee | Where honored |
|---|---|
| Exact URL used verbatim, redaction only for diagnostics | `request.url().to_string()` for the request (`lib.rs:289`, `lib.rs:365`); `request.redacted()` only inside messages |
| Caller bounds validated at the I/O boundary | `limits.validate()?` first line of both methods (`lib.rs:280`, `lib.rs:344`); re-run every call because `FetchLimits` fields stay public for 0.1.x |
| `min(request, adapter)` per phase | `effective_request_timeout` → per-request `.timeout()` (`lib.rs:132-135`, `lib.rs:296`, `lib.rs:378`) |
| `NotFound` is data, never an error, never fallback | `classify` returns `NotFound` only for exact 404 (`lib.rs:479-485`); returned as `Ok(FetchOutcome::NotFound)` |
| `Unavailable` reserved for adapter inability to attempt | Only the runtime-build / thread-teardown paths (`lib.rs:250`, `lib.rs:259`, `lib.rs:266`) |
| `TooLarge.limit` is the bound that was exceeded | `map_fetch_error` takes a mandatory `u64` (`lib.rs:514-518`) and every call site passes the bound it actually enforces |
| Artifacts streamed, never fully buffered | `bytes_stream()` + chunk loop (`lib.rs:413-434`); no whole-body `Vec` on the artifact path |
| `dest` created atomically, no-clobber, temp in destination parent | `__acquire_exclusive_temp(&parent, "eggup-eggfetch")` (`lib.rs:395-396`) then `__promote_no_clobber` (`lib.rs:442`) |
| Owned-temp-only cleanup | local `Guard` calling `__remove_owned_temp(&self.path)` (`lib.rs:397-411`, `lib.rs:450`) |
| Downloaded bytes are data, never executed | no process spawning anywhere in the crate |

The two seam methods are the crate's entire public behavior; everything else is
configuration, classification, or mapping.

## 6. Control flow, step by step

### 6.1 `fetch_metadata` (`lib.rs:274-335`)

1. `limits.validate()?` — `InvalidInput` before any network or filesystem work
   (`lib.rs:280`).
2. Pre-cancel check → `Cancelled` (`lib.rs:281-283`).
3. Resolve bounds: `max = limits.max_metadata_bytes` is authoritative;
   `Content-Length` is advisory only (`lib.rs:284-288`).
4. Resolve the effective deadline from `min(request, adapter)` (`lib.rs:290`).
5. Enter `block_on` and issue `client.get(&url).timeout(effective).max_decoded_body_size(max).send().await` (`lib.rs:292-300`). Request-build errors go through `map_request_error`; send errors through `map_fetch_error(&e, request, max as u64)`.
6. Classify the status (`lib.rs:301-308`): `NotFound` → `Ok(NotFound)`; `HardFailure` → `Transport("HTTP {status} from {redacted} while fetching metadata")`; `Success` → continue.
7. Re-check cancellation before consuming the body (`lib.rs:309-311`).
8. If `content_length()` is present and exceeds `max` → `TooLarge { limit }` (`lib.rs:312-316`). This is the advisory check; it fails before reading a body.
9. `response.bytes().await` — this **is** buffered in memory, which is why the path is bounded twice more: Eggfetch's `max_decoded_body_size` and `bytes.len() > max` (`lib.rs:317-323`).
10. Wrap with `__adapter_metadata(bytes.to_vec())` and return `Success` (`lib.rs:324-326`).

There is no additional wall-clock wrapper around `block_on`; the comment at
`lib.rs:330-333` states the inner Eggfetch `Timeout` is authoritative because it
covers connect and total.

### 6.2 `fetch_artifact` (`lib.rs:337-457`)

1. `limits.validate()?` (`lib.rs:344`).
2. Pre-cancel check (`lib.rs:345-347`).
3. Destination preconditions, all before the request: parent must exist (`lib.rs:348-350` → `InvalidInput`), parent must be an existing **real** directory — `symlink_metadata` plus `!is_dir() || file_type().is_symlink()` rejects a symlinked parent (`lib.rs:351-357`), and an already-present `dest` fails fast with `InvalidInput` (`lib.rs:358-364`). The comment at `lib.rs:358-359` is explicit that this pre-check is an optimization only; the race-safe guarantee comes from promotion.
4. Resolve `max_artifact = limits.max_artifact_bytes`, the redacted URL for reuse in messages, and the effective deadline (`lib.rs:365-368`).
5. Issue the request: `client.get(&url).timeout(effective).send().await` (`lib.rs:374-381`). Note the **absence** of `max_decoded_body_size` on this path — the artifact bound is enforced by the streaming counter in step 7, not by the client's decoded-body cap.
6. Classify the status (`lib.rs:382-391`). `NotFound` returns `Ok(NotFound)` with no file created and no temp; `HardFailure` returns `Transport` with the redacted URL.
7. Create staging: `__acquire_exclusive_temp(&parent, "eggup-eggfetch")` — exclusive `create_new`, `0600` on Unix, collision-bounded retry, inside the destination parent so promotion stays on one filesystem (`lib.rs:392-396`). A local `Guard` is armed immediately after the call returns `Ok` (`lib.rs:397-411`) — the seam's own `Drop`-safety contract is that no other owner can clean the file at that point.
8. Stream: `tokio::fs::File::from_std(std_file)`, then loop over `response.bytes_stream()` (`lib.rs:412-434`). Per iteration: cancellation check, error mapping, skip empty chunks, `written = written.saturating_add(len)`, and `written > max_artifact` → `TooLarge { limit: max_artifact }`. Writes use `AsyncWriteExt::write_all`; failures are `Io("writing part file: …")`.
9. `file.flush()` then `drop(file)` (`lib.rs:435-438`) so the file is closed before promotion. Write errors → `Io("flushing part file: …")`.
10. Final cancellation check before commit (`lib.rs:439-441`).
11. Commit: `__promote_no_clobber(&tmp, &dest)` (`lib.rs:442-454`). Success disarms the guard and returns `Success(__adapter_artifact(written))` — `bytes_written` is the streamed counter, not the file size and not `Content-Length`. Failure removes the owned temp, disarms the guard (to avoid a double removal), and propagates the seam error, which is `InvalidInput` when `dest` raced in and `Io` for other promotion failures.

### 6.3 Where cancellation and size limits are enforced

| Concern | Enforcement point |
|---|---|
| Cancellation, before any I/O | `lib.rs:281-283`, `lib.rs:345-347` |
| Cancellation, metadata after headers | `lib.rs:309-311` |
| Cancellation, artifact per chunk | `lib.rs:418-420` |
| Cancellation, artifact before promotion | `lib.rs:439-441` |
| Metadata size, client-side decoded cap | `.max_decoded_body_size(max)` at `lib.rs:297` |
| Metadata size, declared length (advisory) | `lib.rs:312-316` |
| Metadata size, actual body length (authoritative) | `lib.rs:321-323` |
| Artifact size, streamed counter | `lib.rs:425-430` |
| Time bounds | per-request `.timeout(effective)`: `lib.rs:296`, `lib.rs:378` |
| Redirect bound | client `RedirectPolicy::strict(max_redirects)` at build time: `lib.rs:170`, `lib.rs:151-153` |

Cancellation is cooperative and chunk-granular. It cannot interrupt a read that
is already blocked on a stalled connection; such a fetch is bounded by the
total deadline instead, not by the flag. This is visible in
`artifact_body_stall_after_partial_data_times_out` (`lib.rs:1074-1101`), which
expects `Timeout`, not `Cancelled`.

The artifact size check happens *after* accumulating the chunk, so the temp file
can briefly hold up to `max + one chunk` bytes before the call fails. The temp
is removed on that path, so the excess never reaches `dest`.

## 7. The sync-over-async bridge

The seam trait is synchronous (`fetch_metadata` / `fetch_artifact` return
`Result`, not a future), while Eggfetch is async. The bridge is
`EggfetchTransport::block_on` (`lib.rs:221-270`) plus a `thread_local!` runtime
(`lib.rs:196-200`):

```rust
thread_local! {
    static RUNTIME: RefCell<Option<tokio::runtime::Runtime>> = const { RefCell::new(None) };
}
```

- The runtime is `tokio::runtime::Builder::new_current_thread().enable_all()` — no `rt-multi-thread`, no work-stealing pool (`lib.rs:240-243`).
- It is created lazily on first use per calling thread and cached for the thread's life (`lib.rs:236-255`).
- Rationale given in-code (`lib.rs:224-235`): a `current_thread` runtime is `Send` but not `Sync`, so storing one in `EggfetchTransport` would make the type `!Sync` and break callers that share one transport across threads. The thread-local keeps the type's auto traits intact and builds the reactor and driver once per thread instead of once per artifact.
- Construction failure (fd exhaustion, driver creation) returns `AcquisitionError::__adapter_unavailable` rather than panicking — a `Result` API must not abort, and `Unavailable` is what lets `ComposedTransport` fall back safely (`lib.rs:244-253`).
- If the thread-local is being destroyed (thread teardown), `try_with` fails and the call returns `Unavailable("eggfetch runtime unavailable during thread teardown")` (`lib.rs:264-269`).

What a reviewer must know:

- `block_on` **blocks the calling thread** for the whole fetch, including DNS, TLS handshake, redirects, and body streaming. There is no yield point. An async consumer must wrap the sync seam call in `spawn_blocking` (the doc comment says exactly this at `lib.rs:225-228`).
- Because the runtime is current-thread, a fetch cannot make progress unless the thread is running it — i.e. a `CancelFlag` set from another thread is observed only at the checkpoints in §6.3.
- The bridge is not re-entrant in the sense of being callable from inside another future's poll on the same thread without blocking that thread; the `Unavailable`-on-teardown branch is the only guard.
- `Unavailable` is used **only** for the bridge. Every other failure — including connect refusal and DNS failure — is `Transport`, so composition falls back on it only under an explicit opt-in policy, not by default.

## 8. TLS and networking posture

Declared at client construction in `build_client` (`lib.rs:155-183`):

| Setting | Value | Line |
|---|---|---|
| Protocol version | `HttpVersionPolicy::Http1Only` — no HTTP/2, no upgrade negotiation | `lib.rs:169` |
| TLS | `eggfetch-core` feature `tls-rustls`, plus `tls-native-roots` for native root store; `README.md:10-12` also states the packaged WebPKI fallback that eggfetch applies by default, with full certificate and hostname verification | `Cargo.toml:25`, `README.md:10-12` |
| Redirects | `RedirectPolicy::strict(max_redirects)` — bounded, and HTTPS → HTTP downgrade is rejected | `lib.rs:151-153`, `lib.rs:170` |
| Decompression | `automatic_decompression(false)` | `lib.rs:172` |
| Cookies / retry / JSON | features not enabled at all; also asserted by the `eggfetch_version()` string | `Cargo.toml:25`, `lib.rs:216-219` |
| Proxy | explicit `ProxyEnvironment`; `Disabled` by default | `lib.rs:173-174`, `lib.rs:39-47` |

The downgrade rule is exercised directly against `RedirectPolicy` rather than
over the wire in `downgrade_policy_denies_https_to_http`
(`lib.rs:666-678`): same-scheme redirect is accepted, `https → http` is an
error. The over-the-wire test `redirect_limit_is_enforced`
(`lib.rs:831-862`) runs a self-redirect loop with `max_redirects = 2` and
asserts a `Transport` hard failure.

**Scope of the TLS claim.** This is transport confidentiality and server
authentication of the *connection*, plus downgrade and redirect bounds. It is
**not** artifact authenticity. A successful TLS handshake tells you the bytes
came from the host the caller named; it says nothing about whether the producer
vouched for them. Authenticity is deliberately absent from the workspace
([overview.md](overview.md) verification ladder, [ADR-0003](../plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md)),
and the byte-level evidence this crate produces is a length count
(`ArtifactEvidence::bytes_written`), not a digest.

## 9. Error and redaction behavior

The crate surfaces only seam variants; it never defines its own error type.

| Eggfetch condition | Seam category | Where |
|---|---|---|
| `DecodedBodyTooLarge` | `TooLarge { limit: <bound enforced at this call site> }` | `lib.rs:521` |
| `Timeout { phase: Connect }` | `Timeout { phase: "connect" }` | `lib.rs:495-501`, `lib.rs:522-528` |
| `Timeout { phase: Total }` | `Timeout { phase: "total" }` | same |
| Any other `TimeoutPhase` | `Timeout { phase: "total" }` (conservative default) | `lib.rs:499`, `lib.rs:526` |
| `TransportIoTimeout` | `Timeout { phase: "total" }` — an inactivity timeout on an established connection, so caller deadlines stay truthful | `lib.rs:529-531` |
| `InvalidProxyUrl` | `Transport("proxy configuration/routing failure")` — fixed text | `lib.rs:492-494`, `lib.rs:534-536` |
| Any other error | `Transport("request build failed for {redacted}: {kind}")` or `Transport("fetch failed for {redacted}: {kind}")` | `lib.rs:502-506`, `lib.rs:537-541` |
| HTTP non-2xx, non-404 | `Transport("HTTP {status} from {redacted} while fetching …")` | `lib.rs:304-307`, `lib.rs:386-389` |
| Runtime build / thread teardown | `Unavailable` | `lib.rs:250`, `lib.rs:259`, `lib.rs:266` |
| Parent missing / symlinked / `dest` present | `InvalidInput` | `lib.rs:348-364` |
| Staging or promotion filesystem error | `Io` | `lib.rs:352`, `lib.rs:433`, `lib.rs:437` |

Redaction discipline, all four layers:

- URLs appear only as `request.redacted()`, which is the seam's
  `redact_url`: credentials replaced with `<redacted>@`, fragments and query
  strings replaced, 256-byte truncation (`crates/eggup-acquisition/src/lib.rs:350-379`).
- Upstream `eggfetch_core::Error` **display** is never embedded; only
  `e.kind()` category text is (`lib.rs:505`, `lib.rs:540`). The comments at
  `lib.rs:490-491` and `lib.rs:532-533` give the reason: upstream strings may
  carry proxy URLs with credentials.
- Every message passes through the local `bound()` (`lib.rs:460-470`), which
  truncates to 512 bytes on a UTF-8 char boundary. The boundary-walking loop is
  the one covered by `diagnostic_bound_never_splits_multibyte_code_points`
  (`lib.rs:549-557`).
- `error_diagnostics_never_expose_credential_sentinels` (`lib.rs:1173-1220`)
  asserts, against userinfo / query / proxy-password sentinels, that neither
  `Display` nor `Debug` of the adapter's errors contain them — and separately
  asserts that a raw `InvalidProxyUrl` *does* contain the password, so the test
  proves the mapper is load-bearing rather than accidentally clean.

`NotFound` is never produced from an error path. It exists only as the exact-404
branch of `classify` (`lib.rs:482`).

## 10. Invariants and failure modes

| Condition | Observable outcome | Where handled |
|---|---|---|
| Exact HTTP 404 | `Ok(FetchOutcome::NotFound)`; no file created, no temp left behind | `lib.rs:303`, `lib.rs:384` |
| Any other non-2xx, including 5xx and observed 3xx | `Err(Transport)` | `lib.rs:304-307`, `lib.rs:385-389`, `lib.rs:479-485` |
| Redirect bound exceeded | `Err(Transport)` | client `RedirectPolicy` + `map_fetch_error`; test `lib.rs:831-862` |
| HTTPS → HTTP redirect target | `Err(Transport)` | `RedirectPolicy::strict`; test `lib.rs:666-678` |
| Metadata body exceeds `max_metadata_bytes` | `Err(TooLarge { limit: max })`; no `dest` touched | `lib.rs:314`, `lib.rs:322` |
| Artifact body exceeds `max_artifact_bytes` | `Err(TooLarge { limit: max })`; temp removed, `dest` never created | `lib.rs:426-430` |
| Connect or total deadline elapsed (including mid-body stall) | `Err(Timeout { phase })` | `lib.rs:296`, `lib.rs:378`, `lib.rs:522-531` |
| Cancellation observed at a checkpoint | `Err(Cancelled)`; artifact temp removed by `Guard` | `lib.rs:281`, `lib.rs:309`, `lib.rs:345`, `lib.rs:418`, `lib.rs:439` |
| Truncated body (declared `Content-Length` not delivered) | `Err(Transport)`; `dest` never promoted | stream error mapping, `lib.rs:415`, `lib.rs:421`; test `lib.rs:766-791` |
| `dest` already exists (or races in during streaming) | `Err(InvalidInput)`; foreign bytes preserved byte-for-byte | pre-check `lib.rs:360-364`; commit `lib.rs:442` via `__promote_no_clobber` |
| Parent missing, not a directory, or a symlink | `Err(Io)` / `Err(InvalidInput)` | `lib.rs:348-357` |
| Adapter ceilings zero or `connect > total` | `EggfetchTransport::strict` returns `Err(InvalidInput)`; no transport exists | `lib.rs:137-149`, `lib.rs:156` |
| Caller `FetchLimits` invalid via public struct literal | `Err(InvalidInput)` before route, network, or filesystem work | `lib.rs:280`, `lib.rs:344`; test `lib.rs:690-733` |
| Invalid proxy URL in config | `EggfetchTransport::strict` returns `Err(Transport)`; never a silent direct connection | `lib.rs:176-182`; test `lib.rs:897-904` |
| Runtime build failure or thread teardown | `Err(Unavailable)` — fallible, and safe for composition fallback | `lib.rs:244-269` |
| Write or flush failure on the temp | `Err(Io)`; `dest` never promoted | `lib.rs:433`, `lib.rs:437` |

The universal rule across the table: a `dest` exists only after a complete,
fully flushed body has been no-clobber-promoted. Every failure above that
occurs before promotion leaves `dest` absent and removes only the temp this
call created.

## 11. Testing approach

All 25 `#[test]` functions are in `crates/eggup-eggfetch/src/lib.rs:545-1231`.
The design goal is that **no test requires the public internet or an external
process**: a minimal HTTP/1.1 server is built on `std::net::TcpListener` bound
to `127.0.0.1:0`.

| Test | Line | Exercises |
|---|---|---|
| `diagnostic_bound_never_splits_multibyte_code_points` | 549 | `bound()` UTF-8 safety with 2-byte, 3-byte, and 4-byte code points |
| `strict_config_uses_expected_policy` | 654 | The four default constants and the `eggfetch_version()` pin |
| `downgrade_policy_denies_https_to_http` | 666 | `RedirectPolicy::strict` accepts same-scheme, denies HTTPS→HTTP |
| `metadata_200_is_success` | 680 | 200 metadata returns exact bytes |
| `invalid_public_limits_fail_before_eggfetch_io` | 690 | Five invalid `FetchLimits` literals rejected for both methods; asserts `dest` absent and the temp dir empty |
| `oversized_metadata_is_too_large` | 735 | 128 KiB against a 64 KiB cap → `TooLarge` |
| `streaming_download_writes_exact_bytes` | 746 | 8192-byte artifact written byte-exact; `bytes_written == 8192` |
| `truncation_is_a_hard_failure_without_promotion` | 766 | Abort after headers → `Transport`, `dest` never appears |
| `not_found_is_typed_distinctly` | 793 | 404 for both methods; artifact case asserts `dest` absent |
| `server_error_never_becomes_not_found` | 821 | 500 → `Transport`, never `NotFound` |
| `redirect_limit_is_enforced` | 831 | Self-redirect loop with `max_redirects = 2` → `Transport` |
| `total_timeout_is_a_hard_failure` | 864 | 3 s server stall vs 300 ms deadline |
| `invalid_proxy_fails_closed` | 897 | Malformed `HTTPS_PROXY` makes `strict()` return `Err` |
| `output_file_is_cleaned_on_failure` | 906 | 500 on the artifact path: no `dest`, no `.eggup-eggfetch-*` temp |
| `seam_fixture_and_adapter_agree_on_not_found` | 932 | Cross-checks `NotFound` typing against the seam's `FixtureTransport` |
| `request_stricter_than_adapter_wins` | 958 | 5 s adapter vs 50 ms request → `Timeout` (request wins) |
| `adapter_stricter_than_request_wins` | 986 | 50 ms adapter vs 5 s request → `Timeout` (adapter wins) |
| `connect_exceeding_total_is_rejected_in_both_layers` | 1015 | Seam `FetchLimits::new` and adapter `strict()` both reject; zero deadline rejected |
| `metadata_body_stall_times_out_without_fallback` | 1029 | Headers then 2 s stall vs 300 ms → `Timeout` |
| `artifact_body_stall_after_partial_data_times_out` | 1074 | Partial body then stall → `Timeout`, no promotion, no `.part` residue |
| `timeout_never_becomes_not_found` | 1103 | Timeout stays `Timeout`, is not `NotFound` or `InvalidInput` |
| `artifact_refuses_existing_destination_without_overwrite` | 1128 | Pre-existing `dest` fails; foreign content intact; no residue |
| `exclusive_temp_helper_is_owner_private_for_eggfetch_prefix` (unix) | 1158 | Temp file mode is `0o600` |
| `error_diagnostics_never_expose_credential_sentinels` | 1173 | Redaction audit across `Display`, `Debug`, and the proxy mapper |
| `effective_formula_is_minimum_per_phase` | 1222 | `min()` per phase, pinned numerically |

Harness helpers: `temp_dir` (`lib.rs:581-589`, pid + nanos under the OS temp
dir), `serve_once` (`lib.rs:592-638`, parameterized status / headers / body /
split-body / pre-body stall / abort-after-headers), `ok_server`
(`lib.rs:640-652`), `serve_partial_then_stall` (`lib.rs:1052-1072`).

The tests that need no server at all: `bound()` UTF-8 safety, config defaults,
downgrade policy, proxy fail-closed, two-layer timeout validation, the seam
fixture cross-check, and the redaction audit. Everything else uses loopback.
TLS itself is **not** covered by a live handshake test — the downgrade test
asserts against `eggfetch_core::RedirectPolicy` directly.

Focused commands:

```sh
cargo test -p eggup-eggfetch
cargo test -p eggup-eggfetch <test_name>
cargo clippy -p eggup-eggfetch --all-targets --locked -- -D warnings
./scripts/check-local.sh   # full workspace gate
```

## 12. Trade-offs vs. the curl adapter

Evidence-based comparison, both read from source.

| Dimension | `eggup-eggfetch` | `eggup-curl` |
|---|---|---|
| Dependencies the consumer links | `eggfetch-core` (HTTP/1, Rustls, native roots), `tokio`, `futures-util` (`Cargo.toml`) | `eggup-acquisition` only; no HTTP/TLS stack ([curl-adapter.md](curl-adapter.md)) |
| Process model | in-process, per-thread cached current-thread runtime (`lib.rs:236-255`) | spawns and reaps a child process; kill on cancellation |
| Where deadlines are applied | real in-library request-level timeout override (`lib.rs:296`, `lib.rs:378`) | CLI arguments, serialized with a locale-independent decimal formatter so sub-second values are not widened (M008) |
| Sub-second deadlines | expressed natively as `Duration` | requires a bespoke serializer; the M008 corrective exists specifically because whole-second rounding was widening the deadline |
| Redirect / protocol control | `RedirectPolicy::strict(max_redirects)` with library-level downgrade denial | explicit `--proto` for the initial request plus narrowed `--proto-redir` so `https` never follows down to `http` |
| Failure when the transport itself cannot run | `Unavailable` only for runtime build / thread teardown (`lib.rs:250`, `lib.rs:266`) | `Unavailable` also for a missing executable or spawn failure; discovery via `PATH` is opt-in |
| Blocking the caller | yes, for the whole fetch | yes, waiting on the child |
| Determinism / hermeticity | no external process, but the Rustls + tokio stack and OS root store enter the binary | depends on an external `curl` binary and its build-time TLS backend |
| Consumer-supplied per-request user agent | `EggfetchConfig::user_agent` | not modelled the same way; `CurlConfig` has no user-agent field |
| Explicit protocol allowlist per request | not exposed — HTTP/1 is fixed to `Http1Only` | `CurlConfig::allowed_protocols` is caller-configurable |

In short: eggfetch costs binary size and a TLS stack but buys a
library-integrated timeout and no subprocess lifecycle; curl costs a
subprocess, an external binary dependency, and a hand-written duration
serializer, but keeps the consumer's own HTTP/TLS footprint near zero. This is
the decision [ADR-0003](../plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md)
leaves to the caller, and the reason
[`eggup-transport-footprint`](transport-footprint.md) exists.

## 13. Cross-references

- [overview.md](overview.md) — workspace index, dependency graph, cross-cutting invariants
- [acquisition.md](acquisition.md) — the seam contract, `FetchLimits::effective`, temp/promotion helpers, redaction
- [curl-adapter.md](curl-adapter.md) — the interchangeable external-process adapter
- [transport-footprint.md](transport-footprint.md) — what each transport stack costs a consumer
- [core-transaction.md](core-transaction.md) — where the bytes this adapter lands are verified
- [`crates/eggup-eggfetch/README.md`](../crates/eggup-eggfetch/README.md) and [`CHANGELOG.md`](../crates/eggup-eggfetch/CHANGELOG.md) — stated policy, M003/M004 correctives, and the audit fixes
- [ADR-0003](../plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md) — verification layers, transport neutrality, explicit adapter policy
- [ADR-0001](../plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md) — mechanism/policy ownership split
- [acquisition-transport roadmap](../plans/subsystems/acquisition-transport-roadmap.md) — M001–M010 status
- Closure records: [001](../plans/closure/acquisition-transport/001-status.md) · [002](../plans/closure/acquisition-transport/002-status.md) · [003](../plans/closure/acquisition-transport/003-status.md) (timeout/temp/redaction corrective) · [004](../plans/closure/acquisition-transport/004-status.md) (validated limits, promotion state) · [005](../plans/closure/acquisition-transport/005-status.md) · [006](../plans/closure/acquisition-transport/006-status.md) (Windows portability) · [007](../plans/closure/acquisition-transport/007-status.md) (conditionally closed: boundary safety) · [008](../plans/closure/acquisition-transport/008-status.md) (sub-second deadline truthfulness) · [009](../plans/closure/acquisition-transport/009-status.md)
- [tooling-governance.md](tooling-governance.md) — the verification gate and planning process

## Known doc/code drift

`curl-adapter.md` and `transport-footprint.md` are linked from
[overview.md](overview.md)'s deep-dive index but are owned by another work
stream; those two links are expected to resolve once those documents land.

- `architecture/eggfetch-adapter.md` (prior revision) cited 1180 source lines and
  described `block_on` as building a runtime per call with `.expect()` on
  failure. The current code caches one runtime per calling thread and returns
  `Unavailable` (`lib.rs:196-200`, `lib.rs:236-269`); the file is 1231 lines.
  The `expect` description is now contradicted by
  `CHANGELOG.md` (`Unreleased`) as well.
- The same prior revision wrote `TooLarge { limit: max.unwrap_or(0) }`. The
  mapper now takes a mandatory `u64` and there is no `Option` (`lib.rs:510-518`).
- `README.md:7` attributes the policy to `EggfetchTransport::strict`; the
  values are chosen by `EggfetchConfig::strict` and merely validated by the
  transport.
- The `EggfetchTransport` doc comment lists "checksum mismatch" among hard
  failures (`lib.rs:186-190`), but this adapter never computes a checksum.
  Read that phrase as a statement about the wider pipeline, not about this
  crate.
- TLS detail: `README.md:10-12` claims native roots **plus** a packaged WebPKI
  fallback. Only `tls-native-roots` is requested in `Cargo.toml`; the fallback
  is an upstream eggfetch default the crate does not itself request. Treat the
  fallback as a claim inherited from `eggfetch-core`, not as something this
  manifest pins.
