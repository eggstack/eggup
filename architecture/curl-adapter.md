# curl Adapter Deep Dive (`crates/eggup-curl`)

> Source: `crates/eggup-curl/src/lib.rs` (1672 lines, single file, 24 `#[test]`
> functions), `examples/curl_fetch.rs`, `README.md`, `CHANGELOG.md`, `Cargo.toml`.
> Dependency position: `eggup-curl -> eggup-acquisition` **only** — no HTTP client,
> no TLS stack, no `tokio`, no `eggup-core`
> (`Cargo.toml:23-24`). Published to crates.io at `0.1.2` with the requirement
> `eggup-acquisition = { version = "0.1.0", path = "../eggup-acquisition" }`;
> the caret was deliberately **not** tightened to `=0.1.2`
> (`plans/closure/acquisition-transport/009-status.md`, "Metadata decision").
> Crate attributes: `#![forbid(unsafe_code)]`, `#![deny(missing_docs)]`
> (`lib.rs:1-2`).

Terminology, cross-cutting invariants, and the composition graph used below are
defined in [overview.md](overview.md); this file is the review entry point for
this crate only and is not itself a contract.

## 1. Purpose and ownership boundary

`eggup-curl` turns one exact, caller-supplied URL into bytes by running the
**system `curl` binary as a direct child process** and typing the result as
`eggup-acquisition::FetchOutcome<MetadataBytes>` /
`FetchOutcome<ArtifactEvidence>`. It is the external-process half of a
two-adapter seam: the other half is the native HTTP adapter in
[eggfetch-adapter.md](eggfetch-adapter.md), and this crate has no compile-time or
source-level knowledge of it.

It **owns**:

- argv construction for a `curl` invocation that expresses the caller's URL and
  the adapter's bounded policy (`build_curl_args`, `lib.rs:512-557`).
- Executable selection, which is explicit and caller-driven
  (`CurlTransport::with_executable`, `lib.rs:177-189`).
- Process lifecycle: spawn, bounded polling, kill, reap, reader-thread joins
  (`run_curl_to_file`, `lib.rs:573-716`).
- HTTP status capture for the **same** request that produced the body, and the
  mapping of that status plus curl's exit code onto seam outcomes
  (`classify_curl_result`, `lib.rs:719-787`).
- Redirect, protocol, and proxy flags; `--disable` neutralization of the user's
  `~/.curlrc`.
- Exclusive owner-private temp staging and no-clobber promotion, using the
  seam's helpers rather than its own.

It **never decides** any of the following. Each is asserted by absence, not by
comment:

| Not decided here | Where it lives instead |
|---|---|
| Release, version, SemVer, mirror, or source selection | the caller; the seam explicitly refuses to learn it ([acquisition.md](acquisition.md) §1) |
| Transport fallback order or scope | `eggup-acquisition::ComposedTransport` + `CompositionPolicy` (`acquisition/src/lib.rs:932-1024`) |
| Destination path or install root | the caller; `fetch_artifact` only requires an existing real parent and an absent `dest` |
| Whether `NotFound` means anything | the consumer; the seam types it as data |
| Authenticity, signing, or publisher trust | nobody, by design ([ADR-0003](../plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md)) |
| Integrity verification | `eggup-core` SHA-256; this crate hashes nothing |
| Service quiesce/restore | `eggup-service` |
| Execution of downloaded bytes | never — bodies are written to a file and returned; nothing is invoked |

`CurlTransport`'s own rustdoc repeats the negative list: "No release, version,
fallback, destination, or service policy lives here. Only exact 404 becomes
`NotFound`; TLS/5xx/timeout are hard failures and missing/spawn failures become
`Unavailable` for safe composition" (`lib.rs:159-163`).

## 2. Position in the workspace

Compile-time graph edge: exactly one, `eggup-curl -> eggup-acquisition`
(`overview.md` dependency graph; `cargo tree -p eggup-curl` in the M005 closure
record shows no other node). Consequences a reviewer should hold onto:

- **`eggup-curl` cannot grow an HTTP/TLS dependency.** The crate's whole reason
  to exist is that a consumer can link a transport without an embedded TLS
  stack. Adding `reqwest`, `curl` bindings, or even `eggup-eggfetch` would
  dissolve the footprint difference that motivated the crate.
- **`eggup-curl` is not Eggpack-shaped and has no `eggup-core` edge.** It never
  sees `ArtifactSet`, `InstallPlan`, or ownership state. Anything that would
  require that knowledge is caller code.
- **It is one of two interchangeable adapters.** Choosing between
  `eggup-curl` and `eggup-eggfetch` is a footprint and trust decision made by the
  caller; the seam, not the adapter, composes them. See
  [transport-footprint.md](transport-footprint.md) for the measurement harness
  (`eggup-transport-footprint`, `publish = false`, three `required-features`
  binaries).
- **Footprint implication of choosing it.** From the M005 closure record
  (`plans/closure/acquisition-transport/005-status.md:36`, `:117`): `curl_only`
  release binary 417 KiB (346 KiB stripped) versus `eggfetch_only` 1.9 MiB
  (1.6 MiB stripped). In exchange, the consumer's binary depends on a
  `curl` binary existing on the host at a path the caller chose, and inherits
  whatever that `curl` build supports (proxy protocols, TLS backend, HTTP
  versions). The adapter constrains what it *asks for*; it cannot constrain what
  the executable *is*.
- **`examples/curl_fetch.rs` is a footprint fixture, not a demo fetch.** It
  performs no network I/O; it constructs the adapter, then prints the executable
  path, `request.redacted()`, and `FetchLimits::default()`
  (`examples/curl_fetch.rs:11-18`). The crate ships no example that reaches the
  network.

## 3. Public surface

Every exported item, with its role. Nothing else is reachable from outside the
crate; all remaining functions are private (`discover_in`, `bound`,
`duration_decimal_seconds`, `proxy_env_snapshot`, `redirect_protocols`,
`build_curl_args`, `proxy_kind`, `run_curl_to_file`, `classify_curl_result`) or
private types (`CurlOutcome`, `ProxyKind`, the two local `Guard` structs).

