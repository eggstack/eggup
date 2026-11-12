# Verified Update Core — Milestone 018 closure record

## Status

**closed** — the D1 rule is implemented and pinned, the D2 and D3 rustdoc
corrections are in place, and the D4/D5/D6 record corrections are made. The
native evidence for this milestone is **macOS only**; the hosted qualification
that covers Linux and Windows was run by the parent session in hosted run
`40067964510` (commit `a431aaa`), which is green on all five lanes and is
recorded in the parent-session verification addendum at the end of this
record. Finding F1 is closed on that evidence.

No medium-or-higher finding about this milestone's own change remains open.
One medium finding remains open about the *publication* of this change; it is
named as a follow-on in F2 and is not a defect of this milestone.

## Source plan and roadmap

- Plan: `plans/implementation/verified-update-core/018-bounded-stdin-outcome-priority-corrective.md`
- Roadmap: `plans/subsystems/verified-update-core-roadmap.md`
- Long-term requirements: `plans/000-long-term-specification.md`,
  `plans/001-terminology-and-domain-model.md`
- Applicable ADRs: `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`,
  `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md`

## Reviewed repository baseline

- Baseline commit: `bf439f519300354ac14823e74acd48c7995033a6`
- Branch: `plans/corrective-batch-2026-10-23`
- The tree was **clean** at the baseline: `git status --short` printed nothing
  before this milestone's first edit.
- **Other agents wrote to the same branch while this milestone was running.**
  During the milestone, `architecture/**`, `crates/eggup-acquisition/**`,
  `crates/eggup-curl/**`, `crates/eggup-eggfetch/**`,
  `crates/eggup-service/**`, `docs/**`, `scripts/check-docs-truth.sh`,
  `AGENTS.md`, the root `CHANGELOG.md`, `.opencode/skills/**`,
  `plans/registry.md`, `plans/subsystems/**`, and several
  `plans/{implementation,closure}/**` records of other subsystems became
  modified. Those changes are **not** part of this milestone and were not
  touched by it: `git status --short` was read before and after every edit,
  and the only paths this milestone wrote are the ones listed in the table
  below.

## Implementation commits / PRs

**None by this milestone.** This milestone ran no git command that mutates
repository state (`git stash`, `git checkout <path>`, `git restore`,
`git commit`, `git push`, `git tag`, `git clean`, `git reset`, and branch
creation were all forbidden and none was executed). The parent session owns
every commit, every push, and every tag in this batch, and it has carried the
change on the branch:

- `640e97e` — the production change (`candidate.rs`), the `FailureCategory`
  rustdoc, the crate documents, and the fixture file with the bounded marker
  wait;
- `3d91ab1` — this plan, this closure record, the M015 and M017 addenda, the
  registry, and the roadmap;
- `a431aaa` — the sweep's two-millisecond ladder step and its coverage
  counters; this is also the commit the hosted run qualified;
- `896b564` — the hosted-qualification addendum at the end of this record;
- `d9428fe` — the re-derived anchors in this record and the load-sensitivity
  requirement in the plan.

Files this milestone created or changed:

| File | Change |
| --- | --- |
| `crates/eggup-core/src/candidate.rs` | Production change: the stamped writer verdict, the verdict classification against the deadline, the deadline's computation point, the final drain of the writer's verdict, the `run_bounded` rustdoc (the rule, the kill-and-pipe caveat, the outcome list), and the unit tests that pin the rule. |
| `crates/eggup-core/src/transaction.rs` | The `FailureCategory` rustdoc's producer count (D3). |
| `crates/eggup-core/tests/candidate_stdin.rs` | **New fixtures**: one helper role, three fixtures, one helper function, and the sweep ladder. |
| `crates/eggup-core/CHANGELOG.md` | The M018 `Unreleased` entry, and the correction of M017's two overstated bullets. |
| `crates/eggup-core/README.md` | The bounded-runner paragraph: the write failure's deadline qualification. |
| `crates/eggup-core/docs/verification.md` | The candidate-command paragraph: the rule and the pointer at the normative rustdoc. |
| `plans/implementation/verified-update-core/018-bounded-stdin-outcome-priority-corrective.md` | **New.** The plan. |
| `plans/closure/verified-update-core/018-status.md` | **New.** This record. |
| `plans/closure/verified-update-core/015-status.md` | **Dated addendum**: U1 closed with its evidence, and the status wording reconciled. |
| `plans/closure/verified-update-core/017-status.md` | **Dated addendum**: the superseded statement corrected and pointed at M018. |

No file outside `crates/eggup-core/**` and `plans/{implementation,closure}/verified-update-core/`
was written. In particular `architecture/**`, `plans/registry.md`,
`plans/subsystems/**`, the root `CHANGELOG.md`, and every other crate were not
touched — see the documentation table and the appendix below.

## Executive finding

`run_bounded`'s bounded stdin write published a rule the implementation did not
apply. The published rule was that a write failure detected **after** the
deadline was reached does not retroactively turn the deadline outcome into an
`Err`, because the deadline already decided that outcome. The implementation
evaluated the stdin-error arm **before** the deadline arm inside the poll loop,
and the writer's verdict carried no instant at all, so "detected before or
after the deadline" was decided by the runner's poll phase rather than by a
fact about the write. The same input and the same timeout therefore produced
two different public classifications of one logical scenario.

M018 fixes this by making the classification a fact:

- the writer stamps its verdict with the instant at which it observed the
  write's completion (`candidate.rs:376-383`);
- one helper classifies that verdict against the deadline
  (`candidate.rs:562-573`), and both the poll loop (`candidate.rs:413-436`)
  and the final drain (`candidate.rs:479-494`) use it;
- a write failure observed **before** the deadline is an `Err` and fails
  closed, exactly as before;
- a write failure observed **at or after** the deadline is the deadline
  outcome — `Ok` with `timed_out` set — because the deadline already decided
  it;
- a writer that had not delivered its verdict when the deadline was reached is
  abandoned, exactly like a reader blocked on a pipe a surviving descendant
  holds.

Two neighbouring statements were corrected rather than implemented: the
kill-and-pipe sentence in the same paragraph overstated what the kill does
when a descendant inherited the pipe (D2), and `FailureCategory`'s rustdoc
counted six producers for a mapping that produces five in a shipped build
(D3).

The public surface is unchanged. `Error` keeps exactly the seven published
variants, `FailureCategory` keeps its members and its published `as_str()`
strings, no public item was added or removed, the dependency surface is
unchanged, and no publication was performed.

## The rule, stated precisely

> **The deadline has priority over the stdin writer's verdict, and the verdict
> is classified by the instant at which the writer observed the write's
> completion — not by the instant at which the runner polled for it.**
>
> - a write failure observed **before** the deadline is an `Err`: the payload
>   was not delivered, the child is killed and reaped, and the call fails
>   closed;
> - a write failure observed **at or after** the deadline does not turn the
>   deadline outcome into an `Err`: the deadline already decided that outcome,
>   and the call returns `Ok` with `timed_out` set;
> - a writer that had not delivered its verdict when the deadline was reached
>   is abandoned, exactly like a reader blocked on a pipe a surviving
>   descendant holds; the deadline outcome stands and the verdict is never
>   observed;
> - a payload that was not delivered is never reported as a success: the
>   outcome is either an `Err` or an `Ok` whose `timed_out` flag is set.

### The alternative that was rejected

**Rejected: keep the write failure as an unconditional `Err` and rewrite the
rustdoc to match the code** — that is, state that a write failure is always an
`Err`, even when it is detected after the deadline.

