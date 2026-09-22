# Acquisition Transport M001 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/acquisition-transport/001-acquisition-seam-and-fixture-transport.md`

Source roadmap: `plans/subsystems/acquisition-transport-roadmap.md#M001--acquisition-seam-and-fixture-transport`

Reviewed repository baseline: `9f527beb20da585bc3cf56f45f1fd96fd418c124` (plan baseline; implementation ran at M006 closure `8aab5bd` plus seam changes)

## Implementation commits/PRs

- M001 seam commit (this pass): `feat: add acquisition seam and fixture transport (M001)` (pending SHA; see git log)
- Prior: `8aab5bd` (`feat: qualify eggup-core package (M006)`)
- No PR was required for this local implementation pass.

## Executive finding

M001 is complete. The transport-neutral acquisition seam is small enough that an Eggfetch implementation and a lightweight implementation can both satisfy it without policy leakage. `eggup-core` remains transport-free; deterministic fixture coverage proves the contract without network access.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| New transport package boundary without contaminating core | `crates/eggup-acquisition` added to workspace; `eggup-core` manifest untouched (`cargo tree -p eggup-core` → `sha2` only) | passed |
| Typed request descriptors | `AcquisitionRequest::new` (exact URL, http(s) only, 8 KiB bound, control-char rejection) | passed |
| Typed `Success \| NotFound \| Failure` | `FetchOutcome::{Success, NotFound}` + `Err(AcquisitionError)`; `NotFound` is data, never fallback | passed |
| Bounded small-body fetch | `fetch_metadata` with `max_metadata_bytes`; `metadata_limit_is_enforced` | passed |
| Streamed file fetch | `fetch_artifact` chunked to temp sibling + atomic rename; `streamed_artifact_success`, `artifact_limit_is_enforced_without_promotion` | passed |
| Cancellation/drop cleanup | `CancelFlag`; temp guard deletes `.part` on failure/cancel; `cancellation_cleans_partial_output`, `partial_write_failure_promotes_nothing` | passed |
| Deterministic fixture transport | `FixtureTransport` in-memory routes; no network; unregistered URL is hard failure (`no_fallback_on_unregistered_url`) | passed |
| Progress callback only if required | Omitted: no consumer evidence required it; seam stays to two operations | passed |
| No release/fallback/version/destination/service policy | Trait takes exact URL + dest path; no SemVer/Cargo/GitHub/service types; `grep` clean | passed |
| Redaction | `redact_url` (credentials + query stripped, 256-char bound); `diagnostics_are_redacted` | passed |
| Exact body success | `exact_body_success` | passed |
| Timeout/cancellation abstraction | `FetchLimits{connect,total}`, `Timeout{phase}`, `Slow` fixture; `timeout_is_a_hard_failure` | passed |
| Partial write failure | `Truncated` fixture; `partial_write_failure_promotes_nothing` | passed |
| No fallback invocation | Unregistered URL + 500 + truncation all hard-fail; never `NotFound`; `hard_failures_never_become_not_found` | passed |
| Missing parent fails without creation | `missing_artifact_parent_fails_without_creation` | passed |

## Consumer-call mapping

| Consumer | Current download calls | Seam mapping | Notes |
|---|---|---|---|
| eggsact `src/update.rs` | `eggfetch_core::Client` bounded metadata (checksum/manifest) + streamed binary; `RedirectPolicy`, `Timeout{connect,total}`, `ProxyEnvironment`, `redact_url_string`, `DecodedBodyTooLarge` | `fetch_metadata` (checksum/manifest, `TooLarge` ↔ `DecodedBodyTooLarge`) + `fetch_artifact` (binary stream, atomic promotion) | Release/fallback (`Cargo` conditions) stays in eggsact; transport never falls back |
| stegoeggo `stegoeggo-cli/src/update.rs` | `eggfetch_core::Client` via `ProxyEnvironment::from_env`, `Timeout::builder`, `RedirectPolicy::strict`, `BodyTooLarge` mapping | Same two-op mapping; `Strict` redirect + proxy + timeout policy moves to M002 adapter, not the seam | Second independent consumer proves genericity |
| eggsearch | `Eggfetch`-family updater + service lifecycle (per roadmap) | Same two ops; service composition explicitly out of scope for M001 | Validates no service/manager types in seam |