| Item | Kind | Role | Location |
|---|---|---|---|
| `DEFAULT_CONNECT_TIMEOUT` | const | Connect deadline ceiling, `Duration::from_secs(10)` | `lib.rs:24` |
| `DEFAULT_TOTAL_TIMEOUT` | const | Total wall-clock ceiling, `Duration::from_secs(120)` | `lib.rs:26` |
| `DEFAULT_MAX_REDIRECTS` | const | Redirect bound when following, `10` | `lib.rs:28` |
| `CurlProxy` | enum | Explicit proxy decision: `Disabled` (default) / `FromEnvironment` / `Custom(Vec<(String,String)>)` | `lib.rs:36-45` |
| `CurlConfig` | struct | The single policy point: ceilings, redirect bound + follow flag, protocol allowlist, proxy | `lib.rs:49-67` |
| `CurlConfig::strict()` | fn | Strict default; identical to `Default::default()` | `lib.rs:84-86` |
| `CurlConfig::timeouts(connect, total)` | fn | Builder for both adapter ceilings | `lib.rs:92-96` |
| `CurlConfig::max_redirects(n)` | fn | Builder for the redirect bound | `lib.rs:99-102` |
| `CurlConfig::follow_redirects(bool)` | fn | Builder for whether `--location` is passed | `lib.rs:105-108` |
| `CurlConfig::allowed_protocols(Vec<String>)` | fn | Builder for the `--proto` / `--proto-redir` allowlist | `lib.rs:111-114` |
| `CurlConfig::proxy(CurlProxy)` | fn | Builder for the proxy decision | `lib.rs:117-120` |
| `CurlConfig::effective_timeouts(FetchLimits)` | fn | `min(request, adapter)` per phase -> `(connect, total)` | `lib.rs:123-125` |
| `CurlTransport` | struct | The adapter: private `executable: PathBuf` + `config: CurlConfig` | `lib.rs:165-168` |
| `CurlTransport::with_executable(path, config)` | fn | Only constructor; validates config and non-empty path, does **not** touch the filesystem | `lib.rs:177-189` |
| `CurlTransport::executable()` | fn | Accessor for the configured path | `lib.rs:192-194` |
| `CurlTransport::config()` | fn | Accessor for the active config | `lib.rs:197-199` |
| `AcquisitionTransport for CurlTransport::fetch_metadata` | trait method | Bounded small body into memory | `lib.rs:203-270` |
| `AcquisitionTransport for CurlTransport::fetch_artifact` | trait method | Streamed body into `dest` via no-clobber promotion | `lib.rs:272-363` |
| `discover_curl_executable()` | fn | **Opt-in** `PATH` search returning the first regular `curl` file, or `Unavailable` | `lib.rs:378-380` |

API-surface notes for review:

- `CurlConfig` and `CurlProxy` derive `Debug, Clone` (+ `Default` for
  `CurlProxy`) but **not** `PartialEq`, so a caller cannot compare two configs
  directly. There is no public `validate()`; validation runs inside
  `with_executable` and again at every fetch entry point.
- All `CurlConfig` fields are public for 0.1.x source compatibility, exactly like
  `FetchLimits`. That is why `config.validate()` runs at construction *and*
  again per fetch (`lib.rs:210`, `lib.rs:280`) — a directly constructed struct
  literal cannot bypass it.
- `with_executable` performs **no** `fs` check on the path. Existence,
  executability, and spawnability are deliberately deferred to the request so
  they classify as `Unavailable` rather than as a construction error
  (`lib.rs:172-188`).

## 4. The policy point: `CurlConfig`

```rust
pub struct CurlConfig {
    pub connect_timeout: Duration,
    pub total_timeout: Duration,
    pub max_redirects: usize,
    pub follow_redirects: bool,
    pub allowed_protocols: Vec<String>,
    pub proxy: CurlProxy,
}
```
(`lib.rs:49-67`)

| Field | Default (`Default` / `strict()`) | Meaning and enforcement |
|---|---|---|
| `connect_timeout` | `DEFAULT_CONNECT_TIMEOUT` = 10 s | Ceiling only. Serialized to `--connect-timeout` and clamped against the caller's value (§4.3). Enforced by curl; there is no parent-side connect deadline. |
| `total_timeout` | `DEFAULT_TOTAL_TIMEOUT` = 120 s | Ceiling only. Serialized to `--max-time` **and** independently enforced by the parent's wall deadline `start + eff_total` (`lib.rs:588-589`, `:668-675`). |
| `max_redirects` | `DEFAULT_MAX_REDIRECTS` = 10 | Passed as `--max-redirs` only when `follow_redirects` is true. `validate()` rejects `> 50` (`lib.rs:138-142`). |
| `follow_redirects` | `true` | Decides whether `--location` is passed at all (`lib.rs:526-530`). When false, a 3xx response is a hard `Transport` failure, not `NotFound`. |
| `allowed_protocols` | `vec!["http", "https"]` | Bounds the initial request via `--proto =http,https` and the redirect targets via a possibly-narrowed `--proto-redir` (`lib.rs:539-545`). `validate()` requires 1..=8 entries, each 1..=16 ASCII-alphanumeric bytes (`lib.rs:143-154`). |
| `proxy` | `CurlProxy::Disabled` | `Disabled` adds `--noproxy *` and contributes no environment; `FromEnvironment` snapshots 8 proxy keys; `Custom` injects caller pairs verbatim (§4.4). |

`strict()` is a literal alias for `default()` (`lib.rs:84-86`). There is no
"relaxed" constructor and no hidden default beyond the table above.

### 4.1 Validation (`CurlConfig::validate`, `lib.rs:127-156`)

Fails closed with `AcquisitionError::InvalidInput` on: zero `connect_timeout` or
`total_timeout`; `connect_timeout > total_timeout`; `max_redirects > 50`; an
empty or >8-element protocol list; and any protocol name that is empty, longer
than 16 bytes, or not ASCII-alphanumeric. The protocol-name check is what makes
`--proto`/`--proto-redir` argument injection structurally impossible (§7).

### 4.2 Builder methods

`timeouts`, `max_redirects`, `follow_redirects`, `allowed_protocols`, `proxy`
(`lib.rs:92-120`) are all consuming `self -> Self` builders that only assign
fields. They perform **no** validation; the value is checked at
`with_executable` and at every fetch. A builder chain that produces an invalid
config therefore fails at the constructor, not at the builder call.

### 4.3 `effective_timeouts` and the `min(request, adapter)` clamp

`effective_timeouts` is a one-line delegation to the seam
(`lib.rs:123-125`):

```text
effective connect = min(request.connect_timeout, self.connect_timeout)
effective total   = min(request.total_timeout,   self.total_timeout)
```

which is `FetchLimits::effective` (`acquisition/src/lib.rs:143-151`). Both fetch
methods call it after `limits.validate()?`, and both hand the result to the
`run_curl_to_file` call (`lib.rs:214`, `:299`).

Worked example 1 — adapter looser than caller in connect, tighter in nothing:

| Input | connect | total |
|---|---|---|
| `CurlConfig::strict()` ceiling | 10 s | 120 s |
| caller `FetchLimits` | 30 s | 45 s |
| **effective** (`min` per phase) | **10 s** | **45 s** |

Resulting argv fragments: `--connect-timeout 10 --max-time 45`, plus a parent
wall deadline of 45 s measured from just before `spawn`. The adapter tightened
connect; it did not extend total.

