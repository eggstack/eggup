# Verified Update Core Milestone 007 — Deferred Finalization and Post-Commit Failure Policy

Status: ready for handoff

Repository baseline: `66acd739f792437cb9fa1701b8fa456c4ba4c403`

Source roadmap:

- `plans/subsystems/verified-update-core-roadmap.md`

Long-term requirements:

- `plans/000-long-term-specification.md#5-transaction-model`
- `plans/000-long-term-specification.md#12-rollback-model`
- `plans/000-long-term-specification.md#13-service-lifecycle`

Applicable ADRs:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`
- `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md`
- `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md`

Primary class: invariant / capability

## 1. Objective

Implement the still-missing ADR-0002 post-commit boundary in `eggup-core`: after a complete verified artifact set has become live, retain the mutation lock and rollback evidence long enough for one caller-supplied post-commit check to succeed or fail, then resolve failure using explicit `KeepInstalled` or `RollBack` policy.

This milestone exists because the current `ValidatedTransaction::commit()` finalizes/removes backup state before any service restart, health check, or other post-install verification can run. Service Lifecycle M005 therefore cannot truthfully offer rollback-on-restart/health-failure until the core exposes this boundary.

The implementation remains policy-neutral. The core must not learn service managers, health endpoints, networks, release selection, or product semantics.

## 2. Readiness and dependencies

Hard dependencies are closed:

- verified-update-core M005 safety/API corrective;
- verified-update-core M006 package qualification.

The accepted ADR already defines the required policy:

```rust
enum PostCommitFailurePolicy {
    KeepInstalled,
    RollBack,
}
```

Current source evidence at the baseline:

- `ValidatedTransaction::commit(CommitOwnership)` owns the complete verified commit path;
- `commit_inner` retains a backup set while committing members, but successful commit finalization removes that set before returning;
- `TransactionDisposition` already distinguishes `Committed`, `RolledBack`, and `RecoveryRequired`;
- `FailureReport` preserves phase/category/member/detail, but `FailurePhase` has no post-commit/post-install phase;
- the `CleanupDisposition` rustdoc explicitly states that ADR-0002's `KeepInstalled | RollBack` policy is reserved for a future boundary and is not implemented.

No architecture decision is required: this plan implements an accepted ADR rather than changing it.

## 3. Current evidence

The existing core correctly provides:

- validated preparation and private staging;
- all-member integrity verification;
- bounded candidate validation;
- mutation locking;
- locked ownership and staged-digest revalidation;
- coherent multi-member backup/commit;
- rollback on commit-phase failure;
- recovery-required evidence if rollback fails;
- immediate successful finalization through `ValidatedTransaction::commit()`.

The missing state transition is:

```text
new coherent artifact set is live
        |
        v
caller post-install check
        |
        +--> success ------------> finalize backup -> Committed
        |
        +--> failure + KeepInstalled -> finalize backup -> Committed + post-commit failure evidence
        |
        `--> failure + RollBack ----> restore old set -> RolledBack
                                          |
                                          `--> rollback failure -> RecoveryRequired
