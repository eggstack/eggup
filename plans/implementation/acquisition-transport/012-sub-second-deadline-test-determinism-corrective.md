# Acquisition Transport Milestone 012 — sub-second deadline test determinism corrective

Status: ready for handoff

Repository baseline: `93267304f96bb4de50128080769f3c77dd03892c`

Source roadmap:

- `plans/subsystems/acquisition-transport-roadmap.md`

Original closed work affected:

- `plans/implementation/acquisition-transport/008-subsecond-deadline-truthfulness-corrective.md`
- `plans/closure/acquisition-transport/008-status.md`

Finding of record:

- `plans/registry.md` § "Post-0.1.3 open findings" item 1

Long-term requirements:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`

Applicable ADRs:

- `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md`
- `plans/adrs/ADR-0005-local-archive-extraction-safety-contract.md`

Precedent shape:

- `plans/implementation/service-lifecycle/008-operation-deadline-test-determinism-corrective.md`
  — the same defect class (scheduler-sensitive wall-clock assertions in a unit
  test) in a different crate. M012 must not repeat M008's mistakes and must not
  contradict its discipline.

Primary class: invariant

## 1. Objective

Make the two sub-second deadline tests in `eggup-curl` deterministic on a
loaded host **without weakening any production deadline**, so that the
transport's deadline contract is proven by assertion rather than by the host's
scheduler.

The two tests are:

- `build_curl_args_passes_sub_second_deadlines_to_fake_curl`
  (`crates/eggup-curl/src/lib.rs:1367`)
- `sub_second_total_timeout_kills_child_promptly`
  (`crates/eggup-curl/src/lib.rs:1393`)

Both currently hold sub-second `FetchLimits` values that serve **two
incompatible purposes at once**: they are the values whose serialization the
test wants to assert, *and* they are the live wall-clock budgets the parent
process enforces against a real spawned child. Under host scheduling pressure
the second role fails, and a correct transport is reported as broken.

M012 must separate those roles. It must not widen a production deadline, add
production slack, retry, or reclassify a genuine runtime failure as test
flakiness.

## 2. Why this milestone is ready

The finding is fully characterized, reproduced locally, and scoped to a crate
with no open milestones and no dependency on unpublished work.

Evidence:

- **Reproduced on this host**, at baseline `9326730`, on macOS with 14 cores
  under sustained external load (loadavg 62–68 during the run).
  40 sequential single-threaded iterations of each test:

  | Test | Result |
  |---|---|
  | `build_curl_args_passes_sub_second_deadlines_to_fake_curl` | **3 failures / 40** |
  | `sub_second_total_timeout_kills_child_promptly` | 0 failures / 40 |

- The failure is an explicit parent-side deadline expiry, not a transport bug:

  ```text
  thread 'tests::build_curl_args_passes_sub_second_deadlines_to_fake_curl' panicked
  at crates/eggup-curl/src/lib.rs:1378:14:
  called `Result::unwrap()` on an `Err` value: Timeout { phase: "total" }
  ```

  Reported test durations were `0.77s`, `0.76s` — i.e. the test overran **its
  own 750 ms `total_timeout`** while trying to succeed.

- The defect predates the 0.1.3 train and is unrelated to it. It was observed at
  pristine `bb8fe41` via `git stash`, and no commit in the 0.1.3 train touched
  `crates/eggup-curl`. Hosted CI passes both tests on every lane, which is
  exactly why this went unnoticed: the lane runners are not loaded this way.

- Severity is **low**. Production behavior is correct; the defect is test
  robustness. Per this repository's own gate, "medium-or-higher" is what forces
  a corrective before a milestone closes, so nothing is blocked. It is being
  authored because a test that fails ~7.5% of the time on a busy developer
  machine will eventually be misread as a transport regression, and because
  `plans/registry.md` explicitly records that no plan covers it.

- Scope is bounded and self-contained: 2 tests in 1 crate, no public API, no
  dependency, no publication implication.

## 3. Current implementation evidence

### 3.1 Parent-side deadline enforcement (production)

`eggup-curl` enforces the total deadline **itself**, by polling the child:

```rust
const POLL_INTERVAL: Duration = Duration::from_millis(5);   // lib.rs:32
...
match child.try_wait() {
    Ok(Some(status)) => break status.code(),
    Ok(None) => {
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            let _ = body_reader.join();
            let _ = status_reader.join();
            eggup_acquisition::__remove_owned_temp(output);
            return Err(AcquisitionError::Timeout { phase: "total" });  // lib.rs:674
        }
        std::thread::sleep(POLL_INTERVAL);                            // lib.rs:676
    }
    ...
}
```

`curl` also receives `--connect-timeout` / `--max-time` and exits `28` on its
own timeout, classified at `lib.rs:740-751`.

Two facts follow, and both matter:

1. The fake-curl test child is **not real curl**. It is an `sh` script that
   writes its `argv`, dumps `env`, pipes through `xxd`, and exits `0`. It
   **ignores `--connect-timeout` and `--max-time` entirely.** Therefore only the
   parent's own `deadline` can end that fetch — the parent deadline is not a
   fallback, it is the *only* enforcement.
2. `validate()` (`lib.rs:127-135`) rejects `connect_timeout > total_timeout`, so
   a sub-second *connect* value can always be paired with a generous *total*.

### 3.2 Flaky test A — the success-path ceiling

```rust
let cfg = FetchLimits {
    connect_timeout: Duration::from_millis(250),   // lib.rs:1374
    total_timeout: Duration::from_millis(750),     // lib.rs:1375
};
t.fetch_metadata(&req("https://example.com/x"), cfg, &CancelFlag::new())
    .unwrap();                                      // lib.rs:1377-1378  <-- panics
