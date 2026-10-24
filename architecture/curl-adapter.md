# curl Adapter Deep Dive (`crates/eggup-curl`)

> Source: `crates/eggup-curl/src/lib.rs` (2532 lines, single file, 34 `#[test]`
> functions), `examples/curl_fetch.rs`, `README.md`, `CHANGELOG.md`, `Cargo.toml`.
> Dependency position: `eggup-curl -> eggup-acquisition` **only** — no HTTP client,
> no TLS stack, no `tokio`, no `eggup-core`
> (`Cargo.toml:23-24`). Published to crates.io at `0.2.0` with the requirement
> `eggup-acquisition = { version = "0.2", path = "../eggup-acquisition" }`;
> the caret remains deliberately **not** tightened to an exact pin
> (`plans/closure/acquisition-transport/009-status.md`, "Metadata decision"), so
> the adapter inherits a seam fix on the next resolve. The `0.1.0` caret became
> `0.2` in the `0.2.0` publication train (Acquisition Transport M017): the
> seam's `0.2.0` is source-breaking, so a caret left at `^0.1.0` would let a
> consumer's lockfile resolve a seam the `0.2.0` adapter cannot compile
> against.
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
  the adapter's bounded policy (`build_curl_args`, `lib.rs:669-728`).
- Executable selection, which is explicit and caller-driven
  (`CurlTransport::with_executable`, `lib.rs:191-207`).
- Process lifecycle: spawn, bounded polling, kill, reap, reader-thread joins
  (`run_curl_to_file`, `lib.rs:744-906`).
- HTTP status capture for the **same** request that produced the body, and the
  mapping of that status plus curl's exit code onto seam outcomes
  (`classify_curl_result`, `lib.rs:909-977`).
- Redirect, protocol, and proxy flags; `--disable` neutralization of the user's
  `~/.curlrc`.
- Exclusive owner-private temp staging and no-clobber promotion, using the
  seam's helpers rather than its own.

It **never decides** any of the following. Each is asserted by absence, not by
comment:

| Not decided here | Where it lives instead |
|---|---|
| Release, version, SemVer, mirror, or source selection | the caller; the seam explicitly refuses to learn it ([acquisition.md](acquisition.md) §1) |
| Transport fallback order or scope | `eggup-acquisition::ComposedTransport` + `CompositionPolicy` (`acquisition/src/lib.rs:979-1071`) |
| Destination path or install root | the caller; `fetch_artifact` only requires an existing real parent and an absent `dest` |
| Whether `NotFound` means anything | the consumer; the seam types it as data |
| Authenticity, signing, or publisher trust | nobody, by design ([ADR-0003](../plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md)) |
| Integrity verification | `eggup-core` SHA-256; this crate hashes nothing |
| Service quiesce/restore | `eggup-service` |
| Execution of downloaded bytes | never — bodies are written to a file and returned; nothing is invoked |

`CurlTransport`'s own rustdoc repeats the negative list: "No release, version,
fallback, destination, or service policy lives here. Only exact 404 becomes
`NotFound`; TLS/5xx/timeout are hard failures and missing/spawn failures become
`Unavailable` for safe composition" (`lib.rs:168-172`).

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
| `MAX_USER_AGENT_BYTES` | const | Maximum accepted outbound `User-Agent` length, 512 bytes | `lib.rs:37` |
| `CurlProxy` | enum | Explicit proxy decision: `Disabled` (default) / `FromEnvironment` / `Custom(Vec<(String,String)>)` | `lib.rs:45-54` |
| `CurlConfig` | struct | The single policy point: ceilings, redirect bound + follow flag, protocol allowlist, proxy | `lib.rs:58-76` |
| `CurlConfig::strict()` | fn | Strict default; identical to `Default::default()` | `lib.rs:93-95` |
| `CurlConfig::timeouts(connect, total)` | fn | Builder for both adapter ceilings | `lib.rs:101-105` |
| `CurlConfig::max_redirects(n)` | fn | Builder for the redirect bound | `lib.rs:108-111` |
| `CurlConfig::follow_redirects(bool)` | fn | Builder for whether `--location` is passed | `lib.rs:114-117` |
| `CurlConfig::allowed_protocols(Vec<String>)` | fn | Builder for the `--proto` / `--proto-redir` allowlist | `lib.rs:120-123` |
| `CurlConfig::proxy(CurlProxy)` | fn | Builder for the proxy decision | `lib.rs:126-129` |
| `CurlConfig::effective_timeouts(FetchLimits)` | fn | `min(request, adapter)` per phase -> `(connect, total)` | `lib.rs:132-134` |
| `CurlTransport` | struct | The adapter: private `executable: PathBuf` + `config: CurlConfig` + private `user_agent: Option<String>` | `lib.rs:174-179` |
| `CurlTransport::with_executable(path, config)` | fn | Constructor without a user agent; validates config and non-empty path, does **not** touch the filesystem | `lib.rs:191-207` |
| `CurlTransport::with_executable_and_user_agent(path, config, agent)` | fn | Constructor with a caller-supplied `User-Agent`; validates config, path, and the agent's grammar; `InvalidInput` on rejection | `lib.rs:238-257` |
| `CurlTransport::executable()` | fn | Accessor for the configured path | `lib.rs:260-262` |
| `CurlTransport::config()` | fn | Accessor for the active config | `lib.rs:265-267` |
| `CurlTransport::user_agent()` | fn | Accessor for the accepted outbound `User-Agent`; `None` when none is configured | `lib.rs:275-277` |
| `AcquisitionTransport for CurlTransport::fetch_metadata` | trait method | Bounded small body into memory | `lib.rs:330-398` |
| `AcquisitionTransport for CurlTransport::fetch_artifact` | trait method | Streamed body into `dest` via no-clobber promotion | `lib.rs:400-492` |
| `discover_curl_executable()` | fn | **Opt-in** `PATH` search returning the first regular `curl` file, or `Unavailable` | `lib.rs:507-509` |

API-surface notes for review:

- `CurlConfig` and `CurlProxy` derive `Debug, Clone` (+ `Default` for
  `CurlProxy`) but **not** `PartialEq`, so a caller cannot compare two configs
  directly. There is no public `validate()`; validation runs inside
  `with_executable` and again at every fetch entry point.
- All `CurlConfig` fields are public for 0.1.x source compatibility, exactly like
  `FetchLimits`. That is why `config.validate()` runs at construction *and*
  again per fetch (`lib.rs:346`, `lib.rs:417`) — a directly constructed struct
  literal cannot bypass it.
- `with_executable` performs **no** `fs` check on the path. Existence,
  executability, and spawnability are deliberately deferred to the request so
  they classify as `Unavailable` rather than as a construction error
  (`lib.rs:181-207`).
