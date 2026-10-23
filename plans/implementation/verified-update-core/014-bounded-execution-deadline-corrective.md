# Verified Update Core Milestone 014 — bounded-execution deadline correctness corrective

Status: closed 2026-10-23; implemented and green on the native macOS lane and hosted-qualified on Stable/MSRV/macOS/Windows/systemd, run `40008039843`. Closure record: `plans/closure/verified-update-core/014-status.md` — the previously outstanding hosted qualification is discharged, and the published `eggup-core 0.1.3` still carries the defect this milestone fixed.

Repository baseline: `c4805411f74b85158e0ae5d0a219a32c4209d98e`

Source roadmap:

- `plans/subsystems/verified-update-core-roadmap.md`

Original closed work affected:

- `plans/implementation/verified-update-core/004-integrity-and-candidate-validation.md`
- `plans/closure/verified-update-core/004-status.md`

Finding of record:

- `plans/registry.md` § "Post-0.1.3 open findings" — entry **3**:
  "3. `eggup-core` bounded candidate execution can exceed its documented wall-clock deadline."
  Registered at baseline `c480541` alongside this plan. The finding is assessed
  **high** severity and is closed when this milestone closes.

Long-term requirements:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`

Applicable ADRs:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`
- `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md`

Primary class: invariant

## 1. Objective

`eggup-core`'s bounded candidate runner documents `CommandSpec::timeout` as the
**maximum wall-clock execution time** of a candidate command
(`crates/eggup-core/src/candidate.rs:63-67`);
`crates/eggup-core/docs/verification.md:32-34` repeats that candidate commands
run under "a wall-clock deadline". The implementation does not honor that
promise: once the direct child has exited the deadline is no longer evaluated,
and the runner can wait forever for pipe EOF.

Target contract after M014:

- the deadline is an **unconditional maximum wall-clock bound** on the whole
  bounded execution, evaluated whether or not the direct child has exited;
- `run_bounded` always returns — it never waits past the deadline for a pipe a
  descendant still holds open;
- every outcome is bounded and fail-closed, and the observable result
  (`CommandOutput` flags, or `Error`) distinguishes deadline expiry, output
  overflow, spawn failure, and non-zero exit;
- no reader thread is left unaccounted for: a reader that delivered its result
  is joined, and a reader still blocked at the deadline is abandoned
  **explicitly and truthfully**, not silently detached.

Nothing else changes: the output bound, the ownership model, the transaction
model, the `Error` variant set, and every other crate stay as they are.

## 2. Why this milestone is ready

- **The defect is fully characterized and read-verified.** It was found by
  static reading of the baseline tree at
  `c4805411f74b85158e0ae5d0a219a32c4209d98e`, not by a failing production
  report; no production incident is known and none is claimed. The §3 analysis
  is a control-flow reading of `crates/eggup-core/src/candidate.rs:160-202` and
  depends on no unverified runtime assumption; §10 requires a fixture that
  proves it.
- **The crate has no open milestone.** `plans/registry.md` § "Active subsystem
  roadmaps" records Verified update core as "M001-M013 closed/hosted-green",
  and `plans/subsystems/verified-update-core-roadmap.md` § 7 closes M001–M013.
  M014 does not disturb a closed milestone's contract; it corrects a closed
  milestone's implementation against its own published contract.
- **Scope is bounded.** One production file in one crate
  (`crates/eggup-core/src/candidate.rs`), one new integration test file under
  `crates/eggup-core/tests/`, no dependency change, no other crate, no
  publication.
- **Severity is high.** `eggup-core 0.1.3` is published, and a published library
  can hang forever while violating a documented safety bound. A hang is not a
  slow path: the calling thread never returns, both reader threads stay
  blocked, and the candidate's process-table entry is retained. The hang is
  reachable from ordinary use — `run_bounded` is the only place core spawns a
  process (`architecture/core-transaction.md:302`) and it is on the path from
  `VerifiedTransaction::validate` (`candidate.rs:412`) to
  `ExactIdentityValidator::validate` (`candidate.rs:310-331`). Per this
  repository's gate, "medium-or-higher" forces a corrective before a milestone
  closes; this finding is above that bar.
- **Precedent shape.** The acquisition-transport and service-lifecycle
  determinism correctives
  (`plans/implementation/acquisition-transport/012-sub-second-deadline-test-determinism-corrective.md`,
  `plans/implementation/service-lifecycle/008-operation-deadline-test-determinism-corrective.md`)
  exist because sub-second wall-clock assertions in a unit test are
  load-sensitive. M014 must not repeat that mistake: §10 requires a
  deterministic fixture whose discriminating assertion is "the call must
  return", not "the call must return within N milliseconds".

## 3. Current implementation evidence

### 3.1 The bounded runner

`run_bounded` (`crates/eggup-core/src/candidate.rs:125-210`) clears the
environment, uses null stdin, pipes stdout and stderr, spawns one reader thread
per stream (`candidate.rs:153-158`), then polls (`candidate.rs:160-198`):