let argv = fs::read_to_string(dir.join("argv")).unwrap();
assert!(argv.contains("--connect-timeout 0.25"), ...);
assert!(argv.contains("--max-time 0.75"), ...);
```

The invariant actually under test is *sub-second `FetchLimits` values are
serialized truthfully into the spawned child's argv*. The 750 ms budget is
incidental — but it is enforced against `sh` + `env` + `xxd` spawn latency, so it
is an assertion about **host scheduling**, with effectively zero headroom.

### 3.3 Flaky test B — the absolute elapsed ceiling

```rust
let short_total = FetchLimits {
    connect_timeout: Duration::from_millis(100),   // lib.rs:1406
    total_timeout: Duration::from_millis(250),     // lib.rs:1407
};
let started = Instant::now();
let err = t.fetch_artifact(...).unwrap_err();
let elapsed = started.elapsed();
assert!(matches!(err, AcquisitionError::Timeout { phase: "total" }));
assert!(elapsed < Duration::from_millis(750), "elapsed {elapsed:?}");  // lib.rs:1424
assert!(!dest.exists());
assert_eq!(fs::read_dir(&dir).unwrap().count(), 0);
```

The server stalls the body for 4000 ms. The discriminating fact is that the
fetch returns in ~250 ms rather than ~4000 ms. The 750 ms ceiling additionally
charges the test for process spawn, `kill`, `wait`, and **two reader-thread
joins** — none of which the transport's deadline contract governs, and all of
which inflate under load.

### 3.4 Tests that are already deterministic and must stay that way

An inventory of all 24 tests in the crate shows only these two use sub-second
values as **live deadlines**. The others pass sub-second values as *arguments* to
pure functions, with no subprocess and no wall clock:

- `exit_28_is_labelled_total_when_connect_and_total_ceilings_coincide`
- `connect_and_total_timeouts_are_typed`
- `duration_decimal_seconds_serializes_sub_second_inputs_truthfully`
- `build_curl_args_passes_decimal_deadlines_never_above_ceiling`

`cancellation_during_body_stream_kills_and_reaps_child` sleeps 150 ms but pairs
it with `limits()` (2 s / 10 s) and a 5000 ms stall, leaving ample headroom; it
is **not** in scope and must not be churned.

M012 must therefore be a two-test change. Broadening it to "make curl tests
robust" would put correct, deterministic tests at risk.

## 4. Invariants that must not regress

- `POLL_INTERVAL` remains 5 ms and the parent remains the enforcer of the total
  deadline.
- The parent kills and reaps the child on deadline expiry and removes the owned
  temp file (`__remove_owned_temp`) on every failure path.
- `Timeout { phase }` classification is unchanged, including the
  `exit 28` connect-vs-total discrimination at `lib.rs:746` and its refusal to
  over-claim when the two ceilings coincide.
- `CurlConfig::validate()` still rejects zero timeouts and `connect > total`.
- `build_curl_args` still emits truthful decimal seconds and never a value above
  the effective ceiling; sub-microsecond positive durations still fail
  validation rather than widening.
- `FetchLimits::effective` still yields `min(request, adapter ceiling)`.
- No production deadline is widened, no slack/tolerance is added, and
  `POLL_INTERVAL + 5 ms` stays a labeling margin, never a deadline extension.
- No retry, sleep, poll, or backoff is introduced into test infrastructure to
  mask a real deadline failure.
- A real `Timeout { phase: "total" }` is never converted into a success, and a
  genuine transport regression is never relabelled as test flakiness.
- Public API, error types, dependency graph, and published behavior are unchanged.
- `eggup-curl`'s only Eggup dependency remains `eggup-acquisition`; it must not
  learn about `eggup-eggfetch`.
- Rust 1.89 remains supported; no new dependency; no `unsafe`.
- Windows behavior is unchanged: test A stays `#[cfg(unix)]` (it needs `sh` +
  `xxd`), test B stays `#[cfg(not(windows))]`.

