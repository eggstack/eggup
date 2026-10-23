# Verified Update Core — Milestone 014 closure record

## Status

**Closed with unresolved findings recorded.**

The defect is corrected in the working tree, the discriminating fixture is proven to
fail against the pre-M014 baseline and to pass against the corrected tree, and every
invariant of the source plan's §4 holds. Four findings remain open; none is
medium-or-higher, and each is recorded below with its severity and the reason for
that severity. Hosted CI lanes and the Linux and Windows platforms were **not run**
from this machine, so the milestone's hosted qualification is still outstanding and
is recorded as such rather than inferred.

## Source plan and roadmap

- Source plan: `plans/implementation/verified-update-core/014-bounded-execution-deadline-corrective.md`
- Roadmap: `plans/subsystems/verified-update-core-roadmap.md`
- Finding of record: `plans/registry.md` § "Post-0.1.3 open findings" — entry **3**
  ("`eggup-core` bounded candidate execution can exceed its documented wall-clock
  deadline"), registered at baseline `c480541` and assessed **high**.
- Corrects the implementation of `plans/implementation/verified-update-core/004-integrity-and-candidate-validation.md`
  against its own published contract.

## Reviewed repository baseline

`c4805411f74b85158e0ae5d0a219a32c4209d98e`, branch `plans/service-m010-failed-unit-recovery`.

The baseline was read directly and the defect was confirmed by a control-flow reading
of `crates/eggup-core/src/candidate.rs:160-202`, then proven at runtime by the
baseline-failure proof below. No production incident is known and none is claimed.

## Implementation commits / PRs

Working tree, uncommitted — no commit was made. The parent session owns commit and
PR creation.

Changed files (this milestone's exclusive ownership list):

- `crates/eggup-core/src/candidate.rs` — production change and rustdoc.
- `crates/eggup-core/tests/candidate_deadline.rs` — new integration test file.
- `crates/eggup-core/CHANGELOG.md` — `Unreleased` entry.
- `crates/eggup-core/docs/verification.md` — candidate-command paragraph.
- `architecture/core-transaction.md` — `run_bounded` paragraph and export table row.

The root `CHANGELOG.md` was **not** edited: the parent session aggregates it. The
required root `Unreleased` entry is still owed and is called out in § "Registry
updates" below.

## Executive finding

`run_bounded` evaluated its wall-clock deadline only while the direct child was still
alive, and joined its reader threads only on the success path. The baseline's deadline
arm was guarded by `status.is_none()`, so once `try_wait()` yielded `Some`, the only
remaining exit from the polling loop required **both** reader threads to have
delivered, which required pipe EOF. A descendant that inherited the stdout or stderr
pipe and held it open after the direct child exited therefore pinned the call
indefinitely, and the timeout path additionally dropped both reader threads without
joining them. `CommandSpec::timeout` is documented as the maximum wall-clock execution
time, so this was a direct contract violation in a published library.

The correction makes the deadline an **unconditional maximum bound on the whole
bounded execution**: it is evaluated on every poll whether or not the direct child has
already exited, the loop leaves without waiting for pipe EOF, and a still-running
direct child is killed and reaped exactly as before while an already-exited child is
neither killed nor reaped again. The four failure classes — deadline expiry, output
overflow, spawn failure, non-zero exit — remain distinguishable through the
published observables, with no new public type and no change to the `Error` variant
set.

## Requirement → evidence matrix

| # | Requirement (source plan §) | Evidence | Where |
|---|---|---|---|
| 6.1 | The deadline is unconditional; the call returns within the deadline plus a bounded teardown allowance | `candidate_deadline_descendant_holding_the_pipe_returns_at_the_deadline`; `candidate_deadline_timeout_kills_and_reaps_the_direct_child` | `crates/eggup-core/tests/candidate_deadline.rs`; `crates/eggup-core/src/candidate.rs:249-258` |
| 6.2 | Results delivered at the deadline are still reported; captured output never exceeds the bound | `candidate_deadline_output_overflow_is_reported_and_discards_bytes`; final drain in the runner | `crates/eggup-core/src/candidate.rs:262-265` |
| 6.3 | Deadline expiry, output overflow, spawn failure and non-zero exit stay distinguishable | `candidate_deadline_timeout_kills_and_reaps_the_direct_child`, `candidate_deadline_output_overflow_is_reported_and_discards_bytes`, `candidate_deadline_spawn_failure_reports_the_operation`, `candidate_deadline_non_zero_exit_is_distinguishable` | `crates/eggup-core/tests/candidate_deadline.rs` |
| 6.4 | The descendant case is bounded and fail-closed: `timed_out == true`, `exit_code == Some(code)`, `output_limited == false`, `success() == false` | `candidate_deadline_descendant_holding_the_pipe_returns_at_the_deadline` | `crates/eggup-core/tests/candidate_deadline.rs` |
| 6.5 | A delivered reader is joined; a reader still blocked at the deadline is abandoned explicitly and truthfully | Implementation (`poll_reader` + join rule) and rustdoc; `candidate_deadline_timeout_returns_and_leaves_the_runner_usable` | `crates/eggup-core/src/candidate.rs:262-273`, `:292-309`; rustdoc `:163-169` |
| 6.6 | The validator path still fails closed | `candidate_deadline_validator_fails_closed_on_a_candidate_that_overruns` | `crates/eggup-core/tests/candidate_deadline.rs` |
| 10.1 | Descendant-holds-pipe fixture, discriminating on "the call returns", fails against the baseline | Baseline-failure proof below: exit status 101, 10/10 reproducible | § "Baseline-failure proof" |
| 10.2 | Timeout kills and reaps the direct child, bounded elapsed time | `candidate_deadline_timeout_kills_and_reaps_the_direct_child` | `crates/eggup-core/tests/candidate_deadline.rs` |
| 10.3 | Output overflow: `output_limited`, empty captured bytes for the overflowing stream | `candidate_deadline_output_overflow_is_reported_and_discards_bytes` | `crates/eggup-core/tests/candidate_deadline.rs` |
| 10.4 | Spawn failure: `Err(Error::Io { operation: "spawning bounded candidate", .. })`, operation asserted | `candidate_deadline_spawn_failure_reports_the_operation` | `crates/eggup-core/tests/candidate_deadline.rs` |
| 10.5 | Non-zero exit is distinguishable | `candidate_deadline_non_zero_exit_is_distinguishable` | `crates/eggup-core/tests/candidate_deadline.rs` |
| 10.6 | Normal exit is unchanged | `candidate_deadline_normal_exit_is_unchanged` | `crates/eggup-core/tests/candidate_deadline.rs` |
| 10.7 | The validator path still fails closed | `candidate_deadline_validator_fails_closed_on_a_candidate_that_overruns` | `crates/eggup-core/tests/candidate_deadline.rs` |
| 10.8 | Reader-thread lifecycle is not observably leaking a runner | `candidate_deadline_timeout_returns_and_leaves_the_runner_usable` plus the reasoning recorded under "Failure / rollback / recovery review" | `crates/eggup-core/tests/candidate_deadline.rs` |
| 10.9 | Pre-M014 failure proof, fixtures 2–6 green on both trees | Baseline-failure proof below: exactly the descendant fixture fails, the other 11 pass | § "Baseline-failure proof" |
| 12 | Rustdoc, crate changelog, architecture paragraph, verification contract | § "Documentation / operations evidence" | see below |
| 13 | `Error` still has exactly the seven published variants; an exhaustive downstream `match` still compiles | `Error` enum untouched (`crates/eggup-core/src/error.rs:9-44`); `cargo test -p eggup-core --locked` compiles the crate's own exhaustive matches | § "Compatibility and migration review" |

## Production implementation evidence

All references are `crates/eggup-core/src/candidate.rs` unless stated.

- **The deadline is unconditional.** The baseline's deadline arm was
  `else if status.is_none() && Instant::now() >= deadline`; the corrected arm is
  `if Instant::now() >= deadline` at `candidate.rs:252`, evaluated on every poll
  whether or not the direct child has already exited, and it is the last exit from
  the loop before the poll sleep at `candidate.rs:260`.
- **The loop leaves without waiting for pipe EOF.** The corrected loop breaks on the
  deadline at `candidate.rs:258`, without waiting for `stdout_result` /
  `stderr_result`.
- **Kill and reap only a still-running child.** Both the overflow arm
  (`candidate.rs:235-243`) and the deadline arm (`candidate.rs:252-259`) guard the
  kill and the reap with `if status.is_none()`, so an already-exited child is neither
  killed nor reaped again.
- **The teardown allowance is bounded and stated.** The only work past the deadline is
  the kill and reap of a still-running direct child; it is bounded by that child's own
  termination and is stated in the rustdoc at `candidate.rs:159-161`. It is not a
  second deadline: no second timer, no tolerance, no retry, and no timeout value was
  changed anywhere.
- **Reader-thread lifecycle is explicit.** `poll_reader` (`candidate.rs:292-309`)
  reports a reader as finished only when it delivered its result or ended without one;
  the join rule at `candidate.rs:266-273` joins exactly those readers (or a reader
  whose handle already reports `is_finished()`), and abandons a reader still blocked
  in `read()`. The abandonment is documented truthfully in the rustdoc at
  `candidate.rs:163-169`.
- **A delivered result is not lost at the bound.** One last non-blocking drain runs
  after the loop (`candidate.rs:262-265`).
- **A reap failure no longer pins the call.** The baseline could loop forever when
  `child.wait()` failed after a kill (`status` stayed `None` and the break condition
  required `status.is_some()`); the corrected loop breaks on the overflow and
  deadline arms regardless of the reap outcome.
- **Rustdoc.** `CommandSpec::timeout` (`candidate.rs:63-74`), `CommandSpec::max_output_bytes`
  (`candidate.rs:77-86`), `CommandOutput::timed_out` (`candidate.rs:122-132`), and
  `run_bounded` (`candidate.rs:144-179`).
- **Fixture.** `crates/eggup-core/tests/candidate_deadline.rs` (547 lines), four helper
  roles plus eight fixtures; every fixture and helper name contains the substring
  `candidate`, so `cargo test -p eggup-core --locked candidate` selects all of them
  (verified below).

### The descendant fixture's mechanism

The fixture re-executes this very test binary as its helper, so it needs no shell, no
`cmd.exe`, and no external tool, and it runs natively on Linux, macOS, and Windows.

- The direct child is the test binary invoked with `--exact <descendant helper>`; the
  helper spawns a *holder* with `stdout`/`stderr` inherited, so the holder keeps both
  pipes open, then the direct child exits with code `7`.
- The holder waits for a release marker with a hard 120-second cap, so the descendant's
  lifetime is bounded even if the parent dies.
- The helper role is selected from the helper's own **argv**, not from the
  environment, because `run_bounded` clears the inherited environment
  (`candidate.rs:184`). The payload and exit code are passed through
  `CommandSpec::environment`, which `run_bounded` re-adds explicitly.
- The discriminating assertion is that the bounded call **returns**: the call runs on
  a worker thread and the parent waits with a 30-second bound whose expiry is a
  failure. It is not "returns within N milliseconds"; the fixture's own deadline is
  2 seconds and the generous bound is 15 times that, so a loaded host can only delay
  the fixed implementation, never make it hang.

## Exact commands run and results

All commands ran on **macOS 25.6.0 arm64 (darwin)**, Rust 1.89.0 toolchain, with
`--offline` on every cargo invocation.

### 1. `cargo fmt --all -- --check`

**Fails — but not because of this milestone.** The workspace-wide check reports
formatting diffs in `crates/eggup-service/src/commit_engine.rs`,
`crates/eggup-service/src/disposition.rs`,
`crates/eggup-service/src/lifecycle_update.rs`,
`crates/eggup-service/tests/commit_engine_parity.rs`,
`crates/eggup-service/tests/commit_engine_policy.rs`, and
`crates/eggup-service/tests/common/mod.rs`. Those files belong to the concurrent
`eggup-service` milestone and are outside this milestone's ownership; they were not
touched here. Scoped to this milestone's crate, the check is clean:

```
$ cargo fmt -p eggup-core
$ cargo fmt -p eggup-core -- --check
fmt OK for eggup-core
```

### 2. `cargo clippy -p eggup-core --all-targets --locked --offline -- -D warnings`

```
   Checking eggup-core v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-core)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.08s
```

Clean, no warnings. (The plan's workspace-wide clippy, `cargo test --workspace`, and
`./scripts/check-local.sh` are deliberately out of this stage's scope; the parent
session runs the full workspace gate once at the end.)

### 3. `cargo test -p eggup-core --locked --offline`

```
running 53 tests
test result: ok. 53 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.87s
running 12 tests
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.03s
running 11 tests
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.80s
running 14 tests
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

90 tests pass. The existing candidate fixtures in `crates/eggup-core/src/lib.rs`
(`bounded_candidates_are_exact_and_environment_is_cleared`,
`bounded_runner_kills_timeouts_and_limits_output`) were **not** rewritten and remain
green.

### 4. `cargo test -p eggup-core --locked --offline candidate`

```
running 2 tests
test tests::integrity_verification_precedes_candidate_validation ... ok
test tests::bounded_candidates_are_exact_and_environment_is_cleared ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 51 filtered out; finished in 0.25s
running 12 tests
test candidate_deadline_helper_sleeper_outlives_its_deadline ... ok
test candidate_deadline_helper_writes_output_and_exits ... ok
test candidate_deadline_helper_descendant_leaves_a_holder ... ok
test candidate_deadline_helper_holder_keeps_the_pipe_open ... ok
test candidate_deadline_spawn_failure_reports_the_operation ... ok
test candidate_deadline_non_zero_exit_is_distinguishable ... ok
test candidate_deadline_output_overflow_is_reported_and_discards_bytes ... ok
test candidate_deadline_normal_exit_is_unchanged ... ok
test candidate_deadline_timeout_kills_and_reaps_the_direct_child ... ok
test candidate_deadline_timeout_returns_and_leaves_the_runner_usable ... ok
test candidate_deadline_validator_fails_closed_on_a_candidate_that_overruns ... ok
test candidate_deadline_descendant_holding_the_pipe_returns_at_the_deadline ... ok
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.01s
running 1 test
test wrong_candidate_digest_fails_before_mutation ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.06s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.00s
```

All 12 new fixtures are selected by the `candidate` name filter.

### 5. `cargo test -p eggup-core --locked --offline --test candidate_deadline`

```
running 12 tests
test candidate_deadline_helper_descendant_leaves_a_holder ... ok
test candidate_deadline_helper_holder_keeps_the_pipe_open ... ok
test candidate_deadline_helper_sleeper_outlives_its_deadline ... ok
test candidate_deadline_helper_writes_output_and_exits ... ok
test candidate_deadline_spawn_failure_reports_the_operation ... ok
test candidate_deadline_output_overflow_is_reported_and_discards_bytes ... ok
test candidate_deadline_non_zero_exit_is_distinguishable ... ok
test candidate_deadline_normal_exit_is_unchanged ... ok
test candidate_deadline_timeout_kills_and_reaps_the_direct_child ... ok
test candidate_deadline_timeout_returns_and_leaves_the_runner_usable ... ok
test candidate_deadline_validator_fails_closed_on_a_candidate_that_overruns ... ok
test candidate_deadline_descendant_holding_the_pipe_returns_at_the_deadline ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.02s
```

### 6. Stability of the new fixture (fixed tree)

Ten consecutive runs of the new suite, all green, all ~2.02 s:

```
run 01: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.01s
run 02: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.01s
run 03: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.03s
run 04: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.01s
run 05: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.02s
run 06: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.02s
run 07: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.02s
run 08: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.02s
run 09: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.03s
run 10: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.03s
```

### 7. `cargo doc -p eggup-core --no-deps --locked --offline`

```
   Documenting eggup-core v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-core)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.47s
  Generated /Users/davidbowman/projects/eggup/target/doc/eggup_core/index.html
```

Clean, no rustdoc warnings. Rendered-section check:

```
$ grep -rho 'id="errors"' target/doc/eggup_core/ | wc -l
0
```

Zero `# Errors` sections were added anywhere; the crate-wide `# Errors` sweep remains
owned by core M015, and `clippy::missing_errors_doc` was not enabled.

### 8. `cargo tree -p eggup-core --locked --offline` (review step, not a gate)

```
eggup-core v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-core)
└── sha2 v0.10.9
    ├── cfg-if v1.0.5
    ├── cpufeatures v0.2.17
    │   └── libc v0.2.189
    └── digest v0.10.7
        ├── block-buffer v0.10.4
        │   └── generic-array v0.14.7
        │       └── typenum v1.20.1
        │           [build-dependencies]
        │           └── version_check v0.9.5
        └── crypto-common v0.1.7
            ├── generic-array v0.14.7 (*)
            └── typenum v1.20.1
```

No new dependency edge: `sha2` only on this platform, plus the Windows-only
`self-replace` which is not resolved on macOS. No `libc`, `nix`, `windows-sys`, or
process-group crate was added; `libc` appears only as a transitive dependency of
`sha2`'s `cpufeatures`, which was already there at the baseline.

### 9. `git diff --check`

```
$ git diff --check
```

Clean — no whitespace errors in the tracked files, and none in this milestone's files.

## Baseline-failure proof

**Mechanism used: `git worktree add` at the baseline commit** (the plan's §11.1
sequence, with a dedicated `CARGO_TARGET_DIR` so the main workspace build was not
disturbed). The `git stash` alternative was not used, and no git state was mutated.

```text
$ git worktree add /tmp/eggup-baseline-core014 c4805411f74b85158e0ae5d0a219a32c4809d98e
Preparing worktree (detached HEAD c480541)
HEAD is now at c480541 docs(service): reconcile wg-basic M004 closure

$ cp crates/eggup-core/tests/candidate_deadline.rs \
    /tmp/eggup-baseline-core014/crates/eggup-core/tests/candidate_deadline.rs

$ cd /tmp/eggup-baseline-core014
$ CARGO_TARGET_DIR=/tmp/eggup-baseline-core014-target \
    cargo test -p eggup-core --locked --offline --test candidate_deadline
```

Verbatim output of the proof run:

```text
running 12 tests
test candidate_deadline_helper_descendant_leaves_a_holder ... ok
test candidate_deadline_helper_sleeper_outlives_its_deadline ... ok
test candidate_deadline_helper_holder_keeps_the_pipe_open ... ok
test candidate_deadline_helper_writes_output_and_exits ... ok
test candidate_deadline_spawn_failure_reports_the_operation ... ok
test candidate_deadline_output_overflow_is_reported_and_discards_bytes ... ok
test candidate_deadline_non_zero_exit_is_distinguishable ... ok
test candidate_deadline_normal_exit_is_unchanged ... ok
test candidate_deadline_timeout_kills_and_reaps_the_direct_child ... ok
test candidate_deadline_timeout_returns_and_leaves_the_runner_usable ... ok
test candidate_deadline_validator_fails_closed_on_a_candidate_that_overruns ... ok
test candidate_deadline_descendant_holding_the_pipe_returns_at_the_deadline ... FAILED

failures:

---- candidate_deadline_descendant_holding_the_pipe_returns_at_the_deadline stdout ----

thread 'candidate_deadline_descendant_holding_the_pipe_returns_at_the_deadline' panicked at crates/eggup-core/tests/candidate_deadline.rs:176:26:
the bounded call did not return within 30s
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    candidate_deadline_descendant_holding_the_pipe_returns_at_the_deadline

test result: FAILED. 11 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 30.02s

error: test failed, to rerun pass `-p eggup-core --test candidate_deadline`
```

**Exit status: 101 (non-zero).** The fixture **compiles** against the pre-M014
baseline — it is a runtime failure, not a compile failure — and exactly the
discriminating fixture fails. The other 11 tests pass on the baseline, which matches
the plan's expectation that fixtures 2–6 are regression guards green on both trees.

**Reproducibility.** The baseline proof was executed **10 times**; the descendant
fixture failed on **10 of 10** runs, and the other 11 fixtures passed on all 10. The
worktree was then removed with `git worktree remove --force /tmp/eggup-baseline-core014`.

### Root-cause confirmation

An instrumented copy of the fixture (a scratch edit inside the throwaway worktree,
not part of the deliverable) dumped the process table at the moment the bounded call
failed to return. Every run showed the same shape:

```text
79773 79772 S          00:30 .../candidate_deadline-<hash>                                  # the test binary
79781     1 S          00:30 .../candidate_deadline-<hash> --exact candidate_deadline_helper_holder_keeps_the_pipe_open --nocapture
```

The holder process — reparented to `init` because the direct child already exited —
is still alive at 30 seconds and still holds the pipe write end open. That is the
defect exactly as the plan's §3.2 describes it, and it is why the baseline's
`run_bounded` never returns.

## Invariant review (§4, item by item)

| §4 invariant | Status | Evidence |
|---|---|---|
| `timeout` remains a **maximum** wall-clock bound; never widened, no tolerance, no renewal, no retry | **Holds** | No timeout value was changed: `DEFAULT_TIMEOUT` is still 5 s (`candidate.rs:14`) and the test's own deadlines are the fixture's choice. The deadline is now enforced *unconditionally*, which is a tightening, not a widening. No second timer exists in the implementation. |
| The output bound is unchanged: per stream, overflow discards that stream's bytes, defaults 64 KiB / 5 s | **Holds** | `DEFAULT_OUTPUT_LIMIT` is still 64 KiB (`candidate.rs:15`); `read_limited` is unchanged (`candidate.rs:311-324`); `max_output_bytes` is still passed separately per stream. Only the *wording* of the doc changed. |
| `CommandOutput` field meanings are stable; `success()` stays the three-part definition; `success()` is `false` for every deadline expiry | **Holds** | `success()` is unchanged (`candidate.rs:138-141`); `timed_out` is now true in the descendant case too, which is what makes `success()` false there. `candidate_deadline_descendant_holding_the_pipe_returns_at_the_deadline` asserts `success() == false`. |
| `CommandOutput`'s published shape and `CommandSpec` are unchanged; no new public type | **Holds** | Same fields, same accessors; `poll_reader` is a private function. No `pub` item was added, removed, or re-exported. |
| `Error`'s published variant set is unchanged (seven variants) | **Holds** | `crates/eggup-core/src/error.rs` was not touched. No variant was added. |
| The ownership model is untouched | **Holds** | `Ownership`, `OwnershipVerifier`, and `classify_all` were not touched; the change is confined to the bounded runner. |
| Core never creates a live destination parent; candidate execution never mutates a live destination; validation runs before the mutation lock | **Holds** | `ValidatedTransaction::validate` (`candidate.rs:511-522`) is unchanged and still runs before any lock is taken. A deadline expiry produces `Err(Error::CandidateExecution(...))` — never a receipt, never a success. |
| `run_bounded` keeps clearing the inherited environment and passing null stdin | **Holds** | `candidate.rs:184-186` — `env_clear()` and `Stdio::null()` are unchanged. |
| `eggup-core` keeps its dependency surface | **Holds** | `cargo tree` above: `sha2` only on this platform. No new dependency, no `build.rs`, no FFI, no `unsafe` (`#![forbid(unsafe_code)]` is untouched and the build is clean). |
| Rust 1.89 MSRV; `missing_docs` satisfied | **Holds** | Built and tested with the 1.89.0 toolchain; `cargo doc` is clean. `JoinHandle::is_finished` has been stable since 1.61, so the MSRV is unaffected. The hosted `msrv` lane was **not run** (see the platform matrix). |

## Failure / rollback / recovery review

- **A deadline expiry is never a success.** `success()` is false, and both validators
  convert a non-successful `CommandOutput` into `Err(Error::CandidateExecution(...))`
  (`candidate.rs:419-427`). Proven by
  `candidate_deadline_validator_fails_closed_on_a_candidate_that_overruns`.
- **Fail-closed direction.** Every uncertain outcome — deadline expiry, output
  overflow, a pipe that never closed — is reported as a failure with an explicit flag,
  never as a partially-trusted success.
- **Transaction state is untouched by a deadline expiry.** Candidate validation runs
  before the mutation lock is acquired and before any live destination is touched, so
  a deadline expiry leaves **no** transaction state: no lock held, no backup set, no
  receipt, no live file modified. The caller receives
  `Err(Error::CandidateExecution(...))` and may retry under its own policy; core
  imposes no retry.
- **Restart semantics are unchanged.** `run_bounded` holds no persistent state and
  creates no file, so a failed candidate leaves nothing to clean up beyond the direct
  child, which is killed and reaped at the deadline.
- **Contention is unchanged.** `run_bounded` does not take the mutation lock and does
  not interact with the lock record.
- **Descendants are not terminated.** Core kills and reaps only the direct child it
  spawned. A descendant that inherited the pipe survives the call. This is stated in
  the rustdoc (`candidate.rs:152-157`) and in
  `architecture/core-transaction.md`, not implied away.
- **Reader-thread residual — recorded truthfully.** A reader still blocked in `read()`
  at the deadline is abandoned, not reclaimed. It ends when the pipe reaches EOF,
  that is, when the holding descendant exits or closes the pipe. In the descendant
  case that means the reader thread lives as long as the descendant does; core cannot
  shorten it because it owns no handle to the descendant (no `libc` / `nix` /
  `windows-sys`, no job object, `unsafe` denied). The number of reader threads per
  call stays at two. This is the residual the plan's §6.5 and §15 require to be stated
  plainly rather than papered over.
- **Reader reclamation is not portably observable.** `candidate_deadline_timeout_returns_and_leaves_the_runner_usable`
  asserts what *is* observable — the bounded call returns, and a later bounded call
  still succeeds — and the reasoning is recorded here: with no descendant, the killed
  child's pipe write end was the only holder, so each reader reaches EOF and ends on
  its own. Core cannot prove thread reclamation from a test without a
  platform-specific thread counter, so **no such proof is claimed**. In the ordinary
  timeout path the reader is very likely to have already ended when the join rule is
  evaluated (`is_finished()`), and it is joined; only a genuinely blocked reader is
  abandoned.
- **Poll-failure path is unchanged.** `child.try_wait()` failure still returns
  `Err(Error::Io { operation: "polling bounded candidate", .. })` and still does not
  kill the child. That is pre-existing behavior; the plan (§8) states M014 does not
  widen it, so no kill was added on that path. It is recorded as an unresolved
  finding below.

## Compatibility and migration review

- **Source compatibility: unchanged.** `CommandSpec`, `CommandOutput`, and every
  accessor keep their published shape. `Error` keeps exactly the seven variants
  `0.1.2` published (`InvalidInput`, `UnknownMember`, `Io`, `UpdateInProgress`,
  `DestinationConflict`, `VerificationFailed`, `CandidateExecution`), so an exhaustive
  downstream `match` on `Error` still compiles — the property M012 protected is
  untouched, and the crate's own exhaustive matches compile unchanged
  (`cargo test -p eggup-core --locked` is green).
- **Behavior compatibility.** The behavior change is confined to calls that today
  never return: such a call now returns at the deadline with `timed_out == true`.
  No caller can depend on a hang, so this is a defect correction, not a semantic
  break. Every previously-returning path keeps its previous result, with one
  deliberate refinement: an execution that completed (the child exited and both pipes
  reached EOF) is reported as a success even if the poll that observed the completion
  also observed the deadline elapsing. The refinement is fail-open only in the
  direction of "the work demonstrably finished"; an execution that did *not* finish is
  still reported as a timeout.
- **Published `0.1.3` still carries the defect.** `eggup-core 0.1.3` is published
  (crates.io, published 2026-10-06 from `bd43683`). M014 changes packaged behavior in
  `src/candidate.rs`, so the published `0.1.3` retains the defect and a consumer
  resolving it can still hang. The fix reaches consumers only when a new version is
  published.
- **M014 must not publish, and does not.** No version was bumped, no `Cargo.toml` was
  touched, and `Cargo.lock` is unchanged. Publication is a tracked milestone of its
  own (M013 is the precedent); the crate changelog states that the published `0.1.3`
  is still affected and that republication is authorized only by a separate
  milestone.
- **The `eggup-eggpack` pin blocks propagation.** `eggup-eggpack` pins
  `eggup-core = { version = "=0.1.3", path = "../eggup-core" }`
  (`crates/eggup-eggpack/Cargo.toml:20`; likewise `eggup-archive` and
  `eggup-acquisition`). The core fix cannot reach `eggup-eggpack` until a separate
  adapter republication re-pins the new core version. That republication is its own
  milestone, sequenced after the core publication.
- **Packaged bytes.** The new integration test file is excluded from the package
  (`crates/eggup-core/Cargo.toml:15-19`), so only the `src/` change alters the
  packaged artifact.
- **Migration path: none required.** No API changed, no data format changed, and no
  consumer action is needed beyond picking up the fixed version.

## Security review

- The inherited environment is still cleared (`candidate.rs:184`); only explicitly
  added variables reach the candidate (`candidate.rs:191-193`). Unchanged.
- Stdin is still null (`candidate.rs:185`). Unchanged.
- Captured output is still bounded per stream (`max_output_bytes`, default 64 KiB) and
  an overflowing stream is still discarded rather than truncated-and-trusted.
- A partially captured stream can never be mistaken for a complete one: the deadline
  path reports `timed_out`, the overflow path reports `output_limited`, and neither is
  a success. The descendant path contributes **no** bytes for the stream the
  descendant still holds, because they were never captured — the caller is not told
  they were.
- The change adds **no dependency** and **no `unsafe`**; the crate keeps
  `#![forbid(unsafe_code)]` and `sha2` only.
- The change does not widen any authority: core still kills only the direct child it
  spawned, and never terminates a process it does not own.

## Documentation / operations evidence

| Document | Change | Status |
|---|---|---|
| `crates/eggup-core/src/candidate.rs` — `CommandSpec::timeout` | States that the value is an unconditional maximum bound on the whole bounded execution, evaluated whether or not the direct child exited, and that the deadline starts after the spawn returns | done (`candidate.rs:63-74`) |
| `crates/eggup-core/src/candidate.rs` — `CommandSpec::max_output_bytes` | Says unambiguously that the bound is **per stream** and that an overflowing stream is reported as empty | done (`candidate.rs:77-86`) |
| `crates/eggup-core/src/candidate.rs` — `CommandOutput::timed_out` | Amended so it is true in both deadline cases: the deadline was reached, and the direct child is killed **only if it was still running** | done (`candidate.rs:122-132`) |
| `crates/eggup-core/src/candidate.rs` — `run_bounded` | States the unconditional deadline, the descendant case, the reader-thread abandonment, the bounded teardown allowance, and the four observable outcomes | done (`candidate.rs:144-179`) |
| `crates/eggup-core/CHANGELOG.md` | `Unreleased` entry: the correction, the observable change, the unpublished status, the unchanged public surface, and the absence of a migration path | done |
| `crates/eggup-core/docs/verification.md` | Candidate-command paragraph states the unconditional bound and points at the normative rustdoc and architecture paragraph rather than restating a new rule | done |
| `architecture/core-transaction.md` | `run_bounded` paragraph states the unconditional deadline and the descendant caveat; the export-table row no longer contradicts it | done (`:117`, `:302-320`) |
| root `CHANGELOG.md` | Matching aggregate `Unreleased` entry | **not done — owed by the parent session** (the root changelog is outside this milestone's ownership) |

Diff summary for the tracked files in this milestone's ownership:

```text
 architecture/core-transaction.md       |  14 +++-
 crates/eggup-core/CHANGELOG.md          |  43 ++++++++++
 crates/eggup-core/docs/verification.md  |  14 ++--
 crates/eggup-core/src/candidate.rs      | 147 +++++++++++++++++++++++++++------
 4 files changed, 188 insertions(+), 30 deletions(-)
```

plus the new file `crates/eggup-core/tests/candidate_deadline.rs` (547 lines).

Operations: no new configuration, no new environment variable, no new file format,
no migration step, and no change to any operational runbook. The only new runtime
artifact is the abandoned reader thread described above, which is bounded by the
descendant's lifetime.

## Platform results

| Platform | Result | How |
|---|---|---|
| macOS 25.6.0 arm64 (darwin) | **PASS** — 12/12 new fixtures green, 90/90 crate tests green, clippy clean, rustdoc clean, 10/10 stability runs green | Native run on this machine |
| Linux | **NOT RUN** | No Linux host is reachable from this machine. Not inferred from macOS. |
| Windows | **NOT RUN** | No Windows host is reachable from this machine. Not inferred from macOS. |
| Hosted `stable` lane | **NOT RUN** | No hosted runner is reachable from this machine. |
| Hosted `msrv` lane | **NOT RUN** | No hosted runner is reachable from this machine. |
| Hosted `macos` lane | **NOT RUN** | No hosted runner is reachable from this machine. |
| Hosted `windows-check` lane | **NOT RUN** | No hosted runner is reachable from this machine. |
| Hosted `service-systemd` lane | **NOT RUN** | No hosted runner is reachable from this machine. |

The plan's §11.2 hosted qualification, including the requirement that the
`windows-check` lane be confirmed to have executed the new test file, is therefore
**still outstanding**. The fixture is written to satisfy it: it uses no feature gate
(the `windows-check` lane runs without `--all-features`), no shell, and no external
tool, and it selects its helper roles from argv rather than from the environment
because `run_bounded` clears the environment.

## Unresolved findings with severity

**F1 — Hosted qualification and the Linux / Windows platforms are not run. Severity:
medium.** The reason for the severity: the *implementation* is platform-neutral
Rust with no platform-specific code, and the fixture is designed to be
platform-neutral (argv-selected helper roles, no shell, no external tool, no
platform-specific thread counter). But "designed to be portable" is not "observed to
be portable", and the plan's §13 requires a fresh hosted `stable` / `msrv` / `macos`
/ `windows-check` matrix with the new fixture observed executing in the Windows lane.
That evidence cannot be manufactured from a macOS machine. The milestone is closed
with this finding open rather than closed as fully qualified. Owner: the parent
session, which runs the workspace gate and the hosted lanes.

**F2 — `eggup-service` carries its own bounded subprocess runner. Severity: medium.**
Recorded in the plan's §3.5. `SystemExecutor` (`crates/eggup-service/src/lib.rs:671`),
`fn run` (`:700`), and `read_bounded_generic` (`:952`) duplicate core's `run_bounded`,
and `eggup-service` already depends on `eggup-core`, so the duplication is not
required by any dependency-boundary rule. The reason for the severity: the same
class of defect — a deadline that is not unconditionally enforced — is plausible in
the duplicated runner, and the service crate is not covered by this milestone's
fixtures; but the service runner is a separate implementation whose own deadline
contract has not been read-verified here, and the plan explicitly forbids touching
`eggup-service` from M014. Owner: a separate service milestone, which must first
resolve the recorded overflow-semantics asymmetry (core discards the overflowing
stream's bytes; the service retains up to `limit + 8192` bytes).

**F3 — A reader I/O error is reported as `output_limited`. Severity: low.** Recorded in
the plan's §3.4. `poll_reader` keeps the published semantics: a read failure is
indistinguishable from an output-bound overflow, because both are carried as the
`Err` arm of the reader's result. The reason for the severity: it is a
classification imprecision on a fail-closed path — an overflow and an I/O error both
report failure, so no caller can be misled into trusting a truncated stream — and
the plan forbids widening M014's scope to fix it. M014 does not make it worse: the
same `result.is_err()` classification is applied before and after.

**F4 — The output bound is per stream, not combined. Severity: low.** Recorded in the
plan's §3.4. `max_output_bytes` bounds stdout and stderr independently, so one run
can capture up to twice the configured value in total. The reason for the severity:
it is a documentation-accuracy issue, not a safety hole — each stream is still
bounded, and the total volume is bounded at 2×; and the plan forbids changing the
bound. M014 resolves the *wording* (the field doc now says unambiguously that the
bound is per stream) without changing the bound itself.

**F5 — A blocking spawn is not covered by the deadline. Severity: low.** Recorded in
the plan's §3.4. The deadline is computed after `command.spawn()` returns, so a spawn
that blocks is not bounded. The reason for the severity: bounding a spawn would
require a platform-specific mechanism (a supervisor process, a job object, or
`posix_spawn` semantics) that the dependency and `unsafe` constraints exclude, and
the plan states M014 does not attempt it. The rustdoc now says so explicitly, so the
published contract is accurate about what is and is not bounded.

**F6 — The poll-failure path does not kill the child. Severity: low.** Pre-existing
behavior, unchanged by M014. If `child.try_wait()` fails, `run_bounded` returns
`Err(Error::Io { operation: "polling bounded candidate", .. })` and the `Child` is
dropped without being killed, so the child may outlive the call as an orphan. The
reason for the severity: the plan (§8) states the existing behavior is unchanged and
M014 does not widen it, and adding a kill on that path would be an additive change to
an error path that the plan requires to be justified separately. It is a rare,
platform-level failure, and the caller receives a hard `Err` and no result, so nothing
is trusted. Recorded for a follow-up.

**F7 — A reader blocked at the deadline is abandoned, not reclaimed. Severity: low.**
This is the §6.5 residual, recorded here for completeness rather than as a defect:
in the descendant case the reader thread lives as long as the holding descendant.
The reason for the severity: the thread count per call is fixed at two, the thread
ends deterministically when the pipe reaches EOF, and core has no lawful way to
shorten it (no process-group or job-object dependency, `unsafe` denied). The
abandonment is documented truthfully in the rustdoc rather than presented as
reclamation.

**F8 — One-off interference observed in the first baseline proof run. Severity:
low.** In the very first baseline run, the `candidate_deadline_non_zero_exit_is_distinguishable`
fixture also failed, with the same "the bounded call did not return within 30s"
message, in the same run as the expected descendant failure. It did not reproduce in
any subsequent run: the baseline proof then failed on the descendant fixture **10 of
10** times with the other 11 fixtures green, and the fixed tree was green **10 of 10**
times. The reason for the severity: it was observed once and never reproduced, the
fixed tree has no sign of it, and it occurred in the *baseline* tree, where the
descendant fixture deliberately stalls the runner for 30 seconds by design. It is
recorded here rather than dismissed, because the plan forbids classifying a genuine
runtime failure as flakiness: the instrumented process dumps collected during those
runs always showed the holder process holding the pipe, which is the expected
mechanism, and the non-zero fixture's own child was never observed lingering. The
mechanism behind the one-off interference was not conclusively established, so it is
reported as an observation with an unknown cause rather than as explained.

## Roadmap disposition

- `plans/subsystems/verified-update-core-roadmap.md` § 7 (the milestone status table)
  gains a row for **M014 — bounded-execution deadline correctness corrective**,
  recorded as **closed with unresolved findings** (native macOS evidence only; hosted
  qualification outstanding, see F1).
- The roadmap's summary line for the subsystem moves from "M001-M013 closed /
  hosted-green" to "M001-M013 closed / hosted-green; M014 closed (native macOS
  evidence), hosted qualification outstanding".
- `plans/registry.md` § "Post-0.1.3 open findings" entry **3** ("`eggup-core` bounded
  candidate execution can exceed its documented wall-clock deadline") is **closed**
  by M014 and moved out of the open-findings list.
- `plans/registry.md` § "Active subsystem roadmaps" records Verified update core as
  "M001-M013 closed/hosted-green, M014 closed (hosted qualification outstanding)".
- No new open finding is registered by M014 itself. The findings M014 records
  (F2–F7) are inherited from the plan's §3.4 / §3.5 observations and from the
  platform-evidence gap; the parent session decides whether F2 is registered as a
  new open finding for the service subsystem, which is the plan's §16 follow-up.
- The root `CHANGELOG.md` `Unreleased` aggregate entry for this correction is owed
  by the parent session (the root changelog is outside this milestone's ownership).

## Registry updates

The parent session should apply the following, verbatim:

**`plans/registry.md` § "Post-0.1.3 open findings":** remove entry **3**
("`eggup-core` bounded candidate execution can exceed its documented wall-clock
deadline") from the open-findings list, and record it as closed by
`plans/closure/verified-update-core/014-status.md` at baseline `c480541`. Entry 3 was
registered at the same baseline and is assessed **high**; it is now corrected in the
working tree.

**`plans/registry.md` § "Active subsystem roadmaps":** change the Verified update
core entry from "M001-M013 closed/hosted-green" to "M001-M013 closed/hosted-green;
M014 closed (native macOS evidence), hosted qualification outstanding".

**`plans/subsystems/verified-update-core-roadmap.md` § 7 status table:** add the row

```text
| M014 | bounded-execution deadline correctness corrective | closed (native macOS evidence); hosted qualification outstanding | plans/implementation/verified-update-core/014-bounded-execution-deadline-corrective.md | plans/closure/verified-update-core/014-status.md |
```

**`plans/subsystems/verified-update-core-roadmap.md` summary:** update the subsystem
status line to "M001-M013 closed / hosted-green; M014 closed (native macOS evidence),
hosted qualification outstanding".

**Root `CHANGELOG.md` `Unreleased`:** add the aggregate entry for the M014
correction, matching `crates/eggup-core/CHANGELOG.md` — the unbounded candidate
execution is corrected, the fix is unpublished, the published `eggup-core 0.1.3` still
carries the defect, and no publication milestone authorizes it.

**Not applied by this milestone:** the root `CHANGELOG.md` edit, the `plans/registry.md`
edit, and the roadmap edit — all three are outside this milestone's ownership and are
described here as prose for the parent to apply.

---

## Parent-session verification addendum (2026-10-23)

The implementation milestone recorded the whole-tree gate as deliberately **not
run** at this stage, because the parent session owns it. The parent session has
since run it. This addendum records that result so the closure record does not
leave a stale "not run" claim standing against a command that has now been
executed.

Commands run by the parent session after this milestone closed, from the same
working tree and the same baseline `c4805411f74b85158e0ae5d0a219a32c4209d98e`:

```sh
./scripts/check-local.sh
# → exit 0
cargo test --workspace --all-targets --all-features --locked --offline --no-fail-fast
# → 449 passed; 0 failed; 36 test binaries
```

`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`,
and `cargo doc --workspace --no-deps --locked` are all clean as part of the gate.

**Still not run:** Linux, Windows, and every hosted CI lane (`stable`, `msrv`,
`service-systemd`, `macos`, `windows-check`). No hosted runner is reachable from
this machine, so this milestone is qualified on the native macOS lane only.
