# Verified Update Core Milestone 017 — bounded stdin execution corrective

Status: implemented

Repository baseline: `20ac770dacfdcd3d82e734c22613bfccd36f9c09`

Source roadmap:

- `plans/subsystems/verified-update-core-roadmap.md`

Long-term requirements:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`

Applicable ADRs:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`
- `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md`

Primary class: capability

## 1. Objective

`eggup-core`'s bounded candidate runner (`run_bounded`, `crates/eggup-core/src/candidate.rs`)
executes a child with **null stdin**. It cannot give a child any input at all.
A downstream adapter that owns a workflow which genuinely requires a child to
read a payload — a service manager that installs a unit or a crontab from
standard input — therefore cannot delegate process execution to core without
losing that capability: it has to keep, or re-implement, its own process
runner.

The concrete downstream consumer is `eggup-service`'s `SystemExecutor`
(`crates/eggup-service/src/lib.rs:1016`), whose `run` takes an
`stdin_data: Option<&[u8]>` and writes it through a dedicated writer thread
(`crates/eggup-service/src/lib.rs:1086-1118`). `eggup-service` already depends
on `eggup-core`, so the duplication is not required by any dependency-boundary
rule; the missing piece is **core's** bounded stdin capability, not a policy
decision about where the runner lives.

Target contract after M017:

- `CommandSpec` can carry a **bounded stdin payload**, attached with one
  infallible builder method;
- the payload bound is enforced **before any child is spawned**, so an
  oversized payload fails closed with no process created;
- the payload is written by a dedicated writer thread, and the existing
  unconditional wall-clock deadline bounds that writer exactly as it bounds the
  output readers;
- a writer failure — including the broken pipe a child produces when it exits
  without reading its stdin — kills and reaps the child and fails closed with a
  **typed** error;
- the three `Error::Io` `operation` strings that `run_bounded` produces become
  a **stable classification surface** a downstream adapter may branch on;
- the published `Error` variant set, the transaction model, the ownership
  model, the deadline semantics, and the output bound are all unchanged.

Nothing else changes. In particular this milestone does **not** touch
`eggup-service`: consolidating `SystemExecutor` onto core's primitive is a
separate service milestone (service M015) and is explicitly out of scope here
(§5).

## 2. Why this milestone is ready

- **The gap is fully characterized and read-verified.** It was found by
  reading the baseline tree at `20ac770`: `run_bounded` sets
  `.stdin(Stdio::null())` unconditionally (`crates/eggup-core/src/candidate.rs:197`)
  and `CommandSpec` has no stdin field at all
  (`crates/eggup-core/src/candidate.rs:19-26`). No production incident is
  known and none is claimed.
- **The downstream consumer already exists and already has the semantics.**
  `eggup-service`'s `SystemExecutor::run` writes a bounded stdin payload
  through a dedicated writer thread, kills and reaps the child when the write
  fails, and bounds the write with the same deadline
  (`crates/eggup-service/src/lib.rs:1086-1118`, `:1119-1155`). The behavior
  this milestone adds to core is therefore a **transplant of an already
  specified and already working mechanism**, not a new design.
- **Core's M014 deadline work is the prerequisite, and it is closed.**
  `run_bounded`'s deadline is now an unconditional maximum bound on the whole
  bounded execution (`plans/closure/verified-update-core/014-status.md`), and
  the reader-thread lifecycle is explicit: a reader that delivered is joined, a
  reader still blocked at the deadline is abandoned explicitly and truthfully.
  A stdin **writer** needs exactly the same treatment, and that treatment is
  now available to copy.
- **Scope is bounded.** One production file in one crate
  (`crates/eggup-core/src/candidate.rs`), one crate-root re-export line, one
  new integration test file under `crates/eggup-core/tests/`, no dependency
  change, no other crate, no publication.
- **Severity is medium.** The published `eggup-core 0.1.3` cannot express a
  bounded stdin write at all, so a downstream adapter must either keep a
  duplicated runner or lose the capability. That is a duplication and
  capability gap, not a safety hole: nothing today misbehaves, and no
  published consumer can observe a wrong result. Per this repository's gate,
  "medium-or-higher" forces a corrective before a milestone closes, so the
  capability must be delivered in core rather than left to each adapter.

