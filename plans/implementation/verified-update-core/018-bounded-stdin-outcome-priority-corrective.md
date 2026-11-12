# Verified Update Core Milestone 018 — bounded-stdin outcome-priority corrective

Status: implemented

Repository baseline: `bf439f519300354ac14823e74acd48c7995033a6`

Source roadmap:

- `plans/subsystems/verified-update-core-roadmap.md`

Long-term requirements:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`

Applicable ADRs:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`
- `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md`

Primary class: invariant

## 1. Objective

`eggup-core`'s bounded candidate runner published a rule for the bounded stdin
write that the implementation did not apply, and the failure taxonomy published
a producer count that is only true in a test build.

The published rule (`crates/eggup-core/src/candidate.rs`, the `run_bounded`
rustdoc) was:

> A write failure detected **after** the deadline was reached does not
> retroactively turn the deadline outcome into an `Err`: the deadline already
> decided that outcome.

The implementation evaluated the stdin-error arm **before** the deadline arm
inside the poll loop, so a write failure that was observed after the deadline
was reached was reported as an `Err` anyway. The public classification of one
logical scenario therefore flipped on the runner's poll phase: the same input
and the same timeout produced both `Err(Error::Io { operation: "writing
bounded stdin", .. })` and `Ok(timed_out = true)`.

M018 states **one** rule, implements it so the rustdoc is literally true, and
corrects the two neighbouring statements that were not true:

- the deadline's effect on the writer (the "the child is killed, which breaks
  the child's stdin pipe and unblocks the writer" sentence overstated what the
  kill does when a descendant inherited the pipe);
- the `FailureCategory` producer count ("Six of them are produced by the crate's
  internal error-to-category mapping" — five are produced in a shipped build;
  the sixth, `FailureCategory::Injected`, exists only in this crate's own test
  builds).

M018 also closes the open finding **U1** of the M015 closure record, whose
stated premise ("`crates/eggup-core/src/error.rs` was left untouched") is false
against the baseline, and reconciles the M015 closure record's status with the
registry and the roadmap.

## 2. Why this milestone is ready

- **The defect is read-verified and reproduced.** At the baseline
  `bf439f5`, the stdin-error arm is at `candidate.rs:375-385` and the deadline
  arm at `candidate.rs:404-411`, in that order inside one loop body. A
  reproduction on macOS with a 1 MiB payload and a child that exits without
  reading its stdin produced 13 `Err` results (every one with `elapsed >
  timeout`) and 2 `Ok(timed_out = true)` results at `timeout = 5 ms`, and
  `Ok(timed_out = true)` every time at `timeout = 0/1/2 ms`.
- **The fix is local to one function.** The classification of one writer
  verdict is decided in one place in one file. No transaction, staging, lock,
  commit, rollback, or ownership machinery is involved, and no other crate
  consumes the changed code path.
- **The invariant that must not regress is already pinned.** M014's
  unconditional deadline and M017's bounded stdin capability are closed, with
  hosted qualification in run `40062299630`. M018 changes the *classification*
  of one failure, not the deadline's bound, not the bound on the payload, and
  not the error's public shape.
- **Scope is bounded.** Two production files in one crate
  (`crates/eggup-core/src/candidate.rs`, `crates/eggup-core/src/transaction.rs`),
  one integration-test file under `crates/eggup-core/tests/`, the crate's own
  `CHANGELOG.md`, `README.md`, and `docs/verification.md`, and the plan and
  closure records. No dependency change, no other crate, no publication.

## 3. Current implementation evidence

### 3.1 The published rule and the code that contradicted it

At the baseline, `crates/eggup-core/src/candidate.rs:203-205` published:

```text
/// is never lost behind a "the child exited" observation. A write failure
/// detected **after** the deadline was reached does not retroactively turn the
/// deadline outcome into an `Err`: the deadline already decided that outcome.
```

The loop body evaluated the stdin arm first (`candidate.rs:375-385`):

```rust
if let Some(Err(source)) = stdin_result.take() {
    stdin_error = Some(source);
    if status.is_none() {
        let _ = child.kill();
        status = child.wait().ok();
    }
    break;
}
```