Worked example 2 — coincident ceilings (the case M009 shipped a fix for):
caller `(1 s, 1 s)` against `strict()` gives effective `(1 s, 1 s)`. Because the
two are equal, an elapsed-time test against the connect ceiling would hold for
*every* curl exit 28, so the classifier reports the weaker, always-true `"total"`
phase instead of mislabelling a total timeout as a connect timeout
(`lib.rs:740-752`, test `exit_28_is_labelled_total_when_connect_and_total_
ceilings_coincide` at `lib.rs:803`). See `CHANGELOG.md` "Fixed: a mislabelled
timeout phase".

Serialization for the two flags is `duration_decimal_seconds` (`lib.rs:432-456`):
microsecond-resolution decimal seconds with a `.` separator and trailing
fractional zeros stripped (`100 ms -> "0.1"`, `250 ms -> "0.25"`,
`1.5 s -> "1.5"`, `2 s -> "2"`). Truncation to whole microseconds can only
decrease the value, so curl never receives a longer deadline than the caller
asked for. Zero and sub-microsecond positive durations are rejected with
`InvalidInput` rather than silently rounded up to a second — the deliberate
M008 behavior, pinned by `duration_decimal_seconds_serializes_sub_second_inputs_
truthfully` (`lib.rs:1228`) and `build_curl_args_passes_decimal_deadlines_never_
above_ceiling` (`lib.rs:1316`).

### 4.4 `CurlProxy` and environment handling

`proxy_env_snapshot` (`lib.rs:458-479`) turns the decision into an explicit
pair list: `Disabled` yields an empty list; `FromEnvironment` reads exactly
`HTTP_PROXY`, `http_proxy`, `HTTPS_PROXY`, `https_proxy`, `ALL_PROXY`,
`all_proxy`, `NO_PROXY`, `no_proxy` at call time; `Custom` clones the caller's
pairs. That list is the **only** thing added to the child environment, which is
otherwise cleared (§7). `ProxyKind` (`lib.rs:560-570`) is just the boolean
"should `--noproxy *` be passed" projection of the same decision.

## 5. Executable selection and hardening

Two selection paths, one of them explicitly opt-in:

- `CurlTransport::with_executable(path, config)` (`lib.rs:177-189`) — the
  primary, and only, way to construct the adapter. The caller supplies the exact
  path. An empty path is `InvalidInput`; an invalid config is `InvalidInput`; a
  path that does not exist is **not** detected here.
- `discover_curl_executable()` (`lib.rs:378-380`) — a free function, not an
  implicit default. It calls `discover_in(std::env::var_os("PATH"))`
  (`lib.rs:382-406`), which splits `PATH`, skips empty entries, and returns the
  first candidate whose `symlink_metadata` reports a regular file (`curl`, plus
  `curl.exe` on Windows). Unset `PATH` or no hit yields
  `AcquisitionError::Unavailable`, never `Transport` and never a `panic`.

Why discovery is opt-in, in review terms: `PATH` is ambient, mutable by any
process the user runs, and first-match-wins. An implicit default would make the
transport binary a function of the caller's environment, and a substituted
earlier `PATH` entry would be invoked with the adapter's full argv — arbitrary
local code execution with the updater's privileges. Requiring the caller to pass
a path, or to call `discover_curl_executable()` and then pass what it got,
keeps the selection decision visible at the call site and lets a caller pin
`/usr/bin/curl` and assert it is a regular file first.

`--disable` is the first argument of every invocation (`lib.rs:519-525`), with
the in-code rationale "Ignore ~/.curlrc so ambient config cannot smuggle
proxy/auth policy". This matters because curl reads `~/.curlrc` (and
`CURL_HOME`) by default, and a user config could otherwise inject `--proxy`,
`--user`, `--insecure`, an extra `--config`, or a redirect policy the adapter
never approved. Defense in depth, not the only layer: the child environment is
cleared (§7), so `HOME`/`CURL_HOME` are absent from the child and curl cannot
locate the file even if `--disable` were dropped.

Review check: `--disable` must stay the first argument, and no code path may add
`--config`, `--rcfile`, `--user`, `--insecure`, or `--proxy` outside the
`CurlProxy` policy point.

## 6. Seam implementation

`impl AcquisitionTransport for CurlTransport` (`lib.rs:202-364`) implements the
two methods of the seam trait
(`acquisition/src/lib.rs:578-606`). Where each seam guarantee is honored:

| Seam guarantee | Honored in `eggup-curl` at |
|---|---|
| Exact caller URL, no discovery/version/mirror logic | `let url = request.url().to_string()` used verbatim in argv (`lib.rs:584`); nothing else about the URL is inspected except the scheme for `--proto-redir` narrowing |
| `limits.validate()` before any I/O, every call | `lib.rs:209` and `lib.rs:279`, ahead of temp creation, spawn, or filesystem access |
| Re-check `CurlFlag` at the boundary | `if cancel.is_cancelled()` at `lib.rs:211-213` and `lib.rs:281-283`, before staging |
| Effective deadlines are `min(request, adapter)` | `effective_timeouts` at `lib.rs:214` / `:299`, and `POLL_INTERVAL`/parent deadline enforcement in the poll loop |
| `dest` parent must already exist as a real directory | `symlink_metadata(parent)`, rejected unless `is_dir() && !file_type().is_symlink()` (`lib.rs:287-293`) |
| `dest` must be absent; never overwrite | Fast-fail `symlink_metadata(dest).is_ok()` -> `InvalidInput` (`lib.rs:294-298`); the authoritative race-safe guard is the seam's `__promote_no_clobber` (`lib.rs:348`) |
| Stage to an exclusively-created owner-private temp | `__acquire_exclusive_temp(parent, "eggup-curl")` (artifact, `lib.rs:301`) or `__acquire_exclusive_temp(temp_dir(), "eggup-curl-meta")` (metadata, `lib.rs:220-221`) |
| Promote no-clobber only on full success | `__promote_no_clobber` only on the `CurlOutcome::Success` + size-ok + not-cancelled path (`lib.rs:334-360`) |
| Partial output cleans only the owned temp | Local `Guard` calling `__remove_owned_temp` on every non-committed path (`lib.rs:222-236`, `:302-316`); no prefix scan, no sibling deletion |
| `NotFound` is data, never a fallback trigger | `CurlOutcome::NotFound` -> `Ok(FetchOutcome::NotFound)` only (`lib.rs:264-268`, `:329-333`); the adapter contains no fallback branch |
| Downloads are data, never executed | The only child process is `curl`; body bytes are written to a `File` and returned |
| `MetadataBytes`/`ArtifactEvidence` are adapter-constructed | `__adapter_metadata(bytes)` (`lib.rs:261`) and `__adapter_artifact(size)` (`lib.rs:352`) — the seam has no public constructor for either |

Two seam-adjacent facts worth confirming on review:

- The metadata path creates its temp inside `std::env::temp_dir()` and calls
  `let _ = fs::create_dir_all(&parent);` first (`lib.rs:218-219`). This is the
  only directory the crate ever creates, and it exists only to guarantee the
  system temp directory is present; the artifact path never creates the
  destination parent.
- The metadata path reads the whole body back with `fs::read` and *then* checks
  `bytes.len() > max` (`lib.rs:253-259`). Peak memory is still bounded, because
  the reader thread stops at `max + 1` bytes and curl is additionally given
  `--max-filesize` (§7), so a file larger than `max` cannot reach this read.

## 7. Process lifecycle

### 7.1 argv construction

`build_curl_args` (`lib.rs:512-557`) produces, in order:

```text
--disable --silent --show-error --no-progress-meter
[--location --max-redirs <max_redirects>]          if follow_redirects
--connect-timeout <decimal-seconds>
--max-time         <decimal-seconds>
--max-filesize     <max_bytes>
[--proto =a,b --proto-redir =x,y]                   if allowed_protocols non-empty
[--noproxy *]                                       if proxy == Disabled
--output -
--write-out %{stderr}%{http_code}
--
<exact url>
```

Notes a reviewer needs:

- **No shell anywhere.** `Command::new(executable).args(&args)` passes an argv
  array to `execvp`; nothing is concatenated into a command string, and no
  `sh -c` is involved. The workspace-level "no shell interpolation" invariant
  holds here (`overview.md`, cross-cutting invariants).
- **The `--` terminator** (`lib.rs:554`) prevents the URL from being parsed as
  an option. Combined with `validate()`'s ASCII-alphanumeric protocol-name rule,
  a caller cannot inject an extra flag through `allowed_protocols`.
- **`--output -`** sends the body to stdout. The adapter never hands curl a
  pathname: the temp file was created by the adapter with `create_new`, and the
  adapter keeps writing through its own retained descriptor. This is the M007
  fix for curl reopening an Eggup-exclusive temp by name
  (`lib.rs:606-607`, `CHANGELOG.md` M007 entry,
  `plans/closure/acquisition-transport/007-status.md`).
- **`--write-out %{stderr}%{http_code}`** routes the status to stderr, keeping
  the body channel on stdout uncontaminated. The status therefore comes from
  the *same* transfer as the bytes; there is no second probe and no re-request
  whose answer could disagree with the body.
- **`--max-filesize <max_bytes>`** gives curl its own bound, so a large response
  can be refused by curl itself (exit 63) in addition to the adapter's counter.
- **`--proto-redir` narrowing** (`redirect_protocols`, `lib.rs:489-509`): if the
  request URL's scheme is `https` (case-insensitive), `http` is filtered out of
  the redirect allowlist, so a `302 Location: http://…` is refused rather than
  followed in cleartext. A cleartext request keeps the full list. If narrowing
  would empty the list, the un-narrowed list is emitted so `--proto-redir` is
  never an empty argument — in that configuration `--proto` already refuses the
  initial `https` request. This matches `RedirectPolicy::strict` in
  `eggup-eggfetch` ([eggfetch-adapter.md](eggfetch-adapter.md) §4) and is
  documented as a fixed defect in `CHANGELOG.md` "Fixed: an `https` request
  could follow a redirect down to `http`".
- **No `--fail`.** curl exits 0 on 404 and 500, which is precisely why the
  captured `%{http_code}` is the sole status evidence and why the classifier
  (§8) must handle every status itself.

### 7.2 spawn

`run_curl_to_file` (`lib.rs:573-716`) does, in order: take `start = Instant::now()`
and `deadline = start + eff_total` (`:588-589`); clone the caller's `File` handle
(`:590-592`); build the `Command` with `stdin(Stdio::null())`,
`stdout(Stdio::piped())`, `stderr(Stdio::piped())`, and `env_clear()`; re-add
only the proxy pairs; then `spawn()` (`:594-605`).

Consequences of `env_clear()` worth stating explicitly: the child receives **no**
`HOME`, `PATH`, `TMPDIR`, `SSL_CERT_FILE`, `CURL_CA_BUNDLE`, or
`SSL_CERT_DIR`. Proxy decision is the only environment this adapter expresses.
A consumer whose TLS trust depends on `CURL_CA_BUNDLE`/`SSL_CERT_FILE` in the
parent environment will not have that honored; it depends on the discovered
executable's compiled-in CA store.

Any `spawn()` error — `ENOENT`, `EACCES`, `ENOTDIR`, and equally a real
resource failure — is mapped to
`AcquisitionError::Unavailable("curl spawn failed for {redacted}")`
(`lib.rs:603-605`). This is the single most important classification decision in
the crate: `Unavailable` is the only hard failure that `ComposedTransport` will
fall back on under the default `CompositionPolicy::UnavailableOnly`
(`acquisition/src/lib.rs:987-995`).

### 7.3 pipes, threads, and the poll loop

Two reader threads are started (`:612-646`):

- **Body reader** owns the piped stdout plus the *cloned* `File`, and reads
  through `stdout.take(max_bytes.saturating_add(1))` (`:614`). It accumulates
  `written`, sets an `AtomicBool` overflow flag when `written > max_bytes`
  (`:623-625`), and `write_all`s each chunk to the retained descriptor. When the
  `take` limit is reached, `read` returns `Ok(0)` and the loop ends, so the
  reader always terminates on its own.
- **Status reader** owns piped stderr, reads to EOF, and retains at most
  `MAX_WRITE_OUT_BYTES` = 64 bytes (`:29-30`, `:632-646`). It keeps *draining*
  after the bound is reached (`n.min(room)` with `room == 0`), so a chatty curl
  can never block on a full stderr pipe. Raw stderr text is never surfaced in an
  error message.

The parent then polls every `POLL_INTERVAL` = 5 ms (`:31-32`, `:648-689`), in
this order per iteration:

1. overflow flag set -> kill, wait, join both readers, remove owned temp, return
   `TooLarge { limit: max_bytes }`;
2. `cancel.is_cancelled()` -> same teardown, return `Cancelled`;
3. `try_wait()`:
   - `Ok(Some(status))` -> break out with `status.code()`;
   - `Ok(None)` and `Instant::now() >= deadline` -> teardown, return
     `Timeout { phase: "total" }`;
   - `Ok(None)` otherwise -> `thread::sleep(POLL_INTERVAL)`;
   - `Err(e)` -> teardown, return `Io("waiting for curl: {e}")`.

The teardown sequence is identical in all four branches: `child.kill()`,
`child.wait()`, `body_reader.join()`, `status_reader.join()`,
`__remove_owned_temp(output)`, then return. On the normal path the child was
already reaped by `try_wait`, and a second `let _ = child.wait();` at `:690` is
harmless belt-and-braces.

### 7.4 What a reviewer must confirm here

