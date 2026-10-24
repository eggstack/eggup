# Closure record — service-lifecycle M015

## Status

**Closed, with one residual published-crate finding recorded for the parent.**

Every production change, fixture, guard, and documentation update this
milestone requires is in the tree, every requirement is proven on the native
**macOS** lane, and the milestone's own headline requirement — that
`SystemExecutor` delegates to `eggup_core::run_bounded` — is proven
structurally (`grep -c "std::process::Command" crates/eggup-service/src/lib.rs`
→ `0`) and behaviourally (the full crate suite passes, including the M014
public-API guard with its snapshot unmodified).

One item is **not** closed by this milestone and is recorded at **medium**
severity: **UF-1**, a defect in the *published* `eggup-service 0.1.3` — its
bounded runner reports a **successful** stdin write as a stdin failure. The
consolidation **fixes** that defect in the source line (it is the difference
**D8**), but the published crate keeps it until a republication milestone.
Under the repository's rule that *"medium-or-higher is the de-facto
blocking-severity gate"*, that residual is the parent's decision: see
"Unresolved findings with severity" and "Roadmap disposition".

## Source plan and roadmap

- Source plan: `plans/implementation/service-lifecycle/015-bounded-executor-consolidation-corrective.md`
  (`Status: implemented`).
- Roadmap: `plans/subsystems/service-lifecycle-roadmap.md` (service-lifecycle
  subsystem). **The parent owns that file**; the drafted row is in
  "Roadmap disposition" below.
- Finding discharged: `plans/closure/service-lifecycle/012-status.md` §
  "Unresolved findings with severity" — **Finding 1**, severity **low**:
  *"`SystemExecutor` is not consolidated onto `eggup_core::run_bounded`."*
- Prerequisite: Verified Update Core **M017**
  (`plans/implementation/verified-update-core/017-bounded-stdin-execution-corrective.md`),
  which landed in this working tree as **uncommitted** work owned by another
  milestone. It was **read only** by this milestone; nothing under
  `crates/eggup-core/**` was modified here.

## Reviewed repository baseline

- HEAD at implementation time: `20ac770dacfdcd3d82e734c22613bfccd36f9c09`
  on branch `plans/corrective-batch-2026-10-23`.
- The working tree was **not** clean at implementation time. It carried
  uncommitted **Core M017**, **Acquisition M014**, and the transport-footprint
  planning documents, and it continued to change under other agents while
  this milestone was in flight (the parent applied `plans/registry.md`,
  `plans/subsystems/*`, the root `CHANGELOG.md`, and
  `.github/workflows/ci.yml` during the implementation window). **No file
  outside this milestone's ownership was modified here**, and **no git state
  was mutated**: no stash, checkout, restore, commit, push, tag, clean, reset,
  or branch creation was performed. The baseline evidence below was produced
  with `git archive` (read-only) into a scratch directory, so no worktree or
  index state was touched.

## Implementation commits / PRs

None. The working tree is uncommitted; the parent session owns the commit and
the branch. **Nothing was published, no tag moved, and no version was
bumped.**

## Executive finding

`eggup-service` carried a **second**, hand-rolled implementation of the
bounded-subprocess mechanic: `SystemExecutor::run` spawned the child itself,
attached its own reader threads, wrote stdin on its own writer thread, and
drove its own deadline-checked wait loop. `eggup-core` had carried the same
mechanic — `run_bounded` — since core M001. Two implementations of one
safety-relevant mechanic is exactly the drift risk the repository's
architecture rules exist to prevent, and it was already recorded as **Finding
1** of the M012 closure at **low** severity.

This milestone removes the second implementation. `SystemExecutor::run` is now
a **thin policy layer over `eggup_core::run_bounded`**: it keeps every policy
decision in front of the core call — the argv emptiness and NUL checks, the
timeout range check, the `MAX_CRONTAB_BYTES` stdin bound, the allowlisted
absolute program path, and the filtered child environment — and it maps core's
result onto the service's taxonomy with an exhaustive, explicitly fall-back
match.

What the consolidation did **not** change: the `CommandExecutor` trait,
`SystemExecutor` (including its **public** `max_output_bytes` field),
`ManagerCommandOutput`, the `ServiceError` / `ManagerFailureKind` taxonomies,
every manager's argv, the executable allowlist, and the environment policy.
The M014 public-API guard passes with its snapshot **unmodified**.

What the consolidation **did** change, beyond the seven differences the plan
enumerated: it **fixed a defect in the runner it replaced**. The old runner
polled its stdin-writer channel with `try_recv()` on every wait-loop
iteration, and `try_recv` *removes* the message — so a verdict of `Ok(())` was
consumed and discarded by a check that only looks for `Err(_)`, and the
post-exit check then found an empty, disconnected channel and reported
`Manager(Spawn, "manager stdin failed for {program}")` for a write that had
**succeeded**. The affected path is the `crontab -` install, the crate's only
production stdin caller. The defect is present in the **published**
`eggup-service 0.1.3` (source `feb6ae5`) and is recorded as **D8** and as
**UF-1** below.

## Core-contract confirmation (R-D1)

Quoted from the working tree, uncommitted:

| Contract item | `file:line` | Quoted text |
|---|---|---|
| `MAX_STDIN_BYTES` | `crates/eggup-core/src/candidate.rs:27` | `pub const MAX_STDIN_BYTES: usize = 1024 * 1024;` |
| `CommandSpec::stdin_data` | `crates/eggup-core/src/candidate.rs:125` | `pub fn stdin_data(mut self, data: impl Into<Vec<u8>>) -> Self {` |
| Operation `"spawning bounded candidate"` | `crates/eggup-core/src/candidate.rs:307` | `.map_err(\|source\| Error::io("spawning bounded candidate", source))?;` |
| Operation `"polling bounded candidate"` | `crates/eggup-core/src/candidate.rs:373` | `.map_err(\|source\| Error::io("polling bounded candidate", source))?;` |
| Operation `"writing bounded stdin"` | `crates/eggup-core/src/candidate.rs:439` | `return Err(Error::io("writing bounded stdin", source));` |
| `Error::Io` public fields | `crates/eggup-core/src/error.rs:34-39` | `Io {` / `operation: &'static str,` / `source: io::Error,` / `},` |
| `Error::source()` yields the inner `io::Error` | `crates/eggup-core/src/error.rs:216`, `:219` | `impl std::error::Error for Error {` / `Self::Io { source, .. } => Some(source),` |
| Crate-root re-export | `crates/eggup-core/src/lib.rs:32` | `pub use candidate::{` |

All three operation strings match the plan's §3.4 contract **exactly**;
`Error`'s variant set is the published seven (`InvalidInput`,
`UnknownMember`, `Io`, `UpdateInProgress`, `DestinationConflict`,
`VerificationFailed`, `CandidateExecution`), plus the `#[cfg(test)]`-only
`Injected` variant, which does not exist in a dependency build. **§14's stop
conditions 1, 2 and 4 did not fire.** Nothing under `crates/eggup-core/**`
was modified (R-D2).

