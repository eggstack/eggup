# Closure record — acquisition-transport M018

## Status

**Closed; hosted-qualified.** M018 removes the shell-interpreter exposure from
`eggup-curl`'s fake-child tests. The focused suite, local workspace gate, Rust
1.89 check, and hosted CI matrix all passed. This milestone does **not** close
registry finding 1: the separate Core shell fixture and general host-level
spawn/resource failures remain outside scope.

## Source plan and roadmap

- Source plan: `plans/implementation/acquisition-transport/018-native-fake-curl-test-helper.md`
- Source roadmap: `plans/subsystems/acquisition-transport-roadmap.md`
- Registry: `plans/registry.md`

## Reviewed repository baseline

| Item | Value |
|---|---|
| Branch | `main` |
| Plan baseline | `63e06b0d0f90cef4477b319dc769c22578e7f8ad` |
| Implementation commit | `10e0bd719de95d52e297eb768a468adcbda0a6aa` (`test(curl): use native fake-curl helper`) |
| Hosted run head SHA | `10e0bd719de95d52e297eb768a468adcbda0a6aa` |
| Hosted run | [44155438351](https://github.com/eggstack/eggup/actions/runs/44155438351), completed 2026-11-17; overall conclusion `success` |
| Closure record introduced by | `84d22d4484b04bd76bdc48cf337c2b2dbbea3446` (`plans(acquisition): close M018 native fake-curl fix`) |

## Implementation commits

| Commit | Role |
|---|---|
| `10e0bd719de95d52e297eb768a468adcbda0a6aa` | Replaces the Unix shell-backed fake-curl script with a native Rust test helper and adds this milestone's plan/architecture/roadmap/registry tracking. |

No pull request, publication, tag, or release-history change was made.

## Executive finding

Hosted run `44024401775` failed twice in Stable on the same source SHA, each time
with `ExecutableFileBusy (os error 26)` while spawning a shell-backed fake-curl
fixture, but in different tests. M018 replaced the `#!/bin/sh` script and its
`xxd` dependency with a native Rust helper, compiled once per Unix test process
and hard-linked into per-fixture directories with separate configuration files.
The helper still records argv and environment, emits the configured body and
HTTP code, and exits with the configured status. A regression test rejects a
shebang helper.

The implementation commit's hosted run `44155438351` passed all five lanes.
This supports closing M018's bounded fixture correction; it does not establish
that every host-level spawn/resource failure is impossible, so registry finding
1 remains open.

## Requirement → evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Native helper replaces the Unix shell fixture without changing its observable test contract | `crates/eggup-curl/src/lib.rs`, `native_fake_curl_helper`, `fake_curl_executable`, and `FAKE_CURL_HELPER_SOURCE`; all seven prior fake-curl call sites use the native helper | Passed |
| The helper is not a shebang script | `fake_curl_test_helper_is_native_executable`; focused test output | Passed |
| Existing fake-child assertions remain active | `tls_like_process_failure_is_transport_not_unavailable`, `build_curl_args_passes_sub_second_deadlines_to_fake_curl`, `proxy_disabled_uses_noproxy_and_cleared_env`, `proxy_custom_env_is_explicit`, `artifact_temp_is_owner_private_and_no_clobber`, `credential_material_is_redacted`, and `invalid_limits_fail_before_spawn` | Passed |
| No production, dependency, API, version, or release change | Implementation diff is confined to `#[cfg(test)]` support and test call sites in `eggup-curl`; no manifest, lockfile, public API, version, tag, or release file changed. `cargo tree --workspace --locked` showed no dependency-boundary change | Passed |
| Required local and MSRV checks pass | Exact commands and results below | Passed |
| Hosted qualification passes on the implementation commit | Run `44155438351`, all five jobs successful; per-platform detail below | Passed |
| Broader registry finding 1 is not overstated | Its remaining Core fixture and host-resource modes are explicitly retained as open | Passed |

## Production implementation evidence

There is no production implementation change. The native helper, its compiler
invocation, per-fixture hard links/configuration, shebang assertion, and migrated
call sites are all inside the `eggup-curl` test module. Production `Command::spawn`
behavior, timeout/cancellation handling, error classification, transport
composition, and fallback behavior are unchanged.

## Verification commands and results

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | Passed |
| `git diff --check` | Passed |
| `cargo test -p eggup-curl --all-targets --locked` | Passed: 34 passed, 0 failed, 1 pre-existing ignored; assertion guard 3 passed, 0 failed |
| `./scripts/check-local.sh` | Passed (exit 0): workspace formatting, clippy, tests, docs, dependency-tree review, and documentation-truth checks |
| `cargo +1.89.0 check --workspace --all-targets --locked` | Passed |
| `cargo tree --workspace --locked` (run by the local gate) | Reviewed; no dependency-boundary or dependency change |

## Hosted platform evidence

All five jobs in [run 44155438351](https://github.com/eggstack/eggup/actions/runs/44155438351)
completed successfully on implementation commit
`10e0bd719de95d52e297eb768a468adcbda0a6aa`.

| Platform/lane | Evidence | Result |
|---|---|---|
| Linux / Stable checks | `cargo fmt`, clippy, full all-features workspace tests, footprint builds, docs, and packaged-service verification | Passed; includes Unix fake-curl tests |
| Rust 1.89 / MSRV check | Workspace all-target check, three footprint configurations, and packaged-service verification | Passed; this lane checks compilation and does not claim to execute tests |
| macOS tests | Full all-features workspace tests, footprint builds, and packaged-service verification | Passed; Unix fake-curl tests execute here |
| Windows archive, acquisition, and service tests | Targeted and full workspace tests plus packaged-service verification | Passed; Unix-only fake-curl helper is cfg-excluded, so no Windows helper execution is claimed |
| Linux systemd service qualification | Published-version fixtures and real-systemd service fixture | Passed; independent of the fake-curl helper |

## Invariant review

- The change is test-only; production spawn and transport behavior are unchanged.
- No production retry, test retry, fallback change, assertion weakening, or failure masking was added.
- No dependency, public API, package version, publication, tag, or release-history change was made.
- Rust 2021 and the Rust 1.89 floor are preserved; the local MSRV check and hosted MSRV lane passed.
- No public network is used by the fake-curl tests.

## Failure, cancellation, rollback, and recovery review

The helper compiles once per Unix test process and each fixture receives a hard
link plus a separate config file; the executable itself is not rewritten after
compilation. If helper compilation or spawning fails, the test fails with the
observed cause rather than skipping or retrying. Fixture files remain test-local.
No install destination, service state, transaction, or production recovery path
is touched. There is no production rollback or recovery behavior to change.

## Compatibility and migration review

No consumer-visible behavior or public type changes. No migration is required.
The helper uses the repository's existing Rust toolchain and standard library;
no third-party dependency was added. Windows continues to use its existing
portable and real-`curl.exe` test coverage, with the Unix fake-child tests
excluded as before.

## Security review

The native helper removes the shell interpreter and `xxd` from these test
fixtures. It operates on test configuration and records argv/environment in the
fixture's temporary directory for assertions, consistent with the former fake
script's test contract. It does not send data over a network or add a logging
path to production. No credential values were added to source or diagnostics.

## Unresolved findings and severity

Registry finding 1 remains **open, low severity**. M018 removes the shebang
interpreter exposure from `eggup-curl`'s fake-child tests but does not address
the separate Core shell fixture or every possible host-level spawn/resource
failure. The passing implementation run narrows the evidence for this bounded
fixture issue; it does not close the broader finding.

## Documentation, operations, and registry disposition

- The curl adapter deep dive now describes the native helper and updated test count/references.
- The M018 plan, acquisition roadmap header/body/status table, and registry status surfaces now agree that M018 is closed and hosted-qualified.
- The registry's finding 1 remains open, with its out-of-scope causes stated explicitly.
- No consumer-facing docs or changelog update is required for this test-only change.

**Roadmap disposition:** M018 closed and hosted-qualified; M012 retains its
conditional status; registry finding 1 remains open.

**Registry update:** M018 is marked closed in the acquisition-transport summary
and milestone row. Finding 1 remains open pending evidence beyond this bounded
curl test-fixture correction.
