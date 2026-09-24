# Service Lifecycle — `eggup-service` Deep Dive

Source: `crates/eggup-service/{Cargo.toml, README.md, src/lib.rs, src/lifecycle_update.rs, src/windows_scm.rs}`.
Crate attributes: `#![forbid(unsafe_code)]`, `#![deny(missing_docs)]`.

## 1. Purpose and boundaries

- Manager-neutral service registration, ownership, lifecycle-state model plus
  concrete Unix mechanics (`systemd`, `launchd`, user `crontab`), host
  detection/policy helpers, a native Windows SCM adapter, and a Core-composing
  update orchestrator (`commit_with_lifecycle`).
- Explicit non-goals stated in `lib.rs` docs and `README.md`: no privilege
  escalation / `sudo` / UAC flow, no updater/release/network policy, no shell
  interpolation, no consumer migration (unit/plist bodies, usernames, paths,
  hardening text, health checks, CLI messages stay consumer-owned).
- Dependencies (`Cargo.toml`): `eggup-core` (path), `windows-args = "=0.2.0"`
  (cross-platform SCM command parser used in tests on Unix), Windows-only
  `windows-service = "=0.8.1"`.

## 2. Neutral model (`src/lib.rs` §§1–602)

### 2.1 `ServiceId` / `ServiceSpec`

- `ServiceId(String)`: opaque validated identity (unit name, launchd label, SCM
  name). `new` rejects empty, `len > 256`, control chars. `as_str()`,
  `Display`.
- `ServiceSpec { id, executable: PathBuf, args: Vec<String>, config: Option<PathBuf> }`:
  `new` requires absolute non-empty `executable`, args with no control chars,
  absolute `config` when present. Accessors `id()`, `executable()`, `args()`,
  `config()`.
- Identity rule (docs + `RegistrationSnapshot::ownership`): exact executable +
  critical args + optional config path. Same name / different executable is
  `Foreign`; same executable / different critical args is `Foreign` (documented);
  malformed is `Unknown`.

### 2.2 Ownership

```rust
pub enum Ownership { Absent, Owned, Foreign, Unknown }
```

Shared with `eggup-core` vocabulary. `Absent` = no record; `Owned` = exact
match, mutable; `Foreign` = another deployment, must not mutate; `Unknown` =
  unprovable, deny mutation.

### 2.3 `LifecycleState` / `HealthState`

Both `#[non_exhaustive]`:

- `LifecycleState::{Stopped, Running, Transitioning, Unknown}` — manager state.
- `HealthState::{Healthy, Degraded, Unknown}` — application health, distinct.
  Tests assert they can diverge (`Running` + `Degraded`).

### 2.4 Snapshots

- `RegistrationSnapshot { present: bool, executable: Option<PathBuf>, args: Vec<String>, config: Option<PathBuf>, malformed: bool }`
  + `absent()` + `ownership(&self, spec)`:
  `!present → Absent`; `malformed` or `executable: None → Unknown`;
  exe/args/config mismatch → `Foreign`; else `Owned`.
- `LifecycleSnapshot { id, ownership, state, health, was_registered: bool, was_running: bool }`
  + `restore_running() -> was_running`. Preserves pre-update intent so
  orchestration can restore “was running” vs “registered but stopped”.
  Stopped stays stopped by default.
- `RestoreIntent::{Preserve, EnsureRunning, EnsureStopped}`. `Preserve`
  returns to observed running/stopped; the other two are explicit caller policy.

### 2.5 Operations, results, errors

- `TransitionResult { operation: ServiceOperation, completed: bool, detail: String }`
  — bounded human-readable detail, never secrets; `completed()` accessor.
- `ServiceOperation::{Inspect, Install, Start, Stop, Restart, Uninstall}`
  (`#[non_exhaustive]`). `Inspect` always allowed; `Install` allowed for
  `Absent` (create) or `Owned` (refresh); the rest require `Owned`.
- `ServiceError::{InvalidInput(String), OwnershipDenied{id: String, ownership}, Conflict(String), Manager(String)}`
  (`#[non_exhaustive]`). Constructors `invalid/bounded/conflict/manager`
  truncate to 512 bytes; `denied(id, ownership)`; `Display` prefixes
  `invalid service input / ownership denies mutation / service conflict /
  service manager failed`; implements `std::error::Error`.