## Requirement → evidence matrix

| Requirement | Evidence (`file:line`) | Command and observed result |
|---|---|---|
| **R-A1** — the six pre-flight checks, in order, ahead of core | `crates/eggup-service/src/lib.rs:1032` (argv emptiness), `:1035` (NUL), `:1038` (timeout range), `:1041` (stdin bound), `:1050` (`resolve_manager_program`), `:1070` (`filtered_manager_environment`) | `cargo test -p eggup-service --locked --offline` → `rejects_a_zero_timeout_an_over_range_timeout_and_an_over_bound_stdin_payload ... ok`; `the_pre_flight_rejections_are_service_errors_not_core_errors ... ok` |
| **R-A2** — one `CommandSpec`, one `run_bounded` | `lib.rs:1062-1065` (`CommandSpec::new` / `args` / `timeout` / `max_output_bytes`), `:1070-1075` (`environment`), `:1076-1078` (`stdin_data`), `:1080` (`run_bounded`) | `grep -c "std::process::Command" crates/eggup-service/src/lib.rs` → `0`; `grep -rn "read_bounded_generic\|Stdio::piped\|thread::spawn" crates/eggup-service/src/lib.rs` → no hits |
| **R-A3** — the `program` string is the resolved path | `lib.rs:1051` (`program_path.to_string_lossy()`) | `classifies_a_missing_program_as_not_found_with_the_current_detail ... ok`; `classifies_a_spawn_failure_as_a_typed_manager_failure ... ok` |
| **R-A4** — the consolidation is internal; the rustdoc names `run_bounded` | `lib.rs:976-990` (the rustdoc), `:989-992` (the struct and its **public** field), `:994` (`Default`), `:1013` (`with_max_output`) | `cargo test -p eggup-service --locked --offline --test public_api_surface` → `the_committed_snapshot_lists_every_public_item ... ok`, `every_top_level_public_item_is_pinned ... ok`; `git diff --stat -- crates/eggup-service/tests/public-api-surface.txt` → no output |
| **R-A5** — `read_bounded_generic` deleted iff it had no other caller | deleted from `crates/eggup-service/src/lib.rs` (it occupied `:1305-1328` before the change) | `grep -rn "read_bounded_generic" crates/eggup-service/src/` → no hits |
| **R-B1** — the mapping table, exhaustively, with a fallback arm | `lib.rs:1080-1139`: `:1082` (`timed_out`), `:1088` (`output_limited`), `:1094-1098` (success), `:1104-1111` (spawn), `:1112-1115` (poll), `:1116-1119` (stdin write), `:1123-1128` (**the fallback arm**), `:1130-1133` (`CandidateExecution`), `:1134` (`InvalidInput`), `:1135-1138` (the remaining variants) | `cargo test -p eggup-service --locked --offline` → all twelve tests in `bounded_executor_consolidation` pass |
| **R-B2** — the timeout is the caller's timeout less the elapsed pre-flight; a zero remaining budget spawns nothing | `lib.rs:1049` (the budget starts before the resolve), `:1052-1054` (the remaining budget), `:1055-1060` (the zero-budget early return, before any spawn) | `cargo test -p eggup-service --locked --offline` → `a_timeout_yields_the_timeout_kind ... ok` |
| **R-B3** — a valid-UTF-8 pair is byte-identical; a non-UTF-8 pair is dropped | `lib.rs:1070-1075` | `cargo test -p eggup-service --locked --offline --test bounded_executor_environment` → `the_child_environment_is_cleared_and_only_the_session_bus_variables_are_injected ... ok` |
| **R-C1** — the argv builders untouched; no existing `fake.expect(...)` vector edited | `lib.rs:1691` (`show_argv`), `:1729` (`run_unit`), `:1894` (`is-active`), `:2392` (`disable`), `:2428` and `:2470` (`daemon-reload`), `:2481` (`enable`), `:2658` (`launchctl list`), `:2938` (`bootout`), `:3024` (`stop`), `:3046` (`bootstrap`), `:3059` (`kickstart`), `:3412` (`crontab -l`), `:3455` (`crontab -`) | `cargo test -p eggup-service --locked --offline` → `the_managers_argv_is_byte_identical ... ok`; `git diff -- crates/eggup-service/src/lib.rs` shows no change to any `fake.expect(` vector |
| **R-C2** — the taxonomies unchanged | `lib.rs:339` (`ManagerFailureKind`), `:406` (`ServiceError`) | `cargo test -p eggup-service --locked --offline --test manager_failure_kinds` → `14 passed; 0 failed` |
| **R-C3** — the M014 public-API guard passes with the snapshot unmodified | `crates/eggup-service/tests/public-api-surface.txt` (unmodified) | `cargo test -p eggup-service --locked --offline --test public_api_surface` → `5 passed; 0 failed`; `git diff --stat -- crates/eggup-service/tests/public-api-surface.txt` → no output |
| **R-C4** — `Cargo.toml` and `Cargo.lock` byte-identical | `crates/eggup-service/Cargo.toml`, `Cargo.lock` (unmodified) | `git status --short -- crates/eggup-service/Cargo.toml Cargo.lock` → no output; `cargo tree -p eggup-service --locked --offline --depth 1` → `eggup-core v0.1.3` + `windows-args v0.2.0` (unchanged) |
| **R-D1** — the core contract confirmed | see "Core-contract confirmation (R-D1)" above | quoted lines from the working tree |
| **R-D2** — no file under `crates/eggup-core/**` modified | — | `git status --short -- crates/eggup-core` shows only **M017's** entries (`crates/eggup-core/src/candidate.rs`, `src/lib.rs`, `README.md`, `CHANGELOG.md`, `docs/verification.md`, `tests/candidate_stdin.rs`), all of which pre-date this milestone and are M017's |
| **R-D3** — core's `run_bounded` expresses bounded stdin *and* an unconditional deadline that bounds the writer | `crates/eggup-core/src/candidate.rs:278-452` | `delivers_the_stdin_payload_to_the_child ... ok` and `a_timeout_yields_the_timeout_kind ... ok` |

### The mapping, as implemented (R-B1)

