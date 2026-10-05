# Verified Update Core Milestone 011 — proof-authorized stale-lock recovery

Status: ready

Repository baseline: `db5b3121f17f92029a47c389e68a464bc4478b27`

Source roadmap: `plans/subsystems/verified-update-core-roadmap.md`

Primary class: invariant / capability corrective

Hard dependencies:

- Verified Update Core M009 is closed.
- Existing `MutationLock` create-new/owner-token behavior is qualified and remains the fail-closed default.

Reference-consumer evidence:

- `eggstack/eggpool@fe3c308df1450cd3216f50abc160a0a9f426f8d9` recovers an update lock after bounded owner evidence indicates the original updater is gone; replacing it with current Eggup semantics would regress unattended recovery.
- Gregg does not require EggPool's exact stale policy, demonstrating why Eggup must expose proof authorization rather than hard-code one PID/age heuristic.

## 1. Objective

Implement the long-term specification's missing stale-lock capability: stale-lock recovery only when staleness can be proven, with fail-closed behavior when ownership is ambiguous.

Current `MutationLock` never removes an existing lock. That conservative default remains valid, but it is not the documented end state and would regress EggPool's standalone updater if adopted directly.

M011 adds an explicit caller-authorized recovery seam. Core owns safe lock mutation; the consumer supplies deployment-specific evidence that the exact observed owner is stale. Ordinary `MutationLock::acquire` remains fail-closed and performs no automatic recovery.

## 2. Readiness and dependencies

This work is dependency-ready and independent of M010. The canonical specification already assigns stale-lock recovery to Core and rejects PID existence alone as universal proof.

If M010 lands first and touches lock plumbing incidentally, rebase the execution baseline while preserving this contract.

## 3. Current implementation evidence

Current `MutationLock` creates `.eggup-mutation.lock` using create-new, records bounded pid/nonce/product/release text, returns UpdateInProgress on any existing record, exposes read-only `inspect`, and removes its own record on Drop only when bytes still equal its token.

A process crash can therefore leave an installation permanently blocked until manual cleanup.

EggPool currently stores bounded pid/executable/time evidence and recovers when its owner is demonstrably gone. Those exact heuristics remain consumer policy.

A naive `inspect -> stale -> remove_file` is insufficient because another process can replace the lock pathname between proof and deletion. Recovery therefore needs compare/claim discipline and must never delete an unobserved lock by pathname alone.

## 4. Invariants that must not regress

- One mutating transaction owns an installation domain.
- Ordinary `MutationLock::acquire` remains fail-closed/backward-compatible.
- Malformed, oversized, symlink, non-regular, or unreadable records are never automatically removed.
- Core never decides staleness from PID liveness, age, executable name, or service state by itself.
- Caller authorization applies only to the exact bounded observation inspected.
- A changed/replaced lock invalidates authorization.
- Core never removes a lock merely because its pathname matches.
- No secret-bearing consumer data is required.
- Recovery failure cannot silently become successful acquisition.
- Diagnostics remain bounded.
- No unsafe/process-enumeration dependency enters Core.

## 5. Scope and non-scope

### In scope

- typed bounded lock observation/record parsing;
- explicit caller-supplied recovery decision;
- safe claim/removal of the exact authorized stale record;
- bounded acquisition retry after successful stale claim;
- race/fault injection around observation, claim, replacement, reacquisition and cleanup;
- backwards-compatible fail-closed acquire;
- documentation and consumer examples.

### Explicitly out of scope

- Core process enumeration or hard-coded timeout/age policy;
- dead PID as universal proof;
- removing malformed locks;
- service-manager inspection;
- persistent installation database;
- distributed locks;
- takeover of live/foreign transactions;
- EggPool migration itself.

## 6. Required production changes

### 6.1 Expose a bounded typed observation

Introduce a read-only observation containing only facts Core can prove about the lock record: path, exact bounded record identity, parsed pid/product/release/nonce where available, optional creation timestamp if the format is extended, and file metadata facts needed for safe recovery.

Prefer accessors over public representation fields. Existing `LockStatus` may expose this observation.

If the record format changes, keep it bounded and version or parse old records conservatively. Old/malformed records remain non-recoverable unless the required evidence is present.

### 6.2 Add caller-supplied stale proof

Add a policy-neutral interface conceptually equivalent to:

```rust
enum StaleLockDecision {
    Active,
    ProvenStale,
    Unknown,
}

trait StaleLockVerifier {
    fn classify(&self, observed: &LockObservation) -> StaleLockDecision;
}
```

Exact naming is flexible. Only `ProvenStale` authorizes recovery. `Active` and `Unknown` retain the record and return contention.

The verifier cannot directly delete the lock path.

### 6.3 Add explicit recover-capable acquisition

Keep `MutationLock::acquire(...)` as no-recovery default. Add a visibly separate `acquire_with_recovery`/builder policy.

Sequence:

1. create-new;
2. on existing record, inspect within bounds;
3. get caller decision;
4. only on ProvenStale, safely claim exact record;
5. retry create-new once or within a small bounded loop;
6. any observation change returns contention/unknown.

### 6.4 Make stale claiming path-race safe

Do not use check-then-`remove_file` as the only guard.