### 2.6 `HealthProbe`, `ServiceManager`, `require_owned`, `TestDoubleManager`

- `trait HealthProbe: Debug { fn check(&self, spec) -> HealthState; }` —
  read-only, never mutates manager state. `NoProbe` always returns `Unknown`.
- `trait ServiceManager { inspect(&self); install/start/stop/restart/uninstall(&mut self, …, timeout: Duration); }`
  Platform adapters implement it; `TestDoubleManager` implements it for tests.
- `require_owned(id, ownership)`: `Owned → Ok`, else `OwnershipDenied`.
  Shared guard for all destructive ops.
- `TestDoubleManager`: `HashMap<String, TestRegistration{spec, state, malformed}>`,
  `new/put/put_malformed/snapshot_for`. Semantics: `install` creates `Absent→Stopped`
  or refreshes `Owned→Stopped`, denies `Foreign/Unknown`; `start/stop`
  idempotent (`already running/stopped`, `completed: true`); `restart` =
  `stop` then `start`; `uninstall` removes entry. All destructive paths call
  `require_owned`.

## 3. Shared Unix mechanics (`src/lib.rs` M002 block)

### 3.1 Bounds

- `MAX_COMMAND_OUTPUT_BYTES = 256 KiB` per stream, `MAX_CRONTAB_BYTES = 1 MiB`,
  `MAX_DEFINITION_BYTES = 1 MiB`, `MAX_TRANSITION_TIMEOUT = 300 s`.

### 3.2 `CommandExecutor` / `SystemExecutor` / `FakeExecutor`

- `CommandOutput { status: Option<i32>, stdout/stderr: Vec<u8> }` +
  `stdout_text()/stderr_text()` (lossy UTF-8).
- `trait CommandExecutor: Debug { fn run(&self, argv: &[String], stdin_data: Option<&[u8]>, timeout: Duration) -> Result<CommandOutput, ServiceError>; }`
  Contract: literal argv, no shell; `stdin None` = null, `Some` = bounded input
  (only `crontab -`); missing binary / timeout / overflow = `Err(Manager)`;
  nonzero exit = `Ok` with status for caller classification.
- `SystemExecutor { max_output_bytes }`: `new/default` (256 KiB),
  `with_max_output` allows `1..=16 MiB`. `run` validates: non-empty argv, no
  NUL, `0 < timeout <= MAX+60 s`, stdin `<= MAX_CRONTAB_BYTES`; resolves program
  via `resolve_manager_program`; `env_clear()` + `filtered_manager_environment`;
  null/piped stdin, piped stdout/stderr; bounded stdin write (fail closed);
  two reader threads with 8 KiB chunks (`read_bounded_generic`); `try_wait`
  loop with `min(5 ms)` sleep; kill+reap on deadline; overflow → `Manager`
  without assuming state.
- `resolve_manager_program`: absolute paths validated; bare names allowlisted:
  `systemctl → /usr/bin|/bin`, `launchctl → /bin|/usr/bin`,
  `crontab → /usr/bin|/bin`; anything else `InvalidInput`. Missing trusted
  binary → `Manager("trusted manager binary missing: …")`. Ambient `PATH`
  ignored (covered by test with a shadow `systemctl` in temp dir).
- `validate_executable_path`: absolute, regular file, Unix exec bit.
- `filtered_manager_environment`: only user-scoped systemd
  (`systemctl … --user`) retains `DBUS_SESSION_BUS_ADDRESS`,
  `XDG_RUNTIME_DIR`, `SYSTEMD_BUS_ADDRESS`; everything else (including system
  scope) gets an empty env.
- `FakeExecutor`: `Mutex<VecDeque<FakeExpectation>>` + `Mutex<Vec<FakeCall{argv, stdin}>>`;
  `expect/expect_error/calls/call_count/is_exhausted`; `run` records the call,
  pops the queue, requires exact argv equality, else `Manager`. Also
  implemented for `&FakeExecutor`.

### 3.3 `atomic_write_definition`

`pub fn atomic_write_definition(path, bytes, allow_overwrite) -> Result<(), ServiceError>`:

- Rejects empty / `> MAX_DEFINITION_BYTES` / NUL bytes.
- Parent must exist as a real directory (symlink parent rejected); never
  creates missing parents or changes parent permissions.
- `dest` exists + `!allow_overwrite` → `Conflict` (no-replace path uses
  `hard_link(tmp, dest)` so `AlreadyExists → Conflict`).
- Mode: preserve existing `& 0o777` on overwrite, else `0644`; temp created
  exclusive (`create_new`, `0o600`) in the same directory as
  `.eggup-def-<pid>-<nanos>[-<attempt>].tmp` with 32-collision retry.
- `write_all + flush + sync_all`, then `chmod(final_mode)`, then promote:
  `allow_overwrite` → `rename` (Windows fallback: `remove_file` + `rename`
  after ownership authorization); else `hard_link` + `remove_file(tmp)`.
  Best-effort parent `fsync`. `Guard` removes only the owned temp on failure.

### 3.4 `OperationDeadline` and small helpers

- `OperationDeadline(Instant)`: `new` rejects zero / `> 300 s`; `remaining()`,
  `command_timeout()` (`Manager("transition deadline exhausted…")` when
  expired), `expired()`. One monotonic deadline is shared per transition by
  ownership queries, manager commands, and polling. `bounded_timeout` clamps to
  `MAX_TRANSITION_TIMEOUT`.
- `reconcile_config_identity(observed, spec)`: no spec config → as-is;
  non-UTF-8 config → malformed; counts exact config-string occurrences in
  spec vs observed args: `0` observed → `Foreign` (returned unchanged);
  `>1` either side → malformed (`Unknown`); `1==1` → sets
  `observed.config`. Used by systemd/launchd/SCM.
- `truncate` (256 chars for stderr excerpts), `permission_hint` (appends
  `"(permission denied; re-run with appropriate user/system scope; no automatic
  elevation)"` when a `Manager` message contains permission/denied/access; never
  mentions `sudo`), `ensure_mutation_success` (nonzero → `Manager`).

## 4. Adapters

### 4.1 systemd — `SystemdManager<E: CommandExecutor>`

- `SystemdScope::{System(--system), User(--user)}` — explicit, never guessed
  from EUID.
- `SystemdInstall { unit_name, scope, unit_path: PathBuf, definition: Vec<u8>, enable: bool, reload: bool, transition_timeout }`:
  `unit_name` must be bare `*.service` (rejects `/ \ space @`, controls,
  `>256`); `unit_path` absolute; definition non-empty, `<= 1 MiB`, no NUL;
  timeout in range.
- Observation: `systemctl <scope> show <unit> -p LoadState,ActiveState,ExecStart`.
  `parse_exec_start` accepts exactly one `{ path=… ; argv[]=… ; … }` group,
  rejects `% $ " ' \`, requires `argv[0] == path` and absolute exe.
  `systemd_ownership`: `not-found/""/missing LoadState → Absent/Stopped`;
  non-`loaded` → `Unknown/Unknown`; `active/inactive/activating|deactivating|reloading`
  mapped, else `Unknown`; `!= 1` ExecStart or parse failure → `Unknown`, then
  `reconcile_config_identity`.
- Transitions share one `OperationDeadline`: `run_unit(start|stop|restart)` +
  `poll_active` (`is-active`: `active+0` vs `inactive+3`, 100 ms sleep).
  `start/stop` short-circuit already-in-state; `restart` issues
  `systemctl restart` then polls active. Nonzero → `Manager` via
  `permission_hint`.
- `install`: `Absent` → `atomic_write(false)` + `post_write`;
  `Owned` → re-inspect then `atomic_write(true)`; `Foreign/Unknown` denied.
  `post_write` runs `daemon-reload` and/or `enable` only when the explicit
  flags are set, each requiring exit 0.
- `uninstall`: double `require_owned` check; best-effort `disable` (failure →
  `completed: false`, file kept); `remove_file(unit_path)`; optional
  `daemon-reload` (failure → `completed: false` after delete).

### 4.2 launchd — `LaunchdManager<E: CommandExecutor>`

- `LaunchdDomain::{UserAgent, SystemDaemon}`; `LaunchdInstall { label, domain, target, plist_path, definition, bootstrap_on_install, transition_timeout }`:
  label `[A-Za-z0-9._-]`, `<= 256`, no `/ \ space @`; target `<= 128`;
  `UserAgent ⇒ gui/<uid>`, `SystemDaemon ⇒ system` (mismatches rejected).
- File gates ownership: `read_plist_observation` (`NotFound → Absent`, else
  `parse_launchd_plist`). `parse_launchd_plist`: `> 1 MiB` / non-UTF-8 /
  `!= 1 ProgramArguments` key / intervening `<key>` / non-`<string>` tags /
  `<` in values / unsupported entities (only `&amp; &lt; &gt; &quot; &apos;`) /
  empty / relative exe → malformed (`Unknown`). Label-only never `Owned`.
- Loaded state: `launchctl list <label>`; exit nonzero → `(false, Stopped)`;
  numeric `PID` → `Running`, else `Stopped`. Non-`Owned` files are never
  probed beyond reporting `Unknown` state.
- `install` mirrors systemd (`atomic_write` + optional `bootstrap`).
  `start_until`: already-running short-circuit; `bootstrap` when not loaded;
  `kickstart -k <target>/<label>`; `confirm_running` poll. `stop_until`:
  `launchctl stop <label>` + `confirm_stopped`. `restart`: incomplete stop
  returns without attempting start. `uninstall`: double ownership check,
  `bootout <target> <plist>` when loaded (failure → `completed: false`),
  then `remove_file`.

### 4.3 cron — `CronManager<E: CommandExecutor>` + pure helpers

- Managed `# BEGIN <marker>` / `# END <marker>` block. `validate_marker`
  (`<= 128`, no controls/newlines), `validate_block` (`<= 512 KiB`, no NUL).
