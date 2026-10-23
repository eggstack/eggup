# Closure record — acquisition-transport M012

## Status

**Conditionally closed.**

The two scheduler-sensitive wall-clock assertions are replaced by
deterministic ones, the sub-second serialization coverage is retained
explicitly at the layer where it can be deterministic, and the discriminating
**loaded** re-run shows **0 failures** where the pre-change tree fails. Every
new assertion is proven to fire against a mutated control.

The condition is the same one the sibling acquisition M013 closure carries: the
**Linux** and **Windows** lanes and the hosted CI matrix were **not run** at
this stage, and the parent session owns the whole-tree gate. Everything not
run is recorded as **not run**, never inferred from the macOS lane.

One finding is recorded as **open** with an explicit severity, and one
truthfulness gap is recorded for the owner of a file this milestone does not
own:

- **UF-1 (low)** — the pre-change **test B** absolute ceiling
  (`elapsed < 750 ms`) was **never observed to fail on this host**: 120
  iterations at loadavg up to 228 measured a maximum elapsed of **282.8 ms**,
  i.e. **0 of 120** samples over the 750 ms bound. The §6.3 change is still
  required by the plan (it removes the absolute constant and adds the missing
  lower bound), but this record does **not** claim a pre-change test-B failure.
  See *Baseline-failure proof*.
- **UF-3 (low)** — `architecture/curl-adapter.md` still carries the pre-M012
  test table (`argv actually carries 0.25 / 0.75`, `< 750 ms elapsed`) and
  stale line anchors. That file is owned by acquisition M013 under this
  session's file-ownership split, so it was **recorded, not edited**.

No publication was performed, no tag was moved, and no published version was
bumped.

## Source plan and roadmap

- Plan: `plans/implementation/acquisition-transport/012-sub-second-deadline-test-determinism-corrective.md`
- Roadmap: `plans/subsystems/acquisition-transport-roadmap.md`
- Finding of record: `plans/registry.md` § "Post-0.1.3 open findings" —
  item **1** (the two sub-second wall-clock deadline tests).
- Precedent shape followed: `plans/implementation/service-lifecycle/008-operation-deadline-test-determinism-corrective.md`
  — in particular its discipline that an **unloaded** rerun may not close the
  finding.

The plan file's `Status:` header was updated from `ready for handoff` to
`conditionally closed` and now points at this record.

## Reviewed repository baseline

- `HEAD` = `c4805411f74b85158e0ae5d0a219a32c4209d98e`
  (`docs(service): reconcile wg-basic M004 closure`). The tree carried
  **uncommitted work from earlier closed milestones** (core M014/M015, archive
  M005, service M012/M013, eggpack M006, and acquisition **M013**); those
  changes were treated as correct and were **not** reverted or modified.
- Acquisition M013 had already changed `crates/eggup-curl/src/lib.rs` in this
  same tree: the nine vacuously-passing curl test sites now **fail loudly**
  through `require_curl_executable` / `require_transport`
  (`crates/eggup-curl/src/lib.rs:915-933`), the acquisition seam's I/O surface
  is the typed `Io { operation, source }` form, and the adapter's local
  `bound` copy is gone. M012 **kept** the fail-loudly fixture behaviour
  byte-for-byte and changed only the assertions M012 owns.
- The plan's cited line numbers (`lib.rs:1367`, `:1393`) are **stale** after
  M013; every site was located by test name and by content.
- The pre-M012 baseline was materialised with `git worktree add` at a
  **unique** path (`/tmp/eggup-acq-m012-base-14577`, detached at `c480541`),
  **never** by mutating the main working tree's git state. No `git stash`,
  `git checkout <path>`, `git restore`, `git commit`, `git push`, `git tag`,
  `git clean`, or `git reset` was run. No branch was created or switched.

## Implementation commits / PRs

**None.** No commit was created, no branch was created or switched, and no tag
was moved. The work exists as uncommitted changes in the working tree, in the
files listed under *Files touched* below, on top of the baseline above.

## Executive finding

The two tests were asserting the host's scheduler, not the transport. Both are
now deterministic, and **no production code changed**.

- **§6.1 — test A** (`build_curl_args_passes_sub_second_deadlines_to_fake_curl`,
  `crates/eggup-curl/src/lib.rs:1595`). The connect deadline stays **sub-second
  (250 ms)** and is still proven end-to-end through a real spawn; the total
  budget moves from 750 ms to **30 s**, so the parent deadline can no longer
  expire for scheduling reasons. The assertion follows the value it now
  protects: `--connect-timeout 0.25` and `--max-time 30`.
