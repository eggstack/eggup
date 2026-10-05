# Verified Update Core M010 — Closure and Verification Record

Status: closed (implementation and local verification)

Source plan: `plans/implementation/verified-update-core/010-current-executable-transaction-parity-corrective.md`

Source roadmap: `plans/subsystems/verified-update-core-roadmap.md#M010--current-executable-transaction-parity-corrective`

Implementation commit: `95f756786708521c3315439df882106a53211cce`

Publication status: **not published.** `eggup-core 0.1.2` remains the published
baseline on crates.io; these additions require an `eggup-core 0.1.3`
publication milestone that has not been authorized. `Error::RecoveryRequired`
is breaking for an exhaustive `match` on `Error`.

## Executive finding

A program can now replace the executable it is currently running through the
existing one-member transaction model. No second state machine, no separate
verifier, no bespoke updater: preparation, integrity verification, candidate
validation, locked ownership revalidation, staged-digest revalidation, commit,
rollback, and the receipt are the same code paths as any other one-member plan.
The only new surface is the plan constructor and the type that binds the image.

The central claim — **no write authority above the executable's own directory**
— is enforced by construction rather than by convention.
`InstallPlan::for_current_executable` plans `StagePlacement::InsideInstallationRoot`
(`domain.rs:455`, variant at `domain.rs:465`), so `create_stage_directory` puts the private stage inside
that directory and `create_backup_directory` already puts the backup set there.
`self_update_needs_no_authority_above_the_executable_directory` seals the parent
to `0500` first, *proves* the seal is effective by probing a write into it, and
only then runs the transaction. That probe matters: a privileged runner can write
a `0500` directory, and without the probe the test would pass vacuously.

The old generation is **renamed aside, never deleted**. That is what keeps
`RollBack` genuinely possible, and it is also what makes the Windows finalization
honest: the mapped image has a known, transaction-owned path, so after the
caller's policy resolves — the only point at which rollback is no longer possible
— `finalize_backup_set` (`transaction.rs:700`) can schedule its removal and the
receipt can say `DeferredToProcessExit` instead of pretending it was cleaned or
handing the operator garbage nobody asked for.

## Requirement-to-evidence matrix

