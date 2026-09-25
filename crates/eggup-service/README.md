# eggup-service

Manager-neutral service-registration, ownership, state, and
lifecycle-snapshot model plus reusable Unix manager mechanics
(systemd, launchd, user crontab) and a native Windows SCM adapter.

- `ServiceId` / `ServiceSpec` identify desired registrations with exact
  executable, arguments, and config evidence.
- `Ownership::{Absent, Owned, Foreign, Unknown}` reuses the canonical Eggup
  vocabulary: destructive manager operations are authorized only for `Owned`.
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
- `windows-service` is a Windows-target dependency. `windows-args` is the small
  cross-platform parser dependency used for the same ownership tests on Unix.
- Bounded `TransitionResult` with conflict diagnostics; no automatic privilege
  elevation; no updater/release/network policy.
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
    with state confirmation, no `sudo`;
  - launchd: explicit `gui/<uid>` vs `system` targets (never guessed),
    structured `ProgramArguments` parsing (malformed yields `Unknown`),
    label-only never proves ownership, bounded `bootstrap`/`bootout`/
    `kickstart`/`stop`, foreign labels fail closed;
  - cron: managed `# BEGIN/# END <marker>` block (caller-owned marker +
    exact block), pure `cron_merge`/`cron_remove` preserving unrelated
    bytes, idempotent install, owned-only removal, no redundant `crontab -`
    writes, re-read before destructive writes to expose races, explicit
    "no start/stop semantics" errors;
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