It was rejected for two reasons.

1. It leaves the outcome dependent on the poll phase. The runner can only
   classify a verdict it observed; a verdict that arrives after the deadline
   arm has broken the loop is never observed at all. With the write failure
   made unconditional, the same logical scenario would still be reported as an
   `Err` in some runs and as the deadline outcome in others — the flip M018
   exists to remove, only with the documentation rewritten to match it.
2. It discards the deadline's unconditional priority, which is the safer
   direction and the one M014 and M017 already published.

**Also rejected: move the deadline arm ahead of the stdin arm without a
stamp.** "Observed before the deadline" would still be a poll-phase fact: a
verdict observed in the poll that follows the deadline would be classified as a
pre-deadline failure, so the classification would still flip on poll timing,
and a write failure that happened *before* the deadline but was observed after
it would be lost behind the deadline outcome.

**The trade-off the chosen rule accepts.** The rule cannot report a write
failure that the runner never observed, and it cannot distinguish "the write
failed after the deadline" from "the write failed before the deadline but the
runner learned of it later" when the verdict's delivery races the deadline
arm. Both residuals are bounded by the runner's poll interval and are stated
in the rustdoc rather than hidden. The alternative — reporting the failure
unconditionally — trades that bounded residual for an unbounded one: a
classification that depends on when the runner happened to look.

## Requirement → evidence matrix