## 3. Current implementation evidence

### 3.1 Core's bounded runner has no stdin

`crates/eggup-core/src/candidate.rs:19-26`:

```rust
pub struct CommandSpec {
    program: PathBuf,
    args: Vec<String>,
    current_dir: Option<PathBuf>,
    timeout: Duration,
    max_output_bytes: usize,
    environment: Vec<(String, String)>,
}
```

There is no stdin field, and `run_bounded` (`candidate.rs:192-199`) configures
the child with:

```rust
command
    .args(&spec.args)
    .env_clear()
    .stdin(Stdio::null())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());
```

`Stdio::null()` is unconditional, so the child's stdin is always the null
device: a child that reads stdin sees EOF immediately and can never receive a
payload.

### 3.2 The duplicated runner in `eggup-service` already has the semantics

`crates/eggup-service/src/lib.rs:1016-1118` (`SystemExecutor::run`):

- the payload is bounded by `MAX_CRONTAB_BYTES` (1 MiB,
  `crates/eggup-service/src/lib.rs:897`) and rejected before the spawn
  (`:1032-1036`);
- stdin is piped only when a payload is present, otherwise null
  (`:1048-1052`);
- the reader threads are attached **before** the stdin write, so a child that
  fills stdout/stderr before draining stdin cannot deadlock both sides
  (`:1065-1067`);
- the write runs on its own thread because a blocking `write_all` on a pipe
  cannot be interrupted from the writing thread, and only closing the child's
  read end — which `child.kill()` does — unblocks it (`:1086-1113`);
- a completed write that failed fails closed: the child is killed and reaped
  and the call returns an error (`:1122-1131`).

That mechanism is what M017 transplants into core, with core's own bound, core's
own deadline, and core's own typed error.

### 3.3 The error surface this milestone must not disturb

`crates/eggup-core/src/error.rs:28-63`: `Error` is a **closed** enum with
exactly the seven published variants — `InvalidInput`, `UnknownMember`, `Io`,
`UpdateInProgress`, `DestinationConflict`, `VerificationFailed`,
`CandidateExecution` — plus the `#[cfg(test)]`-only `Injected`. `Error::Io`
carries a public `operation: &'static str` and a public `source: io::Error`,
and `std::error::Error::source` already returns the inner `io::Error`
(`error.rs:216-229`), so a downstream consumer can recover the OS error and
re-classify it.

`Error::Io` is therefore already a usable classification surface; what is
missing is a **published promise** that the `operation` strings are stable.
M017 supplies that promise for the three strings `run_bounded` produces.

### 3.4 Observations M017 does not silently inherit

Recorded so M017 does not inherit them unnoticed. Only the first is in M017's
scope.

- **The writer failure and the output overflow can both be true at once.** If a
  child writes past `max_output_bytes` *and* exits without reading its stdin,
  two failures are live. M017 must pick one deterministically and document the
  precedence (§6.4).
- **A write can succeed against a child that never read it.** A payload smaller
  than the OS pipe buffer is accepted by the kernel even when the child exits
  immediately, so "the write succeeded" is not evidence that "the child
  consumed the payload". Core can observe the write, not the consumption.
  M017 must not claim more than that (§6.3).
- **The bound is per call, not per stream.** `max_output_bytes` bounds stdout
  and stderr independently (`candidate.rs:217-225`). The new stdin bound is a
  separate, single, per-call bound.
- **The deadline starts after the spawn.** `deadline` is computed at
  `candidate.rs:227`, after `command.spawn()` returned, so a blocking spawn is
  not covered. Unchanged by M017.
- **A reader I/O error is reported as `output_limited`.** Unchanged by M017.

## 4. Invariants that must not regress

- `CommandSpec::timeout` remains an **unconditional maximum** wall-clock bound
  on the whole bounded execution, evaluated whether or not the direct child has
  exited. A stdin writer does not get a second, longer deadline (§6.5).