- **`CurlConfig` deliberately does NOT carry the `User-Agent`** (M014). Its
  fields are public and the struct is **not** `#[non_exhaustive]`, so adding a
  field would be source-breaking for every consumer that constructs it with a
  struct literal; making it `#[non_exhaustive]` would itself be source-breaking.
  The control therefore lives on `CurlTransport`, whose fields are private and
  whose only construction path is a constructor, so adding a private field is
  not source-breaking. The value is validated at construction and re-validated
  at argv-build time (`lib.rs:286-327`, `lib.rs:677-682`). See §13.1.

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
(`lib.rs:58-76`)

| Field | Default (`Default` / `strict()`) | Meaning and enforcement |
|---|---|---|
| `connect_timeout` | `DEFAULT_CONNECT_TIMEOUT` = 10 s | Ceiling only. Serialized to `--connect-timeout` and clamped against the caller's value (§4.3). Enforced by curl; there is no parent-side connect deadline. |
| `total_timeout` | `DEFAULT_TOTAL_TIMEOUT` = 120 s | Ceiling only. Serialized to `--max-time` **and** independently enforced by the parent's wall deadline `start + eff_total` (`lib.rs:760-761`, `:840-847`). |
| `max_redirects` | `DEFAULT_MAX_REDIRECTS` = 10 | Passed as `--max-redirs` only when `follow_redirects` is true. `validate()` rejects `> 50` (`lib.rs:147-151`). |
| `follow_redirects` | `true` | Decides whether `--location` is passed at all (`lib.rs:690-694`). When false, a 3xx response is a hard `Transport` failure, not `NotFound`. |
| `allowed_protocols` | `vec!["http", "https"]` | Bounds the initial request via `--proto =http,https` and the redirect targets via a possibly-narrowed `--proto-redir` (`lib.rs:703-709`). `validate()` requires 1..=8 entries, each 1..=16 ASCII-alphanumeric bytes (`lib.rs:143-154`). |
| `proxy` | `CurlProxy::Disabled` | `Disabled` adds `--noproxy *` and contributes no environment; `FromEnvironment` snapshots 8 proxy keys; `Custom` injects caller pairs verbatim (§4.4). |

`strict()` is a literal alias for `default()` (`lib.rs:93-95`). There is no
"relaxed" constructor and no hidden default beyond the table above.

### 4.1 Validation (`CurlConfig::validate`, `lib.rs:136-165`)

Fails closed with `AcquisitionError::InvalidInput` on: zero `connect_timeout` or
`total_timeout`; `connect_timeout > total_timeout`; `max_redirects > 50`; an
empty or >8-element protocol list; and any protocol name that is empty, longer
than 16 bytes, or not ASCII-alphanumeric. The protocol-name check is what makes
`--proto`/`--proto-redir` argument injection structurally impossible (§7).

### 4.2 Builder methods

`timeouts`, `max_redirects`, `follow_redirects`, `allowed_protocols`, `proxy`
(`lib.rs:101-129`) are all consuming `self -> Self` builders that only assign
fields. They perform **no** validation; the value is checked at
`with_executable` and at every fetch. A builder chain that produces an invalid
config therefore fails at the constructor, not at the builder call.

### 4.3 `effective_timeouts` and the `min(request, adapter)` clamp

`effective_timeouts` is a one-line delegation to the seam
(`lib.rs:132-134`):

```text
effective connect = min(request.connect_timeout, self.connect_timeout)
effective total   = min(request.total_timeout,   self.total_timeout)
```

which is `FetchLimits::effective` (`acquisition/src/lib.rs:143-151`). Both fetch
methods call it after `limits.validate()?`, and both hand the result to the
`run_curl_to_file` call (`lib.rs:364`, `:445`).

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
(`lib.rs:930-942`, test `exit_28_is_labelled_total_when_connect_and_total_
ceilings_coincide` at `lib.rs:1001`). See `CHANGELOG.md` "Fixed: a mislabelled
timeout phase".

Serialization for the two flags is `duration_decimal_seconds` (`lib.rs:589-613`):
microsecond-resolution decimal seconds with a `.` separator and trailing
fractional zeros stripped (`100 ms -> "0.1"`, `250 ms -> "0.25"`,
`1.5 s -> "1.5"`, `2 s -> "2"`). Truncation to whole microseconds can only
decrease the value, so curl never receives a longer deadline than the caller
asked for. Zero and sub-microsecond positive durations are rejected with
`InvalidInput` rather than silently rounded up to a second — the deliberate
M008 behavior, pinned by `duration_decimal_seconds_serializes_sub_second_inputs_
truthfully` (`lib.rs:1665`) and `build_curl_args_passes_decimal_deadlines_never_
above_ceiling` (`lib.rs:1757`).

### 4.4 `CurlProxy` and environment handling

`proxy_env_snapshot` (`lib.rs:615-636`) turns the decision into an explicit
pair list: `Disabled` yields an empty list; `FromEnvironment` reads exactly
`HTTP_PROXY`, `http_proxy`, `HTTPS_PROXY`, `https_proxy`, `ALL_PROXY`,
`all_proxy`, `NO_PROXY`, `no_proxy` at call time; `Custom` clones the caller's
pairs. That list is the **only** thing added to the child environment, which is
otherwise cleared (§7). `ProxyKind` (`lib.rs:731-741`) is just the boolean
"should `--noproxy *` be passed" projection of the same decision.

## 5. Executable selection and hardening

Two selection paths, one of them explicitly opt-in:

- `CurlTransport::with_executable(path, config)` (`lib.rs:191-207`) — the
  primary, and only, way to construct the adapter. The caller supplies the exact
  path. An empty path is `InvalidInput`; an invalid config is `InvalidInput`; a
  path that does not exist is **not** detected here.
- `discover_curl_executable()` (`lib.rs:507-509`) — a free function, not an
  implicit default. It calls `discover_in(std::env::var_os("PATH"))`
  (`lib.rs:511-534`), which splits `PATH`, skips empty entries, and returns the
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

`--disable` is the first argument of every invocation (`lib.rs:683-689`), with
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

`impl AcquisitionTransport for CurlTransport` (`lib.rs:329-493`) implements the
two methods of the seam trait
(`acquisition/src/lib.rs:625-653`). Where each seam guarantee is honored:

| Seam guarantee | Honored in `eggup-curl` at |
|---|---|
| Exact caller URL, no discovery/version/mirror logic | `let url = request.url().to_string()` used verbatim in argv (`lib.rs:756`); nothing else about the URL is inspected except the scheme for `--proto-redir` narrowing |
| `limits.validate()` before any I/O, every call | `lib.rs:336` and `lib.rs:407`, ahead of temp creation, spawn, or filesystem access |
| Re-check `CurlFlag` at the boundary | `if cancel.is_cancelled()` at `lib.rs:338-340` and `lib.rs:409-411`, before staging |
| Effective deadlines are `min(request, adapter)` | `effective_timeouts` at `lib.rs:341` / `:427`, and `POLL_INTERVAL`/parent deadline enforcement in the poll loop |
| `dest` parent must already exist as a real directory | `symlink_metadata(parent)`, rejected unless `is_dir() && !file_type().is_symlink()` (`lib.rs:415-421`) |
| `dest` must be absent; never overwrite | Fast-fail `symlink_metadata(dest).is_ok()` -> `InvalidInput` (`lib.rs:422-426`); the authoritative race-safe guard is the seam's `__promote_no_clobber` (`lib.rs:477`) |
| Stage to an exclusively-created owner-private temp | `__acquire_exclusive_temp(parent, "eggup-curl")` (artifact, `lib.rs:429`) or `__acquire_exclusive_temp(temp_dir(), "eggup-curl-meta")` (metadata, `lib.rs:347-348`) |
| Promote no-clobber only on full success | `__promote_no_clobber` only on the `CurlOutcome::Success` + size-ok + not-cancelled path (`lib.rs:463-489`) |
| Partial output cleans only the owned temp | Local `Guard` calling `__remove_owned_temp` on every non-committed path (`lib.rs:349-363`, `:430-444`); no prefix scan, no sibling deletion |
| `NotFound` is data, never a fallback trigger | `CurlOutcome::NotFound` -> `Ok(FetchOutcome::NotFound)` only (`lib.rs:392-396`, `:458-462`); the adapter contains no fallback branch |
| Downloads are data, never executed | The only child process is `curl`; body bytes are written to a `File` and returned |
| `MetadataBytes`/`ArtifactEvidence` are adapter-constructed | `__adapter_metadata(bytes)` (`lib.rs:389`) and `__adapter_artifact(size)` (`lib.rs:481`) — the seam has no public constructor for either |

Two seam-adjacent facts worth confirming on review:

- The metadata path creates its temp inside `std::env::temp_dir()` and calls
  `let _ = fs::create_dir_all(&parent);` first (`lib.rs:345-346`). This is the
  only directory the crate ever creates, and it exists only to guarantee the
  system temp directory is present; the artifact path never creates the
  destination parent.
- The metadata path reads the whole body back with `fs::read` and *then* checks
  `bytes.len() > max` (`lib.rs:381-387`). Peak memory is still bounded, because
  the reader thread stops at `max + 1` bytes and curl is additionally given
  `--max-filesize` (§7), so a file larger than `max` cannot reach this read.

## 7. Process lifecycle

### 7.1 argv construction

`build_curl_args` (`lib.rs:669-728`) produces, in order:

```text
--disable --silent --show-error --no-progress-meter
[--location --max-redirs <max_redirects>]          if follow_redirects
--connect-timeout <decimal-seconds>
--max-time         <decimal-seconds>
--max-filesize     <max_bytes>
[--proto =a,b --proto-redir =x,y]                   if allowed_protocols non-empty
[--noproxy *]                                       if proxy == Disabled
[--user-agent <agent>]                              if a user agent is configured
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
- **The `--` terminator** (`lib.rs:725`) prevents the URL from being parsed as
  an option. Combined with `validate()`'s ASCII-alphanumeric protocol-name rule,
  a caller cannot inject an extra flag through `allowed_protocols`.
- **`--output -`** sends the body to stdout. The adapter never hands curl a
  pathname: the temp file was created by the adapter with `create_new`, and the
  adapter keeps writing through its own retained descriptor. This is the M007
  fix for curl reopening an Eggup-exclusive temp by name
  (`lib.rs:778-779`, `CHANGELOG.md` M007 entry,
  `plans/closure/acquisition-transport/007-status.md`).
- **`--write-out %{stderr}%{http_code}`** routes the status to stderr, keeping
  the body channel on stdout uncontaminated. The status therefore comes from
  the *same* transfer as the bytes; there is no second probe and no re-request
  whose answer could disagree with the body.
- **`--max-filesize <max_bytes>`** gives curl its own bound, so a large response
  can be refused by curl itself (exit 63) in addition to the adapter's counter.
- **`--proto-redir` narrowing** (`redirect_protocols`, `lib.rs:646-666`): if the
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
- **`--user-agent <agent>`** (`lib.rs:714-720`, M014) is emitted only when a
  caller installed a `User-Agent`; by default the element is absent and curl
  applies its own default, so the published adapter's wire behavior is
  unchanged. When present, the value is a **literal argv element**: it follows
  `--user-agent` immediately and sits **before** the `--` separator and the
  URL. curl takes the element after `--user-agent` as the value
  unconditionally, so a value beginning with `-` is not re-parsed as an
  option, and no shell ever sees it. The value is validated against a stated
  grammar before a single element is pushed (`lib.rs:677-682`), so a rejected
  value produces **no** argv at all — see §13.1.

### 7.2 spawn

`run_curl_to_file` (`lib.rs:744-906`) does, in order: take `start = Instant::now()`
and `deadline = start + eff_total` (`:760-761`); clone the caller's `File` handle
(`:762-764`); build the `Command` with `stdin(Stdio::null())`,
`stdout(Stdio::piped())`, `stderr(Stdio::piped())`, and `env_clear()`; re-add
only the proxy pairs; then `spawn()` (`:766-777`).

Consequences of `env_clear()` worth stating explicitly: the child receives **no**
`HOME`, `PATH`, `TMPDIR`, `SSL_CERT_FILE`, `CURL_CA_BUNDLE`, or
`SSL_CERT_DIR`. Proxy decision is the only environment this adapter expresses.
A consumer whose TLS trust depends on `CURL_CA_BUNDLE`/`SSL_CERT_FILE` in the
parent environment will not have that honored; it depends on the discovered
executable's compiled-in CA store.

Any `spawn()` error — `ENOENT`, `EACCES`, `ENOTDIR`, and equally a real
resource failure — is mapped to `AcquisitionError::Unavailable`
(`lib.rs:775-777`). This is the single most important classification decision in
the crate: `Unavailable` is the only hard failure that `ComposedTransport` will
fall back on under the default `CompositionPolicy::UnavailableOnly`
(`acquisition/src/lib.rs:1034-1042`).

**The detail carries the OS cause.** The failure is reported as:

```text
curl spawn failed for {redacted}: {kind}[ (os error {code})]
```

where `{redacted}` is the same redacted URL every other message uses, `{kind}`
is the `std::io::ErrorKind` in its `Debug` form (`NotFound`,
`PermissionDenied`, `WouldBlock`, `NotADirectory`, …), and ` (os error {code})`
is present only when the error carries a raw OS code — a missing executable
renders as `NotFound (os error 2)` on Unix. The detail is produced by the
private helper `spawn_failure_detail` (`lib.rs:568`) and bounded by the seam's
shared 512-byte UTF-8-safe rule.

**Why the cause is now preserved (Acquisition M015).** The classification above
was always deliberate; the defect was that the `io::Error` returned by
`Command::spawn()` was **discarded** by a `map_err(|_| …)`, so the only record a
consumer had of the failure was the same string for every possible cause. A
hosted CI run failed with exactly that message and could not be attributed: a
missing executable, a non-executable file, and a transient resource failure
were indistinguishable in the diagnostic. The cause is now **evidence inside
the detail**, and it never changes the variant — `Unavailable` still falls back,
and `Io` (terminal under every policy) is never used for a spawn failure.

**The cause segment carries no free-form OS text.** Only the `ErrorKind`
discriminant and the numeric code are rendered; the OS-provided message text is
not. An OS message is free-form, platform-specific, and can echo a path, so
rendering it would risk carrying caller-supplied material into a diagnostic that
this crate's redaction rule governs. The added cause segment therefore
introduces **no URL material and no credential material** beyond the redacted
URL the detail already carried.

### 7.3 pipes, threads, and the poll loop

Two reader threads are started (`:784-818`):

- **Body reader** owns the piped stdout plus the *cloned* `File`, and reads
  through `stdout.take(max_bytes.saturating_add(1))` (`:786`). It accumulates
  `written`, sets an `AtomicBool` overflow flag when `written > max_bytes`
  (`:794-796`), and `write_all`s each chunk to the retained descriptor. When the
  `take` limit is reached, `read` returns `Ok(0)` and the loop ends, so the
  reader always terminates on its own.
- **Status reader** owns piped stderr, reads to EOF, and retains at most
  `MAX_WRITE_OUT_BYTES` = 64 bytes (`:38-39`, `:804-818`). It keeps *draining*
  after the bound is reached (`n.min(room)` with `room == 0`), so a chatty curl
  can never block on a full stderr pipe. Raw stderr text is never surfaced in an
  error message.

The parent then polls every `POLL_INTERVAL` = 5 ms (`:40-41`, `:820-859`), in
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
already reaped by `try_wait`, and a second `let _ = child.wait();` at `:860` is
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
  and `let _ = child.wait()` discard errors (`:822-823`, `:830-831`, `:841-842`,
  `:851-852`). If `kill()` were ever to fail *and* the child kept running,
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

### 7.5 Why this crate owns its bounded subprocess runner

This adapter implements its own bounded subprocess runner because it **may
depend only on `eggup-acquisition`** (`crates/eggup-curl/Cargo.toml` declares
exactly one dependency). The workspace's other bounded runners are therefore
unreachable from it:

| Runner | Owner | Why it is unreachable from `eggup-curl` |
|---|---|---|
| `run_bounded` | `eggup-core` | `eggup-core` is *above* the seam in the graph; an `eggup-curl` -> `eggup-core` edge would invert the dependency direction the seam sits under |
| `SystemExecutor::run` | `eggup-service` | `eggup-service` is above `eggup-core`; an `eggup-curl` -> `eggup-service` edge would create a cycle and pull the service-manager surface into a transport adapter |

Adding either dependency edge to remove the duplication would be a **boundary
violation**, not a simplification: it would either invert the graph or make a
transport adapter depend on the service-manager layer. The duplication is
therefore deliberate and bounded, and it is recorded here so a reviewer does not
read it as an oversight.

The runner shares one shape with core's `run_bounded`: an **unbounded post-exit
reader join** — the reader threads are joined without a deadline once the child
has exited (§7.4). The deadline defect that shape carries is tracked by
`plans/implementation/verified-update-core/014-bounded-execution-deadline-corrective.md`
and is **out of scope for this seam milestone**. M013 changes no process
lifecycle, deadline, kill/reap behavior, or environment clearing here, and does
not depend on that milestone closing.

## 8. Status classification and outcome mapping

`classify_curl_result` (`lib.rs:909-977`) takes the child's exit code, the
bounded stderr bytes, the temp path, the redacted URL, and the timing inputs,
and returns `CurlOutcome::Success | CurlOutcome::NotFound` or an
`AcquisitionError`. Exit code is consulted **first**, and status second.

| Condition | Result | Location |
|---|---|---|
| `Some(28)` (curl operation timeout) | `Timeout { phase }`; `"connect"` only when `eff_connect < eff_total` **and** `elapsed <= eff_connect + POLL_INTERVAL + 5 ms`; otherwise `"total"` | `lib.rs:930-942` |
| `Some(63)` (`--max-filesize` exceeded) | remove owned temp, `TooLarge { limit: max_bytes }` | `lib.rs:944-947` |
| `Some(0)` | fall through to status classification | `lib.rs:948` |
| Any other `Some(_)`, or `None` (killed by signal) | remove owned temp, `Transport("curl transfer failed (exit N) for {redacted}")` | `lib.rs:949-959` |
| `Some(200..=299)` with exit 0 | `CurlOutcome::Success` | `lib.rs:962` |
| `Some(404)` with exit 0 | `CurlOutcome::NotFound` | `lib.rs:963` |
| Any other status (3xx not followed, 4xx, 5xx) | remove owned temp, `Transport("HTTP {status} from {redacted}")` | `lib.rs:964-969` |
| Unparseable/absent status with exit 0 | remove owned temp, `Transport("curl status capture failed for {redacted}")` | `lib.rs:970-975` |

Then, before the outcome is trusted (`lib.rs:861-895`): the body reader is
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
(`lib.rs:950-958`), which is what `truncated_transfer_is_hard_failure_without_
promotion` (`lib.rs:1465`) pins.

## 9. Control flow, step by step

### 9.1 `fetch_metadata(request, limits, cancel)` (`lib.rs:330-398`)

1. `limits.validate()?`; `self.config.validate()?`; early `Cancelled`.
2. `effective_timeouts(limits)`; `max = limits.max_metadata_bytes`.
3. `fs::create_dir_all(temp_dir())` (result ignored), then
   `__acquire_exclusive_temp(temp_dir(), "eggup-curl-meta")` -> owned `(File,
   PathBuf)`.
4. Build the local `Guard` (`lib.rs:349-363`): on drop, unless disarmed,
   `__remove_owned_temp(path)`.
5. `run_curl_to_file(...)` with `max as u64` and the transport's configured
   `User-Agent` (if any); a `?` here propagates and the `Guard` cleans up.
6. `Success`: re-check cancel; `fs::read(&tmp)` -> `__remove_owned_temp` ->
   disarm; `bytes.len() > max` -> `TooLarge`; else
   `Ok(FetchOutcome::Success(__adapter_metadata(bytes)))`.
7. `NotFound`: remove temp, disarm, `Ok(FetchOutcome::NotFound)`.

### 9.2 `fetch_artifact(request, dest, limits, cancel)` (`lib.rs:400-492`)

1. `limits.validate()?`; `self.config.validate()?`; early `Cancelled`.
2. `dest.parent()` must exist and be a real directory, not a symlink
   (`lib.rs:412-421`).
3. Fast-fail if `dest` already exists (`lib.rs:422-426`).
4. `effective_timeouts(limits)`; `max_artifact = limits.max_artifact_bytes`.
5. `__acquire_exclusive_temp(parent, "eggup-curl")` — created in the **destination
   parent**, so promotion is same-filesystem (`lib.rs:429`).
6. Build the `Guard` (`lib.rs:430-444`).
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
(`acquisition/src/lib.rs:566-603`). `artifact_temp_is_owner_private_and_no_
clobber` (`lib.rs:2120`) asserts both the `0600` mode and that a pre-existing
destination keeps its `FOREIGN` contents.

## 10. Errors and redaction

Every message this crate authors is category text plus the seam's redacted URL,
truncated by `bound` to at most 512 bytes **at a UTF-8 character boundary**
(`lib.rs:536-544`; the boundary property is pinned by
`diagnostic_bound_never_splits_multibyte_code_points`, `lib.rs:1055`). `bound`
itself is a one-line delegation to the seam's shared rule
(`eggup_acquisition::__adapter_bound_detail`), so this crate keeps **no local
copy** of the truncation logic.

The redacted string is `request.redacted()` -> `redact_url`
(`acquisition/src/lib.rs:404-432`), computed once per fetch at `lib.rs:757` and
passed down. It strips `user:password@`, replaces the fragment with
`#<redacted>`, replaces the query with `?<redacted>`, and truncates overlong
URLs at 256 bytes with an ellipsis.

