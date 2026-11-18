# Closure record — verified-update-core M021

## Status

**Closed; hosted-qualified.** M021 replaces the six Unix shell-backed Core bounded-candidate fixtures with one native Rust helper compiled once per test process. It closes the recurring interpreter-related ETXTBSY exposure in the Core tests. The broader host-level spawn/resource residual in registry finding 1 remains open.

## Source plan and roadmap

- Source plan: `plans/implementation/verified-update-core/021-native-bounded-candidate-test-fixture.md`
- Source roadmap: `plans/subsystems/verified-update-core-roadmap.md`
- Registry: `plans/registry.md`

## Reviewed repository baseline

| Item | Value |
|---|---|
| Branch | `release/0.2.1-preparation` |
| Plan baseline | `ecfe02a1c376db4bbaaabeb057407109ea3c46e1` |
| Implementation commit | `5f7f074baa491f62a5e372d0caacaacea5148716` (`test(core): replace shell candidate fixtures (M021)`) |
| Hosted push run | [44380404044](https://github.com/eggstack/eggup/actions/runs/44380404044), exact head SHA, all five jobs successful |
| Hosted PR run | [44380418766](https://github.com/eggstack/eggup/actions/runs/44380418766), exact same head SHA, all five jobs successful |

## Implementation commits

| Commit | Role |
|---|---|
| `5f7f074baa491f62a5e372d0caacaacea5148716` | Replaces the Core Unix shell fixtures with the native helper, registers M021, and updates the Core test-table anchors. |

No new PR was created. No merge, tag, GitHub Release, or crate publication occurred. Core M016 remains active and unchanged.

## Executive finding

Hosted PR run [44370818511](https://github.com/eggstack/eggup/actions/runs/44370818511) on baseline `ecfe02a1c376db4bbaaabeb057407109ea3c46e1` failed in `bounded_candidates_are_exact_and_environment_is_cleared` with `ExecutableFileBusy (os error 26)` while spawning a freshly written `#!/bin/sh` candidate. M021 replaces all six shell fixtures across the three affected Core Unix tests with private copies of a shared native Rust helper. The helper compiles once per Unix test process; its basename selects exact-identity, environment-probe, timeout, output-overflow, and matching cross-member modes. The tests continue to exercise the real validators and `run_bounded`, and the helper-format guard rejects a shebang binary.

## Requirement → evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| All six Core shell-backed bounded-runner fixtures are replaced | `crates/eggup-core/src/lib.rs`; `bounded_test_helper_copy` and `BOUNDED_TEST_HELPER_SOURCE`; no `#!/bin/sh` remains in the Core source | Passed |
| Helper is native and compiled once per Unix unit-test process | `native_bounded_test_helper` uses `OnceLock` and `RUSTC` (fallback `rustc`); `bounded_runner_test_helper_is_native_executable` passed | Passed |
| Existing bounded-candidate semantics remain asserted | Exact staged identity output and cleared environment; timeout/kill and output-limit cases; cross-member agreement. The four focused tests and all 60 Core library tests passed | Passed |
| No production behavior, dependency, API, or version changed | Diff is confined to the Core `#[cfg(test)]` module and planning/architecture records; no Cargo manifest, lockfile, production module, or release file changed | Passed |
| Required local qualification passes | Exact commands and results below; full local gate exited 0 | Passed |
| Hosted qualification passes on exact implementation SHA | Runs [44380404044](https://github.com/eggstack/eggup/actions/runs/44380404044) and [44380418766](https://github.com/eggstack/eggup/actions/runs/44380418766), both on `5f7f074baa491f62a5e372d0caacaacea5148716`; all five lanes successful in each | Passed |
| Broader finding and release boundary are not overstated | Registry finding 1 remains open for unrelated host-level spawn/resource failures; M016 remains active; no merge or publication | Passed |

## Production implementation evidence

There is no production implementation change. The helper type, compiler invocation, per-fixture copies, native-format check, and migrated test call sites are all inside the Core test module. `run_bounded`, spawn classification, timeout semantics, environment clearing, output bounds, public API, and transaction behavior are unchanged.

## Verification commands and results

| Command | Result |
|---|---|
| `cargo test -p eggup-core --locked --lib tests::bounded_runner_test_helper_is_native_executable -- --exact` | Passed: 1 test |
| `cargo test -p eggup-core --locked --lib tests::bounded_candidates_are_exact_and_environment_is_cleared -- --exact` | Passed five consecutive runs; 1 test per run |
| `cargo test -p eggup-core --locked --lib tests::bounded_runner_kills_timeouts_and_limits_output -- --exact` | Passed: 1 test |
| `cargo test -p eggup-core --locked --lib tests::cross_member_identity_requires_bundle_agreement -- --exact` | Passed: 1 test |
| `cargo test -p eggup-core --locked --lib` | Passed: 60 tests, 0 failed |
| `./scripts/check-local.sh` | Passed, exit 0: formatting, Clippy, workspace tests, docs, dependency-tree resolution/review, and documentation-truth checks |
| `git diff --check` | Passed |
| `cargo tree --workspace --locked` (run by the local gate) | Reviewed; no dependency or boundary change |

The local run was on macOS. It is not used as evidence for Linux or Windows; those platforms are reported separately from hosted CI.

## Hosted platform evidence

Both hosted runs used exact head SHA `5f7f074baa491f62a5e372d0caacaacea5148716` and completed successfully.

| Platform/lane | Push run 44380404044 | PR run 44380418766 |
|---|---|---|
| Linux / Stable checks | Passed | Passed |
| Rust 1.89 / MSRV check | Passed | Passed |
| macOS tests | Passed | Passed |
| Windows archive, acquisition, and service tests | Passed | Passed |
| Linux systemd service qualification | Passed | Passed |

Linux Stable and macOS ran the Unix-only helper tests. Windows passed its curated tests; the Unix helper remains cfg-excluded there.

## Invariant review

- The fix is test-only; production spawn behavior and error classification are unchanged.
- No spawn retry, fallback, weakened assertion, or test skip was added.
- The helper compiles once, and fixture copies are completed before execution and are not rewritten afterward.
- No dependency, public API, package version, publication, tag, or release file changed.
- The test-only destination basenames select helper modes; they do not change consumer behavior.

## Failure, cancellation, rollback, and recovery review

Failure to create the helper root, write or compile the source, copy the binary, or spawn a fixture fails the test with the observed error. The test does not retry a spawn failure. Each test uses a private fixture root, preventing candidate copies from being shared across parallel tests. Timeout and output overflow continue to be handled and asserted by the real bounded runner. No live installation, service state, or production recovery path is touched.

## Compatibility and migration review

No consumer-visible behavior, public type, or migration changes. Tests remain `#[cfg(unix)]`-gated; existing Windows behavior and coverage remain unchanged. The compatibility note in the plan records that distinct destination basenames are used solely for test-helper mode selection.

## Security review

Native binaries eliminate the shebang interpreter race source in these fixtures. The environment helper reports only whether `PATH` or `EGGUP_SECRET` is present; it never prints secret values. No network access or production logging path was added.

## Unresolved findings and severity

Registry finding 1 remains **open, low severity** for broader host-level spawn/resource-pressure modes and the already narrowed timeout/spawn observations. M021 closes only the Core shell-backed fixture exposure; two green runs on this fix do not prove that every transient host-level spawn failure is impossible.

## Documentation, operations, and registry disposition

- The Core architecture test-table anchors now match the test definitions at `lib.rs:1399`, `lib.rs:1440`, and `lib.rs:1458`.
- The M021 plan is marked implemented; the Core roadmap and registry mark M021 closed and link this closure record.
- Registry finding 1 records the fixed Core fixture while retaining the broader host-level residual.
- Core M016 remains active and its publication gates are unchanged.

**Roadmap disposition:** M021 closed and hosted-qualified; M016 remains active.

**Registry disposition:** M021 closed on implementation commit `5f7f074baa491f62a5e372d0caacaacea5148716`; registry finding 1 remains open for broader host-level spawn/resource failures.