and the deadline arm last (`candidate.rs:404-411`):

```rust
if Instant::now() >= deadline {
    timed_out = true;
    if status.is_none() {
        let _ = child.kill();
        status = child.wait().ok();
    }
    break;
}
```

The consequence is that "detected after the deadline" was decided by the
**poll phase**, not by a fact about the write: a verdict that arrived between
the deadline and the next poll was classified as a pre-deadline failure.

### 3.2 The writer verdict carried no instant

At the baseline the writer thread sent a bare `io::Result<()>` over the
channel (`candidate.rs:344-345`). Nothing recorded *when* the write completed,
so the runner had no way to distinguish "the write failed before the deadline"
from "the write failed after the deadline" — the only information it had was
the moment at which it happened to poll.

### 3.3 The deadline's effect on the writer was overstated

At the baseline, `candidate.rs:197-201` stated:

> when the deadline is reached, the child is killed, which breaks the child's
> stdin pipe and unblocks the writer

Killing the direct child breaks the pipe only when the child is still running
**and** no descendant inherited the pipe. A descendant that inherited the
child's stdin keeps the read end open, so the writer stays blocked and is
abandoned; the abandonment itself was disclosed later in the same paragraph
(`candidate.rs:218-225`) and as residual **F4** of the M017 closure record, so
the paragraph contradicted itself rather than hiding the leak.

### 3.4 The taxonomy's producer count was stated for a test build

`crates/eggup-core/src/transaction.rs:113-118` stated:

> Every member of this enum describes a failure that can actually occur. Six
> of them are produced by the crate's internal error-to-category mapping

The mapping (`transaction.rs:260-280`) produces `LockContention`,
`OwnershipConflict`, `Verification`, `InvalidInput`, and `Filesystem` — five
categories — and produces `Injected` only under `#[cfg(test)]`, because
`Error::Injected` (`crates/eggup-core/src/error.rs:61-62`) and
`FailureInjector`/`FailurePoint` (`crates/eggup-core/src/lib.rs:55-56`) exist
only in this crate's own test builds. `architecture/core-transaction.md` already
distinguished the two cases; the crate rustdoc did not.

### 3.5 The M015 closure record's premise for U1 is false

`plans/closure/verified-update-core/015-status.md:499-513` records finding
**U1** — four missing type-level rustdocs (`Error`, `RecoveryError`,
`RecoveryResult`, `Result`) — and states that the file "was **left
untouched**" because it was outside that session's ownership grant. Against
the baseline this is false: commit `ab90688` — the very commit that ships that
record — added the rustdoc for `Result` (`error.rs:4-8`), `Error`
(`error.rs:10-27`), `RecoveryError` (`error.rs:65-71`), and `RecoveryResult`
(`error.rs:89-95`).

### 3.6 The M015 closure record's status disagrees with the registry and the roadmap

`plans/closure/verified-update-core/015-status.md:3` says "conditionally
closed", while `plans/registry.md:436` ("M001-M017 closed") and
`plans/subsystems/verified-update-core-roadmap.md:502` ("M015 | closed") say
closed. `AGENTS.md` requires the closure record, the registry, and the roadmap
to agree.

### 3.7 The M017 closure record restates the false rule as verified evidence

`plans/closure/verified-update-core/017-status.md:207-210` asserts the
pre-M018 rule as fact and cites `candidate.rs:203-205` as its authority. That
statement is superseded by this milestone.

## 4. Invariants that must not regress

- `CommandSpec::timeout` remains an **unconditional maximum** wall-clock bound
  on the whole bounded execution, evaluated whether or not the direct child has
  exited. M018 must not widen it, add a tolerance, or add a second deadline.
- M014's guarantee is unchanged: a bounded call always returns, including when
  a descendant holds the output pipe open.
- An **undelivered payload is never reported as a success**. The outcome is
  either an `Err` or an `Ok` whose `timed_out` flag is set. `Ok` with no timeout
  indication and no error is not an acceptable outcome for an undelivered
  payload.
