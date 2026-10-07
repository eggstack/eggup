# Verified Update Core Milestone 010 — current-executable transaction parity corrective

Status: closed; see `plans/closure/verified-update-core/010-status.md` (implementation `95f75678`; hosted-qualified on Stable/MSRV/macOS/Windows, run `37376971555`)

Repository baseline: `db5b3121f17f92029a47c389e68a464bc4478b27`

Source roadmap: `plans/subsystems/verified-update-core-roadmap.md`

Primary class: invariant / capability corrective

Hard dependencies:

- Verified Update Core M009 is closed; the 0.1.2 core/archive package boundary is published and qualified.
- ADR-0002 already defines single-binary self-update as an `ArtifactSet` with one member.
- Service M006 is closed and supplies the manager/direct/stopped/foreign-preserved lifecycle composition this path must be able to feed.
- No acquisition milestone is required: this work starts from an already acquired local candidate.

Reference-consumer evidence:

- `eggstack/gregg@1aac89f1a82fcd347066379b55105e2ff6bfe770` requires current-executable replacement that assumes write authority only in the executable directory and has qualified Windows running-image behavior.
- `eggstack/eggpool@fe3c308df1450cd3216f50abc160a0a9f426f8d9` has a standalone-Rust transaction with explicit rollback and post-restart rollback behavior; package-manager/provenance paths remain out of scope.

## 1. Objective

Close the gap between Eggup's generic one-member transaction model and a real program updating the executable that is currently running.

The resulting path remains an ordinary Eggup transaction: integrity and candidate validation precede mutation, ownership is revalidated under the mutation lock, the old generation remains rollback-addressable until finalization, and the caller chooses `KeepInstalled | RollBack`.

The corrective additionally provides the platform behavior real self-updaters require:

- no write authority broader than the executable's own installation directory;
- same-filesystem staging for the committed executable;
- exact current-executable identity rather than PATH/basename discovery;
- deliberate symlink behavior;
- Windows running-image replacement with rollback evidence retained until post-commit policy resolves.

Do not replace ADR-0002 with a second updater state machine.

## 2. Readiness and dependencies

This work is dependency-ready. The normative specification already requires a one-member artifact-set self-update, same-filesystem staging where practical, destination-writability preflight, Windows running-image replacement semantics, and explicit rollback/recovery.

No new ADR is required unless implementation proves Windows self-replacement cannot be expressed while preserving ADR-0002 rollback semantics.

The plan may execute before or after the pending package-publication work. If the workspace version or planning head changes first, refresh the execution baseline and repeat package/dependency review without changing these invariants.

## 3. Current implementation evidence

Current `eggup-core` creates its stage as a sibling of the installation root:

```text
<root-parent>/.eggup-stage-<root>-...
```

For `/usr/local/bin/app`, using `/usr/local/bin` as the installation root therefore requires creating state under `/usr/local`. That is broader authority than the mature Gregg updater, which needs write permission in `/usr/local/bin` itself.

The ordinary commit path uses filesystem renames from live destination to backup and staged candidate to live destination. Current Eggup does not have native Windows evidence proving that a running image can be renamed aside, a new generation installed at the original path, and the old image restored or safely cleaned after process exit.

Gregg currently uses `self-replace`; EggPool maintains an explicit rollback pathname. Together they define the generic mechanism gap. A direct `self_replace::self_replace` call is insufficient for this milestone if it hides the old-image pathname before Eggup's possible `RollBack` decision.

## 4. Invariants that must not regress

- Self-update remains a one-member `ArtifactSet`; no release/version/fallback policy enters Core.
- Existing multi-member `InstallPlan` preparation/commit behavior stays source- and behavior-compatible.
- Unverified bytes never execute or become live.
- Candidate identity is verified before live mutation.
- Destination ownership is classified before and revalidated under the mutation lock.
- Foreign/unknown destinations fail closed.
- No PATH search decides which executable to replace.
- No implicit privilege elevation.
- `KeepInstalled` and `RollBack` remain caller policy.
- A rollback-capable transaction does not schedule/delete the old running image before the rollback decision resolves.
- RecoveryRequired remains distinct when neither generation can be proven coherent.
- Core gains no HTTP/TLS/service-manager/release-authority dependency.
- Rust 1.89 and first-party unsafe denial remain intact.