- `find_blocks` (trimmed-end match; unterminated `BEGIN` → block-to-EOF ⇒
  `Unknown`); `cron_classify`: 0 blocks → `Absent`; 1 block with exact content
  (`desired` or `desired.trim_matches('\n')`) → `Owned`, else `Foreign` (same
  marker, different content = another deployment); `>1` → `Unknown`.
- `cron_merge` (idempotent, preserves unrelated bytes, appends with exactly
  one trailing newline; `Foreign → denied(cron:<marker>)`,
  `Unknown → Conflict`) and `cron_remove` (only exact `Owned` removed;
  `Absent` no-op; preserves trailing-newline convention).
- Adapter: `list_crontab` (`crontab -l`; `no crontab` → empty; oversize →
  `Manager`); `write_crontab` (`crontab -` with bounded stdin). `cron_ownership`
  additionally requires the live/desired block to mention the spec executable.
  `inspect` double-lists (listing failure → `Unknown` without mutating; state
  always `Stopped` when classifiable, `was_running: false`).
  `install/uninstall` reuse the pure merge/remove, enforce the neutral
  `Absent|Owned` gate, re-list before destructive write (`Conflict` on race),
  and write exactly once (idempotent install / already-absent uninstall do no
  `crontab -` call). `start/stop/restart` return `InvalidInput("cron has no …
  semantics; supervision is external")`.

### 4.4 Host facts vs selection policy

- `HostFacts { os, systemd_available, launchd_available, crontab_available }`.
- `inspect_host(executor)`: probes `systemctl --version` (Linux only, and
  requires `/run/systemd/system`), `launchctl version` (macOS only),
  `crontab -l` (all); `missing`-binary → `false`, other `Manager` → `Err`.
  Distinguishes detection failure (`Err`) from absent (`false`); never
  mutates or elevates.
- `candidate_managers(facts)`: macOS+launchd → `[Launchd(UserAgent), Cron?]`;
  Linux+systemd → `[Systemd(System), Cron?]`; else `[Cron]` or `[]`.
  Overridable; consumers may construct a specific adapter directly.

### 4.5 Windows SCM — `src/windows_scm.rs`, `WindowsScmManager`

- Bounds: `MAX_SCM_NAME_CHARS = 256` (UTF-16 count), `MAX_SCM_COMMAND_CHARS =
  8192`, `DEFAULT_SCM_TIMEOUT = 30 s`, poll `50 ms` capped at `500 ms`.
- `WindowsStartType::{Automatic, Manual, Disabled}`,
  `WindowsErrorControl::{Ignore, Normal, Severe, Critical}`,
  `WindowsServiceDependency::{Service(String), Group(String)}`.