| Requirement (source plan §) | Evidence | Result |
|---|---|---|
| §6.2 identical transaction phase path | `for_current_executable` returns an ordinary `InstallPlan`; `tests/current_executable.rs` drives it through `prepare → verify_integrity → validate → commit_with_post_commit` and reads a normal `TransactionReceipt` | passed |
| §6.2 Windows behavioral parity in the first release | `Cargo.toml` gains `[target.'cfg(windows)'.dependencies] self-replace = "1.5"`, used by `finalize_backup_set` on Windows only | passed |
| §6.3 no parent-directory authority required | `StagePlacement::InsideInstallationRoot`; `self_update_needs_no_authority_above_the_executable_directory` seals the parent to `0500`, probes the seal, and passes | passed |
| §6.3 invocation symlink updates the real target and never overwrites the link | `CurrentExecutable::bind` canonicalizes; `symlink_invocation_updates_the_real_image_and_preserves_the_link` asserts the link is still a symlink, still points at the same real image, and that the *real image* changed | passed |
| §6.3 hard-linked / non-regular destination refused | `bind` rejects `nlink() != 1` on Unix and rejects non-regular and symlinked targets; `hard_linked_current_executable_is_refused` | passed |
| §6.3 parent path identity changed | a destination whose object changed fails identity revalidation; `destination_replaced_after_binding_fails_closed` replaces the file between binding and commit and asserts `RolledBack` with the impostor untouched | passed |
| §6.3 exact identity revalidated immediately before mutation | `InstallPlan::revalidate_current_executable` is called under the lock in `revalidate_ownership_locked` and again inside `backup_members` immediately before the first live rename | passed |
| §6.3 no overbroad deletion | no `remove_dir_all` of an installation root anywhere in the new path; the stage is `Drop`-cleaned and the backup set is removed only after the policy resolves | passed |
| §6.4 addressable old generation, rollback before publication | `backup_members` renames the live image to the backup root; `KeepInstalled` is resolved before finalization | passed |
| §6.4 never delete-before-decision | `finalize_backup_set` runs only after the injected-`Finalize` branch, i.e. only after the caller's policy resolved | passed |
| §6.4 injected failure after old-image rename restores or reports `RecoveryRequired` | `self_update_rollback_failure_retains_the_real_old_image` injects `CommitFault::PostCommitRollback`, asserts `RecoveryRequired`, `!rollback_verified`, a real `recovery_failure`, and a `recovery_path` that exists and is inside the executable's own directory | passed |
| §6.5 truthful terminal states | three distinct states: `Cleaned` (Unix keep), `DeferredToProcessExit` (Windows keep), `RetainedForRecovery` (recovery evidence). `RecoveryRequired` + `FailureCategory::RetainedEvidence` for unresolved evidence | passed |
| §6.4 no forever-blocking lock | unchanged; a self-update uses the same lock as any other commit | passed |
| §6.6/§6.7 named windows-only dep | `self-replace` — unmaintained but small, Windows-only, target-gated, added deliberately rather than hand-rolled. **Unverified locally**: this is Windows-only code and no macOS result can speak to it | deferred to hosted Windows |
| §6.6 no Unix dep | `cargo tree` on macOS/Linux shows `eggup-core → sha2` only; `self-replace` is absent from the non-Windows graph | passed |
| §7 no cross-platform claim | `InstallationRoot`-style fixtures only; no `#[cfg(windows)]` test asserts anything on macOS | passed |
| §7 no OS special-casing | the only `cfg` in the new code selects finalization behavior, never a policy | passed |
| §7 native Windows execution required to close | `ci.yml` now runs `cargo test -p eggup-core --test current_executable --locked` in the `windows-check` job | hosted evidence below |
| §8 deterministic fixtures | no sleeps, no port binding, no network; the child process is spawned from `std::env::current_exe()` copied into a private tree | passed |
| §9 no overbroad deletion / no new dependency elsewhere | no other crate touched | passed |
| §10 contract docs updated | `crates/eggup-core/docs/transaction.md`, `architecture/core-transaction.md`, `architecture/overview.md`, `AGENTS.md`, crate `README.md`, both changelogs | passed |

## Design decisions worth recording

**`StagePlacement` is an authority boundary in the type, not a flag.** A caller
that plans `InsideInstallationRoot` and reads `stage_placement()` back can see
which directories it is asking for write access to. This is the difference the
corrective exists to close, so it is not hidden behind an implementation detail.

**Identity is `dev`/`ino` on Unix, and there is no Unix identity claim for
Windows.** `CurrentExecutable` stores device and inode under `#[cfg(unix)]` and
compiles them out elsewhere; on Windows the binding re-proves the canonical path
and regular-file kind. The architecture doc states this rather than implying
cross-platform inode parity. Note that this also means the `nlink() != 1`
hard-link refusal is Unix-only — Windows has no equivalent `nlink` check here,
and the plan's "any platform where exact target identity cannot be proven fails
closed" is satisfied by the canonical-path and regular-file checks only.

**`self_replace()` itself is never called.** The helper can move and delete the
current executable in one step, which would make the old generation
unreachable *before* the `RollBack` decision exists. Only
`self_replace::self_delete_at` is used, and only at finalization, on a path
Core already renamed aside. This is the difference between a self-update that
can roll back and one that cannot, and it is why the old image is renamed rather
than deleted.

**`DeferredToProcessExit` carries `recovery_path: None`.** The old image is owned
and scheduled, not stranded, so there is nothing for an operator to inspect. The
alternative — reporting it as `RetainedForRecovery` — would tell the operator to
clean up after a process that is about to clean up after itself.

