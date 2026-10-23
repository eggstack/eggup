# Changelog

`eggup-service` is on crates.io at `0.1.0`, `0.1.1` (the lockstep `0.1.1` patch),
and `0.1.2`. The `0.1.2` release below was published manually from a later source
commit than the shared `v0.1.2` tag, which denotes the `eggup-core`/
`eggup-archive` publication; the tag was not moved.

## 0.1.2 — 2026-10-05

This release publishes the already-qualified M001-M008 service substrate. No
service runtime behavior changed for the publication: the working tree carried
these entries, and they are released as-is.

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
- Service M007: make service diagnostic byte bounds UTF-8 safe while preserving
  existing 256/512-byte ceilings and lifecycle behavior.

The published dependency requirement is unchanged (`eggup-core ^0.1.0`), so
consumers of `0.1.1` need no migration and pick up the corrected behavior by
upgrading within the `0.1.x` range. No public API was added, removed, or
changed by this release.

## 0.1.3 — 2026-10-09

- **Fixed: owned failed systemd units could never complete `stop`.** The
  adapter now recognizes the specific `ActiveState=failed` case without
  changing the public `LifecycleState` enum, revalidates the exact `ExecStart`
  identity around the stop, and reports completion only after manager job,
  process IDs, active-state output, and cgroup evidence agree that the unit is
  quiescent. Unknown states and remaining cgroup tasks fail closed. No
  `reset-failed`, privilege escalation, or new dependency is introduced.
  The runtime implementation is Service M010 source commit
  `0bde3fefbda07019529ad7566c02e6e4ec141fd6`; the package's exact source
  revision is recorded by `.cargo_vcs_info.json` as
  `feb6ae5aea4c9b61c4051957f3662ca49d845f9e`. The published crate checksum is
  `9f7f7ea854577158e66aa202709ab1c97a3aedcf00b06c1ab914d25b132124dc`.

## Unreleased

**Not published by this milestone.** The changes below are on the source line
only; `eggup-service 0.1.3` remains the published version and the `v0.1.3` tag
is unmoved. Republication is a separate, later milestone. **No consumer
migration is required for the type-identity changes**: `eggup_service::Ownership`,
`eggup_service::CronOwnership`, and `eggup_service::CommandOutput` all keep
resolving under their existing spellings.

The milestone entries below are also **not published** and require **no
consumer migration until a republication milestone ships them**. One of them —
the typed manager failure kinds — is a source-breaking change for a consumer
that matches `ServiceError::Manager` positionally or branches on its message
text; that migration becomes necessary only at republication, and the exact
replacement is named in its entry below.

- **Fixed: two divergent copies of the destructive commit/rollback engine.**
  `disposition.rs` and `lifecycle_update.rs` each carried a full orchestrator
  and their own copies of the transition, rollback-quiesce, panic-safe-inspect,
  panic-safe-restore, deadline, and detail-truncation mechanics. The copies had
  already drifted apart in safety-relevant ways: the disposition copy did not
  re-inspect after a transition, it tolerated an ownership that was no longer
  `Owned`, its rollback quiesce swallowed an `inspect()` error and could stop an
  unproven target, its rollback restore was not panic-guarded, and the
  lifecycle copy could panic on an `unreachable!()` for a desired state outside
  `{Running, Stopped}`. There is now **one** engine (`commit_engine`), and both
  public orchestrators reach it. The stricter behaviour wins everywhere: a
  manager's own `completed` flag is a self-report and is now re-verified by
  observation; an ownership that is no longer `Owned` is always a typed
  failure; an unobservable or non-owned service is never stopped before a
  rollback; and a desired state outside `{Running, Stopped}` produces a typed,
  bounded failure instead of a panic.