## 5. Scope

### In scope

- Decouple the asserted serialization from the enforced wall-clock budget in
  `build_curl_args_passes_sub_second_deadlines_to_fake_curl`.
- Replace the absolute `elapsed` ceiling in
  `sub_second_total_timeout_kills_child_promptly` with a bound that proves
  promptness structurally instead of by absolute wall clock.
- Strengthen the deterministic (no-subprocess) coverage so no end-to-end
  sub-second proof is lost in the process.
- Re-run the full crate suite repeatedly under synthetic load as evidence.
- Record the result in the closure record, the acquisition roadmap, and
  `plans/registry.md`.

### Explicitly out of scope

- Any change to production deadline enforcement, `POLL_INTERVAL`, tolerance, or
  classification logic.
- Changing `FetchLimits` defaults or `CurlConfig` ceilings.
- Changing transport semantics, redirection policy, protocol allowlist, or
  byte-limit handling.
- The other 22 tests in the crate, including the four already-deterministic
  sub-second tests and `cancellation_during_body_stream_kills_and_reaps_child`.
- Any change to `eggup-acquisition`, `eggup-eggfetch`, or any other crate.
- Publication of a new `eggup-curl` version. **A test-only change does not alter
  packaged bytes' behavior and does not require a republication**; if the
  implementer concludes otherwise, that is a separate release milestone.
- CI configuration, runner sizing, or lane scheduling.
- `CleanupDisposition` / `Error` extensibility (the second open finding), which
  belongs to a 0.2/1.0 API-boundary milestone.
- The load-sensitive-test finding for any *other* crate.

## 6. Required production/test changes

**No production change is required.** M012 is a test-only milestone. If an
implementer finds themselves editing non-`#[cfg(test)]` code, that is a signal to
stop and re-read §14.

### 6.1 Test A — decouple serialization from the enforced budget

The invariant to preserve: *a sub-second `FetchLimits` deadline is serialized
truthfully into the spawned child's argv*.

Because `validate()` permits `connect < total`, give the fetch a generous total
while keeping a **sub-second connect**, so a sub-second value is still proven
end-to-end through the real spawn path with no wall-clock ceiling:

```rust
let cfg = FetchLimits {
    connect_timeout: Duration::from_millis(250),          // sub-second, asserted
    total_timeout: Duration::from_secs(30),               // generous: not the thing under test
    max_metadata_bytes: 64 * 1024,
    max_artifact_bytes: 256 * 1024,
};
t.fetch_metadata(&req("https://example.com/x"), cfg, &CancelFlag::new())
    .expect("fake curl completes well inside the 30s budget");
assert!(argv.contains("--connect-timeout 0.25"), "argv missing 0.25: {argv}");
```

This removes the flake entirely: the parent deadline is now 30 s, so host
scheduling cannot expire it, while the assertion still proves sub-second
serialization reaches `argv`.

A sub-second `--max-time` cannot be proven end-to-end without a sub-second wall
clock — the parent deadline *is* `eff_total`. That is a real constraint, not an
oversight, and §6.2 covers it deterministically. Record this trade-off in the
closure record rather than pretending both are covered end-to-end.