```rust
let deadline = Instant::now() + spec.timeout;
let mut timed_out = false;
let mut output_limited = false;
let mut status = None;
let mut stdout_result = None;
let mut stderr_result = None;
loop {
    if stdout_result.is_none() {
        if let Ok(result) = stdout_rx.try_recv() {
            output_limited |= result.is_err();
            stdout_result = Some(result.unwrap_or_default());
        }
    }
    if stderr_result.is_none() { /* symmetric */ }
    if status.is_none() {
        status = child
            .try_wait()
            .map_err(|source| Error::io("polling bounded candidate", source))?;
    }
    if output_limited {
        let _ = child.kill();
        status = child.wait().ok();
    } else if status.is_none() && Instant::now() >= deadline {
        timed_out = true;
        let _ = child.kill();
        status = child.wait().ok();
    }
    if ((timed_out || output_limited) && status.is_some())
        || (status.is_some() && stdout_result.is_some() && stderr_result.is_some())
    {
        break;
    }
    thread::sleep(Duration::from_millis(2));
}
if !timed_out && !output_limited {
    let _ = stdout_thread.join();
    let _ = stderr_thread.join();
}
```

### 3.2 Defect (a) — the deadline stops being enforced once the direct child exits

The deadline arm at `candidate.rs:187` is guarded by `status.is_none()`. Once
`try_wait()` has yielded `Some` (`candidate.rs:180-183`), `status` is `Some`
forever, so the `else if` arm is never evaluated again. The only remaining exit
from the loop is the second disjunct at `candidate.rs:193`,
`status.is_some() && stdout_result.is_some() && stderr_result.is_some()`, which
requires **both** reader threads to have delivered, which requires pipe EOF.

A descendant that inherits the child's stdout/stderr and holds it open after the
direct child exits keeps the pipe write end alive, so EOF never arrives,
`stdout_result`/`stderr_result` stay `None`, and the loop spins on
`thread::sleep(Duration::from_millis(2))` (`candidate.rs:197`) indefinitely.
`CommandSpec::timeout` is documented as the maximum wall-clock execution time
(`candidate.rs:63-67`), so this is a direct contract violation, not a tuning
question. The consequences are cumulative: the call never returns, both reader
threads stay blocked in `read()`, and the candidate's process-table entry is
retained. The guard is not about descendants specifically: the deadline is
evaluated only while the child is running, so it is unenforceable for any
reader that is slow to deliver.

### 3.3 Defect (b) — reader threads are detached on the timeout and overflow paths

The joins at `candidate.rs:199-202` are inside `if !timed_out &&
!output_limited`. On the timeout path (`candidate.rs:187-191`) and the overflow
path (`candidate.rs:184-186`) the reader threads are dropped without being
joined. In the common case (no descendant) the pipes close when the child is
killed, so those threads reach EOF and end shortly afterwards — but the join is
skipped regardless, so reclamation is accidental rather than guaranteed. In the
descendant case of §3.2 the threads remain blocked in `read()` indefinitely, so
every such timeout retains two threads.

### 3.4 Recorded observations M014 does not silently inherit

Read-verified in the baseline and recorded so M014 does not inherit them
unnoticed. Only the first two items are in M014's scope.

- **Bounded-output semantics differ between crates.** Core's `read_limited`
  (`candidate.rs:212-225`) returns `Err` on overflow and drops the accumulated
  buffer with it, so an overflowed stream is reported as **empty**, not as the
  first `limit` bytes. `eggup-service`'s `read_bounded_generic`
  (`crates/eggup-service/src/lib.rs:952-975`) returns `(buf, true)` and retains
  the chunk that crossed the bound — up to `limit + 8192` bytes. Both implement
  "bounded output" with different observable semantics. M014 must not change
  core's published overflow semantics (the tests pin it); the asymmetry is
  recorded for a separate service milestone (§16).
- **A reader I/O error is reported as `output_limited`.** `candidate.rs:169`
  and `candidate.rs:175` set `output_limited |= result.is_err()`, so a read
  failure is indistinguishable from an overflow. M014 must not make this worse
  and must not widen scope to fix it; recorded as a low-severity follow-up.
- **The bound is per stream, not combined.** `max_output_bytes`
  (`candidate.rs:24`, `candidate.rs:70`) is passed to `read_limited` separately
  for stdout and stderr (`candidate.rs:153-158`), so total captured volume can
  reach twice the configured value; the field doc says "combined per-stream
  output bound" (`candidate.rs:69`). M014 does not change the bound; §12
  requires unambiguous wording.
- **The deadline starts after the spawn.** `deadline` is computed at
  `candidate.rs:160`, after `command.spawn()` returns (`candidate.rs:139-141`),
  so a blocking spawn is not covered. Bounding a spawn is not achievable with
  the current dependency surface; M014 does not attempt it and §12 requires the
  rustdoc to say so.
- **`timeout` accepts zero** (`candidate.rs:64-67`, no validation), producing an
  already-elapsed deadline. Unchanged by M014.

### 3.5 The duplicated runner