| core result | service result | `file:line` |
|---|---|---|
| `Ok(out)`, `out.timed_out()` | `Err(manager(Timeout, "manager command timed out for {program}"))` | `lib.rs:1082-1087` |
| `Ok(out)`, `out.output_limited()` | `Err(manager(Protocol, "manager output exceeded bound for {program}"))` | `lib.rs:1088-1093` |
| `Ok(out)` otherwise | `Ok(ManagerCommandOutput { status: out.exit_code(), stdout: out.stdout().to_vec(), stderr: out.stderr().to_vec() })` | `lib.rs:1094-1098` |
| `Err(Error::Io { operation, source })`, `operation == "spawning bounded candidate"` | `from_spawn_error(&source)`; `NotFound` → `"manager binary missing: {program}"`, else `"manager spawn failed for {program}: {source}"` | `lib.rs:1104-1111` |
| `Err(Error::Io { .. })`, `operation == "polling bounded candidate"` | `Err(manager(Spawn, "manager wait failed for {program}: {source}"))` | `lib.rs:1112-1115` |
| `Err(Error::Io { .. })`, `operation == "writing bounded stdin"` | `Err(manager(Spawn, "manager stdin failed for {program}"))` | `lib.rs:1116-1119` |
| any other `operation` | `Err(manager(Spawn, "manager spawn failed for {program}: unrecognised bounded execution operation {other}"))` — the fallback **names** the operation | `lib.rs:1123-1128` |
| `Err(Error::CandidateExecution(msg))` | `Err(manager(Protocol, "manager command failed for {program}: {msg}"))` | `lib.rs:1130-1133` |
| `Err(Error::InvalidInput(msg))` | `Err(ServiceError::invalid(msg))` | `lib.rs:1134` |
| any other `Error` variant | `Err(manager(Spawn, "manager command failed for {program}: {other}"))` | `lib.rs:1135-1138` |

## Production implementation evidence

`crates/eggup-service/src/lib.rs`, `impl CommandExecutor for SystemExecutor`
(`:1021-1141`):

| Element | `file:line` |
|---|---|
| The service's own pre-flight (six checks, in order) | `:1032`, `:1035`, `:1038`, `:1041`, `:1050`, `:1070` |
| The budget starts before the resolve | `:1049` |
| The resolved absolute path and the `program` string | `:1050-1051` |
| The remaining budget, and the zero-budget early return | `:1052-1060` |
| The `CommandSpec` construction | `:1062-1065` |
| The environment conversion (valid UTF-8 passed, non-UTF-8 dropped) | `:1070-1075` |
| The stdin payload attachment | `:1076-1078` |
| The `run_bounded` call | `:1080` |
| The mapping, with the exhaustive fallback arm | `:1081-1139` |

`read_bounded_generic` was deleted. It had exactly two callers, both in the
replaced runner body (`crates/eggup-service/src/lib.rs:1079` and `:1083`
before the change); `grep -n "read_bounded_generic" crates/eggup-service/src/`
now returns **no hits**, so the crate no longer carries two bounded readers
(R-A5).

### The six pre-flight rejections are the service's, not core's