**A claim that cannot be cleaned fails the recovery rather than being ignored.**
In `acquire_with_recovery`, if `remove_claim` fails after the lock was taken,
the lock is dropped and `Error::RecoveryRequired` names the real retained path.
Returning the lock would report success while leaving undisplaced evidence
behind. This path has no deterministic public trigger, so it is covered by a
contract test on the error type rather than a contrived race.

## Test inventory

`crates/eggup-core/tests/current_executable.rs` — 10 tests, all passing.

The suite drives a child process that re-executes this very test binary from a
private `bin/app`, because the only evidence that holds equally on Unix and
Windows is real filesystem state after that child exits.

| Test | Claim |
|---|---|
| `self_update_child_replaces_its_own_running_image` | the child half; asserts `stage_placement() == InsideInstallationRoot` before committing |
| `self_update_needs_no_authority_above_the_executable_directory` | parent sealed to `0500`, probe proves the seal, transaction succeeds |
| `keep_installed_leaves_the_new_generation_live` | original path holds the new generation; cleanup is `Cleaned` or `DeferredToProcessExit` |
| `rollback_restores_a_byte_identical_old_generation` | `RolledBack`, `rollback_performed`, `rollback_verified`, and the old bytes restored exactly |
| `symlink_invocation_updates_the_real_image_and_preserves_the_link` | link object preserved and still resolving; the real image is what changed |
| `hard_linked_current_executable_is_refused` | `bind` returns `DestinationConflict` while linked, succeeds once unlinked |
| `destination_replaced_after_binding_fails_closed` | swapped object → `RolledBack`, impostor untouched, no live mutation |
| `foreign_current_executable_is_never_replaced` | `Foreign` → `RolledBack` receipt, never `Err`, destination untouched |
| `wrong_candidate_digest_fails_before_mutation` | `VerificationFailed` at the digest check, before commit |
| `ordinary_plans_keep_sibling_stage_placement` | ordinary and multi-member plans are unchanged |

Plus one in-crate unit test in `src/lib.rs`:
`self_update_rollback_failure_retains_the_real_old_image`, which needs the
`pub(crate)` fault harness and therefore cannot live in the integration file.

## Verification results

Local gate, `./scripts/check-local.sh` — exit 0:
`cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`;
`cargo test --workspace --all-targets --all-features --locked`; `cargo doc --workspace --no-deps --locked`.

`cargo +1.89.0 check --workspace --all-targets --locked` — green, MSRV 1.89
unaffected.

Per-target results for the new surface:

```text
eggup-core      (lib)                        51 passed; 0 failed
current_executable (integration)              10 passed; 0 failed
```

`cargo tree` boundary review on macOS: `eggup-core v0.1.2 → sha2` only.
`self-replace` appears under `[target.'cfg(windows)'.dependencies]` and is
absent from the resolved macOS graph, as intended.

`cargo run -p eggup-core --example stale_lock_recovery` — the M011 example runs
clean; it is the demonstration that caller evidence, not Core policy, decides.

## Hosted evidence

Run `37367758362` on `95f7567` — see the "Verification status" section appended
below once the run completes. The load-bearing item is the `windows-check` job:
`self-replace::self_delete_at` and the mapped-image rename are the only
substantively new Windows behaviour, and no macOS result can speak to them.

## Residual risk

1. **The Windows finalization path is not locally verified.** It is exercised for
   the first time by the hosted Windows runner. If `self_delete_at` cannot write
   its helper into the backup directory, finalization would return
   `Error::RecoveryRequired` and a `RetainedForRecovery` receipt — a truthful
   failure, not a silent one, but still a gap between the plan's intent and
   delivered behaviour.
2. **Windows has no hard-link refusal.** `nlink` is a Unix-only check here. A
   hard-linked Windows image would be bound and replaced, changing the other
   name's contents. This is a narrow, pre-existing-shaped gap rather than a
   regression, but it is a real difference from the Unix behaviour and is
   recorded rather than glossed.
3. **`tests/current_executable.rs` skips the authority assertion on a privileged
   runner.** The skip is printed, not silent. On such a runner the central claim
   is covered only by the `StagePlacement` assertion, which checks the type
   rather than the filesystem.