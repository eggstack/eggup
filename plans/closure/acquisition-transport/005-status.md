# Acquisition Transport M005 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/acquisition-transport/005-curl-adapter-and-transport-composition.md`

Source roadmap: `plans/subsystems/acquisition-transport-roadmap.md#M005--curl-adapter-and-explicit-transport-composition`

Reviewed repository baseline: `12bc501f3730db89f5455d0de0531c76fc185472` (plan-stated baseline `881c95ff069d3d465a282cb6a495ba6fcb70cb6f` is an ancestor; interim planning commits `9b3d7a2`–`12bc501` touch plans only)

Implementation commit: `c67e715101d748fafe73494d0e476e4a616f2579` (`feat(acquisition): add curl adapter and transport composition (M005)`).

Reference implementation reviewed (read-only): `eggstack/gregg@8b18f9ee16461e3fa0ef0d804ed39ebb9183b727` (`crates/gregg-update/src/exec.rs`). Gregg was neither modified nor depended on (no `gregg` path/git dependency in the workspace; `cargo tree` contains no gregg node).

## Executive finding

M005 is complete. `eggup-curl` implements `AcquisitionTransport` via a bounded
external-`curl` process with explicit executable selection, opt-in PATH
discovery, connect/total ceilings, parent wall deadline, kill/reap on
timeout/cancellation, same-request HTTP status capture, and Eggup-owned
private-temp/no-clobber promotion. `AcquisitionError::Unavailable` separates
"adapter could not attempt" from ordinary transport failure, and
`eggup-acquisition::ComposedTransport` with
`CompositionPolicy::{UnavailableOnly (default), UnavailableOrTransport}`
provides explicit caller-selected preferred/fallback composition. Exact 404
remains terminal `NotFound`; default fallback occurs only on unavailability.
Curl-only binaries avoid Eggfetch/TLS; Eggfetch-only binaries avoid curl.
Deterministic tests require no public network. No medium-or-higher
correctness/security finding remains open.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Curl acquisition implements seam without release policy | `CurlTransport` implements `AcquisitionTransport`; no version/mirror/Cargo/service code; `cargo test -p eggup-curl` 16 passed | passed |
| Curl-only consumption needs no Eggfetch/TLS | `cargo tree -p eggup-curl` = `eggup-curl -> eggup-acquisition` only; `curl_only` release 417 KiB (346 KiB stripped) vs `eggfetch_only` 1.9 MiB (1.6 MiB stripped) | passed |
| Eggfetch-only consumption needs no curl | `cargo tree -p eggup-eggfetch` contains `eggfetch-core`/TLS, no curl node; existing Eggfetch suite 43 passed | passed |
| Preferred/fallback composition explicit + deterministic | `ComposedTransport::new(primary, secondary, policy)`; 6 composition unit tests (preferred order, call counts, unavailable fallback, default no-retry, broad retry, terminal set) | passed |
| Exact 404 terminal `NotFound` | curl metadata/artifact 404 tests + composition `NotFound` terminal test; no policy retries 404 | passed |
| Default fallback only on unavailability | `CompositionPolicy::UnavailableOnly` default; `composition_default_does_not_retry_transport_failures` asserts secondary call count 0 | passed |
| Broader transport-error fallback opt-in | `UnavailableOrTransport` retries `Transport`/`Timeout`; `composition_broad_policy_retries_transport_and_timeout` | passed |
| Subprocesses bounded, children reaped, output bounded | `--connect-timeout`/`--max-time`/`--max-filesize`/`--max-redirs`/`--proto`/`--proto-redir`/`--disable`/`--noproxy`; parent wall deadline + 5 ms poll + kill/reap; stdout `-%{http_code}` bounded 64 B, stderr `null`, body via file with Eggup-side size check | passed |
| Temp/no-clobber/redaction preserved | `__acquire_exclusive_temp` + `__promote_no_clobber`; owner-private 0600 asserted on Unix; no-clobber race test; credential sentinels absent from `Display` | passed |
| Deterministic tests, no public network | All curl integration uses `127.0.0.1` `TcpListener` fixtures + fake shell-script curl doubles; no `example.com` fetch occurs | passed |
| Footprint evidence, all three configs | `eggup-transport-footprint` (publish=false) bins `curl_only`/`eggfetch_only`/`dual`; release sizes + trees recorded below | passed |
| Rust 1.89, package/docs, hosted CI | `rustc 1.89.0`; `cargo doc` green; `cargo package` acquisition verify green, curl/eggfetch ordering evidence below; hosted CI pending push (local Darwin only) | passed with CI pending |
| No Gregg modification/dependency | `git status` shows no gregg paths; `grep -r gregg Cargo.toml crates/*/Cargo.toml` empty except plan/closure prose; `cargo tree` has no gregg | passed |