- The output bound is unchanged: `max_output_bytes` still bounds each stream,
  overflow still discards that stream's bytes, and the defaults remain 64 KiB
  and 5 s.
- `CommandOutput`'s published shape is unchanged: same fields, same
  accessors, same meanings. No new public item is added by M018.
- `Error` keeps exactly the seven published variants, plus the
  `#[cfg(test)]`-only `Error::Injected`. **Adding an `Error` variant is a stop
  condition** (§14).
- `FailureCategory` keeps its members and its published `as_str()` strings, and
  stays `#[non_exhaustive]`.
- The published precedence rule survives: when a failed stdin write and an
  output overflow are live at the same time, the write failure wins.
- The ownership model is untouched: `Absent | Owned | Foreign | Unknown` stays
  caller-proven through `OwnershipVerifier`, `Owned` is never inferred.
- Core never creates a live destination parent, and candidate execution never
  mutates a live destination.
- `run_bounded` keeps clearing the inherited environment and keeps null stdin
  when no payload is attached.
- `eggup-core` keeps its dependency surface: `sha2` only, plus `self-replace`
  on Windows. **No new dependency of any kind**, no `build.rs`, no FFI, no
  `unsafe`.
- The crate keeps `#![forbid(unsafe_code)]` and `#![deny(missing_docs)]`, and
  Rust 1.89 remains the MSRV.
- SHA-256 remains integrity evidence only. No authenticity or signature claim
  is introduced anywhere.

## 5. Scope

### In scope

- `crates/eggup-core/src/candidate.rs` — the writer verdict's observation
  instant, the classification of that verdict against the deadline, the
  deadline's computation point, the final drain of the writer's verdict, the
  `run_bounded` rustdoc (the rule and the kill/pipe caveat), and the unit tests
  that pin the rule.
- `crates/eggup-core/src/transaction.rs` — the `FailureCategory` rustdoc's
  producer count.
- `crates/eggup-core/tests/candidate_stdin.rs` — the fixtures for the new
  rule.
- `crates/eggup-core/CHANGELOG.md`, `crates/eggup-core/README.md`,
  `crates/eggup-core/docs/verification.md` — the crate's own statements of the
  rule.
- `plans/implementation/verified-update-core/018-bounded-stdin-outcome-priority-corrective.md`
  (this plan) and `plans/closure/verified-update-core/018-status.md`.
- `plans/closure/verified-update-core/015-status.md` and
  `plans/closure/verified-update-core/017-status.md` — dated addenda that close
  U1 and correct the superseded statement. Additions only: no existing closure
  evidence is rewritten.

### Explicitly out of scope

- **No change to `architecture/**`, `docs/**`, `plans/registry.md`,
  `plans/subsystems/**`, the root `CHANGELOG.md`, `AGENTS.md`,
  `.opencode/skills/**`, `.github/**`, or `scripts/**`.** The parent session
  owns those; the intended prose is drafted in the closure record's appendix
  (§15).
- **No change to `eggup-service`, `eggup-acquisition`, `eggup-curl`,
  `eggup-eggfetch`, `eggup-archive`, or `eggup-eggpack`.**
- **No new `Error` variant** and no change to `FailureCategory`'s members.
- **No new dependency** — in particular no `libc`, `nix`, or `windows-sys`, and
  no process-group or job-object mechanism.
- **No change to the transaction, staging, locking, commit, rollback, or
  ownership machinery**, and no change to the deadline's bound or to the output
  bound.
- **No publication, no version bump, no tag movement.**
- **No change to the M015 plan, the M017 plan, or any accepted ADR.** Their
  closure records gain dated addenda; their plans are historical.

## 6. Required production changes

### 6.1 The rule

`run_bounded`'s bounded stdin write is governed by exactly one rule:

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
> - a payload that was not delivered is never reported as a success.

### 6.2 The verdict carries its observation instant

The writer thread stamps its verdict with `Instant::now()` taken immediately
after the write completes (the write, then the flush). The channel carries
that stamped verdict instead of a bare `io::Result<()>`. The runner classifies
the verdict against the deadline through one helper, so the classification has
exactly one implementation.