- **No zombies.** The child is always `wait`ed, on every exit path, including
  the four early returns. Reader threads are always joined before returning, so
  no thread outlives the call and no `File` clone is dropped mid-write.
- **No orphans by design, but no process-group kill.** `kill()` targets the
  direct child only; there is no `setpgid`/process-group teardown. Curl invoked
  this way does not fork helpers, and `--disable` plus the absence of
  `--config` removes the config-driven ways to make it do so, so the exposure is
  small — but it is not zero, and a future flag that spawns a helper would need
  a process-group kill to stay correct.
- **`wait()` after a failed `kill()` is unbounded.** Both `let _ = child.kill()`
  and `let _ = child.wait()` discard errors (`:650-651`, `:658-659`, `:669-670`,
  `:679-680`). If `kill()` were ever to fail *and* the child kept running,
  `wait()` would block with no deadline. In practice a `kill()` failure on a
  child we spawned means the child is already gone, and `wait()` then returns
  immediately; this is a documented-shape assumption, not an enforced one.
- **Back-pressure is handled by kill, not by draining.** When the body reader
  hits the `max + 1` limit it stops reading, so curl can block writing to a full
  stdout pipe. The overflow flag is set before the reader exits, and the parent's
  next poll iteration kills the child, so the pipe is never a permanent
  deadlock. The window is bounded by `POLL_INTERVAL`.
- **The total deadline has two enforcers, connect has one.** `--max-time` and
  the parent wall deadline both bound the total phase; only curl's
  `--connect-timeout` bounds connect. Connect-phase *attribution* therefore
  depends on curl's exit 28 plus the elapsed-time test in §8, not on an
  independent parent-side connect deadline.
- **The exact URL is in `argv`.** The verbatim URL — including any userinfo or
  query token the seam's `AcquisitionRequest` permits — is present in the
  child's command line for the life of the process and is therefore readable by
  other local users through process inspection. `redact_url` protects
  *diagnostics*, not the process table. This is inherent to an argv-based
  adapter and is a fact a reviewer of any credential-bearing URL should weigh.

## 8. Status classification and outcome mapping

`classify_curl_result` (`lib.rs:719-787`) takes the child's exit code, the
bounded stderr bytes, the temp path, the redacted URL, and the timing inputs,
and returns `CurlOutcome::Success | CurlOutcome::NotFound` or an
`AcquisitionError`. Exit code is consulted **first**, and status second.

| Condition | Result | Location |
|---|---|---|
| `Some(28)` (curl operation timeout) | `Timeout { phase }`; `"connect"` only when `eff_connect < eff_total` **and** `elapsed <= eff_connect + POLL_INTERVAL + 5 ms`; otherwise `"total"` | `lib.rs:740-752` |
| `Some(63)` (`--max-filesize` exceeded) | remove owned temp, `TooLarge { limit: max_bytes }` | `lib.rs:754-757` |
| `Some(0)` | fall through to status classification | `lib.rs:758` |
| Any other `Some(_)`, or `None` (killed by signal) | remove owned temp, `Transport("curl transfer failed (exit N) for {redacted}")` | `lib.rs:759-769` |
| `Some(200..=299)` with exit 0 | `CurlOutcome::Success` | `lib.rs:772` |
| `Some(404)` with exit 0 | `CurlOutcome::NotFound` | `lib.rs:773` |
| Any other status (3xx not followed, 4xx, 5xx) | remove owned temp, `Transport("HTTP {status} from {redacted}")` | `lib.rs:774-779` |
| Unparseable/absent status with exit 0 | remove owned temp, `Transport("curl status capture failed for {redacted}")` | `lib.rs:780-785` |

Then, before the outcome is trusted (`lib.rs:691-705`): the body reader is
joined, `bytes_written > max_bytes` is re-checked (a second, race-closing
overflow check), the parent's handle is flushed, and the status reader is
joined.

Mapping onto the seam's categories, in the vocabulary of
[acquisition.md](acquisition.md):

| Situation | Seam error | Fallback input? |
|---|---|---|
| `PATH` unset / no `curl` found | `Unavailable` | yes, under both policies |
| Executable missing, non-executable, or spawn failure | `Unavailable` | yes, under both policies |
| TLS handshake, DNS, connect, or proxy failure (curl non-zero exit) | `Transport` | only under `UnavailableOrTransport` |
| Truncated/partial body (non-zero exit) | `Transport` | only under `UnavailableOrTransport` |
| Connect or total deadline | `Timeout { phase }` | only under `UnavailableOrTransport` |
| Response over the caller's byte bound | `TooLarge { limit }` | no |
| `CancelFlag` observed | `Cancelled` | no |
| Owned-file or `wait()`/promotion failure | `Io` | no |
| Bad caller `FetchLimits` or bad `CurlConfig` | `InvalidInput` | no |
| Exact `404` | `Ok(FetchOutcome::NotFound)` | no — terminal data |

Only exact `404` becomes `NotFound`; there is no `== 404`-range widening and no
HEAD-probe fallback. Because the status comes from the same request, a body is
never promoted under a status that belongs to a different request. A non-zero
exit is treated as "the transfer did not complete" **even when curl still
printed an HTTP code** — the partial body is removed and never promoted
(`lib.rs:760-768`), which is what `truncated_transfer_is_hard_failure_without_
promotion` (`lib.rs:1121`) pins.

## 9. Control flow, step by step

### 9.1 `fetch_metadata(request, limits, cancel)` (`lib.rs:203-270`)

1. `limits.validate()?`; `self.config.validate()?`; early `Cancelled`.
2. `effective_timeouts(limits)`; `max = limits.max_metadata_bytes`.
3. `fs::create_dir_all(temp_dir())` (result ignored), then
   `__acquire_exclusive_temp(temp_dir(), "eggup-curl-meta")` -> owned `(File,
   PathBuf)`.
4. Build the local `Guard` (`lib.rs:222-236`): on drop, unless disarmed,
   `__remove_owned_temp(path)`.
5. `run_curl_to_file(...)` with `max as u64`; a `?` here propagates and the
   `Guard` cleans up.
6. `Success`: re-check cancel; `fs::read(&tmp)` -> `__remove_owned_temp` ->
   disarm; `bytes.len() > max` -> `TooLarge`; else
   `Ok(FetchOutcome::Success(__adapter_metadata(bytes)))`.
7. `NotFound`: remove temp, disarm, `Ok(FetchOutcome::NotFound)`.

### 9.2 `fetch_artifact(request, dest, limits, cancel)` (`lib.rs:272-363`)

1. `limits.validate()?`; `self.config.validate()?`; early `Cancelled`.
2. `dest.parent()` must exist and be a real directory, not a symlink
   (`lib.rs:284-293`).