- The output bound is unchanged: `max_output_bytes` still bounds each stream,
  overflow still discards that stream's bytes, and the defaults remain 64 KiB
  (`candidate.rs:15`) and 5 s (`candidate.rs:14`).
- `CommandOutput`'s field meanings are stable: `timed_out` means the deadline
  was the cause of the bounded execution not completing; `output_limited` means
  an output bound was exceeded; `success()` means exited `0`, no timeout, no
  overflow (`candidate.rs:138-141`).
- `CommandOutput`'s published shape is unchanged — same fields, same
  accessors. No new public type is required by M017.
- `Error`'s published variant set is unchanged: exactly the seven published
  variants. **Adding an `Error` variant is out of scope and is a stop
  condition** (§14). The new stdin failure is reported through the existing
  `Error::Io` and `Error::InvalidInput` variants.
- `FailureCategory` is untouched and stays `#[non_exhaustive]`.
- The ownership model is untouched: `Absent | Owned | Foreign | Unknown` stays
  caller-proven through `OwnershipVerifier`, `Owned` is never inferred.
- Core never creates a live destination parent, and candidate execution never
  mutates a live destination. Validation runs before the mutation lock is taken
  (`candidate.rs:539-551`), so a stdin failure can never leave a partially
  applied transaction.
- `run_bounded` keeps clearing the inherited environment (`candidate.rs:195`)
  and keeps null stdin when no payload is attached (§6.2).
- `eggup-core` keeps its dependency surface: `sha2` only, plus `self-replace` on
  Windows (`crates/eggup-core/Cargo.toml:24-33`). **No new dependency of any
  kind**, no `build.rs`, no FFI, no `unsafe`.
- The workspace denies `unsafe_code` and `eggup-core` carries
  `#![forbid(unsafe_code)]` and `#![deny(missing_docs)]`
  (`crates/eggup-core/src/lib.rs:1-2`); every new public item needs rustdoc,
  and every public fallible item needs an `# Errors` section (crate-local
  `clippy::missing_errors_doc`, `crates/eggup-core/src/lib.rs:8`). Rust 1.89
  remains the MSRV.
- SHA-256 remains integrity evidence only. No authenticity or signature claim
  is introduced anywhere.
- The four documented `run_bounded` outcomes are preserved; the new stdin
  `Err` case is added to the `# Errors` section, not to the outcome list.

## 5. Scope

### In scope

- One production file: `crates/eggup-core/src/candidate.rs` — the public
  constant `MAX_STDIN_BYTES`, the `CommandSpec::stdin_data` builder, the
  `run_bounded` stdin path, and the rustdoc for all three.
- One crate-root re-export line in `crates/eggup-core/src/lib.rs` (the crate
  root re-exports every public item of `candidate`, so the new constant is
  re-exported with them).
- One new integration test file, `crates/eggup-core/tests/candidate_stdin.rs`.
  Integration tests are excluded from the packaged artifact
  (`crates/eggup-core/Cargo.toml:15-19`), so the fixture does not change
  published bytes.
- The rustdoc contract, the `eggup-core` `CHANGELOG.md` `Unreleased` entry, the
  `eggup-core` `README.md`, the candidate-command paragraph of
  `crates/eggup-core/docs/verification.md`, and the candidate-execution
  paragraph plus the export table of `architecture/core-transaction.md`.
- The closure record, and the plan's own status line.

### Explicitly out of scope

- **No change to `eggup-service`.** `SystemExecutor`
  (`crates/eggup-service/src/lib.rs:1016`), its `run` (`:1017`), its stdin
  writer thread (`:1086-1118`), and `read_bounded_generic` are **not** touched
  by this milestone. Consolidating `SystemExecutor` onto core's primitive is
  service milestone **M015**, which is planned in parallel and must not be
  affected by this edit.
- **No change to `eggup-acquisition`, `eggup-curl`, `eggup-eggfetch`,
  `eggup-archive`, or `eggup-eggpack`.**
- **No new `Error` variant** and no change to `FailureCategory`.
- **No new dependency** — in particular no `libc`, `nix`, or `windows-sys`, and
  no process-group or job-object mechanism.