| # | Requirement (plan §6 / §10) | Evidence | Command and result |
| --- | --- | --- | --- |
| R1 | The writer's verdict carries the instant at which the writer observed the write's completion | `crates/eggup-core/src/candidate.rs:376-383` (the writer thread stamps `observed_at` with `Instant::now()` after the write and the flush) | `cargo clippy -p eggup-core --all-targets --locked --offline -- -D warnings` → exit 0; `cargo test -p eggup-core --locked --offline` → 110 passed, 0 failed |
| R2 | The verdict's classification against the deadline has exactly one implementation | `crates/eggup-core/src/candidate.rs:536-573` (`WriterVerdict`, `WriterVerdictClass`, `classify_writer_verdict`); used at `:413` and `:486` | `cargo clippy … -- -D warnings` → exit 0; `cargo doc -p eggup-core --no-deps --locked --offline` → exit 0 |
| R3 | A write failure observed **before** the deadline is an `Err` with the operation string `"writing bounded stdin"`, and the child is killed and reaped | `crates/eggup-core/src/candidate.rs:415-426`; `crates/eggup-core/src/candidate.rs:1091-1111` (unit pin `candidate_writer_verdict_before_the_deadline_fails_closed`); `crates/eggup-core/tests/candidate_stdin.rs:537` (`candidate_stdin_write_failure_before_the_deadline_is_an_error`) | `cargo test -p eggup-core --locked --offline --lib candidate_writer_verdict` → **3 passed; 0 failed**; `cargo test -p eggup-core --locked --offline --test candidate_stdin` → **14 passed; 0 failed** |
| R4 | A write failure observed **at or after** the deadline is the deadline outcome: `Ok` with `timed_out` set, never an `Err` | `crates/eggup-core/src/candidate.rs:427-436`; `crates/eggup-core/src/candidate.rs:1113-1131` (unit pin `candidate_writer_verdict_after_the_deadline_does_not_fail_closed`); `crates/eggup-core/tests/candidate_stdin.rs:574` (`candidate_stdin_write_failure_at_or_after_the_deadline_is_not_an_error`) | `cargo test -p eggup-core --locked --offline --lib candidate_writer_verdict` → **3 passed; 0 failed**; `cargo test -p eggup-core --locked --offline --test candidate_stdin` → **14 passed; 0 failed** |
| R5 | An undelivered payload is never reported as a success | `crates/eggup-core/src/candidate.rs:255-259` (the outcome list's deadline entry); `crates/eggup-core/tests/candidate_stdin.rs:594` and `:509` (both assert `!output.success()`) | `cargo test -p eggup-core --locked --offline --test candidate_stdin` → **14 passed; 0 failed** |
| R6 | The classification is stable across the deadline: a sweep over the child's lifetime never reports an undelivered payload as a success, a reported write failure comes from a child that exited on its own, and a child killed at the deadline reports the deadline outcome | `crates/eggup-core/tests/candidate_stdin.rs:631` (`candidate_stdin_write_failure_classification_is_stable_across_the_deadline`), 14 ladder samples at `candidate_stdin.rs:61`, `:73`, `:76` | `cargo test -p eggup-core --locked --offline --test candidate_stdin` → **14 passed; 0 failed** (the sweep alone: 14 samples, 1.63 s) |
| R7 | The deadline is fixed before any thread exists, so "the deadline starts once the spawn has returned" is literally true | `crates/eggup-core/src/candidate.rs:335-340` | `cargo clippy … -- -D warnings` → exit 0; `cargo test -p eggup-core --locked --offline` → 110 passed |
| R8 | A write failure the loop never saw is not lost: the final drain classifies it with the same rule | `crates/eggup-core/src/candidate.rs:478-495` | `cargo test -p eggup-core --locked --offline --test candidate_stdin` → **14 passed; 0 failed** |
| R9 | The published precedence rule survives: a write failure wins over a concurrent output overflow | `crates/eggup-core/src/candidate.rs:488-491` (the drain's `FailedBeforeDeadline` arm sets `stdin_error`) and `:415-426` (the loop's) | `cargo test -p eggup-core --locked --offline` → 110 passed, 0 failed |
| R10 | The rustdoc states the rule literally | `crates/eggup-core/src/candidate.rs:207-236` ("# The deadline and a stdin write failure") | `cargo doc -p eggup-core --no-deps --locked --offline` → exit 0, no rustdoc warning; the section renders |
| R11 | The kill-and-pipe sentence carries the descendant caveat (D2) | `crates/eggup-core/src/candidate.rs:197-206` | `cargo doc -p eggup-core --no-deps --locked --offline` → exit 0 |
| R12 | `FailureCategory`'s rustdoc counts five producers in a shipped build and names the sixth as test-only (D3) | `crates/eggup-core/src/transaction.rs:110-120` | `cargo doc -p eggup-core --no-deps --locked --offline` → exit 0; `cargo clippy … -- -D warnings` → exit 0 |
| R13 | The M015 closure record's finding U1 is closed with its evidence, and its status agrees with the registry and the roadmap (D4, D5) | `plans/closure/verified-update-core/015-status.md` — the dated addendum at the end of the record | Read-only verification of commit `ab90688`: `git show ab90688 -- crates/eggup-core/src/error.rs` shows the rustdoc for `Result`, `Error`, `RecoveryError`, and `RecoveryResult` added by that commit |
| R14 | The M017 closure record's restatement of the false rule is superseded (D6) | `plans/closure/verified-update-core/017-status.md` — the dated addendum at the end of the record | Read-only verification against the baseline and this milestone's rustdoc |
| R15 | The existing core test suite still passes, with no assertion weakened or deleted | `crates/eggup-core/tests/candidate_deadline.rs` and the pre-existing `crates/eggup-core/tests/candidate_stdin.rs` fixtures are unmodified (`git status --short` shows only additions to the latter) | `cargo test -p eggup-core --locked --offline` → **110 passed; 0 failed** across 6 test binaries (56 → 59 in the lib binary: the three new unit tests; 10 → 14 in `candidate_stdin`: one helper role and three fixtures) |
| R16 | No new dependency edge | `crates/eggup-core/Cargo.toml` is unmodified | `cargo tree -p eggup-core --locked --offline --depth 1` → `eggup-core v0.1.3` → `sha2 v0.10.9` only |
| R17 | `Error` keeps exactly the seven published variants; `FailureCategory` keeps its members | `crates/eggup-core/src/error.rs` is unmodified (`git status --short` shows no change to it) | `cargo clippy … -- -D warnings` → exit 0; `cargo test -p eggup-core --locked --offline` → 110 passed |
| R18 | The crate's own documents state the rule | `crates/eggup-core/CHANGELOG.md` (the M018 entry and the corrected M017 bullets), `crates/eggup-core/README.md:45-57`, `crates/eggup-core/docs/verification.md:32-47` | `cargo doc -p eggup-core --no-deps --locked --offline` → exit 0 |
| R19 | The new fixtures do not fail on a busy host: a helper that never starts is a distinct, reported condition, and no fixture asserts that a helper started | `crates/eggup-core/tests/candidate_stdin.rs:84` (`MARKER_WAIT`), `:154` (`wait_for_marker`), applied at `:522`, `:604`, and `:652`; the distinct "the helper never started" branch at `:653-655` and the coverage note at `:710-718` | `cargo test -p eggup-core --locked --offline --test candidate_stdin -- --nocapture` → **14 passed; 0 failed**, note: "the classification sweep took 14 samples: 9 observed a write failure before the deadline, 1 observed one at or after the deadline, 4 were decided at the deadline, and 0 had a helper binary that did not start before the sweep deadline"; three consecutive full-crate runs under a concurrent cold workspace compile and one under four concurrent cold compiles → **110 passed; 0 failed** (see "Load-sensitivity reproduction and fix") |

## Production implementation evidence

### The stamped verdict

`crates/eggup-core/src/candidate.rs:376-383`:

```rust
let result = pipe.write_all(&payload).and_then(|()| pipe.flush());
// The verdict carries the instant at which the writer observed the write's
// completion, so that "before or after the deadline" is a fact about the
// write and not about when this call polls for it.
let verdict = WriterVerdict {
    observed_at: Instant::now(),
    result,
};
let _ = stdin_tx.send(verdict);
```

The stamp is taken immediately after the write and the flush complete, so it is
the instant at which the writer observed the write's outcome. The channel
carries the stamped verdict instead of a bare `io::Result<()>`, and nothing
else in the crate observes a writer verdict.

### The classification

`crates/eggup-core/src/candidate.rs:562-573`:

```rust
fn classify_writer_verdict(verdict: WriterVerdict, deadline: Instant) -> WriterVerdictClass {
    match verdict.result {
        Ok(()) => WriterVerdictClass::Delivered,
        Err(source) if verdict.observed_at < deadline => {
            WriterVerdictClass::FailedBeforeDeadline(source)
        }
        Err(_) => WriterVerdictClass::FailedAtOrAfterDeadline,
    }
}
```

The comparison is `observed_at < deadline`: a failure observed at the deadline
instant itself is a failure the deadline had already decided.

### The loop

`crates/eggup-core/src/candidate.rs:413-436`. The stdin arm no longer decides
on its own; it classifies the verdict:

```rust
if let Some(verdict) = stdin_result.take() {
    match classify_writer_verdict(verdict, deadline) {
        WriterVerdictClass::Delivered => {}
        WriterVerdictClass::FailedBeforeDeadline(source) => { /* fail closed */ }
        WriterVerdictClass::FailedAtOrAfterDeadline => { /* the deadline decided */ }
    }
}
```

### The final drain

`crates/eggup-core/src/candidate.rs:479-494`. The pre-M018 code never polled
the writer again after the loop, so a verdict that arrived at the bound was
lost. The drain polls once more — guarded by the same "the writer already
delivered" flag the loop uses, because after the writer has delivered the slot
is empty and the channel is disconnected, and a second poll would mistake that
for a writer that ended without a verdict — and classifies the verdict with the
same helper. A failure observed before the deadline therefore wins over
whatever the loop decided (including an output overflow, which is the
published precedence rule); a failure observed at or after the deadline leaves
the loop's decision untouched.

The drain deliberately does **not** set `timed_out`: a post-deadline failure
means the deadline had already decided the outcome, and the loop had already
recorded that decision. Setting the flag here could produce a result with both
`timed_out` and `output_limited` set, which is not one of the four documented
outcomes.

### The deadline's computation point

`crates/eggup-core/src/candidate.rs:335-340`. The deadline is now computed
immediately after the spawn returns, before the pipes are taken and before any
thread exists. The published sentence "the deadline starts once the spawn has
returned" is now literally true, and the instant a writer observes its write
can be compared against a deadline that was already decided. The deadline
moved earlier by the cost of taking the pipes and spawning three threads
(microseconds); the bound itself is unchanged.

### Fixture discrimination proof

The rule is pinned by a **unit test** on the classification helper, because
the classification is the rule. The pre-M018 code has no stamped verdict and
therefore cannot express the rule at all: the rule's subject — "was the
failure observed before or after the deadline" — did not exist in the code.

**Mutation 1 — the observation instant is ignored** (the classification is
changed to `Err(source) => WriterVerdictClass::FailedBeforeDeadline(source)`,
that is, every observed failure is treated as a pre-deadline failure, which is
the pre-M018 classification):

```text
cargo test -p eggup-core --locked --offline --lib candidate_writer_verdict
→ test result: FAILED. 2 passed; 1 failed; 0 ignored
   FAILED candidate::tests::candidate_writer_verdict_after_the_deadline_does_not_fail_closed

cargo test -p eggup-core --locked --offline --test candidate_stdin
→ test result: FAILED. 11 passed; 3 failed; 0 ignored
   FAILED candidate_stdin_deadline_bounds_a_child_that_never_reads_stdin
   FAILED candidate_stdin_write_failure_at_or_after_the_deadline_is_not_an_error
   FAILED candidate_stdin_write_failure_classification_is_stable_across_the_deadline
```

The failure message of the unit test is the rule itself: *"a failure observed
at or after the deadline must not fail closed"*. The mutation was applied,
observed, and reverted **twice** — once before the load-sensitivity fix and
once after it — and produced the same four failures both times. The final tree
does not contain it.

**What the pre-M018 tree does against the new fixtures, measured.** With the
pre-M018 behaviour restored exactly (no drain and no stamp), the new
integration fixtures pass. The measurement was repeated **after** the
load-sensitivity fix, against the current fixture file and the current ladder,
in a copy of the workspace outside the repository (`/tmp/eggup-preM018`); the
repository's own tree was not modified and no git state was mutated:

```text
git show bf439f5:crates/eggup-core/src/candidate.rs \
  > /tmp/eggup-preM018/crates/eggup-core/src/candidate.rs
cd /tmp/eggup-preM018 && cargo test -p eggup-core --locked --offline --test candidate_stdin -- --nocapture
→ test result: ok. 14 passed; 0 failed   (finished in 2.00s)
   note: the classification sweep took 14 samples: 5 observed a write failure before the deadline,
   1 observed one at or after the deadline, 8 were decided at the deadline, and 0 had a helper
   binary that did not start before the sweep deadline
```

An earlier measurement of the same question, with the pre-load-fix ladder,
gave the same answer: 14 passed, 0 failed, three consecutive runs.

This is reported rather than hidden, because it is a property of the defect
and not of the fixtures. The observable difference between the pre-M018 tree
and this milestone is confined to a window of **at most one poll interval**
(2 ms) around the deadline: the pre-M018 tree reports the write failure as an
`Err` only when it observed the verdict before its deadline arm ran, which for
a write failure that happened *after* the deadline is a race between the
verdict's delivery and the poll grid. A measurement on this host, sweeping
the child's lifetime across the deadline in 5 ms steps, put the transition
between the 50 ms and 55 ms steps of a 60 ms deadline, and the 2 ms window
around the transition is where the two builds differ. The fixtures' ladder
(two-millisecond steps over a 120 ms deadline) does not land in that window
reliably enough to be a deterministic red, and a fixture that asserts a tight
millisecond outcome is exactly what this repository's plan template forbids
(§14).

**The deterministic pin is therefore the unit test**, which fails against any
build that classifies a verdict without its observation instant. The
integration fixtures pin the rule's observable halves and the
never-a-false-success invariant end to end.

## Load-sensitivity reproduction and fix

**What was wrong.** The sweep fixture read the helper's start marker
immediately after its bounded call returned. On a loaded host the helper
binary can start *after* the sweep's deadline, be killed at the deadline
before it writes anything, and never produce the marker — so the fixture
failed on a marker assertion even though the classification it exists to
check was correct. The failure was a property of the host's load, not of the
rule. This is the load-sensitive-test class the registry records as finding 1.

**The parent session's observation of the same class.** The hosted run
`40067739294` (commit `3d91ab1`) failed the `stable` lane on the
*pre-existing* core unit test `bounded_candidates_are_exact_and_environment_is_cleared`,
with `ETXTBSY` on the script the fixture had just written; the parent session
recorded it as a re-observation of registry finding 1 (see the parent-session
verification addendum below). The sweep's marker assertion was the same class
of failure, observed locally.

**The fix.** The bounded wait landed in commit `640e97e`; the ladder step and
the coverage counters landed in `a431aaa`. Both are ancestors of the hosted
run's commit `a431aaa`.

- a bounded wait replaces the immediate read: `MARKER_WAIT` (100 ms,
  `candidate_stdin.rs:84`) and `wait_for_marker()` (`:154`) poll for the
  marker for a bounded time;
- "the marker never appeared" became a **distinct condition** instead of a
  failure: the two single-sample fixtures then assert only the deadline
  outcome (`:522-527`, `:604-609`), and the sweep counts those samples
  separately (`:653-655`) and reports them (`:710-718`). **No semantic
  assertion was weakened**: the operation string, the `timed_out` flag, and
  `!output.success()` are asserted on every sample exactly as before;
- the sweep's deadline was raised from the 40 ms it was first written with to
  120 ms (`:61`), and the ladder was thinned from one-millisecond steps to
  two-millisecond steps (`:73`, `:76`), so the sweep takes 14 samples instead
  of 27 and still crosses the deadline on both sides, with less pressure on the
  host's startup latency.

**Re-verification under load.**

```text
# three consecutive full-crate runs with one concurrent cold workspace compile
CARGO_TARGET_DIR=/tmp/eggup-l2$i cargo build --workspace --locked --offline   # background
cargo test -p eggup-core --locked --offline                                   # → 110 passed; 0 failed  (× 3)

# extreme load: four concurrent cold workspace compiles, then the whole suite
cargo test -p eggup-core --locked --offline --no-fail-fast                    # → 110 passed; 0 failed
cargo test -p eggup-core --locked --offline --test candidate_stdin candidate_stdin_write_failure_classification -- --nocapture
→ note: the classification sweep took 14 samples: 10 observed a write failure before the deadline,
  1 observed one at or after the deadline, 3 were decided at the deadline, and 0 had a helper binary
  that did not start before the sweep deadline
```

The sweep's split between "decided at the deadline" and "a write failure
observed before the deadline" varies with the host's timing — the idle run
reported 9 / 1 / 4 / 0, the extreme-load run 10 / 1 / 3 / 0 — which is why
the fixture asserts properties that hold on both sides and reports its
coverage instead of pinning a millisecond outcome.

Two **pre-existing** lib tests failed in two earlier extreme-load runs —
`bounded_candidates_are_exact_and_environment_is_cleared` and
`cross_member_identity_requires_bundle_agreement` — both with
`timed_out: true` after their 5 s default timeout. Neither attaches a stdin
payload, so M018's change is inert for them; neither reproduced in the later
runs, and both belong to the same load-sensitive class as registry finding 1.
They are recorded here rather than dropped.

## Exact commands run and results

All commands were run from `/Users/davidbowman/projects/eggup` on the baseline
tree `bf439f519300354ac14823e74acd48c7995033a6` with the change in the working
tree. Toolchain: `rustc 1.89.0 (29483883e 2025-08-04)` / `cargo 1.89.0`,
macOS 25.6.0 (Darwin), arm64 — the workspace MSRV toolchain.

### 1. `cargo fmt -p eggup-core -- --check`

```text
EXIT=0
```

### 2. `cargo clippy -p eggup-core --all-targets --locked --offline -- -D warnings`

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.11s
EXIT=0
```

The crate-local `clippy::missing_errors_doc` is enabled for `eggup-core`, so
this run also proves that the changed public surface still carries the required
rustdoc.

### 3. `cargo test -p eggup-core --locked --offline`

```text
running 59 tests
test result: ok. 59 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.74s
running 12 tests
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.04s
running 14 tests
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.63s
running 11 tests
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.75s
running 14 tests
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**110 passed, 0 failed** across 6 test binaries. The lib binary went from 56 to
59 tests (the three new unit tests) and the `candidate_stdin` binary from 10 to
14 (one helper role and three fixtures). `candidate_deadline.rs` (11 tests)
and the other binaries are unmodified.

### 4. `cargo test -p eggup-core --locked --offline candidate`

```text
running 5 tests
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 54 filtered out; finished in 0.92s
running 12 tests
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.10s
running 14 tests
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.10s
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.46s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.00s
```

**32 passed, 0 failed.** The 5-test binary is the lib binary filtered on
`candidate` (the three new unit tests plus two pre-existing ones); the 14-test
binary is the whole `candidate_stdin` fixture file.

### 5. `cargo test -p eggup-core --locked --offline --test candidate_stdin`

```text
running 14 tests
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.64s
```

With `--nocapture`, the sweep's coverage note is visible:

```text
note: the classification sweep took 14 samples: 9 observed a write failure before the deadline,
1 observed one at or after the deadline, 4 were decided at the deadline, and 0 had a helper binary
that did not start before the sweep deadline (those samples prove only the deadline outcome).
```

### 6. `cargo test -p eggup-core --locked --offline --lib candidate_writer_verdict`

```text
test candidate::tests::candidate_writer_verdict_success_is_a_delivery ... ok
test candidate::tests::candidate_writer_verdict_before_the_deadline_fails_closed ... ok
test candidate::tests::candidate_writer_verdict_after_the_deadline_does_not_fail_closed ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 56 filtered out; finished in 0.00s
```

### 7. `cargo doc -p eggup-core --no-deps --locked --offline`

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.04s
Generated /Users/davidbowman/projects/eggup/target/doc/eggup_core/index.html
```

No rustdoc warnings. The new "# The deadline and a stdin write failure"
section renders, and the intra-doc links to `CommandOutput::timed_out`,
`Error::Io`, and `Error::InvalidInput` resolve.

### 8. `cargo tree -p eggup-core --locked --offline --depth 1` (review step, not a gate)

```text
eggup-core v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-core)
└── sha2 v0.10.9
```

**Dependency-boundary result: unchanged.** `sha2` only on this (non-Windows)
target; `self-replace` remains the sole Windows-only edge. No new dependency
of any kind was added, no `build.rs` was added, and `unsafe` remains denied.

### 9. `git diff --check`

```text
$ git diff --check -- crates/eggup-core plans/implementation/verified-update-core plans/closure/verified-update-core
EXIT=0
```

The repository-wide `git diff --check` reported three "new blank line at EOF"
findings at the time of the run, in
`plans/closure/acquisition-transport/014-status.md`,
`plans/closure/service-lifecycle/015-status.md`, and
`plans/implementation/acquisition-transport/014-adapter-capability-parity-corrective.md`.
Those files belong to other agents working in parallel on the same branch and
were not touched by this milestone; the scoped check over this milestone's own
paths is clean.

### 10. Load runs and the pre-M018 comparison

Two additional verifications are recorded in full above, because they are the
evidence for the load-sensitivity fix and for the fixture discrimination
claim:

- **Load runs** — three consecutive full-crate runs with one concurrent cold
  workspace compile, and one full-crate run with four concurrent cold
  workspace compiles (`--no-fail-fast`): 110 passed, 0 failed in every run.
  See "Load-sensitivity reproduction and fix".
- **The pre-M018 comparison** — the current fixture file run against
  `bf439f5`'s `candidate.rs` in a copy of the workspace outside the
  repository: 14 passed, 0 failed. See "Fixture discrimination proof".

Neither run mutates the repository: the first uses `/tmp` target directories,
the second a `/tmp` copy of the tree, and no git state was changed.

### 11. Not run by this milestone

- `./scripts/check-local.sh` and the other workspace-wide gate steps — owned by
  the parent session (the plan's §11).
- The hosted CI lanes and the Linux and Windows platforms — run by the parent
  session in hosted run `40067964510` (commit `a431aaa`); see the
  parent-session verification addendum below and F1.

## Invariant review (plan §4, item by item)

| Invariant | Result | Evidence |
| --- | --- | --- |
| The deadline remains an unconditional maximum bound on the whole bounded execution | **Holds** | The deadline arm is unchanged in substance (`candidate.rs:454-462`); the deadline is now fixed earlier (`:335-340`), which shortens it by microseconds, never lengthens it. R7, and the pre-existing `candidate_deadline.rs` suite is green. |
| M014's guarantee that a bounded call always returns | **Holds** | No path waits for the writer; the writer is abandoned at the deadline. The `candidate_stdin_deadline_bounds_a_child_that_never_reads_stdin` fixture is green. |
| An undelivered payload is never reported as a success | **Holds** | R5; the sweep asserts it on 14 samples. |
| The output bound is unchanged: per stream, overflow discards that stream's bytes | **Holds** | `read_limited` and `poll_reader` are untouched. |
| `CommandOutput`'s published shape and field meanings are unchanged | **Holds** | No field, accessor, or constructor of `CommandOutput` changed. |
| `Error` keeps exactly the seven published variants | **Holds** | `crates/eggup-core/src/error.rs` is unmodified. R17. |
| `FailureCategory` is untouched and stays `#[non_exhaustive]` | **Holds** | Only the rustdoc of the enum changed; no member, no `as_str()` string. R12. |
| The ownership model is untouched; `Owned` is never inferred | **Holds** | `domain.rs` and the validators are unmodified. |
| Core never creates a live destination parent; candidate execution never mutates a live destination | **Holds** | Validation runs before the mutation lock is taken and before any live mutation. |
| The inherited environment is still cleared | **Holds** | `.env_clear()` is still applied (`candidate.rs:317`). |
| Stdin stays null without a payload | **Holds** | The pre-existing `candidate_stdin_without_a_payload_leaves_stdin_null` fixture is green. |
| `eggup-core` keeps its dependency surface; no new dependency, no `unsafe` | **Holds** | Step 8; `crates/eggup-core/src/lib.rs:1` still carries `#![forbid(unsafe_code)]`. |
| Rust 1.89 remains the MSRV | **Holds** | The whole verification sweep ran on 1.89.0. |
| Every new public item has rustdoc; every public fallible item has `# Errors` | **Holds** | Steps 2 and 7 (`#![deny(missing_docs)]` plus the crate-local `clippy::missing_errors_doc`). M018 adds **no** public item. |
| SHA-256 remains integrity evidence only | **Holds** | No authenticity, signature, or provenance claim was added anywhere. |
| The four documented `run_bounded` outcomes are preserved | **Holds** | The outcome list is unchanged in shape (`candidate.rs:251-262`); the deadline entry now also names the undelivered-payload case. No result carries both `timed_out` and `output_limited`. |