3. Fast-fail if `dest` already exists (`lib.rs:294-298`).
4. `effective_timeouts(limits)`; `max_artifact = limits.max_artifact_bytes`.
5. `__acquire_exclusive_temp(parent, "eggup-curl")` — created in the **destination
   parent**, so promotion is same-filesystem (`lib.rs:301`).
6. Build the `Guard` (`lib.rs:302-316`).
7. `run_curl_to_file(...)`; `?` propagates and the `Guard` cleans up.
8. `NotFound`: remove temp, disarm, `Ok(FetchOutcome::NotFound)`. No `dest`
   is created.
9. `Success`: re-check cancel; `fs::metadata(&tmp).len()`; `size > max_artifact`
   -> remove temp, disarm, `TooLarge`.
10. `__promote_no_clobber(&tmp, dest)`:
    - `Ok` -> disarm, `Ok(FetchOutcome::Success(__adapter_artifact(size)))`;
    - `Err` -> remove temp, disarm, propagate (a raced-in `dest` surfaces as
      `InvalidInput` from the seam, with the foreign file preserved).

The seam's promotion is `hard_link`, so the committed artifact inherits the
temp's `0600` Unix mode and can never be world-readable; the temp name is then
unlinked best-effort after commit, and a failure there cannot retroactively turn
a committed destination into an error
(`acquisition/src/lib.rs:519-556`). `artifact_temp_is_owner_private_and_no_
clobber` (`lib.rs:1562`) asserts both the `0600` mode and that a pre-existing
destination keeps its `FOREIGN` contents.

## 10. Errors and redaction

Every message this crate authors is category text plus the seam's redacted URL,
truncated by `bound` to at most 512 bytes **at a UTF-8 character boundary**
(`lib.rs:408-417`; the boundary property is pinned by
`diagnostic_bound_never_splits_multibyte_code_points`, `lib.rs:848`).

The redacted string is `request.redacted()` -> `redact_url`
(`acquisition/src/lib.rs:350-379`), computed once per fetch at `lib.rs:585` and
passed down. It strips `user:password@`, replaces the fragment with
`#<redacted>`, replaces the query with `?<redacted>`, and truncates overlong
URLs at 256 bytes with an ellipsis.

| Message | Contains |
|---|---|
| `Unavailable("curl spawn failed for {redacted}")` | redacted URL |
| `Unavailable("curl not found on PATH")` / `"PATH is unset"` | nothing |
| `Transport("curl transfer failed (exit {code}) for {redacted}")` | numeric exit code + redacted URL |
| `Transport("HTTP {status} from {redacted}")` | numeric status + redacted URL |
| `Transport("curl status capture failed for {redacted}")` | redacted URL |
| `Timeout { phase }`, `TooLarge { limit }`, `Cancelled` | numeric category only |
| `Io("...: {io error}")` | OS error text for owned filesystem/`wait` operations only |
| `InvalidInput("...")` | fixed policy-violation text, no caller data |

Deliberately excluded from every message: curl's own stderr (the bounded 64-byte
capture is read and parsed for the status, then dropped), the command line, the
argv, proxy environment values, the exact URL, and any body bytes.
`credential_material_is_redacted` (`lib.rs:1610`) asserts that userinfo, query,
and fragment sentinels never appear in the `Display` output of a failing fetch.

One asymmetry to keep in mind when reading logs: the redaction covers Eggup's
diagnostics, while the child process's command line carries the exact URL
(§7.4).

## 11. Invariants and failure modes