Core's own `MAX_STDIN_BYTES` is `1024 * 1024` and the service's
`MAX_CRONTAB_BYTES` is also `1024 * 1024` (`crates/eggup-service/src/lib.rs:897`),
so the two bounds coincide. The service's check runs **first**
(`lib.rs:1041-1045`), so core's `InvalidInput("stdin data exceeds the bounded
stdin bound")` is unreachable in practice; the arm at `lib.rs:1134` exists so
that a future divergence is a typed `InvalidInput` rather than a swallowed
error (§14's stop condition 11 did not fire — both bounds are 1 MiB).

## Behaviour-equivalence result (the plan's §8 properties)

| # | Property | How it is proven |
|---|---|---|
| **P1** | The deadline is unconditional | `a_timeout_yields_the_timeout_kind` (`bounded_executor_consolidation.rs`) — the child sleeps 60 s, the budget is 300 ms, the call returns the `Timeout` kind, and the assertion allows 20 s. |
| **P2** | The deadline still bounds the writer | Core's contract (M017) states it; the observable consequence is covered by P1 plus `delivers_the_stdin_payload_to_the_child` (a payload is delivered, not truncated). |
| **P3** | A broken pipe still fails closed | The kind and the detail text are preserved by the mapping arm at `lib.rs:1116-1119`; the **ordinary success case**, which the replaced runner got wrong, is now correct — see **D8**. |
| **P4** | An output overflow still fails closed and discards the stream | `an_output_overflow_yields_the_protocol_kind_and_discards_the_stream` — the child emits 8 192 bytes against a 1 024-byte bound; the kind is `Protocol` and the detail is exactly `manager output exceeded bound for {program}`. |
| **P5** | A nonzero exit is still `Ok` with the observed status | `a_nonzero_exit_is_ok_with_the_observed_status` — the child exits 3; the result is `Ok` with `status == Some(3)`. |
| **P6** | The `crontab -` path still receives its payload | `delivers_the_stdin_payload_to_the_child` — the payload arrives byte-for-byte. **This is the one assertion the replaced runner fails** (see "Baseline evidence"). |
| **P7** | No manager's argv changes | `the_managers_argv_is_byte_identical` — the golden vectors for `systemctl` (`show`, `is-active`, `start`, `stop`, `enable`, `disable`, `daemon-reload`), `launchctl` (`list`, `bootstrap`, `bootout`, `kickstart`, `stop`), and `crontab` (`-l`, `-`) are compared with the observed argv, and the `FakeExecutor` refuses any deviation. |
| **P8** | The child environment is cleared; only the session-bus variables are injected | `the_child_environment_is_cleared_and_only_the_session_bus_variables_are_injected` (the environment is empty for a non-`--user` command) plus the crate-private `the_session_bus_filter_is_exactly_three_variables` (`lib.rs:4268`), which pins the filter to exactly `DBUS_SESSION_BUS_ADDRESS`, `SYSTEMD_BUS_ADDRESS`, `XDG_RUNTIME_DIR` on every lane. |
| **P9** | The allowlist is the only path to a binary; no `PATH` lookup | `resolve_manager_program` and `validate_executable_path` are unmodified (`lib.rs:1143`, `:1171`); the **resolved** path is what reaches core (`lib.rs:1062`). `the_pre_flight_rejections_are_service_errors_not_core_errors` proves no child is started by a rejected call. |
| **P10** | The pre-flight rejections are unchanged | `rejects_a_zero_timeout_an_over_range_timeout_and_an_over_bound_stdin_payload` — all four `ServiceError::invalid` messages are asserted exactly. |

## The eight differences

| # | Difference | Direction | Pinned by |
|---|---|---|---|
| **D1** | A mid-stream reader error is now fail-closed (`Protocol`, "manager output exceeded bound for {program}") instead of a silently truncated success. | Fail-closed (stronger) | The mapping arm `lib.rs:1088-1093`. No portable fixture exists for a mid-stream reader error. |
| **D2** | Core kills and reaps a still-running child on an output overflow; the replaced runner did not (it only noticed the overflow after the child had exited). | Strictly safer | `an_output_overflow_yields_the_protocol_kind_and_discards_the_stream`. |
| **D3** | The timeout detail text is normalised to `"manager command timed out for {program}"`; the replaced runner could also produce `"manager output collection timed out"` and `"manager command timed out"`. | Detail text only — `manager_kind()` is unchanged | `a_timeout_yields_the_timeout_kind`. |
| **D4** | A descendant holding the stdout/stderr pipe open now ends at the deadline with the *timeout* detail instead of `"manager output collection timed out"`. | Same kind, different detail text (D3 covers the text) | No portable fixture; recorded here. |
| **D5** | A non-UTF-8 environment pair is dropped rather than passed through byte-exactly, because core's setter takes `impl Into<String>`. | Narrowing, fail-closed | The conversion at `lib.rs:1070-1075`; the closure records it as **UF-2** (low). |
| **D6** | The spawn-failure `NotFound` detail names the **resolved** program; the pre-flight `NotFound` names the **requested** one. | No change | `classifies_a_missing_program_as_not_found_with_the_current_detail` and `classifies_a_spawn_failure_as_a_typed_manager_failure` assert the two paths separately. |
| **D7** | A stdin writer still blocked at the deadline is reported as a **timeout**, not as a stdin failure. | Kind change, in a deep corner | **Recorded, not tested — no portable fixture.** It needs a descendant holding the child's stdin pipe open after the child exits. Its two ordinary cases are covered: the broken pipe is `Spawn` (mapping arm `lib.rs:1116-1119`) and the still-running child at the deadline is `Timeout` (`a_timeout_yields_the_timeout_kind`). |
| **D8** | **A successful stdin write is no longer reported as a stdin failure.** The replaced runner's `try_recv()` consumed and discarded the writer's `Ok(())` verdict, so the post-exit check found an empty, disconnected channel and reported `Manager(Spawn, "manager stdin failed for {program}")` for a write that had succeeded. | **Defect fixed** (a failure becomes a success) | `delivers_the_stdin_payload_to_the_child` — it **fails on the replaced runner** and passes on the consolidated one. Baseline and published-crate evidence below. |

D8 is a difference beyond the plan's enumerated set. It is **not** a
regression: it is the consolidation correcting a defect of the replaced
runner, in the fail-safe direction. The plan's §8.1 P3 premise — that the
broken pipe and the still-running child are *the* two ordinary cases and that
both implementations agree on them — is factually wrong, and the plan's
implementation note records the correction. The plan's §14 stop condition 5
asks the implementer to stop on a difference that indicates a regression; this
one is the opposite, the tree is green, and the finding is recorded rather
than suppressed.

## Baseline evidence

The mechanism: `git archive <sha> | tar -x -C <scratch>` — **read-only**; no
worktree, no index, no branch, and no other git state was touched. The new
fixture files were copied into the scratch tree and run against the **old**
implementation. The scratch directories were `/tmp/eggup-m015-baseline`
(baseline `20ac770`), `/tmp/eggup-m015-baseline-target` (its build directory),
`/tmp/eggup-m015-published` (the published `0.1.3` source `feb6ae5`), and
`/tmp/eggup-m015-published-target`.

### Baseline `20ac770` — the pre-change implementation

```sh
cd /tmp/eggup-m015-baseline
CARGO_TARGET_DIR=/tmp/eggup-m015-baseline-target \
  cargo test -p eggup-service --locked --offline \
    --test bounded_executor_consolidation --test bounded_executor_environment
```

Observed:

```text
test eggup_fixture_child ... ok
test rejects_a_zero_timeout_an_over_range_timeout_and_an_over_bound_stdin_payload ... ok
test classifies_a_missing_program_as_not_found_with_the_current_detail ... ok
test the_pre_flight_rejections_are_service_errors_not_core_errors ... ok
test classifies_a_spawn_failure_as_a_typed_manager_failure ... ok
test an_output_overflow_yields_the_protocol_kind_and_discards_the_stream ... ok
test a_nonzero_exit_is_ok_with_the_observed_status ... ok
test runs_a_command_and_returns_its_status_and_bounded_output ... ok
test passes_the_argv_literally_without_a_shell ... ok
test delivers_the_stdin_payload_to_the_child ... FAILED
test the_managers_argv_is_byte_identical ... ok
test a_timeout_yields_the_timeout_kind ... ok

thread 'delivers_the_stdin_payload_to_the_child' panicked at
  crates/eggup-service/tests/bounded_executor_consolidation.rs:195:10:
the fixture child runs: Manager { kind: Spawn, detail: "manager stdin failed for
  /tmp/eggup-m015-baseline-target/debug/deps/bounded_executor_consolidation-1cd9b9fbecb492a7",
  os_code: None }

test result: FAILED. 11 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
```

and, separately,

```sh
cd /tmp/eggup-m015-baseline
CARGO_TARGET_DIR=/tmp/eggup-m015-baseline-target \
  cargo test -p eggup-service --locked --offline --test bounded_executor_environment
# test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

**Reading.** Eleven of the twelve consolidation fixtures and both environment
fixtures **pass against the replaced implementation**. That is the point of an
equivalence milestone: the same assertions hold before and after, which is the
evidence that the consolidation preserved the behaviour. The single failure is
the stdin-delivery fixture — the difference **D8**.

### Minimal reproduction of D8

A two-line probe, run against the replaced runner and against the
consolidated one, with the smallest possible child:

| Implementation | Command | Observed |
|---|---|---|
| Replaced runner (baseline `20ac770` **and** published `feb6ae5`) | `SystemExecutor::new().run(&["/bin/cat".into()], Some(b"hello\n"), 5 s)` | `Err(Manager { kind: Spawn, detail: "manager stdin failed for /bin/cat" })` |
| Consolidated runner (this milestone) | the same call | `Ok(status = Some(0), stdout = [104, 101, 108, 108, 111, 10])` |

The published-crate probe was run in `/tmp/eggup-m015-published`, extracted
from `git archive feb6ae5aea4c9b61c4051957f3662ca49d845f9e` — the exact
source revision the published `eggup-service 0.1.3` carries (per
`crates/eggup-service/CHANGELOG.md` § "0.1.3"). The defect is present there:
`crates/eggup-service/src/lib.rs:801` (`rx.try_recv()`) and `:819`
(`rx.recv_timeout(remaining)`) in that revision.

The mechanism, stated exactly: `std::sync::mpsc::Receiver::try_recv`
**removes** the message it returns. The replaced runner's wait loop called
`try_recv()` on every iteration and matched only `Ok(Err(_))`, so a verdict of
`Ok(())` was consumed and discarded; the post-exit check then received
`Err(Disconnected)` from the now-empty channel and reported a stdin failure.
The probe's instrumented run showed exactly that: the writer's own verdict was
`Ok(())` while the receiving side observed `Err(Disconnected)`.

### Structural baseline

`grep -c "std::process::Command" crates/eggup-service/src/lib.rs` → `0`
after the change; the replaced implementation had 1 hit (its own
`std::process::Command::new`). `grep -rn "read_bounded_generic\|Stdio::piped\|thread::spawn"`
→ no hits after the change; the replaced implementation had the bounded
reader, two reader threads, and one writer thread.

## Exact commands run and results

All commands were run from `/Users/davidbowman/projects/eggup` on the
**macOS** lane with `--offline`.

| # | Command | Result |
|---|---|---|
| 1 | `cargo fmt -p eggup-service -- --check` | exit 0, no diff |
| 2 | `cargo clippy -p eggup-service --all-targets --all-features --locked --offline -- -D warnings` | exit 0, `Finished \`dev\` profile` |
| 2b | `cargo clippy -p eggup-service --all-targets --locked --offline -- -D warnings` | exit 0 |
| 3 | `cargo test -p eggup-service --all-targets --all-features --locked --offline` | exit 0 — `112 passed` (lib) + `1` + `12` + `2` + `3` + `4` + `8` + `5` + `2` + `2` + `14` + `0` + `5` + `1` + `0` + `2` (doc-tests); **0 failed** everywhere |
| 3b | `cargo test -p eggup-service --locked --offline` | exit 0, identical result set |
| 4 | `cargo doc -p eggup-service --no-deps --locked --offline` | exit 0, `Generated …/target/doc/eggup_service/index.html` |
| 5 | `cargo check --target x86_64-pc-windows-msvc --all-targets --locked --offline -p eggup-service` | exit 0, **no warnings** |
| 5b | `cargo clippy --target x86_64-pc-windows-msvc --all-targets --locked --offline -p eggup-service -- -D warnings` | exit 0, no warnings |
| 6 | `cargo tree -p eggup-service --locked --offline --depth 1` | `eggup-service v0.1.3` → `eggup-core v0.1.3`, `windows-args v0.2.0` — **unchanged**, no new dependency |
| 7 | `git status --short -- crates/eggup-service/Cargo.toml Cargo.lock` | no output — **both byte-identical** |
| 8 | `git status --short -- crates/eggup-service/tests/public-api-surface.txt` | no output — the snapshot is **unmodified** |
| 9 | `grep -c "std::process::Command" crates/eggup-service/src/lib.rs` | `0` |
| 10 | `grep -rn "read_bounded_generic\|Stdio::piped\|thread::spawn" crates/eggup-service/src/lib.rs` | no hits |
| 11 | `./scripts/check-docs-truth.sh` | `documentation-truth checks passed` |

The **workspace-wide** gate (`./scripts/check-local.sh`) is the **parent's**
step and was not run here: it covers crates this milestone does not own, and
other agents were editing the tree concurrently.

### Per-platform results — reported individually, never inferred

| Platform | What was run | Result |
|---|---|---|
| **macOS** (this machine: darwin 25.6.0 arm64; Rust 1.89) | commands 1–11 above, including the whole `eggup-service` suite | **All green. 0 failed.** The twelve `bounded_executor_consolidation` fixtures, the two `bounded_executor_environment` fixtures, the 112 lib tests, the M014 public-API guard, and the `manager_failure_kinds` taxonomy all pass. |
| **Linux** | nothing | **Not run.** No Linux host was available in this session. No Linux result is inferred from the macOS result. |
| **Windows** | `cargo check --target x86_64-pc-windows-msvc --all-targets --locked --offline -p eggup-service` and the matching `clippy -D warnings` | **Compile-only, and warning-free.** The Windows **suite was not executed** — no Windows host was available. The hosted `windows-check` lane (which runs `cargo test --workspace --all-targets --locked --no-fail-fast` and therefore the whole `eggup-service` suite, including both new fixture targets) is the **parent's** qualification step, and this record does **not** predict its result. |

### Commands left to the parent session at this stage

- `./scripts/check-local.sh` (the workspace-wide gate).
- The hosted lanes, in particular `windows-check`, which executes the Windows
  lane's workspace suite — and therefore the new fixture targets
  `bounded_executor_consolidation` and `bounded_executor_environment` — on a
  real Windows host.
- The commits. The working tree is uncommitted by design.

## Invariant review

- **The allowlist is still the only path to a binary.** `resolve_manager_program`
  (`lib.rs:1143`) and `validate_executable_path` (`lib.rs:1171`) are
  byte-for-byte unmodified; core receives the **resolved** path
  (`lib.rs:1062`) and never a program name.
- **The child environment is still cleared**, and the only injected variables
  are the three session-bus names for `systemctl --user`
  (`lib.rs:1070`, `lib.rs:1198-1218`).
- **The stdin bound is still the service's own** (`lib.rs:1041-1045`,
  `MAX_CRONTAB_BYTES` = 1 MiB) and still runs before core sees anything.
