# Verified Update Core — Milestone 017 closure record

## Status

**closed** — hosted-qualified in CI run `40062299630` (commit `b9885fa`), all
five lanes green, including native Linux and Windows executions of the new
bounded-stdin fixture. The parent-session addendum at the end of this record
supersedes the earlier macOS-only qualification and finding **F1**.

No medium-or-higher finding about this milestone's own change remains open.
Two medium findings remain open about the *publication* of this change and
about the downstream consolidation; they are named as follow-ons in F2 and F3
and are not defects of this milestone.

## Source plan and roadmap

- Plan: `plans/implementation/verified-update-core/017-bounded-stdin-execution-corrective.md`
- Roadmap: `plans/subsystems/verified-update-core-roadmap.md`
- Long-term requirements: `plans/000-long-term-specification.md`,
  `plans/001-terminology-and-domain-model.md`
- Applicable ADRs: `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`,
  `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md`

## Reviewed repository baseline

- Baseline commit: `20ac770dacfdcd3d82e734c22613bfccd36f9c09`
  (`docs(closure): record the hosted qualification of the corrective batch`)
- Branch: `plans/corrective-batch-2026-10-23`
- The tree was **not clean** at the baseline: the batch's parallel milestones
  had already written `architecture/acquisition.md`,
  `architecture/curl-adapter.md`, `architecture/eggfetch-adapter.md`,
  `crates/eggup-curl/**`, and the untracked
  `plans/implementation/acquisition-transport/**`,
  `plans/implementation/service-lifecycle/015-bounded-executor-consolidation-corrective.md`,
  `plans/implementation/transport-footprint/**`, and
  `plans/subsystems/transport-footprint-roadmap.md`.
  Those changes are **not** part of this milestone and were not touched by it;
  `git status --short` was read before and after every edit to confirm the
  ownership boundary held.

## Implementation commits / PRs

**None.** The parent session owns every commit, every push, and every tag in
this batch; no git command that mutates repository state was run by this
milestone (`git stash`, `git checkout <path>`, `git restore`, `git commit`,
`git push`, `git tag`, `git clean`, `git reset`, and branch creation were all
forbidden and none was executed). The change is present in the working tree
only.

Files this milestone created or changed:

| File | Change |
| --- | --- |
| `crates/eggup-core/src/candidate.rs` | Production change: the constant, the private field, the builder, the bound check, the writer thread, the writer polling, the typed write failure, the deadline bound on the writer, and the rustdoc. |
| `crates/eggup-core/src/lib.rs` | Crate-root re-export of `MAX_STDIN_BYTES` (one line). |
| `crates/eggup-core/tests/candidate_stdin.rs` | **New.** Six fixtures plus four helper roles. |
| `crates/eggup-core/docs/verification.md` | Candidate-command paragraph: the bounded stdin capability and the stable operation strings. |
| `crates/eggup-core/README.md` | One paragraph on the bounded runner's stdin capability and the stable operation strings. |
| `crates/eggup-core/CHANGELOG.md` | `## Unreleased` entry for M017. |
| `architecture/core-transaction.md` | Candidate-execution section, export table, and failure-semantics table: the bounded stdin capability, the stable operation strings, and two new failure rows. |
| `plans/implementation/verified-update-core/017-bounded-stdin-execution-corrective.md` | **New.** The plan. |
| `plans/closure/verified-update-core/017-status.md` | **New.** This record. |

No file outside `crates/eggup-core/**`, `architecture/core-transaction.md`, and
`plans/{implementation,closure}/verified-update-core/` was written. In
particular `crates/eggup-service/**` was not touched — see the compatibility
review.

## Executive finding

Core's bounded candidate runner gave every child a **null** stdin, so core
could not express "run this command and give it this payload" at all. The
downstream consumer that needs it — `eggup-service`'s `SystemExecutor`
(`crates/eggup-service/src/lib.rs:1016`) — therefore carries its own process
runner with its own bounded stdin write
(`crates/eggup-service/src/lib.rs:1086-1118`). That duplication is not forced
by any dependency-boundary rule: `eggup-service` already depends on
`eggup-core`.

M017 transplants the mechanism into core, with core's own bound, core's own
unconditional deadline, and core's own typed error:

- `CommandSpec::stdin_data` attaches a bounded payload (infallible builder);
- `MAX_STDIN_BYTES` (1 MiB, inclusive) bounds it, and an oversized payload is
  rejected with `Error::InvalidInput` **before any child is spawned**;
- a dedicated writer thread performs the write, and the deadline bounds that
  writer exactly as it bounds the output readers;
- a failed write — including the broken pipe of a child that exits without
  reading its stdin — kills and reaps the child and fails closed with
  `Error::Io { operation: "writing bounded stdin", source }`;
- the three `Error::Io` `operation` strings `run_bounded` produces are now a
  **published, stable classification surface** a downstream adapter may branch
  on.

Nothing else moved. `Error` keeps exactly the seven published variants, so an
exhaustive downstream `match` on `Error` still compiles. `FailureCategory` is
untouched. The transaction, staging, locking, commit, rollback, and ownership
machinery are untouched. The dependency surface is unchanged. No publication,
no version bump, and no tag movement was performed.