### 6.2 Reinforce deterministic sub-second serialization coverage

The existing platform-neutral tests
(`duration_decimal_seconds_serializes_sub_second_inputs_truthfully`,
`build_curl_args_passes_decimal_deadlines_never_above_ceiling`) already exercise
`build_curl_args` directly with sub-second inputs and no subprocess. M012 must
make the coverage explicit rather than incidental — assert, with no subprocess
and no wall clock:

- a sub-second **connect** and sub-second **total** pair (e.g. 250 ms / 750 ms)
  serializes to `--connect-timeout 0.25` and `--max-time 0.75`;
- `--max-time` equals the *effective total* ceiling, never the raw request when
  the adapter ceiling is lower;
- sub-microsecond positive durations still yield `InvalidInput`.

This makes §6.1 strictly additive: the sub-second `--max-time` serialization
proof is retained, just at the layer where it can be deterministic.

### 6.3 Test B — prove promptness without an absolute wall clock

Replace `elapsed < 750ms` with assertions that discriminate the contract rather
than the host:

```rust
assert!(matches!(err, AcquisitionError::Timeout { phase: "total" }));

// The deadline was actually reached: the fetch did not succeed early.
assert!(elapsed >= short_total.total_timeout, "elapsed {elapsed:?}");

// The child was killed rather than left to stall: comfortably inside the
// server's 4000 ms body stall.
assert!(elapsed < Duration::from_secs(2), "elapsed {elapsed:?}");
```

Rationale for each:

- `elapsed >= total_timeout` is the half that is currently missing. Without it,
  a bound alone would also pass if the fetch returned immediately.
- `elapsed < 2s` still separates kill-from-stall by 2x (the stall is 4 s) while
  giving kill/reap/join latency 1.75 s of headroom instead of 500 ms. It proves
  the deadline fired; it does **not** claim the OS reaped within X ms.
- `!dest.exists()` and the empty-directory assertion already prove residue
  cleanup and must be retained unchanged — they are the part of this test that is
  genuinely valuable and fully deterministic.

Do **not** reintroduce a tight absolute bound. If the implementer wants a tight
promptness proof, the correct route is the optional seam in §6.4 — not a
smaller wall-clock constant.

The `eff_connect = 100 ms` / `eff_total = 250 ms` separation was inspected and
is retained. The server sends headers before stalling, so the connect phase
completes immediately and `elapsed` cannot fall under
`eff_connect + tolerance` (110 ms); `lib.rs:746` therefore classifies as
`total` correctly. M012 must not "fix" this by widening values.

### 6.4 Optional private deadline seam

If the implementer wants a deterministic promptness proof, mirror M008's
discipline: add a **private**, `#[cfg(test)]`-visible helper that evaluates
deadline expiry at an injected `Instant`, keeping the production loop on the
real monotonic clock. It must stay private to `eggup-curl` and must not change
any production call site's behavior.

This is **optional**. The §6.1/§6.3 decoupling is sufficient to close the
finding. Adding the seam is only justified if §6.3's bound is judged too loose to
prove promptness, and it must be justified in the closure record.

## 7. Ordered work packages

1. Record the pre-change state: baseline `9326730`, the 3/40 reproduction, and
   the exact panic text, in a scratch note for the closure record.
2. Apply §6.1 to test A.
3. Apply §6.2 — make sub-second serialization coverage explicit in the
   deterministic tests.
4. Apply §6.3 to test B.
5. Decide §6.4: skip it, or add the private seam and justify it.
6. Run the two tests 100× sequentially, unloaded, and confirm 0 failures.
7. Run the two tests 40× sequentially under synthetic CPU load
   (`yes > /dev/null` × 2× core count, or equivalent) and confirm 0 failures.
   **This is the discriminating check** — §15 requires it, and M008's precedent
   explicitly forbids closing on an unloaded rerun.
8. Run `cargo test -p eggup-curl --all-targets --all-features --locked`.
9. Run the full local gate.
10. Push and require a fresh hosted Stable/MSRV/macOS/Windows matrix.
11. Write `plans/closure/acquisition-transport/012-status.md`.
12. Reconcile the acquisition roadmap, `plans/registry.md` (including removing
    open finding 1), and `architecture/tooling-governance.md` if a new drift row
    is warranted.