- **No change to the transaction, staging, locking, commit, rollback, or
  ownership machinery**, and no change to the deadline semantics or the output
  bound.
- **No publication, no version bump, no tag movement.** `eggup-core` is
  published at `0.1.3` and that published crate keeps whatever this milestone
  changes (§9).
- **No consumer-facing how-to rewrite.** `docs/` is non-normative and must not
  become the place a rule is introduced (§12).
- **No registry, roadmap, or root-changelog edit.** `plans/registry.md`,
  `plans/subsystems/**`, and the root `CHANGELOG.md` are owned by the parent
  session; the intended prose is drafted in the closure record instead (§15).

## 6. Required production changes

This section states required **behavior**, not implementation. The implementer
chooses the mechanism; the observables below are fixed.

### 6.1 The public constant and the builder

- A public constant `MAX_STDIN_BYTES: usize = 1024 * 1024` (1 MiB) in
  `crates/eggup-core/src/candidate.rs`, re-exported from the crate root with
  the other `candidate` exports.
- `CommandSpec` gains one **private** field `stdin_data: Option<Vec<u8>>`,
  defaulting to `None`, and one public **infallible** builder method:

```rust
pub fn stdin_data(mut self, data: impl Into<Vec<u8>>) -> Self
```

The builder is infallible on purpose: the bound is enforced at execution time,
so an oversized payload is rejected by `run_bounded` — which is the only place
that can fail closed without creating a child.

### 6.2 No payload means unchanged behavior

When no payload is attached, the child's stdin stays `Stdio::null()`, no writer
thread is created, and `run_bounded` behaves exactly as it does at the
baseline. This is the M014 behavior, unchanged.

### 6.3 An attached payload is written by a dedicated writer thread

When a payload is attached:

- the child's stdin becomes `Stdio::piped()`;
- a **dedicated writer thread** performs `write_all` followed by `flush`, and
  the pipe's write end is closed when that thread ends, so the child observes
  EOF;
- the writer thread is started **after** the stdout and stderr readers are
  attached, so a child that fills stdout or stderr before draining its stdin
  cannot deadlock both sides;
- the payload is written as-is. Core observes that the write succeeded; it
  does **not** claim that the child consumed the payload, and it does not
  claim anything about the payload's content.

### 6.4 The bound is enforced before the spawn; the write failure is typed

- A payload larger than `MAX_STDIN_BYTES` is rejected with
  `Err(Error::InvalidInput("stdin data exceeds the bounded stdin bound"))`
  **before any child is spawned**. Fail closed: no process is created.
- A payload of exactly `MAX_STDIN_BYTES` is accepted (the bound is inclusive).
- A writer failure — including the broken pipe a child produces when it exits
  without reading its stdin — must kill and reap the child and then return
  `Err(Error::Io { operation: "writing bounded stdin", source })`.
- When a writer failure and an output overflow are live at the same time, the
  **writer failure wins**: it is an `Err`, and the caller must not be told the
  execution merely overflowed. The precedence must be documented, not left to
  the reader of the code.

### 6.5 The deadline bounds the writer exactly as it bounds the readers

The deadline stays unconditional and bounds the whole bounded execution,
including the stdin write:

- when the deadline is reached, the child is killed — which breaks the child's
  stdin pipe and unblocks the writer — and the writer thread that is still
  blocked in `write` is **abandoned explicitly**, exactly like a reader blocked
  on a pipe a surviving descendant holds;
- a writer that delivered its result is joined, which is bounded, because after
  the send the thread only has to return;
- a completed bounded execution is not reported until the writer has delivered
  its verdict, so a write failure is never lost behind a "the child exited"
  observation;
- the deadline outcome stays `Ok` with `timed_out` set. A writer failure
  detected **after** the deadline was reached does not retroactively turn the
  deadline outcome into an `Err`: the deadline already decided the outcome.

### 6.6 The three operation strings are a stable classification surface

The `operation` string of `Error::Io` is a published, stable classification
surface for `run_bounded`. A downstream adapter may branch on it. The three
strings are:

| `operation` | Meaning | What a downstream adapter is expected to do |
| --- | --- | --- |
| `"spawning bounded candidate"` | the child could not be spawned | treat it as an environment problem; nothing was created, so retrying under the caller's own policy is safe |
| `"polling bounded candidate"` | the child's status could not be polled | do not treat the execution as complete; the child may still be running |
| `"writing bounded stdin"` | the stdin payload could not be written, including the broken pipe a child produces when it exits without reading its stdin | treat the payload as **not delivered**: the child was killed and reaped, so the execution is not complete and the result must not be trusted |

These strings must be documented as stable in the `run_bounded` rustdoc (the
`Error::Io` variant's own rustdoc lives in `crates/eggup-core/src/error.rs`,
which this milestone does not own) and restated as a contract in
`crates/eggup-core/docs/verification.md` and
`architecture/core-transaction.md`.

### 6.7 The four documented outcomes are preserved

`run_bounded` still reports four distinguishable outcomes through
`CommandOutput` — the child ran to completion; the deadline was reached; a
stream exceeded `max_output_bytes`; and — with the stdin addition — the
stdin write failed, which is an `Err`, not a fourth `Ok` shape. The
`# Errors` section gains the two new cases: `Error::InvalidInput` for an
oversized payload and `Error::Io` with `operation` `"writing bounded stdin"`
for a failed write.

## 7. Ordered work packages

- **WP1 — The plan and the contract.** Write this plan, fix the observable
  contract (§6), and name the exact public API (§6.1).
- **WP2 — The production change.** Implement §6.1–§6.7 in
  `crates/eggup-core/src/candidate.rs`: the constant, the private field, the
  builder, the bound check before the spawn, the writer thread, the writer
  polling in the loop, the typed write failure, the deadline bound on the
  writer, and the rustdoc. Re-export the constant from the crate root.
- **WP3 — The fixtures.** Add `crates/eggup-core/tests/candidate_stdin.rs`
  with the delivery fixture, the oversized-payload fixture, the write-failure
  fixture, the null-stdin fixture, the deadline fixture, and the at-the-bound
  fixture (§10).
- **WP4 — The contract documentation.** Update the rustdoc (§12),
  `crates/eggup-core/docs/verification.md`,
  `architecture/core-transaction.md`, `crates/eggup-core/README.md`, and the
  `eggup-core` `CHANGELOG.md` `Unreleased` section.
- **WP5 — The regression sweep.** Run every command in §11, write the closure
  record (§15), and flip this plan's status to `implemented`.

WP2 precedes WP3 deliberately: the fixture must exist against the implemented
contract, not against a guess at it.

## 8. Failure, cancellation, restart, and contention semantics

- **A stdin failure is never a success.** The writer failure is an `Err`, so
  `run_bounded` returns no `CommandOutput` at all. `success()` is never
  consulted for that call, and the validators
  (`ExactIdentityValidator::validate`, `candidate.rs:428-451`) convert the
  error into `Err(Error::CandidateExecution(...))` for their own callers.
- **An oversized payload is never a success and never a process.** The
  `Error::InvalidInput` is returned before the spawn, so no child exists.
- **Fail-closed direction.** Every uncertain outcome — deadline expiry, output
  overflow, a failed stdin write, an oversized payload — is reported as a
  failure, never as a partially-trusted success.
- **The child is killed and reaped on a writer failure.** The payload was not
  delivered, so leaving the child running would be an unbounded process. Core
  kills only the direct child it spawned; a descendant is never killed.
- **Transaction state.** Candidate validation runs before the mutation lock is
  acquired and before any live destination is touched
  (`candidate.rs:539-551`). A stdin failure therefore affects **no**
  transaction state: no lock is held, no backup set exists, no receipt is
  produced, no live file is modified. The caller receives an `Err` and may
  retry under its own policy; core imposes no retry.
- **Restart semantics** are unchanged: `run_bounded` holds no persistent state
  and creates no file.
- **Contention** is unchanged: `run_bounded` does not take the mutation lock.
- **Cancellation**: core has no cancellation token for candidate execution and
  M017 does not add one.