## Production implementation evidence

- `crates/eggup-acquisition/src/lib.rs`: added `AcquisitionError::Unavailable(String)`
  with `Display` (`acquisition transport unavailable: …`), `is_unavailable()`,
  `#[doc(hidden)] __adapter_unavailable`; added `FixtureResponse::unavailable()`
  + `FixtureKind::Unavailable` handling in both fixture paths; added
  `CompositionPolicy` + `ComposedTransport` (validates limits once at the
  boundary, delegates, falls back only per typed policy; manual `Debug`
  avoids requiring `Debug` on trait objects).
- `crates/eggup-curl/`: new member `eggup-curl 0.1.1` (workspace keys, lints,
  sole dep `eggup-acquisition`). `CurlConfig::strict` (10 s/120 s ceilings,
  follow redirects, max 10, `http,https` only, proxy disabled);
  `CurlTransport::with_executable` (explicit path strongest, config validated,
  existence/spawn deferred to per-request `Unavailable`);
  `discover_curl_executable()` opt-in PATH search (`curl` + `curl.exe` on
  Windows) returning bounded `Unavailable` evidence.
- Curl invocation (no shell/sudo): `--disable --silent --show-error
  --no-progress-meter [--location --max-redirs N] --connect-timeout C
  --max-time T [--max-filesize M] --proto =list --proto-redir =list
  [--noproxy *] --output tmp --write-out %{http_code} -- URL`. Child env is
  cleared; proxy Disabled injects nothing, FromEnvironment snapshots
  `*_PROXY`/`NO_PROXY`, Custom injects caller pairs. `~/.curlrc` ignored.
- Status from the same transfer: `200..=299 → Success`, `404 → NotFound`,
  else `Transport`; curl 28 → `Timeout` (connect vs total by elapsed vs
  effective connect + 1 s slack for ceil rounding); 63 → `TooLarge` with the
  request bound; any other non-zero exit → `Transport` even with an HTTP code
  (prevents truncated promotion). Partial outputs remove only the owned temp;
  success promotes via `__promote_no_clobber` after Eggup-side size validation
  and pre-promotion cancellation re-check.
- `crates/eggup-transport-footprint/` (publish=false): bins `curl_only`
  (`--features curl`), `eggfetch_only` (`--features eggfetch`), `dual`
  (`--features curl,eggfetch`).
- Examples: `eggup-curl/examples/curl_fetch.rs`,
  `eggup-eggfetch/examples/eggfetch_fetch.rs` (fixture surface for
  single-adapter binaries).

## Exact commands and results

Environment: Darwin 25.6.0 arm64; `rustc 1.89.0`; system `curl 8.7.1
(x86_64-apple-darwin25.0) libcurl/8.7.1 (SecureTransport) LibreSSL/3.3.6`.