- **The timeout range check is unchanged** (`lib.rs:1038-1040`).
- **The argv is still literal.** No shell, no stringification, no `PATH`
  mediation — proven by `passes_the_argv_literally_without_a_shell`.
- **The failure taxonomy is unchanged.** `ServiceError`'s four variants and
  `ManagerFailureKind`'s six variants are untouched; `manager_kind()` keeps
  its meaning.
- **No manager's argv changed**, and **no existing `fake.expect(...)` argv
  vector was edited** — the `git diff` of `crates/eggup-service/src/lib.rs`
  shows the adapter test vectors untouched.

## Failure / rollback / recovery review

- **The deadline stays unconditional**, and the service's own budget still
  bounds the **whole** call: the budget starts before the resolve
  (`lib.rs:1049`) and core receives the remainder (`lib.rs:1052-1054`). A zero
  remainder returns the timeout failure **without spawning anything**
  (`lib.rs:1055-1060`).
- **Zombie discipline is preserved.** Core kills **and reaps** the child on
  every failure path, and the service adds no path of its own.
- **Cancellation** is unchanged: the mechanism is the deadline, and a caller
  that aborts a manager operation still observes `ManagerFailureKind::Timeout`.
- **Restart** is unchanged: `restart` is `stop` + `start`, each under its own
  `OperationDeadline`.
- **Contention** is unchanged: no lock, no change in the number of spawned
  children, and a concurrent caller's failure is still a typed `Manager` error.
- **No rollback or recovery mechanism is touched.** This crate has no
  crash-journal and claims none.

## Compatibility and migration review

- **The public API of `eggup-service` is unchanged.** `CommandExecutor`
  (`lib.rs:961`), `SystemExecutor` (`lib.rs:989`) and its **public** field
  `max_output_bytes` (`lib.rs:991`), `SystemExecutor::new` (`lib.rs:1004`),
  `SystemExecutor::with_max_output` (`lib.rs:1013`), `ManagerCommandOutput`
  (`lib.rs:921`), `CommandOutput` (`lib.rs:953`), `ManagerFailureKind`
  (`lib.rs:339`), `ServiceError` (`lib.rs:406`) and its constructors — all
  keep their exact shapes. The M014 public-API guard passes with its
  snapshot **unmodified**.
- **A consumer observes no difference in kind.** `NotFound`, `Permission`,
  `Timeout`, `Spawn`, and `Protocol` keep their meanings; the only
  difference a consumer can observe is the **detail text** of a timeout
  (D3) and of an output overflow (unchanged), which §9 of the plan already
  places outside the contract: the **kind** is the programmatic discriminator.