## 8. Failure, cancellation, restart, and contention semantics

No production failure semantics may change.

- A parent deadline expiry remains a hard `Timeout { phase: "total" }` after
  `kill` + `wait` + both reader-thread joins + `__remove_owned_temp`.
- M012 must not retry a timed-out fetch, renew a deadline, clamp an expired
  deadline to a positive duration, or treat a first timeout as a retryable
  condition.
- Test infrastructure must not add retry loops, sleeps, or polling that could
  hide a real deadline failure. The §7 step 7 load loop runs the **test binary**
  repeatedly; it must not be implemented as a retry inside the test.
- Both tests must remain independent: neither may depend on the other having run
  first, and neither may leave a stale `temp_dir` that perturbs a later run.

## 9. Compatibility and migration

No public API, no consumer migration, no dependency or feature change.

`eggup-curl` remains at `0.1.2`. Because M012 changes only `#[cfg(test)]` code,
the packaged library is behaviorally and API-wise identical; **no republication
is authorized or implied by this milestone.**

If the implementer concludes the change alters packaged bytes or public
behavior — it should not — stop and treat it as a publication milestone per
`.opencode/skills/release-workflow`.

## 10. Required tests

At minimum:

- test A asserts `--connect-timeout 0.25` appears in the recorded child argv;
- test A completes without depending on a sub-second wall-clock budget;
- §6.2 deterministic coverage asserts a sub-second connect **and** total pair
  serializes to `0.25` / `0.75`;
- §6.2 deterministic coverage asserts `--max-time` never exceeds the effective
  ceiling;
- §6.2 deterministic coverage retains sub-microsecond `InvalidInput` rejection;
- test B asserts `Timeout { phase: "total" }`;
- test B asserts `elapsed >= total_timeout`;
- test B asserts `elapsed` is far below the 4000 ms stall;
- test B retains `!dest.exists()` and the empty-directory residue check;
- the four already-deterministic sub-second tests are unchanged and green;
- `cancellation_during_body_stream_kills_and_reaps_child` unchanged and green;
- full `eggup-curl` suite green (24 tests), sequential and default-threaded;
- full workspace all-target/all-feature suite green;
- Rust 1.89 all-target check green;
- hosted macOS and Windows lanes execute and pass both tests.

No test in M012 may require a sub-second wall-clock budget to prove a
serialization or classification invariant.

## 11. Required verification commands

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p eggup-curl --all-targets --all-features --locked
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo +1.89.0 check --workspace --all-targets --locked
git diff --check
```

Required discriminating loop (must show **0** failures). Build the test binary
first so compilation is not measured:

```text
cargo test -p eggup-curl --lib --no-run --locked
BIN=$(ls -t target/debug/deps/eggup_curl-* | grep -v '\.d$' | head -1)

# unloaded
for i in $(seq 1 100); do "$BIN" build_curl_args_passes_sub_second_deadlines_to_fake_curl --test-threads=1 || exit 1; done
for i in $(seq 1 100); do "$BIN" sub_second_total_timeout_kills_child_promptly --test-threads=1 || exit 1; done

