# Acquisition Transport Milestone 018 — native fake-curl test helper

Status: active

Repository baseline: `63e06b0d0f90cef4477b319dc769c22578e7f8ad`

Source roadmap:
- `plans/subsystems/acquisition-transport-roadmap.md`

Long-term requirements:
- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`

Applicable ADRs:
- `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md`

Primary class: polish (test/qualification corrective)

## 1. Objective

Remove the intermittent Linux `ETXTBSY` hosted-CI failures from `eggup-curl`'s fake-curl tests by replacing their temporary `#!/bin/sh` executable with a native test helper. Preserve each test's observable contract: the helper records argv and environment, emits the configured body and HTTP code, and exits with the configured status.

## 2. Why this milestone is ready

Hosted run [44024401775](https://github.com/eggstack/eggup/actions/runs/44024401775), on `main` at baseline `63e06b0d0f90cef4477b319dc769c22578e7f8ad`, failed Stable twice. Attempt 1 reported `ExecutableFileBusy (os error 26)` in `build_curl_args_passes_sub_second_deadlines_to_fake_curl`; attempt 2 reported the same error in `artifact_temp_is_owner_private_and_no_clobber`. The other four lanes passed on both attempts. The failures occur when spawning the temporary shell-backed fake executable; the transport behavior itself is not under test in these failure frames.

## 3. Current implementation evidence

`crates/eggup-curl/src/lib.rs` defines `fake_curl_script`, which writes a shell script with `#!/bin/sh`, marks it executable, and calls `xxd` to decode the test body. Seven Unix-only tests use it. `Command::spawn` reports the OS cause as `ExecutableFileBusy`, but retrying production spawns or masking the failure in the test would weaken the evidence. The intended assertions are independent of shell behavior: they need a deterministic child that records its received argv/environment and emits fixed outputs.

## 4. Invariants that must not regress

- No production code, transport classification, timeout, retry, or fallback behavior changes.
- Fake-child tests continue to prove actual adapter-spawned argv and environment, with no public network access.
- Body bytes, response-code bytes, and configured exit status remain exact.
- No dependency, public API, package version, publication, tag, or release changes.
- Preserve Rust 2021 and Rust 1.89 MSRV.

## 5. Scope

### In scope
- Replace the shell script test double with a native Rust helper compiled once per Unix test process and addressed through per-test links/config files.
- Keep the existing fake-curl assertions and add a guard that the helper is not a shebang script.
- Update the curl deep dive, this plan/closure, roadmap, and registry evidence.

### Explicitly out of scope
- Production retry or re-selection for `ETXTBSY` or any other spawn error.
- Changes to unrelated shell-backed fixtures, including the separate Core test.
- CI workflow retry policy, dependency changes, publication, or release history.

## 6. Required production changes

None. All code changes must remain inside `#[cfg(test)]` support in `crates/eggup-curl/src/lib.rs`.

## 7. Ordered work packages

1. Implement a small native fake-curl helper that reads per-fixture config, writes argv/environment records, emits configured stdout/stderr, and returns the configured exit status.
2. Compile it once per test process; create unique per-test executable names and config files without rewriting the executable after compilation.
3. Migrate all existing `fake_curl_script` call sites and retain each assertion unchanged in meaning.
4. Run focused tests, the local workspace gate, and hosted CI; reconcile the registry and closure only from observed results.

## 8. Failure, cancellation, restart, and contention semantics

The helper is test-only and runs once per fetch, exactly as the previous fake child did. No retry is added. If the helper cannot be compiled or spawned, the test fails with the compiler or spawn cause rather than silently skipping assertions. Each fixture gets a unique executable link and config file; no shared mutable fixture state is introduced.

## 9. Compatibility and migration

No production or consumer-visible behavior changes. The native helper is compiled only by Unix unit tests; Windows continues to exclude the shell-dependent fake-child tests as before and retains its existing portable and real-`curl.exe` coverage. No manifest or published artifact changes.

## 10. Required tests

- The existing fake-child tests: `tls_like_process_failure_is_transport_not_unavailable`, `build_curl_args_passes_sub_second_deadlines_to_fake_curl`, `proxy_disabled_uses_noproxy_and_cleared_env`, `proxy_custom_env_is_explicit`, `artifact_temp_is_owner_private_and_no_clobber`, `credential_material_is_redacted`, and `invalid_limits_fail_before_spawn`.
- A helper-format assertion proving the generated executable is a native binary, not a `#!` script.
- Full workspace tests to retain all platform-neutral and platform-gated coverage.

## 11. Required verification commands

```sh
cargo test -p eggup-curl --all-targets --locked
./scripts/check-local.sh
cargo +1.89.0 check --workspace --all-targets --locked
```

Then push the implementation commit to `main`, wait for the hosted workflow, and read back all five lane conclusions. `cargo tree --workspace --locked` must show no dependency-boundary change. Record any failed attempt separately; a rerun is not a pass until its result is read.

## 12. Documentation updates

Update `architecture/curl-adapter.md` to describe the native helper and remove claims that the fake child relies on `sh`/`xxd`. Record the two hosted `ETXTBSY` observations and the disposition of registry finding 1 in the closure and `plans/registry.md`. Reconcile the acquisition roadmap milestone status after verification. No changelog or consumer documentation change is required because no packaged behavior changes.

## 13. Acceptance criteria

- All seven fake-child tests preserve their assertions and pass locally.
- The Unix fixture is a native executable and no longer invokes a shebang interpreter or `xxd`.
- Focused tests, `./scripts/check-local.sh`, and the Rust 1.89 workspace check pass.
- Hosted Stable, MSRV, macOS, Windows, and Linux systemd jobs pass on the implementation commit.
- No production code, dependency, public API, version, or release file changes.

## 14. Stop conditions

Stop if the change requires production retries, a public API, a new crate/dependency, or weakening an existing assertion. If Rust compiler discovery or helper execution cannot be made reliable on Unix CI, do not replace the shell helper with another interpreter-backed fixture; report the blocker and preserve the current tests.

## 15. Closure evidence required

Record exact implementation/closure commit SHAs; the two original failed attempts and the successful hosted run; focused and full local command results; Rust 1.89 result; all five hosted lane results; invariant review; confirmation that no production source or manifest changed; and the remaining status of registry finding 1.

## 16. Handoff notes

This corrects only the curl fake-child interpreter exposure. It does not close registry finding 1 wholesale: the separate Core fixture and other host-level spawn/resource failures remain outside this milestone. No crate publication, tag, or release is authorized or required.
