# eggup-service

Manager-neutral service-registration, ownership, state, and
lifecycle-snapshot model plus reusable Unix manager mechanics
(systemd, launchd, user crontab) and a native Windows SCM adapter.

- `ServiceId` / `ServiceSpec` identify desired registrations with exact
  executable, arguments, and config evidence.
- `Ownership::{Absent, Owned, Foreign, Unknown}` **is** `eggup_core::Ownership`:
  the service crate re-exports the canonical type rather than defining its
  own, so the two crates denote one type and no conversion exists.
  `Ownership::Owned` is never inferred — it is returned only by a
  caller-proven ownership check. Destructive manager operations are authorized
  only for `Owned`; `Foreign` and `Unknown` fail closed.
  `CronOwnership` is a public type alias of the same canonical type.
- `LifecycleState::{Stopped, Running, Transitioning, Unknown}` is distinct
  from application health (`HealthState::{Healthy, Degraded, Unknown}`).
- `LifecycleSnapshot` preserves pre-update intent (`was_running`,
  `was_registered`) so orchestration can restore "was running" versus "was
  registered but stopped". Stopped services remain stopped by default.
- `ServiceManager` trait plus `TestDoubleManager` for deterministic tests;
  `HealthProbe` seam is consumer-supplied and never mutates manager state.
- `WindowsScmManager` uses the safe `windows-service` SCM wrapper on Windows
  and a Windows command-line parser for exact executable/argument ownership.
  Service key name alone never authorizes a mutation; ambiguous registrations
  are `Unknown` and foreign registrations are denied.
- `WindowsScmInstall` supplies caller-owned display/start/error settings.
  Creation never starts the service. Optional caller-supplied service or
  load-order-group dependencies are supported. Refresh preserves service type,
  unspecified dependencies, account, load-order group, and tag fields outside
  the adapter's ownership. Account selection is create-time only; custom passwords,
  descriptions, and recovery actions are not supported by this generic adapter.
- Windows start/stop/restart polling shares one monotonic deadline per
  transition. Uninstall reports incomplete while SCM still exposes a
  marked-for-delete entry and completes only after confirmed absence. Access
  denial returns bounded remediation and never starts an elevation flow.
- The native Windows SCM adapter returns **typed errors on every path**. The
  SCM's "the service does not exist" (`1060`) and "the service is marked for
  deletion" (`1072`) codes keep their exact meanings — `Absent`,
  `MarkedForDelete`, and the "the service disappeared before the operation"
  conflict — and every other SCM code becomes a `ServiceError::Manager` that
  still carries the raw OS code. No SCM condition panics, and no new error
  variant or taxonomy was added: the codes are surfaced through the existing
  `ServiceError::Manager` / `ServiceError::Conflict` constructors and the
  existing `ManagerFailureKind` set.
- The native Windows SCM backend is exercised against the **real** Service
  Control Manager by a Windows-only fixture (`tests/windows_scm_live.rs`): it
  registers a disposable service through the crate's public API and drives
  install → inspect → ownership → start → stop → uninstall, asserting the
  `windows-service` mapping layer explicitly. It removes its own registrations
  on every exit path, never touches a pre-existing service, waits only under
  explicit deadlines, and skips with a recorded reason when the runner lacks
  the administrator rights a real registration needs.
- `windows-service` is a Windows-target dependency. `windows-args` is the small
  cross-platform parser dependency used for the same ownership tests on Unix.
- Bounded `TransitionResult` with conflict diagnostics; no automatic privilege
  elevation; no updater/release/network policy.
- Crate-produced service errors and manager stderr excerpts share **one**
  declared ceiling, `MAX_DIAGNOSTIC_BYTES` (512 bytes), truncated only at
  UTF-8 character boundaries.
