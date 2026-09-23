# Verified Update Core M007 Closure

Status: closed and qualified

Implementation commit: `8d5fc12f7224145285f22d0975a7bb91e1e363ea` (`feat(core): add post-commit failure policy`).

Plan: `plans/implementation/verified-update-core/007-post-commit-policy-and-deferred-finalization.md`.

## Result

`ValidatedTransaction::commit()` retains its immediate-finalize behavior. The additive `commit_with_post_commit(ownership, policy, check)` path commits the full artifact generation, invokes one caller check while the mutation lock and backup set remain owned, and resolves the result under `PostCommitFailurePolicy::{KeepInstalled, RollBack}`. The callback receives no transaction mutation authority. Its error is converted to bounded core-owned text (at most 512 bytes); panics are caught and recorded as a post-commit check failure.

## Public API and state transitions

Before M007, `commit()` performed ownership/stage revalidation, backup, complete commit, and backup finalization before returning; no caller verification could choose rollback after the new generation was live. After M007, `commit()` is unchanged and callers needing a post-install check can use `commit_with_post_commit`. Receipts expose `post_commit_failure()` independently of the primary failure and cleanup disposition.

```text
validated transaction
  -> lock + ownership/stage revalidation
  -> backup old members; install every new member
  -> post-commit check while lock + backup remain held
       success -----------------------> finalize -> Committed
       failure + KeepInstalled -------> finalize -> Committed + post-commit report
       failure + RollBack ------------> restore and verify -> RolledBack + trigger report
                                           rollback failure -> RecoveryRequired
                                                              + both reports + evidence
```

| Outcome | Live generation | Receipt | Recovery evidence |
|---|---|---|---|
| Check succeeds | Complete new generation | `Committed`, no post-commit report | Ordinary cleanup |
| Check fails, `KeepInstalled` | Complete new generation | `Committed`, post-commit report | Finalize failure follows retained-recovery contract |
| Check fails, `RollBack` succeeds | Previous generation restored; newly created members removed | `RolledBack`, triggering post-commit report | Ordinary cleanup |
| Check fails, rollback fails | May require operator recovery | `RecoveryRequired`, post-commit and rollback reports | Real retained evidence path |

## Qualification evidence

- Tests exercise a multi-member callback observing all new members and lock contention during the callback; successful rollback tests restore an old member and remove a member that was previously absent.
- Failure coverage includes `KeepInstalled`, rollback success, injected rollback failure with both reports/evidence, finalize failure after `KeepInstalled`, bounded UTF-8 diagnostics, and callback panic under rollback policy. The existing immediate commit tests remain green.
- Core has no public pending-transaction handle. A Rust panic in the callback is caught and processed under the selected policy. Dropping a pending handle is inapplicable because none escapes. Process termination or power loss during the callback cannot execute Rust rollback/finalization; the core makes no crash-durability or automatic-recovery claim for that event.
- `cargo tree -p eggup-core --locked` reviewed: the core has no service, network, or acquisition dependency; its normal dependency is SHA-256 support only.
- Local checks passed: formatting; locked workspace check; all-feature workspace Clippy with warnings denied; 43 `eggup-core` tests; 165 workspace tests across nine suites; workspace docs; package build/verification; Rust 1.89 workspace check and core tests (43); repository `scripts/check-local.sh`; and `git diff --check`.
- Hosted [CI run 35881754923](https://github.com/eggstack/eggup/actions/runs/35881754923) passed all jobs: stable checks, Rust 1.89 MSRV check, macOS all-feature workspace tests, and Windows workspace check. Windows evidence is compile/check only; no Windows live running-image replacement claim is made.
- No unresolved M007 acceptance findings remain. No persistent journal, service semantics, or network behavior was added.

## Handoff

Service Lifecycle M005 is ready for plan authoring. It should use `ValidatedTransaction::commit_with_post_commit` for the transaction boundary and retain service restart/health policy in the service layer. The API does not provide a timeout: callers must bound their supplied check. The callback runs while the mutation lock is held, so callers should keep it appropriately bounded.