- `WindowsScmInstall { service_id, display_name, start_type, error_control, account_name: Option<String>, dependencies: Option<Vec<…>>, transition_timeout }`:
  names UTF-16-bounded, no controls/NUL, service name no `/ \`; dependencies
  validated (no path-like, case-insensitive duplicate rejected; `None`
  preserves on refresh, `Some([])` clears); `with_transition_timeout`
  `0 < t <= 300 s`. Intentionally omits failure actions, descriptions, custom
  passwords, entrypoint config; refresh preserves service type, unspecified
  dependencies, account, load-order group/tag. Account is create-time only;
  `account_password` is always `None`. Creation never starts the service.
- `validate_spec`: spec id must equal install id; executable must be absolute
  Windows path (`X:\|/` or `\\server\share…`) and UTF-8; config UTF-8 when
  present; UTF-16-weighted command budget `<= 8192`.
- `ScmBackend` trait (`query/create/refresh/start/stop/delete`, `sleep`
  injectable); `ScmState` all seven SCM states + `Unknown`; `ScmRecord
  { command_line: Option<String> (None when non-UTF-8), state, wait_hint }`;
  `ScmLookup::{Absent, MarkedForDelete, Present}` (production 1060/1072
  mapping lives in the `#[cfg(windows)] WindowsBackend`).
- Ownership: `parse_service_command` via `windows-args::Args::parse_cmd`
  (rejects empty/overlong/controls/unbalanced quotes, requires absolute
  `argv[0]`, requires leading `"` when exe contains whitespace, requires
  unquoted first token to end `.exe`); failure or non-UTF-8 → `Unknown`; else
  `reconcile_config_identity`. Key name alone never authorizes mutation.
  `lifecycle_state`: `Stopped→Stopped`, `Running→Running`,
  `*Pending→Transitioning`, `Paused|Unknown→Unknown`.
- `WindowsBackend` (`cfg(windows)` only, `windows-service` wrapper): minimal
  access per op, `OWN_PROCESS` create, refresh carries forward queried
  `service_type`/`dependencies`, leaves account/group/tag unchanged,
  disappeared-before-op → `Conflict`; Win32 code 5 → bounded
  `“…denied by Windows SCM; run with an account authorized…”` with no
  elevation hint. No `sc.exe`, shell, ambient `PATH`, or auto-elevation.
- `WindowsScmManager` (`new` only on Windows; `with_backend` for tests):
  per-transition monotonic `OperationDeadline`; `wait_for_state` honors
  `wait_hint` (capped); `start/stop` state machines (pending states poll,
  conflicting states return `completed: false` without issuing a command);
  `restart` = confirmed stop phase then start; `install` create-vs-refresh
  with post-op re-observation; `uninstall` stops when needed, rechecks
  `Stopped`, deletes, then polls until `Absent` (`MarkedForDelete` stays
  `completed: false`). `inspect` health is always `Unknown`.

### 4.6 `commit_with_lifecycle` — `src/lifecycle_update.rs`

Composes an already-validated Core M007 transaction with owned-service
quiescence/restoration and a bounded caller check. Never installs/refreshes
the registration; artifact backup is Core’s rollback, not a service copy.

- `LifecycleUpdatePhase::{Inspect, Quiesce, Commit, RestoreNew, PostInstallCheck, QuiesceForRollback, RestoreOld, FinalInspect}`.
- `LifecycleFailure { phase, service_id, detail (no controls, ≤512 chars, no secrets), ownership?, state? }`.
- `LifecycleRestorationStatus::{Restored, Failed, NotAttemptedRecoveryRequired}`.
- `LifecycleUpdatePolicy { restore: RestoreIntent, post_commit_failure: PostCommitFailurePolicy, quiesce_timeout, post_commit_timeout, rollback_restore_timeout }`;
  `validate()` rejects zero / `> 300 s` before any mutation.
- `PostInstallCheckError` (bounded) + `trait PostInstallCheck: Debug { fn check(&self, spec, snapshot, remaining: Duration) -> Result<…>; }`
  (must honor `remaining`; panics are caught); `NoPostInstallCheck` is a no-op.
  Note: `HealthProbe` has no timeout contract and does not satisfy this bound.