- `ServiceError::Manager` is a **struct** variant: it carries a machine-readable
  `ManagerFailureKind` (`Spawn | Timeout | Permission | NotFound | Protocol |
  LocalFilesystem`), the bounded diagnostic detail, and the platform OS code
  when the platform supplied a structured one. The kind — not the text — is the
  carrier of meaning: `ServiceError::manager_kind()` and
  `ServiceError::manager_os_code()` are the programmatic discriminators, and no
  adapter or helper classifies a failure by substring-matching its own error
  text. The classification is decided by two pure mapping functions
  (`ManagerFailureKind::from_spawn_error`,
  `ManagerFailureKind::from_scm_os_code`), so it is assertable on every lane
  without a real privilege failure, a real spawn, or a wall-clock sleep.
- Every adapter creates exactly **one** `OperationDeadline` per public
  operation and threads it through every manager invocation that operation
  makes. No invocation is granted a fresh full budget: the single initial clamp
  happens at deadline construction and each call receives the *remaining*
  budget. `start`/`stop`/`restart` use the caller-supplied `timeout`;
  `install`/`uninstall`/`inspect` use the descriptor's `transition_timeout` for
  the whole operation.
- Cron has **one** ownership rule, stated once and used by `inspect`, `install`,
  and `uninstall` alike: ownership is decided only by `cron_classify`, i.e.
  exact content identity of the managed block body. `inspect` lists the
  crontab exactly once, and the pre-write re-listing before a destructive write
  is retained. An **executable-containment gate** applies before any listing
  or mutation: the caller's managed block body must contain
  `spec.executable()`, otherwise every cron operation refuses with
  `ServiceError::InvalidInput`.
- `commit_with_lifecycle` and `commit_with_disposition` are two orchestrators
  over **one** commit-and-rollback engine (`commit_engine`). The transition,
  rollback-quiesce, panic-safe-inspect, panic-safe-restore, deadline, and
  detail-truncation mechanics have a single implementation, and both
  orchestrators reach it. A desired lifecycle state outside `{Running,
  Stopped}` is a typed, bounded failure, never a panic.
- The two behavioural inputs that used to differ between the former engine
  copies are named, separately testable policy inputs on `CommitEnginePolicy`:
  - `PostTransitionVerification::Verify` (default) re-inspects the manager and
    validates the observed ownership and state after every transition — a
    manager's own `completed` flag is a self-report, not evidence;
    `TrustManagerReport` is the explicitly selected non-default input.
  - `OwnershipLossTolerance::Deny` (default) makes an ownership that is no
    longer `Owned` a typed failure; `TolerateAlreadyStopped` is the
    explicitly selected non-default input.
  The selected policy is observable in the returned receipt's `engine_policy`
  field.
- `commit_with_lifecycle` composes an already validated `eggup_core` transaction
  with owned-service quiescence, successful-update restoration, a bounded
  caller-owned `PostInstallCheck`, and Core's `KeepInstalled` / `RollBack`
  policy. It requires an existing `Owned` registration in `Running` or
  `Stopped` state; it never installs or refreshes the registration.
- `commit_with_disposition` (M006) generalizes orchestration with a
  product-neutral `UpdateRuntimeDisposition::{ManagedRunning, ManagedStopped,
  DirectRunning, Stopped, ForeignPreserved}`. Artifact commit authority and
  manager mutation authority are separate facts: `ForeignPreserved` performs
  zero manager mutation, `DirectRunning` uses only the caller-owned
  `DirectRuntimeControl` seam (exact instance/config, bounded deadlines, no
  health-protocol content), and `Stopped` fabricates no restart. Pure
  `plan_unix` / `plan_windows` planners reproduce the reference decision
  matrix without native managers. The disposition observed after preparation
  is revalidated immediately before mutation; owned-to-foreign/unknown or
  direct-identity changes fail before destructive action with zero further
  mutation.
- With `RestoreIntent::Preserve`, a stopped service stays stopped after
  success. `RestoreIntent` only controls the successful new generation; after
  a successful artifact rollback, orchestration restores the pre-update
  running/stopped state. If Core reports `RecoveryRequired`, it does not
  automatically start the service against uncertain artifacts.