- **Added: two named, separately testable commit-engine policy inputs.**
  `CommitEnginePolicy::post_transition_verification`
  (`PostTransitionVerification::{Verify, TrustManagerReport}`) and
  `CommitEnginePolicy::ownership_loss_tolerance`
  (`OwnershipLossTolerance::{Deny, TolerateAlreadyStopped}`) default to the
  stricter of the two former behaviours. The permissive former disposition
  behaviour survives only as an explicitly selected, non-default input, and the
  selected policy is observable in the returned receipt's `engine_policy`
  field. `commit_with_lifecycle_with_engine_policy` and
  `commit_with_disposition_with_engine_policy` select it;
  `commit_with_lifecycle` and `commit_with_disposition` keep their existing
  signatures and run the engine under the default policy.
- **Fixed: three structurally identical ownership enums.** `eggup_service::Ownership`
  is now a re-export of `eggup_core::Ownership`, and `eggup_service::CronOwnership`
  is a public type alias of it, so all three denote **one** type and no
  conversion is needed anywhere. `Ownership` is never inferred, and `Foreign` /
  `Unknown` still fail closed.
- **Renamed: the manager command output type.** `CommandOutput` is now
  `ManagerCommandOutput`; the old spelling survives as a type alias, so existing
  `CommandExecutor` implementations written against either spelling still
  satisfy the trait. The alias is deliberately **not** deprecated — that is a
  publication-policy decision owned by the later republication milestone.
- **Fixed: two byte ceilings for one data class.** Manager-output diagnostic
  text is now bounded by one declared ceiling, `MAX_DIAGNOSTIC_BYTES` (512
  bytes), which the crate doc now names. The former second, smaller 256-byte
  ceiling is gone.

- **Changed: manager failures carry a machine-readable kind.**
  `ServiceError::Manager` is now a struct variant carrying
  `ManagerFailureKind` (`Spawn | Timeout | Permission | NotFound | Protocol |
  LocalFilesystem`), the bounded diagnostic detail, and the platform OS code
  when the platform supplied one. `ServiceError::manager_kind()` and
  `ServiceError::manager_os_code()` are the programmatic discriminators. The
  crate no longer classifies its **own** failures by substring-matching its own
  error text: the `permission_hint` remediation re-classifier and its eleven
  call sites are gone, and `inspect_host`'s three `contains("missing")` tests
  now read the typed kind. The Windows SCM mapping stays lossless — the raw OS
  code remains reachable through `manager_os_code()` and is also rendered into
  the bounded detail. **Consumer impact:** code that matched
  `ServiceError::Manager(detail)` must now match
  `ServiceError::Manager { kind, detail, os_code }` and should branch on `kind`
  rather than on the message text. A manager that *runs* and reports an access
  denial in its own stderr is classified `Protocol`, because the platform
  supplied no structured denial; only a structured platform denial is
  `Permission`.
- **Changed: one timeout model.** Every adapter now creates exactly one
  `OperationDeadline` per public operation and threads it through every manager
  invocation that operation makes, so no invocation is granted a fresh full
  budget. `install` / `uninstall` / `inspect` spend the descriptor's
  `transition_timeout` for the whole operation; `start` / `stop` / `restart`
  spend the caller-supplied `timeout`. Previously a single systemd `install`
  could consume roughly 4-5x its declared budget. This is a fail-closed
  tightening: an operation that previously overran its budget now fails with a
  `Timeout` kind instead of overrunning.
- **Changed: cron has one listing, one ownership rule, and an explicit
  executable-containment gate.** `CronManager::inspect` now issues exactly one
  `crontab -l` (it previously spawned two and discarded one result). The
  ownership rule is stated once and used identically by `inspect`, `install`,
  and `uninstall`: ownership is decided only by `cron_classify`, i.e. exact
  content identity of the managed block body. The former whole-crontab
  containment test is gone — an unrelated crontab entry mentioning the
  executable path no longer makes a non-matching block look `Owned`. **Consumer
  impact:** the managed block body must embed the absolute executable path of
  the spec. A consumer whose `desired_block` does not contain
  `spec.executable()` now receives `ServiceError::InvalidInput` from `inspect`,
  `install`, and `uninstall`, where it previously received success (`install`)
  or a normal result. The pre-write re-listing before a destructive write is
  retained.