## Failure / rollback / recovery review

- **A stdin failure before the deadline is never a success.** The verdict is an
  `Err`, so `run_bounded` returns no `CommandOutput` and `success()` is never
  consulted for that call.
- **A stdin failure at or after the deadline is the deadline outcome.**
  `Ok`, with `timed_out` set, and `success()` false. The payload was not
  delivered, and the result does not claim it was.
- **A failed write kills and reaps the child** (`candidate.rs:415-426`). The
  payload was not delivered, so leaving the child running would be an unbounded
  process. Core kills only the direct child it spawned; a descendant is never
  killed.
- **Transaction state is untouched by every new failure.** Candidate
  validation runs before the mutation lock is acquired and before any live
  destination is touched, so an oversized payload, a failed write, a spawn
  failure, and the deadline outcome all occur with **no lock held, no backup
  set, no receipt, and no live file modified**.
- **No rollback is needed**, because no live mutation can precede these
  failures. Core's rollback and recovery machinery was not touched.
- **Recovery from an abandoned writer.** A writer still blocked in `write` at
  the deadline is abandoned, not reclaimed, exactly like a reader blocked on a
  pipe a surviving descendant holds. It ends when the child's stdin pipe
  breaks — that is, when the child is killed, or when a holding descendant
  exits or closes the pipe. This residual is stated in the rustdoc
  (`candidate.rs:197-206`, `:242-249`) and is unchanged from M017's F4.
