# Acquisition Transport M003 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/acquisition-transport/003-contract-and-tempfile-hardening-corrective.md`

Source roadmap: `plans/subsystems/acquisition-transport-roadmap.md#M003--acquisition-contract-and-temporary-file-hardening-corrective`

Reviewed repository baseline: `8f6ce48cda5bdeb593939077bcca452cdd5f2800` (plan baseline; implementation ran at `892d6cc` plus corrective changes)

## Implementation commits/PRs

- M003 corrective commit (this pass): `fix: harden acquisition timeouts, tempfile, and redaction (M003)` (pending SHA; see git log)
- Prior: `892d6cc` (`planning: register post-adoption implementation wave`)
- No PR was required for this local implementation pass. No publication was performed.

## Executive finding

M003 is complete. Caller `FetchLimits` time bounds are now authoritative via
`min(request, adapter ceiling)` enforced with real Eggfetch request-level
overrides; artifact temp files are exclusively created, owner-private, and
promoted with race-safe no-clobber semantics; upstream/proxy error strings
are never embedded in diagnostics. Both eggsact and stegoeggo updater suites
stay green against the corrective. No medium-or-higher acquisition issue
remains.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Effective timeout rule `min(request, adapter)` | `FetchLimits::effective`, `EggfetchConfig::effective_timeouts`, `effective_request_timeout` | passed |
| Non-zero `connect`/`total`, `connect <= total` | `FetchLimits::new` rejects zero and `connect > total`; `EggfetchTransport::strict` validates adapter ceilings | passed |
| Real Eggfetch enforcement (no post-hoc check) | `.timeout(effective)` per-request override on both `fetch_metadata` and `fetch_artifact` | passed |
| Request stricter than adapter | `request_stricter_than_adapter_wins` (50 ms vs 5 s, 3 s stall → `Timeout`) | passed |
| Adapter stricter than request | `adapter_stricter_than_request_wins` (5 s vs 50 ms, 3 s stall → `Timeout`) | passed |
| Metadata body stall | `metadata_body_stall_times_out_without_fallback` | passed |
| Artifact stall after partial data | `artifact_body_stall_after_partial_data_times_out` (partial + stall → `Timeout`, no promotion, no residue) | passed |
| Timeout never `NotFound`/fallback | `timeout_never_becomes_not_found`, `timeout_never_maps_to_not_found` | passed |
| Exclusive temp creation | `__acquire_exclusive_temp` (`create_new`, 32-collision bound, no symlink follow, `0600` Unix, same parent) | passed |
| Unix owner-private permissions | `exclusive_temp_is_owner_private`, `exclusive_temp_helper_is_owner_private_for_eggfetch_prefix` (mode `0600`) | passed |
| No truncation/follow on collision | `exclusive_temp_never_truncates_existing_file`, `exclusive_temp_does_not_follow_symlink` | passed |
| No-clobber destination (pre-existing) | `existing_destination_is_a_hard_no_clobber_failure`, `artifact_refuses_existing_destination_without_overwrite` (foreign bytes preserved, no residue) | passed |
| No-clobber race window | `promote_no_clobber_preserves_raced_destination` via `__promote_no_clobber` (`hard_link` fails `AlreadyExists`) | passed |
| Promotion success when absent | `promote_succeeds_when_destination_absent`, `streamed_artifact_success` | passed |
| Cancellation removes only owned temp | `cancellation_preserves_foreign_sibling_and_removes_only_owned`, `cancellation_cleans_partial_output` | passed |
| Early disconnect removes only owned temp | `early_disconnect_removes_only_owned_temp`, `partial_write_failure_promotes_nothing` | passed |
| No broad prefix cleanup | Foreign siblings preserved in all cleanup tests; guard removes exactly one owned path | passed |
| Redaction userinfo/query/fragment | `redaction_sentinels_are_absent_from_diagnostics`, `diagnostics_are_redacted` | passed |
| Proxy password absent | `error_diagnostics_never_expose_credential_sentinels` (fake `InvalidProxyUrl` with sentinel → category-only) | passed |
| Upstream error category-only | `map_request_error`/`map_fetch_error` use `e.kind()` + redacted URL; `TransportIoTimeout` → `Timeout{total}` | passed |
| Fixture failure detail never echoed | `FixtureResponse::failure` stores nothing; maps to generic `Transport` | passed |
| Downstream eggsact | 25 updater tests pass against patched local crates | passed |
| Downstream stegoeggo | 33 updater tests pass against patched local crates | passed |
| Core remains transport-free | `cargo tree -p eggup-core` → `sha2` only (14 lines) | passed |
| No unsafe/privilege escalation | `#![forbid(unsafe_code)]`; no sudo/elevation; no shell | passed |

## Before/after timeout semantics