- **Abandonment is truthful.** A writer still blocked at the deadline is
  abandoned, not reclaimed, and the rustdoc says so.

## 9. Compatibility and migration

- **Additive source compatibility.** `CommandSpec` gains one private field and
  one public builder method; `MAX_STDIN_BYTES` is a new public constant. No
  published item changes shape, and no published item is removed. `Error` keeps
  exactly the seven published variants, so an exhaustive downstream `match` on
  `Error` still compiles — the property M012 protected, re-proven by the
  closure (§13).
- **Behavior compatibility.** A call that attaches no payload behaves exactly
  as before: null stdin, no writer thread, the same four outcomes. A call that
  attaches a payload is a **new** capability; no published caller can be
  affected by it.
- **The published `eggup-core 0.1.3` still lacks the capability.**
  `eggup-core 0.1.3` is published (crates.io, published 2026-10-06 from
  `bd43683`; `plans/closure/verified-update-core/013-status.md`). M017 changes
  packaged behavior in `src/candidate.rs`, so the published `0.1.3` still
  cannot express a bounded stdin write. The capability reaches consumers only
  when a new version is published.
- **M017 must not publish.** Publication is a tracked milestone of its own
  (M013 is the precedent); the closure record must state that the published
  `0.1.3` is still affected and that republication is authorized only by a
  separate milestone.
- **The `eggup-eggpack` pin blocks propagation.** `eggup-eggpack` pins
  `eggup-core = { version = "=0.1.3", path = "../eggup-core" }`
  (`crates/eggup-eggpack/Cargo.toml`). A core fix therefore cannot reach
  `eggup-eggpack` until a separate adapter republication re-pins the new core
  version. That republication is its own milestone.
- **Packaged bytes.** The new integration test file is excluded from the
  package (`crates/eggup-core/Cargo.toml:15-19`), so only the `src/` change
  alters the packaged artifact.
- **Migration path: none required.** No consumer action is needed beyond
  picking up the fixed version; the new builder is optional.

## 10. Required tests

All new fixtures live in `crates/eggup-core/tests/candidate_stdin.rs`. Every
fixture name contains the substring `candidate`, so
`cargo test -p eggup-core --locked candidate` selects them all.

The fixtures must be cross-platform: Linux, macOS, **and** Windows. `sh -c` is
not available on Windows, so the fixture re-executes **this very test binary**
as its helper and selects the helper role from the helper's own **argv** — not
from the environment, because `run_bounded` clears the inherited environment
(`candidate.rs:195`). No shell, no `cmd.exe`, and no external tool is used.
A Unix-only fixture may be added for a Unix-only observation, but it must not
be the only coverage of any requirement.

1. **The payload is delivered and the child's echo comes back on stdout.**
   A helper reads all of stdin and writes those exact bytes to stdout. The
   fixture asserts the child's stdout ends with the exact payload bytes.
2. **An oversized payload is rejected and no child is spawned.**
   A payload of `MAX_STDIN_BYTES + 1` yields
   `Err(Error::InvalidInput("stdin data exceeds the bounded stdin bound"))`,
   and the helper's "I started" marker file is **not** created — the proof
   that no child was spawned.
3. **A payload of exactly `MAX_STDIN_BYTES` is accepted.** The bound is
   inclusive; the child reports the exact byte count it read.
4. **A child that exits without reading its stdin yields the typed `Err`.**
   The error is `Error::Io` with `operation == "writing bounded stdin"`, and
   the underlying `io::Error` is recoverable through `std::error::Error::source`.
   The payload used must be larger than the OS pipe buffer so that the write
   cannot silently succeed, and the fixture must not depend on the exact
   `io::ErrorKind`, which differs across platforms.
5. **With no payload attached, stdin is still null and the behavior is
   unchanged.** The child observes EOF on stdin and reads zero bytes, the exit
   code is observable, and `success()` is `true`.
6. **The deadline still bounds a child that never reads a large stdin
   payload.** The outcome is the documented one: `Ok`, `timed_out == true`,
   `output_limited == false`, `success() == false`, and the call returns
   within a generous ceiling.