- **No crash-journaling claim** is made or implied. `run_bounded` creates no
  file and holds no persistent state.

## Compatibility and migration review

**For a consumer of the published `eggup-core 0.1.3`:**

- **Nothing changes for them.** The published crate on crates.io still
  classifies a post-deadline write failure as an `Err` in the runs where its
  poll phase happens to observe the verdict, and still carries the pre-M018
  `FailureCategory` rustdoc wording. The change is on the branch only; the
  published crate is unchanged (F2).
- **The change is additive and invisible at the type level.** No public item
  changes shape, no public item is removed, and no public item is added. `Error`
  keeps exactly the seven published variants (plus the `#[cfg(test)]`-only
  `Error::Injected`), so an exhaustive downstream `match` on `Error` still
  compiles unchanged. `FailureCategory` keeps its members and its published
  `as_str()` strings.
- **One behavioural change.** A write failure observed at or after the deadline
  is now reported as the deadline outcome (`Ok` with `timed_out` set) instead
  of as an `Err`. A downstream adapter that branched on `operation ==
  "writing bounded stdin"` to mean "the payload was not delivered" must now
  read `timed_out` as well: the payload was still not delivered, and
  `timed_out == true` says so. The operation string itself is unchanged and
  remains a stable classification surface.
- **No behavioural change for a caller that attaches no payload.** Null stdin,
  no writer thread, the same four outcomes.