- **Fixed: one byte ceiling, applied consistently.** The crate's single
  declared ceiling for manager-derived free text, `MAX_DIAGNOSTIC_BYTES` (512
  bytes), is now the only cap on that data class; the redundant 256-byte
  re-cap and the text-derived `permission_hint` re-classification are gone.
- **Added: documented failure semantics.** Every public `Result`-returning item
  now carries a `# Errors` rustdoc section naming the concrete error variants
  and the condition for each, and the `FakeExecutor` methods that can panic
  carry `# Panics` sections. The Windows SCM public surface is deliberately
  excluded: service milestone `014` owns it.

### Service milestone 014 — Windows SCM execution and public-API surface

- **Fixed: five latent panics in the native Windows SCM adapter.**
  `windows_scm.rs` carried five `unreachable!()` arms on the
  "the SCM returned something unexpected" path of an SCM service-handle open.
  They were unreachable only because the handle-open helper's inner error type
  was `u32` and it produced exactly two codes — `1060` (does not exist) and
  `1072` (marked for deletion). Any other SCM code, or a future change to that
  helper's contract, would have reached them and **panicked inside a
  privileged SCM operation**. The inner error type is now a named
  `OpenFailure` that can carry any code, and all five sites return a typed
  error instead: `1060` and `1072` keep their exact lossless meanings
  (`Absent`, `MarkedForDelete`, and the "the service disappeared before the
  operation" conflict), and every other code becomes a typed
  `ServiceError::Manager` that still carries the raw OS code. **No new error
  variant and no new taxonomy were added**: the codes are surfaced through the
  existing `ServiceError::Manager` / `ServiceError::Conflict` constructors and
  the existing `ManagerFailureKind` set. The observable error text, kind, and
  `os_code` for every previously-reachable path are unchanged.
- **Fixed: the native Windows SCM backend had never been executed on any
  platform.** The `windows-service` mapping layer and the SCM calls themselves
  were covered only by a fake backend. There is now a Windows-only fixture
  (`tests/windows_scm_live.rs`) that registers a **disposable** service
  through the crate's **public** API — `WindowsScmInstall` plus
  `WindowsScmManager::new`, never a crate-internal back door — and drives
  install → inspect → ownership → start → stop → uninstall against the
  genuine Service Control Manager, asserting the `windows-service` mapping
  layer explicitly (start type, error control, service dependencies, and the
  `ServiceInfo` round-trip). The fixture removes its own registrations on
  every exit path, including a failing assertion; it never touches,
  disables, stops, or reconfigures a pre-existing service; it waits only
  under explicit deadlines and fails with the observed state on timeout; and
  it skips with a **recorded reason** when the runner lacks the administrator
  rights a real registration needs, rather than passing silently.
- **Added: a local, dependency-free public-API surface guard.**
  `tests/public_api_surface.rs` binds every public item of the crate to an
  explicit type, so a removed item or a changed signature is a **compilation
  error** in the gate `scripts/check-local.sh` already runs. The committed
  snapshot `tests/public-api-surface.txt` is the reviewable artifact; it is
  maintained by hand, because the only tool that can emit a machine-readable
  public surface from Rust sources needs a nightly compiler and is therefore
  not part of the stable/MSRV lanes. The existing
  `tests/api_compatibility.rs` is **retained** and folded into the guard
  rather than deleted. **No new dependency, and no public API change.**
- **Changed: the SCM state enums are gated per variant.** The two enum-wide
  `#[allow(dead_code)]` attributes on `ScmState` and `ScmLookup` are replaced
  by a per-variant allowance, so a variant added later is no longer covered by
  an allowance that was written for the variants that exist today. The variant
  sets are unchanged.
- **Added: `# Errors` documentation for the Windows SCM public surface.**
  `WindowsScmManager`, `WindowsScmInstall` and its validating builders, the
  SCM configuration enums, and the SCM error mapping now name their concrete
  failure conditions, including the access-denied case (SCM code `5`) and the
  "the service disappeared before the operation" case. **No `# Panics` section
  is added**: the absence of panics on those paths is the outcome of the
  typed-error conversion above, not an independent claim.