7. **The existing core test suite still passes**, including
   `crates/eggup-core/tests/candidate_deadline.rs`, which encodes the M014
   deadline guarantees. **No existing assertion may be weakened or deleted.**

## 11. Required verification commands

```text
cargo fmt -p eggup-core -- --check
cargo clippy -p eggup-core --all-targets --locked --offline -- -D warnings
cargo test -p eggup-core --locked --offline
cargo test -p eggup-core --locked --offline candidate
cargo test -p eggup-core --locked --offline --test candidate_stdin
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
fixture must not depend on a feature; the lane's final workspace step must be
confirmed to have executed the new test file, and the test names must be
visible in the lane log. A lane that was not run is **not run**, never
inferred.

## 12. Documentation updates

- `crates/eggup-core/src/candidate.rs` — `MAX_STDIN_BYTES`: the bound, its
  inclusive edge, and the fact that an oversized payload is rejected before
  any child is spawned.
- `crates/eggup-core/src/candidate.rs` — `CommandSpec::stdin_data`: the
  infallible builder, the execution-time bound, and the fact that no payload
  means null stdin.
- `crates/eggup-core/src/candidate.rs` — `run_bounded`: the stdin capability,
  the writer thread, the deadline bound on the writer, the abandoned-writer
  rule, the new `# Errors` cases, and the **stable operation-string contract**
  (§6.6) with the three strings and what a downstream adapter is expected to
  do with each.
- `crates/eggup-core/src/lib.rs` — the crate-root re-export of
  `MAX_STDIN_BYTES`.
- `crates/eggup-core/docs/verification.md` — the candidate-command paragraph
  must state the bounded stdin capability and the stable operation strings,
  and must continue to point at the `run_bounded` rustdoc as the normative
  statement rather than restating a new rule.
- `architecture/core-transaction.md` — the candidate-execution paragraph and
  the export table must state the bounded stdin capability and the stable
  operation strings.
- `crates/eggup-core/README.md` — one sentence recording that the bounded
  runner can deliver a bounded stdin payload.
- `crates/eggup-core/CHANGELOG.md` — an `Unreleased` entry describing the new
  capability, the bound, the typed write failure, the unchanged public error
  surface, the unpublished status, and the absence of a migration path.

## 13. Acceptance criteria

M017 closes only when:

- `CommandSpec::stdin_data` attaches a payload, and `run_bounded` delivers it
  to the child; the child's echo of it comes back on stdout;
- a payload larger than `MAX_STDIN_BYTES` yields
  `Err(Error::InvalidInput("stdin data exceeds the bounded stdin bound"))` and
  **no child is spawned**;
- a child that exits without reading its stdin yields
  `Err(Error::Io { operation: "writing bounded stdin", .. })`, with the child
  killed and reaped;
- with no payload attached, stdin is null and the existing behavior is
  unchanged;
- the deadline still bounds a child that never reads a large stdin payload,
  with the documented outcome;
- the three operation strings are documented as a stable classification
  surface, with the expected downstream handling for each;
- every invariant of §4 holds, with the ownership model, the transaction
  model, the deadline semantics, the output bound, `Error`'s seven published
  variants, and `FailureCategory` unchanged, and an exhaustive downstream
  `match` on `Error` still compiling;
- `cargo tree -p eggup-core --locked --offline --depth 1` shows no new
  dependency edge;
- the §11 commands are green;
- the existing core suite, including `candidate_deadline.rs`, is green with no
  assertion weakened or deleted;
- no medium-or-higher finding related to this milestone remains open.

## 14. Stop conditions

Stop and write another plan if:

- the capability would require a new dependency, `unsafe`, FFI, or a
  `build.rs`;
- the capability would require adding an `Error` variant or otherwise
  changing the published `Error` surface — that is a 0.2/1.0 API-boundary
  decision, not a patch;
- the capability would change the `Ownership` enum, the ownership model, the
  transaction state machine, the lock, or the receipt semantics;
- the capability would require changing `eggup-service`, `eggup-curl`, or any
  other crate — in particular, the `SystemExecutor` consolidation is service
  M015 and must not be pulled into this milestone;