`eggup-service` implements its own bounded subprocess runner — `SystemExecutor`
(`crates/eggup-service/src/lib.rs:671`), `fn run`
(`crates/eggup-service/src/lib.rs:700`), `read_bounded_generic`
(`crates/eggup-service/src/lib.rs:952`) — duplicating core's `run_bounded`.
`eggup-service` already depends on `eggup-core`, so the duplication is not
required by any dependency-boundary rule. `eggup-curl` has a third copy
(`crates/eggup-curl/src/lib.rs:594`), and that one **is** justified:
`eggup-curl` may depend only on `eggup-acquisition` and must not depend on
`eggup-core` (`architecture/tooling-governance.md` §2). M014 records both facts
and changes neither (§5, §16).

## 4. Invariants that must not regress

- `CommandSpec::timeout` remains a **maximum** wall-clock bound: never widened,
  never given a tolerance, never renewed, never extended by a retry. A fix that
  needs a longer deadline to pass is a wrong fix (§14).
- The output bound is unchanged: `max_output_bytes` (`candidate.rs:24`,
  `candidate.rs:70`) still bounds each stream, overflow still discards that
  stream's bytes, and the defaults remain 64 KiB (`candidate.rs:15`) and 5 s
  (`candidate.rs:14`). M014 does not tune timeouts.
- `CommandOutput`'s field meanings are stable: `timed_out` means the deadline
  was the cause of the bounded execution not completing; `output_limited` means
  an output bound was exceeded; `success()` means exited `0`, no timeout, no
  overflow (`candidate.rs:108-121`). `success()` must remain `false` for every
  deadline expiry, including the descendant case (§6.4).
- `CommandOutput`'s published shape is unchanged — same fields, same
  accessors — and `CommandSpec` is unchanged. No new public type is required.
- `Error`'s published variant set is unchanged. `Error` is a closed enum
  (`crates/eggup-core/src/error.rs:9-44`) and M012 deliberately preserved the
  seven published `0.1.2` variants. **Adding an `Error` variant is out of scope
  and is a stop condition** (§14); the descendant case is reported through the
  existing `Error::CandidateExecution` / `CommandOutput` surface.
- The ownership model is untouched: `Absent | Owned | Foreign | Unknown` stays
  caller-proven through `OwnershipVerifier`, `Owned` is never inferred, no
  classification changes.
- Core never creates a live destination parent, and candidate execution never
  mutates a live destination. Validation runs before the mutation lock is taken
  (`candidate.rs:412-423`), so a deadline expiry can never leave a partially
  applied transaction. The transaction model's fail-closed behavior is
  unchanged: a validator failure is `Err`, never a receipt and never a success
  (`architecture/core-transaction.md:644`).
- `run_bounded` keeps clearing the inherited environment (`candidate.rs:129`)
  and passing null stdin (`candidate.rs:130`).
- `eggup-core` keeps its dependency surface: `sha2` only, plus `self-replace` on
  Windows (`crates/eggup-core/Cargo.toml:24-33`). No transport, service
  manager, or Eggpack dependency is introduced.
- The workspace denies `unsafe_code` (`Cargo.toml` `[workspace.lints.rust]`) and
  `eggup-core` carries `#![forbid(unsafe_code)]` and `#![deny(missing_docs)]`
  (`crates/eggup-core/src/lib.rs:1-2`); any new public item needs rustdoc. Rust
  1.89 remains the MSRV.

## 5. Scope

### In scope

- One production file: `crates/eggup-core/src/candidate.rs`, function
  `run_bounded` (`candidate.rs:125-210`), plus the rustdoc of
  `CommandSpec::timeout` (`candidate.rs:63-67`), `CommandOutput::timed_out`
  (`candidate.rs:108-111`), and `run_bounded` (`candidate.rs:124`).
- One new integration test file, `crates/eggup-core/tests/candidate_deadline.rs`,
  holding the deterministic descendant fixture and the deadline / overflow /
  spawn / exit classification fixtures. Integration tests are excluded from the
  packaged artifact (`crates/eggup-core/Cargo.toml:15-19`), so the fixture does
  not change published bytes.
- The rustdoc contract, the `eggup-core` `CHANGELOG.md` `Unreleased` entry, the
  root `CHANGELOG.md` `Unreleased` entry, and the two lines that state the
  candidate-execution contract (`architecture/core-transaction.md:302-307`,
  `crates/eggup-core/docs/verification.md:32-34`).
- Registry and roadmap registration of M014 and of the new open finding, and the
  closure record.

### Explicitly out of scope

- **No change to `eggup-service`.** `SystemExecutor`
  (`eggup-service/src/lib.rs:671`), `fn run` (`:700`) and `read_bounded_generic`
  (`:952`) are recorded in §3.5 and §16 as a candidate for a separate service
  milestone. M014 must not touch them. **No change to `eggup-curl`**, whose copy
  of a bounded runner (`eggup-curl/src/lib.rs:594`) is justified by its
  dependency boundary.
- **No republication and no publication.** `eggup-core 0.1.3` is published; M014
  changes packaged behavior but publishes nothing (§9).
- **No new dependency** — in particular no `libc`, `nix`, `windows-sys`, or
  process-group crate; `eggup-core` stays at `sha2` plus the Windows-only
  `self-replace`. **No `unsafe`**, no FFI, no `build.rs`.
- **No change to the `Ownership` enum, the ownership model, the transaction
  state machine, the lock, the receipts, the failure taxonomy, or the `Error`
  variant set**, and no new public error type.