```

The mutation lock must span that entire decision.

## 4. Invariants

- The post-commit check runs only after every member of the new artifact set is live and coherent.
- The mutation lock remains held from pre-commit ownership revalidation through post-commit policy resolution and backup finalization/rollback.
- Backup evidence is not removed before a post-commit `RollBack` decision can be honored.
- `KeepInstalled` never reports that the post-commit check succeeded; failure evidence remains machine-readable.
- `RollBack` never reports success unless the old generation is restored and verified.
- Rollback failure yields `RecoveryRequired` with both the triggering post-commit failure and rollback failure preserved.
- Existing pre-commit and commit failure semantics remain unchanged.
- Existing `ValidatedTransaction::commit()` remains an immediate-finalize convenience path with source-compatible behavior unless a narrowly justified pre-1.0 correction is required.
- No service/network/release-policy dependency enters `eggup-core`.
- Post-commit diagnostics are bounded and must not retain arbitrary unbounded caller error payloads.
- No hidden timeout is invented by core. The caller owns the bounded operation it supplies.
- Crash durability beyond the documented process-level backup/lock evidence remains unclaimed.

## 5. Scope

### In scope

- public `PostCommitFailurePolicy::{KeepInstalled, RollBack}` or an API-equivalent type matching ADR-0002 semantics;
- a core-controlled post-commit execution seam on a validated transaction;
- retaining lock and backup state through the post-commit check;
- explicit post-commit failure phase/category/reporting;
- rollback of an otherwise successfully installed new generation;
- `KeepInstalled` result semantics that preserve post-commit failure evidence;
- rollback-failure recovery evidence;
- deterministic fault-injection coverage;
- compatibility/rustdoc/examples/documentation updates;
- stable/MSRV/package qualification of the changed public surface.

### Out of scope

- service-manager calls or service-specific types;
- HTTP, release discovery, manifest parsing, or artifact acquisition;
- application health semantics;
- retry/fallback policy;
- persistent crash journal;
- background update scheduling;
- authenticity/signing;
- Windows running-image replacement qualification;
- changing producer/consumer ownership established by ADR-0004.

## 6. Required production changes

### A. Public post-commit policy vocabulary

Add the ADR-0002 policy as a public, documented core type.

The public API must distinguish:

- post-commit check succeeded;
- post-commit check failed but caller chose `KeepInstalled`;
- post-commit check failed and rollback succeeded;
- post-commit check failed and rollback also failed.

Do not overload `CleanupDisposition` to encode this policy.

### B. Core-controlled deferred finalization

Add a transaction path that commits the entire new artifact set but defers backup finalization until one caller-supplied post-commit operation returns.

The preferred design is a non-escaping core-controlled operation such as `commit_with_post_commit` or equivalent. A public half-committed transaction handle is acceptable only if its cancellation/drop semantics are demonstrably fail-safe and preserve recovery evidence; do not expose one merely for ergonomics.

The caller-supplied operation must receive only what it needs to run a check. It must not gain authority to mutate transaction internals or bypass ownership/integrity invariants.

### C. Failure representation

Extend transaction failure vocabulary with a distinct post-commit/post-install phase.

Caller failure input must be converted to bounded core-owned evidence. Do not retain arbitrary trait objects, backtraces, command output, secrets, or unbounded strings.

For `KeepInstalled`:

- the complete new set remains live;
- the backup is finalized/removed using ordinary cleanup rules;
- terminal disposition remains explicitly successful-as-installed, preferably `Committed`;
- the receipt retains a post-commit failure report so callers cannot mistake the check as passing;
- cleanup failure still uses the existing retained-recovery evidence contract.

For `RollBack`:

- restore all old members using the same proven rollback machinery;
- verify restoration;
- return `RolledBack` with the post-commit failure preserved as the triggering failure;
- if restoration fails, return `RecoveryRequired` with both failure reports and real retained backup/lock evidence.

### D. Backward-compatible immediate commit

Keep `ValidatedTransaction::commit()` as the simple path for callers that have no post-install check.

Its successful semantics remain: commit coherent bytes, finalize transaction-owned backup state, return `Committed`.

Avoid forcing all existing consumers to supply a no-op callback.

### E. Interruption and panic behavior

Document what happens if caller code aborts/panics while the post-commit operation is in progress.

The implementation must not deliberately delete the only rollback evidence before policy resolution. If a callback-based design relies on unwinding/drop, tests must prove owned backup/lock evidence is retained or otherwise leaves a truthful recoverable state. If that cannot be made reliable, choose a different API shape rather than weakening the invariant.

The milestone does not claim recovery from process kill/power loss without a persistent journal.

## 7. Ordered work packages

1. Add the post-commit policy and bounded failure vocabulary without changing the existing commit path.
2. Refactor successful commit sequencing so backup finalization can be delayed internally without duplicating backup/rollback logic.
3. Add the core-controlled post-commit operation.
4. Implement `KeepInstalled`, `RollBack`, and rollback-failure terminal semantics.
5. Add post-commit fault injection needed to exercise restoration and finalize failures.
6. Add single- and multi-member tests, lock-span tests, and interruption/drop tests appropriate to the selected API shape.
7. Update rustdoc, transaction documentation, examples, README/changelog, and ADR cross-references.
8. Run stable/MSRV/package/docs/dependency/hosted CI qualification.
9. Write closure and update the core/service roadmaps and registry; only then mark Service Lifecycle M005 ready for plan authoring.

## 8. Failure, restart, and contention semantics

- failure before any live mutation: unchanged existing behavior;
- commit-phase failure: unchanged existing rollback/recovery behavior;
- post-commit check success: finalize -> `Committed`;
- post-commit failure + `KeepInstalled`: keep coherent new set, finalize old backup, return `Committed` plus explicit post-commit failure evidence;
- post-commit failure + `RollBack`: restore old generation -> `RolledBack`;
- rollback failure: `RecoveryRequired`, retain real evidence and both causes;
- backup-finalize failure after keeping installed: `Committed` with `RetainedForRecovery` and finalize failure evidence, preserving any post-commit failure evidence without ambiguity;
- a second transaction attempting the same installation domain while the post-commit operation runs must observe lock contention;
- caller cancellation/abort semantics must be explicitly documented and must not be presented as a successful commit.

## 9. Compatibility and migration

This is additive to the qualified core API.

Existing consumers using `ValidatedTransaction::commit()` need no behavior change.

Service Lifecycle M005 is blocked on this closure and must compose with the resulting core API rather than reimplement backup retention or rollback in `eggup-service`.

Future EggPool or other consumers may use the same policy boundary without importing service semantics into core.

## 10. Required tests

At minimum:

- existing immediate commit success remains unchanged;
- post-commit success for one member;
- post-commit success for multi-member set;
- callback/check observes the complete new generation, never a partial set;
- mutation lock remains held during the post-commit operation;
- post-commit failure + `KeepInstalled` keeps every new member and records failure;
- post-commit failure + `RollBack` restores every old member;
- rollback removes newly created members that were absent before commit;
- rollback failure -> `RecoveryRequired` with original + rollback reports;
- finalize cleanup failure after `KeepInstalled` retains a real evidence path;
- ownership/stage revalidation still occurs before backup;
- existing commit/rollback fault matrix remains green;
- bounded post-commit failure detail;
- selected API's panic/drop/interruption behavior;
- no service/network dependency appears in `eggup-core`.

## 11. Required verification commands

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p eggup-core --all-targets --all-features --locked
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo tree -p eggup-core --locked
cargo package -p eggup-core --locked --allow-dirty
cargo +1.89.0 check --workspace --all-targets --locked
cargo +1.89.0 test -p eggup-core --all-targets --locked
./scripts/check-local.sh
git diff --check
```