- the writer thread cannot be bounded by the existing deadline without
  widening it, adding a tolerance, or adding a second deadline;
- the fix would require weakening, deleting, or loosening an existing
  assertion in `crates/eggup-core/tests/candidate_deadline.rs` or in the
  crate's own test module;
- the fix would change the published overflow byte semantics, the per-stream
  output bound, or the deadline semantics;
- the fixture can only be made deterministic by asserting a tight millisecond
  ceiling, or requires a shell, `sh`, `cmd.exe`, `sleep`, `kill`, or any
  other external tool not guaranteed on all three target platforms.

Do not classify a genuine runtime failure as test flakiness to restore a green
baseline.

## 15. Closure evidence required

`plans/closure/verified-update-core/017-status.md` must contain everything
`plans/closure/README.md` requires, and in addition:

- a **requirement → evidence matrix** mapping each requirement of §6 and §10 to
  the fixture or command that proves it, with a `file:line` anchor and the
  **exact command and its result**;
- **separate Linux, macOS, and Windows results**, each from a native run. A
  platform that was not run is recorded as **not run** and is never inferred
  from another platform;
- an **invariant review** against §4, item by item;
- a **failure/rollback/recovery review**: what a stdin failure does to
  transaction state (nothing), and the abandoned-writer residual recorded
  truthfully;
- a **compatibility and migration review**: exactly what changes for a consumer
  of the published `eggup-core 0.1.3` (the new builder and constant are
  additive; the published crate still lacks them until republication; no
  `Error` variant changed), and the explicit statement that no republication
  was performed;
- a **security review**: the environment is still cleared, stdin is still null
  without a payload, the payload is bounded, the captured output is still
  bounded, and the change adds no dependency and no `unsafe`;
- **documentation/operations evidence**: a table of every document changed and
  what changed in it;
- **unresolved findings with severity**, with the reason for each severity;
  anything still open at medium-or-higher severity must be named as a follow-on
  corrective;
- the **roadmap disposition** and the **registry updates**, drafted as prose
  for the parent session — `plans/registry.md`,
  `plans/subsystems/verified-update-core-roadmap.md`, and the root
  `CHANGELOG.md` are owned by the parent session and are **not** edited here;
- the `## Parent-session verification addendum` heading is **not** written by
  this milestone; the parent adds it after the hosted CI qualification.

## 16. Handoff notes

- **M017 does not publish.** The published `eggup-core 0.1.3` still cannot
  express a bounded stdin write; the capability reaches consumers only when a
  new version is published, and no publication milestone authorizes that here.
- **M017 does not touch `eggup-service`.** The downstream consumer named in
  §1 is `eggup-service`'s `SystemExecutor`, and its consolidation onto core's
  primitive is **service milestone M015**, which is planned in parallel. M017
  supplies the core capability and nothing else; service M015 must consume the
  API exactly as specified in §6.1, §6.4, and §6.6.
- **The follow-ons M017 names are:**
  1. **Service M015 — `SystemExecutor` consolidation onto core's bounded
     runner.** This is the consumer of the new capability and the place the
     duplicated runner (`crates/eggup-service/src/lib.rs:1016-1118`) is
     removed. Severity of the *duplicate runner* as a standing finding:
     **medium** — it is the reason the capability was missing from core in the
     first place, and it will keep the two runners' failure semantics apart
     until it lands.
  2. **A core republication milestone** that publishes the new capability, and
     the matching `eggup-eggpack` / `eggup-archive` / `eggup-acquisition`
     republication that re-pins `eggup-core`. Severity: **medium** — until it
     lands, no published consumer has the capability. This is the same
     publication cascade every prior core change has named.
  3. **A core republication milestone for the M014 deadline correction and the
     M015 contract corrections**, which remain unpublished for the same reason.
     Severity: **medium** — carried forward from M014 and M015, unchanged by
     M017.
- **The operation strings are now a promise.** §6.6 turns three
  `&'static str` values into a published contract. A future change to any of
  them is a source-visible break for an adapter that branches on it, and must
  be treated as an API change.