### 6.3 The deadline is fixed before any thread exists

The deadline is computed immediately after the spawn returns, before the
pipes are taken and before the reader and writer threads are spawned. This
makes the published sentence "the deadline starts once the spawn has returned"
literally true, and it guarantees that the instant a writer observes its write
can be compared against a deadline that was already decided.

### 6.4 The verdict is classified where it is observed

Both the poll loop and the final drain classify a verdict with the same
helper. The final drain is guarded by the same "the writer already delivered"
flag the loop uses: once the writer has delivered, the slot is empty and the
channel is disconnected, and a second poll would mistake that for a writer
that ended without a verdict.

A verdict drained after the loop broke is classified by the same rule. A
failure observed before the deadline therefore **wins** over whatever outcome
the loop decided (including an output overflow), which is the published
precedence rule; a failure observed at or after the deadline leaves the
outcome the loop already decided untouched.

### 6.5 The kill-and-pipe sentence is corrected

The rustdoc states that killing the direct child breaks the child's stdin pipe
**for a child that is still running and whose stdin pipe no descendant
inherited**, and that a descendant which inherited the pipe keeps it open, so
the writer stays blocked until that descendant exits or closes the pipe.

### 6.6 The taxonomy's producer count is qualified

`FailureCategory`'s rustdoc states that **five** categories are produced by
the crate's internal error-to-category mapping in a shipped build, and that a
sixth — `Injected` — is produced by that same mapping only in this crate's own
test builds. `PostCommitCheck` is produced directly by the commit path.

### 6.7 The closure records are corrected

- The M015 closure record gains a dated addendum that closes **U1** on the
  evidence of commit `ab90688` and reconciles the record's status with the
  registry and the roadmap.
- The M017 closure record gains a dated addendum that names the statement it
  superseded and points at M018.

## 7. Ordered work packages

- **WP1 — The plan and the rule.** Write this plan and fix the observable
  contract (§6.1–§6.6).
- **WP2 — The production change.** Implement §6.2–§6.5 in
  `crates/eggup-core/src/candidate.rs` and §6.6 in
  `crates/eggup-core/src/transaction.rs`.
- **WP3 — The fixtures.** Add the unit tests that pin the rule, and the
  integration fixtures that pin the rule's observable halves (§10).
- **WP4 — The contract documentation.** Update the rustdoc, the crate
  `CHANGELOG.md`, the crate `README.md`, and
  `crates/eggup-core/docs/verification.md`.
- **WP5 — The record corrections.** Add the dated addenda to the M015 and M017
  closure records.
- **WP6 — The regression sweep.** Run every command in §11, write the closure
  record (§15), and flip this plan's status to `implemented`.

WP2 precedes WP3 deliberately: the fixtures must exist against the implemented
contract, not against a guess at it.

## 8. Failure, cancellation, restart, and contention semantics

- **A stdin failure before the deadline is never a success.** The verdict is an
  `Err`, so `run_bounded` returns no `CommandOutput` at all and `success()` is
  never consulted for that call.
- **A stdin failure at or after the deadline is the deadline outcome.** The
  call returns `Ok` with `timed_out` set and no `CommandOutput` field claims
  the payload was delivered. `success()` is `false` for that result.
- **A write failure that the loop never saw is not lost.** The final drain
  classifies a verdict that arrived at the bound, so a failure observed before
  the deadline cannot disappear behind a "the child exited" observation.
- **The child is killed and reaped on a pre-deadline write failure.** The
  payload was not delivered, so leaving the child running would be an unbounded
  process. Core kills only the direct child it spawned; a descendant is never
  killed.
- **Transaction state.** Candidate validation runs before the mutation lock is
  acquired and before any live destination is touched. A stdin failure
  therefore affects **no** transaction state: no lock is held, no backup set
  exists, no receipt is produced, no live file is modified. The caller
  receives an `Err` (or the deadline outcome) and may retry under its own
  policy; core imposes no retry.
- **Restart semantics** are unchanged: `run_bounded` holds no persistent state
  and creates no file.