- **§6.2 — deterministic coverage** (new, platform-neutral,
  `sub_second_deadlines_serialize_truthfully_without_a_subprocess`,
  `crates/eggup-curl/src/lib.rs:1514`). With **no subprocess, no network, and
  no wall clock**, it asserts that a sub-second connect **and** total pair
  (250 ms / 750 ms) serializes to `--connect-timeout 0.25` / `--max-time 0.75`,
  that `--max-time` is the **effective** total ceiling and never the raw
  request when the adapter ceiling is lower (400 ms → `0.4`, never `0.75`),
  and that sub-microsecond positive durations are still rejected as
  `InvalidInput`.
- **§6.3 — test B** (`sub_second_total_timeout_kills_child_promptly`,
  `crates/eggup-curl/src/lib.rs:1621`). The absolute `elapsed < 750 ms`
  ceiling is replaced by two discriminating assertions: `elapsed >=
  short_total.total_timeout` (the deadline was actually reached — the fetch
  did not succeed early) and `elapsed < 2 s` (the child was killed rather
  than left to stall, against a 4000 ms server stall). The residue assertions
  (`!dest.exists()`, empty directory) are unchanged.
- **§6.4 — the optional private deadline seam was SKIPPED.** Justification is
  in *Deviations* below.

## Requirement → evidence matrix