| Condition | Observable outcome | Enforced at |
|---|---|---|
| Executable path empty | `InvalidInput("curl executable path must not be empty")` | `lib.rs:183-187` |
| Config invalid (zero/out-of-order timeouts, redirects > 50, protocol list 0 or > 8, non-alphanumeric protocol name) | `InvalidInput` at construction, and again at each fetch | `lib.rs:127-156`, called at `:181`, `:210`, `:280` |
| Caller limits invalid (zero bound, zero timeout, `connect > total`) | `InvalidInput` before staging, spawn, or any filesystem write | `lib.rs:209`, `:279` |
| `dest` has no parent / parent missing / parent is a symlink | `InvalidInput` (no parent) or `Io` (unreadable parent) | `lib.rs:284-293` |
| `dest` already exists | `InvalidInput("artifact destination already exists; refusing to overwrite")`; foreign content preserved | `lib.rs:294-298`; race-safe re-check in `__promote_no_clobber` |
| Cancel already requested | `Cancelled`, no temp left behind | `lib.rs:211-213`, `:281-283`; test `lib.rs:1431` |
| Executable missing / not executable / spawn failure | `Unavailable`; no `dest`; temp cleaned by `Guard` | `lib.rs:603-605`; test `lib.rs:1008` |
| curl exits non-zero (TLS, DNS, connect, proxy, partial body, signal) | `Transport("curl transfer failed (exit N) …")`; temp removed; never promoted | `lib.rs:759-769`; tests `lib.rs:1106`, `:1121` |
| curl exit 28 (operation timeout) | `Timeout { phase: "connect" \| "total" }` | `lib.rs:740-752`; tests `lib.rs:803`, `:1182` |
| Parent wall deadline reached | kill + reap + join, `Timeout { phase: "total" }` | `lib.rs:668-675`; test `lib.rs:1393` |
| Response exceeds `max_metadata_bytes` / `max_artifact_bytes` | `TooLarge { limit }` from three independent places (curl `--max-filesize` exit 63, the reader's `overflow` flag, the post-join byte count); `dest` never created | `lib.rs:754-757`, `:623-625`, `:649-655`, `:695-698`, `:257-259`, `:341-347` |
| Cancel observed mid-transfer | kill + reap + join, `Cancelled`, owned temp removed, `dest` untouched | `lib.rs:657-664`; test `lib.rs:1453` |
| `try_wait` error | kill + reap + join, `Io("waiting for curl: …")` | `lib.rs:678-687` |
| Owned-file write/flush failure, reader thread panic | `Io`; child already reaped or reaped by the teardown | `lib.rs:691-694`, `:699-705` |
| Status unparseable with exit 0 | `Transport("curl status capture failed …")` — fail closed, never a guessed `NotFound` | `lib.rs:780-785` |
| Sub-microsecond positive duration | `InvalidInput` rather than a widened second | `lib.rs:439-444`; test `lib.rs:1228` |
| Reader thread blocked on a full stdout pipe | Resolved by the overflow flag plus parent kill within one `POLL_INTERVAL` | `lib.rs:614`, `:623-625`, `:649-656` |

### Review checklist

- [ ] `CurlConfig` is still the only policy surface; no new ambient default crept in.
- [ ] `--disable` is still the first argv element and no `--config`/`--user`/`--insecure`/`--proxy` is added outside `CurlProxy`.
- [ ] Executable is still caller-supplied; `discover_curl_executable()` is still opt-in and still returns `Unavailable`, not `Transport`, on failure.
- [ ] Both fetch methods still call `limits.validate()` and `config.validate()` before any I/O.
- [ ] Effective deadlines still resolve `min(request, adapter)` and are still serialized at microsecond precision without rounding up.
- [ ] Coincident connect/total ceilings still report phase `"total"`, never `"connect"`.
- [ ] `--output -` is still in argv and the body is still written through the retained exclusive `File` clone; curl is never given the temp pathname.
- [ ] Every teardown path still does kill -> wait -> join readers -> remove owned temp.
- [ ] Only exact `404` maps to `NotFound`; non-zero exit still wins over a printed status code; status still comes from the same request as the body.
- [ ] `--proto-redir` is still narrowed per request for `https` URLs.
- [ ] Promotion is still `__promote_no_clobber` from a temp in the destination parent; a raced-in `dest` still fails without overwrite.
- [ ] Diagnostics still use `request.redacted()` + numeric category only, `bound()` at a UTF-8 boundary; raw curl stderr still never surfaces.
- [ ] No authenticity, signature, or publisher-trust claim has appeared in any doc or error string.

## 12. Testing approach

24 `#[test]` functions, all in-crate (`lib.rs:789-1672`). No test reaches a
public network: the two harnesses are a loopback `TcpListener`
(`serve_once`, `lib.rs:891-929`) and a fake `curl` shell script
(`fake_curl_script`, `lib.rs:955-983`) that records its own `"$@"` and `env`
into files so argv and environment can be asserted directly. `real_curl()` /
`transport()` return `Option` and every network test begins with
`let Some(t) = transport() else { return };`, so a host without `curl` skips
rather than fails.

**What makes process-based behavior testable here:**

| Technique | Enables |
|---|---|
| `CurlTransport::with_executable(&script, ...)` against a recording fake | Asserting exact argv (`--noproxy`, `--disable`, `--output -`, no temp pathname) and the child environment, without a network |
| Fake script emits a chosen body on stdout, chosen code on stderr, chosen exit | Covering exit 28 / 35 / 7 / 0 and status 200/302/404/500 without an HTTP server |
| `discover_in(Some(path))` takes the `PATH` value as an argument | Testing the miss path without mutating process `PATH` from parallel tests |
| `classify_curl_result` takes `start: Instant` and the temp path as parameters | Unit-testing exit-28 phase labelling with synthetic elapsed time and no process |
| `duration_decimal_seconds` / `build_curl_args` are pure | Pinning serialization and flag policy without spawning anything |
| Loopback `serve_once` with `stall_before_body_ms` and `abort_after_headers` | Real curl against real sockets for timeout, truncation, redirect, and cancellation-during-transfer |

| Test | Covers | Line |
|---|---|---|
| `exit_28_is_labelled_total_when_connect_and_total_ceilings_coincide` | Coincident-ceiling phase attribution, both directions | 803 |
| `diagnostic_bound_never_splits_multibyte_code_points` | UTF-8-safe 512-byte truncation | 848 |
| `executable_selection_and_discovery_are_explicit` | Empty path, invalid config, discovery hit/miss typed as `Unavailable` | 986 |
| `missing_executable_and_spawn_failure_are_unavailable` | `Unavailable` for both operations, no `dest` | 1008 |
| `metadata_success_not_found_and_server_error` | 200 bytes, 404 `NotFound`, 500 `Transport` | 1035 |
| `artifact_success_not_found_and_server_error` | 4096 bytes byte-exact, 404 leaves no `dest`, 500 no `dest` | 1059 |
| `tls_like_process_failure_is_transport_not_unavailable` | Exit 35 is `Transport`, not `is_unavailable()` | 1106 |
| `truncated_transfer_is_hard_failure_without_promotion` | Headers-then-close with a declared `Content-Length` | 1121 |
| `metadata_and_artifact_size_overflow` | `TooLarge` on both paths, no promotion | 1148 |
| `connect_and_total_timeouts_are_typed` | Unroutable connect -> `Timeout`; stalled body -> `phase: "total"` | 1182 |
| `duration_decimal_seconds_serializes_sub_second_inputs_truthfully` | Exact decimal output, zero and sub-microsecond rejection | 1228 |
| `https_request_never_allows_a_cleartext_redirect_target` | `--proto` / `--proto-redir` narrowing incl. `HTTPS://` and http-only config | 1258 |
| `build_curl_args_passes_decimal_deadlines_never_above_ceiling` | Both deadline flags, sub-microsecond rejection | 1316 |
| `build_curl_args_passes_sub_second_deadlines_to_fake_curl` | argv actually carries `0.25` / `0.75` | 1367 |
| `sub_second_total_timeout_kills_child_promptly` | 250 ms total vs 4 s stall; < 750 ms elapsed, dir empty | 1393 |
| `cancellation_before_spawn_cleans_temp` | Early `Cancelled`, no `.part` leftovers | 1431 |
| `cancellation_during_body_stream_kills_and_reaps_child` | Cancel during an active 64 KiB transfer | 1453 |
| `redirect_follow_and_reject` | Follow returns target bytes; `follow_redirects(false)` -> `Transport` | 1484 |
| `proxy_disabled_uses_noproxy_and_cleared_env` | `--noproxy`, `--disable`, `--output -`, no `HTTP_PROXY` in child env | 1520 |
| `proxy_custom_env_is_explicit` | No `--noproxy`, exact injected pair in child env | 1540 |
| `artifact_temp_is_owner_private_and_no_clobber` | Promoted mode `0600`; foreign `dest` preserved | 1562 |
| `credential_material_is_redacted` | userinfo / query / fragment sentinels absent from `Display` | 1610 |
| `invalid_limits_fail_before_spawn` | Zero metadata bound -> `InvalidInput`, no `dest` | 1627 |
| `composition_with_fixture_falls_back_from_unavailable_curl` | `Unavailable` -> `ComposedTransport` + `FixtureTransport` under `UnavailableOnly` | 1651 |

**Platform gating.** 9 tests are ungated and run everywhere; 8 are
`#[cfg(not(windows))]` (live loopback HTTP: status, truncation, bounds,
timeouts, redirect, active-transfer cancellation) and 7 are `#[cfg(unix)]`
(fake-child, `sh` + `xxd`, and Unix mode assertions). The Windows lane therefore
runs the portable adapter, process, and error-path tests only: hosted Windows
runners refuse connections from spawned curl to the suite's local servers
(curl exit 7), so **no Windows live-HTTP curl behavior is claimed** anywhere
(`CHANGELOG.md` platform note; `plans/closure/acquisition-transport/007-status.md`;
M009 §9). The gating is per test, never per module, so the module still
compiles and runs on Windows.

**Commands:**

```sh
cargo test -p eggup-curl --locked
cargo clippy -p eggup-curl --all-targets --locked -- -D warnings
# whole-crate gate
./scripts/check-local.sh
```

## 13. Trade-offs vs. the Eggfetch adapter

Both adapters implement the same trait, enforce the same `min(request, adapter)`
deadline derivation, type only exact `404` as `NotFound`, and use the same seam
temp/promotion helpers. The differences a consumer actually pays for:

| Dimension | `eggup-curl` | `eggup-eggfetch` |
|---|---|---|
| Dependencies | `eggup-acquisition` only (`Cargo.toml:23-24`) | `eggfetch-core 0.2` (`http1`,`tls-rustls`,`tls-native-roots`,`proxy`) + `tokio` + `futures-util` |
| Binary size (M005, Darwin) | `curl_only` 417 KiB / 346 KiB stripped | `eggfetch_only` 1.9 MiB / 1.6 MiB stripped |
| External requirement | a `curl` executable must exist at the chosen path | none beyond the binary itself |
| TLS behavior | whatever the host `curl` links (e.g. libcurl/SecureTransport on macOS); no crate-side trust-root control | Rustls with native roots, decided at compile time |
| `env_clear()` side effect | child gets no `HOME`/`PATH`/`SSL_CERT_FILE`/`CURL_CA_BUNDLE`; only proxy keys are expressed | no child process; proxy expressed in-process via `ProxyEnvironment` |
| Per-call overhead | one process + two reader threads per fetch | a private current-thread `tokio` runtime per call (`eggfetch/src/lib.rs:196-228`) |
| Status evidence | `%{stderr}%{http_code}` from the same transfer, bounded to 64 bytes | `StatusClass` from the response itself (`eggfetch/src/lib.rs:475-490`) |
| Unavailability | a real, typed signal: missing/spawn failure -> `Unavailable` (the seam's `Unavailable` variant was added for this adapter) | reported through `__adapter_unavailable` for client-construction failures only |
| Redirect downgrade denial | per-request `--proto-redir` narrowing (`lib.rs:489-509`) | `RedirectPolicy::strict` via eggfetch |
| Ambient config surface | exists and is neutralized by `--disable` plus `env_clear()` | none |
| URL exposure | exact URL in `argv` for the child's lifetime (§7.4) | no command line |
| `https`-only callers | same | same |

Both remain interchangeable and neither may reference the other; the
composition decision belongs to the caller, per
[transport-footprint.md](transport-footprint.md) and
[ADR-0003](../plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md).

## 14. Cross-references

**Sibling deep dives**

- [overview.md](overview.md) — module map, dependency graph, cross-cutting
  invariants, composition paths, publication state.
- [acquisition.md](acquisition.md) — the seam contract: `FetchLimits::effective`,
  `AcquisitionError` variants, `redact_url`, `__acquire_exclusive_temp`,
  `__promote_no_clobber`, `ComposedTransport` / `CompositionPolicy`.
- [eggfetch-adapter.md](eggfetch-adapter.md) — the native HTTP adapter, and the
  matching strict redirect-downgrade posture.
- [transport-footprint.md](transport-footprint.md) — the three
  `required-features` binaries that measure what each transport choice links.

**Contracts and decisions**

- [`plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md`](../plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md)
  — transport neutrality; integrity is never authenticity; adapters return typed
  results and do not select fallback.
- [`crates/eggup-acquisition/src/lib.rs`](../crates/eggup-acquisition/src/lib.rs)
  — `AcquisitionTransport` (`lib.rs:578-606`), `FetchLimits::effective`
  (`:143-151`), `AcquisitionError` (`:249-279`), `redact_url` (`:350-379`).
- [`crates/eggup-curl/README.md`](../crates/eggup-curl/README.md),
  [`crates/eggup-curl/CHANGELOG.md`](../crates/eggup-curl/CHANGELOG.md).

**Planning and closure**

- [`plans/subsystems/acquisition-transport-roadmap.md`](../plans/subsystems/acquisition-transport-roadmap.md)
  — M005 (adapter + composition), M007 (three defect fixes), M008 (truthful
  sub-second deadlines), M009 (publication), M010 (seam publication).
- [`plans/closure/acquisition-transport/009-status.md`](../plans/closure/acquisition-transport/009-status.md)
  — the `0.1.2` publication record: registry evidence, the deliberate
  `eggup-acquisition ^0.1.0` caret, the retained Windows loopback limitation, and
  the two open medium findings in the *seam*.
- [`plans/closure/acquisition-transport/007-status.md`](../plans/closure/acquisition-transport/007-status.md)
  — UTF-8-safe truncation, finite artifact cap, and the retained-handle body
  streaming fix.
- [`plans/closure/acquisition-transport/005-status.md`](../plans/closure/acquisition-transport/005-status.md)
  — original adapter closure and the footprint numbers cited in §2.

### Known doc/code drift

Observed while reading the sources; the **code** is the behavior in each case.
Nothing here was changed — these are notes for the owning documents.

| Claim | Reality |
|---|---|
| `README.md:20` writes the status flag as `-w "%{stderr}%{http_code}"` | The code uses the long form `--write-out` (`lib.rs:552-553`). Same flag; the README is shorthand. |
| `acquisition/src/lib.rs:290-298` documents `__adapter_unavailable` as the constructor "`eggup-curl`" uses; `acquisition.md` §2 repeats that | `eggup-curl` constructs `AcquisitionError::Unavailable(..)` directly at `lib.rs:384`, `:403`, `:604` (the variant is public). Only `eggup-eggfetch` and `FixtureTransport` call `__adapter_unavailable`. The seam's stated reason for the constructor — not widening the public seam with a general constructor — is not achieved for this adapter, because the variant itself is public. |
| `architecture/eggfetch-adapter.md:3` and `:77` cite `lib.rs` line numbers for `eggup-eggfetch` from an older revision (1180 lines) | Current `eggup-eggfetch/src/lib.rs` is 1231 lines; several cited offsets have shifted. Owned by that document, not this one. |
| `discover_in` reads "the first existing regular file" (`lib.rs:374-375`) | It tests `fs::symlink_metadata(...).is_file()` (`lib.rs:396-400`), which does **not** follow symlinks. A `curl` reachable only through a symlink in a `PATH` entry is skipped and the search continues to the next entry. A caller who needs a symlinked `curl` should pass the path to `with_executable` directly. |
| Nothing in `README.md`/`CHANGELOG.md` mentions the child environment beyond proxy keys | `env_clear()` (`lib.rs:599`) also withholds `HOME`, `PATH`, `SSL_CERT_FILE`, and `CURL_CA_BUNDLE` from curl, so a consumer's environment-based CA-bundle configuration is not honored. This is consistent with the documented "cleared environment" posture, but the CA-bundle consequence is unstated. |
| Nothing in the docs mentions it | The exact URL is passed on the child's command line for the life of the process (§7.4). `redact_url` covers diagnostics only. |
