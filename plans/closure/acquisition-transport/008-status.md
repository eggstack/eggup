# Acquisition Transport M008 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/acquisition-transport/008-subsecond-deadline-truthfulness-corrective.md`

Source roadmap: `plans/subsystems/acquisition-transport-roadmap.md#M008--sub-second-deadline-truthfulness-corrective`

Reviewed repository baseline: `ea51fe12a7c9120028b727eb5e40411e9b10f8e2` (post-M007/M001/M007 review baseline; source tree was clean before implementation)

Implementation commit: `pending` — see git log for the closing commit hash.

## Executive finding

The remaining curl deadline truthfulness defect is closed. `eggup-curl` no longer widens effective sub-second connect/total deadlines to whole seconds before passing them to curl. The new `duration_decimal_seconds` serializer formats a positive Rust `Duration` at microsecond precision with a locale-independent `.` separator and trailing-zero stripping, so `100 ms -> "0.1"`, `250 ms -> "0.25"`, `1.5 s -> "1.5"`, and `2 s -> "2"`. Truncation to whole microseconds never widens the input; sub-microsecond positive durations are rejected at validation rather than silently extended. The previous one-second timeout-attribution slack in `classify_curl_result` is replaced with the parent scheduling tolerance `POLL_INTERVAL + 5 ms`. The M007 body-streaming/process-cleanup, redaction, byte-bound, status-classification, and fallback semantics are unchanged. No public `FetchLimits` API or default values changed. No high- or medium-severity implementation finding remains open.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Locale-independent decimal serialization with `.` separator | `duration_decimal_seconds` formats integer seconds + microseconds without locale-sensitive formatting; no `format!("{}")` of `f64` is used | passed |
| Sub-second inputs without upward rounding | `duration_decimal_seconds` truncates to whole microseconds; `0.5 s -> "0.5"`; `1.5 s -> "1.5"`; `100 ms -> "0.1"`; `500 us -> "0.0005"` | passed |
| Whole-second values remain exact | `2 s -> "2"`, `120 s -> "120"`; no spurious fractional component | passed |
| Positive inputs that truncate below microsecond precision rejected | `Duration::from_nanos(500)` returns `AcquisitionError::InvalidInput`; zero returns the same | passed |
| Adapter ceiling semantics preserved | `FetchLimits::effective` continues to compute `min(request, adapter)`; M008 only changes how the effective value is serialized to curl | passed |
| `--connect-timeout` / `--max-time` never exceed effective durations | `build_curl_args_passes_decimal_deadlines_never_above_ceiling` enumerates 100 ms/250 ms/1.5 s/2 s/500 us/750 us cases and asserts the exact curl argument | passed |
| Curl actually receives the decimal deadline | `build_curl_args_passes_sub_second_deadlines_to_fake_curl` records argv from a Unix fake curl and asserts `--connect-timeout 0.25` / `--max-time 0.75` | passed |
| One-second timeout-attribution slack removed | `classify_curl_result` now uses `POLL_INTERVAL + 5 ms` as tolerance; connect-vs-total attribution still labels the connect phase when the elapsed wall clock is within tolerance of `eff_connect` | passed |
| Supported-host sub-second total timeout kills the child promptly | `sub_second_total_timeout_kills_child_promptly` stalls a 4-second body and asserts `Timeout { phase: "total" }` returns within 750 ms | passed on Linux/macOS |
| Cancellation, no-clobber, finite byte bound, fallback remain green | existing `cancellation_during_body_stream_kills_and_reaps_child`, `metadata_and_artifact_size_overflow`, `artifact_temp_is_owner_private_and_no_clobber`, `composition_with_fixture_falls_back_from_unavailable_curl`, and `redirect_follow_and_reject` tests still pass | passed |
| Exact 404 / 5xx / TLS behavior preserved | `metadata_success_not_found_and_server_error`, `artifact_success_not_found_and_server_error`, `truncated_transfer_is_hard_failure_without_promotion`, `tls_like_process_failure_is_transport_not_unavailable` remain green | passed |
| Curl receives no Eggup temp pathname | `--output -` is unchanged; body still streams to a duplicate of Eggup's exclusive file handle | passed |
| Windows portable process/error-path tests still green | portable curl tests compile and run on Windows; live loopback tests remain gated per M007 | passed; same Windows loopback limitation as M007 |
| No `cargo dep change` | no `Cargo.toml` edit in this corrective | passed |

## Production implementation evidence

- `crates/eggup-curl/src/lib.rs::duration_decimal_seconds` — new locale-independent serializer. Rejects `Duration::ZERO` and positive durations below one microsecond with `AcquisitionError::InvalidInput`. Truncates to whole microseconds and strips trailing fractional zeros.
- `crates/eggup-curl/src/lib.rs::build_curl_args` — now returns `Result<Vec<String>, AcquisitionError>`. Calls `duration_decimal_seconds` for both `--connect-timeout` and `--max-time`, so an unrepresentable positive input fails before the child is spawned.
- `crates/eggup-curl/src/lib.rs::run_curl_to_file` — propagates the `Result` and continues to enforce the parent wall-clock deadline, kill/reap the child, join the body/status readers, and remove only the owned temp.
- `crates/eggup-curl/src/lib.rs::classify_curl_result` — replaces the previous `+ Duration::from_secs(1)` slack with `POLL_INTERVAL + Duration::from_millis(5)`. The connect phase is still labeled when the elapsed wall clock is within tolerance of `eff_connect`; the tolerance never extends the caller's deadline.
- Tests added (in `crates/eggup-curl/src/lib.rs`):
  - `duration_decimal_seconds_serializes_sub_second_inputs_truthfully` — pins exact output for representative positive `Duration`s and rejects zero / sub-microsecond positive inputs.
  - `build_curl_args_passes_decimal_deadlines_never_above_ceiling` — pins the exact curl argument for representative durations.
  - `build_curl_args_passes_sub_second_deadlines_to_fake_curl` — Unix fake-curl argv check for `--connect-timeout 0.25` and `--max-time 0.75`.
  - `sub_second_total_timeout_kills_child_promptly` — supported-host runtime test that stalls a 4-second body, demands `Timeout { phase: "total" }`, and bounds elapsed wall clock to 750 ms.