- **No timeout-value tuning**: `DEFAULT_TIMEOUT` (`candidate.rs:14`) and every
  caller's chosen timeout stay as they are.
- **No restructuring of `eggup-core`'s module layout.** `candidate.rs` stays
  where it is; no new module and no re-export reshuffle in `lib.rs:26-29`.
- **No change to the output bound or to the overflow byte semantics** (§3.4).
- **No change to `eggup-eggpack`, `eggup-archive`, `eggup-acquisition`, or
  `eggup-eggfetch`.**
- **No CI configuration change**, unless §11's Windows requirement cannot be met
  by the existing lane steps — and then only by adding a step, never by
  weakening one.
- **No consumer-facing how-to rewrite.** `docs/` is non-normative and must not
  become the place a rule is introduced (§12).

## 6. Required production changes

This section states required **behavior**, not implementation. The implementer
chooses the mechanism; the observables below are fixed.

### 6.1 The deadline is unconditional

`run_bounded` must evaluate the deadline on every iteration of the polling loop,
regardless of whether the direct child has already exited. When the deadline
elapses: if the direct child is still running it is killed and reaped, exactly
as today (`candidate.rs:187-191`); if it has already exited, nothing is killed
and nothing is reaped again; and the loop leaves without waiting for pipe EOF.
The call must return within the deadline plus a bounded teardown allowance (the
kill and reap of the direct child). The allowance must be bounded and stated in
the rustdoc; it must not become a second, larger deadline.

### 6.2 Results delivered at the deadline are still reported

Reader results already delivered when the deadline is observed must still be
reported. A reader that has not delivered is not waited for, and its bytes are
not reported. The returned `stdout`/`stderr` must never exceed
`max_output_bytes`.

### 6.3 Required observable outcome per scenario

| Scenario | `run_bounded` returns | `timed_out` | `output_limited` | `exit_code` | `success()` |
|---|---|---|---|---|---|
| Child exits `0`, both pipes reach EOF, within the deadline | `Ok` | `false` | `false` | `Some(0)` | `true` |
| Child exits non-zero, both pipes reach EOF, within the deadline | `Ok` | `false` | `false` | `Some(code)` | `false` |
| Child still running at the deadline | `Ok` | `true` | `false` unless the bound was also exceeded | `None` on a signal death; the observed code on a platform that reports one | `false` |
| A stream exceeded `max_output_bytes` | `Ok` | `false` unless the deadline also elapsed | `true` | as observed | `false` |
| **Direct child exited, a descendant still holds the pipe open** | `Ok` | `true` | `false` unless the bound was also exceeded | `Some(code)` of the direct child, when observed | `false` |
| Spawn failure | `Err(Error::Io { operation: "spawning bounded candidate", .. })` | — | — | — | — |
| Poll failure | `Err(Error::Io { operation: "polling bounded candidate", .. })` | — | — | — | — |

The last two rows are unchanged from `candidate.rs:139-141` and
`candidate.rs:180-183`; the second row is unchanged from today's `exit_code`
behavior. Only the deadline rows change. The four failure classes remain
distinguishable by the caller: **deadline expiry** — `timed_out == true`;
**output overflow** — `output_limited == true` with `timed_out == false`;
**spawn failure** —
`Err(Error::Io { operation: "spawning bounded candidate", .. })`; **non-zero
exit** — `exit_code == Some(code)` with `code != 0` and both flags `false`.

### 6.4 The descendant case is bounded and fail-closed

When the direct child has exited but a descendant still holds the stdout or
stderr pipe open: the call **returns** at the deadline (§6.1); the result is
`Ok(CommandOutput)` with `timed_out == true` and `exit_code == Some(code)` of
the direct child — the child is not killed because it already exited, and the
caller can still see what it returned; `success()` is `false`, so the validator
path fails closed; and the pending stream contributes no bytes (they were never
captured; the caller must not be told they were).

`timed_out` therefore keeps its operational meaning — *the deadline was
reached, so the bounded execution did not complete* — and its rustdoc wording is
amended (§12) to state that the direct child is killed **if it is still
running**. The flag is not given a second, unrelated meaning, and `success()`
keeps its current three-part definition (`candidate.rs:119-121`). A caller that
must distinguish "the deadline elapsed while the child was running" from "the
deadline elapsed after the child exited" can already do so through `exit_code`
(`None` versus `Some(code)`); no new public API is required by M014.

### 6.5 Reader-thread lifecycle is explicit

- A reader thread that has delivered its result is **joined**. The join is
  bounded: after the send, the thread only has to return.
- A reader thread still blocked in `read()` at the deadline **cannot be joined**
  without making the caller wait past the deadline, and core cannot terminate a
  descendant it does not own: `eggup-core` has no `libc` / `windows-sys`
  dependency (so no `killpg`, no job object), and `unsafe_code` is denied. Such
  a reader is therefore **abandoned explicitly**, and the abandonment must be
  stated truthfully in the `run_bounded` rustdoc: the thread ends when the pipe
  reaches EOF, which is when the holding descendant exits or closes the pipe.