- **No migration path is required.** The change is source-compatible, and a
  consumer adopts it at its own pace.

**No republication was performed.** No version bump, no tag movement, no
`cargo publish`, and no publication milestone authorizes one here. Reaching
consumers additionally requires the separate republication cascade recorded in
F2 — and `eggup-eggpack`'s `=0.1.3` pin (`crates/eggup-eggpack/Cargo.toml`)
blocks propagation to that adapter until a separate adapter republication
re-pins the new core version.

**`eggup-service` is unaffected.** `crates/eggup-service/**` was not modified.

### Every place that describes the old behaviour (for the parent session)

These are the passages that state the pre-M018 behaviour. The parent session
owns `architecture/**`, `plans/registry.md`, `plans/subsystems/**`, and the
root `CHANGELOG.md`; this milestone did not edit them. The wording below is
quoted exactly.

| # | File and passage | What it says now | What it must say now |
| --- | --- | --- | --- |
| 1 | `architecture/core-transaction.md:330-337` (the candidate-execution section) | "The deadline bounds the writer exactly as it bounds the readers: at the deadline the child is killed, which breaks the child's stdin pipe and unblocks the writer, and a writer still blocked in `write` is abandoned explicitly, exactly like a reader blocked on a pipe a surviving descendant holds. A bounded execution is not reported as complete until the writer has delivered its verdict, so a write failure is never lost behind a "the child exited" observation." | The kill breaks the pipe only for a still-running child whose stdin pipe no descendant inherited; a descendant that inherited the pipe keeps it open, so the writer stays blocked until that descendant exits or closes the pipe. **And** the deadline has priority over the writer's verdict: the verdict is classified by the instant the writer observed the write, so a write failure observed before the deadline is an `Err` and one observed at or after it is the deadline outcome. |
| 2 | `architecture/core-transaction.md:341-349` (the operation-string table) | "`"writing bounded stdin"` (the stdin payload could not be written, which includes the broken pipe a child produces when it exits without reading its stdin; the child is killed and reaped, so the payload was **not delivered** and the execution must not be treated as complete). When a failed stdin write and an output overflow are live at the same time, the write failure wins: it is an `Err`, and the caller must not be told that the execution merely overflowed." | The same text plus the deadline qualification: the write failure is an `Err` when it is observed **before** the deadline; when it is observed at or after the deadline the deadline outcome stands (`Ok`, `timed_out` set), and the payload was still not delivered. |
| 3 | `architecture/core-transaction.md:732` (the failure-semantics table) | "\| Stdin payload cannot be written, including the broken pipe of a child that exits without reading it \| `Err(Io)` (`operation` `"writing bounded stdin"`) \| The payload was **not delivered**; the child was killed and reaped, and nothing was committed \| `candidate.rs:375-385`, `candidate.rs:439` \|" | The same row plus "when the failure is observed before the deadline"; and the `candidate.rs:` anchors must be refreshed to the post-M018 lines (`candidate.rs:413-436`, `candidate.rs:515` for the error construction). |
| 4 | `architecture/core-transaction.md:117` and `:125` (the export table) | "`CommandSpec` \| Bounded external-command request (program, args, cwd, timeout, output cap, explicit env only, optional bounded stdin payload) \| `candidate.rs:31` \|" and "`MAX_STDIN_BYTES` \| Inclusive bound on one bounded stdin payload (1 MiB) \| `candidate.rs:27` \|" | The `candidate.rs:` anchors are stale after M018 (the struct is at `candidate.rs:31` still, and `MAX_STDIN_BYTES` at `candidate.rs:27` — both are unchanged by M018, so no edit is required; the refresh is only needed if the parent prefers the whole file to be re-anchored). |
| 5 | `plans/subsystems/verified-update-core-roadmap.md` (the status line at `:3` and the § 7 status table) | The status line reads "…; M018 bounded-stdin outcome-priority corrective closed (native macOS evidence, hosted qualification outstanding)" and the § 7 table carries the `M018` row with the status cell `closed; hosted qualification outstanding` | Both cells carry the hosted qualification: the status line's clause becomes "M018 bounded-stdin outcome-priority corrective closed 2026-10-24 (hosted run `40067964510`)" and the row's status cell becomes `closed; hosted-qualified in run \`40067964510\``. See the appendix below. |
| 6 | `plans/registry.md` (the subsystem summary row and the § "Active subsystem roadmaps" row) | Both rows already read "M001-M018 closed…" but still end with "M018 closed (native macOS evidence), hosted qualification outstanding" | Both rows carry the hosted qualification: "M018 closed and hosted-qualified in run `40067964510`". See the appendix below. |
| 7 | The root `CHANGELOG.md` `Unreleased` | The M017 aggregate entry | It must carry the M018 aggregate entry, matching `crates/eggup-core/CHANGELOG.md`: the deadline has priority over the stdin writer's verdict, the verdict is classified by its own observation instant, a post-deadline write failure is the deadline outcome instead of an `Err`, the kill-and-pipe wording was corrected, the taxonomy's producer count was corrected, the fix is unpublished, and no publication milestone authorizes it. |

**Contracts deliberately left unchanged.** `crates/eggup-core/docs/domain.md`
and `crates/eggup-core/docs/transaction.md` were reviewed and **not** changed:
neither mentions the stdin writer, its deadline, or the failure taxonomy, so
there is no contract statement to reconcile. The candidate-execution contract
lives in `crates/eggup-core/docs/verification.md`, which was updated.

## Security review

- The inherited environment is still cleared (`.env_clear()`), so no ambient
  environment variable can influence a child.
- Without an attached payload the child's stdin is still the null device; core
  cannot be made to feed a child by accident.
- The stdin payload is **bounded** (1 MiB, inclusive) and an oversized payload
  is rejected **before any child is spawned**, so an unbounded allocation into a
  pipe and an unbounded process are both prevented.
- Captured output remains bounded per stream; the stdin bound does not widen
  the output bound.
- The payload is written verbatim. Core does not interpret, transform, or
  validate its content, and claims nothing about it.
- **The classification change does not weaken the fail-closed direction.** A
  payload that was not delivered is still never reported as a success: the
  outcome is an `Err` or an `Ok` whose `timed_out` flag is set, and `success()`
  is false for both.
- No new dependency, no `unsafe`, no FFI, no `build.rs`, no process-group or
  job-object mechanism, and no elevated privilege.
- The core rules are unchanged: SHA-256 remains integrity evidence only, no
  authenticity or signature claim is introduced, and no invented timeout is
  introduced — the only timeout remains the caller-supplied
  `CommandSpec::timeout`.

## Documentation / operations evidence