- The post-install check receives the remaining shared deadline and must
  finish within it. Existing `HealthProbe` has no timeout contract and alone
  does not meet this bound. Application health semantics remain caller-owned.
- M003 hardening: each start/stop/restart validates one monotonic caller
  deadline shared by ownership queries, manager commands, and state polling.
  Zero transition timeouts are rejected; expiry is returned as an incomplete
  transition or a manager timeout.
- Production `SystemExecutor` resolves `systemctl`, `launchctl`, and `crontab`
  only from fixed absolute platform paths, never ambient `PATH`. It clears the
  child environment; only user-scoped systemd retains `DBUS_SESSION_BUS_ADDRESS`,
  `XDG_RUNTIME_DIR`, and `SYSTEMD_BUS_ADDRESS` when present.
- `ManagerCommandOutput` is the bounded output of one manager command
  invocation. The former `CommandOutput` spelling survives as a type alias to
  it, so existing `CommandExecutor` implementations written against either
  spelling still satisfy the trait. The alias is deliberately not deprecated;
  its disposition is settled at the intentional 0.2/1.0 API boundary.
- systemd/launchd config ownership is proven by one exact config path occurrence
  in canonical argv. Missing identity is `Foreign`; repeated/ambiguous identity
  is `Unknown`.
- M002 Unix adapters (`SystemdManager`, `LaunchdManager`, `CronManager`)
  implement `ServiceManager` on top of the M001 contract:
  - shared bounded command runner (literal argv, no shell, null/controlled
    stdin, bounded output, deadline with kill/reap, injectable `FakeExecutor`
    for tests);
  - systemd: explicit `--system`/`--user` scope, narrow `ExecStart` parsing
    (ambiguous/specifier shapes yield `Unknown`), atomic definition writes,
    explicit `enable`/`daemon-reload` only, bounded `start`/`stop`/`restart`
    with state confirmation, no `sudo`; a recognized exact-owned `failed`
    unit is stopped only after ownership revalidation and proof that its manager
    job, main/control processes, and cgroup work are gone;
  - launchd: explicit `gui/<uid>` vs `system` targets (never guessed),
    structured `ProgramArguments` parsing (malformed yields `Unknown`),
    label-only never proves ownership, bounded `bootstrap`/`bootout`/
    `kickstart`/`stop`, foreign labels fail closed;
  - cron: managed `# BEGIN/# END <marker>` block (caller-owned marker +
    exact block), pure `cron_merge`/`cron_remove` preserving unrelated
    bytes, idempotent install, owned-only removal, no redundant `crontab -`
    writes, re-read before destructive writes to expose races, explicit
    "no start/stop semantics" errors, one listing per `inspect`, one ownership
    rule, and the executable-containment gate on the destructive paths;
  - `atomic_write_definition` (same-dir exclusive temp, owner-safe mode,
    `fsync`, no-replace vs overwrite authority, owned-temp-only cleanup,
    never creates privileged parents);
  - `inspect_host` facts vs `candidate_managers` policy (Linux systemd or
    cron; macOS launchd or cron; detection failure distinct from absent).
- Ownership for systemd/launchd uses exact executable plus critical args and
  reconciles optional config identity against those exact args. Consumer
  unit/plist bodies,
  usernames, paths, hardening text, health checks, and CLI messages stay
  consumer-owned. No consumer is migrated here.
- The crate's **public API surface is guarded locally**. Every public item is
  bound to an explicit type in `tests/public_api_surface.rs`, so a removed
  item or a changed signature is a compilation error in the crate's own test
  gate. The committed snapshot `tests/public-api-surface.txt` is the
  reviewable artifact: it is maintained by hand, and a change to it, or to
  the guard, is a deliberate, separately reviewed edit. The guard adds no
  dependency and changes no public API.