The six new fixtures are green, and each was shown to **discriminate**: two
deliberate mutations of the production code were applied, run, and reverted, and
each mutation broke the fixtures that are supposed to catch it (§ "Fixture
discrimination proof").

## Requirement → evidence matrix

| # | Requirement (plan §6 / §10) | Evidence | Command and result |
| --- | --- | --- | --- |
| R1 | `MAX_STDIN_BYTES` is a public constant of 1 MiB, re-exported from the crate root | `crates/eggup-core/src/candidate.rs:27`; `crates/eggup-core/src/lib.rs:34` | `cargo doc -p eggup-core --no-deps --locked --offline` → exit 0, `MAX_STDIN_BYTES` documented in the rendered rustdoc; `cargo clippy -p eggup-core --all-targets --locked --offline -- -D warnings` → exit 0 (`#![deny(missing_docs)]` satisfied) |
| R2 | `CommandSpec` carries a private `stdin_data: Option<Vec<u8>>` defaulting to `None`, and one infallible builder | `crates/eggup-core/src/candidate.rs:38`, `:125` | `cargo clippy … -- -D warnings` → exit 0; `cargo test -p eggup-core --locked --offline` → 103 passed |
| R3 | The payload is delivered to the child and the child's echo comes back on stdout | `crates/eggup-core/tests/candidate_stdin.rs:229` (`candidate_stdin_payload_is_delivered_to_the_child`) | `cargo test -p eggup-core --locked --offline --test candidate_stdin` → **10 passed; 0 failed** |
| R4 | A payload larger than `MAX_STDIN_BYTES` yields `Err(Error::InvalidInput("stdin data exceeds the bounded stdin bound"))` **and no child is spawned** | `crates/eggup-core/src/candidate.rs:281-287`; `crates/eggup-core/tests/candidate_stdin.rs:317` (`candidate_stdin_oversized_payload_is_rejected_without_spawning`) | `cargo test -p eggup-core --locked --offline --test candidate_stdin` → **10 passed; 0 failed** (the fixture also asserts the helper's start marker is absent after a 500 ms settle) |
| R5 | A payload of exactly `MAX_STDIN_BYTES` is accepted and delivered in full (the bound is inclusive) | `crates/eggup-core/tests/candidate_stdin.rs:250` (`candidate_stdin_payload_at_the_bound_is_accepted`) | `cargo test -p eggup-core --locked --offline --test candidate_stdin` → **10 passed; 0 failed** |
| R6 | A child that exits without reading its stdin yields the typed `Err` with operation `"writing bounded stdin"`, and the child is killed and reaped | `crates/eggup-core/src/candidate.rs:375-385`, `:439`; `crates/eggup-core/tests/candidate_stdin.rs:345` (`candidate_stdin_write_failure_reports_the_operation`) | `cargo test -p eggup-core --locked --offline --test candidate_stdin` → **10 passed; 0 failed** (the fixture also asserts the underlying `io::Error` is preserved and is not `ErrorKind::Other`) |
| R7 | With no payload attached, stdin is null and the existing behavior is unchanged | `crates/eggup-core/src/candidate.rs:292-298`; `crates/eggup-core/tests/candidate_stdin.rs:255` (`candidate_stdin_without_a_payload_leaves_stdin_null`) | `cargo test -p eggup-core --locked --offline --test candidate_stdin` → **10 passed; 0 failed** (the child observes EOF and reads 0 bytes; exit code 0; `success()` is `true`) |
| R8 | The deadline still bounds a child that never reads a large stdin payload, with the documented outcome | `crates/eggup-core/src/candidate.rs:404-411`; `crates/eggup-core/tests/candidate_stdin.rs:377` (`candidate_stdin_deadline_bounds_a_child_that_never_reads_stdin`) | `cargo test -p eggup-core --locked --offline --test candidate_stdin` → **10 passed; 0 failed** (`Ok`, `timed_out == true`, `output_limited == false`, `success() == false`, elapsed below the generous ceiling) |
| R9 | The three `operation` strings are documented as a stable classification surface, with the expected downstream handling for each | `crates/eggup-core/src/candidate.rs:238-258` (the `# Error classification` rustdoc section); `crates/eggup-core/docs/verification.md:32-45`; `architecture/core-transaction.md:323-350` | `cargo doc -p eggup-core --no-deps --locked --offline` → exit 0; `./scripts/check-docs-truth.sh` → exit 0 |
| R10 | The existing core test suite still passes, with no assertion weakened or deleted | `crates/eggup-core/tests/candidate_deadline.rs` is unmodified (`git status --short` shows no change to it) | `cargo test -p eggup-core --locked --offline` → **103 passed; 0 failed** across 6 test binaries; `cargo test -p eggup-core --locked --offline candidate` → **25 passed; 0 failed** |
| R11 | No new dependency edge | `crates/eggup-core/Cargo.toml:24-33` unchanged | `cargo tree -p eggup-core --locked --offline --depth 1` → `eggup-core v0.1.3` → `sha2 v0.10.9` only |
| R12 | `Error` keeps exactly the seven published variants; `FailureCategory` is unchanged | `crates/eggup-core/src/error.rs` is unmodified (`git status --short` shows no change to it) | `cargo clippy -p eggup-core --all-targets --locked --offline -- -D warnings` → exit 0; `cargo test -p eggup-core --locked --offline` → 103 passed |
| R13 | `eggup-service` is unaffected by this edit | `crates/eggup-service/**` is unmodified (`git status --short` shows no change to it) | `cargo check -p eggup-service --all-targets --locked --offline` → exit 0 (`Checking eggup-service v0.1.3 … Finished`) |

## Production implementation evidence

### The public surface

```rust
// crates/eggup-core/src/candidate.rs:27
pub const MAX_STDIN_BYTES: usize = 1024 * 1024;

// crates/eggup-core/src/candidate.rs:125
pub fn stdin_data(mut self, data: impl Into<Vec<u8>>) -> Self
```

`CommandSpec` gained exactly one **private** field,
`stdin_data: Option<Vec<u8>>` (`candidate.rs:38`), defaulting to `None` in
`CommandSpec::new` (`candidate.rs:51`). The builder is infallible on purpose:
the bound is a runtime property of the execution, and only `run_bounded` can
reject it without creating a child.

`MAX_STDIN_BYTES` is re-exported from the crate root
(`crates/eggup-core/src/lib.rs:34`) alongside the crate's other `candidate`
exports, matching the crate's convention that every public item of the module
is reachable as `eggup_core::<item>`.

### The bound, enforced before the spawn

`crates/eggup-core/src/candidate.rs:281-287`:

```rust
// The stdin bound is enforced before anything is spawned, so an oversized
// payload fails closed with no child created.
if let Some(data) = &spec.stdin_data {
    if data.len() > MAX_STDIN_BYTES {
        return Err(Error::InvalidInput(
            "stdin data exceeds the bounded stdin bound".into(),
        ));
    }
}
```

The check is the **first** statement of `run_bounded`, before `Command::new`,
before the spawn, and therefore before any process exists. The comparison is
`>`, so a payload of exactly `MAX_STDIN_BYTES` is accepted (R5). The error uses
the already-published `Error::InvalidInput` variant; **no `Error` variant was
added**.

### The writer thread

`crates/eggup-core/src/candidate.rs:316-347`:

- `child.stdin.take()` is performed **before** the readers are spawned, so the
  writer owns the pipe handle. A missing stdin pipe is reported as
  `Error::CandidateExecution("missing stdin pipe")` after the child is killed
  and reaped.
- The stdout and stderr reader threads are spawned **first**
  (`candidate.rs:333-338`), and only then is the writer thread spawned
  (`candidate.rs:339-347`). This ordering is what prevents a child that fills
  stdout or stderr before draining its stdin from deadlocking both sides.
- The writer performs `write_all` followed by `flush`, and sends its verdict
  over a channel. The `ChildStdin` handle is dropped when the thread ends,
  which closes the write end, so the child observes EOF.
- The writer thread's body cannot panic (no `unwrap`, no indexing, no
  assertion) and its send is unconditional, so the channel cannot disconnect
  without a verdict. `poll_writer` (`candidate.rs:459-483`) nevertheless
  treats a disconnected channel as a write failure, so the outcome fails
  closed rather than silently succeeding.

### The deadline bounds the writer

- The writer is polled in the **same** loop that polls the readers and the
  child's status (`candidate.rs:360-369`), so it is bounded by the same
  unconditional wall-clock deadline.
- The "completed" break now additionally requires `stdin_finished`
  (`candidate.rs:395-400`): a bounded execution is not reported as complete
  until the writer has delivered its verdict, so a write failure is never lost
  behind a "the child exited" observation.
- At the deadline the child is killed and reaped (`candidate.rs:404-411`),
  which breaks the child's stdin pipe and unblocks the writer. A writer that is
  still blocked in `write` is **abandoned explicitly** (`candidate.rs:426-437`),
  exactly like a reader blocked on a pipe a surviving descendant holds: a
  writer that delivered is joined, and one that has not is left to end when the
  pipe breaks. Core terminates only the direct child it spawned and owns no
  process group, so it cannot reclaim a writer blocked behind a surviving
  descendant.
- A writer failure detected **after** the deadline was reached does not
  retroactively turn the deadline outcome into an `Err`: the deadline already
  decided that outcome. This is stated in the rustdoc
  (`candidate.rs:203-205`).

### The write failure is typed and fails closed

`crates/eggup-core/src/candidate.rs:375-385`:

```rust
if let Some(Err(source)) = stdin_result.take() {
    // A failed stdin write decides the outcome: the payload was not
    // delivered, so the child is killed and reaped and the call fails
    // closed rather than reporting a partial success.
    stdin_error = Some(source);
    if status.is_none() {
        let _ = child.kill();
        status = child.wait().ok();
    }
    break;
}
```

and `candidate.rs:439`:

```rust
return Err(Error::io("writing bounded stdin", source));
```

The check runs **before** the output-overflow check in the loop body, so a
failed write wins over a concurrent overflow. That precedence is deliberate and
documented (`candidate.rs:256-258`): the write failure is an `Err`, and the
caller must not be told that the execution merely overflowed when the payload
was in fact never delivered.

### The stable operation strings

`crates/eggup-core/src/candidate.rs:238-258` is a new `# Error classification`
rustdoc section on `run_bounded`. It states that the `operation` string of
`Error::Io` is a stable classification surface a downstream adapter may branch
on, names the three strings, and says what an adapter is expected to do with
each:

| `operation` | Documented downstream expectation |
| --- | --- |
| `"spawning bounded candidate"` | Treat it as an environment problem; nothing was created, so retrying under the caller's own policy is safe. |
| `"polling bounded candidate"` | Do not treat the execution as complete; the child may still be running. |
| `"writing bounded stdin"` | Treat the payload as **not delivered**: the child was killed and reaped, so the execution is not complete and the result must not be trusted. |

The contract is stated on `run_bounded` rather than on `Error::Io` because
`crates/eggup-core/src/error.rs` is **not** owned by this milestone; the plan
recorded that constraint in §6.6 and the placement is the plan's own choice.

### Fixture discrimination proof

The six fixtures were shown to discriminate, by applying two deliberate
mutations to the production code, running the fixture file, and reverting each
mutation. Both mutations were reverted before the verification sweep below; the
final tree contains neither.

**Mutation 1 — stdin capability removed** (`Stdio::piped()` replaced by
`Stdio::null()` unconditionally in `run_bounded`):

```text
cargo test -p eggup-core --locked --offline --test candidate_stdin
→ test result: FAILED. 6 passed; 4 failed; 0 ignored
   FAILED candidate_stdin_deadline_bounds_a_child_that_never_reads_stdin
   FAILED candidate_stdin_payload_at_the_bound_is_accepted
   FAILED candidate_stdin_write_failure_reports_the_operation
   FAILED candidate_stdin_payload_is_delivered_to_the_child
```

The two fixtures that did **not** fail are the two that are *supposed* to be
insensitive to that mutation: the oversized-payload fixture (the bound check
happens before the stdin wiring) and the null-stdin fixture (it asserts the
no-payload behavior, which the mutation preserves).

**Mutation 2 — the write performed synchronously on the calling thread**
(the writer thread replaced by an inline `write_all` on the caller's stack):

```text
cargo test -p eggup-core --locked --offline --test candidate_stdin candidate_stdin_deadline
→ test result: FAILED. 0 passed; 1 failed
   FAILED candidate_stdin_deadline_bounds_a_child_that_never_reads_stdin
```

This is the decisive proof that the deadline fixture guards the property it
claims: with the write on the caller's stack, a child that never reads its
stdin pins the bounded call past its own deadline. The dedicated writer thread
is what keeps the deadline an unconditional maximum bound on the whole bounded
execution.

## Exact commands run and results

All commands were run from `/Users/davidbowman/projects/eggup` on the baseline
tree `20ac770dacfdcd3d82e734c22613bfccd36f9c09` with the change in the working
tree. Toolchain: `rustc 1.89.0 (29483883e 2025-08-04)` / `cargo 1.89.0`,
macOS 26.6.2 (build 25G83), arm64 — the workspace MSRV toolchain.

### 1. `cargo fmt -p eggup-core -- --check`

```text
EXIT=0
```

### 2. `cargo clippy -p eggup-core --all-targets --locked --offline -- -D warnings`

```text
Checking eggup-core v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-core)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
EXIT=0
```

The crate-local `clippy::missing_errors_doc` (M015) is enabled for
`eggup-core`, so this run also proves that the new public surface carries the
required rustdoc.

### 3. `cargo test -p eggup-core --locked --offline`

```text
running 56 tests
test result: ok. 56 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.74s
running 12 tests
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.02s
running 10 tests
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.51s
running 11 tests
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.73s
running 14 tests
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**103 passed, 0 failed** across 6 test binaries. The 10-test binary is the new
`candidate_stdin.rs`; the 11-test binary is the pre-existing
`candidate_deadline.rs`, which is unmodified.

### 4. `cargo test -p eggup-core --locked --offline candidate`

```text
running 2 tests
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 54 filtered out; finished in 0.27s
running 12 tests
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.03s
running 10 tests
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.51s
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.06s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.00s
```

**25 passed, 0 failed.** Every new fixture name contains the substring
`candidate`, so this filter selects all six of them.

### 5. `cargo test -p eggup-core --locked --offline --test candidate_stdin`

```text
running 10 tests
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.51s
```

10 = 4 helper roles (inert unless the parent selected them) + 6 fixtures.

### 6. Stability of the new fixture file

```text
for i in $(seq 1 10); do cargo test -p eggup-core --locked --offline --test candidate_stdin; done
→ test result: ok. 10 passed; 0 failed; …   (× 10, every run)
```

11 consecutive green runs (one in step 5 plus ten here). The fixture file is
stable on this host; it is not asserted to be stable on a loaded host, which is
why the fixture's discriminating assertion is "the call returns" and not "the
call returns quickly".

### 7. `cargo doc -p eggup-core --no-deps --locked --offline`

```text
Documenting eggup-core v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-core)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.25s
Generated /Users/davidbowman/projects/eggup/target/doc/eggup_core/index.html
```

No rustdoc warnings. The new `# Error classification` section renders, and the
intra-doc links to `MAX_STDIN_BYTES`, `CommandSpec::stdin_data`,
`Error::InvalidInput`, and `Error::Io` resolve.

### 8. `cargo tree -p eggup-core --locked --offline --depth 1` (review step, not a gate)

```text
eggup-core v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-core)
└── sha2 v0.10.9
```

**Dependency-boundary result: unchanged.** `sha2` only on this (non-Windows)
target; `self-replace` remains the sole Windows-only edge
(`crates/eggup-core/Cargo.toml:31-33`). No new dependency of any kind was
added, no `build.rs` was added, and `unsafe` remains denied
(`crates/eggup-core/src/lib.rs:1`).

### 9. `git diff --check`

```text
EXIT=0
```

### 10. `./scripts/check-docs-truth.sh` (read-only repository check)

```text
note 7: skipped — pass --baseline <ref> to scope the no-production-delta check
documentation-truth checks passed
EXIT=0
```

### 11. `cargo check -p eggup-service --all-targets --locked --offline`

```text
Checking eggup-core v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-core)
Checking eggup-service v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-service)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.77s
```

Run because `eggup-service` is the downstream consumer this capability exists
for. It compiles unchanged against the modified core, which is the required
"additive" property.

### 12. Not run by this milestone

- `./scripts/check-local.sh` and the other workspace-wide gate steps — owned by
  the parent session (this milestone's plan §11).
- Every hosted CI lane, and the Linux and Windows platforms — see F1.

## Invariant review (plan §4, item by item)

| Invariant | Result | Evidence |
| --- | --- | --- |
| The deadline remains an unconditional maximum bound on the whole bounded execution | **Holds** | The deadline is still evaluated on every poll whether or not the child exited (`candidate.rs:404-411`); the writer is polled in the same loop and is bounded by it (R8, mutation 2). |
| The output bound is unchanged: per stream, overflow discards that stream's bytes | **Holds** | `read_limited` and `poll_reader` are untouched; `max_output_bytes` and its default (64 KiB) are untouched. |
| `CommandOutput`'s field meanings and published shape are unchanged | **Holds** | No field, accessor, or constructor of `CommandOutput` changed. |
| `Error` keeps exactly the seven published variants | **Holds** | `crates/eggup-core/src/error.rs` is unmodified. The new failures use `Error::InvalidInput` and `Error::Io`. |
| `FailureCategory` is untouched and stays `#[non_exhaustive]` | **Holds** | `error.rs` is unmodified. |
| The ownership model is untouched; `Owned` is never inferred | **Holds** | `domain.rs` and the validators' ownership handling are unmodified. |
| Core never creates a live destination parent; candidate execution never mutates a live destination | **Holds** | Validation runs before the mutation lock is taken and before any live mutation; the stdin path only adds a child-process input. |
| The inherited environment is still cleared | **Holds** | `.env_clear()` is still applied (`candidate.rs:291`); the stdin change is orthogonal to it. |
| Stdin stays null without a payload | **Holds** | R7. |
| `eggup-core` keeps its dependency surface; no new dependency, no `unsafe` | **Holds** | Step 8; `crates/eggup-core/src/lib.rs:1` still carries `#![forbid(unsafe_code)]`. |
| Rust 1.89 remains the MSRV | **Holds** | The whole verification sweep ran on 1.89.0. |
| Every new public item has rustdoc; every public fallible item has `# Errors` | **Holds** | Steps 1, 2 and 7 (`#![deny(missing_docs)]` plus the crate-local `clippy::missing_errors_doc`). |
| SHA-256 remains integrity evidence only | **Holds** | No authenticity, signature, or provenance claim was added anywhere. |
| The four documented `run_bounded` outcomes are preserved | **Holds** | The outcome list is unchanged (`candidate.rs:227-236`); the new stdin `Err` case was added to the `# Errors` section, not to the outcome list. |

## Failure / rollback / recovery review

- **A stdin failure is never a success.** The writer failure is an `Err`, so
  `run_bounded` returns no `CommandOutput` at all and `success()` is never
  consulted for that call.
- **A failed write kills and reaps the child** (`candidate.rs:380-383`). The
  payload was not delivered, so leaving the child running would be an
  unbounded process. Core kills only the direct child it spawned; a descendant
  is never killed.
- **Transaction state is untouched by every new failure.** Candidate
  validation runs before the mutation lock is acquired and before any live
  destination is touched, so an oversized payload, a failed write, and a
  spawn failure all occur with **no lock held, no backup set, no receipt, and
  no live file modified**. The caller receives an `Err` and may retry under its
  own policy; core imposes no retry and no backoff.
- **No rollback is needed**, because no live mutation can precede these
  failures. Core's rollback and recovery machinery was not touched.
- **Recovery from an abandoned writer.** A writer still blocked in `write` at
  the deadline is abandoned, not reclaimed, exactly like a reader blocked on a
  pipe a surviving descendant holds. It ends when the child's stdin pipe
  breaks — that is, when the child is killed or a holding descendant exits.
  This residual is stated in the rustdoc (`candidate.rs:197-205`,
  `:426-437`), in `crates/eggup-core/docs/verification.md`, and in
  `architecture/core-transaction.md`. It is recorded as F4.
- **No crash-journaling claim** is made or implied. `run_bounded` creates no
  file and holds no persistent state.

## Compatibility and migration review

**For a consumer of the published `eggup-core 0.1.3`:**

- **Nothing changes for them.** The published crate on crates.io still gives
  every child a **null** stdin and still has no `stdin_data` builder and no
  `MAX_STDIN_BYTES` constant. The change is in the working tree only.
- **The change is additive.** `CommandSpec` gains one private field and one
  public builder method; `MAX_STDIN_BYTES` is a new public constant. No
  published item changes shape and no published item is removed.
- **No `Error` variant changed.** `Error` keeps exactly the seven published
  variants (plus the `#[cfg(test)]`-only `Error::Injected`), so an exhaustive
  downstream `match` on `Error` still compiles unchanged — the property M012
  protected and M015 re-proven. `FailureCategory` is unchanged.
- **No behavioral change for a caller that attaches no payload.** Null stdin,
  no writer thread, the same four outcomes.
- **No migration path is required.** A consumer adopts the new builder at its
  own pace, and the builder is optional.

**For a consumer of the working tree:** the new `operation` strings are a
promise. The `operation` string of `Error::Io` was already a public field of a
public enum variant, so a downstream adapter could already read it; what
changed is that the three strings `run_bounded` produces are now a **published
contract** with documented meaning. Changing any of them from now on is a
source-visible break for an adapter that branches on it and must be treated as
an API change.

**No republication was performed.** No version bump, no tag movement, no
`cargo publish`, and no publication milestone authorizes one here. Reaching
consumers additionally requires the separate republication cascade recorded in
F2 — and `eggup-eggpack`'s `eggup-core = "=0.1.3"` pin
(`crates/eggup-eggpack/Cargo.toml`) blocks propagation to that adapter until a
separate adapter republication re-pins the new core version.

**`eggup-service` is unaffected.** `crates/eggup-service/**` was not modified;
`cargo check -p eggup-service --all-targets --locked --offline` exits 0. The
`SystemExecutor` consolidation onto core's primitive is **service milestone
M015**, planned in parallel, and is not part of this milestone.

## Security review

- The inherited environment is still cleared (`.env_clear()`), so no ambient
  environment variable can influence a child.
- Without an attached payload the child's stdin is still the null device; core
  cannot be made to feed a child by accident.
- The stdin payload is **bounded** (1 MiB, inclusive) and an oversized payload
  is rejected **before any child is spawned**, so an unbounded allocation into
  a pipe and an unbounded process are both prevented.
- Captured output remains bounded per stream; the stdin bound does not widen
  the output bound.
- The payload is written verbatim. Core does not interpret, transform, or
  validate its content, and claims nothing about it — the capability is a
  transport of bytes to a child the caller already chose to run.
- No new dependency, no `unsafe`, no FFI, no `build.rs`, no process-group or
  job-object mechanism, and no elevated privilege.
- The core rules are unchanged: SHA-256 remains integrity evidence only, no
  authenticity or signature claim is introduced, and no invented timeout is
  introduced — the only timeout remains the caller-supplied
  `CommandSpec::timeout`.

## Documentation / operations evidence

| Document | What changed |
| --- | --- |
| `crates/eggup-core/src/candidate.rs` | The `MAX_STDIN_BYTES` rustdoc (the bound, its inclusive edge, and the pre-spawn rejection); the `CommandSpec::stdin_data` rustdoc (the infallible builder, the execution-time bound, and null stdin without a payload); the `run_bounded` rustdoc (the stdin capability, the writer thread, the deadline bound on the writer, the abandoned-writer rule, the new `# Errors` cases, and the new `# Error classification` section naming the three stable `operation` strings and the expected downstream handling of each). |
| `crates/eggup-core/src/lib.rs` | The crate-root re-export of `MAX_STDIN_BYTES`. |
| `crates/eggup-core/docs/verification.md` | The candidate-command paragraph now states the bounded stdin capability, the bound, the pre-spawn rejection, the writer thread, the deadline bound on the writer, and the stable operation strings, and still points at the `run_bounded` rustdoc as the normative statement rather than restating a rule. |
| `architecture/core-transaction.md` | § "Candidate execution and bounded validators" gained the stdin paragraph and the stable-operation-string paragraph; the export table gained the `MAX_STDIN_BYTES` row; the failure-semantics table gained the two new stdin rows and now names the two `run_bounded` `Error::Io` operation strings. |
| `crates/eggup-core/README.md` | A closing paragraph recording the bounded runner's stdin capability, the bound, the typed write failure, and the three stable operation strings. |
| `crates/eggup-core/CHANGELOG.md` | An `## Unreleased` entry for M017 covering the capability, the bound, the typed write failure, the stable operation strings, the unchanged public error surface, the unchanged dependency surface, and the unpublished status with no migration required. No published version section was touched. |
| `plans/implementation/verified-update-core/017-bounded-stdin-execution-corrective.md` | **New.** The 16-section plan. Its `Status:` header now reads `implemented`. |
| `plans/closure/verified-update-core/017-status.md` | **New.** This record. |

**Truthfulness note.** While updating the candidate-execution section of
`architecture/core-transaction.md`, the stale `candidate.rs:` line references
in that section and in the "Candidate execution" export table were refreshed to
the post-change line numbers. Several of them were **already stale before this
milestone** (`CommandOutput` was cited as `candidate.rs:84` and `run_bounded`
as `candidate.rs:125`, against actual pre-change lines 98 and 192). The
refresh is a mechanical truthfulness fix in a file this milestone owns; it
introduces no new rule. The remaining stale `candidate.rs:` references in that
file — the ones about `ValidatedTransaction`, `commit`, `commit_with_post_commit`,
and the post-commit check — are **pre-existing** and were deliberately left
alone; they are recorded as F5.

**Contracts deliberately left unchanged.** `crates/eggup-core/docs/domain.md`
and `crates/eggup-core/docs/transaction.md` were reviewed and **not** changed.
The domain contract (members, integrity status, the `Ownership` model) and the
transaction contract (staging, the mutation lock, commit, rollback, receipts)
are untouched by this milestone: the bounded stdin capability is a property of
the candidate **execution** primitive, not of the domain model or of the
transaction state machine, and the plan's §4 invariants for both hold
unchanged. The candidate-execution contract lives in
`crates/eggup-core/docs/verification.md`, which was updated.

## Platform results

Reported separately. **No platform result is inferred from another.**

| Platform | Result | Evidence |
| --- | --- | --- |
| **macOS** (26.6.2, build 25G83, arm64) | **PASS** | Native: the full §"Exact commands run and results" sweep — fmt, clippy with `-D warnings`, the 103-test crate suite, the filtered `candidate` run, the 10-test fixture file (11 consecutive green runs), rustdoc, `git diff --check`, `check-docs-truth.sh`, and the `eggup-service` compile check. Toolchain `rustc 1.89.0` — the workspace MSRV. |
| **Linux** | **NOT RUN** | No hosted runner is reachable from this machine. |
| **Windows** | **NOT RUN** | No hosted runner is reachable from this machine. |

The fixture file is written to be cross-platform: it re-executes **this very
test binary** as its helper and selects the helper role from the helper's own
**argv** (not the environment, because `run_bounded` clears the inherited
environment). No shell, no `cmd.exe`, and no external tool is used, so
`sh -c` is not required on Windows. The fixture file is **not** `#[cfg]`-gated,
so the `windows-check` lane's workspace step executes it natively, exactly as
it executes `candidate_deadline.rs`.

Two Windows-specific behaviors are worth stating for the hosted run, because
they are the platform-sensitive parts of the new fixtures:

1. The write-failure fixture relies on the OS pipe buffer being smaller than
   `MAX_STDIN_BYTES` (1 MiB). The Windows anonymous pipe default is 4 KiB and
   the Linux and macOS defaults are 64 KiB and 16 KiB, so the 1 MiB payload
   cannot be silently accepted by the kernel.
2. The write-failure fixture asserts the `operation` string and that the
   underlying `io::Error` is preserved and is not `ErrorKind::Other`. It
   deliberately does **not** assert a specific `io::ErrorKind`, because
   Windows reports a broken pipe differently from Unix.

## Unresolved findings with severity

**F1 — Hosted qualification and the Linux / Windows platforms are not run.**
Severity: **medium**. The milestone's native evidence is macOS only. The
plan's §11.1 requires a fresh hosted run of `.github/workflows/ci.yml` covering
at least the `stable`, `msrv`, `macos`, and `windows-check` lanes, and
confirmation that the `windows-check` lane's workspace step executed
`crates/eggup-core/tests/candidate_stdin.rs` natively. **This finding's
condition is discharged only by that hosted run**, and the parent session owns
it. It is **not** a defect of the change.

**F2 — The capability is unpublished.** Severity: **medium**. The published
`eggup-core 0.1.3` still gives every child a null stdin and still lacks
`stdin_data` and `MAX_STDIN_BYTES`. Until a core republication lands, no
published consumer has the capability, and `eggup-eggpack`'s `=0.1.3` pin
blocks propagation to that adapter until a separate adapter republication
re-pins the new core version. **Follow-on: a core republication milestone
publishing the new capability, plus the matching `eggup-eggpack` /
`eggup-archive` / `eggup-acquisition` republication cascade.** This is the
same publication cascade every prior core change has named, and it is
**carried forward**, not created, by this milestone. It is not a defect of the
change.

**F3 — The duplicated runner in `eggup-service` still exists.**
Severity: **medium**. `SystemExecutor`
(`crates/eggup-service/src/lib.rs:1016-1118`) still carries its own process
runner and its own bounded stdin write, so core's runner and the service's
runner have two independent implementations of the same mechanism and two
independent sets of failure semantics. **Follow-on: service milestone M015 —
"bounded executor consolidation" — which is planned in parallel and is named in
the plan's §16.** M017 deliberately did not touch `eggup-service`; that
milestone is not this milestone's to implement.

**F4 — An abandoned stdin writer is a residual, not a defect.** Severity:
**low**. When the deadline is reached while the writer is blocked in `write`,
the writer thread is abandoned rather than reclaimed, exactly as a reader
blocked on a pipe a surviving descendant holds. Core owns no process group and
denies `unsafe`, so it cannot terminate a descendant that inherited the child's
stdin pipe and keep the pipe from breaking. The residual is bounded in the same
way the reader residual is — the writer ends when the pipe breaks — and it is
documented in the rustdoc and in the architecture doc rather than left implicit.
No corrective is proposed; reclaiming it would require a process-group or
job-object dependency, which the plan's §14 lists as a stop condition.

**F5 — Pre-existing stale `candidate.rs:` line references in
`architecture/core-transaction.md`.** Severity: **low**. The references about
`ValidatedTransaction`, `commit`, `commit_with_post_commit`, the post-commit
check, and the fault-injection seams are stale, and were stale **before** this
milestone. The candidate-execution references were refreshed by this milestone;
the remainder were deliberately not, because a whole-file reference refresh is
a docs-hygiene task rather than part of a bounded-stdin milestone. **Follow-on:
a documentation-truthfulness pass over the remaining references.** No consumer
decision depends on a line reference.

**No medium-or-higher finding about this milestone's own change remains open.**
F1, F2 and F3 are open at medium severity and are named as follow-ons: F1 is
discharged by the parent's hosted qualification, and F2 and F3 are named
correctives (a republication milestone, and service M015 respectively).

## Roadmap disposition

- `plans/subsystems/verified-update-core-roadmap.md` § 7 (the milestone status
  table) gains a row for **M017 — bounded stdin execution corrective**,
  recorded as **closed with unresolved findings** (native macOS evidence only;
  hosted qualification outstanding, see F1).
- The roadmap's summary line for the subsystem moves from "M001-M015 closed /
  hosted-green" to "M001-M015 closed / hosted-green; M017 closed (native macOS
  evidence), hosted qualification outstanding".
- `plans/registry.md` § "Post-0.1.3 open findings" gains no new entry for a
  defect in core. The publication gap (F2) is the standing publication cascade
  every prior core milestone already names, and the duplication in
  `eggup-service` (F3) belongs to the service subsystem's roadmap, where
  service M015 already records it.
- `plans/registry.md` § "Active subsystem roadmaps" records Verified update
  core as "M001-M015 closed/hosted-green, M017 closed (hosted qualification
  outstanding)".
- The root `CHANGELOG.md` `Unreleased` aggregate entry for this capability is
  owed by the parent session (the root changelog is outside this milestone's
  ownership).

## Registry updates

The parent session should apply the following, verbatim.

**`plans/registry.md` § "Active subsystem roadmaps":** change the Verified
update core entry from "M001-M015 closed/hosted-green" to "M001-M015 closed /
hosted-green; M017 closed (native macOS evidence), hosted qualification
outstanding".

**`plans/subsystems/verified-update-core-roadmap.md` § 7 status table:** add the
row

```text
| M017 | bounded stdin execution corrective | closed (native macOS evidence); hosted qualification outstanding | plans/implementation/verified-update-core/017-bounded-stdin-execution-corrective.md | plans/closure/verified-update-core/017-status.md |
```

**`plans/subsystems/verified-update-core-roadmap.md` summary:** update the
subsystem status line to "M001-M015 closed / hosted-green; M017 closed (native
macOS evidence), hosted qualification outstanding".