No GitHub JSON, SemVer, Cargo, service-manager, or second staging model was required. Had any been required, the plan's stop condition would have triggered; it did not.

## API surface

```rust
AcquisitionRequest::new(url) -> Result<_, AcquisitionError>
FetchLimits { max_metadata_bytes, max_artifact_bytes, connect_timeout, total_timeout }
CancelFlag::{new, cancel, is_cancelled}
FetchOutcome<T>::{Success(T), NotFound}
MetadataBytes::{bytes, len}
ArtifactEvidence::bytes_written
AcquisitionError::{InvalidInput, Transport, TooLarge{limit}, Timeout{phase}, Cancelled, Io}
AcquisitionTransport::{fetch_metadata, fetch_artifact}
FixtureTransport::{new, route}
FixtureResponse::{body, not_found, failure, truncated, slow}
redact_url(url) -> String
```

Sync shape chosen: no async runtime in the seam, so lightweight (curl-style) and Eggfetch adapters can both satisfy it without forcing async into `eggup-core`. M002 bridges async Eggfetch behind this sync contract with explicit timeout/proxy/redirect policy.

## Exact tests/commands actually run

Environment: `Darwin 25.6.0 arm64`, stable `1.98.1`, MSRV `1.89.0`.

```text
cargo +1.89.0 fmt --all -- --check
cargo +1.89.0 check --workspace --all-targets --locked
cargo +1.89.0 clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo +1.89.0 test --workspace --all-targets --all-features --locked
cargo +1.89.0 doc --workspace --no-deps --locked
cargo +1.89.0 tree --workspace --locked
cargo +1.89.0 package --workspace --locked --allow-dirty
./scripts/check-local.sh
```

Results: 12/12 `eggup-acquisition` tests + 37/37 `eggup-core` tests passed on stable and 1.89. `cargo tree -p eggup-acquisition` shows zero dependencies. `cargo tree -p eggup-core` shows `sha2` only. `cargo package --allow-dirty` green for both crates.

## Invariant review

- `eggup-core` transport-free: proven by dependency tree + manifest diff (no change).
- No release/fallback: exact-URL only; 404 never falls back; hard failures never become `NotFound`.
- No execution: bytes are written, never spawned; no `Command` in acquisition crate.
- Bounded: metadata in-memory cap, artifact disk cap, URL 8 KiB, detail 512 chars, redact 256 chars.
- Streamed: 8 KiB chunked writes to `.part` + rename; no full buffering required.
- Partial cleanup: guard deletes temp on failure/cancel; `dest` never partially promoted.
- Redaction: credentials/query stripped in all diagnostics.

## Failure/recovery review

- `NotFound` returns `Ok(NotFound)` with no file created.
- `TooLarge`/`Timeout`/`Cancelled`/`Transport`/`Io` return `Err` with no promotion.
- Truncation simulates partial write then hard failure; temp cleaned, `dest` absent.
- Missing/symlinked parent fails without creation.
- Cancellation before or during streaming fails closed with cleanup.

## Compatibility/migration review

New crate; no existing consumers to migrate. The sync seam is the stable contract for M002 (Eggfetch) and any future lightweight adapter. No breaking change to `eggup-core`.

## Security review

- No network in M001 (fixture only); no TLS/proxy policy invented here (M002 owns it).
- URL validation rejects control chars, non-http(s), overlong input.
- Parent must be an existing real directory; no `create_dir_all` on live paths.
- Atomic promotion prevents partial artifacts from becoming verified candidates.
- Redaction prevents credential/token leakage in errors.
- No privilege escalation; `#![forbid(unsafe_code)]`.

## Docs/operations evidence

- `crates/eggup-acquisition/README.md`: seam contract, two operations, no-policy rule.
- Rustdoc on every public type/method with bounds, failure, and no-fallback semantics.
- `cargo doc` clean with `#![deny(missing_docs)]`.
- Dependency trees recorded above.

## Unresolved findings with severity

- None (M001 scope). Real HTTPS/redirect/proxy/TLS policy is explicitly deferred to M002.
- Informational: Hosted CI not run locally; lanes cover stable/MSRV/macOS/Windows-check.

## Disposition and roadmap transition

M001 is closed. Unblocks acquisition M002 (Eggfetch adapter). Core M006 remains closed; service M001 remains independently ready. Consumer adoption remains blocked until M002 closes (plus core M006, already closed).