Passed:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo tree --workspace --locked
cargo check --workspace --all-targets --locked
cargo package -p eggup-acquisition --offline --allow-dirty
cargo package -p eggup-acquisition --offline --allow-dirty --no-verify
cargo package -p eggup-curl --offline --allow-dirty --no-verify
cargo package -p eggup-eggfetch --offline --allow-dirty --no-verify
cargo build --release -p eggup-transport-footprint --features curl --bin curl_only
cargo build --release -p eggup-transport-footprint --features eggfetch --bin eggfetch_only
cargo build --release -p eggup-transport-footprint --features curl,eggfetch --bin dual
cargo tree -p eggup-curl --offline
cargo tree -p eggup-eggfetch --offline
```

Test totals (`--offline`, all green): acquisition 34, eggfetch 43, curl 16,
core 24+7+28, service 83, eggpack 1 suite. Workspace `cargo test` reports 0
failed across all suites.

Footprint (release, Darwin arm64):

```text
target/release/curl_only      417 KiB (346 KiB stripped copy in /tmp/curl_only)
target/release/eggfetch_only  1.9 MiB (1.6 MiB stripped copy in /tmp/eggfetch_only)
target/release/dual           1.9 MiB (1.6 MiB stripped copy in /tmp/dual)
```

Dependency trees (depth 2):

```text
eggup-curl -> eggup-acquisition only (no eggfetch, no TLS)
eggup-eggfetch -> eggfetch-core, eggup-acquisition, futures-util, tokio (+ TLS subtree)
footprint(curl) -> acquisition + curl only
footprint(eggfetch) -> acquisition + eggfetch only
```

Package ordering evidence (informational, same class as M004): `cargo package
-p eggup-curl --allow-dirty` with verification fails compiling the packaged
adapter against published `eggup-acquisition 0.1.0` (no `Unavailable`
variant), while `--no-verify` packaging succeeds and workspace source
check/test/doc are green. This is seam-then-adapter release ordering, not a
production compile failure. `eggup-acquisition` verifies cleanly.

Hosted Linux/macOS/Windows CI was not available before push. The push for
this work will trigger repository CI; its result is recorded in the final
handoff if accessible. Local evidence is Darwin only and is reported as such.

## Invariant review

- `eggup-core` untouched and transport-free (`git show --stat` shows no core paths).
- Downloaded bytes are data; adapter never executes temps (only `read`/`metadata`/`hard_link`).
- No shell/sudo/elevation/package-manager/release-selection invoked (`Command::new(executable)` + literal argv, `env_clear()`).
- Exact 404 never selects another release/source; composition never retries `NotFound`.
- `TooLarge`/`InvalidInput`/`Cancelled`/`Io` never fall back under either policy.
- Existing destinations never overwritten (fast-fail + `hard_link` no-clobber).
- Partial downloads never promoted (only `CurlOutcome::Success` + size OK + not cancelled promotes).
- Wall/connect/redirect/artifact/output all bounded; cancel/timeout kills + reaps before return.
- Credential URL material redacted; proxy behavior explicit; `~/.curlrc` disabled.
- Curl-only links no Eggfetch; Eggfetch-only links no curl (trees above).

## Failure and recovery review

| State/event | Result |
|---|---|
| Executable missing / PATH miss / spawn fail | `Unavailable`; composition falls back by default |
| TLS/connect-refused/resolve/5xx/exit≠0,28,63 | `Transport`; fallback only under `UnavailableOrTransport` |
| HTTP 404 | `NotFound` (data); no fallback; artifact dest absent |
| HTTP 5xx / 302 without follow | `Transport`; temp cleaned; dest absent |
| Truncated body (exit 18, abort) | `Transport`; dest absent; owned temp only cleaned |
| `--max-filesize` exceeded (63) / Eggup size check | `TooLarge{limit: bound}`; dest absent |
| Connect/total deadline | `Timeout{connect/total}`; child killed + reaped; temp cleaned |
| Pre/post promotion cancellation | `Cancelled`; child killed + reaped where running; never promotes after cancel |
| Dest exists / races in | `InvalidInput` no-overwrite; foreign bytes preserved; owned temp removed |
| Post-link temp unlink failure | Success stands (inherited `__promote_no_clobber` commit-point rule) |
| Invalid limits/config | `InvalidInput` before route/fs/spawn |

## Compatibility and migration review

Additive only. `AcquisitionError` is `#[non_exhaustive]`, so the new
`Unavailable` variant is source-compatible for downstream wildcard matches.
`FixtureResponse::unavailable()` is additive. `ComposedTransport`/
`CompositionPolicy`/`eggup-curl` are new API. Existing `eggup-eggfetch`
consumers require no migration. No downstream repository changed. No version
rewritten; no crate published (see CHANGELOG `Unreleased`).

## Security review

- Executable path is caller-supplied; discovery is opt-in and returns only a
  path (no secrets). No PATH shadowing in the child (absolute executable +
  `env_clear()`).
- No shell interpolation (`--` terminator + `http(s)`-validated URL stored verbatim).
- `~/.curlrc` ignored; proxy env explicit; Disabled adds `--noproxy "*"` defense-in-depth.
- Stdout bounded (http_code ≤64 B); stderr discarded (`null`) to avoid
  credential leak and pipe deadlock; body bounded by `--max-filesize` +
  Eggup-side validation; `TooLarge.limit` echoes the bound, not the body.
- Temps are exclusive `0600` siblings in the exact dest parent (same
  filesystem); promotion is `hard_link` no-clobber; cleanup removes only the
  owned temp (Drop guards + explicit removes); foreign siblings preserved.