| Req | Requirement | Evidence | Command / result |
|---|---|---|---|
| §6.1 | Test A's asserted serialization is decoupled from the enforced budget | `crates/eggup-curl/src/lib.rs:1595-1617` — `connect_timeout: 250 ms`, `total_timeout: 30 s` | `cargo test -p eggup-curl --locked --offline` → `build_curl_args_passes_sub_second_deadlines_to_fake_curl ... ok` |
| §6.1 | Test A still asserts `--connect-timeout 0.25` in the recorded child argv | `:1611-1614` | same run → `ok`; NC-5 proves the assertion is live |
| §6.1 | Test A completes without depending on a sub-second wall-clock budget | parent deadline is 30 s | loaded loop: 120/120 and 40/40 at loadavg 337→366 and 205→300 |
| §6.2 | Sub-second connect **and** total pair → `0.25` / `0.75`, no subprocess | `:1514-1570`, `sub_second_deadlines_serialize_truthfully_without_a_subprocess` | `cargo test -p eggup-curl --locked --offline` → `... ok`; NC-1 / NC-2 prove both halves are live |
| §6.2 | `--max-time` is the **effective** ceiling, never the raw request | `:1539-1558` (adapter ceiling 400 ms → `0.4`; `assert_ne!(…, "0.75")`) | NC-1: mutated to expect `0.75` → **FAILED** (`left: "0.4"`, `right: "0.75"`) |
| §6.2 | Sub-microsecond positive durations still yield `InvalidInput` | `:1560-1569` | NC-3: mutated to expect success → **FAILED** (`sub-microsecond duration must be rejected`) |
| §6.3 | Test B asserts `Timeout { phase: "total" }` | `:1649` | `cargo test -p eggup-curl --locked --offline` → `sub_second_total_timeout_kills_child_promptly ... ok` |
| §6.3 | Test B asserts `elapsed >= total_timeout` | `:1657-1661` | NC-6: mutated to `elapsed >= 2 s` → **FAILED** (`elapsed 251.461917ms is below the 250ms total deadline`) |
| §6.3 | Test B asserts `elapsed` is far below the 4000 ms stall | `:1668-1671` (`elapsed < 2 s`) | NC-8: with the deadline removed the elapsed is **4.02 s** and the fetch succeeds → the bound discriminates kill-from-stall; NC-7: mutated to `< 100 ms` → **FAILED** |
| §6.3 | Test B retains `!dest.exists()` and the empty-directory residue check | `:1672-1673` (unchanged) | `cargo test -p eggup-curl --locked --offline` → `ok` |
| §6.4 | Optional private deadline seam | **skipped** — see *Deviations* | no production seam added; the production poll loop is untouched |
| §4 | No production deadline, tolerance, `POLL_INTERVAL`, or classification changed | SHA-256 of lines 1–804 (everything before `#[cfg(test)]` at `:805`) is **identical** pre- and post-change: `a5ac59ad42427f73391a34c71c941d42a2b03f0a36b77036b2d563e26c988e88` | `head -n 804 … \| shasum -a 256` on both files → identical |
| §4 | No new dependency edge | `cargo tree -p eggup-curl --locked --offline` → `eggup-curl v0.1.3` → `eggup-acquisition v0.1.3` only | same command before and after → unchanged |
| §10 | The four already-deterministic sub-second tests are unchanged and green | `exit_28_is_labelled_total_when_connect_and_total_ceilings_coincide` (`:827`), `total_phase_timeout_is_typed_end_to_end` (`:1231`), `connect_phase_timeout_is_typed_end_to_end` (`:1276`), `duration_decimal_seconds_serializes_sub_second_inputs_truthfully` (`:1366`), `build_curl_args_passes_decimal_deadlines_never_above_ceiling` (`:1454`) — **none of them is in the M012 diff**. (The plan's §3.4 names the connect/total pair `connect_and_total_timeouts_are_typed`; after acquisition M013 split it, the two live tests are `total_phase_timeout_is_typed_end_to_end` and `connect_phase_timeout_is_typed_end_to_end`) | `cargo test -p eggup-curl --locked --offline` → all `ok` except `connect_phase_timeout_is_typed_end_to_end`, which is **ignored with a recorded reason** by M013's design |
| §10 | `cancellation_during_body_stream_kills_and_reaps_child` unchanged and green | `:1700`, **not in the M012 diff** | same run → `ok` |
| §10 | Full `eggup-curl` suite green | 26 passed, 0 failed, 1 ignored (M013's environment-conditional connect test, reason recorded) | `cargo test -p eggup-curl --locked --offline` → exit 0 |
| §13 | `git diff` touches only `#[cfg(test)]` code in `crates/eggup-curl/src/lib.rs` | the M012 diff has **three** hunks, all at source line ≥ 1500, i.e. inside the `#[cfg(test)] mod tests` that opens at `:805` | `diff -u` of the pre-M012 file vs the current file → hunks at `@@ -1500,6 +1500,96 @@`, `@@ -1510,19 +1600,19 @@`, `@@ -1557,9 +1647,28 @@` |
| §13 | Loaded loop shows 0 failures for both tests | see *Loaded re-run evidence* | 40/40 and 120/120 for both tests |
| §13 | Unloaded loop shows 0 failures across 100 iterations | see *Loaded re-run evidence* | 100/100 and 100/100 |
| §13 | The pre-change 3/40 reproduction is recorded as superseded evidence | see *Baseline-failure proof* | pre-change: **1 failure / 60** at loadavg 119.88→177.86; post-change: **0 / 240** under the same generator |
| §13 | M007's `TooLarge { limit: 0 }`-style environment limits and the Windows live-curl limitation still apply | M012 closed neither; they are **retained limitations**, not findings | see *Retained limitations* |
| §13 | `plans/registry.md` no longer carries open finding 1 | **not done by this milestone** — the parent session owns `plans/registry.md` | see *Roadmap disposition* |

## Production implementation evidence

**None — by design.** §6 of the plan states: *"No production change is required.
M012 is a test-only milestone. If an implementer finds themselves editing
non-`#[cfg(test)]` code, that is a signal to stop and re-read §14."* No
non-`#[cfg(test)]` line of `crates/eggup-curl/src/lib.rs` was touched:

```text
$ head -n 804 /tmp/eggup-m012-baseline-pristine.rs | shasum -a 256
a5ac59ad42427f73391a34c71c941d42a2b03f0a36b77036b2d563e26c988e88  -
$ head -n 804 crates/eggup-curl/src/lib.rs | shasum -a 256
a5ac59ad42427f73391a34c71c941d42a2b03f0a36b77036b2d563e26c988e88  -
```

Line 805 is `#[cfg(test)]`; every M012 hunk is at line ≥ 1500.

Invariants that were checked and did **not** regress:

| Invariant (§4) | Status | Evidence |
|---|---|---|
| `POLL_INTERVAL` remains 5 ms; the parent remains the total-deadline enforcer | **held** | production region byte-identical (hash above) |
| Parent kills + reaps the child and removes the owned temp on every failure path | **held** | production region byte-identical |
| `Timeout { phase }` classification unchanged, incl. the `exit 28` connect-vs-total discrimination | **held** | production region byte-identical; the M013 `classify_curl_result_labels_connect_and_total_without_a_subprocess` proof still passes |
| `CurlConfig::validate()` still rejects zero timeouts and `connect > total` | **held** | production region byte-identical |
| `build_curl_args` still emits truthful decimal seconds and never above the effective ceiling; sub-microsecond positive durations still fail validation | **held** | §6.2 test + NC-1/NC-2/NC-3 |
| `FetchLimits::effective` still yields `min(request, adapter ceiling)` | **held** | §6.2 test asserts the derived effective pair before serializing |
| No production deadline widened; no slack/tolerance added; `POLL_INTERVAL + 5 ms` stays a labelling margin | **held** | production region byte-identical |
| No retry, sleep, poll, or backoff added to test infrastructure | **held** | the load loop runs the **test binary** repeatedly, outside the test; no test contains a retry loop |
| A real `Timeout { phase: "total" }` is never converted into a success; a genuine regression is never relabelled as flakiness | **held** | NC-8 shows that removing the deadline yields an `Ok` result and a failing test, not a relabelled pass |
| Public API, error types, dependency graph, and published behaviour unchanged | **held** | `cargo tree -p eggup-curl` unchanged; no public item added or removed |
| `eggup-curl`'s only Eggup dependency remains `eggup-acquisition` | **held** | `cargo tree -p eggup-curl` → one edge only |
| Rust 1.89 supported; no new dependency; no `unsafe` | **held** | no manifest change; `unsafe_code = "deny"` workspace-wide |
| Windows behaviour unchanged: test A stays `#[cfg(unix)]`, test B stays `#[cfg(not(windows))]` | **held** | gates untouched; the new §6.2 test is platform-neutral and therefore **adds** Windows coverage |
| M013's fail-loudly fixture behaviour preserved | **held** | `require_curl_executable` / `require_transport` (`:915-933`) are not in the M012 diff; no silent early exit was reintroduced (`crate_sources_have_no_vacuous_early_exit` still passes) |

## Exact commands run and results

### Scoped local gate (the commands this milestone owns)

| Command | Result |
|---|---|
| `cargo test -p eggup-curl --locked --offline` | **exit 0** — lib: `26 passed; 0 failed; 1 ignored` (the ignored one is M013's environment-conditional `connect_phase_timeout_is_typed_end_to_end`, with a recorded reason); `tests/assertion_guard.rs`: `3 passed; 0 failed`; doc-tests: `0` |
| `cargo clippy -p eggup-curl --all-targets --all-features --locked --offline -- -D warnings` | **exit 0** — `Finished dev profile`, no warnings |
| `cargo doc -p eggup-curl --no-deps --locked --offline` | **exit 0** — `Generated …/target/doc/eggup_curl/index.html` |
| `cargo fmt -p eggup-curl -- --check` | **exit 0** — this crate's sources are fmt-clean |
| `cargo fmt --all -- --check` | **exit 1** — the only diffs are in `crates/eggup-service/src/windows_scm.rs`, `crates/eggup-service/tests/public_api_surface.rs`, and `crates/eggup-service/tests/windows_scm_live.rs`. Those are **service-lifecycle** files owned by a sibling milestone; they were **not** reformatted by this milestone (see UF-4) |
| `git diff --check` | **clean** (no whitespace errors) |
| `cargo tree -p eggup-curl --locked --offline` | `eggup-curl v0.1.3` → `eggup-acquisition v0.1.3` — no new edge |

### Plan §11 commands — status

| Command (§11) | Status in this record |
|---|---|
| `cargo fmt --all -- --check` | **run** — fails only on three service-lifecycle files (not owned here) |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | **not run** — parent session owns the whole-tree gate (UF-2) |
| `cargo test -p eggup-curl --all-targets --all-features --locked` | **run** in scoped form (`cargo test -p eggup-curl --locked --offline`, which builds all targets of the crate); the `--all-features` form is equivalent here — the crate has no features |
| `cargo test --workspace --all-targets --all-features --locked` | **not run** — parent session (UF-2) |
| `cargo doc --workspace --no-deps --locked` | **not run** in workspace form; the crate-scoped `cargo doc -p eggup-curl --no-deps --locked --offline` was run and is clean (UF-2) |
| `cargo +1.89.0 check --workspace --all-targets --locked` | **not run** — parent session (UF-2) |
| `git diff --check` | **run** — clean |
| Hosted Stable / MSRV / macOS / Windows matrix | **not run** — no hosted run was triggered by this milestone (UF-2) |

## Baseline-failure proof

### Mechanism

The main working tree's git state was **never** mutated. The baseline was
materialised with:

```text
git worktree add /tmp/eggup-acq-m012-base-14577 HEAD      # detached at c480541
cp crates/eggup-acquisition/src/lib.rs  <worktree>/crates/eggup-acquisition/src/lib.rs
cp crates/eggup-curl/src/lib.rs         <worktree>/crates/eggup-curl/src/lib.rs
```

and then the three M012 edits were reverted **inside the worktree copy
only**, so the worktree held the exact pre-M012 state (M013 present, M012
absent). The resulting file was verified to differ from the current file by
**exactly** the three M012 hunks and nothing else. The worktrees were removed
afterwards with `git worktree remove --force`.

### Observed pre-change failure — test A (the plan's headline defect)

Load generator: **224 `yes > /dev/null` burners (16 × ncpu = 16 × 14)**, 25 s
settle, then the test binary run sequentially with `--test-threads=1`.

```text
== label=BASELINE(pre-M012) heavy binary=/tmp/eggup-acq-m012-base-14577/.../eggup_curl-07215b0f29bd1711
   iters=60 burners=224 (16x ncpu=14) settle=25s
== loadavg at start: { 119.88 44.44 29.92 }
== test=build_curl_args_passes_sub_second_deadlines_to_fake_curl pass=59 fail=1
---- first failure output ----
thread 'tests::build_curl_args_passes_sub_second_deadlines_to_fake_curl' panicked
  at crates/eggup-curl/src/lib.rs:1516:14:
called `Result::unwrap()` on an `Err` value: Timeout { phase: "total" }
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 25 filtered out; finished in 0.75s
------------------------------
== test=sub_second_total_timeout_kills_child_promptly pass=60 fail=0
== loadavg at end: { 177.86 63.96 37.37 }
```

This is the plan §2 failure verbatim in shape: the parent deadline
(`Timeout { phase: "total" }`) expires **while the test is trying to
succeed**, at a reported duration of **0.75 s** against the test's own 750 ms
`total_timeout`. The mechanism is exactly §3.1's: the fake-curl child is an
`sh` script that ignores `--connect-timeout` and `--max-time`, so the parent's
deadline is the **only** enforcement, and under scheduling pressure the parent
is starved past its own budget.

### Observed pre-change behaviour — test B (honest negative)

Test B's pre-change absolute ceiling was **not** observed to fail on this
host. A dedicated probe build replaced the 750 ms assertion with a print of the
measured elapsed, and the test was run 120 times under the same load
generator:

```text
== probe iters=120 burners=224 (16x ncpu=14) settle=25s
== loadavg at start: { 186.97 103.41 56.68 }
== loadavg at end:   { 228.49 123.01 65.70 }
== samples: 120
M012PROBE elapsed 273.286834ms
M012PROBE elapsed 281.01125ms
M012PROBE elapsed 282.796083ms
-- count over 750ms (the pre-change absolute ceiling):
0
```

**0 of 120** samples exceeded 750 ms; the maximum observed elapsed was
**282.8 ms**. This matches the plan's own record for test B ("0 failures / 40"
at baseline). The pre-change test-B ceiling is therefore a **latent**
load-sensitivity on this host, not an observed failure, and this record does
not claim otherwise (UF-1). The §6.3 change remains correct and required: it
removes the absolute constant, adds the missing lower bound, and widens the
headroom from 500 ms to 1.75 s — and the probe shows the measured elapsed
sits at 251–283 ms, i.e. **the removed 750 ms bound had only ~2.6× headroom
over the observed maximum**, while the new 2 s bound has ~7× over it.

### Negative controls — every new assertion proven to fire

Eight controls were run against the **post-M012** code, each mutating exactly
one assertion, in a second worktree
(`/tmp/eggup-acq-m012-neg-37223`). The pristine post-M012 file was restored
before each control, so the controls are independent and the shipped code was
never left mutated. **All eight FAILED as required.**

| Control | Mutation | Observed failure |
|---|---|---|
| NC-1 | `--max-time` expected `"0.75"` instead of the effective `"0.4"` | `assertion left == right failed` — `left: "0.4"`, `right: "0.75"` |
| NC-2 | `--connect-timeout` expected `"1"` instead of `"0.25"` | `left: "0.25"`, `right: "1"` |
| NC-3 | sub-microsecond duration expected to be **accepted** | `sub-microsecond duration must be rejected` |
| NC-4 | test A expected `--max-time 0.75` (the **exact pre-change assertion**) | `argv missing 0.75: --disable --silent --show-error --no-progress-meter --location --max-redirs 10 --connect-timeout 0.25 --max-time 30 --max-filesize 65536 --proto =http,https --proto-redir =https --noproxy * --output - --write-out %{stderr}%{http_code} -- https://example.com/x` |
| NC-5 | test A expected `--connect-timeout 1` | `argv missing 1: …` |
| NC-6 | test B lower bound raised to `elapsed >= 2 s` | `elapsed 251.461917ms is below the 250ms total deadline` |
| NC-7 | test B upper bound lowered to `elapsed < 100 ms` | `elapsed 258.437458ms is not below the 2 s bound` |
| NC-8 | test B's total deadline raised to 10 s (the deadline no longer fires) | `called Result::unwrap_err() on an Ok value: Success(ArtifactEvidence { bytes_written: 8192 })`, test duration **4.02 s** |

NC-4 is the decisive proof that §6.1's decoupling is real: the **exact
pre-change assertion** now fails, and the recorded child argv shows the
decoupled pair `--connect-timeout 0.25 --max-time 30`.

NC-8 is the decisive proof that §6.3's 2 s bound discriminates kill-from-stall:
with the deadline removed, the fetch runs the full 4000 ms body stall and
**succeeds** — an order of magnitude above the bound — so the bound is live and
the elapsed measurement is meaningful.

## Loaded re-run evidence — the discriminating check

Post-change, the **same** load generator was used (224 `yes` burners =
16 × ncpu, 25 s settle), and the **same** test binary invocation pattern
(`--test-threads=1`, one test per process).

| Run | Burners | Loadavg (start → end) | `build_curl_args_passes_sub_second_deadlines_to_fake_curl` | `sub_second_total_timeout_kills_child_promptly` |
|---|---|---|---|---|
| Unloaded, 100 iterations each | 0 synthetic | 62.28 → 60.55 (host's unrelated external build load) | **100 / 100**, 0 failures | **100 / 100**, 0 failures |
| **Loaded, 40 iterations each** (plan §7 step 7 / §11) | 224 (16 × ncpu) | **205.42 → 300.46** | **40 / 40**, 0 failures | **40 / 40**, 0 failures |
| Loaded, 120 iterations each (extension, matching the baseline's iteration count) | 224 (16 × ncpu) | **337.17 → 366.43** | **120 / 120**, 0 failures | **120 / 120**, 0 failures |

**Pre-change vs post-change, same generator, same host:**

| | Pre-M012 | Post-M012 |
|---|---|---|
| test A, loaded | **1 failure / 60** (loadavg 119.88 → 177.86) | **0 failures / 160** (loadavg 205.42 → 366.43) |
| test B, loaded | 0 failures / 180 (loadavg up to 228.49) | **0 failures / 160** (loadavg up to 366.43) |

The post-change runs were executed under a **higher** observed load average
than the baseline run that failed (205–366 vs 120–178), and the loaded
failures are **0**. The unloaded rerun alone is explicitly **not** treated as
the discriminating evidence; the loaded runs are.

## Invariant review

See *Production implementation evidence* for the full table. Every §4 invariant
is **held**. Specifically:

- No production deadline was widened. The only `Duration` values that changed
  are **inside `#[cfg(test)]`**: test A's `total_timeout` (750 ms → 30 s) and
  test B's assertion bounds.
- No production tolerance, retry, sleep, poll, backoff, or slack was added.
- The parent's 5 ms poll and `POLL_INTERVAL + 5 ms` labelling margin are
  untouched.
- No genuine runtime failure was relabelled as test flakiness: the pre-change
  failure was reproduced **as a failure** (NC-8 shows the un-deadlined fetch
  succeeding rather than being tolerated).
- No test was made order-dependent, and neither test leaves a stale
  `temp_dir` behind: both clean up with `fs::remove_dir_all`.

## Failure / rollback / recovery review

- **No production failure semantics changed.** A parent deadline expiry remains
  a hard `Timeout { phase: "total" }` after `kill` + `wait` + both reader-thread
  joins + `__remove_owned_temp`.
- **No rollback was needed**: the change is test-only and additive; if it were
  reverted, the crate returns to its pre-M012 (flake-prone) state with no other
  consequence.
- **Cancellation semantics untouched**: `cancellation_before_spawn_cleans_temp`
  and `cancellation_during_body_stream_kills_and_reaps_child` are not in the
  M012 diff and are green.
- **Test infrastructure introduces no masking**: the load loop runs the **test
  binary** repeatedly from outside; no test contains a retry loop, sleep, or
  poll that could hide a real deadline failure.

## Compatibility and migration review

- **Public API: unchanged.** No public item added, removed, or retyped in
  `eggup-curl`. The new test and its helpers are inside `#[cfg(test)]`.
- **Error types: unchanged.** `AcquisitionError` is not touched by M012; the
  seam's typed `Io { operation, source }` form (acquisition M013) is used
  as-is.
- **Dependency graph: unchanged.** `cargo tree -p eggup-curl` → one Eggup
  edge, `eggup-acquisition`.
- **Published behaviour: unchanged.** The change is confined to
  `#[cfg(test)]` code, so the packaged library is behaviourally and
  API-wise identical.
- **No publication is authorized or implied.** `eggup-curl` is published at
  `0.1.2`; M012 performs no version bump, no tag movement, and no crates.io
  action. If a future implementer concludes the change alters packaged bytes
  or public behaviour — it does not — that is a separate release milestone per
  `.opencode/skills/release-workflow`.
- **No consumer migration is required.**

## Security review

| Concern | Status | Evidence |
|---|---|---|
| No production deadline widened to mask a failure | **held** | production region byte-identical (SHA-256 above) |
| No retry / backoff that could turn a real timeout into a success | **held** | no retry in any test; NC-8 shows a non-deadlined fetch reported as `Ok`, not absorbed |
| Test-only surface cannot leak into the packaged library | **held** | every added line is inside `#[cfg(test)] mod tests` (`:805`–end) |
| URL / credential material exposure unchanged | **unchanged, disclosed** | `architecture/curl-adapter.md` §7.4 already records the exact URL in `argv`; M012 changed no process lifecycle. The recorded argv in NC-4 is the synthetic `https://example.com/x` test URL |
| SHA-256 remains integrity evidence only | **held** | not touched |
| The seam still holds no transport or service-manager dependency | **held** | `cargo tree -p eggup-curl` → `eggup-acquisition` only |

## Documentation / operations evidence

| Item | Status |
|---|---|
| `crates/eggup-curl/CHANGELOG.md` `Unreleased` — M012 entry appended **after** M013's entry (M013's text left byte-for-byte intact): the decoupled test A, the removed absolute ceiling, the new deterministic coverage, plus a compatibility paragraph stating test-only / no publication / no migration | **done** |
| `plans/implementation/acquisition-transport/012-sub-second-deadline-test-determinism-corrective.md` `Status:` header → `conditionally closed`, pointing at this record | **done** |
| `plans/closure/acquisition-transport/012-status.md` — this record | **done** |
| `architecture/curl-adapter.md` test table (`:708-709`) still describes the pre-M012 assertions and carries stale line anchors | **NOT done — finding UF-3.** The file is owned by acquisition M013 under this session's file-ownership split, so it was recorded rather than edited. It needs: `build_curl_args_passes_sub_second_deadlines_to_fake_curl` → "argv actually carries `0.25` / `30`"; `sub_second_total_timeout_kills_child_promptly` → "250 ms total vs 4 s stall; `elapsed >= 250 ms` and `elapsed < 2 s`, dir empty"; a new row for `sub_second_deadlines_serialize_truthfully_without_a_subprocess`; and refreshed line anchors for the rows at `:703-711` |
| `plans/subsystems/acquisition-transport-roadmap.md` and `plans/registry.md` | **not done** — the parent session owns both (see *Roadmap disposition*) |
| `docs/` (non-normative) | **untouched** — no rule is introduced there |
| Historical closure records `008-status.md` / `010-status.md`, which describe the pre-M012 assertions | **deliberately untouched** — closure records state what was true when written and are not rewritten (same discipline as superseded ADRs). Recorded as UF-5 |

## Deviations and decisions

### §6.4 — the optional private deadline seam was **SKIPPED**

Justification, per the plan's own condition ("Adding the seam is only
justified if §6.3's bound is judged too loose to prove promptness"):

1. **§6 forbids it.** A deadline seam requires the production poll loop to
   evaluate expiry through a helper — a non-`#[cfg(test)]` edit — which §6
   explicitly flags as a signal to stop. §6.1/§6.3 are sufficient to close
   the finding.
2. **§6.3's bounds are not loose.** They discriminate the contract without a
   wall-clock constant, and every half is proven live by a negative control:
   NC-6 (the lower bound fires), NC-7 (the upper bound fires), NC-8 (without
   the deadline the elapsed is 4.02 s — an order of magnitude above the 2 s
   bound — and the fetch succeeds).
3. The smallest coherent change is the two-test decoupling plus one
   deterministic test. Adding a seam would have broadened the diff into
   production code for no proven gain.

### The §6.1 trade-off, stated explicitly

A sub-second `--max-time` is **not** proven end-to-end. It is proven
**deterministically at the `build_curl_args` layer** (250 ms / 750 ms →
`--connect-timeout 0.25` / `--max-time 0.75`), and the reason is structural,
not an oversight: the parent's deadline **is** the effective total ceiling, so
an end-to-end proof of a sub-second `--max-time` would require a sub-second
wall-clock budget — exactly the coupling M012 removes. The end-to-end test now
proves the sub-second **connect** value reaches the child's argv through a
real spawn, with a generous total budget.

## Per-platform results

Reported separately. **Nothing is inferred from one platform to another.**

| Platform | Status | Result |
|---|---|---|
| **macOS** (this host: `darwin` / `arm64`, 14 cores) | **run** | `cargo test -p eggup-curl --locked --offline` → 26 passed / 0 failed / 1 ignored (+3 guard tests); clippy `-D warnings` clean; `cargo doc` clean; `cargo fmt -p eggup-curl -- --check` clean; unloaded 100/100 both tests; **loaded** 40/40 and 120/120 both tests at loadavg up to 366.43; pre-change baseline 1 failure / 60 reproduced; 8 negative controls fired |
| **Linux** | **not run** | no Linux host, container, or hosted lane was available in this session. Not inferred from the macOS result |
| **Windows** | **not run** | no Windows host or hosted lane was available in this session. Not inferred from the macOS result. Note: test A remains `#[cfg(unix)]` and test B remains `#[cfg(not(windows))]`, so the Windows lane runs neither of the two changed tests; the **new** §6.2 test is platform-neutral and does run on Windows, which is where the sub-second `--max-time` proof now lives |

## Retained limitations (not closed by M012)

- **M007's Windows live-curl limitation stands.** The Windows lane still
  carries only the portable adapter, process, and error-path fixtures; no
  Windows live HTTP/curl result is claimed. M012 closes neither this nor the
  `TooLarge { limit: 0 }`-style environment limits.
- **Test A remains `#[cfg(unix)]`** — it needs `sh` + `xxd`.
- **Test B remains `#[cfg(not(windows))]`** — it drives the real `curl`
  subprocess against an in-process loopback server.
- The end-to-end sub-second **`--max-time`** proof does not exist and cannot
  exist without reintroducing the wall-clock coupling (see *§6.1 trade-off*).

## Unresolved findings

### UF-1 — the pre-change test B absolute ceiling was never observed to fail on this host — **low**

120 iterations at loadavg up to 228.49 measured a maximum elapsed of
282.796083 ms; **0 of 120** samples exceeded the pre-change 750 ms bound. The
plan's own baseline record shows 0/40 for test B. The §6.3 change is still
required (it removes an absolute wall-clock constant and adds the missing
lower bound), but the *evidence* for test B is a **latent** load-sensitivity
plus the negative controls, not an observed pre-change failure. This record
makes no stronger claim.

**Disposition:** carried as a low finding, closed by this milestone's §6.3
change. No follow-up milestone needed.

### UF-2 — the whole-tree gate and the non-macOS lanes were not run — **low**

`cargo test --workspace --all-targets --all-features --locked`,
`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`,
`cargo doc --workspace --no-deps --locked`,
`cargo +1.89.0 check --workspace --all-targets --locked`, and
`./scripts/check-local.sh` were **not run**, and the Linux, Windows, and
hosted-CI lanes were **not run**.

**Severity: low** for this milestone's own scope (one crate, test-only, fully
green on the native lane with a discriminating baseline proof and a loaded
re-run), but the plan's §13 acceptance criteria that depend on those commands
are **not** met by this record. The parent session's whole-tree gate owns
them.

### UF-3 — `architecture/curl-adapter.md` carries the pre-M012 test table — **low**

`architecture/curl-adapter.md:708-709` still reads "argv actually carries
`0.25` / `0.75`" and "250 ms total vs 4 s stall; < 750 ms elapsed, dir empty",
and the line anchors in the table (`:703-711`) are stale after M013 and M012.
The new §6.2 test is not listed.

**Disposition:** the file is owned by acquisition M013 under this session's
file-ownership split, so it was **recorded, not edited**. The exact edits
required are enumerated in *Documentation / operations evidence*. The owner
should apply them; the parent session should not close the milestone's
documentation gate until they are applied.

### UF-4 — `cargo fmt --all -- --check` fails on three service-lifecycle files — **low**

`crates/eggup-service/src/windows_scm.rs`,
`crates/eggup-service/tests/public_api_surface.rs`, and
`crates/eggup-service/tests/windows_scm_live.rs` are unformatted. They are
**not** owned by this milestone and were deliberately **not** reformatted.
`cargo fmt -p eggup-curl -- --check` is clean. The parent session's whole-tree
gate owns this.

### UF-5 — historical closure records describe the pre-M012 assertions — **informational**

`plans/closure/acquisition-transport/008-status.md:31-33,49-50` and
`010-status.md:110` describe the pre-M012 assertions. They state what was true
when written and are deliberately **not** rewritten; the live truth is
`architecture/curl-adapter.md`, which is UF-3.

## Roadmap disposition

`plans/subsystems/acquisition-transport-roadmap.md` and `plans/registry.md`
are **owned by the parent session** and were **not** edited by this milestone.
The parent session should:

1. mark the acquisition-transport M012 row **closed (conditionally)**, with
   the pointer to this record;
2. mark the "Post-0.1.3 open findings" entry **1** (the two sub-second
   wall-clock deadline tests) as **closed**, with the pointer to this record
   and a note that the pre-change failure was reproduced on the macOS lane at
   loadavg 119.88→177.86 (1 failure / 60) while test B's absolute ceiling was
   **not** reproduced on this host (0 / 120, max elapsed 282.8 ms);
3. reconcile the roadmap's M012 row and the surrounding prose that repeats the
   pre-change reproduction figures (e.g. `:284`);
4. apply the `architecture/curl-adapter.md` table correction described in
   UF-3 — or route it to the M013 owner.

## Registry updates

**None performed by this milestone** — `plans/registry.md` is owned by the
parent session (see *Roadmap disposition*). This section records what the
parent session must apply.

## Files touched

**Source (1 file, `#[cfg(test)]` module only):**

- `crates/eggup-curl/src/lib.rs` — the `#[cfg(test)] mod tests` block only
  (`:805`–end); three hunks, all at line ≥ 1500; lines 1–804 byte-identical
  to the pre-M012 state

**Changelog (1 file):**

- `crates/eggup-curl/CHANGELOG.md` — the `Unreleased` section, appended after
  M013's entry

**Plan + closure (2 files):**

- `plans/implementation/acquisition-transport/012-sub-second-deadline-test-determinism-corrective.md`
  (`Status:` header only)
- `plans/closure/acquisition-transport/012-status.md` (this file)

**Deliberately NOT touched:** `Cargo.toml`, `Cargo.lock`, every
`crates/*/Cargo.toml`, `crates/eggup-acquisition/**`, `crates/eggup-eggfetch/**`,
`crates/eggup-core/**`, `crates/eggup-archive/**`, `crates/eggup-service/**`,
`crates/eggup-eggpack/**`, `crates/eggup-transport-footprint/**`,
`architecture/**`, `docs/**`, `plans/registry.md`, `plans/subsystems/**`,
`plans/adrs/**`, the root `CHANGELOG.md`, and every other crate.