- `LifecycleUpdateReceipt { before, final_snapshot?, transaction: TransactionReceipt (exact Core evidence), pre_commit_quiesced: bool, restoration, post_commit_failure?, rollback_quiesce_failure?, rollback_restore_failure?, observation_failure? }`
  + `artifact_disposition()`, `manual_artifact_recovery_required()`,
  `new_generation_retained()`, `service_state_restored()`.
- `LifecycleUpdateError::{InvalidPolicy{field}, Preflight{failure}, Quiesce{failure, restore_failure?}, CoreBeforeReceipt{core, restore_failure?}}`.
- Flow (`commit_with_lifecycle(manager, spec, transaction: ValidatedTransaction, ownership: CommitOwnership, policy, check)`):
  1. Validate policy; `safe_inspect` (panic-safe). Require `Owned` +
     `Running|Stopped`, else `Preflight` (covers Absent/Foreign/Unknown/
     Transitioning before any artifact commit).
  2. If `Running`, `stop(quiesce_timeout)` + re-inspect `Owned+Stopped`;
     error/incomplete/drift → `Quiesce` + best-effort restore to `Running`.
  3. `transaction.commit_with_post_commit(…)` with a shared
     `post_commit_timeout` deadline: `restore_success_state` maps
     `Preserve→before.state`, `EnsureRunning→Running`,
     `EnsureStopped→Stopped` via `transition_to` (no-op when already there;
     rejects `Transitioning/Unknown`; requires `Owned`; requires
     `completed` + re-inspection), re-inspects, then runs the caller check
     with the remaining budget. Failure is recorded as `post_commit_failure`
     and converted to `PostInstallCheckError` so Core applies
     `KeepInstalled|RollBack`; `RollBack` first attempts
     `quiesce_for_rollback` (stop the new generation).
  4. Core `Err` before receipt → restore pre-update state when quiesced →
     `CoreBeforeReceipt` (artifacts untouched on quiesce failure paths —
     tests assert `service.bin` absent).
  5. On receipt: `Committed` + `RestoreNew` failure → `Failed`;
     `RolledBack` → restore `before.state` (failure → `Failed`, kept separate
     from the Core receipt); `RecoveryRequired` → `NotAttemptedRecoveryRequired`
     (no automatic start against uncertain artifacts).
  6. Final `safe_inspect` is read-only and nonessential: failure yields
     `observation_failure` with `final_snapshot: None` without changing Core
     disposition.

## 5. Review checklist

- [ ] New destructive paths call `require_owned` (or re-observe and compare to
  `Owned`) and map `Foreign/Unknown` to denial, not overwrite.
- [ ] New parsers fail closed to `Unknown`: ambiguous counts, specifiers
  (`%`/`$`), quotes/escapes, non-UTF-8, unbalanced quotes, relative exes.
- [ ] `config` identity goes through `reconcile_config_identity` (0 → Foreign,
  >1 → Unknown, exactly-1 → Owned path).
- [ ] Manager commands stay literal argv via `CommandExecutor`; no shell, no
  ambient `PATH`, no new env passthrough, no `sudo`/elevation text.
- [ ] Timeouts: `0 < t <= 300 s`, one `OperationDeadline` per transition,
  expiry → `completed: false` or `Manager` timeout, never assumed state.
- [ ] Outputs truncated (`truncate`/512-byte error bound/256 KiB stream bound);
  no secrets in `TransitionResult.detail` or `LifecycleFailure.detail`.
- [ ] Definition writes use `atomic_write_definition` with the correct
  `allow_overwrite` (`false` create, `true` owned refresh); no parent creation.
- [ ] Install paths re-inspect before overwrite; uninstall paths re-inspect
  before delete and confirm absence where required (SCM `Absent`, cron race
  check, systemd disable/reload semantics).
- [ ] Cron changes preserve unrelated bytes, stay idempotent, write at most
  once, and return explicit no-semantics errors for start/stop/restart.
- [ ] `commit_with_lifecycle` callers pass an already-validated transaction,
  an `Owned`+`Running|Stopped` service, a validated policy, and a
  deadline-honoring `PostInstallCheck`; `RecoveryRequired` routes to manual
  recovery, never auto-start.