| Message | Contains |
|---|---|
| `Unavailable("curl spawn failed for {redacted}: {kind} (os error {code})")` | redacted URL + the `ErrorKind` and the raw OS code of the failed `spawn()` |
| `Unavailable("curl not found on PATH")` / `"PATH is unset"` | nothing |
| `Transport("curl transfer failed (exit {code}) for {redacted}")` | numeric exit code + redacted URL |
| `Transport("HTTP {status} from {redacted}")` | numeric status + redacted URL |
| `Transport("curl status capture failed for {redacted}")` | redacted URL |
| `Timeout { phase }`, `TooLarge { limit }`, `Cancelled` | numeric category only |
| `Io { operation, source }` | the operation label plus the OS error text, for owned filesystem/`wait` operations only; the underlying `io::Error` is preserved, so `source()` returns it and `source.kind()` is matchable |
| `InvalidInput("...")` | fixed policy-violation text, no caller data |

Deliberately excluded from every message: curl's own stderr (the bounded 64-byte
capture is read and parsed for the status, then dropped), the command line, the
argv, proxy environment values, the exact URL, and any body bytes.
`credential_material_is_redacted` (`lib.rs:2168`) asserts that userinfo, query,
and fragment sentinels never appear in the `Display` output of a failing fetch.

One asymmetry to keep in mind when reading logs: the redaction covers Eggup's
diagnostics, while the child process's command line carries the exact URL
(§7.4).

## 11. Invariants and failure modes