- **Contention** is unchanged: `run_bounded` does not take the mutation lock.
- **Cancellation**: core has no cancellation token for candidate execution and
  M018 does not add one.
- **Abandonment is truthful.** A writer still blocked at the deadline is
  abandoned, not reclaimed, and the rustdoc says so.

## 9. Compatibility and migration

- **Additive source compatibility.** No public item changes shape and no
  public item is removed. `Error` keeps exactly the seven published variants,
  so an exhaustive downstream `match` on `Error` still compiles. No new public
  item is added.
- **Behaviour compatibility for a call with no payload.** Unchanged: null
  stdin, no writer thread, the same outcomes.
- **Behaviour change for a call with a payload.** A write failure observed at
  or after the deadline is now reported as the deadline outcome (`Ok` with
  `timed_out` set) instead of as an `Err`. The same input and the same
  timeout no longer produce two different public classifications. Every place
  that describes the old behaviour is listed in the closure record's
  documentation table, for the parent to propagate.
- **The published `eggup-core 0.1.3` still carries the defect.** It is
  published (crates.io, 2026-10-06, from `bd43683`;
  `plans/closure/verified-update-core/013-status.md`). M018 changes packaged
  behaviour in `src/candidate.rs` and `src/transaction.rs`, so the published
  crate keeps the pre-correction classification and the pre-correction
  taxonomy wording. The correction reaches consumers only when a new version is
  published.
- **M018 must not publish.** Publication is a tracked milestone of its own
  (M013 is the precedent).
- **The `eggup-eggpack` pin blocks propagation.**
  `crates/eggup-eggpack/Cargo.toml` pins `eggup-core = { version = "=0.1.3",
  path = "../eggup-core" }`, so a core fix cannot reach `eggup-eggpack` until a
  separate adapter republication re-pins the new core version.
- **Migration path: none required.** No consumer action is needed beyond
  picking up the fixed version.

## 10. Required tests

All new fixture names contain the substring `candidate`, so
`cargo test -p eggup-core --locked --offline candidate` selects them all.

The fixtures must be cross-platform: Linux, macOS, **and** Windows. `sh -c` is
not available on Windows, so the fixtures re-execute **this very test binary**
as their helper and select the helper role from the helper's own **argv** — not
from the environment, because `run_bounded` clears the inherited environment.
No shell, no `cmd.exe`, and no external tool is used.

1. **The rule pin (unit test).** `classify_writer_verdict` classifies a verdict
   by its observation instant: a failure observed strictly before the deadline
   is `FailedBeforeDeadline` and keeps its underlying `io::Error`; a failure
   observed at or after the deadline is `FailedAtOrAfterDeadline`; a
   successful write is `Delivered` whatever the deadline says. These are the
   deterministic pins of the rule: the pre-M018 code has no stamped verdict and
   therefore cannot express the rule at all.
2. **A write failure before the deadline is an `Err`.** A child that exits
   without reading its stdin, with a generous deadline, yields
   `Err(Error::Io { operation: "writing bounded stdin", .. })` with the
   underlying `io::Error` preserved.
3. **A write failure at or after the deadline is not an `Err`.** A child that
   outlives the deadline and is killed there yields `Ok` with `timed_out` set,
   `output_limited` false, and `success()` false: an undelivered payload is
   never reported as a success.
4. **The classification is stable across the deadline.** A sweep walks the
   child's lifetime across the deadline in two-millisecond steps. Every sample
   must never report an undelivered payload as a success; a reported write
   failure must come from a child that exited on its own (only the child's own
   exit breaks the pipe before the deadline — the runner's kill happens at the
   deadline); and a child that recorded no exit was killed at the deadline, so
   the deadline outcome must stand. The sweep reports its coverage counts on
   stderr, so a run states what it actually observed instead of only asserting.
5. **The fixtures must not depend on the host's load.** A helper binary can
   start after a short deadline, be killed at the deadline before it writes
   anything, and never produce its start marker. Every fixture that waits for
   that marker must therefore poll for it for a bounded time and must treat
   "the marker never appeared" as a distinct condition that asserts only the
   deadline outcome. No fixture may assert that a helper started.