- **The published `eggup-service 0.1.3` keeps the hand-rolled runner.** The
  consolidation lands in the workspace source only. The published crate was
  published 2026-10-09 from source `feb6ae5` with checksum
  `9f7f7ea8…32124dc`, and **no republication was performed**: no
  `cargo publish`, no tag movement, no version bump. §8's differences — and
  the **D8** defect fix — therefore reach consumers only at republication.
- **No dependency change, so no downstream pin cascade.** `eggup-service`
  already depended on `eggup-core` by path; `eggup-eggpack`'s exact pins are
  untouched.
- **No migration step is required of any consumer.**

## Security review

- **No new privilege, no new filesystem authority, no new network surface.**
- The child is still spawned with a **cleared** environment and a **literal**
  argv; core's `Command::new` + `args` adds no shell.
- The service's pre-flight still runs **before** anything is spawned, so a
  rejected request creates no process at all — proven by
  `the_pre_flight_rejections_are_service_errors_not_core_errors`, which uses a
  fixture child that would create a marker file and asserts the marker does
  not exist.
- The stdin payload is still bounded at 1 MiB **before** the spawn.
- The output bound is still per-stream and still discards an over-bound
  stream rather than returning a truncated buffer.
- **D5** is the only narrowing with a security flavour: a non-UTF-8
  environment pair is dropped rather than lossily converted, so a mangled
  session-bus address can never reach the child. Fail-closed.
- The fixture child is the crate's **own** test binary. No network access, no
  root, no real service manager: the fixtures are the crate's own binaries.

## Documentation / operations evidence

| Document | What changed |
|---|---|
| `crates/eggup-service/README.md` | The M002 bullet's "shared bounded command runner" entry now states that the bounded execution is delegated to `eggup_core::run_bounded`, and that the crate's own pre-flight — the allowlisted absolute program path, the cleared environment, the bounded stdin payload, and the timeout range — stays in front of that call and is unchanged. |
| `crates/eggup-service/CHANGELOG.md` | One entry added at the top of the existing `## Unreleased` section: the delegation, the preserved observable outcomes, the eight differences, the unchanged public API, "no new dependency, no manifest change, no lockfile change", and the **unpublished** disclaimer with "no consumer migration is required". **No published version section was touched.** |
| `architecture/service-lifecycle.md` | § "Process execution: `SystemExecutor`" (§ line 507): the intro now describes the thin policy layer over `eggup_core::run_bounded` and names the three core operation strings with their `file:line`; "Executable allowlisting and path resolution", "Environment clearing", "Output and timeout bounds", "The stdin deadlock fix" and "No shell interpolation" were **re-anchored** to the new code; a new subsection "Behaviour differences of the consolidation" records **D1–D8** with their directions; the `###` `FakeExecutor` subsection was re-anchored; the crate's **test-distribution table** now lists the six Unix-only runner tests (gated per test), the new crate-private session-bus assertion, and the **two new integration targets**. |
| `plans/implementation/service-lifecycle/015-bounded-executor-consolidation-corrective.md` | `Status:` changed from `ready for handoff` to `implemented`, and an **implementation note** was added recording the three deviations (the fixture without a manifest change, the Unix-only `--user` case, and the D8 correction to §8's P3 premise). The plan's structure and decisions are otherwise unchanged. |
| `plans/registry.md`, `plans/subsystems/service-lifecycle-roadmap.md`, root `CHANGELOG.md` | **Not edited by this milestone.** The parent owns them. The intended prose is drafted in "Registry updates" and "Roadmap disposition" below. |

## Plan deviations

Two sentences of the plan's §10 could not be satisfied literally. Both were
resolved in the direction the plan's own hard constraints point, and both are
recorded in the plan's implementation note.

1. **The portable child fixture does not use a `[[test]]` target with
   `harness = false`.** §10 prescribes one, but a `[[test]]` entry is a
   `crates/eggup-service/Cargo.toml` edit, and **R-C4** and **AC9** require
   that manifest — and `Cargo.lock` — to be **byte-identical** after the
   milestone; §5's "no manifest change" and §14's stop condition 9 say the
   same. The conflict is resolved in favour of the manifest constraint: the
   fixture child is **the test binary itself**, re-executed with a mode
   argument (`crates/eggup-service/tests/common/fixture_child.rs`). That
   keeps the fixture portable — `current_exe()` is an absolute path to a real
   executable on every platform — and needs no shell, no interpreter, and no
   manifest change. The mode is selected from **argv**, never from the
   environment, because the environment is the property under test. The
   payload is framed on stdout between two NUL-delimited markers, so it is
   recovered byte-exactly regardless of what the test harness writes around
   it.
2. **§10's T9 "`systemctl --user`" case is Unix-only.** The service's
   environment filter selects the user-scoped case by the program's **file
   name**, so the child must be named `systemctl`; only Unix can execute a
   program whose name has no extension. The case is `#[cfg(unix)]`-gated, and
   its **portable counterpart** is the crate-private assertion
   `the_session_bus_filter_is_exactly_three_variables`
   (`crates/eggup-service/src/lib.rs:4268`), which pins the same filter to
   exactly the three session-bus names on every lane.

A third deviation is a **difference**, not a test: **D8**, above.

## Unresolved findings with severity

### UF-1 — The published `eggup-service 0.1.3` reports a successful stdin write as a stdin failure — **medium — OPEN (published exposure)**

**Finding.** The published crate's bounded runner (`0.1.3`, source
`feb6ae5`, `crates/eggup-service/src/lib.rs:801` and `:819` in that
revision) polls its stdin-writer channel with `try_recv()`, which **removes**
the message it returns. The verdict `Ok(())` is therefore consumed and
discarded by a check that only looks for `Err(_)`, and the post-exit check
receives `Err(Disconnected)` and reports
`Manager(Spawn, "manager stdin failed for {program}")` for a write that
**succeeded**.

**Evidence.** The minimal reproduction is a six-byte payload to `/bin/cat`:

- published `feb6ae5`: `Err(Manager { kind: Spawn, detail: "manager stdin failed for /bin/cat" })`;
- this milestone's consolidated runner: `Ok(status = Some(0), stdout = "hello\n")`.

**Impact.** The crate's only production stdin caller is the `crontab -`
install (`crates/eggup-service/src/lib.rs:3455-3459`). A
consumer that installs a cron block through the published crate receives a
typed `Spawn` failure instead of a successful install. The failure is
**fail-closed** — nothing is corrupted, no state is misreported, no privilege
is escalated — and the published crate has **no consumer yet** (the Eggup
product itself is not released), so the exposure is "the next Eggup release
would ship a broken cron-install path", not "consumers are broken today".

**Severity: medium.** It is not high: it fails closed, it is not a safety or
privilege issue, and the affected crate has no consumer. It is not low: it
is a **functional defect in a published crate's only stdin path**, it is
**reproducible with a two-line probe**, and it is invisible to the current
test suite because the adapter tests drive `FakeExecutor` rather than a real
child.