# loaded
for j in $(seq 1 $(( $(sysctl -n hw.ncpu) * 2 ))); do yes > /dev/null & done
trap 'kill %1 %2 %3 %4 %5 %6 %7 %8 %9 %10 %11 %12 /dev/null 2>/dev/null' EXIT
for i in $(seq 1 40); do "$BIN" build_curl_args_passes_sub_second_deadlines_to_fake_curl --test-threads=1 || exit 1; done
for i in $(seq 1 40); do "$BIN" sub_second_total_timeout_kills_child_promptly --test-threads=1 || exit 1; done
```

The loaded loop is mandatory. M012 **MUST NOT** close on an unloaded rerun —
that is precisely the error this finding came from, since hosted CI is never
loaded this way and therefore never reproduced it.

Hosted closure requires:

- Stable Linux full checks;
- Rust 1.89 MSRV all-target check;
- macOS full workspace tests;
- Windows archive/acquisition/curl/service tests + workspace all-target check.

## 12. Documentation updates

Update:

- `plans/subsystems/acquisition-transport-roadmap.md` — add M012, its status
  table row, and reconcile the header `Status:` line;
- `plans/registry.md` — register M012 and **remove open finding 1** from
  "Post-0.1.3 open findings", leaving only the `CleanupDisposition` item;
- `plans/closure/acquisition-transport/012-status.md` — new closure record;
- `architecture/tooling-governance.md` — only if a new `Known doc/code drift`
  row is warranted.

A crate changelog entry is **not** required: M012 changes no packaged behavior
and publishes nothing. If one is added, it must be filed as test/qualification
hardening and must not imply a transport behavior change or a new release.

Per `docs-hygiene`, closure status is owned by the closure record's `Status:`
line; roadmap and registry cells must not contradict it.

## 13. Acceptance criteria

M012 closes only when:

- test A no longer depends on a sub-second wall-clock budget to pass;
- test B no longer asserts an absolute sub-second `elapsed` ceiling;
- sub-second serialization coverage for **both** connect and total is retained,
  deterministically;
- the loaded loop in §11 shows 0 failures for both tests across 40 iterations;
- the unloaded loop shows 0 failures across 100 iterations;
- no production code changed (`git diff` touches only `#[cfg(test)]` code in
  `crates/eggup-curl/src/lib.rs`);
- `cargo tree -p eggup-curl` still shows only `eggup-acquisition` as an Eggup
  dependency;
- a fresh hosted Stable/MSRV/macOS/Windows matrix is green;
- the pre-change 3/40 reproduction is recorded as superseded evidence;
- the closure record states plainly that the `TooLarge { limit: 0 }`-style
  environment limits and Windows live-curl limitation from M007 still apply —
  M012 does not close them;
- `plans/registry.md` no longer carries open finding 1.

## 14. Stop conditions

Stop and write a broader acquisition corrective if:

- the loaded loop still fails after §6.1/§6.3 — that indicates a genuine
  production deadline defect, not test flakiness;
- separating the budget from the assertion turns out to require widening a
  production deadline or adding production tolerance;
- a deterministic re-read of the code shows `classify_curl_result` can mislabel a
  total timeout as `connect` at these durations;
- the parent deadline is found not to be the effective enforcer for real `curl`,
  which would invalidate the §3.1 analysis;
- the fake-curl child is found to honor `--max-time` after all, which would mean
  the flake has a different cause;
- test A cannot be made deterministic without losing end-to-end sub-second
  proof, and §6.2 does not recover it deterministically.

Do not classify a genuine runtime failure as test flakiness merely to restore a
green baseline.

## 15. Closure evidence required

Record:

- implementation SHA and the exact test diff;
- proof no production (non-`#[cfg(test)]`) code changed;
- `cargo tree -p eggup-curl` before/after, showing no new dependency edge;
- the pre-change reproduction: 3/40 failures at load ~62–65 with the verbatim
  panic text and observed `0.76–0.77s` durations;
- post-change unloaded results: 100/100 both tests;
- post-change **loaded** results: 40/40 both tests, with the load generation
  method and observed loadavg recorded;
- exact local command results for every command in §11;
- new hosted run ID with per-platform job conclusions;
- the §6.1 trade-off stated explicitly: sub-second `--max-time` is proven
  deterministically at the `build_curl_args` layer, not end-to-end, and why;
- the §6.4 decision (skipped or added) with justification;
- retained limitations: M007's Windows live-loopback limitation, and the fact
  that test A remains `#[cfg(unix)]`;
- unresolved findings by severity.

## 16. Handoff notes

Keep this milestone small and test-only.

The intended proof is not "750 ms usually lasts long enough." The intended
proofs are:

- a sub-second `FetchLimits` deadline reaches the child's argv as a truthful
  decimal, proven with a generous parent budget and no wall-clock race;
- a sub-second total deadline is enforced by the parent, reported as
  `Timeout { phase: "total" }`, and leaves no residue — proven by comparing
  elapsed against the deadline and against the server stall, not against a
  constant.

Do not change production timeout policy to make a unit test pass. If the fix
feels like it needs a wider deadline, a retry, or a tolerance, the analysis in
§3 is wrong — re-derive it and stop rather than patching the symptom.