| Document | What changed |
| --- | --- |
| `crates/eggup-core/src/candidate.rs` | The `run_bounded` rustdoc gained the "# The deadline and a stdin write failure" section (the rule, stated in the three cases and the never-a-false-success guarantee); the kill-and-pipe sentence was corrected to carry the descendant caveat; the outcome list's deadline entry now names the undelivered-payload case. |
| `crates/eggup-core/src/transaction.rs` | The `FailureCategory` rustdoc's producer count: five in a shipped build, plus `Injected` as test-only. |
| `crates/eggup-core/CHANGELOG.md` | The M018 `Unreleased` entry (the rule, the behaviour change, the two corrected statements, the unchanged public and dependency surfaces, the unpublished status). The M017 entry's "the deadline now bounds the writer too" and "Added: a typed write failure" bullets were corrected, because that entry has never been published and its text would otherwise ship this release stating the pre-M018 behaviour. |
| `crates/eggup-core/README.md` | The bounded-runner paragraph now states the deadline qualification of a write failure. |
| `crates/eggup-core/docs/verification.md` | The candidate-command paragraph now states the rule and keeps pointing at the `run_bounded` rustdoc as the normative statement. |
| `plans/implementation/verified-update-core/018-bounded-stdin-outcome-priority-corrective.md` | **New.** The 16-section plan. Its `Status:` header reads `implemented`. |
| `plans/closure/verified-update-core/018-status.md` | **New.** This record. |
| `plans/closure/verified-update-core/015-status.md` | **Dated addendum** closing U1 on the evidence of commit `ab90688` and reconciling the record's status wording with the registry and the roadmap. |
| `plans/closure/verified-update-core/017-status.md` | **Dated addendum** naming the statement this milestone supersedes and pointing at M018. |

## Platform results

Reported separately. **No platform result is inferred from another.**

| Platform | Result | Evidence |
| --- | --- | --- |
| **macOS** (25.6.0, Darwin, arm64) | **PASS** | Native: the full §"Exact commands run and results" sweep — fmt, clippy with `-D warnings`, the 110-test crate suite, the filtered `candidate` run, the 14-test fixture file, the three unit pins, rustdoc, the scoped `git diff --check`, the dependency review, the load runs, and the pre-M018 comparison (step 10). Toolchain `rustc 1.89.0` — the workspace MSRV. |
| **Linux** | **PASS (hosted)** | Hosted run `40067964510` (commit `a431aaa`): the `stable` lane, 528 passed, 0 failed. The `msrv` lane is a `cargo check` by design and runs no test. Recorded in the parent-session verification addendum below. No Linux runner is reachable from this machine, so no result is inferred from another platform. |
| **Windows** | **PASS (hosted)** | Hosted run `40067964510` (commit `a431aaa`): the `windows-check` lane, 650 passed, 0 failed. That lane executes the workspace test targets directly, so `crates/eggup-core/tests/candidate_stdin.rs` ran natively on Windows. Recorded in the parent-session verification addendum below. |

The fixture file is written to be cross-platform: it re-executes **this very
test binary** as its helper and selects the helper role from the helper's own
**argv** (not the environment, because `run_bounded` clears the inherited
environment). No shell, no `cmd.exe`, and no external tool is used, so
`sh -c` is not required on Windows. The fixture file is **not** `#[cfg]`-gated,
so the `windows-check` lane's workspace step executes it natively, exactly as
it executes `candidate_deadline.rs`.

Three Windows-specific behaviours are worth stating for the hosted run, because
they are the platform-sensitive parts of the new fixtures:

1. The write-failure fixtures rely on the OS pipe buffer being smaller than
   `MAX_STDIN_BYTES` (1 MiB). The Windows anonymous pipe default is 4 KiB and
   the Linux and macOS defaults are 64 KiB and 16 KiB, so the 1 MiB payload
   cannot be silently accepted by the kernel.
2. The write-failure fixtures assert the `operation` string and that the
   underlying `io::Error` is preserved and is not `ErrorKind::Other`. They
   deliberately do **not** assert a specific `io::ErrorKind`, because Windows
   reports a broken pipe differently from Unix.
3. The sweep's ladder is a *sequence of separate bounded calls*, each with its
   own spawn and its own deadline, so Windows' coarser timer resolution
   changes which samples fall on which side of the deadline — it does not
   change any assertion, because the fixture asserts only the properties that
   hold on both sides.

## Unresolved findings with severity

**F1 — Hosted qualification and the Linux / Windows platforms. CLOSED.**
Severity: **medium** when it was raised. The milestone's native evidence was
macOS only, and the plan's §11.1 required a fresh hosted run of
`.github/workflows/ci.yml` covering at least the `stable`, `msrv`, `macos`,
and `windows-check` lanes, and confirmation that the `windows-check` lane's
workspace step executed `crates/eggup-core/tests/candidate_stdin.rs`
natively. The parent session ran it: hosted run `40067964510` (commit
`a431aaa`) is green on all five lanes, with the per-platform results recorded
in the parent-session verification addendum at the end of this record. The
hosted commit contains the whole change, including the load-sensitivity fix
(`640e97e` and `a431aaa` are its ancestors). **This finding is closed on that
evidence.**

**F2 — The correction is unpublished.** Severity: **medium**. The published
`eggup-core 0.1.3` still classifies a post-deadline write failure as an `Err`
in the runs where its poll phase observes the verdict, and still carries the
pre-M018 `FailureCategory` rustdoc wording. Until a core republication lands,
no published consumer has the correction, and `eggup-eggpack`'s `=0.1.3` pin
blocks propagation to that adapter until a separate adapter republication
re-pins the new core version. **Follow-on: a core republication milestone
publishing the correction, plus the matching `eggup-eggpack` / `eggup-archive` /
`eggup-acquisition` republication cascade.** This is the same publication
cascade every prior core change has named, and it is **carried forward**, not
created, by this milestone.

**F3 — The end-to-end difference between the two builds is bounded by one poll
interval.** Severity: **low**. The pre-M018 tree and this milestone differ
observably only in a window of at most one poll interval (2 ms) around the
deadline, because the pre-M018 tree classifies a verdict only when its poll
phase happens to observe it before the deadline arm runs. The consequence is
recorded in the "Fixture discrimination proof" section: the integration
fixtures do not fail deterministically against the pre-M018 tree, so the
deterministic pin of the rule is the unit test on the classification helper.
No corrective is proposed: narrowing the window further would require the
runner to wait for the writer's verdict at the deadline, which would
reintroduce a second deadline and regress M014's unconditional bound — the
plan's §14 lists that as a stop condition.

**F4 — An abandoned stdin writer is a residual, not a defect.** Severity:
**low**. When the deadline is reached while the writer is blocked in `write`,
the writer thread is abandoned rather than reclaimed, exactly as a reader
blocked on a pipe a surviving descendant holds. Core owns no process group and
denies `unsafe`, so it cannot terminate a descendant that inherited the child's
stdin pipe and keep the pipe from breaking. The residual is bounded in the same
way the reader residual is — the writer ends when the pipe breaks — and it is
documented in the rustdoc rather than left implicit. Carried forward from
M017's F4 unchanged. No corrective is proposed.

**No medium-or-higher finding about this milestone's own change remains open.**
F1 is closed by the parent session's hosted qualification (run `40067964510`,
commit `a431aaa`). F2 remains open at medium severity and is named as a
republication corrective; it is the standing publication cascade, not a defect
of the change.

## Roadmap disposition

- `plans/subsystems/verified-update-core-roadmap.md` § 7 (the milestone status
  table) carries **M018 — bounded-stdin outcome-priority corrective**, recorded
  as **closed** and **hosted-qualified in run `40067964510`**. The parent
  session pasted the row before that run; the status cell is now stale and
  needs the replacement in the appendix below.
- The roadmap's status line (currently `plans/subsystems/verified-update-core-roadmap.md:3`)
  gains the M018 clause; the exact replacement is in the appendix below.
- The roadmap's **M015** row is unchanged in substance — the row already reads
  `closed` and the hosted qualification — but the parent may add the U1 closure
  pointer M018 records, so that the roadmap and the closure record agree on
  U1's status (see the appendix).