6. **The existing core test suite still passes**, including
   `crates/eggup-core/tests/candidate_deadline.rs`, which encodes the M014
   deadline guarantees, and `crates/eggup-core/tests/candidate_stdin.rs`'s
   existing fixtures. **No existing assertion may be weakened or deleted.**

## 11. Required verification commands

```text
cargo fmt -p eggup-core -- --check
cargo clippy -p eggup-core --all-targets --locked --offline -- -D warnings
cargo test -p eggup-core --locked --offline
cargo test -p eggup-core --locked --offline candidate
cargo test -p eggup-core --locked --offline --test candidate_stdin
cargo test -p eggup-core --locked --offline --lib candidate_writer_verdict
cargo doc -p eggup-core --no-deps --locked --offline
cargo tree -p eggup-core --locked --offline --depth 1
git diff --check
```

`cargo tree -p eggup-core --locked --offline --depth 1` is a **review step,
not a gate**: it must show `sha2` only, plus `self-replace` on Windows. The
workspace-wide gate (`./scripts/check-local.sh`) is owned by the parent
session.

### 11.1 Hosted qualification

A fresh hosted run of `.github/workflows/ci.yml` is required, covering at
minimum the `stable`, `msrv`, `macos`, and `windows-check` lanes. The
`windows-check` lane runs `cargo test` **without** `--all-features`, so the new
fixtures must not depend on a feature; the lane's final workspace step must be
confirmed to have executed the new test file, and the test names must be
visible in the lane log. A lane that was not run is **not run**, never
inferred.

## 12. Documentation updates

- `crates/eggup-core/src/candidate.rs` — the `run_bounded` rustdoc: the new
  "# The deadline and a stdin write failure" section (the rule), the corrected
  kill-and-pipe sentence, the outcome list's deadline entry, and the
  `# Errors` section's unchanged shape.
- `crates/eggup-core/src/transaction.rs` — the `FailureCategory` rustdoc's
  producer count.
- `crates/eggup-core/CHANGELOG.md` — an `Unreleased` entry for M018, and the
  correction of M017's two overstated bullets (the M017 entry has never been
  published, so its text ships with this release and must be true).
- `crates/eggup-core/README.md` — the bounded-runner paragraph.
- `crates/eggup-core/docs/verification.md` — the candidate-command paragraph:
  the rule and the pointer at the normative rustdoc. `docs/domain.md` and
  `docs/transaction.md` are reviewed and **not** changed: the bounded stdin
  capability is a property of the candidate *execution* primitive, not of the
  domain model or the transaction state machine, and neither file mentions the
  stdin writer or its deadline.
- `architecture/core-transaction.md` — **owned by the parent session.** The
  passages that describe the old behaviour are listed, with their exact
  wording, in the closure record's documentation table.

## 13. Acceptance criteria

M018 closes only when:

- the writer's verdict carries the instant at which the writer observed the
  write's completion, and the classification of that verdict against the
  deadline has exactly one implementation;
- a write failure observed before the deadline is an `Err` with the operation
  string `"writing bounded stdin"`, and the child is killed and reaped;
- a write failure observed at or after the deadline is the deadline outcome:
  `Ok` with `timed_out` set, never an `Err`;
- an undelivered payload is never reported as a success;
- the call still always returns by the deadline, and M014's deadline guarantees
  are unchanged;
- the rustdoc states the rule literally, and the kill-and-pipe sentence
  carries the descendant caveat;
- `FailureCategory`'s rustdoc states five producers in a shipped build and
  names the sixth as test-only;
- the M015 closure record's U1 is closed with its evidence, the M015 status
  wording agrees with the registry and the roadmap, and the M017 closure
  record's superseded statement is corrected by a dated addendum;
- every invariant of §4 holds, with `Error`'s seven published variants and
  `FailureCategory` unchanged;
- `cargo tree -p eggup-core --locked --offline --depth 1` shows no new
  dependency edge;
- the §11 commands are green;
- the existing core suite, including `candidate_deadline.rs` and the existing
  `candidate_stdin.rs` fixtures, is green with no assertion weakened or
  deleted;
