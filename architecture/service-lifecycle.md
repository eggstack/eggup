# `eggup-service` — manager-neutral service lifecycle

Deep dive for the `eggup-service` crate: the neutral registration/ownership
model, bounded manager command execution, the Unix (systemd / launchd / cron) and
Windows SCM adapters, lifecycle composition over `eggup-core`, and the M006
runtime-disposition barrier.

This document is a review aid. Where it and the source disagree, **the source is
the behavior**; drift is called out in [Known doc/code drift](#known-doccode-drift).

- Index and terminology: [overview.md](overview.md)
- Transaction contract it composes with: [core-transaction.md](core-transaction.md) and
  [transaction.md](../crates/eggup-core/docs/transaction.md)

## Source layout

| File | Size | Role |
|---|---|---|
| [`src/lib.rs`](../crates/eggup-service/src/lib.rs) | ~4.5k lines | Neutral model, `CommandExecutor` / `SystemExecutor` / `FakeExecutor`, `atomic_write_definition`, systemd + launchd + cron adapters, host detection |
| [`src/disposition.rs`](../crates/eggup-service/src/disposition.rs) | ~2.5k lines | M006 runtime-authority disposition, pure planners, `commit_with_disposition` |
| [`src/lifecycle_update.rs`](../crates/eggup-service/src/lifecycle_update.rs) | ~1.5k lines | `commit_with_lifecycle`, policy/receipt/phase types |
| [`src/windows_scm.rs`](../crates/eggup-service/src/windows_scm.rs) | ~1.6k lines | Native Windows SCM adapter behind an injectable backend |

The three submodules are private (`lib.rs:11-13`); everything is re-exported
flat from the crate root, so there is one public namespace.

## Purpose and ownership boundary

The crate owns the question **"what is registered, who owns it, and may I
change its runtime state?"** It turns a caller-described desired registration
into an observation (`Ownership` + `LifecycleState`), authorizes destructive
operations on that observation, and drives a bounded conversation with a
platform service manager.

What it never decides:

| Never decided here | Where it lives |
|---|---|
| Release selection, ordering, or which version to install | caller, before Eggup |
| Transport choice, download, or `AcquisitionTransport` composition | [`acquisition.md`](acquisition.md) and its adapters |
| Artifact authenticity or signature trust | **nobody in this workspace**, by design ([ADR-0003](../plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md)) |
| Destination / install-root selection | caller; [eggpack-adapter.md](eggpack-adapter.md) exposes bindings, the consumer binds them |
| Unit/plist/command-line **content** | caller. Adapters receive exact bytes (`SystemdInstall::definition`, `LaunchdInstall::definition`) or the spec |
| Application health semantics | caller, via [`PostInstallCheck`](#lifecycle-composition-commit_with_lifecycle) / `HealthProbe` |
| Privilege acquisition | never. Access denial returns a bounded hint, never an elevation flow (`lib.rs:1541`) |

The crate also owns the **absence of shell interpolation**: every manager call
is an argv vector, and manager binaries are resolved from a fixed allowlist of
absolute paths.

## Position in the workspace

```text
eggup-core  (sha2 only)
     ^
     |
eggup-service   ->  windows-args (=0.2.0)
                  ->  windows-service (=0.8.1)   [cfg(windows) only]
```

- `Cargo.toml`: `eggup-core = { version = "0.1.0", path = "../eggup-core" }`,
  `windows-args = "=0.2.0"`, and a `cfg(windows)`-gated `windows-service = "=0.8.1"`.
- `windows-service` is the safe wrapper around the SCM handle; it is a
  **Windows-target dependency**, so the Linux/macOS build never links it.
- `windows-args` is cross-platform and is used for the SCM command-line parser,
  so the ownership tests in `windows_scm.rs` run on every host.
- **The crate is published but lags the workspace.** It is on crates.io at
  `0.1.0`, `0.1.1`, and `0.1.2`; the `0.1.2` service publication came from
  `7fb84bc`, after the shared `v0.1.2` tag, which remains the core/archive source
  tag. The workspace now carries an unpublished `0.1.3` correction for owned
  failed-systemd service quiescence; M011 owns its separate release qualification.
  `eggup-core`, `eggup-archive`, `eggup-acquisition`, `eggup-eggfetch`,
  `eggup-eggpack`, and `eggup-curl` are also published at `0.1.2` or later.
  `eggup-transport-footprint` is `publish = false`
  ([registry.md](../plans/registry.md)).
- Dependency order position: 5th deepest in
  [overview.md's dependency graph](overview.md#dependency-graph). It is a leaf
  consumer of `eggup-core` and depends on nothing else in the workspace.
- `#![forbid(unsafe_code)]` and `#![deny(missing_docs)]` are set on the crate
  root (`lib.rs:1-2`) and repeated in `disposition.rs:1-2`. The other two
  submodules declare no inner attributes; their public items are covered by the
  root attributes because the modules are private and re-exported.

## Public surface

All items below are re-exported from the crate root (`lib.rs:14-27`).

### `lib.rs` — neutral model, execution, Unix mechanics

| Item | Kind | Role | Ref |
|---|---|---|---|
| `ServiceId` | struct | Validated opaque identity (unit name, launchd label, SCM name) | `lib.rs:37` |
| `ServiceId::new` / `as_str` | fn | Rejects empty / >256 bytes / control chars | `lib.rs:41`, `lib.rs:52` |
| `ServiceSpec` | struct | Desired registration: id + executable + critical args + optional config | `lib.rs:70` |
| `ServiceSpec::new` | fn | Requires absolute executable and config, control-free args | `lib.rs:79` |
| `ServiceSpec::{id,executable,args,config}` | accessors | Identity components participating in ownership | `lib.rs:110-127` |
| `Ownership` | enum | `Absent \| Owned \| Foreign \| Unknown` | `lib.rs:132` |
| `LifecycleState` | enum (`#[non_exhaustive]`) | `Stopped \| Running \| Transitioning \| Unknown` | `lib.rs:146` |
| `HealthState` | enum (`#[non_exhaustive]`) | `Healthy \| Degraded \| Unknown` | `lib.rs:160` |
| `RegistrationSnapshot` | struct | Manager-neutral registration observation | `lib.rs:171` |
| `RegistrationSnapshot::absent` | fn | Canonical absent observation | `lib.rs:186` |
| `RegistrationSnapshot::ownership` | fn | Classifies the observation against a spec | `lib.rs:197` |
| `LifecycleSnapshot` | struct | Observation + `was_registered` / `was_running` intent | `lib.rs:216` |
| `LifecycleSnapshot::restore_running` | fn | Restoration intent derived from the observation | `lib.rs:233` |
| `RestoreIntent` | enum | `Preserve \| EnsureRunning \| EnsureStopped` | `lib.rs:240` |
| `TransitionResult` | struct | Bounded outcome: `operation`, `completed`, `detail` | `lib.rs:251` |
| `ServiceOperation` | enum (`#[non_exhaustive]`) | `Inspect \| Install \| Start \| Stop \| Restart \| Uninstall` | `lib.rs:270` |
| `ServiceError` | enum (`#[non_exhaustive]`) | `InvalidInput \| OwnershipDenied \| Conflict \| Manager` | `lib.rs:289` |
| `ServiceError::denied` | fn | Canonical `OwnershipDenied` construction | `lib.rs:323` |
| `HealthProbe` | trait | Caller-supplied, read-only health check | `lib.rs:347` |
| `NoProbe` | struct | Always `HealthState::Unknown` | `lib.rs:354` |
| `ServiceManager` | trait | The six manager operations | `lib.rs:364` |
| `require_owned` | fn | The single destructive-operation gate | `lib.rs:397` |
| `TestDoubleManager` | struct | Deterministic in-memory manager | `lib.rs:408` |
| `MAX_COMMAND_OUTPUT_BYTES` | const | `256 * 1024` per stdout/stderr stream | `lib.rs:608` |
| `MAX_CRONTAB_BYTES` | const | `1024 * 1024` | `lib.rs:610` |
| `MAX_DEFINITION_BYTES` | const | `1024 * 1024` | `lib.rs:612` |
| `MAX_TRANSITION_TIMEOUT` | const | `Duration::from_secs(300)` | `lib.rs:614` |
| `CommandOutput` | struct | `status: Option<i32>`, bounded `stdout` / `stderr` | `lib.rs:618` |
| `CommandExecutor` | trait | Injectable runner: argv in, bounded output out | `lib.rs:645` |
| `SystemExecutor` | struct | Production runner; cleared env, allowlisted paths | `lib.rs:668` |
| `atomic_write_definition` | fn | Same-dir temp + fsync + no-replace/rename promotion | `lib.rs:1087` |
| `FakeExecutor` / `FakeCall` | struct | Scripted argv expectations, call recording | `lib.rs:992`, `lib.rs:983` |
| `SystemdScope` / `SystemdInstall` / `SystemdManager` | enum/struct/struct | systemd adapter, explicit scope, caller-owned unit bytes | `lib.rs:1237`, `1255`, `1327` |
| `LaunchdDomain` / `LaunchdInstall` / `LaunchdManager` | enum/struct/struct | launchd adapter, explicit `gui/<uid>` vs `system` | `lib.rs:1925`, `1934`, `2025` |
| `CronOwnership` | enum | `Absent \| Owned \| Foreign \| Unknown` for a marker block | `lib.rs:2523` |
| `cron_classify` / `cron_merge` / `cron_remove` | fn | Pure block classification and edit | `lib.rs:2592`, `2620`, `2665` |
| `CronManager` | struct | Managed `# BEGIN/# END` block in the user crontab | `lib.rs:2717` |
| `CandidateManager` / `HostFacts` / `inspect_host` / `candidate_managers` | enum/struct/fn/fn | Detection facts separated from selection policy | `lib.rs:2992`, `3003`, `3018`, `3072` |

### `lifecycle_update.rs` — composition over Core

| Item | Kind | Role | Ref |
|---|---|---|---|
| `LifecycleUpdatePhase` | enum (`#[non_exhaustive]`) | `Inspect \| Quiesce \| Commit \| RestoreNew \| PostInstallCheck \| QuiesceForRollback \| RestoreOld \| FinalInspect` | `lifecycle_update.rs:17` |
| `LifecycleFailure` | struct | Bounded service-side evidence: phase, id, detail, optional ownership/state | `:38` |
| `LifecycleRestorationStatus` | enum | `Restored \| Failed \| NotAttemptedRecoveryRequired` | `:70` |
| `LifecycleUpdatePolicy` | struct | `restore`, `post_commit_failure`, three timeouts | `:81` |
| `LifecycleUpdatePolicy::validate` | fn | Rejects zero / >`MAX_TRANSITION_TIMEOUT` before any mutation | `:96` |
| `PostInstallCheckError` | struct | Bounded, control-free failure detail | `:112` |
| `PostInstallCheck` | trait | Caller check receiving the remaining shared budget | `:134` |
| `NoPostInstallCheck` | struct | No-op check | `:144` |
| `LifecycleUpdateReceipt` | struct | `before`, `final_snapshot`, Core receipt, restoration, four failure slots | `:159` |
| `LifecycleUpdateError` | enum (`#[non_exhaustive]`) | `InvalidPolicy \| Preflight \| Quiesce \| CoreBeforeReceipt` | `:205` |
| `commit_with_lifecycle` | fn | Quiesce → Core commit → restore → post-install check | `:254` |

### `disposition.rs` — M006 runtime authority

| Item | Kind | Role | Ref |
|---|---|---|---|
| `UpdateRuntimeDisposition` | enum | `ManagedRunning \| ManagedStopped \| DirectRunning \| Stopped \| ForeignPreserved` | `disposition.rs:31` |
| `KnownConfig` | enum | `Absent \| Present(PathBuf) \| Unknown` for registered config identity | `:46` |
| `DirectState` | enum | `Running \| Stopped \| Unknown` | `:57` |
| `DirectObservation` | struct | `state`, `exact_ownership_proven`, `executable`, `config` | `:68` |
| `UnixPlannerInput` | struct | Manager ownership/state + registered/selected config + direct observation | `:81` |
| `WindowsServiceState` | enum | `Running \| StartPending \| Stopped \| StopPending \| Unknown` | `:98` |
| `WindowsPlannerInput` | struct | `installed`, `executable_known`, `ownership`, `state` | `:113` |
| `plan_unix` | fn | Pure Unix decision matrix | `:125` |
| `plan_windows` | fn | Pure Windows SCM decision matrix | `:224` |
| `DirectRuntimeControl` | trait | Caller seam: `inspect`, `stop`, `start` | `:256` |
| `DispositionBaseline` | struct | Authority/state observed after preparation | `:267` |
| `commit_with_disposition` | fn | Barrier-enforcing orchestration over a `ValidatedTransaction` | `:321` |

### `windows_scm.rs` — native SCM adapter

| Item | Kind | Role | Ref |
|---|---|---|---|
| `WindowsStartType` | enum | Caller-owned start type | `windows_scm.rs:21` |
| `WindowsErrorControl` | enum | Caller-owned error control | `:32` |
| `WindowsServiceDependency` | enum | `Service(name) \| Group(name)` | `:45` |
| `WindowsScmInstall` | struct | Service id, display name, start/error control, account, dependencies, timeout | `:58` |
| `WindowsScmInstall::{new,with_dependencies,with_transition_timeout}` | fn | Validating builders | `:70`, `:97`, `:107` |
| `WindowsScmManager` | struct | `ServiceManager` over a boxed `ScmBackend` | `:232` |
| `WindowsScmManager::new` | fn | `#[cfg(windows)]`; uses the local SCM | `:240` |

Internal bounds: `MAX_SCM_NAME_CHARS = 256`,
`MAX_SCM_COMMAND_CHARS = 8192`, `DEFAULT_SCM_TIMEOUT = 30s`,
`DEFAULT_POLL_INTERVAL = 50ms`, `MAX_POLL_INTERVAL = 500ms`
(`windows_scm.rs:13-17`).

## The neutral model

### `ServiceId` and `ServiceSpec`

`ServiceId` is a validated opaque string. Validation is **byte-length** based:
`value.is_empty() || value.len() > 256 || value.chars().any(char::is_control)`
(`lib.rs:43`). A multi-byte UTF-8 identifier is therefore limited to fewer
than 256 characters even though the constant reads like a character budget.

`ServiceSpec::new` is the identity constructor and applies four rules
(`lib.rs:85-100`):

| Rule | Error |
|---|---|
| `executable.is_absolute()` | `InvalidInput("executable must be absolute")` |
| executable not empty | `InvalidInput("executable must name a file")` |
| every arg free of control characters | `InvalidInput("argument has control characters")` |
| `config`, if present, absolute | `InvalidInput("config path must be absolute")` |

Identity is the **triple** (executable, args, config). That is what
`RegistrationSnapshot::ownership` compares, and it is why "same name, different
executable" and "same executable, different critical args" are both `Foreign`
rather than a managed upgrade.

### `RegistrationSnapshot::ownership` — the classification

```
present == false                       -> Absent
present && malformed                   -> Unknown
executable == None                     -> Unknown
executable != spec.executable          -> Foreign
args != spec.args                      -> Foreign
config != spec.config()                -> Foreign
otherwise                              -> Owned
```

`lib.rs:197-211`. Note the asymmetry that matters for review: **`Owned` is
reachable only from an exact three-way match against a present, non-malformed,
parseable record.** There is no "probably ours" path.

### `LifecycleState` vs `HealthState`

These are separate `#[non_exhaustive]` types and the crate never converts
between them. `LifecycleState` answers *what does the manager report*;
`HealthState` answers *does the application work*. A process can be `Running`
and `Degraded`; a `Stopped` registration's health is `Unknown`, not `Healthy`.

`HealthState` only ever arrives from the caller: `HealthProbe::check`
(`lib.rs:349`), `NoProbe` returning `Unknown` (`lib.rs:357`), or the caller's own
`PostInstallCheck`. Every manager adapter in this crate hard-codes
`health: HealthState::Unknown` into the snapshot it returns (e.g. `lib.rs:1685`)
because the manager has no opinion about application health. Collapsing the two
would let a broken-but-running service pass a "service is up" post-install
assertion, which is exactly the failure mode
[overview.md](overview.md#cross-cutting-invariants) calls out as a cross-cutting
invariant.

### Snapshot and result types

| Type | Fields | Semantics |
|---|---|---|
| `LifecycleSnapshot` (`lib.rs:216`) | `id`, `ownership`, `state`, `health`, `was_registered`, `was_running` | Point-in-time observation. `was_registered` / `was_running` are the *only* record of pre-update intent |
| `LifecycleSnapshot::restore_running` (`lib.rs:233`) | — | Returns `was_running`. Distinguishes "was running" from "registered but stopped" |
| `RestoreIntent` (`lib.rs:240`) | — | Caller policy for the **new** generation only: `Preserve` / `EnsureRunning` / `EnsureStopped` |
| `TransitionResult` (`lib.rs:251`) | `operation`, `completed`, `detail` | `completed == false` is a *bounded outcome*, not an `Err`. Callers must branch on it |
| `LifecycleRestorationStatus` (`lifecycle_update.rs:70`) | — | Terminal lifecycle verdict, independent of Core's artifact disposition |

`RestoreIntent` deliberately does not govern the rollback path: after a
successful artifact rollback, orchestration restores `before.state`
regardless of intent (`lifecycle_update.rs:407-418`,
`disposition.rs:748-764`). A caller asking for `EnsureRunning` on an update that
rolled back gets the pre-update state, not a new start.

### `ServiceError`

`InvalidInput`, `OwnershipDenied { id, ownership }`, `Conflict`, `Manager`
(`lib.rs:289`). The crate's own constructors cap detail at **512 bytes on a UTF-8
character boundary** (`ServiceError::invalid` / `bounded` at `lib.rs:306-320`;
`truncate_utf8_bytes` at `lib.rs:1526`). Manager stderr excerpts are capped at
**256 bytes** (`truncate`, `lib.rs:1537`) before being folded into a
`Manager` error by `ensure_mutation_success` (`lib.rs:1558-1568`).
`bound_detail` in the disposition path additionally strips control characters
before truncating (`disposition.rs:291-294`).

## Ownership and authorization

### How `Ownership` is determined

Ownership is never stored on the spec and never carried across calls. It is
computed per observation:

1. The adapter reads the manager (systemd `show`, launchd plist parse, SCM
   query, or the in-memory test double).
2. The adapter builds a `RegistrationSnapshot`, or marks it `malformed`.
3. `RegistrationSnapshot::ownership(spec)` classifies it.

Adapter-specific ownership rules:

| Adapter | Rule | Ref |
|---|---|---|
| systemd | `LoadState` empty / `not-found` → `Absent`; `loaded` → parse; anything else → `Unknown`. Exactly one `ExecStart` required, else `Unknown` | `lib.rs:1639-1655` |
| systemd | `ExecStart` with specifiers (`%`, `$`), embedded quotes, relative path, or a non-unique brace group → `Unknown` | `lib.rs:1581`, `1603`, `1608`, `1616` |
| systemd | Non-zero `show` exit → `Unknown` / `Unknown` state, never `Absent` | `lib.rs:1676-1680` |
| systemd / launchd | Optional config identity must appear as **exactly one** occurrence in canonical argv; zero → left unset (→ `Foreign`); more than one on either side → `malformed` → `Unknown` | `lib.rs:1491-1524` |
| launchd | Malformed plist → `malformed` → `Unknown`; a matching **label alone never proves ownership** | `lib.rs:2135`, `:2209`, test `launchd_no_label_only_ownership` (`lib.rs:4062`) |
| cron | Ownership is over the managed block: exact desired content → `Owned`; same marker, different content → `Foreign`; multiple or unterminated markers → `Unknown` | `lib.rs:2592-2613` |
| Windows SCM | Ownership is over the exact parsed command line, not the service key name. `MarkedForDelete` → `Unknown` | `windows_scm.rs:317-321` |
| `TestDoubleManager` | Same `RegistrationSnapshot::ownership` classification over injected records | `lib.rs:464-481` |

### Why `Owned` is never inferred

Three separate mechanisms make `Owned` unforgeable from inside the crate:

1. **Classification is exact** — only a full identity match yields `Owned`
   (`lib.rs:209`).
2. **Malformed is contagious upward** — an unparseable registration becomes
   `Unknown`, never `Absent` and never `Owned`. A manager that errors is
   `Unknown`, not "nothing there" (`lib.rs:1676-1680`).
3. **The mutation gate is a single function** — `require_owned`
   (`lib.rs:397-404`) accepts `Owned` and denies `Absent | Foreign | Unknown`
   with `ServiceError::denied`.

Note the third denial case: `Absent` is denied by `require_owned` too. Creating
a registration is a separate, explicitly-caller-authorized act
(`ServiceOperation::Install`, `lib.rs:273`), and every adapter implements it as
a distinct branch from the `Owned` refresh branch.

### Ownership × operation

| Operation | `Absent` | `Owned` | `Foreign` | `Unknown` |
|---|---|---|---|---|
| `inspect` | Allowed (returns `Absent`) | Allowed | Allowed (returns `Foreign`) | Allowed (returns `Unknown`) |
| `install` (create) | Allowed | — | — | — |
| `install` (refresh) | — | Allowed, after re-inspection | Denied | Denied |
| `start` / `stop` / `restart` / `uninstall` | Denied | Allowed | Denied | Denied |
| `commit_with_lifecycle` preflight | Preflight failure | Allowed if `Running` or `Stopped` | Preflight failure | Preflight failure |
| `commit_with_disposition` (`ManagedRunning` / `ManagedStopped`) | Preflight failure | Allowed | Preflight failure | Preflight failure |
| `commit_with_disposition` (`ForeignPreserved`) | Plan-dependent | Preflight failure | **Zero manager mutation** | Preflight failure |

`TestDoubleManager` implements the table directly (`lib.rs:491-600`): `install`
branches `Absent` → create, `Owned` → refresh, `Foreign | Unknown` →
`ServiceError::denied`; `start` / `stop` / `restart` / `uninstall` all call
`require_owned` first. The systemd adapter adds one thing the double does not: a
**second `inspect` immediately before an overwrite** write, so an external
change between the classification and the write denies the refresh
(`lib.rs:1703-1708`).

## The `ServiceManager` trait

`lib.rs:364-394`. Six methods, four of which are mutating and gated on
`Owned`:

| Method | Signature | Contract |
|---|---|---|
| `inspect` | `(&self, ServiceSpec) -> Result<LifecycleSnapshot, ServiceError>` | Read-only. Must not mutate. Must report `Absent` only on positive evidence of absence |
| `install` | `(&mut self, ServiceSpec) -> Result<TransitionResult, ServiceError>` | Create for `Absent`, refresh for `Owned`. Deny `Foreign` / `Unknown` |
| `start` / `stop` / `restart` | `(&mut self, ServiceSpec, Duration) -> Result<TransitionResult, ServiceError>` | Mutate only `Owned`. Honor the deadline. Return `completed: false` rather than an unbounded wait |
| `uninstall` | `(&mut self, ServiceSpec) -> Result<TransitionResult, ServiceError>` | Remove an `Owned` registration only |

What an implementor must honor, in the order the crate itself checks them:

1. **Ownership gate before any mutation.** Call `require_owned` (or the local
   equivalent) before touching the manager.
2. **Deadline handling.** Reject a zero timeout. The reference implementation is
   `OperationDeadline` (`lib.rs:1448-1485`): it rejects zero and
   `> MAX_TRANSITION_TIMEOUT`, and every subsequent command, state poll, and
   re-inspection draws from `remaining()` — the budget only ever shrinks. The
   systemd polling loop sleeps
   `min(remaining, 100ms)` (`lib.rs:1438-1442`).
3. **Non-zero exit is not success.** `ensure_mutation_success` requires
   `status == Some(0)` (`lib.rs:1558`). A `CommandOutput` with a non-zero status
   is *returned* by `SystemExecutor` as `Ok` so the caller can classify it, and
   it is the adapter's job to reject it.
4. **Confirm the resulting state.** A `completed: true` claim must be backed by
   an observation of the end state, not by the absence of an error.
5. **Bounded detail.** `TransitionResult::detail` goes through
   `ServiceError::bounded` (512 bytes) in every in-crate implementation.
6. **No elevation.** Access denial is a bounded hint, not an escalation
   (`permission_hint`, `lib.rs:1541-1556`).

`TestDoubleManager` satisfies all six deterministically and is the reference for
how the crate expects a test double to behave. It additionally treats
"already running" / "already stopped" as `completed: true` idempotent outcomes
(`lib.rs:523-528`, `547-552`) — a shape real adapters also implement
(`windows_scm.rs` test `stop_of_already_stopped_registration_is_complete_without_a_control`).

## Process execution: `SystemExecutor`

`CommandExecutor` (`lib.rs:645`) is the seam: argv in, `CommandOutput` out, with
`stdin_data: None` meaning null stdin and `Some(bytes)` meaning bounded input
(used only for `crontab -`). Missing binaries, timeouts, and output overflows
are `Err(ServiceError::Manager)`; **non-zero exits are `Ok` with the observed
status** (`lib.rs:648-651`).

### Executable allowlisting and path resolution

`resolve_manager_program` (`lib.rs:876-901`) is the only path from a program name
to a binary:

| Requested | Resolution |
|---|---|
| Absolute path | Validated directly |
| `systemctl` | `/usr/bin/systemctl`, then `/bin/systemctl` |
| `launchctl` | `/bin/launchctl`, then `/usr/bin/launchctl` |
| `crontab` | `/usr/bin/crontab`, then `/bin/crontab` |
| Anything else | `Err(InvalidInput("manager program must be absolute or allowlisted"))` |

There is **no `PATH` lookup at all**. `validate_executable_path`
(`lib.rs:903-924`) additionally requires the path to be absolute, to exist, to
be a regular file (not a symlink-to-directory or device), and — under
`#[cfg(unix)]` — to have at least one execute bit (`mode & 0o111 != 0`).
All candidates failing validation yields `Manager("trusted manager binary missing")`.

### Environment clearing

`cmd.env_clear()` (`lib.rs:724`) runs unconditionally. The only variables
re-added come from `filtered_manager_environment` (`lib.rs:926-947`), and only
when the argv is `systemctl` **with** `--user`:

`DBUS_SESSION_BUS_ADDRESS`, `XDG_RUNTIME_DIR`, `SYSTEMD_BUS_ADDRESS`.

Any other argv shape gets an empty environment. Tests
`command_environment_adds_nothing` and
`manager_resolution_ignores_ambient_path_and_environment_filters_sentinels`
(`lib.rs:3425`, `3439`) pin both halves.

### Output and timeout bounds

| Bound | Value | Enforced at |
|---|---|---|
| Per-stream captured output | `max_output_bytes` (default `MAX_COMMAND_OUTPUT_BYTES` = 256 KiB; `with_max_output` accepts `1..=16 MiB`) | `lib.rs:688`, `964` |
| stdin payload | `MAX_CRONTAB_BYTES` = 1 MiB | `lib.rs:712` |
| command timeout | non-zero and `<= MAX_TRANSITION_TIMEOUT + 60s` | `lib.rs:709` |
| argv | non-empty, no NUL in any element | `lib.rs:703`, `706` |

Output overflow is **fail-closed**: `read_bounded_generic` returns an overflow
flag (`lib.rs:949-972`) and `run` returns
`Manager("manager output exceeded bound")` rather than a truncated buffer
(`lib.rs:838-842`). A reviewer should note that the child is *not* killed on
overflow — it has already exited at that point in the flow.

### The stdin deadlock fix

The reader threads are attached **before** the stdin write (`lib.rs:741-761`),
and the stdin write itself runs on its own thread driven by the same
deadline-checked wait loop (`lib.rs:762-793`, `794-805`). A child that fills
stdout before draining stdin can therefore no longer deadlock both sides: the
kill at the deadline closes the pipe and unblocks the writer. A *completed but
failed* write still fails closed, and its verdict is collected before the output
is trusted (`lib.rs:811-823`).

### No shell interpolation

There is no `sh -c`, no `Command::args` stringification, and no
`PATH`-mediated resolution. `cmd.args(&argv[1..])` passes each element as one
argument (`lib.rs:721-723`). `ServiceSpec::new` additionally rejects control
characters in args (`lib.rs:91-95`) and `run` rejects NUL in any argv element
(`lib.rs:706`), which is the argv-level analogue of a shell-metacharacter
rejection. The same executor backs cron, where the desired block *is* a shell
line — that is caller-owned content written to the crontab, never
interpolated into a command this crate builds.

### `FakeExecutor`

`FakeExecutor` (`lib.rs:992`) scripts expectations as a FIFO of exact argv
vectors and records every observed call with its stdin. A mismatch or an
unscripted call is a hard `Err` (`lib.rs:1053-1061`), so **adapter tests fail if
the adapter issues an unexpected command** — this is the mechanism that keeps
the adapter command surface under test. `is_exhausted()` (`lib.rs:1036`) lets a
test assert every scripted command was consumed. It also implements
`CommandExecutor` for `&FakeExecutor` (`lib.rs:1066`) so it can be shared with
adapters that take ownership.

## Definition writing

`atomic_write_definition(path, bytes, allow_overwrite)` (`lib.rs:1087-1231`) is
the only way an adapter lands a unit file, plist, or similar.

| Step | Behavior | Ref |
|---|---|---|
| Byte bounds | Rejects empty or `> MAX_DEFINITION_BYTES` (1 MiB) | `lib.rs:1092` |
| NUL check | Rejects any NUL byte | `lib.rs:1097` |
| Parent check | Parent must exist, be a directory, and not be a symlink. **Never created** | `lib.rs:1100-1109` |
| No-clobber pre-check | `dest` exists and `!allow_overwrite` → `Conflict` | `lib.rs:1111-1115` |
| Mode | Preserve the existing destination mode on overwrite; `0644` for a new file | `lib.rs:1116-1127` |
| Temp creation | Same directory, `create_new(true)`, mode `0600`, name `.eggup-def-<pid>-<nanos>[-<n>].tmp`, 32 collision retries | `lib.rs:1133-1162` |
| Durability | `write_all` → `flush` → `sync_all` on the temp | `lib.rs:1182-1187` |
| Promotion (overwrite) | `std::fs::rename` — which is `MoveFileEx(MOVEFILE_REPLACE_EXISTING)` on Windows, so it replaces atomically | `lib.rs:1197-1214` |
| Promotion (no-replace) | `std::fs::hard_link` then remove temp; `AlreadyExists` → `Conflict` | `lib.rs:1216-1229` |
| Directory fsync | Best-effort after promotion, errors ignored | `lib.rs:1207-1210` |
| Cleanup | A `Drop` guard removes the temp on every error path; disarmed only after successful promotion | `lib.rs:1165-1179` |

**How a partial write is prevented from becoming live:** the payload is written
and `fsync`ed into a *different, exclusively-created, owner-private* file in the
same directory (same filesystem, so promotion is a rename/link, not a copy). The
destination path is only ever touched by the final atomic promotion. A crash or
an I/O error leaves the old definition live and a temp file behind — never a
truncated live definition. The `Drop` guard is disarmed only after the promotion
succeeds, so a failed promotion cleans up rather than leaking.

A former `AlreadyExists` fallback that removed the destination before renaming
was removed because it was unreachable on Windows and would have created a
data-loss window; the comment at `lib.rs:1198-1203` records why.

## Lifecycle composition: `commit_with_lifecycle`

`commit_with_lifecycle` (`lifecycle_update.rs:254`) takes an **already
validated** `ValidatedTransaction` and wraps Core's
`commit_with_post_commit` with service quiescence and restoration. It never
installs or refreshes a registration.

```text
policy.validate()                        InvalidPolicy   (before any manager call)
  -> safe_inspect                        Preflight       panics contained
     gate: ownership == Owned
           state in { Running, Stopped }
  -> [if Running] manager.stop(quiesce_timeout)
        + re-inspect                     Quiesce
        on failure: restore Running, return Quiesce { failure, restore_failure }
  -> transaction.commit_with_post_commit(ownership, post_commit_failure, || {
         deadline = now + post_commit_timeout
         restore_success_state(...)        RestoreNew -> PostInstallCheck
         on failure:
           post_commit_failure = Some(f)
           if policy == RollBack:
              quiesce_for_rollback(...)    QuiesceForRollback
           return Err(PostInstallCheckError(f.detail))
     })
  -> match Core disposition               Committed | RolledBack | RecoveryRequired
  -> safe_inspect                         FinalInspect (nonessential)
```

### What each step can fail with

| Step | Failure | Observable outcome |
|---|---|---|
| `policy.validate()` | `LifecycleUpdateError::InvalidPolicy { field }` | One of the three timeouts is zero or `> MAX_TRANSITION_TIMEOUT` (`lifecycle_update.rs:96-107`) |
| Inspect gate | `Preflight` | `Absent`, `Foreign`, `Unknown`, or `Transitioning`/`Unknown` state (`lifecycle_update.rs:265-279`) |
| Inspect | `Preflight` | A panicking manager is contained by `safe_inspect` and reported as a `LifecycleFailure` |
| Quiesce stop | `Quiesce { failure, restore_failure }` | Manager error, panic, `completed == false`, or post-stop ownership/state drift. Restoration to `Running` is attempted and its failure is reported **separately** |
| Post-commit work | `LifecycleFailure` + Core's `post_commit_failure` policy | `RestoreNew` or `PostInstallCheck` phase; on `RollBack` a `QuiesceForRollback` failure is recorded separately |
| Core before receipt | `CoreBeforeReceipt { core, restore_failure }` | Only lock contention / setup failures. Pre-update state is restored if the service had been quiesced |
| Final inspect | **Not an error** | `final_snapshot: None` plus `observation_failure: Some(..)`; Core's disposition is unchanged |

### The restoration contract

- `Committed`: the new generation is restored to `wanted` derived from
  `policy.restore`. If the failure phase was `RestoreNew`,
  `restoration = Failed` (`lifecycle_update.rs:399-406`).
- `RolledBack`: the **pre-update** state (`before.state`) is restored,
  overriding `policy.restore` (`lifecycle_update.rs:407-418`).
- `RecoveryRequired`: `restoration = NotAttemptedRecoveryRequired` and **no
  start is attempted** — the artifacts are uncertain, so Eggup refuses to run
  them (`lifecycle_update.rs:419-421`).

The post-install check receives the *remaining* budget of the single
`post_commit_timeout` deadline (`lifecycle_update.rs:342`, and the check call
site). `PostInstallCheck::check` documents that it must return within
`remaining` and that it cannot be forcibly cancelled
(`lifecycle_update.rs:130-139`). The existing `HealthProbe` trait has **no
timeout contract** and therefore cannot satisfy this bound on its own; health
semantics stay caller-owned.

Panic attribution is a specific review concern. `safe_restore_state` catches
each transition's panics at the phase actually executing, so the outer
post-commit `catch_unwind` arm can only be reached by the check itself
(CHANGELOG, Unreleased; test
`panicking_restore_transition_is_recorded_as_restore_new_not_a_check`,
`lifecycle_update.rs:1313`).

## Runtime disposition (M006) — the review focus

`disposition.rs` exists because `commit_with_lifecycle` answers only one shape
of question: "quiesce my owned service, commit, put it back." Real deployments
have more cases — a foreign registration that must not be touched, a program run
directly with no manager, or nothing running at all. M006 generalizes that
without loosening the safety posture.

### The central distinction

> **Artifact commit authority and manager mutation authority are separate
> facts.** (`disposition.rs:29`)

Core decides whether the artifact bytes may be committed. The disposition decides
whether Eggup may call a service manager at all. `ForeignPreserved` commits
artifacts and performs **zero** manager mutation. `DirectRunning` mutates only
through a caller-owned seam that Eggup does not interpret. Neither conclusion
implies the other, and `LifecycleUpdateReceipt` keeps them apart:
`artifact_disposition()` and `service_state_restored()` are separate accessors
(`lifecycle_update.rs:182-199`).

### `UpdateRuntimeDisposition`

| Variant | Meaning | Manager mutation | Artifact commit |
|---|---|---|---|
| `ManagedRunning` | Owned manager, running | `stop` before, restore/start after | Yes |
| `ManagedStopped` | Owned manager, stopped | Restore per intent only | Yes |
| `DirectRunning` | Selected direct runtime, running | **None** — only `DirectRuntimeControl` | Yes |
| `Stopped` | No running runtime | None | Yes, with no fabricated restart |
| `ForeignPreserved` | Foreign manager | **None**, ever | Yes |

`disposition.rs:31-42`.

### `DirectRuntimeControl`

`disposition.rs:256-263`. Three methods: `inspect`, `stop`, `start`. It is a
**caller-supplied trait**, and the crate's own comments treat that as a
responsibility transfer, not a delegation: there is no second ownership check
inside `stop`, so Eggup performs every identity verification itself before
calling it (`disposition.rs:1273-1277`). The seam carries no health-protocol
content — `DirectObservation` is a state plus identity facts
(`exact_ownership_proven`, `executable`, `config`), not a health channel.

### The pure planners

`plan_unix` and `plan_windows` are total, side-effect-free functions over
caller-supplied facts. They exist so the decision matrix can be tested without a
native manager, and so a caller can compute and record a disposition *before*
committing anything.

#### `plan_unix` decision table

`disposition.rs:125-221`. First gate: if `manager_state_known == false`
(manager transitioning or unknown), the result is `Stopped` only in the single
narrow case `manager_ownership == Unknown && !manager_active &&
direct.state == Stopped`; otherwise `Err`.

| `manager_ownership` | `manager_active` | Registered vs selected config | `direct.state` / proven | Result |
|---|---|---|---|---|
| `Owned` | true | — | — | `ManagedRunning` |
| `Owned` | false | — | — | `ManagedStopped` |
| `Foreign` | true | `Unknown` | — | **Err** (refuses to infer) |
| `Foreign` | true | `Absent` or `Present(_)` and config matches | — | `ForeignPreserved` |
| `Foreign` | true | config differs | `Running` + proven | `DirectRunning` |
| `Foreign` | true | config differs | `Stopped` | `Stopped` |
| `Foreign` | true | config differs | other | **Err** |
| `Foreign` | false | — | `Running` + proven | `DirectRunning` |
| `Foreign` | false | — | `Stopped` | `Stopped` |
| `Foreign` | false | — | other | **Err** |
| `Absent` | true | — | — | **Err** (contradictory) |
| `Absent` | false | — | `Running` + proven | `DirectRunning` |
| `Absent` | false | — | `Stopped` | `Stopped` |
| `Absent` | false | — | other | **Err** |
| `Unknown` | true | — | — | **Err** |
| `Unknown` | false | — | `Running` (even if proven) | **Err** (refuses direct mutation) |
| `Unknown` | false | — | `Stopped` | `Stopped` |
| `Unknown` | false | — | `Unknown` | **Err** |

Two rows deserve a reviewer's attention:

- The `Foreign` + active + *matching config* row is the only path to
  `ForeignPreserved` from an active foreign manager. It is justified: the
  registered artifact is byte-identical to the selected one, so there is no
  separate runtime to quiesce. The alternative rows fall through to the direct
  seam.
- The `Unknown` + not active + `Running` row refuses `DirectRunning` **even when
  `exact_ownership_proven` is true**. Unknown manager ownership plus an
  unmanageable running process is treated as "manager and direct authority are
  both in doubt", so no destructive call happens.

#### `plan_windows` decision table

`disposition.rs:224-253`. The Windows matrix has no direct-runtime concept.

| `installed` | `executable_known` | `ownership` | `state` | Result |
|---|---|---|---|---|
| false | — | — | — | `Stopped` |
| true | false | — | — | **Err** (executable absent or unparseable) |
| true | true | `Foreign` | any | `ForeignPreserved` |
| true | true | `Owned` | `Running` \| `StartPending` | `ManagedRunning` |
| true | true | `Owned` | `Stopped` \| `StopPending` | `ManagedStopped` |
| true | true | `Owned` | `Unknown` | **Err** |
| true | true | `Absent` | any | **Err** (contradictory) |
| true | true | `Unknown` | any | **Err** (ownership unproven) |

Pending states are folded into their target class: a `StartPending` service is
treated as running for quiesce purposes, a `StopPending` one as stopped.

### Where the revalidation barrier sits

`commit_with_disposition` (`disposition.rs:321-593`) enforces three gates before
any mutation, in this order:

1. **Policy validation** (`disposition.rs:338`) — same zero / over-ceiling rules
   as the lifecycle path.
2. **Baseline coherence** (`disposition.rs:340-368`) — the `planned`
   disposition must actually match the `DispositionBaseline` observed after
   preparation. `ManagedRunning` requires `Owned` + `Running`;
   `ManagedStopped` requires `Owned` + `Stopped`; `DirectRunning` requires a
   baseline direct observation that is `Running` with
   `exact_ownership_proven`; `ForeignPreserved` requires a non-`Owned` manager.
   `Stopped` is unconditionally coherent because it mutates nothing.
3. **Fresh re-inspection** (`disposition.rs:370-383`) — the manager is
   re-inspected and both `ownership` **and** `state` must equal the baseline.
   Any change is `Preflight("lifecycle authority/state changed before mutation")`.
4. **Direct revalidation** (`disposition.rs:386-438`), only when
   `planned == DirectRunning` — `direct.inspect()` must return a value
   **equal** to the baseline observation, with `exact_ownership_proven` true and
   state `Running`. This catches an executable/config identity change, not just
   a state change.

Only after all gates does the `match planned` dispatch run
(`disposition.rs:446`). `ManagedRunning` and `DirectRunning` quiesce and then
re-verify the stopped state before handing the transaction to Core;
`ManagedStopped`, `Stopped`, and `ForeignPreserved` go straight to
`commit_manager_path` with `pre_commit_quiesced = false`
(`disposition.rs:579-591`).

**This is the revalidation barrier, and it is the point of the module.** The
planning phase happens before `prepare`; the baseline is captured after
preparation; the barrier re-observes authority immediately before the first
destructive call. A caller that plans, prepares, waits a long time, and then
commits an owned service that has since been replaced is denied before the stop,
not after.

### Post-commit and rollback behavior per disposition

`commit_manager_path` (`disposition.rs:662-796`) drives
`transaction.commit_with_post_commit`. `restore_manager_success`
(`disposition.rs:798-858`) resolves the target state:

| Planned | Target state |
|---|---|
| `ForeignPreserved` | **No transition at all.** Inspect (read-only) then the post-install check |
| `Stopped` | `EnsureRunning` → `Running`; otherwise `Stopped` |
| any other | `Preserve` → `before.state`; `EnsureRunning` → `Running`; `EnsureStopped` → `Stopped` |

On `PostCommitFailurePolicy::RollBack`, `quiesce_manager_for_rollback`
(`disposition.rs:955-1029`) runs before Core rolls the artifacts back. It fails
closed in three ways worth naming: an **inspect error is treated as
unobservable, not as "proceed"** (`disposition.rs:973-983`); a non-`Owned`
service is refused (`disposition.rs:990-999`); and the stopped state is
re-confirmed by a second `inspect` after the `stop`
(`disposition.rs:1013-1027`). Tests
`unobservable_service_is_not_stopped_before_rollback` and
`unobservable_direct_runtime_is_not_stopped_before_rollback`
(`disposition.rs:1774`, `1745`) pin both.

`commit_direct_path` (`disposition.rs:1032`) is the mirror image for
`DirectRunning`, with one structural difference: the direct `start` is wrapped in
a **nested** `catch_unwind` labelled `RestoreNew`, so the outer arm
(`PostInstallCheck`) is reachable only by the check itself
(`disposition.rs:1065-1113`). The check snapshot comes from a read-only manager
inspect, because direct state is caller-owned and is deliberately not part of
`LifecycleSnapshot` (`disposition.rs:1097-1099`).

Rollback disposition handling, both paths:

| Core disposition | `ManagedRunning` / `ManagedStopped` | `DirectRunning` | `Stopped` / `ForeignPreserved` |
|---|---|---|---|
| `Committed` | `restoration = Failed` if the failure phase was `RestoreNew` | same | same |
| `RolledBack` | Restore `before.state` via the manager | Restart the prior direct generation | No restoration attempted |
| `RecoveryRequired` | `NotAttemptedRecoveryRequired`; no start | same | same |

### Worked walk-through: `ForeignPreserved`

Scenario: a foreign systemd unit named `other-app.service` is active and runs
the same executable and the same config path the caller selected. The caller
wants Eggup to update the artifact underneath it without touching the manager.

1. **Collect facts.** `manager_ownership = Foreign`, `manager_active = true`,
   `manager_state_known = true`, `registered_config = Present("/etc/app/config.toml")`,
   `selected_config = Some("/etc/app/config.toml")`,
   `direct = { state: Stopped, exact_ownership_proven: true, .. }`.
2. **Plan.** `plan_unix` takes the `Foreign` + active branch, finds the
   registered/selected config equal, and returns `ForeignPreserved`
   (`disposition.rs:159-160`).
3. **Prepare.** The caller builds the `ValidatedTransaction` via
   `eggup-core`, then observes the baseline into `DispositionBaseline`:
   `manager_ownership = Foreign`, `manager_state = Running`,
   `direct = None`.
4. **Enter `commit_with_disposition`.** `policy.validate()` passes. The baseline
   gate accepts `Foreign` (`disposition.rs:354-357`). The fresh `inspect` must
   still report `Foreign` + `Running`. The direct revalidation block is skipped —
   `planned != DirectRunning`, so a missing `direct` handle is not an error.
5. **Dispatch.** The `match planned` arm falls through to `commit_manager_path`
   with `pre_commit_quiesced = false` (`disposition.rs:581-591`). **No `stop`,
   no `start`, no definition write.** The test
   `foreign_preserved_performs_zero_manager_mutation`
   (`disposition.rs:1995`) asserts exactly this.
6. **Commit and restore.** Core commits the artifacts. `restore_manager_success`
   short-circuits on `ForeignPreserved` (`disposition.rs:812`): one read-only
   `inspect` and then the caller's `PostInstallCheck` with the remaining budget.
7. **Receipt.** `restoration = Restored`, `pre_commit_quiesced = false`,
   `final_snapshot = Some(..)`, and `artifact_disposition() == Committed` if the
   bytes landed. If Core needed rollback, `quiesce_manager_for_rollback` would
   refuse the non-`Owned` service, but the check failure path is the only way to
   reach it.

The caller gets a committed artifact set and an untouched foreign service. The
two facts are reported separately and neither is inferred from the other.

### What a caller must guarantee

1. **The plan is honest.** `plan_unix` / `plan_windows` are the crate's reference
   matrix; feeding them inputs that do not reflect the real host is a
   precondition failure the crate cannot detect.
2. **The baseline is captured after preparation, immediately before calling.**
   The barrier compares against it, so a stale baseline produces a false
   *denial*, not a false permission.
3. **`DirectRunning` requires a live `DirectRuntimeControl` handle** and an
   exact-identity `DirectObservation`. A `None` handle is a `Preflight` failure
   (`disposition.rs:387-396`).
4. **The caller enforces post-commit rollback policy**, and the crate composes
   with Core's `KeepInstalled` / `RollBack` — it does not invent its own.
5. **Application health stays caller-owned.** Disposition decides *authority*,
   not *health*; a `ManagedRunning` commit whose check reports a degraded
   application still returns `Committed` plus a `post_commit_failure`.

## Platform adapters

### systemd (`SystemdManager`, `lib.rs:1327`)

Drives `systemctl` with an **explicit** scope (`--system` / `--user` via
`SystemdScope::flag`, `lib.rs:1245`) — never guessed from EUID. Inspection is
`show` with `LoadState` / `ActiveState` / `ExecStart`; ownership comes from the
narrow `parse_exec_start` (`lib.rs:1574`) plus `reconcile_config_identity`.
`ExecStart` shapes the parser does not understand yield `Unknown`, not a guess.
`install` writes the unit with `atomic_write_definition` and then runs
`post_write` (`lib.rs:1883`) — `enable` and `daemon-reload` only when the caller
asked for them via `SystemdInstall::{enable,reload}`. `start` / `stop` / `restart`
are bounded by `OperationDeadline` and confirm the resulting state.

`stop` handles an exact-owned unit reported as `failed` without changing the
closed public `LifecycleState` enum. It re-reads the full systemd observation
immediately before stopping and after the stop. Completion requires matching
ownership, a stable failed/inactive state, no active job, zero `MainPID` and
`ControlPID`, and either no unit control group or an empty cgroup v2 process
list with `cgroup.events` reporting `populated 0`. It also checks that
`systemctl is-active` agrees with the final `ActiveState`. Unknown state,
changed identity/control group, inaccessible or malformed cgroup evidence, and
restart/transition races return incomplete or an error; they do not authorize
rollback against a possibly live process. It never calls `reset-failed`.
`plans/closure/service-lifecycle/010-status.md` records the real-systemd
qualification and its negative controls.

`reconcile_config_identity` (`lib.rs:1491-1524`) is the subtle part: the config
path is only treated as part of identity if it appears **exactly once** in the
spec's canonical args and once in the observed args. Zero occurrences in the
observed record leaves `config` unset, which makes the comparison in
`RegistrationSnapshot::ownership` fail → `Foreign`. Two or more on either side
sets `malformed` → `Unknown`. A non-UTF-8 config path also sets `malformed`
(`lib.rs:1498-1501`).

### launchd (`LaunchdManager`, `lib.rs:2025`)

Drives `launchctl` against an **explicit** target. `LaunchdDomain::UserAgent`
requires a `gui/<uid>`-shaped target and `SystemDaemon` requires exactly
`system` (`lib.rs:1966-1977`) — mismatches are rejected at construction, so the
domain is never inferred at call time. `parse_launchd_plist` (`lib.rs:2135`)
reads structured `ProgramArguments`; a malformed plist produces
`malformed_snapshot()` (`lib.rs:2209`) → `Unknown`. A label match alone never
proves ownership — the parsed executable and arguments must also match.
Operations map to `bootstrap` / `bootout` / `kickstart` / `stop`, each bounded
(`lib.rs:2452`, `2466`, `2480`, `2499`). `restart` does not start after an
incomplete stop (test `lib.rs:3999`).

### cron (`CronManager`, `lib.rs:2717`)

**Cron is treated as a distinct scheduling model, not as a service manager.**
It has no `start` / `stop` / `restart` semantics at all, and those methods
return explicit `InvalidInput` errors rather than pretending success
(`lib.rs:2914-2943`; test `cron_has_no_start_stop_semantics`, `lib.rs:4254`).
`inspect` reports registration state from block classification and always
reports `LifecycleState::Stopped`.

Ownership is over a managed `# BEGIN <marker>` / `# END <marker>` block with an
exact desired body:

| `CronOwnership` | Condition | Behavior |
|---|---|---|
| `Absent` | no block | install appends; uninstall is a no-op without a rewrite |
| `Owned` | exactly one block, body equals desired | idempotent; `changed = false` |
| `Foreign` | one block, different body | `ServiceError::denied` — never overwritten |
| `Unknown` | multiple blocks, or an unterminated `BEGIN` | `ServiceError::Conflict` |

`cron_classify` / `cron_merge` / `cron_remove` (`lib.rs:2592`, `2620`, `2665`)
are pure and preserve unrelated bytes, including the trailing-newline
convention. The command adapter re-reads the crontab immediately before a
destructive write to surface races, and skips a redundant `crontab -` write when
`changed == false` (test `lib.rs:4187`). The `Foreign` and `Unknown` errors are
attributed to a **synthetic** `cron:<marker>` id, not to the caller's spec id,
because the marker — not the spec — owns the block (`lib.rs:2650-2654`).

### Windows SCM (`WindowsScmManager`, `windows_scm.rs:232`)

Uses the safe `windows-service` wrapper behind a boxed `ScmBackend`
(`windows_scm.rs:233`), which is how the adapter is tested deterministically off
Windows. No `sc.exe`, no shell, no ambient `PATH`, no elevation.

- **Identity is the command line, not the key name.** `observe`
  (`windows_scm.rs:305-323`) maps `Absent` → `Absent`,
  `MarkedForDelete` → `Unknown`, and otherwise calls `registration_ownership`
  against the parsed command line. `require_owned_observation`
  (`windows_scm.rs:325-338`) is the gate for every mutation.
- **Pending states are real.** `wait_for_state` (`windows_scm.rs:340-365`) polls
  with a per-record `wait_hint` clamped to `MAX_POLL_INTERVAL` (500 ms) and never
  sleeps past the deadline. `stop_phase_for_restart`
  (`windows_scm.rs:367-400`) waits out `StartPending` / `StopPending` /
  `ContinuePending` / `PausePending` before deciding.
- **Uninstall is honest about `MarkedForDelete`.** It reports
  `completed: false` until the SCM confirms absence (tests
  `windows_scm.rs:1547`, `1575`).
- **Access denial is bounded remediation.** `scm_error_detail`
  (`windows_scm.rs:215-224`) special-cases code `5` into a "run with an account
  authorized for this service operation" message; there is no elevation path
  (test `windows_scm.rs:1252`).
- **Refresh preserves what the adapter does not own** — service type,
  unspecified dependencies, account, load-order group, tag.
- **Account selection is create-time only.** Custom passwords, descriptions, and
  recovery actions are out of scope for this generic adapter.
- `validate_spec` (`windows_scm.rs:260-303`) additionally requires the spec id to
  match the install descriptor, the executable to be a Windows-absolute path
  representable as UTF-8, and the assembled command line to fit
  `MAX_SCM_COMMAND_CHARS` (8192 UTF-16 units).

### Behavioral differences, summarized

| Aspect | systemd / launchd | cron | Windows SCM |
|---|---|---|---|
| Start/stop/restart | Yes, bounded and state-confirmed | **No** — explicit `InvalidInput` | Yes, pending-aware polling |
| Ownership proof | Parsed `ExecStart` / `ProgramArguments` | Exact managed block body | Parsed command line |
| Config identity | One-occurrence argv reconciliation | n/a | Exactly one config argument |
| Non-zero exit | `ensure_mutation_success` rejects | Same | SCM status codes |
| Ambient environment | Cleared; three vars for `--user` systemd | Cleared | n/a (no subprocess) |
| Failure-to-observe | `Unknown`, mutation denied | `Unknown`, mutation refused | `Unknown`, mutation denied |

### Host detection

`inspect_host` (`lib.rs:3018`) returns `HostFacts` and **distinguishes
"detection failed" (`Err`) from "not installed" (`*_available = false`)** —
`systemd` is probed on Linux only and additionally requires
`/run/systemd/system` to exist, `launchctl` on macOS only, `crontab`
everywhere. `candidate_managers` (`lib.rs:3072`) is a separate, explicitly
overridable policy layer over those facts: native manager first, cron as the
fallback, empty when nothing is available. Detection never elevates and never
mutates.

## Invariants and failure modes

| Condition | Observable outcome | Enforced at |
|---|---|---|
| Destructive operation on `Foreign` / `Unknown` / `Absent` | `Err(ServiceError::OwnershipDenied)`, no manager call | `require_owned`, `lib.rs:397` |
| Malformed registration | `Ownership::Unknown` → denied | `RegistrationSnapshot::ownership`, `lib.rs:201`; `malformed_snapshot`, `lib.rs:2209` |
| Manager command returns non-zero | `Err(ServiceError::Manager)` with bounded stderr | `ensure_mutation_success`, `lib.rs:1558` |
| Manager command errors during inspect | Treated as `Unknown`, not `Absent` | `lib.rs:1676-1680`; rollback quiesce, `disposition.rs:973` |
| Output exceeds the bound | `Err(Manager("manager output exceeded bound"))` | `read_bounded_generic`, `lib.rs:949`; checked `lib.rs:838` |
| Transition deadline expires | `Err(Manager("transition deadline exhausted before manager command"))` or `TransitionResult { completed: false }` | `OperationDeadline`, `lib.rs:1472`; `wait_for_state`, `windows_scm.rs:351` |
| Zero or overlong policy timeout | `LifecycleUpdateError::InvalidPolicy { field }` before any mutation | `LifecycleUpdatePolicy::validate`, `lifecycle_update.rs:96` |
| Post-install check fails | Core's `KeepInstalled` / `RollBack` policy applied; failure recorded with its phase | `lifecycle_update.rs:340-372` |
| Core reports `RecoveryRequired` | `NotAttemptedRecoveryRequired`; no automatic start | `lifecycle_update.rs:419`; `disposition.rs:765`, `1215` |
| Authority changes between planning and mutation | `Preflight` before the first destructive call | `disposition.rs:370-383` |
| Direct identity changes before mutation | `Preflight` before the first destructive call | `disposition.rs:425-437` |
| Planned disposition contradicts the baseline | `Preflight("planned disposition does not match baseline authority/state")` | `disposition.rs:359-368` |
| `DirectRunning` with no `DirectRuntimeControl` handle | `Preflight` | `disposition.rs:387-396` |
| Unobservable runtime before rollback quiesce | Refused; `QuiesceForRollback` failure recorded | `disposition.rs:973-983`, `1291-1299` |
| Manager or check panics | Contained at the executing phase, not the outer arm | `safe_inspect` / `safe_restore_state` / `safe_transition_owned_to` |
| Definition parent missing or a symlink | `InvalidInput`; parents are never created | `atomic_write_definition`, `lib.rs:1100-1109` |
| Definition destination exists, `allow_overwrite = false` | `Err(Conflict)`; no write | `lib.rs:1111`, `1225` |
| Access denied by a manager | Bounded remediation hint, no elevation | `permission_hint`, `lib.rs:1541`; `scm_error_detail`, `windows_scm.rs:215` |
| Foreign cron marker | `OwnershipDenied` on synthetic `cron:<marker>` id; no overwrite | `cron_merge` / `cron_remove`, `lib.rs:2650`, `2701` |
| Duplicate / unterminated cron marker | `Conflict`; no write | `find_blocks` + `cron_classify`, `lib.rs:2565`, `2611` |
| Shell metacharacters in spec args | Rejected at `ServiceSpec::new` (control chars) and at `run` (NUL) | `lib.rs:91`, `706` |

## Testing approach

104 `#[test]` functions across the four files. There is no `tests/` directory;
everything is a unit test in-module (`Cargo.toml` excludes `tests/` and
`benches/` from the package). The distribution:

| Area | Tests | Demonstrates |
|---|---|---|
| Neutral model (`lib.rs:3093+`) | ~13 | Ownership classification, snapshot intent preservation, health/state separation, denial of `Foreign` / `Unknown`, `RestoreIntent` |
| `SystemExecutor` (`lib.rs:3345-3520`) | ~7 | Timeout kills and reaps, output bound fails closed, literal argv with no shell, non-zero exit is `Ok` with status, missing binary fails closed, env adds nothing, `PATH` ignored, deadline rejects zero and only shrinks |
| systemd (`lib.rs:3520-3850`) | ~11 | Absent / exact-owned / foreign-executable / argv-drift / ambiguous-`ExecStart` / specifier-`ExecStart` / config-identity / state mapping / permission-without-`sudo` / atomic-write fault preserving the old file / bounded start-stop-restart |
| launchd (`lib.rs:3851-4116`) | ~7 | Absent/exact/foreign/unknown, user-vs-system domain distinctness, malformed plist, bootstrap/bootout/kickstart, restart-after-incomplete-stop, label-only is not ownership, config reconciliation |
| cron (`lib.rs:4117-4271`) | ~9 | Empty-to-absent-to-install, byte-for-byte preservation of unrelated entries, idempotence, duplicate marker, modified block is foreign, owned-only removal, no redundant write, timeout/missing binary, no start/stop semantics |
| Definition + detection (`lib.rs:4271-4457`) | ~6 | Atomic private no-clobber write, missing parent rejected, unavailable vs failure distinguished, candidate policy order, native `systemctl` / `launchctl` / `crontab -l` smoke, consumer-specific constants rejected |
| `lifecycle_update.rs` | 17 | Policy bounds, restore-intent matrix, preconditions, quiesce failures, `KeepInstalled` evidence retention, rollback quiesce-then-restore, caller-check panic, restore-phase panic attribution, `CoreBeforeReceipt` restore, post-rollback restore failure separation, `RecoveryRequired` suppression, single shared deadline, no hidden policy retry |
| `disposition.rs` | 20 | Unix reference matrix, Windows reference matrix, unobservable-runtime refusal, ordering per disposition, zero mutation under `ForeignPreserved`, direct seam exclusivity, revalidation failure modes, `KeepInstalled` composition, `RecoveryRequired` suppression, no product terminology in public types, prepare-before-quiesce, rollback quiesce + old restore |
| `windows_scm.rs` | 15 | Name/descriptor validation, access-denied remediation without escalation, Windows command-line parsing (quotes, backslashes, unicode) and its rejections, config-identity arity, explicit lifecycle mapping for every SCM state, absent/exact identity, spec validation, create/refresh, one-deadline transitions, restart safety, `MarkedForDelete` honesty, idempotent stop |

The three tests CI runs individually on Windows are pinned because they assert
UTF-8 boundary behavior that a differently-configured default run would not
exercise (`.github/workflows/ci.yml:50-53`):

```sh
cargo test -p eggup-service --lib diagnostic_byte_bounds_preserve_utf8_at_256_and_512_edges --locked
cargo test -p eggup-service --lib service_errors_output_and_permission_remediation_stay_utf8_bounded --locked
cargo test -p eggup-service --lib lifecycle_failure_detail_is_utf8_safe_bounded_and_control_free --locked
cargo test -p eggup-service --lib windows_scm::tests --locked
```

The first two are in `lib.rs:3109` and `lib.rs:3129`; the third is
`lifecycle_update.rs:710`. The fourth runs the whole SCM test module, whose
platform-independent tests (parser, ownership, lifecycle mapping) run on every
host — the `#[cfg(windows)]` backend is the only part that is Windows-only.

Note the shape choice: `no_gregg_strings_in_public_types`
(`disposition.rs:2243`) asserts that the M006 public types contain no reference
to the behavioral oracle the milestone was qualified against, keeping the
surface product-neutral. That is a real regression guard, not a naming
preference.

Focused commands for review:

```sh
cargo test -p eggup-service --locked
cargo test -p eggup-service --lib commit_with_disposition --locked
cargo test -p eggup-service --lib commit_with_lifecycle --locked
cargo test -p eggup-service --lib plan_ --locked
cargo clippy -p eggup-service --all-targets --locked -- -D warnings
```

The full gate is `./scripts/check-local.sh`; see
[tooling-governance.md](tooling-governance.md).

## Known doc/code drift

- **Three UTF-8 boundary tests, not two.** CI runs three individually
  (`ci.yml:50-52`) — the third, `lifecycle_failure_detail_is_utf8_safe_bounded_and_control_free`,
  lives in `lifecycle_update.rs` rather than `lib.rs`. Two are in `lib.rs`.
- **`ServiceId` length is bytes, not characters.** `lib.rs:43` uses
  `value.len() > 256`, which is a UTF-8 byte count. A 200-character non-ASCII
  identifier is rejected. The doc comment says "overlong" without specifying the
  unit. `validate_unit_name` and `validate_label` use the same byte rule, while
  the SCM validators deliberately use `encode_utf16().count()`
  (`windows_scm.rs:152`), which is the correct unit for a Windows API string.
- **`validate_marker` has a redundant second check.** `marker.chars().any(char::is_control)`
  already rejects `\n` and `\r` (`lib.rs:2536`), so the
  `marker.contains(['\n', '\r'])` arm at `lib.rs:2539` is unreachable. Harmless,
  but it reads as if control characters other than newlines were permitted.
- **CHANGELOG's Windows `AlreadyExists` removal is confirmed in code.** The
  comment at `lib.rs:1198-1203` matches the changelog entry; the current
  `allow_overwrite` path has a single `rename` and no destination-removal
  fallback.

## Cross-references

| Topic | Where |
|---|---|
| Workspace index, dependency graph, cross-cutting invariants | [overview.md](overview.md) |
| Core state machine, commit/rollback mechanics | [core-transaction.md](core-transaction.md) |
| `commit_with_post_commit` and `PostCommitFailurePolicy` | [transaction.md](../crates/eggup-core/docs/transaction.md) |
| `Owned` is never inferred — the Core-side half | [verification.md](../crates/eggup-core/docs/verification.md) |
| Transport composition this crate sits downstream of | [acquisition.md](acquisition.md) |
| No-clobber promotion as a workspace-wide invariant | [archive-extraction.md](archive-extraction.md) |
| Verification layers and the authenticity gap | [ADR-0003](../plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md) |
| M006 implementation plan | [006-daemon-update-disposition-and-reference-qualification.md](../plans/implementation/service-lifecycle/006-daemon-update-disposition-and-reference-qualification.md) |
| M006 closure evidence | [closure/service-lifecycle/006-status.md](../plans/closure/service-lifecycle/006-status.md) |
| Subsystem status | [subsystems/service-lifecycle-roadmap.md](../plans/subsystems/service-lifecycle-roadmap.md) |
| CI matrix and the local gate | [tooling-governance.md](tooling-governance.md) |