**`plans/registry.md` § "Post-0.1.3 open findings":** add no new entry for a
core defect. If the parent wants the publication gap tracked explicitly, the
wording would be: "`eggup-core`'s bounded candidate runner still gives every
child a null stdin in the published `0.1.3`; the bounded stdin capability
(Verified Update Core M017) is unpublished and reaches consumers only through a
core republication and the matching adapter republications. Severity: medium."
This is the standing publication cascade, not a new defect.

**Root `CHANGELOG.md` `Unreleased`:** add the aggregate entry for the M017
capability, matching `crates/eggup-core/CHANGELOG.md` — core's bounded runner
can now deliver a bounded stdin payload, an oversized payload is rejected
before any child is spawned, a failed write fails closed with a typed
`Error::Io`, the fix is unpublished, the published `eggup-core 0.1.3` still
gives every child a null stdin, and no publication milestone authorizes it.

**Not applied by this milestone:** the root `CHANGELOG.md` edit, the
`plans/registry.md` edit, and the roadmap edit — all three are outside this
milestone's ownership and are described here as prose for the parent to apply.

## Parent-session verification addendum (2026-10-24, hosted qualification)

**Hosted CI run `40062299630`** (commit `b9885fa`) — **all five lanes green**.

| Lane | Result | Evidence |
|---|---|---|
| `stable` (Linux) | **success** | `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo test --workspace --all-targets --all-features --locked`, `cargo doc --workspace --no-deps --locked`, and the packaged-service step: **518 passed, 0 failed, 48 test binaries** |
| `msrv` (Linux, 1.89.0) | **success** | `cargo check --workspace --all-targets --locked` at the declared floor and the packaged-service step. This lane runs no tests |
| `macos` | **success** | full `cargo test` plus the packaged-service step: **518 passed, 0 failed, 48 test binaries** |
| `windows-check` | **success** | nine targeted `cargo test` steps plus the full workspace run: **637 passed, 0 failed, 68 test binaries** |
| `service-systemd` (Linux) | **success** | the real-systemd fixtures: **9 passed, 0 failed, 7 test binaries** |

