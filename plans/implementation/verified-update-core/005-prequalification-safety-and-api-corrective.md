# Verified Update Core Milestone 005 — Pre-Qualification Safety and API Corrective

Status: ready for handoff

Repository baseline: `9f527beb20da585bc3cf56f45f1fd96fd418c124`

Source roadmap:

- `plans/subsystems/verified-update-core-roadmap.md`

Corrects post-closure findings from:

- `plans/closure/verified-update-core/002-status.md`
- `plans/closure/verified-update-core/003-status.md`
- `plans/closure/verified-update-core/004-status.md`

Long-term requirements:

- `plans/000-long-term-specification.md#5-transaction-model`
- `plans/000-long-term-specification.md#7-verification-model`
- `plans/000-long-term-specification.md#9-filesystem-safety`
- `plans/000-long-term-specification.md#10-ownership-model`
- `plans/000-long-term-specification.md#11-locking-and-contention`
- `plans/000-long-term-specification.md#12-rollback-model`

Applicable ADRs:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`
- `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md`
- `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md`

Primary class: invariant / corrective

## 1. Objective

Correct the public ownership, verification, transaction-result, filesystem, and recovery contracts discovered after M002-M004 closure before `eggup-core` is package-qualified or adopted by another repository.

This milestone is intentionally allowed to make breaking pre-1.0 API changes because Eggup has no consumers yet. The goal is to stabilize the safety semantics now rather than preserve accidental M002-M004 shapes.

## 2. Why this milestone is ready

M001-M004 are implemented and have closure records.

The current API is small enough to correct without consumer migration.

The post-closure review identified concrete discrepancies between code and the canonical specification:

1. existing-destination ownership is effectively represented as `Ownership::Managed` and is not independently proven before destructive mutation;
2. destination-parent creation occurs before containment revalidation and can create/follow path components before the final check;
3. staged bytes are integrity-checked before candidate execution but are not reverified immediately before commit;
4. `AuthenticityRequirement::Required` can be declared without any enforcement path;
5. the exported `PostCommitFailurePolicy` currently models cleanup disposition rather than the ADR-0002 `KeepInstalled | RollBack` policy;
6. rolled-back/recovery receipts discard the actual failure cause and phase;
7. cleanup failure reports a synthetic child path rather than the real retained recovery root;
8. stage/backup/lock permissions are not explicitly owner-private on Unix;
9. stale-lock recovery promised by planning is not implemented; the current behavior is conservatively fail-closed;
10. repository/crate/architecture documentation still describes pre-M003 behavior.

## 3. Current implementation evidence

At the baseline:

- `ArtifactMember` carries `Ownership::Managed` only;
- `revalidate_destination` checks containment, regular-file type, and Unix hard-link count;
- `ensure_destination_parent` invokes `create_dir_all` before revalidation;
- `VerifiedTransaction` records SHA-256 results, then candidate execution occurs, then the staged paths are committed;
- `AuthenticityRequirement` exposes `None | Required`, but no authenticity validator exists;
- `PostCommitFailurePolicy` exposes `Cleaned | RetainForRecovery`;
- `finish_failure` receives the triggering `Error` as `_error` and omits it from the receipt;
- stage and backup directory names use process/atomic counters;
- current CI is green on Linux and local closure evidence is macOS-focused.

## 4. Invariants that must not regress

- no unverified or changed-after-verification staged bytes may be committed;
- no candidate may execute before required integrity verification;
- destructive operations require `Owned` or explicitly authorized `Absent` state;
- `Foreign` and `Unknown` fail closed;
- destination containment is proven before filesystem creation/mutation outside already-owned transaction state;
- rollback/recovery evidence is never silently discarded;
- transaction results preserve failure phase and cause category;
- public type names match their semantics;
- no HTTP/TLS/service-manager dependency enters `eggup-core`;
- no implicit privilege escalation;
- no authenticity claim is made when no authenticity verifier exists.

## 5. Scope

### In scope

- replace the one-variant ownership placeholder with the canonical `Absent | Owned | Foreign | Unknown` classification;
- introduce a consumer-supplied destination ownership verification contract suitable for both single binaries and bundles;
- require ownership classification immediately before destructive mutation under the mutation lock;
- define exact rules for when an absent destination may be created;
- remove or safely defer automatic destination-parent creation;
- harden stage/backup/lock permissions and naming;
- revalidate staged file metadata and SHA-256 immediately before first live mutation;
- eliminate unenforced authenticity requirements from the commit-capable state machine;
- rename the current cleanup-disposition type;
- preserve structured failure phase/cause/member information in terminal transaction results;
- fix retained-recovery path reporting;
- reconcile stale-lock behavior with the public/planning contract;
- update stale docs;
- expand tests and CI where practical.

### Explicitly out of scope

- signatures/attestations;
- HTTP transport;
- service lifecycle;
- Cargo fallback;
- installer generation;
- persistent crash journal;
- automatic takeover of foreign destinations;
- automatic privilege escalation;
- broad public API convenience layer.

## 6. Required production changes

### A. Destination ownership contract

Replace the current `Ownership::Managed` placeholder with the canonical classification:

```rust
pub enum Ownership {
    Absent,
    Owned,
    Foreign,
    Unknown,
}
```

Do not infer `Owned` merely because a path appears in an `InstallPlan`.

Introduce a small consumer-supplied verification interface, for example a trait or callback receiving the member identity, destination path, and relevant metadata and returning `Ownership`.

The exact API may differ, but it MUST:

- be usable synchronously;
- avoid application-specific version types;
- permit deterministic unit testing;
- run again immediately before destructive mutation;
- treat verifier errors/ambiguity as `Unknown` or hard failure, never `Owned`;
- distinguish authorization to create an absent destination from authorization to replace an existing one.

Provide narrowly useful generic helpers only where evidence exists, such as "must be absent" or exact prior SHA-256 ownership. Do not invent a universal executable-identity rule.

### B. Destination-parent safety

Do not call `create_dir_all` on an unchecked destination path during commit.

For the initial core contract, prefer the narrow rule:

- all destination parent directories must already exist;
- each existing parent/ancestor used for commit must canonicalize beneath the authorized installation root;
- symlink/junction/reparse-point ambiguity fails closed;
- missing parent is an actionable error.

If safe recursive directory creation is later required by a real consumer, plan it as a separate owned-directory transaction feature rather than silently widening this corrective.

### C. Stage and backup safety

Harden transaction-owned state.

On Unix:

- stage and backup directories must be explicitly owner-only, normally mode `0700`;
- lock/state files should be owner-readable/writable only, normally `0600`;
- staged non-executable files should not inherit unexpectedly broad source permissions;
- executable intent should add only the required execute bits to an otherwise private/safe staged mode.

Use collision-resistant transaction names or a safe create-new retry mechanism. Predictable names must not permit precreation to redirect, replace, or cause deletion of attacker-controlled paths.

All cleanup must verify ownership/token identity before deletion.

### D. Verification-to-commit revalidation

The validated transaction must retain the digest expected for every commit-capable member.

Immediately after acquiring the mutation lock and before backing up any live destination:

- re-read staged metadata;
- require a regular non-link file;
- re-hash the staged member;
- compare against the digest that established `Verified`;
- fail before live mutation if any member changed.

The commit path must not accept `IntegrityStatus::NotRequired`.

Candidate execution may alter its own staged executable; such mutation must therefore fail final digest revalidation unless a future explicit validator contract authorizes a transformed output.

### E. Authenticity API correction

No public commit path may accept a declaration equivalent to "authenticity required" without satisfying it.

Because no authenticity verifier exists yet, prefer removing `AuthenticityRequirement::Required` and any artifact field that implies enforceable authenticity.

The long-term terminology/docs should continue to distinguish future authenticity from integrity, but runtime types must not promise an unenforced property.

If implementation retains an authenticity enum for forward compatibility, only an explicitly non-claiming state may be constructible in commit-capable paths until the trust ADR lands.

### F. Transaction result semantics

Rename the existing:

```text
PostCommitFailurePolicy::{Cleaned, RetainForRecovery}
```

to a name that accurately represents evidence cleanup, for example `RecoveryEvidenceDisposition` or `CleanupDisposition`.

Reserve `PostCommitFailurePolicy` for the ADR-0002 service/post-install policy `KeepInstalled | RollBack`, which should be implemented only when a post-commit failure boundary actually exists.

Add a structured failure report to non-success terminal outcomes. It should preserve at minimum:

- phase;
- optional member;
- stable category;
- bounded human-readable detail.

Suggested phase vocabulary:

- Lock;
- Ownership;
- StageRevalidation;
- Backup;
- Commit;
- Rollback;
- Finalize.

Do not require `std::io::Error` itself to be cloneable in a receipt; normalize it into a bounded structured report if needed.

A rolled-back transaction remains a first-class terminal result, but the caller must be able to answer "what failed?" without scraping logs.

### G. Recovery path correction

When committed state is retained because backup cleanup fails:

- `recovery_path` must refer to the actual retained backup root;
- the cleanup failure must be represented separately in the structured failure/warning field;
- never return a path that does not exist solely to encode an error message.

### H. Mutation-lock reconciliation

Preserve fail-closed behavior for ambiguous existing locks.

Implement either:

1. strong stale-lock recovery where process identity can be proven with platform-appropriate evidence; or
2. an explicit lock-inspection API and documented "no automatic stale removal yet" contract, while changing roadmap/closure-facing claims so no unimplemented recovery is implied.

Do not use PID nonexistence alone as proof across the public contract.

This corrective does not require portable automatic stale recovery if strong identity cannot be implemented cleanly. It requires truthful semantics.

### I. Documentation-state cleanup

Update:

- root `README.md`;
- `crates/eggup-core/README.md`;
- crate-level docs in `lib.rs`;
- `architecture/overview.md`;
- transaction/verification docs;
- changelog;
- roadmap/registry.

They must reflect that local verified transaction behavior exists while transport/services remain absent.

## 7. Ordered work packages

### Work package A — Public type correction

- fix Ownership;
- remove/neutralize unenforced authenticity requirement;
- rename cleanup disposition;
- add failure phase/report types.

Acceptance evidence: rustdoc and compile tests show no misleading public type names.

### Work package B — Filesystem ownership and containment hardening

- add destination ownership verifier;
- prohibit unchecked parent creation;
- harden link/reparse handling;
- enforce private stage/backup/lock permissions.

Acceptance evidence: negative tests for foreign/unknown, missing parent, symlinked parent, hard links, unsafe permissions.

### Work package C — Verification-to-commit continuity

- retain verified digest state;
- re-hash/revalidate every staged member after lock and before backup;
- add mutation-between-validation-and-commit tests.

Acceptance evidence: altered staged bytes fail with zero live mutation.

### Work package D — Transaction result/recovery correction

- preserve phase/cause;
- fix cleanup/recovery root semantics;
- ensure rollback and RecoveryRequired receipts are actionable.

Acceptance evidence: every injected failure reports phase and relevant member.

### Work package E — Lock-contract reconciliation

- either implement strong stale proof on supported platforms or make explicit fail-closed inspection semantics;
- update tests and docs to match actual support.

### Work package F — Documentation and platform verification

- remove stale foundation-only wording;
- add Linux/macOS CI where practical;
- add a Windows check/test lane if current code compiles/runs there without requiring future running-image semantics.

## 8. Failure, cancellation, restart, and contention semantics

Commit remains synchronous.

If any ownership or staged-integrity revalidation fails before the first backup:

- no live destination is mutated;
- transaction-owned stage may be cleaned;
- lock is released if safe;
- typed phase/cause is returned.

If failure occurs after mutation begins:

- rollback is attempted;
- rollback outcome is recorded;
- original triggering cause remains available even when rollback succeeds;
- rollback failure preserves actual recovery evidence and lock state.

## 9. Compatibility and migration

There are no external consumers yet, so breaking public corrections are explicitly authorized.

Do not add deprecated aliases merely to preserve accidental APIs from M002-M004 unless doing so materially simplifies later migration.

## 10. Required tests

At minimum:

### Ownership

- existing destination classified Owned -> replacement allowed;
- Foreign -> zero mutation;
- Unknown -> zero mutation;
- Absent + create allowed -> install succeeds;
- Absent + replacement-only policy -> fails;
- ownership changes between preflight and locked revalidation -> fails.

### Parent/path race protection

- missing parent fails without creation;
- symlinked parent fails;
- parent swapped after preparation fails;
- destination switched to symlink/non-regular/hard-linked file fails.

### Staged continuity

- mutate staged bytes after `validate()` -> commit fails before backup;
- replace staged file after `validate()` -> fails;
- change file kind/link state -> fails;
- unchanged validated bytes commit.

### Authenticity

- no public path can set "required" authenticity without verifier;
- docs/API make checksum-only state explicit.

### Results/recovery

- rollback receipt preserves original failure phase/category;
- rollback failure preserves both original and recovery failure information;
- cleanup failure points to real retained backup root;
- cleanup disposition naming matches behavior.

### Permissions

Unix tests for stage/backup directory mode and lock file mode.

### Locking

- active/ambiguous lock fails closed;
- any automatic stale recovery path requires positive process-identity evidence;
- malformed/oversized record does not trigger deletion.

## 11. Required verification commands

```bash
rustc +1.89.0 --version
cargo +1.89.0 fmt --all -- --check
cargo +1.89.0 check --workspace --all-targets --locked
cargo +1.89.0 clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo +1.89.0 test --workspace --all-targets --all-features --locked
cargo +1.89.0 doc --workspace --no-deps --locked
cargo +1.89.0 tree --workspace --locked
cargo +1.89.0 package --workspace --locked --allow-dirty
./scripts/check-local.sh
```

Also record hosted Linux CI. If macOS/Windows CI lanes are added, record them separately and do not infer behavior that the lane does not exercise.

## 12. Documentation updates

All stale M001/M002 status prose must be reconciled.

Do not rewrite historical closure records except for factual errata; this corrective plan and its closure record should explain the post-closure findings.

## 13. Acceptance criteria

M005 closes only when:

- existing destination replacement requires explicit ownership proof;
- unsafe parent creation is gone;
- staged bytes cannot change between verification and commit unnoticed;
- no unenforced authenticity requirement remains;
- cleanup and post-commit policy terminology is no longer conflated;
- terminal rollback/recovery results preserve cause and phase;
- recovery paths refer to real retained evidence;
- transaction-owned Unix state is explicitly private;
- lock behavior is documented truthfully;
- current docs match current capabilities;
- full verification is green.

## 14. Stop conditions

Stop and report if:

- ownership requires a persistent installation receipt or registry that would establish a new durable storage format;
- safe directory creation is necessary for a real consumer and cannot be represented without a larger transaction model;
- Windows running-image replacement requires a different commit architecture;
- the fix would require transport/service dependencies in core;
- a trust/signing standard must be selected.

Those are architecture decisions, not corrective implementation details.

## 15. Closure evidence required

The closure record must include:

- public API before/after summary;
- ownership decision matrix;
- staged-mutation negative evidence;
- path-parent race tests;
- Unix permissions evidence;
- failure-phase/receipt examples;
- lock semantics and any platform limitations;
- exact verification commands;
- hosted CI results;
- remaining accepted limitations by severity.

## 16. Handoff notes

This corrective is the release gate. Do not start package publication or first consumer adoption until it closes.