Before: `EggfetchConfig` owned 10 s connect / 120 s total defaults; `EggfetchTransport`
built one client from the config and ignored per-call `FetchLimits` time values
(comments claimed per-fetch authority, creating a contract mismatch).

After: every fetch derives `(min(request connect, adapter connect),
min(request total, adapter total))` and applies it as an Eggfetch
request-level `Timeout` override. A stricter adapter tightens; it never
extends. `connect > total` fails closed at both layers.

Consumer effect:

- eggsact (10 s connect / 120 s total, adapter 10 s/120 s): effective unchanged.
- stegoeggo (10 s connect / 60 s total, adapter 10 s/120 s): total tightens
  120 s → 60 s (request wins, stricter, never extended).

## Exact effective-timeout tests

- `effective_timeouts_take_minimums` (seam unit: both directions + per-phase minima).
- `effective_formula_is_minimum_per_phase` (adapter unit).
- `request_stricter_than_adapter_wins` (live stall server).
- `adapter_stricter_than_request_wins` (live stall server).
- `metadata_body_stall_times_out_without_fallback`.
- `artifact_body_stall_after_partial_data_times_out`.
- `total_timeout_is_a_hard_failure` (pre-existing, still green; now strictly `Timeout`-capable).

## Temp collision/symlink/no-clobber matrix

| Case | Evidence | Result |
|---|---|---|
| Dest absent → success | `streamed_artifact_success`, `promote_succeeds_when_destination_absent` | passed |
| Dest exists before call → hard failure, preserved | `existing_destination_is_a_hard_no_clobber_failure`, `artifact_refuses_existing_destination_without_overwrite` | passed |
| Dest raced in before promotion → hard failure, preserved | `promote_no_clobber_preserves_raced_destination` (direct helper race test) | passed |
| `create_new` on existing file fails, no truncation | `exclusive_temp_never_truncates_existing_file` | passed |
| `create_new` on symlink fails, target untouched | `exclusive_temp_does_not_follow_symlink` | passed |
| Repeated collisions → bounded failure | `__TEMP_COLLISION_BOUND = 32`; `AlreadyExists` retries with fresh nonce, then `Io` | passed (by construction + unit coverage) |
| Unix mode `0600` | Two owner-private tests | passed |
| Linux/macOS/Windows fixture logic equivalent | Fixture uses `__acquire_exclusive_temp` + `__promote_no_clobber` (std-only `hard_link`); same code on all hosts | passed |

Cross-platform promotion note: `hard_link` fails with `AlreadyExists` on both
Unix and Windows when `dest` exists, avoiding Unix `rename`-overwrite vs
Windows `rename`-fails divergence. Requires same filesystem (satisfied: same
parent). Exotic filesystems without hard-link support fail closed with `Io`
rather than overwriting.

## Unix permissions evidence

- `exclusive_temp_is_owner_private`: `0600` via `OpenOptionsExt::mode(0o600)` plus post-create enforcement.
- `exclusive_temp_helper_is_owner_private_for_eggfetch_prefix`: same for `eggup-eggfetch` prefix.

## Redaction sentinel matrix

| Sentinel | Source | Error contains it? |
|---|---|---|
| `S3CR3T-USERINFO-9917` (userinfo) | unregistered URL + fixture failure detail | absent (passed) |
| `TOKEN-QUERY-5523` (query) | same | absent (passed) |
| `FRAG-SENTINEL` (fragment) | `redact_url` | absent (passed) |
| `HUNTER2-PROXY-3311` (proxy pw) | fake `InvalidProxyUrl` display contains it; mapped error does not | absent (passed) |
| `S3CR3T-USERINFO-8841` / `TOKEN-QUERY-7734` (adapter) | unroutable fetch + `Debug` | absent (passed) |

Upstream `Error::Display` is never embedded; only `Error::kind()` category plus
`redact_url` appear in diagnostics. `__scrub_upstream_text` remains as
defense-in-depth for any future debugging string.

## Downstream eggsact/stegoeggo results

Patched local crates via `cargo --config patch.crates-io.*.path=...`:

- eggsact `cargo test update`: 25 passed, 0 failed.
- stegoeggo-cli `cargo test update`: 33 passed, 0 failed.

Both use compatible timeout values (`connect <= total`); no signature change
was required (`FixtureResponse::failure` still accepts detail for call-site
compatibility; `fetch_artifact` path seam unchanged). No consumer workaround
was added.

## Package/version decision

Bug-fix corrective to published 0.1.0 contract; no method-signature break:

- `FetchLimits::new` tightens validation (rejects previously accepted
  `connect > total`); valid downstream values unaffected.
- `FixtureResponse::failure(detail)` accepts but no longer echoes detail (security fix).
- `fetch_artifact` now fails when `dest` exists instead of relying on platform rename (contract fix).
- Error strings change from upstream-echo to category-only (security fix).