## 5. Scope and non-scope

### In scope

- an explicit current-executable transaction entry point or equivalent one-member specialization;
- exact current-executable resolution and canonical path binding;
- executable-local/same-filesystem staging needing no write permission in the installation root's parent;
- preflight of the actual platform mutations;
- Unix/macOS running-image rename behavior;
- Windows running-image rename-aside/install/rollback/final-cleanup behavior;
- symlink/hard-link rules;
- reuse of existing integrity, validator, ownership, lock, rollback, receipt, and post-commit-policy machinery;
- deterministic and native-platform qualification;
- dependency/footprint review if a target-specific helper dependency is added.

### Explicitly out of scope

- release discovery, target/asset mapping, Cargo/source fallback;
- service-manager policy or CLI presentation;
- package-manager transitions;
- EggPool provenance/PEP-440/database compatibility;
- Gregg client-daemon protocol behavior;
- generic uninstall;
- changing ordinary bundle stage placement merely for convenience.

## 6. Required production changes

### 6.1 Add a current-executable binding without a second transaction model

Add a narrow public constructor/helper that binds the exact current executable, its installation directory, one executable destination member, caller-supplied integrity/candidate requirements, and caller-supplied ownership verifier.

Naming is implementation-owned, but the API should make the special authority obvious, e.g. `CurrentExecutablePlan` or `InstallPlan::for_current_executable`. It must advance into the same prepared/verified/validated/receipt model, not return a bespoke updater result.

Do not infer release identity, product identity, or current version from the executable.

### 6.2 Use executable-local transaction state

For this specialized path, private stage and rollback paths needed by the platform commit must live in the executable's own parent directory or another location whose write requirement is no broader.

Required proof:

```text
parent-of-executable-directory unwritable
executable directory writable
-> preparation + commit succeeds
```

The ordinary multi-member stage layout may remain unchanged. Staged names remain create-new/unpredictable and owner-private, and the candidate must be on the same filesystem before commit.

### 6.3 Bind symlink and destination identity deliberately

Resolve `std::env::current_exe()` and canonicalize the executable target for mutation. Updating an invocation symlink must not overwrite the symlink object.

Cover direct path, Unix symlink invocation, destination replacement between preparation and commit, hard-linked/non-regular executables, and parent path identity changes. Any platform where exact target identity cannot be proven fails closed.

### 6.4 Introduce a rollback-addressable running-image commit primitive

The current-executable path needs an internal primitive with these phases:

```text
validated staged candidate
 -> revalidate exact live executable
 -> move live running image to transaction-owned rollback path
 -> publish candidate at original executable path
 -> validate live new generation
 -> caller post-commit/lifecycle work
    -> KeepInstalled: finalize old-image cleanup
    -> RollBack: restore old image and verify restoration
```

On Unix/macOS this may reduce to same-directory renames.

On Windows, qualify that the running executable may be renamed but not deleted while mapped. Preserve the renamed old image at a known Eggup-owned path until the post-commit decision is complete.

Do not use a helper that irreversibly schedules old-image deletion before a possible `RollBack`.

If a safe helper such as `self-replace` is used for deferred deletion after successful finalization, gate it to the relevant platform where possible and record dependency/size impact. Calling `self_replace()` directly is not acceptable unless old-generation rollback addressability is first proven.

### 6.5 Make final cleanup truthful

After `KeepInstalled` success on Windows, old-image deletion may complete only after the updater process exits.

The terminal contract must distinguish new generation committed with cleanup safely scheduled/owned, rollback completed with old generation restored, and cleanup evidence retained because safe deletion could not be arranged.

Prefer clarifying existing `CleanupDisposition`/recovery-path semantics. If the receipt cannot truthfully represent deferred cleanup, stop and open a narrow API corrective rather than encode the state in a string.

### 6.6 Preserve ordinary transaction semantics

Existing one-member and multi-member callers continue using the generic path without acquiring current-executable assumptions. No default behavior change for CodeGG/Egress-style bundles.

## 7. Ordered work packages

