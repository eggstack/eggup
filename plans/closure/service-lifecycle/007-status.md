# Service Lifecycle M007 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/service-lifecycle/007-utf8-safe-bounded-diagnostics-corrective.md`

Source roadmap: `plans/subsystems/service-lifecycle-roadmap.md#M007--utf-8-safe-bounded-diagnostics-corrective`

Reviewed source baseline: `ee1476ef2a9e0d5569d6dc2469e780a4435cc426` (corrective review baseline). Work began from the plan-registration head `b619fbd3821047e8e317343dbe49c3a41e6fbfc2` after Acquisition M007 and Archive M001 had been formally closed.

Implementation commits:

- `adb7c09626c3ba23499cc58e961995361be24534` — shared UTF-8-safe byte bounding, diagnostic regressions, docs/changelog, and hosted Windows service qualification lane.
- `f87b040646dc8671379de0d4b4bead29a99ffa9d` — restrict the Unix-adapter test module to Unix after Windows execution exposed its POSIX path assumptions.
- `5138f12` — run portable service diagnostic regressions and the Windows SCM suite on Windows; keep full service suites on Linux/macOS.

Hosted final qualification: CI run [36215858056](https://github.com/eggstack/eggup/actions/runs/36215858056) on `5138f12`; Stable Linux, Rust 1.89, macOS, Windows targeted runtime tests, and Windows workspace check passed after a full rerun.

## Executive finding

Production service diagnostic truncation now uses one byte-bounding helper that walks back to a valid UTF-8 boundary. The prior 512-byte service error/failure ceiling and 256-byte manager stderr excerpt ceiling remain enforced. Control-character filtering on lifecycle/disposition failures is unchanged. Long permission errors reserve space for the existing remediation hint so it remains visible inside the 512-byte bound. No lifecycle state machine, ownership, manager call, deadline, or transaction behavior changed.

M007 is closed. No high- or medium-severity related issue remains open.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| ASCII exact and above 256/512-byte ceilings | `diagnostic_byte_bounds_preserve_utf8_at_256_and_512_edges` | passed |
| 2-, 3-, and 4-byte code points crossing each ceiling | same boundary matrix asserts backing down to the last complete code point | passed on Linux, macOS, Windows |
| Service errors, manager output, and permission remediation remain bounded | `service_errors_output_and_permission_remediation_stay_utf8_bounded`; remediation suffix remains present | passed |
| Lifecycle failure details remain bounded and control-free | `lifecycle_failure_detail_is_utf8_safe_bounded_and_control_free` | passed |
| Existing M005/M006 lifecycle and disposition behavior | full workspace/service suites on Linux/macOS; existing Windows SCM model suite on Windows | passed |
| UTF-8 truncation audit | `ServiceError::invalid`, `ServiceError::bounded`, manager `truncate`, lifecycle `bounded_detail`, disposition `bound_detail`; no remaining panic-capable production fixed-byte `String::truncate` sites | passed |
| Hosted qualification | run 36215858056: full Linux/macOS suites; MSRV check; Windows diagnostics/SCM tests and workspace check | passed, platform scope below |

## Production implementation and audit evidence

- `crates/eggup-service/src/lib.rs`: `truncate_utf8_bytes(String, max_bytes)` preserves shorter strings and only truncates at a character boundary. `ServiceError::invalid`, `ServiceError::bounded`, and 256-byte manager stderr excerpts use it.
- `permission_hint` reserves the fixed remediation suffix within 512 bytes, then appends it to the safely bounded manager detail. This preserves the existing wording and prevents the hint from disappearing when the manager message already fills the limit.
- `crates/eggup-service/src/lifecycle_update.rs` and `src/disposition.rs`: existing control-character filtering is preserved; the resulting detail is passed through the same UTF-8-safe bound helper.
- Audited other production string slices in service code. Remaining slices are over ASCII-validated delimiters/prefixes or token boundaries; none truncate arbitrary manager-derived UTF-8.
- `unix_tests` is now gated to Unix because hosted Windows runs showed that its systemd/launchd/crontab fixtures require POSIX absolute paths. This does not change production code or service lifecycle behavior.
- The Windows CI lane runs the three portable diagnostic regressions plus all 15 `windows_scm::tests`; Linux and macOS run the complete service and workspace suites. Existing broader service model/orchestration tests use POSIX executable fixtures and are not claimed as Windows runtime coverage.

## Exact commands and results

Local Darwin arm64:

```text
cargo fmt --all -- --check                                      passed
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings  passed
cargo test -p eggup-service --all-targets --all-features --locked  passed (101)
cargo test --workspace --all-targets --all-features --locked    passed (273 workspace tests)
cargo doc --workspace --no-deps --locked                        passed
cargo +1.89.0 check --workspace --all-targets --locked           passed
./scripts/check-local.sh                                         passed
git diff --check                                                passed
```

Hosted final run 36215858056:

- Stable Linux: format, clippy, full workspace tests (service 101/101), and docs passed.
- MSRV Linux: Rust 1.89 workspace all-target check passed.
- macOS: full workspace tests passed (service 101/101).
- Windows: acquisition/archive/curl portable suites passed (34/17/5); the three new service diagnostic tests passed (1 each), all 15 Windows SCM model tests passed, and workspace all-target check passed.

The initial Windows attempt ran the Unix-specific adapter fixtures and then the shared service model suite; fixtures that construct `/opt/...` paths failed Windows `Path::is_absolute` checks. They are now platform-qualified or run only in the appropriate hosted platform lane. Two macOS attempts each hit a different existing timing assertion (`launchd_restart_does_not_start_after_incomplete_stop`, `<150 ms`; and `transition_deadline_rejects_zero_and_only_shrinks`, 1 ms deadline) while the other 100 service tests passed. A full workflow rerun passed all jobs and all 101 service tests; no timeout or lifecycle behavior was changed.

## Invariant review

- Service errors, manager stderr excerpts, lifecycle failures, and disposition failures remain finitely bounded by their existing byte limits.
- Truncation cannot panic on a valid Rust `String` and never emits invalid UTF-8.
- Lifecycle/disposition control-character filtering remains unchanged; permission remediation wording remains present within the 512-byte cap.
- Ownership remains `Absent | Owned | Foreign | Unknown`; no action is added for Foreign/Unknown.
- Manager executables, environments, privilege policy, transition deadlines, M005 rollback policy, M006 dispositions, and transaction ordering remain unchanged.

## Failure and recovery review

This corrective changes only diagnostic string construction. It introduces no manager calls, filesystem mutation, transaction changes, retries, timeouts, cancellation behavior, or recovery state. Oversized diagnostics are shortened at a character boundary; service lifecycle outcomes are unaffected.

## Compatibility and migration review

No public API signature or error category changed. Crate-produced diagnostics retain the existing 256-byte excerpt and 512-byte error/failure ceilings; only the final display cut may occur a few bytes earlier to avoid splitting a code point. Long permission errors now retain the existing remediation suffix. No consumer migration is required. The crate README and both crate/root `Unreleased` changelogs record the behavior; no publication occurred.

## Security review

The change prevents malformed UTF-8 diagnostics from becoming panics and retains the prior size bounds and permission remediation behavior. It introduces no raw-output expansion, new captured data, or new secret flow. No medium-or-higher finding remains open.

## Unresolved findings

- Informational test-scope note: Windows runs UTF-8 service diagnostics and all Windows SCM model tests, while the complete 101-test service suite runs on Linux/macOS. Existing broad service tests contain POSIX path fixtures and are not portable Windows tests. No M007 requirement depends on those fixtures.
- Informational hosted flakes: separate hosted attempts exposed two existing timing-sensitive tests noted above; each passed on the subsequent full workflow rerun. No repeated failure was observed in the final run, and no M007 code change is implicated.
- None: no high- or medium-severity implementation issue remains.

## Roadmap disposition and downstream unblock

Service Lifecycle M007 moves from ready to closed and completes the currently defined service-lifecycle roadmap through M007. A planning search found no other implementation plan blocked on M007; no additional plan status required an unblock. The Egress M006 and Eggpack interoperability M002 authoring statuses remain ready from Archive Extraction M001 closure. Those are separate plans and are not changed by this service corrective.

## Registry updates

- Service Lifecycle M007: ready → closed.
- Service lifecycle roadmap: M001-M007 closed; no next implementation milestone registered.
- Windows CI: portable service diagnostic boundary tests and Windows SCM model suite execute at runtime; Unix manager fixture suite remains on Unix hosts.
- No consumer migration, package publication, Gregg change, or additional downstream status transition.