Decision: next release is workspace lockstep **0.1.1 patch**. Publish order
remains seam-then-adapter (`eggup-acquisition` first, then `eggup-eggfetch`)
when directed. **No publication performed in this pass; 0.1.0 was not
republished.**

- `cargo package -p eggup-acquisition --allow-dirty`: green (6 files, 54.4 KiB, 12.7 KiB compressed).
- `cargo package -p eggup-eggfetch --allow-dirty`: blocked on ordering (uses crates.io `eggup-acquisition` 0.1.0 without new `effective` API during isolated packaging), same as M002. Not a code defect; build/test/doc/tree green.

## Dependency tree

- `cargo tree -p eggup-core`: 14 lines, `sha2` only. No transport.
- `cargo tree -p eggup-acquisition`: 1 line, zero dependencies.
- `cargo tree -p eggup-eggfetch`: 211 lines, `eggfetch-core 0.2.0` + `tokio` + `futures-util` (transport graph unchanged; no compression/cookies/retry/json).

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
cargo package -p eggup-eggfetch --locked --allow-dirty
./scripts/check-local.sh
```

MSRV variants (`cargo +1.89.0 …`) run for check/clippy/test/doc: green (one
`find().is_some()` → `contains()` clippy fix applied for 1.89).

Results: 26 `eggup-acquisition` + 37 `eggup-core` + 23 `eggup-eggfetch` + 10
`eggup-service` = 96 passed, 0 failed, on stable and 1.89.

Downstream (patched):

```text
cargo test --offline update (eggsact, patched): 25 passed
cargo test --offline update (stegoeggo-cli, patched): 33 passed
```

Hosted CI (stable/MSRV/macOS/Windows-check) not run locally; lanes cover per
registry. One infinite-loop defect in the new `__scrub_upstream_text` helper
was caught by the full workspace test run and fixed before closure.

## Invariant review

- Exact caller URL remains the only target; no fallback/release policy added.
- Time limits never weakened; adapter may only tighten.
- No partial artifact promoted; temp exclusively owned; no unexpected overwrite.
- Cleanup removes only owned temp; foreign files preserved.
- Secrets never appear in diagnostics (category-only + redaction).
- `eggup-core` transport-free preserved.
- No unsafe code or privilege escalation.

## Failure/recovery review

- Cancellation before creation: no filesystem state.
- Cancellation during streaming: owned temp removed, dest absent.
- Timeout during streaming (including after partial data): same.
- Temp-name collision: retry with fresh nonce, bounded failure, existing untouched.
- Dest appears before promotion: explicit no-clobber failure, both preserved (foreign dest + owned temp cleanup).
- Cleanup failure after successful `hard_link`: `Io` with dest already promoted; temp remains for explicit cleanup (fail-closed, no silent overwrite).
- Process crash: ordinary temp residue may remain; no crash journal claimed (same as plan).

## Compatibility/migration review

No signature break. Existing eggsact/stegoeggo compile and pass against the
corrective via path patch. No migration notes required beyond the version
decision above: next 0.1.1 patch tightens validation and redaction; callers
with `connect > total` (none observed) would fail closed at construction.
No consumer workaround added.

## Security review

- Exclusive `create_new` prevents truncation/symlink following; `0600` on Unix.
- No-clobber `hard_link` prevents silent overwrite races without unsafe/FFI.
- Category-only diagnostics eliminate credential exfiltration via upstream strings.
- No shell, no elevation, no network policy change beyond timeout truthfulness.
- `#![forbid(unsafe_code)]` retained.

## Docs/operations evidence

- `crates/eggup-acquisition/README.md`: effective formula, staging, redaction.
- `crates/eggup-eggfetch/README.md`: effective derivation, staging, redaction audit, consumer effect.
- Seam rustdoc: `FetchLimits::effective`, `__acquire_exclusive_temp`, `__promote_no_clobber`, redaction guarantees.
- `CHANGELOG.md`: M003 entry + 0.1.1 version decision.
- `cargo doc` clean on stable and 1.89.

## Unresolved findings with severity

- Low / accepted (ordering): `cargo package -p eggup-eggfetch` in isolation uses crates.io seam without new APIs. Publish seam first when release is directed. No code action.
- Informational: Hosted CI not run locally.
- None: No medium-or-higher acquisition safety/contract issue remains.

## Disposition and roadmap transition

M003 is closed. Unblocks broader updater-bearing consumer migration planning
(eggsearch M003, Gregg M004) per registry gate. Service M002 and distribution
M001 remain independently executable. Optional M004 lightweight/curl adapter
stays deferred/evidence-driven pending remeasurement of the corrected Eggfetch
path.

## Registry updates

- Acquisition M003 → closed.
- Consumer adoption gate: additional updater-bearing migrations no longer blocked by acquisition corrective (service/distribution gates still apply per plan).