- The number of reader threads per call stays at two (`candidate.rs:153-158`).
- The implementer must not pretend an abandoned reader is reclaimed. §15
  requires the closure to record the residual honestly, and §14 makes "the
  abandonment is unacceptable" a stop condition.

### 6.6 The validator path is unchanged

`ExactIdentityValidator::validate` (`candidate.rs:310-331`) and
`CrossMemberAgreementValidator::validate` (`candidate.rs:363-367`) keep
converting a non-successful `CommandOutput` into
`Err(Error::CandidateExecution(...))` (`candidate.rs:320-327`). The descendant
case therefore surfaces to a caller of `ValidatedTransaction::validate` as
`Err(Error::CandidateExecution(...))`, exactly as a timeout does today.

## 7. Ordered work packages

- **WP1 — Deterministic fixture, first.** Add
  `crates/eggup-core/tests/candidate_deadline.rs` with a helper that spawns a
  descendant holding the stdout/stderr pipe open while the direct child exits
  immediately. It must be cross-platform (Linux, macOS, **and** Windows), must
  not depend on a shell, `sh`, `cmd.exe`, or any external tool, must bound the
  descendant's lifetime, must not depend on host scheduling pressure for its
  discriminating assertion (§10), and must **fail against the pre-M014
  implementation** (§11). A workable mechanism is to re-execute the test binary
  itself with an environment-variable-selected helper mode
  (`std::env::current_exe()`), which is shell-free and works on all three
  platforms; another mechanism is acceptable and must be recorded in the closure
  together with its residual process lifetime.
- **WP2 — Deadline semantics.** Implement §6.1–§6.4 in `run_bounded`: evaluate
  the deadline unconditionally, kill/reap only a still-running child, leave the
  loop without waiting for EOF, report the results required by §6.3.
- **WP3 — Thread lifecycle.** Implement §6.5: join every reader that has
  delivered, abandon only those still blocked, keep the thread count at two.
- **WP4 — Documentation and the remaining fixtures.** Write the rustdoc
  contract (§12), add the timeout / overflow / spawn-failure / non-zero-exit /
  normal-exit classification fixtures (§10), and add the `Unreleased` changelog
  entries.
- **WP5 — Regression sweep.** Run every command in §11, including the
  baseline-failure proof and the hosted lane matrix, and write the closure
  record (§15).

WP1 precedes WP2 deliberately: the fixture must exist and must fail against the
baseline before the production change, so the closure shows a real before/after
rather than a claim.

## 8. Failure, cancellation, restart, and contention semantics

- **A deadline expiry is never a success.** `success()` is `false`
  (`candidate.rs:119-121`) and the two validators turn it into
  `Err(Error::CandidateExecution(...))` (`candidate.rs:320-327`). No path lets a
  timeout, an overflow, or an unreaped pipe yield a validated transaction.
- **Fail-closed direction.** Every uncertain outcome — deadline expiry, output
  overflow, a pipe that never closed — is reported as a failure with an explicit
  flag, never as a partially-trusted success.
- **Spawn failure stays an `Err`** (`candidate.rs:139-141`): nothing was
  spawned, nothing was mutated. **Poll failure stays an `Err`**
  (`candidate.rs:180-183`): the child may still be running, so this path must
  not report a result; the existing behavior is unchanged and M014 does not
  widen it. If the implementer finds that the poll-failure path must kill the
  child to avoid an orphan, that is an additive change to an error path and must
  be justified in the closure record.
- **Transaction state.** Candidate validation runs before the mutation lock is
  acquired and before any live destination is touched (`candidate.rs:412-423`;
  `architecture/core-transaction.md:644`). A deadline expiry therefore affects
  **no** transaction state: no lock is held, no backup set exists, no receipt is
  produced, no live file is modified. The caller receives
  `Err(Error::CandidateExecution(...))` and may retry under its own policy; core
  imposes no retry.
- **Restart semantics** are unchanged: `run_bounded` holds no persistent state
  and creates no file, so a failed candidate leaves nothing to clean up beyond
  the direct child, which is killed and reaped at the deadline.
- **Contention** is unchanged: `run_bounded` does not take the mutation lock and
  does not interact with the lock record.
- **Descendants are not terminated.** Core kills and reaps the direct child it
  spawned. A descendant that inherited the pipe survives the call. This must be
  stated in the rustdoc, not implied away.
- **Cancellation**: core has no cancellation token for candidate execution and
  M014 does not add one. A caller that needs one must own the process
  lifecycle itself.

## 9. Compatibility and migration

- **`eggup-core 0.1.3` is published** (crates.io, published 2026-10-06 from
  `bd43683`; `plans/closure/verified-update-core/013-status.md`). M014 changes
  packaged behavior in `src/candidate.rs`, so the published `0.1.3` **retains
  the defect**: a consumer resolving `eggup-core 0.1.3` can still hang. The fix
  reaches consumers only when a new version is published.
- **M014 must not publish.** Publication is a tracked milestone of its own
  (`.opencode/skills/release-workflow`, `architecture/tooling-governance.md`
  §8); M013
  (`plans/implementation/verified-update-core/013-eggup-core-0.1.3-publication.md`)
  is the precedent. M014's closure record must state that the published `0.1.3`
  is still affected and that republication is authorized only by a separate
  milestone.