Preferred design:

```text
observed lock
 -> re-read exact bytes + safe file kind
 -> rename pathname into unique Eggup-owned same-directory claim/quarantine
 -> verify claimed object still equals authorized observation
 -> create-new real lock
 -> delete claimed stale record only after ownership is established
```

If claimed bytes/identity differ, do not treat recovery as success. Restore when safe; otherwise retain evidence and fail.

If another writer creates a new lock after stale claim but before this caller acquires, the new writer wins and its lock is never removed.

An alternative compare-and-swap design is acceptable only with equivalent Linux/macOS/Windows guarantees without unsafe code.

### 6.5 Keep consumer evidence outside Core policy

Core may add a bounded creation timestamp for diagnostics, but does not encode EggPool executable/config/service policy as mandatory fields.

Consumers may derive stronger identity from observed pid/product/root and their own runtime facts.

### 6.6 Preserve recovery evidence

If stale claiming partially succeeds and Core cannot restore/clean the claimed object safely, return a typed high-severity error with the actual retained path. Never silently discard a displaced lock record.

## 7. Ordered work packages

1. Add regression proving a left-behind current lock permanently blocks ordinary acquire.
2. Introduce typed observation/parsing while retaining old inspect behavior.
3. Add caller proof interface and recover-capable acquire.
4. Implement atomic stale claim/quarantine with exact-observation revalidation.
5. Add race/fault injection for replacement before claim, after claim, and before reacquire.
6. Add a deterministic example verifier demonstrating external evidence without Core PID policy.
7. Run native Windows/macOS/Linux filesystem qualification.
8. Review API for consumer-specific leakage.
9. Update docs/roadmap/registry and write closure.

## 8. Failure, restart, cancellation, and contention semantics

Normal contention is unchanged. Verifier Active/Unknown performs no mutation.

If authorization says stale but the record changes before claim, perform no stale deletion and return contention.

If a stale record is claimed but a new lock appears before acquisition, preserve the new writer's lock. Clean the claimed stale artifact if safe; otherwise report its path.

No retry loop is unbounded. A crash after moving a stale record to a claim path is not automatically treated as permission to delete that claim later.

## 9. Compatibility and migration

Existing callers using `MutationLock::acquire` receive identical semantics. Recovery is opt-in.

EggPool can later map its process/executable evidence into the verifier without moving provenance/package-manager policy into Eggup.

Lock records remain ephemeral transaction state, not a persistent user database.

## 10. Required tests

At minimum:

- ordinary acquire still refuses existing lock;
- malformed/oversized/symlink/non-regular lock never reaches destructive recovery;
- Active and Unknown retain record;
- ProvenStale exact record is claimed and acquisition succeeds;
- record replaced after observation is never deleted;
- replacement immediately before claim fails closed;
- second writer after stale claim is preserved;
- claim cleanup failure returns real retained path/evidence;
- current owner Drop never deletes replacement lock;
- old-format parsing is conservative;
- bounded non-ASCII diagnostics remain panic-free;
- Linux/macOS/Windows filesystem semantics execute natively.

## 11. Required verification commands

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p eggup-core --all-targets --all-features --locked
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo +1.89.0 test --workspace --all-targets --locked
cargo tree -p eggup-core --locked
cargo package -p eggup-core --locked
cargo publish -p eggup-core --dry-run --locked
git diff --check
```

Hosted closure requires Stable/MSRV/macOS/Windows green; Windows executes claim/race fixtures rather than compile-only evidence.

## 12. Documentation updates

Update `architecture/core-transaction.md`, core README/rustdoc, root/core Unreleased changelogs, source roadmap, consumer-adoption dependency status, registry, and new `plans/closure/verified-update-core/011-status.md`.

Document clearly that default acquisition never recovers stale locks automatically.

## 13. Acceptance criteria

M011 closes only when Core exposes typed bounded observations; existing acquire remains fail-closed; recovery is explicit/caller-authorized; Core contains no consumer stale heuristic; only an exact observed safe lock may be claimed; pathname replacement cannot cause deletion of an unobserved lock; concurrent writers are preserved; partial recovery retains real evidence; native platforms pass; no new unsafe/process-enumeration dependency enters Core; and EggPool's stronger stale behavior is representable without its provenance policy.

## 14. Stop conditions

Stop and write a lock-architecture ADR/corrective if safe claiming requires a persistent cross-run journal, a platform cannot provide the required race guarantee without unacceptable native/unsafe code, Core would have to inspect process executable paths/service managers, old records cannot be parsed fail-closed, or authorization cannot be bound to one exact observation.

Do not substitute "PID is gone" as Core's universal stale proof.

## 15. Closure evidence required

Record implementation SHA(s), public API/record-format changes, race/fault matrix, native platform results, Active/Unknown/ProvenStale examples, retained-evidence behavior, package/dependency impact, hosted run id, unresolved findings, and EggPool M007 unblock audit.

## 16. Handoff notes

The important distinction is authorization versus mechanism.

A consumer may know enough to prove an updater died. Eggup should let that consumer authorize recovery, while Eggup ensures that the exact proven-stale filesystem object — and no later replacement — is the object displaced.