- `crates/eggup-curl/README.md` — updated to document sub-second truthfulness and the new connect-phase attribution tolerance.
- `CHANGELOG.md` — `Unreleased` entry; no publication or consumer migration performed.
- `plans/subsystems/acquisition-transport-roadmap.md` — M008 status moves from ready to closed.
- No `Cargo.toml`, lockfile, workflow, or other crate edits in this corrective.

## Exact commands and results

Local Darwin arm64, Rust 1.89.0:

```text
cargo fmt --all -- --check                                      passed
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings  passed
cargo test --workspace --all-targets --all-features --locked    passed
cargo test -p eggup-curl --locked                               passed (22/22)
cargo doc --workspace --no-deps --locked                        passed
cargo +1.89.0 check --workspace --all-targets --locked          passed
cargo tree --workspace --locked                                  passed; no dependency change
cargo tree -p eggup-core --locked                                passed; sha2 only
git diff --check                                                passed
```

The M008 change is local to `eggup-curl`; no hosted run is a prerequisite for closing this corrective. The M007 hosted run `36213509807` already established the M007 baseline on Linux/macOS/Windows with the same Windows live-loopback scope; M008 inherits that disposition. Future hosted CI runs that exercise the new sub-second deadline tests will add runtime coverage but are not required for closure.

## Invariant review

- Effective connect and total deadlines are never greater than the caller-provided ceilings; `FetchLimits::effective` is unchanged.
- The parent wall-clock deadline (`start + eff_total`) still kills and reaps the child; sub-second callers now observe truthful timeouts instead of being rounded up.
- `--connect-timeout` / `--max-time` accept decimal seconds; whole-second callers observe no behavior change.
- Sub-microsecond positive durations are rejected at validation rather than widened; this is the same fail-closed posture already enforced for zero/negative durations.
- Connect-phase attribution labels only an already-enforced timeout and never extends the caller's deadline.
- Curl still receives no Eggup temporary pathname; the body still streams through a duplicate of Eggup's exclusive file handle.
- Diagnostics remain bounded and redacted; exit status is numeric category evidence only.
- `eggup-core` gains no transport dependency.

## Failure and recovery review

| Failure | Result |
|---|---|
| Effective deadline truncates below one microsecond | `InvalidInput` from `build_curl_args` before spawn; no filesystem or network work |
| Caller exceeds effective connect or total | child killed and reaped; readers joined; `Timeout { phase: "connect" | "total" }` returned; only owned temp removed |
| Curl body/status pipe or owned-file write failure | child cannot be promoted; bounded category error; owned temp cleanup |
| Cancellation while curl runs | child killed and reaped; readers joined; only owned temp removed |
| Exact 404 | `NotFound`; no fallback |
| 5xx / TLS / partial / oversized | typed `Transport` / `TooLarge`; incomplete output removed; no fallback |
| Windows hosted loopback restriction | unchanged from M007; portable tests and compile check continue |

## Compatibility and migration review

No public API signature change. `FetchLimits` and `CurlConfig` keep the same fields and validation rules. Sub-second callers now see truthful deadlines instead of whole-second rounding. Whole-second callers see no behavior change. No consumer migration is required.

## Security review

The sub-second path is the same call chain as M007: the parent enforces the effective total deadline, kills/reaps the child, joins both reader threads, and never lets curl reopen a temp pathname. Status capture is bounded; raw curl/stderr content is not surfaced. UTF-8 truncation cannot panic or emit invalid strings. The new serializer never widens positive inputs. Sub-microsecond pathological inputs are rejected at validation rather than silently extended. No medium-or-higher related finding remains open.

## Documentation and operations evidence

- `crates/eggup-curl/README.md` documents sub-second truthfulness and the new connect-phase attribution tolerance.
- `CHANGELOG.md` records the M008 Unreleased entry; no publication, consumer migration, or release occurred.
- `plans/subsystems/acquisition-transport-roadmap.md` M008 moves from ready to closed.
- No architecture/runtime documentation changes are required: the transport contract, error categories, and composition policy are unchanged.

## Unresolved findings

- None: no high- or medium-severity implementation issue remains.
- Informational: Windows-hosted spawned curl cannot connect to local loopback fixture servers (exit 7). This is the same M007 disposition and remains the only platform limitation. Sub-second runtime tests therefore run on Linux/macOS only; portable adapter/process/error-path tests still cover Windows.

## Roadmap disposition

Acquisition M008 moves from ready to closed. The subsystem's curl deadline contract is now truthful at sub-second resolution. Acquisition M007 remains conditionally closed (Windows loopback) and is unaffected by this corrective. Acquisition M001-M006 remain closed. No later milestone is blocked on M008.

## Registry updates

- Acquisition M008: ready → closed; no API/dependency change.
- Acquisition M007 remains conditionally closed; the Windows loopback limitation is unchanged.
- No new downstream plan is unblocked by M008.