- **Source compatibility.** Unchanged. `CommandSpec`, `CommandOutput`, and its
  accessors keep their published shape; `Error` keeps its seven published
  variants; no public item is added or removed. An exhaustive downstream
  `match` on `Error` still compiles — the property M012 protected, re-proven by
  the closure (§13).
- **Behavior compatibility.** Behavior-changing for existing callers only in the
  case that is currently a hang: a call that today never returns now returns at
  the deadline with `timed_out == true`. No caller can depend on a hang, so this
  is a defect correction, not a semantic break.
- **The `eggup-eggpack` pin blocks propagation.** `eggup-eggpack` pins
  `eggup-core = { version = "=0.1.3", path = "../eggup-core" }`
  (`crates/eggup-eggpack/Cargo.toml:20`; likewise `eggup-archive` and
  `eggup-acquisition` at `:21-22`). A core fix therefore **cannot reach
  `eggup-eggpack`** until a separate adapter republication re-pins the new core
  version. That republication is its own milestone, sequenced after the core
  publication.
- **Packaged bytes.** The new integration test file is excluded from the package
  (`crates/eggup-core/Cargo.toml:15-19`), so only the `src/` change alters the
  packaged artifact. The `Unreleased` entries in `crates/eggup-core/CHANGELOG.md`
  and the root `CHANGELOG.md` must state that the fix is unpublished and that
  no publication milestone authorizes it.

## 10. Required tests

All new fixtures live in `crates/eggup-core/tests/candidate_deadline.rs` (plus
the existing unit fixtures in `crates/eggup-core/src/lib.rs` where they already
live). At minimum:

1. **Descendant-holds-pipe fixture (the discriminating test).** A helper spawns
   a descendant that inherits stdout/stderr and holds it open; the direct child
   exits immediately with a known code. The fixture asserts that `run_bounded`
   **returns** and that the result is `timed_out == true`,
   `exit_code == Some(<known code>)`, `output_limited == false`,
   `success() == false`.
   - The discriminating assertion is **"the call returns"**, detected by running
     the call on a worker thread and waiting for completion with a **generous**
     bound whose expiry is a **failure**. It is not "the call returns within N
     milliseconds". A loaded host can only delay the fixed implementation; it
     cannot make it hang. This is the discipline acquisition M012 established,
     and M014 must not invert it.
   - The fixture must **fail against the pre-M014 implementation** (§11): under
     the baseline the call never returns, so the bounded wait expires and the
     fixture reports the hang.
   - The fixture must not depend on a shell and must run natively on Linux,
     macOS, and Windows.
2. **Timeout kills and reaps the direct child.** A child outliving its deadline
   yields `timed_out == true`, `success() == false`, and a bounded elapsed time.
   The elapsed bound must be generous (the deadline plus a wide allowance), not
   a tight millisecond ceiling.
3. **Output overflow.** A child writing past `max_output_bytes` yields
   `output_limited == true`, `timed_out == false`, `success() == false`, and
   empty captured bytes for the overflowing stream — pinning the published
   `read_limited` semantics recorded in §3.4.
4. **Spawn failure.** A non-existent program yields
   `Err(Error::Io { operation: "spawning bounded candidate", .. })` — the
   operation string is asserted, not merely the variant.
5. **Non-zero exit is distinguishable.** A child exiting `3` yields
   `exit_code == Some(3)`, `timed_out == false`, `output_limited == false`,
   `success() == false`.
6. **Normal exit is unchanged.** A child printing a known string and exiting `0`
   yields `exit_code == Some(0)`, `success() == true`, and the exact stdout.
7. **The validator path still fails closed.** A candidate that exceeds its
   deadline surfaces through `ExactIdentityValidator` as
   `Err(Error::CandidateExecution(...))`.
8. **Reader-thread lifecycle.** The normal timeout path must not retain reader
   threads: after a timeout with no descendant the call returns and the readers
   have ended. The assertion must be observable without a platform-specific
   thread counter (for example, by asserting the call returns and recording the
   reasoning in the closure); if the implementer can prove thread reclamation
   portably, that proof is required, and if not, the closure must say so plainly.
9. **Pre-M014 failure proof.** Fixture 1 must fail against the baseline tree;
   fixtures 2–6 are regression guards expected to pass on both trees. The
   closure record must show the verbatim baseline failure output.

Existing candidate fixtures (`crates/eggup-core/src/lib.rs:1291-1414`,
including `bounded_runner_kills_timeouts_and_limits_output` at `:1342-1367`)
must remain green and must not be rewritten to accommodate the new behavior.