**Disposition.** The **source-line** defect is **closed by this milestone** —
the consolidated runner delivers the payload, and
`delivers_the_stdin_payload_to_the_child` pins it (it fails on the replaced
runner). The **published** defect is **open** and requires the republication
milestone, which must carry the fix and state it in its notes. Under the
repository's rule that *"medium-or-higher is the de-facto blocking-severity
gate: any such issue still open forces a new corrective before the milestone
closes"*, this is the parent's call: either the republication milestone is
confirmed as the corrective, or the residual is judged low. It is flagged
here explicitly rather than buried.

### UF-2 — A non-UTF-8 environment pair is dropped rather than passed through byte-exactly — **low — accepted**

**Finding.** `filtered_manager_environment` yields `(OsString, OsString)`
pairs; core's `CommandSpec::environment` takes `impl Into<String>`. A pair
that is not valid UTF-8 is **dropped** (`lib.rs:1071-1073`) rather than
lossily converted. The replaced runner passed the bytes through verbatim.

**Severity: low.** The three allowlisted names carry path- and
address-shaped values that are valid UTF-8 in practice; the dropped case is a
corner; and the direction is fail-closed — a mangled session-bus address is
never substituted, and the child ends up in the same state every non-`--user`
command is in. This is the plan's **D5**, and the plan requires it to be
recorded as a residual finding. **Accepted**, not deferred to a corrective.

### UF-3 — The six Unix-shaped executor tests were previously ungated — **low — closed here**

