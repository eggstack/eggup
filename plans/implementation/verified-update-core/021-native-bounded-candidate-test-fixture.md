# Verified Update Core Milestone 021 — native bounded-candidate test fixture

Status: active

Repository baseline: `ecfe02a1c376db4bbaaabeb057407109ea3c46e1`

Source roadmap:
- `plans/subsystems/verified-update-core-roadmap.md`

Long-term requirements:
- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`

Applicable ADRs:
- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`
- `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md`
- `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md`

Primary class: polish (test and qualification corrective)

## 1. Objective

Remove the intermittent `ETXTBSY` exposure from eggup-core Unix unit tests by replacing their temporary `#!/bin/sh` fixtures with one native Rust helper compiled once per test process. Preserve the existing exact-output, cleared-environment, timeout, output-limit, and cross-member-agreement assertions. No production behavior or dependency changes are in scope.

## 2. Why this milestone is ready

Hosted run [44370818511](https://github.com/eggstack/eggup/actions/runs/44370818511) on the release-preparation merge commit reported `ExecutableFileBusy` (OS error 26) while `bounded_candidates_are_exact_and_environment_is_cleared` spawned its freshly written shell candidate. The same commit passed all five lanes in push run [44370802274](https://github.com/eggstack/eggup/actions/runs/44370802274), demonstrating that this is intermittent rather than a deterministic production failure.

The repository already records earlier occurrences of this fixture failure. Acquisition M018 replaced its shell-backed fake with a native test helper but explicitly left the Core fixture out of scope. Core M016 also excludes test-source changes and requires a separate plan if qualification needs one. This milestone supplies that separate bounded plan.

## 3. Current implementation evidence

- At baseline `ecfe02a1c376db4bbaaabeb057407109ea3c46e1`, `crates/eggup-core/src/lib.rs` contains six `#!/bin/sh` fixtures across three Unix tests: exact identity and environment clearing, bounded timeout and output, and cross-member identity.
- The failing test writes the candidate with `InstallationRoot::write_file`, sets executable permissions, then `ExactIdentityValidator` invokes it through `run_bounded`. The surfaced `Io` error is the correct behavior for a spawn failure; retrying it in production would change semantics and is not this fix.
- `run_bounded` already clears inherited environment and applies bounded process behavior. The test fixture—not the runner—is the source of the interpreter exposure.
- Acquisition M018 provides a repository precedent for compiling one minimal native Rust helper with `RUSTC` once per test process and sharing it across unique fixtures.

## 4. Invariants that must not regress

- Do not change production code in `candidate.rs`, spawn classification, timeout semantics, output bounds, or environment policy.
- Continue to execute staged candidate bytes through `ExactIdentityValidator` and `CrossMemberAgreementValidator`; keep their expected output byte-exact.
- Continue to prove that inherited environment is cleared, the timeout kills/reaps a long-running child, and excess output is reported as limited.
- Keep fixture executables unique per test root and do not rewrite a compiled helper while it can be executed.
- Add no dependency, public API, feature, version, publication, tag, or release change.

## 5. Scope

### In scope

- Replace all six shell-script fixtures in the three Core bounded-candidate tests with private copies of one native Rust helper compiled once per Unix unit-test process.
- Give the helper deterministic behavior selected by its executable basename: exact identity output, environment inspection, a bounded timeout case, an output-overflow case, and matching cross-member output.
- Add a regression assertion that the compiled helper does not begin with the `#!` script marker.
- Update Core test line anchors, the Core roadmap, and the registry status/finding records from observed evidence.

### Explicitly out of scope

- Any production retry for `ETXTBSY`, spawn re-selection, changed error mapping, or `run_bounded` semantics.
- Changes to Acquisition M018 or other crates, workflow retry policy, dependencies, public API, package versions, or release files.
- Publishing any crate, creating/moving tags, or creating a GitHub Release. Core M016 remains a separate active milestone.

## 6. Required production changes

None. All implementation changes remain within `#[cfg(test)]` support in `crates/eggup-core/src/lib.rs`.

## 7. Ordered work packages

1. Implement a Rust test-helper source and a `OnceLock` initializer that compiles it once with the active `RUSTC` (falling back to `rustc`) into a unique temporary directory.
2. Copy the native executable into each private fixture root under a name that selects one deterministic helper mode; set executable permissions and do not modify the helper after compilation.
3. Migrate all six Core shell-script fixtures and retain the existing semantic assertions. Add the native-format guard.
4. Run the focused Core tests, full local gate, and exact-SHA hosted CI. Reconcile the plan, closure, roadmap, registry, and test-table line anchors only from observed results.

## 8. Failure, cancellation, restart, and contention semantics

The helper is test-only. Failure to create its directory, write source, invoke `rustc`, compile, copy, or spawn must fail the test with the actual cause; tests must not skip or retry a spawn error. `OnceLock` ensures one compile per test process. Each test uses its existing private fixture root, so candidate copies and outputs are not shared across parallel tests. A failed local or hosted gate stops qualification; no publication or tag action is part of this milestone.

## 9. Compatibility and migration

No production or consumer-visible behavior changes. Unix bounded-runner tests use the same executable paths and still invoke the real `run_bounded` implementation. The tests remain Unix-gated; Windows behavior and coverage are unchanged. No migration is required.

## 10. Required tests

- `bounded_runner_test_helper_is_native_executable` proves the compiled helper is not a shebang script.
- `bounded_candidates_are_exact_and_environment_is_cleared` retains exact staged identity validation and asserts the helper sees neither the parent `PATH` nor `EGGUP_SECRET`.
- `bounded_runner_kills_timeouts_and_limits_output` retains timeout/kill and output-limit assertions using native helper modes.
- `cross_member_identity_requires_bundle_agreement` retains exact common output across both staged members.
- Run the complete Core library tests and full workspace gate; hosted CI must qualify all five current lanes on the fix commit.

## 11. Required verification commands

```sh
cargo test -p eggup-core --locked --lib tests::bounded_runner_test_helper_is_native_executable -- --exact
cargo test -p eggup-core --locked --lib tests::bounded_candidates_are_exact_and_environment_is_cleared -- --exact
cargo test -p eggup-core --locked --lib tests::bounded_runner_kills_timeouts_and_limits_output -- --exact
cargo test -p eggup-core --locked --lib tests::cross_member_identity_requires_bundle_agreement -- --exact
cargo test -p eggup-core --locked --lib
./scripts/check-local.sh
git diff --check
```

Push the exact implementation commit to `release/0.2.1-preparation`; record the workflow run SHA and all five job conclusions separately. A local macOS pass is not evidence for Linux or Windows.

## 12. Documentation updates

- Add this plan and a paired closure record; update `plans/subsystems/verified-update-core-roadmap.md` and `plans/registry.md`.
- Update the `architecture/core-transaction.md` test-table line anchors if source-line movement changes them. Its behavioral descriptions remain valid because the assertions are preserved.
- Record the Core fixture disposition in the existing registry finding 1. Keep the general host-level spawn/resource residual open unless independent evidence closes it.
- No crate changelog, consumer guide, root README, AGENTS index, dependency, or release-history update is warranted for a test-only change.
- Keep Core M016 active and unchanged; this work does not authorize or perform publication.

## 13. Acceptance criteria

1. No `#!/bin/sh` fake candidate remains in the Core bounded-runner tests.
2. The helper compiles once per Unix test process, is a native executable, and is copied to unique test-root names without post-build writes.
3. The four focused tests and all Core library tests pass with their existing semantics intact.
4. `./scripts/check-local.sh` and `git diff --check` pass on the fix source commit.
5. All five hosted CI lanes pass on that exact commit; each lane result is read back from GitHub.
6. No production source, dependency, public API, package version, tag, release, or publication changes.
7. The registry and Core roadmap accurately record M021 and the remaining status of registry finding 1.

## 14. Stop conditions

- Stop if the helper requires a new crate/dependency, production retry, weakened assertion, or non-Unix behavior change.
- Stop if the helper cannot compile or run reliably on either Linux or macOS, or if any hosted lane fails.
- Stop before claiming closure if the exact hosted commit or any of the five lane conclusions cannot be confirmed.
- Do not use a shell-script or other interpreter-backed replacement as a fallback.

## 15. Closure evidence required

Record the implementation commit and branch, exact focused/full local commands and results, the helper-format proof, source diff showing test-only scope, all five hosted lane conclusions on the exact SHA, invariant/failure/security/compatibility review, updated test-table anchors, the registry finding disposition, and confirmation that M016 publication gates and registry state were not changed.

## 16. Handoff notes

This Core fixture was explicitly excluded from Acquisition M018. M021 addresses only the same interpreter-related `ETXTBSY` exposure in Core tests; it does not claim to eliminate unrelated host-level spawn/resource failures. M016 remains a distinct publication milestone and is not authorized by this test fix.