1. Add failing authority tests showing current staging requires the installation-root parent today.
2. Add guarded/native Windows evidence showing ordinary rename is not sufficient proof for current-executable replacement.
3. Introduce the current-executable binding and executable-local stage placement.
4. Factor the running-image commit primitive while reusing existing ownership, digest revalidation, backup/rollback, and receipt logic.
5. Add Unix symlink/current-exe identity regressions.
6. Add Windows child-process fixtures for successful replacement and `RollBack`.
7. Compose the new path through existing post-commit tests for both policies.
8. Measure package/dependency impact.
9. Run full workspace/native qualification.
10. Reconcile roadmap/registry and write M010 closure.

## 8. Failure, restart, cancellation, and contention semantics

Before the first live rename, every error leaves the old executable untouched.

After the old image is renamed aside but before candidate publication, restore immediately; inability is RecoveryRequired.

After the candidate is live, `KeepInstalled` retains the verified new executable while recording later failure; `RollBack` restores and verifies the old executable. Rollback failure preserves the exact old-image recovery path.

The mutation lock covers the complete live mutation and rollback/finalization decision. Process interruption/power loss is never claimed automatically healed; owned artifacts remain diagnosable and are not blindly deleted.

## 9. Compatibility and migration

This is additive pre-1.0 API work. Existing `InstallPlan`, validator, ownership, receipt, and lifecycle consumers should compile unchanged.

A target-specific helper dependency requires package/dependency review. Prefer no new dependency on Unix. A Windows-only safe helper is acceptable if materially smaller/safer than reimplementing platform deletion machinery and it does not change non-Windows graphs.

Gregg/EggPool migration remains downstream work.

## 10. Required tests

At minimum:

- executable directory writable + parent unwritable succeeds;
- unwritable executable directory fails before live mutation;
- candidate staged on same filesystem;
- wrong candidate identity/version fails before mutation;
- ownership changes before commit fail;
- direct current-executable update succeeds on Linux/macOS;
- Unix symlink invocation updates real executable and preserves symlink;
- hard-linked/non-regular destination refused;
- `KeepInstalled` post-commit failure leaves new executable live;
- `RollBack` post-commit failure restores byte-identical old executable;
- injected failure after old-image rename restores or reports RecoveryRequired;
- Windows subprocess updates its own running executable and original path contains new generation after exit;
- Windows injected post-commit failure restores old generation;
- Windows cleanup is verified after process exit or receipt truthfully retains recovery evidence;
- ordinary multi-member regression suite unchanged.

Native Windows execution is required to close M010.

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

Hosted closure requires Stable Linux, MSRV 1.89, macOS, and a Windows job executing the self-replacement child fixture.

## 12. Documentation updates

Update `architecture/core-transaction.md`, core README/rustdoc, root/core Unreleased changelogs, source roadmap, consumer-adoption dependency status, registry, and new `plans/closure/verified-update-core/010-status.md`.

Do not rewrite M009 publication history.

## 13. Acceptance criteria

M010 closes only when the current executable is representable through the existing transaction/receipt model; no write authority above its directory is required; exact identity is revalidated immediately before mutation; Unix symlink behavior is explicit; Windows native evidence proves running-image replacement and rollback addressability; both post-commit policies are covered; failed rollback is truthful; ordinary bundle behavior does not regress; Core remains policy-neutral; dependency impact is recorded; and Stable/MSRV/macOS/Windows qualification is green.

## 14. Stop conditions

Stop and write a narrower architecture/API corrective if Windows rollback requires a persistent multi-process journal/protocol, if verified-candidate ordering would be weakened, if current-executable semantics would become the default for ordinary bundles, if a helper materially increases non-Windows footprint, if exact target identity cannot be proven, or if the receipt cannot truthfully represent deferred Windows cleanup.

Do not weaken `RollBack` merely to match a one-way self-replacement library.

## 15. Closure evidence required

Record implementation SHA(s), exact public API additions, before/after write-authority evidence, Unix symlink behavior, Windows replacement/rollback results, package/dependency and size delta, full command results and hosted run id, receipt examples, unresolved findings by severity, and the Gregg/EggPool unblock audit.

## 16. Handoff notes

The objective is not a convenience `update()` function. It is to make the existing one-member transaction model true for the hardest one-member case: replacing the image executing the transaction.

Preserve the old image as transaction-owned rollback evidence until Eggup's existing post-commit policy says it may be finalized.