## 11. Required verification commands

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p eggup-core --locked
cargo test -p eggup-core --locked candidate
cargo test -p eggup-core --locked --test candidate_deadline
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo doc -p eggup-core --no-deps --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo tree -p eggup-core --locked
git diff --check
./scripts/check-local.sh
```

`cargo tree -p eggup-core --locked` is a review step, not a gate: it must show
`sha2` only, plus `self-replace` on Windows. The new fixtures' test names must
contain the substring `candidate`, so that
`cargo test -p eggup-core --locked candidate` selects them as well as
`cargo test -p eggup-core --locked --test candidate_deadline`.

### 11.1 Proof that the new fixture fails against the pre-M014 baseline

```text
git worktree add /tmp/eggup-m014-baseline c4805411f74b85158e0ae5d0a219a32c4209d98e
cp crates/eggup-core/tests/candidate_deadline.rs /tmp/eggup-m014-baseline/crates/eggup-core/tests/candidate_deadline.rs
cd /tmp/eggup-m014-baseline && cargo test -p eggup-core --locked --test candidate_deadline
```

Expected: the descendant-holds-pipe fixture **fails** against the baseline,
because `run_bounded` never returns there. The exit status must be non-zero and
the verbatim failure text must be copied into the closure record. After the
proof:

```text
git worktree remove /tmp/eggup-m014-baseline
```

A `git stash`-based run is acceptable as an alternative, but the closure record
must name the exact mechanism used and show the verbatim failure.

### 11.2 Hosted qualification

A fresh hosted run of `.github/workflows/ci.yml` is required, covering the
`stable`, `msrv`, `macos`, and `windows-check` lanes. The `windows-check` lane
runs `cargo test` **without** `--all-features`, so the new fixture must not
depend on a feature; that lane's final workspace step
(`cargo test --workspace --all-targets --locked --no-fail-fast`) must be
confirmed to have executed the new test file, and the test name must be visible
in the lane log. A lane that was not run is **not run**, never inferred.

## 12. Documentation updates

- `crates/eggup-core/src/candidate.rs:63-67` — the `CommandSpec::timeout`
  rustdoc must state that the value is an unconditional maximum wall-clock bound
  on the whole bounded execution, including the case where the direct child has
  already exited, and that the deadline starts after the spawn returns (§3.4).
- `crates/eggup-core/src/candidate.rs:124` — the `run_bounded` rustdoc must
  state the unconditional deadline, the descendant case, the reader-thread
  abandonment (§6.5), the bounded teardown allowance (§6.1), and what the caller
  observes in each case (§6.3).
- `crates/eggup-core/src/candidate.rs:108-111` — the `CommandOutput::timed_out`
  rustdoc must be amended so that it is true in both deadline cases: the
  deadline was reached, and the direct child is killed **if it was still
  running**. The flag's operational meaning and `success()`'s definition are
  unchanged.
- `crates/eggup-core/src/candidate.rs:69` — the `max_output_bytes` rustdoc must
  say unambiguously that the bound is per stream (§3.4).
- `crates/eggup-core/CHANGELOG.md` — an `Unreleased` entry describing the
  bounded-candidate deadline correction, the observable change, the unpublished
  status, and the absence of a migration path. The root `CHANGELOG.md`
  `Unreleased` section (`:222`) carries the matching aggregate entry, because an
  unpublished fix in an already-published crate must appear in both.
- `architecture/core-transaction.md:302-307` — the `run_bounded` paragraph must
  state the unconditional deadline and the descendant caveat, and the export
  table row at `:117` must not contradict it.
- `crates/eggup-core/docs/verification.md:32-34` — the candidate-command
  paragraph must state the unconditional bound. `docs/` is non-normative: it
  points at the contract and must not restate a new rule. No new rule may be
  introduced in `docs/`; the normative statements stay in the
  `crates/eggup-core/src/candidate.rs` rustdoc, `architecture/core-transaction.md`,
  and `crates/eggup-core/docs/`.

## 13. Acceptance criteria

M014 closes only when:

- the deadline is enforced unconditionally, including when the direct child has
  already exited, and `run_bounded` returns in the descendant-holds-pipe case
  with `timed_out == true`, `exit_code == Some(<the direct child's code>)`,
  `output_limited == false`, and `success() == false`;
- deadline expiry, output overflow, spawn failure, and non-zero exit remain
  distinguishable through the observables in §6.3;
- a reader that has delivered its result is joined, and a reader still blocked
  at the deadline is abandoned explicitly and documented truthfully (§6.5);
- every invariant of §4 holds, with the ownership model, the transaction model,
  the receipt semantics, the output bound, and `Error`'s seven published
  variants unchanged, and an exhaustive downstream `match` on `Error` still
  compiling;
- `cargo tree -p eggup-core --locked` shows no new dependency edge;
- the §11 commands are green, including `./scripts/check-local.sh`;
- the §11.1 baseline proof shows the new fixture failing against
  `c4805411f74b85158e0ae5d0a219a32c4209d98e`, with the verbatim failure text in
  the closure record;
- a fresh hosted `stable` / `msrv` / `macos` / `windows-check` matrix is green,
  with the new fixture observed executing in the Windows lane;
- `crates/eggup-core/CHANGELOG.md` and the root `CHANGELOG.md` carry the
  `Unreleased` entries;
- no medium-or-higher finding related to bounded candidate execution remains
  open, and the `eggup-service` duplication (§3.5) is recorded as a follow-up
  with its severity.

## 14. Stop conditions

Stop and write another plan if:

- the deterministic fixture cannot be made schedule-independent, or can only be
  made deterministic by asserting a tight millisecond ceiling;
- the fixture requires a shell, `sh`, `cmd.exe`, `sleep`, `kill`, or any other
  external tool not guaranteed on all three target platforms;
- the fix requires a new dependency (notably `libc`, `nix`, or `windows-sys`) to
  obtain process-group, job-object, or timed-read semantics, or requires
  `unsafe` or FFI;
- the fix would require adding an `Error` variant or otherwise changing the
  published `Error` surface — that is a 0.2/1.0 API-boundary decision, not a
  patch;
- the fix would change the `Ownership` enum, the ownership model, the
  transaction state machine, the lock, or the receipt semantics;
- the fix would require changing `eggup-service`, `eggup-curl`, or any other
  crate;
- the fix requires widening a timeout, adding a tolerance, a retry, or a second
  deadline to make a test pass — if it does, the §3 analysis is wrong; re-derive
  it and stop rather than patching the symptom;
- the implementer concludes that descendant termination is required for
  correctness, which would mean the abandonment in §6.5 is not acceptable and
  the constraint set must be revisited in a separate milestone;
- the fix would change the published overflow byte semantics or the per-stream
  bound.

Do not classify a genuine runtime failure as test flakiness to restore a green
baseline.

## 15. Closure evidence required

`plans/closure/verified-update-core/014-status.md` must contain everything
`plans/closure/README.md` requires, and in addition:

- a **requirement → evidence matrix** mapping each requirement of §6 and §10 to
  the fixture or command that proves it, and the **exact commands** of §11 with
  their results, verbatim;
- the **baseline-failure proof**: the §11.1 command sequence, the verbatim
  failure text, and the non-zero exit status;
- **separate Linux, macOS, and Windows results** for the new fixture, each from
  a native run. A lane that was not run is recorded as **not run** and is never
  inferred from another platform; the hosted run identifier and the
  per-platform job conclusions, including the confirmation that the Windows lane
  executed the new test file;
- an **invariant review** against §4, item by item, including the explicit
  statement that the ownership model, the transaction model, and the output bound
  are unchanged;
- a **failure/rollback/recovery review**: what a deadline expiry does to
  transaction state (nothing — no lock is held, no receipt exists), and the
  reader-thread residual recorded truthfully (§6.5);
- a **compatibility and migration review**: `Error` still has exactly the seven
  published variants and an exhaustive downstream `match` still compiles;
  `eggup-core 0.1.3` on crates.io still carries the defect; republication is a
  separate milestone; `eggup-eggpack`'s `=0.1.3` pin blocks propagation until a
  separate adapter republication;
- a **security review**: the environment is still cleared (`candidate.rs:129`),
  stdin is still null, captured output is still bounded, no credential can be
  carried by a partially-captured stream, and the change adds no dependency and
  no `unsafe`;
- **documentation evidence**: the rustdoc, the two `Unreleased` changelog
  entries, `architecture/core-transaction.md`, and
  `crates/eggup-core/docs/verification.md`, with the diff summary;
- **unresolved findings with severity**, explicitly including the `eggup-service`
  duplicate runner (§3.5), the reader-error/`output_limited` conflation
  (§3.4), the per-stream bound wording, the uncovered spawn, and the residual
  thread lifetime — each with the reason for its severity;
- the roadmap and registry reconciliation, including the new registry entry 3 and
  the milestone's status.

## 16. Handoff notes

- **M014 does not publish.** The published `eggup-core 0.1.3` retains the
  defect until a separate publication milestone ships a new version; the
  closure must say so plainly rather than implying the defect is fixed for
  consumers. Core publication is a separate milestone (M013 is the precedent),
  and because `eggup-eggpack` pins `eggup-core = "=0.1.3"`
  (`crates/eggup-eggpack/Cargo.toml:20`), the adapter republication that
  re-pins the fixed core is a further milestone after the core publication.
- **Follow-up service milestone.** `eggup-service` has its own bounded
  subprocess runner — `SystemExecutor` (`crates/eggup-service/src/lib.rs:671`),
  `fn run` (`:700`), `read_bounded_generic` (`:952`) — while already depending
  on `eggup-core`. A service milestone may consolidate it onto
  `eggup-core::run_bounded`, but it must first resolve the semantic asymmetry
  recorded in §3.4 (core discards the overflowing stream's bytes; the service
  retains up to `limit + 8192` bytes) and re-check the service's own deadline
  contract. That is a service-scoped decision and is **not** authorized by this
  plan.
- **`eggup-curl`'s copy is justified** (`crates/eggup-curl/src/lib.rs:594`):
  `eggup-curl` may depend only on `eggup-acquisition` and must not depend on
  `eggup-core`. Do not consolidate it.
- **Not authorized by this plan:** anything listed in §5 "Explicitly out of
  scope" — publication or version bumps; any change to `eggup-service`,
  `eggup-curl`, `eggup-eggpack`, `eggup-archive`, `eggup-acquisition`, or
  `eggup-eggfetch`; any new dependency; any `unsafe`; any change to the
  `Error` variant set, the `Ownership` enum, the transaction state machine, or
  the output bound; any timeout tuning; any restructuring of `eggup-core`'s
  module layout; any CI step removal or weakening.