- no medium-or-higher finding related to this milestone remains open.

## 14. Stop conditions

Stop and write another plan if:

- the rule would require a new dependency, `unsafe`, FFI, or a `build.rs`;
- the rule would require adding an `Error` variant or otherwise changing the
  published `Error` surface — that is a 0.2/1.0 API-boundary decision, not a
  patch;
- the rule would change the `Ownership` enum, the ownership model, the
  transaction state machine, the lock, or the receipt semantics;
- the rule would require changing `eggup-service`, `eggup-curl`, or any other
  crate;
- the classification could not be decided without widening the deadline, adding
  a tolerance, or adding a second deadline;
- the fix would require weakening, deleting, or loosening an existing assertion
  in `crates/eggup-core/tests/candidate_deadline.rs`,
  `crates/eggup-core/tests/candidate_stdin.rs`, or the crate's own test
  module;
- the fix would change the published overflow byte semantics, the per-stream
  output bound, or the deadline semantics;
- the fixtures could only be made deterministic by asserting a tight
  millisecond ceiling, or would require a shell, `sh`, `cmd.exe`, `sleep`,
  `kill`, or any other external tool not guaranteed on all three target
  platforms.

Do not classify a genuine runtime failure as test flakiness to restore a green
baseline.

## 15. Closure evidence required

`plans/closure/verified-update-core/018-status.md` must contain everything
`plans/closure/README.md` requires, and in addition:

- a **requirement → evidence matrix** mapping each requirement of §6 and §10 to
  the fixture or command that proves it, with a `file:line` anchor and the
  **exact command and its result**;
- **separate Linux, macOS, and Windows results**, each from a native run. A
  platform that was not run is recorded as **not run** and is never inferred
  from another platform;
- the **fixture discrimination proof**: the exact mutation that was applied,
  the exact failure it produced, and the fact that the mutation was reverted;
- an **invariant review** against §4, item by item;
- a **failure/rollback/recovery review**: what a stdin failure does to
  transaction state (nothing), and the abandoned-writer residual recorded
  truthfully;
- a **compatibility and migration review**: exactly what changes for a consumer
  of the published `eggup-core 0.1.3`, the explicit statement that no
  republication was performed, and **the complete list of places that describe
  the old behaviour**, quoted exactly, for the parent session to propagate;
- a **security review**: the environment is still cleared, stdin is still null
  without a payload, the payload is bounded, the captured output is still
  bounded, and the change adds no dependency and no `unsafe`;
- **documentation/operations evidence**: a table of every document changed and
  what changed in it;
- **unresolved findings with severity**, with the reason for each severity;
- the **roadmap disposition** and the **registry updates**, drafted as
  ready-to-paste rows for the parent session — `plans/registry.md`,
  `plans/subsystems/**`, and the root `CHANGELOG.md` are owned by the parent
  session and are **not** edited here;
- the `## Parent-session verification addendum` heading is **not** written by
  this milestone; the parent adds it after the hosted CI qualification.

## 16. Handoff notes

- **M018 does not publish.** The published `eggup-core 0.1.3` still classifies
  a post-deadline write failure as an `Err` in some runs; the correction
  reaches consumers only when a new version is published, and no publication
  milestone authorizes that here.
- **M018 does not touch `architecture/**`.** The deep dive's candidate-execution
  section still describes the old behaviour; the exact passages and their
  exact wording are in the closure record's documentation table.
- **The follow-ons M018 names are:**
  1. **A core republication milestone** that publishes the correction, and the
     matching `eggup-eggpack` / `eggup-archive` / `eggup-acquisition`
     republication that re-pins `eggup-core`. Severity: **medium** — until it
     lands, no published consumer has the correction. This is the same
     publication cascade every prior core change has named.
  2. **A hosted qualification** of this milestone on Linux and Windows. The
     native evidence for M018 is macOS only; the hosted run is the parent's
     (§11.1). Severity: **medium** until it lands.
- **The rule is now a single implementation.** The classification of the
  writer's verdict lives in `classify_writer_verdict` and nowhere else; a
  future change to that function is a change to the published contract.
