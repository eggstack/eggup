# Changelog

## Unreleased

- **Fixed: a panicking service transition was reported as a post-install check
  failure.** In the commit-restore path, `transition_to` ran inside the
  post-commit `catch_unwind`, whose panic arm was labelled `PostInstallCheck`. A
  panicking `start`/`stop` during `RestoreNew` was therefore recorded under the
  wrong phase, and the receipt claimed the service state was restored. Each
  transition now catches its own panics at the phase actually executing
  (`safe_restore_state` in the lifecycle path, `safe_transition_owned_to` and a
  nested catch in the direct-runtime path), so the outer arm can only be reached
  by the check itself. Covered by
  `panicking_restore_transition_is_recorded_as_restore_new_not_a_check`.
- **Fixed: a destructive `stop` could run against an unproven target.** The
  rollback quiesce paths swallowed an `inspect()` error with `.ok()` / `if let
  Ok(..)`, treating "could not observe" as "no information, proceed". An inspect
  failure is now `Unknown` and fails closed, matching the fail-closed posture
  used for genuinely unproven ownership. The `DirectRuntimeControl` seam is a
  caller-supplied trait with no second ownership check inside `stop`, so this
  path had no defence in depth. Covered by
  `unobservable_direct_runtime_is_not_stopped_before_rollback` and
  `unobservable_service_is_not_stopped_before_rollback`.
- **Fixed: the stdin write in `SystemExecutor::run` was unbounded.** The write
  ran on the calling thread before the stdout/stderr reader threads were spawned,
  so a child that filled its output pipes before draining stdin would deadlock
  both sides and the caller's deadline could not be enforced. The readers are now
  attached first, and the write runs on its own thread driven by the same
  deadline-checked wait loop; the deadline is enforced by killing the child,
  which closes the pipe and unblocks the writer. A completed write that failed
  still fails closed, and its verdict is collected before output is trusted.
- Removed an unreachable Windows `AlreadyExists` fallback in the service
  definition write. `std::fs::rename` is `MoveFileEx(MOVEFILE_REPLACE_EXISTING)`
  on Windows and already replaces the destination, so the arm could never run; had
  it run it would have deleted the destination before the promotion, leaving a
  data-loss window.
- Removed a dead ownership gate in `CronManager::install`. `cron_merge`
  classifies the same crontab with the same marker and desired block and already
  rejects `Foreign`/`Unknown`, so the second check was unreachable — and would
  have misattributed a `Foreign` failure to `spec.id()` instead of the synthetic
  `cron:{marker}` id that owns the block.
- This release has not been published; there is no migration requirement for downstream consumers.
- Make service diagnostic byte bounds UTF-8 safe while preserving existing
  256/512-byte ceilings and lifecycle behavior.
