# eggup-service

Manager-neutral service-registration, ownership, state, and
lifecycle-snapshot model plus reusable Unix manager mechanics
(systemd, launchd, user crontab). Windows SCM remains a later milestone.

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
- Bounded `TransitionResult` with conflict diagnostics; no automatic privilege
  elevation; no updater/release/network policy.
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
- Ownership for systemd/launchd uses exact executable plus critical args
  (config paths belong in args; a separate `config` field with no observed
  counterpart is `Foreign` by the neutral rule). Consumer unit/plist bodies,
  usernames, paths, hardening text, health checks, and CLI messages stay
  consumer-owned. No consumer is migrated here.