**Finding.** The six `SystemExecutor` tests in
`crates/eggup-service/src/lib.rs` use POSIX absolute paths (`/bin/sleep`,
`/bin/echo`, `/bin/sh`, `/usr/bin/env`). On Windows
`Path::new("/bin/sleep").is_absolute()` is **false** (Rust's Windows
`is_absolute` requires a prefix *and* a root — confirmed against the standard
library's `sys/path/windows.rs`), so `resolve_manager_program` rejects them
with `InvalidInput` and the assertions on the *manager* failure kind cannot
hold. They were not gated.

**Severity: low.** **Closed by this milestone**: each of the six is now
`#[cfg(unix)]`-gated **per test** (not per module), so the Windows lane
shrinks and the Linux and macOS lanes keep every test. Each has a **portable
counterpart** in the new integration target: the timeout, the output bound,
the nonzero exit, the missing binary, the cleared environment, and the
literal argv each have one.

**Note for the parent.** This milestone did **not** establish whether the
hosted Windows lane was red on these six tests before this change; the
Windows suite was not run in this session, and no Windows result is inferred
here. What is established is that the six are now gated, so they cannot fail
on Windows regardless of the previous state.

### Adjacent findings (recorded, deliberately not fixed here)

- **Pre-existing anchor drift in `architecture/service-lifecycle.md`.**
  Several `lib.rs:NNNN` references **outside** the § "Process execution:
  `SystemExecutor`" section were already stale before this milestone — for
  example the "Definition writing" section's `atomic_write_definition`
  anchors said `lib.rs:1087-1231` while the declaration was at `lib.rs:1489`
  before this change. This milestone re-anchored the section the plan named
  (§12) and left the rest alone, because re-anchoring the whole document is
  a separate piece of work. **Severity: low — documentation only, no
  behaviour.** The parent may route it.
- **`crates/eggup-service/tests/published-api-0.1.2/` and
  `published-api-0.1.3/`** are the published-API fixtures; they pass
  unchanged and were not edited.

### UF-4 — The packaged-crate verification against the registry is deferred until the `eggup-core` republication — **medium — OPEN (publication state)**

**Finding.** `cargo package -p eggup-service --locked` resolves `eggup-core`
from **crates.io**, not from the workspace, so it compiles the packaged crate
against the **published** `eggup-core 0.1.3`. This milestone's consolidated
runner calls `CommandSpec::stdin_data` (Core M017) and relies on core's
unconditional execution deadline (Core M014). Neither is in the published
`0.1.3`, so the step fails with
`error[E0599]: no method named 'stdin_data' found for struct 'CommandSpec'`.
The hosted run `40061134915` observed exactly that failure in the `stable`,
`msrv`, `macos`, and `windows-check` lanes.

**This is a publication state, not a defect in this source line.** It is the
same condition this branch already created for `eggup-curl`, `eggup-eggfetch`,
and `eggup-eggpack`, whose sources also use capabilities the registry does not
carry (`__adapter_io` from Acquisition M013). The service package step was
simply the last one that still resolved against the registry, because the
service crate had no unpublished dependency of its own until this milestone.

**Disposition.** The parent changed the four CI steps to
`cargo package -p eggup-service --locked --no-verify` with an inline comment
recording exactly why, so the deferral is visible in the workflow itself and
not silently dropped. The step still packages the crate and validates its
manifest and file list; the compile of the packaged sources is covered by the
workspace build, test, and clippy steps, which resolve `eggup-core` to the
workspace path dependency. The verification becomes meaningful again when the
`eggup-core` republication lands.

**Severity: medium.** It is not high: nothing is miscompiled or mispublished,
the crate stays published at `0.1.3`, and the source line itself is fully
qualified. It is not low: a real CI gate was withdrawn, and the withdrawal
must not outlive its cause. **Follow-on: the `eggup-core` republication
milestone**, after which the strict step is restored. The republication is a
tracked milestone that the parent session owns and that CI never performs.

## Roadmap disposition

**The parent owns `plans/subsystems/service-lifecycle-roadmap.md`.** The
intended row for the milestone-status table (§ 12 of the roadmap), drafted
from the plan's §16.3 and **not applied** here:

> | M015 | closed \<date\> | `plans/implementation/service-lifecycle/015-bounded-executor-consolidation-corrective.md` | `plans/closure/service-lifecycle/015-status.md` | Closes M012 closure Finding 1 (**low**). `SystemExecutor` delegates to `eggup_core::run_bounded`: one bounded-process-runner implementation in the workspace. The service's pre-flight policy (allowlisted absolute paths, cleared environment, bounded stdin, timeout range), the `ManagerFailureKind` taxonomy, the public API, and every manager's argv are unchanged; eight documented behaviour differences are recorded in the closure, one of which (**D8**) is a defect in the replaced runner that the consolidation fixes. No new dependency, no republication, `v0.1.3` must not move |

The roadmap's header status line advances from *"M001–M011 closed"* to
*"M001–M015 closed"*, and the registry's "Active subsystem roadmaps" row for
**Service lifecycle** becomes *"no further service-lifecycle milestone is
registered"* — unless the parent registers another, for example the
republication milestone that **UF-1** needs.

## Registry updates

**The parent owns `plans/registry.md`.** The intended row for §
"Dependency-ready implementation work", drafted from the plan's §16.2 and
**not applied** here:

> | Service lifecycle | M015 bounded executor consolidation corrective | closed \<date\> | `plans/implementation/service-lifecycle/015-bounded-executor-consolidation-corrective.md`; `plans/closure/service-lifecycle/015-status.md` | Closes M012 closure **Finding 1** (**low**): `SystemExecutor` now delegates to `eggup_core::run_bounded`, so the workspace holds one bounded-process-runner implementation instead of two. The service's own pre-flight is unchanged — allowlisted absolute paths, cleared child environment, bounded stdin, timeout range — and the `ManagerFailureKind` taxonomy, the public API, and every manager's argv are unchanged. Eight behaviour differences are recorded in the closure: a reader error is now fail-closed, core kills a still-running child on output overflow, the timeout detail text is normalised, the descendant-holds-pipe case ends at the deadline with the timeout detail, a non-UTF-8 environment pair is dropped rather than passed through, the spawn-failure `NotFound` detail names the resolved path, a stdin writer still blocked at the deadline is reported as a timeout rather than a stdin failure, and a successful stdin write is no longer misreported as a stdin failure. No new dependency (`eggup-core` was already a path dependency), no republication, `v0.1.3` unmoved |

And the matching update to the "Post-0.1.3 open findings" list, if the parent
records the discharge there:

> The M012 closure's **Finding 1** (`SystemExecutor` is not consolidated onto `eggup_core::run_bounded`, **low**) is **closed** by Service M015: `plans/closure/service-lifecycle/015-status.md`. The duplication was plan-deferred pending core M014's deadline corrective and core's bounded-stdin capability; both are in place.

The parent should also consider adding a new entry to that list for **UF-1**
(the published crate's stdin-verdict defect, **medium**), because it is a
finding about a **published** crate and not about the source line.

**The parent also owns the root `CHANGELOG.md`.** The intended bullet for its
`Unreleased` section, drafted from the plan's §16.4 and **not applied**
here:

> - `eggup-service`: `SystemExecutor` now runs manager commands through `eggup_core::run_bounded`, so the workspace holds one bounded-process-runner implementation. The crate's own safety policy — allowlisted absolute program paths, a cleared child environment with only the explicit user-scoped systemd session-bus variables, bounded stdin, and the timeout range check — is unchanged, as are the `ManagerFailureKind` taxonomy, the public API, and every manager's argv. **Unpublished**: the published `eggup-service 0.1.3` keeps the previous runner until a republication. No consumer migration is required.

## Files touched by this milestone

| File | Change |
|---|---|
| `crates/eggup-service/src/lib.rs` | The consolidation (`:1021-1141`), the `read_bounded_generic` deletion, the `SystemExecutor` rustdoc, six `#[cfg(unix)]` gates on the Unix-shaped executor tests, and the new crate-private `the_session_bus_filter_is_exactly_three_variables`. |
| `crates/eggup-service/tests/common/fixture_child.rs` | **New.** The portable child fixture. |
| `crates/eggup-service/tests/bounded_executor_consolidation.rs` | **New.** Twelve tests. |
| `crates/eggup-service/tests/bounded_executor_environment.rs` | **New.** Two tests. |
| `crates/eggup-service/README.md` | The process-execution bullet. |
| `crates/eggup-service/CHANGELOG.md` | One `## Unreleased` entry. |
| `architecture/service-lifecycle.md` | § "Process execution: `SystemExecutor`" re-anchored, D1–D8 recorded, the test-distribution tables updated. |
| `plans/implementation/service-lifecycle/015-bounded-executor-consolidation-corrective.md` | `Status:` and the implementation note. |
| `plans/closure/service-lifecycle/015-status.md` | **New.** This record. |

### Ownership verification

- `crates/eggup-core/**` — **not modified.** Its working-tree changes are
  Core **M017**'s and pre-date this milestone.
- `crates/eggup-acquisition/**`, `crates/eggup-curl/**`,
  `crates/eggup-eggfetch/**`, `crates/eggup-archive/**`,
  `crates/eggup-eggpack/**`, `crates/eggup-transport-footprint/**` — **not
  modified.**
- `plans/registry.md`, `plans/subsystems/**`, root `CHANGELOG.md` — **not
  modified**; the parent owns them.
- `plans/closure/**` other than this record — **not modified.**
- `architecture/**` other than `architecture/service-lifecycle.md` — **not
  modified.**
- `scripts/**`, `.github/**`, `AGENTS.md`, `docs/**` — **not modified.**
- **No git state was mutated**: no `stash`, `checkout`, `restore`, `commit`,
  `push`, `tag`, `clean`, `reset`, or branch creation. The baseline evidence
  used `git archive` into a scratch directory.

## Test inventory added by this milestone

`crates/eggup-service/tests/bounded_executor_consolidation.rs` — 12 tests:

| Test | Proves |
|---|---|
| `eggup_fixture_child` | The fixture child's entry point (a no-op in an ordinary run). |
| `runs_a_command_and_returns_its_status_and_bounded_output` | **T1** — status and bounded output. |
| `delivers_the_stdin_payload_to_the_child` | **T2** — the `crontab -` payload, byte-for-byte. |
| `passes_the_argv_literally_without_a_shell` | The literal-argv property, portably. |
| `rejects_a_zero_timeout_an_over_range_timeout_and_an_over_bound_stdin_payload` | **T3** — the four pre-flight messages. |
| `classifies_a_missing_program_as_not_found_with_the_current_detail` | **T4(a)** — the pre-flight `NotFound`, naming the requested program. |
| `classifies_a_spawn_failure_as_a_typed_manager_failure` | **T4(b)** — the spawn arm, per platform. |
| `a_timeout_yields_the_timeout_kind` | **T5** — the timeout kind. |
| `an_output_overflow_yields_the_protocol_kind_and_discards_the_stream` | **T6** — the overflow kind and detail. |
| `a_nonzero_exit_is_ok_with_the_observed_status` | **T7** — `Ok` with the observed status. |
| `the_managers_argv_is_byte_identical` | **T8** — the golden argv for every manager. |
| `the_pre_flight_rejections_are_service_errors_not_core_errors` | **T10** — `InvalidInput`, and no child started. |

`crates/eggup-service/tests/bounded_executor_environment.rs` — 2 tests:

| Test | Proves |
|---|---|
| `eggup_fixture_child` | The fixture child's entry point. |
| `the_child_environment_is_cleared_and_only_the_session_bus_variables_are_injected` | **T9** — the cleared environment, the three injected variables, and the absent sentinel. |

`crates/eggup-service/src/lib.rs`'s `mod tests` — 1 added test:

| Test | Proves |
|---|---|
| `the_session_bus_filter_is_exactly_three_variables` | The portable counterpart of T9's Unix-only `--user` case. |

**Count: 15 new tests** (13 in the two new integration targets, 1 in the
fixture module's shared helper — which contributes the `eggup_fixture_child`
entry point to each target — and 1 crate-private).