Record hosted Linux, macOS, Windows-check, and Rust 1.89 CI separately. Do not infer Windows live-replacement support from compile-only evidence.

## 12. Documentation updates

- `crates/eggup-core/README.md`;
- public rustdoc;
- transaction documentation;
- transaction/receipt example(s);
- root README/architecture overview if capability summaries mention finalization;
- changelog;
- verified-update-core roadmap;
- service-lifecycle roadmap;
- registry;
- closure record.

ADR-0002 remains accepted and should not be rewritten merely because its deferred capability is now implemented.

## 13. Acceptance criteria

M007 closes only when a complete live artifact generation can remain rollback-capable through one caller-supplied post-commit operation; `KeepInstalled` and `RollBack` are explicit; lock and backup lifetime cover the decision; rollback failure preserves recovery evidence; immediate `commit()` remains usable; post-commit failure is distinguishable from success and from cleanup failure; all inputs/evidence are bounded; no service/network/product policy enters core; and stable/MSRV/package/docs/hosted CI qualification passes with no medium-or-higher finding.

## 14. Stop conditions

Stop and write a corrective/ADR review if:

- implementing the seam requires a service-manager dependency in core;
- a public pending handle cannot have safe cancellation/drop semantics;
- `KeepInstalled` cannot preserve truthful failure evidence without breaking receipt semantics;
- rollback after a coherent live commit cannot reuse the existing backup/restore invariants;
- the design requires a persistent crash journal to make the claimed semantics true;
- implementation would change release or health policy rather than transaction mechanism.

## 15. Closure evidence required

Record:

- exact implementation SHA(s);
- public API before/after;
- transaction state diagram;
- `KeepInstalled` / `RollBack` / rollback-failure matrix;
- single/multi-member and absent-member restoration evidence;
- lock-span evidence;
- panic/drop/interruption disposition for the selected API shape;
- dependency tree;
- stable/MSRV/package/docs results;
- hosted CI;
- unresolved findings;
- explicit statement whether Service Lifecycle M005 is now ready for plan authoring.

## 16. Handoff notes

This is the core prerequisite for Service Lifecycle M005.

It may execute in parallel with CodeGG Consumer Adoption M005 because CodeGG's current upgrade path needs verified multi-artifact replacement but does not require service restart/health rollback orchestration.

Do not expand this milestone into service lifecycle, release manifests, archive extraction, or producer tooling.