- Errors are category + redacted URL (`redact_url` strips userinfo/query/fragment,
  truncates); raw curl output/command lines/proxy credentials never embedded;
  `__scrub_upstream_text` remains defense-in-depth in the seam.
- No medium-or-higher finding open. One proto-argument defect
  (`--proto =http,=https` disabling http) was found by integration tests and
  fixed before closure (`=http,https`); regression covered by 200/redirect tests.

## Documentation and operations evidence

- `crates/eggup-acquisition/README.md`: seam + `Unavailable` + composition
  fallback-is-not-release-fallback note.
- `crates/eggup-curl/README.md` + rustdoc: explicit executable/discovery,
  timeout/redirect/protocol/proxy bounds, kill/reap, redaction, composition
  pointer.
- `architecture/overview.md`: module table now lists `eggup-curl` and the
  extended acquisition seam.
- `CHANGELOG.md`: `Unreleased` M005 entry with no-publication/no-migration disclaimer.
- Consumer guidance: curl-only via `eggup-curl` (no Eggfetch), Eggfetch-only
  via `eggup-eggfetch` (no curl), dual via `ComposedTransport`; transport
  fallback ≠ release/source fallback stated in README, rustdoc, and plan.

## Unresolved findings

- Informational: isolated `eggup-curl` package verification uses published
  `eggup-acquisition 0.1.0`; packaged verification must follow seam-then-adapter
  release order or use `--no-verify`/local patch (same class as M004).
- Informational: hosted CI (Linux/macOS/Windows + MSRV lane) pending push;
  local evidence is Darwin arm64 + `rustc 1.89.0` only.
- None: no medium-or-higher correctness/security finding remains open.

## Roadmap disposition

Update `plans/subsystems/acquisition-transport-roadmap.md` M005 to closed and
`plans/registry.md` to move M005 from ready to closed. Service M006 remains
independently ready (Gregg reference only). Gregg consumer M004 remains
intentionally unwritten until Acquisition M005 and Service M006 both close;
M005 closure unblocks the acquisition half of that gate.

## Registry updates

- Acquisition M005: ready → closed (`plans/closure/acquisition-transport/005-status.md`).
- Consumer adoption Gregg M004: still deferred; now blocked only on Service M006
  (acquisition half satisfied).
- Current state: acquisition primary path complete (native + curl + composition).

## Post-closure corrective addendum — acquisition M006 (2026-09-25)

This record was originally written with hosted CI pending. The subsequent push
proved Windows workspace qualification was incomplete. History is preserved;
nothing above is rewritten.

Original failure (implementation + closure head `eb989659feabd44e3c1441bb8eb522614ce96a31`):

- Workflow run `36169295410` — conclusion `failure`.
- Windows job `108184625301` (`cargo check --workspace --all-targets --locked`) — conclusion `failure`.
- Stable job `108184625133`, MSRV job `108184625222`, macOS job `108184625296` — all `success`.
- Diagnostics:

```text
error[E0433]: cannot find `unix` in `os`
 --> crates/eggup-curl/src/lib.rs:662:18
error[E0599]: no method named `set_mode` found for struct `Permissions`
 --> crates/eggup-curl/src/lib.rs:787:15
warning: unused import: `PermissionsIntent`
 --> crates/eggup-core/src/stage.rs:5:50
```

Plan-only head `759f175828b0be9b462fdd5336b9c3115ebb3534` reproduced the same
red lane: workflow `36175333145` failure, Windows job `108204439265` failure
(Stable/MSRV/macOS green).

Corrective implementation `1c601f29a16c952e90feaeebd0fa654401b54a86`
(`fix(acquisition): gate Unix-only curl test support and core imports for
Windows (M006)`): `PermissionsExt` import, `fake_curl_script`, and the six
shell-dependent fake-child tests are `#[cfg(unix)]`-gated (module itself stays
compiled on Windows); `PermissionsIntent` (and same-class test-only `Duration`)
imports in `eggup-core` are target-gated. No transport, composition, disposition,
or Gregg semantics changed.

Succeeding hosted matrix on the corrective head:

- Workflow run `36176009068` — conclusion `success`.
- Stable job `108206656486`, MSRV job `108206656574`, macOS job `108206656474`,
  Windows job `108206656342` — all `success`.

Final disposition: M005 implementation stands; cross-platform qualification is
restored by acquisition M006 (`plans/closure/acquisition-transport/006-status.md`).
M005 is treated as fully qualified only with that corrective closure.