The Windows lane's 637 against the other lanes' 518 is **not** a coverage
difference. That lane runs nine separate `cargo test` steps, so its aggregate
double-counts the binaries the other lanes run once, and it additionally runs
`--all-features` for `eggup-eggpack`. The per-step counts are the evidence; the
aggregate is a sum over steps, not a distinct-test count.

### What this supersedes

The header of this record states the milestone closed on **native macOS
evidence only**, with hosted CI qualification and the Linux and Windows
platforms **not run**. §12 lists "Every hosted CI lane, and the Linux and
Windows platforms" as not run, and **F1** is the finding that tracks exactly
that. **All of those are superseded.** This milestone is now qualified on all
three platforms.

The new fixture `crates/eggup-core/tests/candidate_stdin.rs` ran **natively**
in all three lanes — `stable` (Linux), `macos`, and `windows-check`. Its
cases are therefore real executions on Linux and Windows, not compile-only
observations:

- the bounded stdin payload is delivered to the child and echoed back;
- a payload over `MAX_STDIN_BYTES` is rejected with `Error::InvalidInput`
  **and no child is spawned**;
- a child that exits without reading its stdin yields the typed `Err` with
  operation `"writing bounded stdin"`;
- with no payload attached, stdin is still null;
- the deadline still bounds a child that never reads a large payload.

### Compatibility consequence, carried forward

Unchanged by the hosted qualification: `eggup-core` is published at **`0.1.3`**
and the published crate still nulls stdin and lacks `stdin_data` and
`MAX_STDIN_BYTES`. The `v0.1.3` tag is unmoved and no republication was
performed. `Error` keeps exactly its seven published variants on both sides of
this change. The `eggup-eggpack` `=0.1.3` pin still blocks propagation of the
core change into the eggpack adapter, exactly as this record's §9 states.