- `plans/registry.md` records Verified update core as "M001-M018 closed" in
  both of the rows that carry the subsystem status (see the appendix). Both
  rows are already pasted with a pre-qualification wording and need the
  replacements in the appendix.
- `plans/registry.md` § "Post-0.1.3 open findings" gains no new entry for a
  defect in core: the publication gap (F2) is the standing publication cascade
  every prior core milestone already names.
- The root `CHANGELOG.md` `Unreleased` aggregate entry for this corrective is
  owed by the parent session (the root changelog is outside this milestone's
  ownership). The intended prose is in the documentation table above, row 7.

## Appendix — ready-to-paste rows for the parent session

**Not applied by this milestone.** `plans/registry.md`,
`plans/subsystems/verified-update-core-roadmap.md`, and the root `CHANGELOG.md`
are owned by the parent session and were not edited.

### `plans/subsystems/verified-update-core-roadmap.md` § 7 status table

**Supersedes the wording the parent session already pasted.** The parent
pasted the row below before the hosted qualification was run, so the row in
the roadmap still reads `closed; hosted qualification outstanding`; the
hosted run has since made that stale. Replace the status cell with
`closed; hosted-qualified in run \`40067964510\`` and leave the rest of the
row unchanged:

```text
| M018 | closed; hosted-qualified in run `40067964510` | `plans/implementation/verified-update-core/018-bounded-stdin-outcome-priority-corrective.md` | `plans/closure/verified-update-core/018-status.md` | The deadline has priority over the bounded stdin writer's verdict, and the verdict is classified by the instant the writer observed the write rather than by the runner's poll phase. A write failure observed before the deadline is an `Err`; one observed at or after it is the deadline outcome (`Ok`, `timed_out` set), so an undelivered payload is never reported as a success. The kill-and-pipe wording and the `FailureCategory` producer count were corrected, and the M015 closure record's U1 is closed on the evidence of commit `ab90688`. `Error` keeps exactly the seven published variants and no new dependency was added. Unpublished; `eggup-core` stays published at `0.1.3` |
```

Update the roadmap's status line (currently
`plans/subsystems/verified-update-core-roadmap.md:3`) by replacing the M018
clause with:

```text
M018 bounded-stdin outcome-priority corrective closed 2026-10-24 (hosted run `40067964510`)
```

### `plans/registry.md` — two rows carry the subsystem status

The registry carries the Verified update core status in **two** places. Both
are quoted as of the M018 closure; the registry is edited by other agents in
parallel, so the parent should apply the change to whatever the current row
says.

**Row 1 — the subsystem summary row** (currently `plans/registry.md:55`),
302 bytes today, **301 bytes** after the change:

```text
| Verified update core | M001-M018 closed; `eggup-core 0.1.3` published 2026-10-06; hosted runs `37376971555`, `37527706900`, `40008039843`, `40062299630`, `40067964510` green on Stable/MSRV/macOS/Windows; M018 closed and hosted-qualified in run `40067964510` | `plans/closure/verified-update-core/` |
```

**Row 2 — the "Active subsystem roadmaps" row** (currently
`plans/registry.md:478`), 420 bytes today, **415 bytes** after the change:

```text
| Verified update core | M001-M018 closed; M014 and M015 closed 2026-10-23 and hosted-qualified in run `40008039843`; M017 closed 2026-10-23 and hosted-qualified in run `40062299630`; M018 closed 2026-10-24 and hosted-qualified in run `40067964510` | M016 `eggup-core` contract-doc packaging — named as the follow-on by planning C013; not yet authored, and it needs the republication to be authorized separately |
```

Both stay under the 481-byte row limit.

### `plans/registry.md` § "Post-0.1.3 open findings"

No new entry is required for a core defect: the publication gap (F2) is the
standing publication cascade every prior core milestone already names. If the
parent wants it tracked explicitly, the wording would be:

```text
`eggup-core`'s bounded candidate runner classifies a bounded stdin write failure that is observed at or after the deadline by the runner's poll phase rather than by the write's own observation instant, so the same input and the same timeout can produce two different public classifications. The correction (Verified Update Core M018) is unpublished and reaches consumers only through a core republication and the matching adapter republications. Severity: medium.
```

This is the standing publication cascade, not a new defect.

### `plans/closure/verified-update-core/015-status.md` — status reconciliation

The record's header currently reads "conditionally closed". With U1 closed on
the evidence of commit `ab90688` (see the dated addendum at the end of that
record), the header becomes:

```text
Status: **closed** — every acceptance criterion in plan §13 is met. The one
§6.2 sub-requirement that was open, the type-level rustdoc for the four types
in `crates/eggup-core/src/error.rs`, was completed by commit `ab90688` — the
very commit that ships this record — and finding U1 is closed on that
evidence (see the dated addendum at the end of this record). This matches
`plans/registry.md` and `plans/subsystems/verified-update-core-roadmap.md`.
```

### Root `CHANGELOG.md` `Unreleased`

Add the aggregate entry for this corrective, matching
`crates/eggup-core/CHANGELOG.md`: the deadline has priority over the bounded
stdin writer's verdict, the verdict is classified by the instant the writer
observed the write rather than by the runner's poll phase, a write failure
observed at or after the deadline is the deadline outcome (`Ok`, `timed_out`
set) instead of an `Err`, an undelivered payload is never reported as a
success, the kill-and-pipe wording and the `FailureCategory` producer count
were corrected, the fix is unpublished, the published `eggup-core 0.1.3` still
carries the pre-correction behaviour, and no publication milestone authorizes
it.

## Parent-session verification addendum

This heading is **not** written by this milestone. The parent session adds it
after the hosted CI qualification (F1).

## Parent-session verification addendum (2026-10-24, hosted qualification)

This milestone is hosted-qualified. CI run `40067964510` (commit `a431aaa`) is
**green on all five lanes**. The per-platform results are reported separately,
and a lane that runs no test says so rather than inferring a result:

| Lane | Runner | Result |
|---|---|---|
| `stable` | ubuntu | **pass** — 528 passed, 0 failed |
| `msrv` | ubuntu, pinned `1.89.0` | **pass** — `cargo check` only; the lane runs no test by design |
| `service-systemd` | ubuntu, privileged | **pass** — 9 passed, 0 failed |
| `macos` | macos-latest | **pass** — 528 passed, 0 failed |
| `windows-check` | windows-latest | **pass** — 650 passed, 0 failed |
| local gate | this machine (macOS, arm64) | **pass** — `./scripts/check-local.sh` exit 0; 528 passed, 0 failed |

**The earlier hosted run of this branch was red, and the evidence is recorded
here rather than dropped.** Run `40067739294` (commit `3d91ab1`) failed the
`stable` lane: the core crate's pre-existing unit test
`bounded_candidates_are_exact_and_environment_is_cleared` failed with
`Io { operation: "spawning bounded candidate", source: Os { code: 26, kind:
ExecutableFileBusy, message: "Text file busy" } }`. That test existed at
baseline `c480541` and this branch did not modify it; the failure is an
`ETXTBSY` on the script the fixture had just written, which the kernel raises
while the file is momentarily open for writing. It did not reproduce in 25
consecutive local runs of `cargo test -p eggup-core --lib`, and the very next
hosted run was green on all five lanes. It is recorded as a re-observation of
registry finding **1** — the load-sensitive-test class — which is low severity
and blocks nothing.

**Windows and Linux evidence for the new fixtures in this milestone is native,
not compile-only**: the `windows-check` lane executes the workspace test
targets directly, and the `stable` and `macos` lanes run the same suite.