| Condition | Observable outcome | Enforced at |
|---|---|---|
| Executable path empty | `InvalidInput("curl executable path must not be empty")` | `lib.rs:197-201` |
| Config invalid (zero/out-of-order timeouts, redirects > 50, protocol list 0 or > 8, non-alphanumeric protocol name) | `InvalidInput` at construction, and again at each fetch | `lib.rs:136-165`, called at `:195`, `:337`, `:408` |
| Caller-supplied `User-Agent` empty, over `MAX_USER_AGENT_BYTES`, or carrying a C0/C1 control character, `DEL`, or a surrounding space | `InvalidInput` at construction (no transport is built) and again at argv-build time (**no argv is produced**); **never** `Unavailable`, so a composed transport does not fall back | `lib.rs:286-327`, `:677-682`; tests `lib.rs:2284`, `:2327`, `:2448` |
| Caller limits invalid (zero bound, zero timeout, `connect > total`) | `InvalidInput` before staging, spawn, or any filesystem write | `lib.rs:336`, `:407` |
| `dest` has no parent / parent missing / parent is a symlink | `InvalidInput` (no parent) or `Io` (unreadable parent) | `lib.rs:412-421` |
| `dest` already exists | `InvalidInput("artifact destination already exists; refusing to overwrite")`; foreign content preserved | `lib.rs:422-426`; race-safe re-check in `__promote_no_clobber` |
| Cancel already requested | `Cancelled`, no temp left behind | `lib.rs:338-340`, `:409-411`; test `lib.rs:1991` |
| Executable missing / not executable / spawn failure | `Unavailable` (never `Transport`, never `Io`), with the detail naming the OS cause — the `ErrorKind` and, when present, the raw OS code; no `dest`; temp cleaned by `Guard` | `lib.rs:775-777`; helper `lib.rs:568`; tests `lib.rs:1263`, `lib.rs:1307` |
| curl exits non-zero (TLS, DNS, connect, proxy, partial body, signal) | `Transport("curl transfer failed (exit N) …")`; temp removed; never promoted | `lib.rs:949-959`; tests `lib.rs:1450`, `:1465` |
| curl exit 28 (operation timeout) | `Timeout { phase: "connect" \| "total" }` | `lib.rs:930-942`; tests `lib.rs:1001`, `:1575` |
| Parent wall deadline reached | kill + reap + join, `Timeout { phase: "total" }` | `lib.rs:840-847`; test `lib.rs:1934` |
| Response exceeds `max_metadata_bytes` / `max_artifact_bytes` | `TooLarge { limit }` from three independent places (curl `--max-filesize` exit 63, the reader's `overflow` flag, the post-join byte count); `dest` never created | `lib.rs:944-947`, `:794-796`, `:821-827`, `:875-877`, `:385-387`, `:470-476` |
| Cancel observed mid-transfer | kill + reap + join, `Cancelled`, owned temp removed, `dest` untouched | `lib.rs:829-836`; test `lib.rs:2013` |
| `try_wait` error | kill + reap + join, `Io("waiting for curl: …")` | `lib.rs:850-857` |
| Owned-file write/flush failure, reader thread panic | `Io`; child already reaped or reaped by the teardown | `lib.rs:861-868`, `:869-881` |
| Status unparseable with exit 0 | `Transport("curl status capture failed …")` — fail closed, never a guessed `NotFound` | `lib.rs:970-975` |
| Sub-microsecond positive duration | `InvalidInput` rather than a widened second | `lib.rs:596-601`; test `lib.rs:1665` |
| Reader thread blocked on a full stdout pipe | Resolved by the overflow flag plus parent kill within one `POLL_INTERVAL` | `lib.rs:786`, `:794-796`, `:823-830` |

### Review checklist

- [ ] `CurlConfig` is still the only policy surface; no new ambient default crept in.
- [ ] The caller-settable `User-Agent` is still validated before the wire, is still rejected **only** as `InvalidInput` (never `Unavailable`), and is still emitted as one literal `--user-agent` element **before** the `--` separator.
- [ ] The default transport still sends **no** `User-Agent`, so the published adapter's wire behavior is unchanged.
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
- [ ] Diagnostics still use `request.redacted()` + category or OS-cause text only (never free-form OS or upstream text), `bound()` at a UTF-8 boundary; raw curl stderr still never surfaces.
- [ ] No authenticity, signature, or publisher-trust claim has appeared in any doc or error string.

## 12. Testing approach

34 `#[test]` functions, all in-crate (`lib.rs:979-2523`). No test reaches a
public network: the two harnesses are a loopback `TcpListener`
(`serve_once`, `lib.rs:1110-1149`) and a fake `curl` shell script
(`fake_curl_script`, `lib.rs:1210-1239`) that records its own `"$@"` and `env`
into files so argv and environment can be asserted directly. A third harness,
`serve_once_recording_head` (`lib.rs:1172-1198`, M014), records the raw request
head the client sent, so a header the adapter sets can be asserted **on the
wire** and not only in the child's argv.

**A test that cannot construct its fixture fails; it never reports a pass.**
The nine tests that drive a *real* `curl` obtain their executable through
`require_curl_executable()` / `require_transport()` (`lib.rs:1089-1108`), which
**panic with an actionable message** naming the missing prerequisite and the
remedy ("no `curl` executable was discoverable on `PATH` … install `curl` or put
a curl executable on `PATH`"). The earlier `let Some(t) = transport() else
{ return };` pattern made a host without `curl` report those nine tests as
**passed while asserting nothing**; it is gone, and the regression is now
mechanically prevented (see the guard suite below).

The one environment-conditional test is
`connect_phase_timeout_is_typed_end_to_end` (`lib.rs:1575`): it needs a host
that blackholes the route to `10.255.255.1:81` so the connect ceiling expires
instead of failing fast with `ENETUNREACH`/`EHOSTUNREACH`. It is explicitly
marked as ignored **with a recorded reason** and is therefore reported as
*ignored*, never as *passed*; its coverage is **not** claimed. The connect/total
**label** it would prove is proven deterministically instead, by
`classify_curl_result_labels_connect_and_total_without_a_subprocess`
(`lib.rs:1606`), which injects `start` and needs no subprocess, no network, and
no sub-second wall-clock budget.

**What makes process-based behavior testable here:**

| Technique | Enables |
|---|---|
| `CurlTransport::with_executable(&script, ...)` against a recording fake | Asserting exact argv (`--noproxy`, `--disable`, `--output -`, no temp pathname) and the child environment, without a network |
| Fake script emits a chosen body on stdout, chosen code on stderr, chosen exit | Covering exit 28 / 35 / 7 / 0 and status 200/302/404/500 without an HTTP server |
| `discover_in(Some(path))` takes the `PATH` value as an argument | Testing the miss path without mutating process `PATH` from parallel tests |
| `classify_curl_result` takes `start: Instant` and the temp path as parameters | Unit-testing exit-28 phase labelling with synthetic elapsed time and no process |
| `include_str!("../src/lib.rs")` in the integration guard suite | Scanning this crate's own source at compile time, with no filesystem discovery and no network |
| `duration_decimal_seconds` / `build_curl_args` are pure | Pinning serialization and flag policy without spawning anything |
| Loopback `serve_once` with `stall_before_body_ms` and `abort_after_headers` | Real curl against real sockets for timeout, truncation, redirect, and cancellation-during-transfer |

| Test | Covers | Line |
|---|---|---|
| `exit_28_is_labelled_total_when_connect_and_total_ceilings_coincide` | Coincident-ceiling phase attribution, both directions | 1001 |
| `diagnostic_bound_never_splits_multibyte_code_points` | UTF-8-safe 512-byte truncation | 1046 |
| `executable_selection_and_discovery_are_explicit` | Empty path, invalid config, discovery hit/miss typed as `Unavailable` | 1241 |
| `missing_executable_and_spawn_failure_are_unavailable` | `Unavailable` for both operations, no `dest` | 1263 |
| `spawn_failure_detail_names_the_os_cause_and_stays_unavailable` | still `Unavailable`; the detail names the `ErrorKind` and the raw OS code; no URL or credential material | 1307 |
| `metadata_success_not_found_and_server_error` | 200 bytes, 404 `NotFound`, 500 `Transport` | 1379 |
| `artifact_success_not_found_and_server_error` | 4096 bytes byte-exact, 404 leaves no `dest`, 500 no `dest` | 1403 |
| `tls_like_process_failure_is_transport_not_unavailable` | Exit 35 is `Transport`, not `is_unavailable()` | 1450 |
| `truncated_transfer_is_hard_failure_without_promotion` | Headers-then-close with a declared `Content-Length` | 1465 |
| `metadata_and_artifact_size_overflow` | `TooLarge` on both paths, no promotion | 1492 |
| `total_phase_timeout_is_typed_end_to_end` | Stalled body -> exactly `Timeout { phase: "total" }` | 1530 |
| `connect_phase_timeout_is_typed_end_to_end` | Unroutable connect -> exactly `Timeout { phase: "connect" }`; **ignored, environment-conditional, reason recorded** | 1575 |
| `classify_curl_result_labels_connect_and_total_without_a_subprocess` | The connect/total label at the `classify_curl_result` layer, with `start` injected; no subprocess, no network | 1606 |
| `duration_decimal_seconds_serializes_sub_second_inputs_truthfully` | Exact decimal output, zero and sub-microsecond rejection | 1665 |
| `https_request_never_allows_a_cleartext_redirect_target` | `--proto` / `--proto-redir` narrowing incl. `HTTPS://` and http-only config | 1695 |
| `build_curl_args_passes_decimal_deadlines_never_above_ceiling` | Both deadline flags, sub-microsecond rejection | 1757 |
| `sub_second_deadlines_serialize_truthfully_without_a_subprocess` | 250 ms/750 ms -> `0.25`/`0.75`; `--max-time` is the *effective* ceiling; sub-microsecond rejection; no subprocess, no network, no wall clock | 1818 |
| `build_curl_args_passes_sub_second_deadlines_to_fake_curl` | argv carries `--connect-timeout 0.25`; the total budget is decoupled from the enforced ceiling (30 s) so the assertion cannot expire for scheduling reasons | 1908 |
| `sub_second_total_timeout_kills_child_promptly` | 250 ms total vs 4 s stall; elapsed `>=` the short total timeout and `<` 2 s, dir empty | 1934 |
| `cancellation_before_spawn_cleans_temp` | Early `Cancelled`, no `.part` leftovers | 1991 |
| `cancellation_during_body_stream_kills_and_reaps_child` | Cancel during an active 64 KiB transfer | 2013 |
| `redirect_follow_and_reject` | Follow returns target bytes; `follow_redirects(false)` -> `Transport` | 2044 |
| `proxy_disabled_uses_noproxy_and_cleared_env` | `--noproxy`, `--disable`, `--output -`, no `HTTP_PROXY` in child env | 2078 |
| `proxy_custom_env_is_explicit` | No `--noproxy`, exact injected pair in child env | 2098 |
| `artifact_temp_is_owner_private_and_no_clobber` | Promoted mode `0600`; foreign `dest` preserved | 2120 |
| `credential_material_is_redacted` | userinfo / query / fragment sentinels absent from `Display` | 2168 |
| `invalid_limits_fail_before_spawn` | Zero metadata bound -> `InvalidInput`, no `dest` | 2185 |
| `composition_with_fixture_falls_back_from_unavailable_curl` | `Unavailable` -> `ComposedTransport` + `FixtureTransport` under `UnavailableOnly` | 2209 |
| `user_agent_is_emitted_as_a_literal_argv_element_before_the_url` | The value is the exact element after `--user-agent`, exactly once, **before** the `--` separator and the URL | 2237 |
| `user_agent_rejects_control_characters_without_producing_argv` | `CR`, `LF`, `NUL`, `HTAB`, `DEL`, and C1 are rejected as `InvalidInput` at both the argv layer and the constructor, and **no** argv is produced | 2283 |
| `user_agent_rejects_empty_and_over_bound_values` | Empty and over-bound values rejected; a value of exactly `MAX_USER_AGENT_BYTES` accepted | 2326 |
| `default_transport_emits_no_user_agent_and_keeps_the_previous_argv` | The default transport carries no user agent and its argv is element-for-element the previously observed one | 2383 |
| `rejected_user_agent_is_a_terminal_invalid_input_not_unavailable` | The rejection is `InvalidInput`, never `Unavailable`; a composed transport does not fall back on it | 2447 |
| `user_agent_reaches_the_wire_as_the_configured_header` | The configured value is the `User-Agent` header the peer actually receives | 2499 |

An **integration** suite lives in `tests/` (excluded from the published package
by `Cargo.toml`'s `exclude`):

| Suite | Tests | Demonstrates |
|---|---|---|
| `tests/assertion_guard.rs` | 3 | the **pattern guard** (no vacuous early exit in this crate's sources), the **reason guard** (every ignored test records a reason), and the **behavior witness** (each required behavior has a named, non-vacuous, asserting test) |

**Platform gating.** 17 tests are ungated and run everywhere; 10 are
`#[cfg(not(windows))]` (live loopback HTTP: status, truncation, bounds,
timeouts, redirect, active-transfer cancellation, and the on-the-wire
`User-Agent` header) and 7 are `#[cfg(unix)]`
(fake-child, `sh` + `xxd`, and Unix mode assertions) — 34 in total. The Windows lane therefore
runs the portable adapter, process, and error-path tests only: hosted Windows
runners refuse connections from spawned curl to the suite's local servers
(curl exit 7), so **no Windows live-HTTP curl behavior is claimed** anywhere
(`CHANGELOG.md` platform note; `plans/closure/acquisition-transport/007-status.md`;
M009 §9). The gating is per test, never per module, so the module still
compiles and runs on Windows.

The `#[cfg(not(windows))]` gate is a **known, deliberate platform difference**,
not a silent coverage gap, and the reason is recorded in the test module itself
(`lib.rs:991-998`): the loopback HTTP fixture is Unix-only, and the Windows lane
exercises the same transport paths through the real `curl.exe` integration tests
instead. The Windows lane therefore runs a **smaller test set** than the Linux
and macOS lanes, and the excluded coverage must not be inferred from them.

`cancellation_before_spawn_cleans_temp` (`lib.rs:1991`) is **not**
`#[cfg]`-gated, so it runs on Windows too — but it now needs a discoverable
`curl` there as well, because the fail-loudly treatment applies on every
platform.

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
| Status evidence | `%{stderr}%{http_code}` from the same transfer, bounded to 64 bytes | `StatusClass` from the response itself (`eggfetch/src/lib.rs:470-483`) |
| Unavailability | a real, typed signal: missing/spawn failure -> `Unavailable` (the seam's `Unavailable` variant was added for this adapter) | reported through `__adapter_unavailable` for client-construction failures only |
| Redirect downgrade denial | per-request `--proto-redir` narrowing (`lib.rs:646-666`) | `RedirectPolicy::strict` via eggfetch |
| Ambient config surface | exists and is neutralized by `--disable` plus `env_clear()` | none |
| URL exposure | exact URL in `argv` for the child's lifetime (§7.4) | no command line |
| `https`-only callers | same | same |
| Caller-settable `User-Agent` | `CurlTransport::with_executable_and_user_agent` (M014); **none sent by default** | `EggfetchConfig::user_agent`; defaults to `DEFAULT_USER_AGENT` |

Both remain interchangeable and neither may reference the other; the
composition decision belongs to the caller, per
[transport-footprint.md](transport-footprint.md) and
[ADR-0003](../plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md).

### 13.1 Adapter capability parity: the `User-Agent` control (M014)

M013 recorded that this adapter exposed no `User-Agent` control while
`eggup-eggfetch` did, and deferred the gap to M014 as finding UF-1. **M014
closes it**: this adapter now exposes a caller-settable outbound `User-Agent`,
so both adapters offer the same caller capability.

**The control.**

- `CurlTransport::with_executable_and_user_agent(executable, config, agent)`
  (`lib.rs:238-257`) is the only way to install one. It validates the config,
  the executable path, and the agent, and returns
  `AcquisitionError::InvalidInput` when the agent is rejected — no transport is
  constructed in that case.
- `CurlTransport::user_agent()` (`lib.rs:275-277`) returns the accepted value,
  or `None` when none is installed. A caller can therefore confirm exactly what
  the adapter will send.
- `CurlTransport::with_executable` (`lib.rs:191-207`) is unchanged and installs
  **no** `User-Agent`.

**The grammar and the bound.** The value must match

```text
user-agent = 1*( VCHAR / SP / obs-text )
VCHAR      = %x21-7E          ; printable ASCII, excluding SP
SP         = %x20            ; permitted as interior whitespace, including runs
obs-text   = %xA0-10FFFF     ; any non-ASCII scalar value (C1 excluded)
```

and must be at most `MAX_USER_AGENT_BYTES` (`lib.rs:37`, 512 bytes) long. It is
rejected with `AcquisitionError::InvalidInput` when it is empty (R1), over the
bound (R2), contains any C0 control character — `CR`, `LF`, `NUL`, `HTAB`, and
the rest (R3) — contains `DEL` (R4) or any C1 control character (R5), or
begins or ends with a space (R6).

**Why the validation is not redundant.** `curl` does **not** sanitize
`--user-agent`. Verified against `curl 8.7.1` with a loopback listener: a value
containing `CR LF` was written into the request head verbatim, producing an
injected `X-Injected: 1` header. A value containing NUL cannot even be
transmitted — `Command::spawn` fails with `io::ErrorKind::InvalidInput`, which
this adapter maps to `Unavailable`, the one variant that **falls back**. The
adapter's own validation is therefore what keeps a malformed value off the
wire and out of the fallback path.

**Why `InvalidInput` and never `Unavailable`.** `Unavailable` is the only
outcome `ComposedTransport` falls back on. Classifying a malformed *caller
input* as `Unavailable` would let a composed transport silently serve the
request from a different adapter. `InvalidInput` is terminal under every
composition policy.

**Why the control is not on `CurlConfig`.** `CurlConfig`'s fields are public
and the struct is **not** `#[non_exhaustive]` (`lib.rs:58-76`), so adding a
field would be source-breaking for every consumer that constructs it with a
struct literal, and marking it `#[non_exhaustive]` would itself be
source-breaking. `CurlTransport`'s fields are private and its only
construction path is a constructor, so a private field is **not**
source-breaking. The consequence for a consumer of the published
`eggup-curl 0.1.2` was therefore **none**: the change was purely additive, and
a republication would have been a non-breaking `0.1.3`. M014 performed no
republication. The republication that did happen was the `0.2.0` train
(Acquisition Transport M017), and it is `0.2.0` rather than `0.1.3` because the
seam's own `0.2.0` is source-breaking and the workspace is version-lockstep:
the adapter inherits that break transitively. The User-Agent control itself
remains additive and source-compatible within `0.2.0`.

**The default does not change.** By default the adapter emits **no**
`--user-agent` element, so `curl` applies its own default and the bytes a
consumer of the published crate sends are unchanged. Defaulting to
`eggup-curl` instead would have changed the observable `User-Agent` header of
every existing consumer for no concrete benefit.

**Parity is capability parity, not placement parity.** The two controls live
in different types — `EggfetchConfig` (published `0.2.0`, whose `user_agent` is
a public field and cannot be moved without a source break) versus
`CurlTransport` (private field, additive). Each adapter keeps the placement
its own publication history allows, and neither may be changed to match the
other. The **default values** also still differ (`eggup-eggfetch` versus "none
sent"), deliberately; that difference is documented in
[acquisition.md](acquisition.md) §11.1 and
[eggfetch-adapter.md](eggfetch-adapter.md) §12.1 rather than papered over.

**How parity is enforced.** A mechanical cross-adapter parity test is
**impossible** in this workspace and none is claimed: `eggup-curl` must not
know `eggup-eggfetch` exists, `eggup-eggfetch` must not know `eggup-curl`
exists, and the seam must not depend on either adapter. A test asserting "both
adapters expose the same capability" would have to import both adapters into
one crate, violating the boundary in one direction or the other. Parity is
enforced instead by **per-adapter tests** (this adapter's six tests above
prove its own control; the eggfetch adapter's control is proven by its own
suite) plus the **documented contract** in [acquisition.md](acquisition.md)
§11.1, which a reviewer compares as two documented contracts rather than as a
shared assertion.

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
  — `AcquisitionTransport` (`acquisition/src/lib.rs:625-653`), `FetchLimits::effective`
  (`acquisition/src/lib.rs:143-151`), `AcquisitionError` (`acquisition/src/lib.rs:256-297`), `redact_url` (`acquisition/src/lib.rs:404-432`).
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
| `README.md:20` writes the status flag as `-w "%{stderr}%{http_code}"` | The code uses the long form `--write-out` (`lib.rs:723-724`). Same flag; the README is shorthand. |
| `acquisition/src/lib.rs:310-316` documents `__adapter_unavailable` as the constructor "`eggup-curl`" uses; `acquisition.md` §2 repeats that | **Resolved in M013.** `eggup-curl` used to construct `AcquisitionError::Unavailable(..)` directly at three call sites; all three now go through `__adapter_unavailable` (`lib.rs:513`, `:531`, `:776`), so the seam's bounding applies to them. Classification and content are unchanged. |
| `architecture/eggfetch-adapter.md:3` and `:77` cited `lib.rs` line numbers for `eggup-eggfetch` from an older revision (1180 lines) | **Resolved.** Every one of that document's 216 anchors was re-derived against the current 1229-line `eggup-eggfetch/src/lib.rs` and resolves; none needed repair. |
| `discover_in` reads "the first existing regular file" (`lib.rs:503-504`) | It tests `fs::symlink_metadata(...).is_file()` (`lib.rs:524-527`), which does **not** follow symlinks. A `curl` reachable only through a symlink in a `PATH` entry is skipped and the search continues to the next entry. A caller who needs a symlinked `curl` should pass the path to `with_executable` directly. |
| Nothing in `README.md`/`CHANGELOG.md` mentions the child environment beyond proxy keys | `env_clear()` (`lib.rs:771`) also withholds `HOME`, `PATH`, `SSL_CERT_FILE`, and `CURL_CA_BUNDLE` from curl, so a consumer's environment-based CA-bundle configuration is not honored. This is consistent with the documented "cleared environment" posture, but the CA-bundle consequence is unstated. |
| Nothing in the docs mentions it | The exact URL is passed on the child's command line for the life of the process (§7.4). `redact_url` covers diagnostics only. |
