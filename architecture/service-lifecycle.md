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
| [`src/commit_engine.rs`](../crates/eggup-service/src/commit_engine.rs) | ~0.7k lines | **The single commit-and-rollback engine**: transition, post-transition verification, rollback quiesce, panic-safe inspect/restore, deadline, detail truncation, and the two named policy inputs |
| [`src/disposition.rs`](../crates/eggup-service/src/disposition.rs) | ~2.3k lines | M006 runtime-authority disposition, pure planners, `commit_with_disposition` |
| [`src/lifecycle_update.rs`](../crates/eggup-service/src/lifecycle_update.rs) | ~1.1k lines | `commit_with_lifecycle`, policy/receipt/error types |
| [`src/windows_scm.rs`](../crates/eggup-service/src/windows_scm.rs) | ~1.6k lines | Native Windows SCM adapter behind an injectable backend |

The four submodules are private (`lib.rs:11-14`); everything is re-exported
flat from the crate root, so there is one public namespace.

**There is one engine.** `commit_engine.rs` is the only implementation of the
transition, post-transition verification, rollback-quiesce, panic-safe-inspect,
panic-safe-restore, deadline, and detail-truncation mechanics. Both
`commit_with_lifecycle` and `commit_with_disposition` reach it; neither keeps a
private copy. Before M012 the crate carried two divergent copies of that
engine — one in `disposition.rs`, one in `lifecycle_update.rs` — which had
already drifted apart in safety-relevant ways. The dependency direction is
acyclic: `commit_engine` depends only on the crate-root neutral model, and the
two orchestrators depend on `commit_engine`.

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
- **The crate is current with the workspace.** It is on crates.io at
  `0.1.0`, `0.1.1`, `0.1.2`, and `0.1.3`; the `0.1.2` service publication came
  from `7fb84bc`, after the shared `v0.1.2` tag, which remains the core/archive
  source tag. Service M011 published the `0.1.3` owned failed-systemd
  quiescence correction from source `feb6ae5`; its exact checksum and registry
  qualification are in the M011 closure.
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
| `ServiceId` | struct | Validated opaque identity (unit name, launchd label, SCM name) | `lib.rs:41` |
| `ServiceId::new` / `as_str` | fn | Rejects empty / >256 bytes / control chars | `lib.rs:45`, `lib.rs:56` |
| `ServiceSpec` | struct | Desired registration: id + executable + critical args + optional config | `lib.rs:74` |
| `ServiceSpec::new` | fn | Requires absolute executable and config, control-free args | `lib.rs:83` |
| `ServiceSpec::{id,executable,args,config}` | accessors | Identity components participating in ownership | `lib.rs:114-131` |
| `Ownership` | **re-export** | `Absent \| Owned \| Foreign \| Unknown` — **the same type** as `eggup_core::Ownership` | `lib.rs:165` |
| `LifecycleState` | enum (`#[non_exhaustive]`) | `Stopped \| Running \| Transitioning \| Unknown` | `lib.rs:170` |
| `HealthState` | enum (`#[non_exhaustive]`) | `Healthy \| Degraded \| Unknown` | `lib.rs:184` |
| `RegistrationSnapshot` | struct | Manager-neutral registration observation | `lib.rs:195` |
| `RegistrationSnapshot::absent` | fn | Canonical absent observation | `lib.rs:210` |
| `RegistrationSnapshot::ownership` | fn | Classifies the observation against a spec | `lib.rs:221` |
| `LifecycleSnapshot` | struct | Observation + `was_registered` / `was_running` intent | `lib.rs:240` |
| `LifecycleSnapshot::restore_running` | fn | Restoration intent derived from the observation | `lib.rs:257` |
| `RestoreIntent` | enum | `Preserve \| EnsureRunning \| EnsureStopped` | `lib.rs:264` |
| `TransitionResult` | struct | Bounded outcome: `operation`, `completed`, `detail` | `lib.rs:275` |
| `ServiceOperation` | enum (`#[non_exhaustive]`) | `Inspect \| Install \| Start \| Stop \| Restart \| Uninstall` | `lib.rs:294` |
| `ManagerFailureKind` | enum (`#[non_exhaustive]`) | `Spawn \| Timeout \| Permission \| NotFound \| Protocol \| LocalFilesystem` — the machine-readable manager failure classification | `lib.rs:339` |
| `ManagerFailureKind::from_spawn_error` | fn | Pure `std::io::ErrorKind` → kind mapping (`NotFound` / `PermissionDenied` / other) | `lib.rs:363` |
| `ManagerFailureKind::from_scm_os_code` | fn | Pure Win32/SCM code → kind mapping (`5` → `Permission`, `1060` → `NotFound`, other → `Protocol`); platform-independent | `lib.rs:378` |
| `ServiceError` | enum (`#[non_exhaustive]`) | `InvalidInput \| OwnershipDenied \| Conflict \| Manager { kind, detail, os_code }` | `lib.rs:406` |
| `ServiceError::denied` | fn | Canonical `OwnershipDenied` construction | `lib.rs:474` |
| `ServiceError::manager` / `manager_with_os_code` | fn | Typed manager-failure construction; the detail is bounded at `MAX_DIAGNOSTIC_BYTES` | `lib.rs:448`, `461` |
| `ServiceError::manager_kind` / `manager_os_code` | fn | Programmatic discriminators: the classified kind, and the platform OS code when one exists | `lib.rs:486`, `495` |
| `HealthProbe` | trait | Caller-supplied, read-only health check | `lib.rs:372` |
| `NoProbe` | struct | Always `HealthState::Unknown` | `lib.rs:379` |
| `ServiceManager` | trait | The six manager operations | `lib.rs:389` |
| `require_owned` | fn | The single destructive-operation gate | `lib.rs:426` |
| `TestDoubleManager` | struct | Deterministic in-memory manager | `lib.rs:437` |
| `MAX_COMMAND_OUTPUT_BYTES` | const | `256 * 1024` per stdout/stderr stream | `lib.rs:636` |
| `MAX_CRONTAB_BYTES` | const | `1024 * 1024` | `lib.rs:638` |
| `MAX_DEFINITION_BYTES` | const | `1024 * 1024` | `lib.rs:640` |
| `MAX_TRANSITION_TIMEOUT` | const | `Duration::from_secs(300)` | `lib.rs:642` |
| `MAX_DIAGNOSTIC_BYTES` | const | `512` — the **one** ceiling for manager-output diagnostic text | `lib.rs:912` |
| `ManagerCommandOutput` | struct | `status: Option<i32>`, bounded `stdout` / `stderr` | `lib.rs:662` |
| `CommandOutput` | **type alias** | Compatibility alias to `ManagerCommandOutput`; not deprecated | `lib.rs:694` |
| `CommandExecutor` | trait | Injectable runner: argv in, bounded output out | `lib.rs:961` |
| `SystemExecutor` | struct | Production runner; cleared env, allowlisted paths | `lib.rs:725` |
| `atomic_write_definition` | fn | Same-dir temp + fsync + no-replace/rename promotion | `lib.rs:1489` |
| `FakeExecutor` / `FakeCall` | struct | Scripted argv expectations, call recording | `lib.rs:1049`, `lib.rs:1040` |
| `SystemdScope` / `SystemdInstall` / `SystemdManager` | enum/struct/struct | systemd adapter, explicit scope, caller-owned unit bytes | `lib.rs:1294`, `1312`, `1384` |
| `LaunchdDomain` / `LaunchdInstall` / `LaunchdManager` | enum/struct/struct | launchd adapter, explicit `gui/<uid>` vs `system` | `lib.rs:2229`, `2238`, `2329` |
| `CronOwnership` | **type alias** | Alias of the canonical `Ownership` for a marker block | `lib.rs:2837` |
| `cron_classify` / `cron_merge` / `cron_remove` | fn | Pure block classification and edit | `lib.rs:2897`, `2925`, `2970` |
| `CronManager` | struct | Managed `# BEGIN/# END` block in the user crontab | `lib.rs:3022` |
| `CandidateManager` / `HostFacts` / `inspect_host` / `candidate_managers` | enum/struct/fn/fn | Detection facts separated from selection policy | `lib.rs:3297`, `3308`, `3323`, `3377` |

### `commit_engine.rs` — the single commit-and-rollback engine

Both orchestrators reach these mechanics; neither keeps a private copy. The
two behavioural differences between the former engine copies are named,
separately testable policy inputs on `CommitEnginePolicy`, both defaulting to
the **stricter** of the two former behaviours.

| Item | Kind | Role | Ref |
|---|---|---|---|
| `LifecycleUpdatePhase` | enum (`#[non_exhaustive]`) | `Inspect \| Quiesce \| Commit \| RestoreNew \| PostInstallCheck \| QuiesceForRollback \| RestoreOld \| FinalInspect` | `commit_engine.rs:52` |
| `LifecycleFailure` | struct | Bounded service-side evidence: phase, id, detail, optional ownership/state | `:73` |
| `PostInstallCheckError` | struct | Bounded, control-free failure detail | `:106` |
| `PostInstallCheck` | trait | Caller check receiving the remaining shared budget | `:123` |
| `NoPostInstallCheck` | struct | No-op check | `:138` |
| `PostTransitionVerification` | enum (`#[non_exhaustive]`) | `Verify` (default) \| `TrustManagerReport` | `:159` |
| `OwnershipLossTolerance` | enum (`#[non_exhaustive]`) | `Deny` (default) \| `TolerateAlreadyStopped` | `:180` |
| `CommitEnginePolicy` | struct | The two named policy inputs | `:201` |
| `remaining` | fn | The one deadline helper | `:228` |
| `bounded_detail` | fn | The one detail-truncation rule (`MAX_DIAGNOSTIC_BYTES`) | `:236` |
| `safe_inspect` | fn | The one panic-safe inspection | `:245` |
| `transition_to` | fn | The one transition mechanic; validates the desired state **at the point of use** | `:293` |
| `safe_transition_to` | fn | The one panic-safe transition | `:373` |
| `validate_observed` | fn | Post-transition ownership/state validation | `:400` |
| `quiesce_service` | fn | The one pre-commit quiesce | `:425` |
| `quiesce_for_rollback` | fn | The one rollback quiesce (fail-closed on unobservable / non-owned) | `:478` |
| `restore_and_check` | fn | The one post-commit restore + caller check composition | `:561` |

### `lifecycle_update.rs` — composition over Core

| Item | Kind | Role | Ref |
|---|---|---|---|
| `LifecycleRestorationStatus` | enum | `Restored \| Failed \| NotAttemptedRecoveryRequired` | `lifecycle_update.rs:18` |
| `LifecycleUpdatePolicy` | struct | `restore`, `post_commit_failure`, three timeouts | `:29` |
| `LifecycleUpdatePolicy::validate` | fn | Rejects zero / >`MAX_TRANSITION_TIMEOUT` before any mutation | `:44` |
| `LifecycleUpdateReceipt` | struct | `before`, `final_snapshot`, Core receipt, restoration, four failure slots, `engine_policy` | `:60` |
| `LifecycleUpdateError` | enum (`#[non_exhaustive]`) | `InvalidPolicy \| Preflight \| Quiesce \| CoreBeforeReceipt` | `:112` |
| `commit_with_lifecycle` | fn | Quiesce → Core commit → restore → post-install check, over the unified engine | `:166` |
| `commit_with_lifecycle_with_engine_policy` | fn | The same, under an explicitly selected `CommitEnginePolicy` | `:192` |

### `disposition.rs` — M006 runtime authority

| Item | Kind | Role | Ref |
|---|---|---|---|
| `UpdateRuntimeDisposition` | enum | `ManagedRunning \| ManagedStopped \| DirectRunning \| Stopped \| ForeignPreserved` | `disposition.rs:35` |
| `KnownConfig` | enum | `Absent \| Present(PathBuf) \| Unknown` for registered config identity | `:50` |
| `DirectState` | enum | `Running \| Stopped \| Unknown` | `:61` |
| `DirectObservation` | struct | `state`, `exact_ownership_proven`, `executable`, `config` | `:72` |
| `UnixPlannerInput` | struct | Manager ownership/state + registered/selected config + direct observation | `:85` |
| `WindowsServiceState` | enum | `Running \| StartPending \| Stopped \| StopPending \| Unknown` | `:102` |
| `WindowsPlannerInput` | struct | `installed`, `executable_known`, `ownership`, `state` | `:117` |
| `plan_unix` | fn | Pure Unix decision matrix | `:129` |
| `plan_windows` | fn | Pure Windows SCM decision matrix | `:228` |
| `DirectRuntimeControl` | trait | Caller seam: `inspect`, `stop`, `start` | `:260` |
| `DispositionBaseline` | struct | Authority/state observed after preparation | `:271` |
| `commit_with_disposition` | fn | Barrier-enforcing orchestration over a `ValidatedTransaction`, over the unified engine | `:289` |
| `commit_with_disposition_with_engine_policy` | fn | The same, under an explicitly selected `CommitEnginePolicy` | `:328` |

### `windows_scm.rs` — native SCM adapter

| Item | Kind | Role | Ref |
|---|---|---|---|
| `WindowsStartType` | enum | Caller-owned start type | `windows_scm.rs:29` |
| `WindowsErrorControl` | enum | Caller-owned error control | `:47` |
| `WindowsServiceDependency` | enum | `Service(name) \| Group(name)` | `:67` |
| `WindowsScmInstall` | struct | Service id, display name, start/error control, account, dependencies, timeout | `:90` |
| `WindowsScmInstall::{new,with_dependencies,with_transition_timeout}` | fn | Validating builders | `:105`, `:144`, `:159` |
| `WindowsScmManager` | struct | `ServiceManager` over a boxed `ScmBackend` | `:388` |
| `WindowsScmManager::new` | fn | `#[cfg(windows)]`; uses the local SCM | `:404` |

Internal bounds: `MAX_SCM_NAME_CHARS = 256`,
`MAX_SCM_COMMAND_CHARS = 8192`, `DEFAULT_SCM_TIMEOUT = 30s`,
`DEFAULT_POLL_INTERVAL = 50ms`, `MAX_POLL_INTERVAL = 500ms`
(`windows_scm.rs:13-17`).

Internal error plumbing (crate-private, `#[cfg(any(windows, test))]`):
`OpenFailure` (`:299`) names the outcome of a service-handle open —
`NotFound` (SCM `1060`), `MarkedForDelete` (SCM `1072`), or `Other(code)`
carrying any other OS code verbatim; `classify_open_code` (`:315`) maps a raw
code onto it; `unmapped_open_error` (`:336`) and `open_failure` (`:357`) turn
it into the typed `ServiceError`. The two codes that carry a meaning about
the registration keep their exact meanings, and every other code becomes a
`ServiceError::Manager` that still carries the raw OS code. No new error
variant and no new taxonomy is introduced: the existing
`ServiceError::Manager` / `ServiceError::Conflict` constructors and the
existing `ManagerFailureKind` set carry them.

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
| `LifecycleSnapshot` (`lib.rs:240`) | `id`, `ownership`, `state`, `health`, `was_registered`, `was_running` | Point-in-time observation. `was_registered` / `was_running` are the *only* record of pre-update intent |
| `LifecycleSnapshot::restore_running` (`lib.rs:257`) | — | Returns `was_running`. Distinguishes "was running" from "registered but stopped" |
| `RestoreIntent` (`lib.rs:264`) | — | Caller policy for the **new** generation only: `Preserve` / `EnsureRunning` / `EnsureStopped` |
| `TransitionResult` (`lib.rs:275`) | `operation`, `completed`, `detail` | `completed == false` is a *bounded outcome*, not an `Err`. Callers must branch on it |
| `LifecycleRestorationStatus` (`lifecycle_update.rs:18`) | — | Terminal lifecycle verdict, independent of Core's artifact disposition |

`RestoreIntent` deliberately does not govern the rollback path: after a
successful artifact rollback, orchestration restores `before.state`
regardless of intent (`lifecycle_update.rs:319-330`,
`disposition.rs:743-786`). A caller asking for `EnsureRunning` on an update that
rolled back gets the pre-update state, not a new start.

### `ServiceError`

`InvalidInput`, `OwnershipDenied { id, ownership }`, `Conflict`, and
`Manager { kind, detail, os_code }` (`lib.rs:406`). The `Manager` variant is a
**struct** variant: it carries the machine-readable
[`ManagerFailureKind`](#managerfailurekind), the bounded diagnostic detail, and
the platform OS code when the platform supplied a structured one.

There is **one** byte ceiling for manager-output diagnostic text:
`MAX_DIAGNOSTIC_BYTES` = **512 bytes**, truncated on a UTF-8 character
boundary (`lib.rs:912`). The crate's own constructors use it
(`ServiceError::invalid` / `bounded` at `lib.rs:433-439`;
`truncate_utf8_bytes` at `lib.rs:2224`), and the engine's failure-detail
truncation uses it too (`bounded_detail`, `commit_engine.rs:236`), which also
strips control characters. Manager stderr excerpts are folded into a `Manager`
error by `ensure_mutation_success` (`lib.rs:2235`) and are bounded by that
same ceiling — there is no second, smaller ceiling for the same data class.

Before M012 this data class carried two caps: 256 bytes for the stderr excerpt
and 512 bytes for the composed error, while the crate doc named only the 512.
See `plans/closure/service-lifecycle/012-status.md`.

Before M013 the `Manager` variant carried only free text, and the crate
classified its **own** failures by substring-matching that text:
`permission_hint` re-scanned the detail for `permission` / `denied` / `access`
and appended a remediation hint, and `inspect_host` matched `contains("missing")`
to decide "not installed". A consumer could not distinguish a timeout from a
missing binary without parsing the message. `permission_hint` is **gone**
(`lib.rs`), and the three `contains("missing")` sites now read the typed kind
(`lib.rs:3822`, `3838`, `3853`). See
`plans/closure/service-lifecycle/013-status.md`.

### `ManagerFailureKind`

`Spawn | Timeout | Permission | NotFound | Protocol | LocalFilesystem`
(`lib.rs:339`). Every `ServiceError::Manager` construction site in the crate
maps to exactly one kind; there is no "unclassified" fallback. The kind — not
the text — is the carrier of meaning, and it is what a consumer matches on
(`ServiceError::manager_kind`, `lib.rs:486`).

The classification is decided where the platform's structured evidence is still
available, by two **pure** mapping functions that need no real privilege
failure, no real spawn, and no wall-clock sleep:

| Mapping | Rule | Ref |
|---|---|---|
| `ManagerFailureKind::from_spawn_error` | `std::io::ErrorKind::NotFound` → `NotFound`; `PermissionDenied` → `Permission`; every other kind → `Spawn` | `lib.rs:363` |
| `ManagerFailureKind::from_scm_os_code` | Win32 code `5` → `Permission`; `1060` → `NotFound`; every other code (and `None`) → `Protocol` | `lib.rs:378` |

A manager that *runs* and reports an access denial in its own stderr is
`Protocol`, because the platform supplied no structured denial. Only a
structured platform denial is `Permission`.

The Windows SCM mapping stays **lossless**: the raw OS code is carried in the
error's `os_code` field and is reachable programmatically
(`ServiceError::manager_os_code`, `lib.rs:495`), and it is also rendered into
the bounded detail (`windows_scm.rs:215-224`). Service milestone `014` owns the
Windows SCM *public-surface* rustdoc; M013 changed only the SCM mapping's code,
not those doc lines.

## Ownership and authorization

### One canonical ownership type

`eggup_service::Ownership` **is** `eggup_core::Ownership`: the service crate
re-exports the canonical type (`lib.rs:165`) rather than defining its own.
`eggup_service::CronOwnership` is a public type alias of the same type
(`lib.rs:2837`). Before M012 three structurally identical enums existed across
two crates with no conversion between them.

The re-export's rustdoc states the service-context meaning (core's wording is
destination-oriented: in the service context "destination" is the registered
unit, launchd label, or SCM service the caller's identity claims), points at
core's canonical documentation, and states that `Owned` is never inferred and
that `Foreign` / `Unknown` fail closed. The fail-closed rule is stated **once**,
in the canonical type's own documentation.

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
| systemd | Non-zero `show` exit → `Unknown` / `Unknown` state, never `Absent` | `lib.rs:1447` |
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
   `Unknown`, not "nothing there" (`lib.rs:1447`).
3. **The mutation gate is a single function** — `require_owned`
   (`lib.rs:426-433`) accepts `Owned` and denies `Absent | Foreign | Unknown`
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
   `OperationDeadline` (`lib.rs:2143-2183`): it rejects zero and
   `> MAX_TRANSITION_TIMEOUT`, and every subsequent command, state poll, and
   re-inspection draws from `remaining()` — the budget only ever shrinks. The
   systemd polling loop sleeps
   `min(remaining, 100ms)`.

   **One deadline per operation.** Every adapter creates exactly *one*
   `OperationDeadline` at the start of each public operation and threads it
   through every manager invocation that operation makes
   (`operation_deadline`: systemd `lib.rs:1796`, launchd `lib.rs:2757`, cron
   `lib.rs:3484`). No invocation is granted a fresh full budget: the single
   initial clamp (`bounded_timeout`, `lib.rs:2185`) happens at deadline
   construction, and each call then receives the *remaining* budget.

   The budget for `start` / `stop` / `restart` is the caller-supplied `timeout`
   parameter. The budget for `install` / `uninstall` / `inspect`, which take no
   timeout parameter, is the adapter descriptor's `transition_timeout` for the
   **whole** operation. Before M013 the systemd adapter spent a fresh full
   budget on each of `ownership_of`, the ownership recheck, `daemon-reload`, and
   `enable`, so one systemd `install` could consume roughly 4-5x its declared
   budget; after M013 the aggregate wall time of one operation is bounded by one
   transition timeout.
3. **Non-zero exit is not success.** `ensure_mutation_success` requires
   `status == Some(0)` (`lib.rs:1859`). A `ManagerCommandOutput` with a
   non-zero status is *returned* by `SystemExecutor` as `Ok` so the caller can
   classify it, and it is the adapter's job to reject it.
4. **Confirm the resulting state.** A `completed: true` claim must be backed by
   an observation of the end state, not by the absence of an error.
5. **Bounded detail.** `TransitionResult::detail` goes through
   `ServiceError::bounded` (`MAX_DIAGNOSTIC_BYTES`, 512 bytes) in every in-crate
   implementation.
6. **No elevation.** Access denial is a classified failure, not an escalation.
   A structured platform denial is `ManagerFailureKind::Permission`; no adapter
   ever retries with elevated rights. The former text-matching remediation
   hint (`permission_hint`) was removed in M013: the kind carries the meaning,
   and the detail carries no re-derived advice.

`TestDoubleManager` satisfies all six deterministically and is the reference for
how the crate expects a test double to behave. It additionally treats
"already running" / "already stopped" as `completed: true` idempotent outcomes
(`lib.rs:550-560`, `574-584`) — a shape real adapters also implement
(`windows_scm.rs` test `stop_of_already_stopped_registration_is_complete_without_a_control`).

## Process execution: `SystemExecutor`

`CommandExecutor` (`lib.rs:961`) is the seam: argv in, `ManagerCommandOutput`
out, with `stdin_data: None` meaning null stdin and `Some(bytes)` meaning
bounded input (used only for `crontab -`). Missing binaries, timeouts, and
output overflows are `Err(ServiceError::Manager)` — carrying the classified
kind (`NotFound`, `Timeout`, `Spawn`, `Protocol`); **non-zero exits are `Ok`
with the observed status** (`lib.rs:968-973`), and it is the adapter's job to
classify them as `Protocol` (`ensure_mutation_success`, `lib.rs:2126`).

`SystemExecutor::run` (`lib.rs:1021-1141`) is a **thin policy layer over
`eggup_core::run_bounded`**. The bounded-execution mechanic — the spawn, the
per-stream capture bound, the unconditional deadline, the kill-and-reap
teardown, the dedicated stdin writer — lives in core. The service keeps every
policy decision in front of it: the argv emptiness and NUL checks, the
timeout range check, the `MAX_CRONTAB_BYTES` stdin bound, the allowlisted
absolute program path, and the filtered child environment. Core never sees the
raw `argv[0]`, never sees the ambient environment, and never decides that a
lock, a path, or a privilege is acceptable.

The mapping from core's result to the service's taxonomy is exhaustive and is
pinned by the classification surface core documents: `"spawning bounded
candidate"`, `"polling bounded candidate"`, and `"writing bounded stdin"`
(`crates/eggup-core/src/candidate.rs:307`, `:373`, `:439`). The match has an
explicit fallback arm that **names** an unrecognised operation, so a future
core change is attributable rather than silently folded into "spawn failed".

The output type is named `ManagerCommandOutput` because it is the output of one
*manager* command. `eggup_core` exports a different, unrelated `CommandOutput`;
before M012 both crates exported a public type under the same name, so a
consumer that glob-imported both crates and then named `CommandOutput`
unqualified was ambiguous. The former spelling survives as a public type
**alias** to `ManagerCommandOutput` (`lib.rs:694`), so existing explicit-path
callers and existing `CommandExecutor` implementations keep compiling and
still satisfy the trait. The alias is deliberately **not** marked
`#[deprecated]`: deprecation is a publication-policy decision owned by a later
republication milestone, and a deprecated alias would break downstream
`-D warnings` builds. Its disposition is settled at the intentional 0.2/1.0 API
boundary, alongside the already-registered `CleanupDisposition` finding in
`plans/registry.md` § "Post-0.1.3 open findings" item 2.

### Executable allowlisting and path resolution

`resolve_manager_program` (`lib.rs:1143-1169`) is the only path from a program name
to a binary:

| Requested | Resolution |
|---|---|
| Absolute path | Validated directly |
| `systemctl` | `/usr/bin/systemctl`, then `/bin/systemctl` |
| `launchctl` | `/bin/launchctl`, then `/usr/bin/launchctl` |
| `crontab` | `/usr/bin/crontab`, then `/bin/crontab` |
| Anything else | `Err(InvalidInput("manager program must be absolute or allowlisted"))` |

There is **no `PATH` lookup at all**. `validate_executable_path`
(`lib.rs:1171-1196`) additionally requires the path to be absolute, to exist, to
be a regular file (not a symlink-to-directory or device), and — under
`#[cfg(unix)]` — to have at least one execute bit (`mode & 0o111 != 0`).
All candidates failing validation yields `Manager("trusted manager binary missing")`.

The **resolved** path — never the raw `argv[0]` — is what the service hands to
core (`lib.rs:1062-1065`), and it is the resolved path that every failure
detail names. Core never resolves a program name.

### Environment clearing

The child environment is cleared unconditionally, and the only variables
re-added come from `filtered_manager_environment` (`lib.rs:1198-1218`), and only
when the argv is `systemctl` **with** `--user`:

`DBUS_SESSION_BUS_ADDRESS`, `XDG_RUNTIME_DIR`, `SYSTEMD_BUS_ADDRESS`.

Any other argv shape gets an empty environment. Tests
`command_environment_adds_nothing` (`lib.rs:4200`) and
`manager_resolution_ignores_ambient_path_and_environment_filters_sentinels`
(`lib.rs:4214`) pin both halves, and
`the_session_bus_filter_is_exactly_three_variables` (`lib.rs:4268`) pins the
allowlist to exactly those three names.

The filter yields `(OsString, OsString)` pairs, while core's
`CommandSpec::environment` takes `impl Into<String>`. A pair whose key **and**
value are valid UTF-8 is converted and passed through byte-identically; a pair
that is **not** valid UTF-8 is **dropped** rather than lossily converted
(`lib.rs:1070-1075`). A mangled session-bus address would be worse than no
session-bus address at all, so the fail-closed direction is to drop the pair.

### Output and timeout bounds

| Bound | Value | Enforced at |
|---|---|---|
| Per-stream captured output | `max_output_bytes` (default `MAX_COMMAND_OUTPUT_BYTES` = 256 KiB; `with_max_output` accepts `1..=16 MiB`) | `lib.rs:895`, `1013` |
| stdin payload | `MAX_CRONTAB_BYTES` = 1 MiB | `lib.rs:897` |
| command timeout | non-zero and `<= MAX_TRANSITION_TIMEOUT + 60s` | `lib.rs:901` |
| argv | non-empty, no NUL in any element | `lib.rs:1032`, `1035` |

Output overflow is **fail-closed**: core's bounded reader refuses a stream that
exceeds the bound, the captured bytes are discarded, and the service maps the
result to `Manager(Protocol, "manager output exceeded bound for {program}")`
(`lib.rs:1088-1093`). Unlike the hand-rolled runner this replaced, **the child
is now killed and reaped on overflow** — a still-running runaway child is
terminated instead of being left to finish on its own.

The deadline the caller hands to the service bounds the **whole** call: the
service starts its budget before the program resolution and gives core the
*remaining* budget (`lib.rs:1049-1060`). If the remaining budget is already
zero the call returns the timeout failure **without spawning anything**.

### The stdin deadlock fix

The stdin payload is attached to core's spec, and core writes it on a
**dedicated writer thread** with the readers already attached, so a child that
fills stdout before draining stdin cannot deadlock both sides. The
deadline is unconditional and bounds the writer exactly as it bounds the
readers: at the deadline the child is killed, which closes the pipe and
unblocks the writer. A *completed but failed* write fails closed with
`Manager(Spawn, "manager stdin failed for {program}")` — the same kind and the
same detail the hand-rolled runner produced.

### No shell interpolation

There is no `sh -c`, no argument stringification, and no `PATH`-mediated
resolution. Each argv element after the program is passed as one literal
argument (`lib.rs:1063`). `ServiceSpec::new` additionally rejects control
characters in args (`lib.rs:91-95`) and `run` rejects NUL in any argv element
(`lib.rs:1035`), which is the argv-level analogue of a shell-metacharacter
rejection. The same executor backs cron, where the desired block *is* a shell
line — that is caller-owned content written to the crontab, never
interpolated into a command this crate builds.

### Behaviour differences of the consolidation

The consolidation preserves every observable outcome except the eight recorded
here. Seven are the honest cost of moving the mechanic into core, and each is
pinned by a test in `crates/eggup-service/tests/bounded_executor_consolidation.rs`
except **D7**, which has no portable fixture and is recorded in
`plans/closure/service-lifecycle/015-status.md`. The eighth, **D8**, is not a
cost: it is a **defect in the runner this consolidation replaced**, which the
consolidation fixes.

| # | Difference | Direction |
|---|---|---|
| **D1** | A mid-stream reader error is now fail-closed. The hand-rolled reader broke out of the read loop and reported a successful capture of partial output; core surfaces a reader error as `output_limited`, which maps to `Manager(Protocol, "manager output exceeded bound for {program}")`. The detail says "exceeded bound" for what is really a read failure — a cosmetic imprecision inherited from the mapping. | Fail-closed (stronger) |
| **D2** | Core kills the child on an output overflow; the hand-rolled runner did not, because the overflow was only detected after the child had already exited. The observable result — a `Protocol` manager failure — is unchanged. | Strictly safer |
| **D3** | The timeout detail text is normalised. The hand-rolled runner could produce `"manager command timed out for {program}"`, `"manager output collection timed out"`, or `"manager command timed out"`; all of them now read `"manager command timed out for {program}"`. The machine-readable discriminator — `ServiceError::manager_kind() == Some(Timeout)` — is unchanged. | Detail text only |
| **D4** | A descendant that holds the stdout/stderr pipe open now ends at the deadline with the *timeout* detail rather than `"manager output collection timed out"`. Core also abandons the still-blocked reader explicitly instead of joining it. | Same kind, different detail text (D3 covers the text) |
| **D5** | A non-UTF-8 environment pair is dropped rather than passed through byte-exactly, because core's setter takes `impl Into<String>`. A valid UTF-8 value is unaffected. | Narrowing, fail-closed |
| **D6** | The spawn-failure `NotFound` detail names the **resolved** program, while the pre-flight `validate_executable_path` `NotFound` names the **requested** one. Both survive the consolidation unchanged. | No change |
| **D7** | A stdin writer that is still blocked when the deadline passes is now reported as a **timeout**, not as a stdin failure. Reaching it requires a descendant to hold the child's stdin pipe open after the child itself exits. In the two ordinary cases — the broken pipe (both report `Spawn`) and the child still running at the deadline (both report `Timeout`) — the kinds are identical. | Kind change, in a deep corner |
| **D8** | **A successful stdin write is no longer reported as a stdin failure.** The replaced runner polled its writer channel with `try_recv()` on every wait-loop iteration; `try_recv` *removes* the message, so a verdict of `Ok(())` was consumed and discarded by a check that only looks for `Err(_)`. The post-exit check then found an empty, disconnected channel and reported `Manager(Spawn, "manager stdin failed for {program}")` for a write that had **succeeded**. Core keeps the verdict and reports the failure only when the write itself failed. The affected path is the `crontab -` install, the crate's only production stdin caller. | **Defect fixed** (a failure becomes a success) |

### `FakeExecutor`

`FakeExecutor` (`lib.rs:1239`) scripts expectations as a FIFO of exact argv
vectors and records every observed call with its stdin. A mismatch or an
unscripted call is a hard `Err` (`lib.rs:1313-1342`), so **adapter tests fail if
the adapter issues an unexpected command** — this is the mechanism that keeps
the adapter command surface under test. `is_exhausted()` (`lib.rs:1308`) lets a
test assert every scripted command was consumed. It also implements
`CommandExecutor` for `&FakeExecutor` (`lib.rs:1344`) so it can be shared with
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

`commit_with_lifecycle` (`lifecycle_update.rs:166`) takes an **already
validated** `ValidatedTransaction` and wraps Core's
`commit_with_post_commit` with service quiescence and restoration. It never
installs or refreshes a registration.

The orchestration below is a *policy* over the unified engine in
`commit_engine.rs`: the mechanics it names — `safe_inspect`, the quiesce stop,
the rollback quiesce, the panic-safe restore — have one implementation, and
`commit_with_disposition` runs the same mechanics. The two behavioural inputs
that used to differ between the two former engine copies are named policy
inputs on `CommitEnginePolicy` (`commit_engine.rs:201`), both defaulting to the
stricter behaviour:

| Input | Default | Non-default input |
|---|---|---|
| `PostTransitionVerification` | `Verify` — re-inspect and validate ownership/state after every transition | `TrustManagerReport` — accept the manager's own completion report |
| `OwnershipLossTolerance` | `Deny` — an ownership that is no longer `Owned` is always a typed failure | `TolerateAlreadyStopped` — tolerate it when the unit is already in the wanted stopped state |

`commit_with_lifecycle` runs the engine under the default policy;
`commit_with_lifecycle_with_engine_policy` (`lifecycle_update.rs:192`) selects
the inputs explicitly, and the selection is observable in the receipt's
`engine_policy` field.

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
| `policy.validate()` | `LifecycleUpdateError::InvalidPolicy { field }` | One of the three timeouts is zero or `> MAX_TRANSITION_TIMEOUT` (`lifecycle_update.rs:44-53`) |
| Inspect gate | `Preflight` | `Absent`, `Foreign`, `Unknown`, or `Transitioning`/`Unknown` state (`lifecycle_update.rs:177-191`) |
| Inspect | `Preflight` | A panicking manager is contained by `safe_inspect` and reported as a `LifecycleFailure` |
| Quiesce stop | `Quiesce { failure, restore_failure }` | Manager error, panic, `completed == false`, or post-stop ownership/state drift. Restoration to `Running` is attempted and its failure is reported **separately** |
| Post-commit work | `LifecycleFailure` + Core's `post_commit_failure` policy | `RestoreNew` or `PostInstallCheck` phase; on `RollBack` a `QuiesceForRollback` failure is recorded separately |
| Core before receipt | `CoreBeforeReceipt { core, restore_failure }` | Only lock contention / setup failures. Pre-update state is restored if the service had been quiesced |
| Final inspect | **Not an error** | `final_snapshot: None` plus `observation_failure: Some(..)`; Core's disposition is unchanged |

### The restoration contract

- `Committed`: the new generation is restored to `wanted` derived from
  `policy.restore`. If the failure phase was `RestoreNew`,
  `restoration = Failed` (`lifecycle_update.rs:311-318`).
- `RolledBack`: the **pre-update** state (`before.state`) is restored,
  overriding `policy.restore` (`lifecycle_update.rs:319-330`).
- `RecoveryRequired`: `restoration = NotAttemptedRecoveryRequired` and **no
  start is attempted** — the artifacts are uncertain, so Eggup refuses to run
  them (`lifecycle_update.rs:331-333`).

The post-install check receives the *remaining* budget of the single
`post_commit_timeout` deadline (`commit_engine.rs:561`, and the check call
site). `PostInstallCheck::check` documents that it must return within
`remaining` and that it cannot be forcibly cancelled
(`commit_engine.rs:123-135`). The existing `HealthProbe` trait has **no
timeout contract** and therefore cannot satisfy this bound on its own; health
semantics stay caller-owned.

Panic attribution is a specific review concern. `safe_restore_state` catches
each transition's panics at the phase actually executing, so the outer
post-commit `catch_unwind` arm can only be reached by the check itself
(CHANGELOG, Unreleased; test
`panicking_restore_transition_is_recorded_as_restore_new_not_a_check`,
`lifecycle_update.rs:1291`).

## Runtime disposition (M006) — the review focus

`disposition.rs` exists because `commit_with_lifecycle` answers only one shape
of question: "quiesce my owned service, commit, put it back." Real deployments
have more cases — a foreign registration that must not be touched, a program run
directly with no manager, or nothing running at all. M006 generalizes that
without loosening the safety posture.

### The central distinction

> **Artifact commit authority and manager mutation authority are separate
> facts.** (`disposition.rs:35`)

Core decides whether the artifact bytes may be committed. The disposition decides
whether Eggup may call a service manager at all. `ForeignPreserved` commits
artifacts and performs **zero** manager mutation. `DirectRunning` mutates only
through a caller-owned seam that Eggup does not interpret. Neither conclusion
implies the other, and `LifecycleUpdateReceipt` keeps them apart:
`artifact_disposition()` and `service_state_restored()` are separate accessors
(`lifecycle_update.rs:177-191`).

### `UpdateRuntimeDisposition`

| Variant | Meaning | Manager mutation | Artifact commit |
|---|---|---|---|
| `ManagedRunning` | Owned manager, running | `stop` before, restore/start after | Yes |
| `ManagedStopped` | Owned manager, stopped | Restore per intent only | Yes |
| `DirectRunning` | Selected direct runtime, running | **None** — only `DirectRuntimeControl` | Yes |
| `Stopped` | No running runtime | None | Yes, with no fabricated restart |
| `ForeignPreserved` | Foreign manager | **None**, ever | Yes |

`disposition.rs:35-46`.

### `DirectRuntimeControl`

`disposition.rs:260-267`. Three methods: `inspect`, `stop`, `start`. It is a
**caller-supplied trait**, and the crate's own comments treat that as a
responsibility transfer, not a delegation: there is no second ownership check
inside `stop`, so Eggup performs every identity verification itself before
calling it (`disposition.rs:1013-1050`). The seam carries no health-protocol
content — `DirectObservation` is a state plus identity facts
(`exact_ownership_proven`, `executable`, `config`), not a health channel.

### The pure planners

`plan_unix` and `plan_windows` are total, side-effect-free functions over
caller-supplied facts. They exist so the decision matrix can be tested without a
native manager, and so a caller can compute and record a disposition *before*
committing anything.

#### `plan_unix` decision table

`disposition.rs:129-225`. First gate: if `manager_state_known == false`
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

`disposition.rs:228-257`. The Windows matrix has no direct-runtime concept.

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

`commit_with_disposition` (`disposition.rs:289-591`) enforces three gates before
any mutation, in this order:

1. **Policy validation** (`disposition.rs:346`) — same zero / over-ceiling rules
   as the lifecycle path.
2. **Baseline coherence** (`disposition.rs:372-377`) — the `planned`
   disposition must actually match the `DispositionBaseline` observed after
   preparation. `ManagedRunning` requires `Owned` + `Running`;
   `ManagedStopped` requires `Owned` + `Stopped`; `DirectRunning` requires a
   baseline direct observation that is `Running` with
   `exact_ownership_proven`; `ForeignPreserved` requires a non-`Owned` manager.
   `Stopped` is unconditionally coherent because it mutates nothing.
3. **Fresh re-inspection** (`disposition.rs:378-392`) — the manager is
   re-inspected and both `ownership` **and** `state` must equal the baseline.
   Any change is `Preflight("lifecycle authority/state changed before mutation")`.
4. **Direct revalidation** (`disposition.rs:393-453`), only when
   `planned == DirectRunning` — `direct.inspect()` must return a value
   **equal** to the baseline observation, with `exact_ownership_proven` true and
   state `Running`. This catches an executable/config identity change, not just
   a state change.

Only after all gates does the `match planned` dispatch run
(`disposition.rs:455`). `ManagedRunning` and `DirectRunning` quiesce and then
re-verify the stopped state before handing the transaction to Core;
`ManagedStopped`, `Stopped`, and `ForeignPreserved` go straight to
`commit_manager_path` with `pre_commit_quiesced = false`
(`disposition.rs:557-591`).

**This is the revalidation barrier, and it is the point of the module.** The
planning phase happens before `prepare`; the baseline is captured after
preparation; the barrier re-observes authority immediately before the first
destructive call. A caller that plans, prepares, waits a long time, and then
commits an owned service that has since been replaced is denied before the stop,
not after.

### Post-commit and rollback behavior per disposition

`commit_manager_path` (`disposition.rs:593-741`) drives
`transaction.commit_with_post_commit`. `restore_manager_success`
(`disposition.rs:743-786`) resolves the target state:

| Planned | Target state |
|---|---|
| `ForeignPreserved` | **No transition at all.** Inspect (read-only) then the post-install check |
| `Stopped` | `EnsureRunning` → `Running`; otherwise `Stopped` |
| any other | `Preserve` → `before.state`; `EnsureRunning` → `Running`; `EnsureStopped` → `Stopped` |

On `PostCommitFailurePolicy::RollBack`, `quiesce_manager_for_rollback`
(`commit_engine.rs:478-553`) runs before Core rolls the artifacts back. It fails
closed in three ways worth naming: an **inspect error is treated as
unobservable, not as "proceed"** (`commit_engine.rs:492-501`); a non-`Owned`
service is refused (`commit_engine.rs:509-518`); and the stopped state is
re-confirmed by a second `inspect` after the `stop`
(`commit_engine.rs:532-545`). Tests
`unobservable_service_is_not_stopped_before_rollback` and
`unobservable_direct_runtime_is_not_stopped_before_rollback`
(`disposition.rs:1774`, `1745`) pin both.

`commit_direct_path` (`disposition.rs:788`) is the mirror image for
`DirectRunning`, with one structural difference: the direct `start` is wrapped in
a **nested** `catch_unwind` labelled `RestoreNew`, so the outer arm
(`PostInstallCheck`) is reachable only by the check itself
(`disposition.rs:821-869`). The check snapshot comes from a read-only manager
inspect, because direct state is caller-owned and is deliberately not part of
`LifecycleSnapshot` (`disposition.rs:853-855`).

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
   (`disposition.rs:163-164`).
3. **Prepare.** The caller builds the `ValidatedTransaction` via
   `eggup-core`, then observes the baseline into `DispositionBaseline`:
   `manager_ownership = Foreign`, `manager_state = Running`,
   `direct = None`.
4. **Enter `commit_with_disposition`.** `policy.validate()` passes. The baseline
   gate accepts `Foreign` (`disposition.rs:380-383`). The fresh `inspect` must
   still report `Foreign` + `Running`. The direct revalidation block is skipped —
   `planned != DirectRunning`, so a missing `direct` handle is not an error.
5. **Dispatch.** The `match planned` arm falls through to `commit_manager_path`
   with `pre_commit_quiesced = false` (`disposition.rs:557-591`). **No `stop`,
   no `start`, no definition write.** The test
   `foreign_preserved_performs_zero_manager_mutation`
   (`disposition.rs:1979`) asserts exactly this.
6. **Commit and restore.** Core commits the artifacts. `restore_manager_success`
   short-circuits on `ForeignPreserved` (`disposition.rs:758`): one read-only
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
   (`disposition.rs:393-402`).
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
exact desired body.

**One rule, stated once.** `CronManager::cron_ownership` (`lib.rs:3499`) is the
single statement of the rule, and `inspect`, `install`, and `uninstall` all use
it. It delegates to `cron_classify` and nothing else: ownership is decided
**only** by exact content identity of the managed block body.

| `CronOwnership` | Condition | Behavior |
|---|---|---|
| `Absent` | no block | install appends; uninstall is a no-op without a rewrite |
| `Owned` | exactly one block, body equals desired | idempotent; `changed = false` |
| `Foreign` | one block, different body | `ServiceError::denied` — never overwritten |
| `Unknown` | multiple blocks, or an unterminated `BEGIN` | `ServiceError::Conflict` |

**The executable-containment gate** (`require_consistent_request`,
`lib.rs:3510`) is a separate, explicit check applied *before any listing or
mutation*: the caller's managed block body must contain `spec.executable()`.
If it does not, `inspect`, `install`, and `uninstall` all refuse with
`ServiceError::InvalidInput`. The gate fails closed — it can only refuse, never
widen.

Before M013 the rule existed in **two** disagreeing copies: one used
`current.contains(exe) || desired.contains(exe)` and the other used
`text.contains(exe)` — whole-crontab containment, which an unrelated user
crontab entry mentioning the executable path could satisfy. The
whole-crontab containment test is gone: ownership is exact block identity
only. Before M013 `inspect` also spawned `crontab -l` **twice** and discarded
one result; it now lists exactly once (`lib.rs:3520`).

`cron_classify` / `cron_merge` / `cron_remove` are pure and preserve unrelated
bytes, including the trailing-newline convention. The command adapter re-reads
the crontab immediately before a destructive write to surface races, and skips
a redundant `crontab -` write when `changed == false`. The `Foreign` and
`Unknown` errors are attributed to a **synthetic** `cron:<marker>` id, not to
the caller's spec id, because the marker — not the spec — owns the block.

### Windows SCM (`WindowsScmManager`, `windows_scm.rs:388`)

Uses the safe `windows-service` wrapper behind a boxed `ScmBackend`
(`windows_scm.rs:389`), which is how the adapter is tested deterministically off
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
  (`windows_scm.rs:279-288`) special-cases code `5` into a "run with an account
  authorized for this service operation" message; there is no elevation path
  (test `windows_scm.rs:1252`).
- **Every SCM condition is a typed error; none panics.** The service-handle
  open (`windows_scm.rs:1149`) classifies the SCM's failure codes through
  `OpenFailure` (`windows_scm.rs:299`): `1060` (does not exist) and `1072`
  (marked for deletion) keep their exact meanings — `Absent`,
  `MarkedForDelete`, and the "the service disappeared before the operation"
  conflict — and any other code becomes a `ServiceError::Manager` that still
  carries the raw OS code. The five former `unreachable!()` arms are gone; the
  lossless `1060`/`1072` mapping is unchanged.
- **Refresh preserves what the adapter does not own** — service type,
  unspecified dependencies, account, load-order group, tag.
- **Account selection is create-time only.** Custom passwords, descriptions, and
  recovery actions are out of scope for this generic adapter.
- `validate_spec` (`windows_scm.rs:260-303`) additionally requires the spec id to
  match the install descriptor, the executable to be a Windows-absolute path
  representable as UTF-8, and the assembled command line to fit
  `MAX_SCM_COMMAND_CHARS` (8192 UTF-16 units).
- **The native backend is exercised against the real SCM.** The fixture
  `crates/eggup-service/tests/windows_scm_live.rs` is Windows-only
  (`#![cfg(windows)]`) and registers a **disposable** service through the
  crate's **public** API — `WindowsScmInstall` plus `WindowsScmManager::new`,
  never a crate-internal back door — then drives install → inspect → ownership
  → start → stop → uninstall against the genuine Service Control Manager. It
  asserts the `windows-service` mapping layer explicitly (start type, error
  control, service dependencies, and the `ServiceInfo` round-trip), removes
  its own registrations on every exit path including a failing assertion,
  never touches a pre-existing service, waits only under explicit deadlines
  and fails with the observed state on timeout, and skips with a **recorded
  reason** when the runner lacks the administrator rights a real registration
  needs. On a non-Windows host the file is compiled out, so the fixture is
  **not run** there and is never inferred from another platform.

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
| Manager command returns non-zero | `Err(ServiceError::Manager { kind: Protocol, .. })` with bounded stderr | `ensure_mutation_success`, `lib.rs:2235` |
| Manager command times out | `Err(ServiceError::Manager { kind: Timeout, .. })` | `OperationDeadline::command_timeout`, `lib.rs:2167` |
| Manager binary or named manager resource is absent | `Err(ServiceError::Manager { kind: NotFound, .. })` | `resolve_manager_program` / `validate_executable_path`, `lib.rs:1227`, `1255` |
| Platform reports an access denial | `Err(ServiceError::Manager { kind: Permission, .. })` | `ManagerFailureKind::from_spawn_error`, `lib.rs:363` |
| Windows SCM failure | `Err(ServiceError::Manager { kind, detail, os_code })` — the raw OS code stays reachable | `ManagerFailureKind::from_scm_os_code`, `lib.rs:378` |
| Manager command errors during inspect | Treated as `Unknown`, not `Absent` | `lib.rs:1447`; rollback quiesce, `commit_engine.rs:492` |
| Output exceeds the bound | `Err(Manager("manager output exceeded bound"))` | `read_bounded_generic`, `lib.rs:949`; checked `lib.rs:838` |
| Transition deadline expires | `Err(Manager("transition deadline exhausted before manager command"))` or `TransitionResult { completed: false }` | `OperationDeadline`, `lib.rs:1472`; `wait_for_state`, `windows_scm.rs:351` |
| Zero or overlong policy timeout | `LifecycleUpdateError::InvalidPolicy { field }` before any mutation | `LifecycleUpdatePolicy::validate`, `lifecycle_update.rs:44` |
| Post-install check fails | Core's `KeepInstalled` / `RollBack` policy applied; failure recorded with its phase | `commit_engine.rs:561-604` |
| Core reports `RecoveryRequired` | `NotAttemptedRecoveryRequired`; no automatic start | `lifecycle_update.rs:334`; `disposition.rs:735`, `1185` |
| Authority changes between planning and mutation | `Preflight` before the first destructive call | `disposition.rs:378-392` |
| Direct identity changes before mutation | `Preflight` before the first destructive call | `disposition.rs:432-444` |
| Planned disposition contradicts the baseline | `Preflight("planned disposition does not match baseline authority/state")` | `disposition.rs:372-377` |
| `DirectRunning` with no `DirectRuntimeControl` handle | `Preflight` | `disposition.rs:393-402` |
| Unobservable runtime before rollback quiesce | Refused; `QuiesceForRollback` failure recorded | `commit_engine.rs:492-501`, `disposition.rs:1542` |
| Manager or check panics | Contained at the executing phase, not the outer arm | `safe_inspect` / `safe_restore_state` / `safe_transition_owned_to` |
| Definition parent missing or a symlink | `InvalidInput`; parents are never created | `atomic_write_definition`, `lib.rs:1100-1109` |
| Definition destination exists, `allow_overwrite = false` | `Err(Conflict)`; no write | `lib.rs:1111`, `1225` |
| Access denied by a manager | Bounded remediation hint, no elevation | `permission_hint`, `lib.rs:1541`; `scm_error_detail`, `windows_scm.rs:215` |
| Foreign cron marker | `OwnershipDenied` on synthetic `cron:<marker>` id; no overwrite | `cron_merge` / `cron_remove`, `lib.rs:2650`, `2701` |
| Duplicate / unterminated cron marker | `Conflict`; no write | `find_blocks` + `cron_classify`, `lib.rs:2565`, `2611` |
| Shell metacharacters in spec args | Rejected at `ServiceSpec::new` (control chars) and at `run` (NUL) | `lib.rs:91`, `706` |

## Testing approach

111 `#[test]` functions across the five source files, plus the integration
test targets under `crates/eggup-service/tests/`. The unit tests are
in-module; the integration targets are separate crates, and the package
excludes `tests/` and `benches/` from the published artifact
(`Cargo.toml` excludes `tests/` and `benches/` from the package). The
distribution:

| Area | Tests | Demonstrates |
|---|---|---|
| Neutral model (`lib.rs:3093+`) | ~13 | Ownership classification, snapshot intent preservation, health/state separation, denial of `Foreign` / `Unknown`, `RestoreIntent` |
| `SystemExecutor` (`lib.rs:4101-4212`) | 6 (Unix-only) | Timeout kills and reaps, output bound fails closed, literal argv with no shell, non-zero exit is `Ok` with status, missing binary fails closed, env adds nothing. These six use POSIX absolute paths, so they are `#[cfg(unix)]`-gated per test and each has a portable counterpart in the integration targets below. |
| Session-bus filter (`lib.rs:4268`) | 1 | The user-scoped environment filter is exactly three variables, on every platform |
| systemd (`lib.rs:3520-3850`) | ~11 | Absent / exact-owned / foreign-executable / argv-drift / ambiguous-`ExecStart` / specifier-`ExecStart` / config-identity / state mapping / permission-without-`sudo` / atomic-write fault preserving the old file / bounded start-stop-restart |
| launchd (`lib.rs:3851-4116`) | ~7 | Absent/exact/foreign/unknown, user-vs-system domain distinctness, malformed plist, bootstrap/bootout/kickstart, restart-after-incomplete-stop, label-only is not ownership, config reconciliation |
| cron (`lib.rs:4117-4271`) | ~9 | Empty-to-absent-to-install, byte-for-byte preservation of unrelated entries, idempotence, duplicate marker, modified block is foreign, owned-only removal, no redundant write, timeout/missing binary, no start/stop semantics |
| Definition + detection (`lib.rs:4271-4457`) | ~6 | Atomic private no-clobber write, missing parent rejected, unavailable vs failure distinguished, candidate policy order, native `systemctl` / `launchctl` / `crontab -l` smoke, consumer-specific constants rejected |
| `lifecycle_update.rs` | 15 | Policy bounds, restore-intent matrix, preconditions, quiesce failures, `KeepInstalled` evidence retention, rollback quiesce-then-restore, caller-check panic, restore-phase panic attribution, `CoreBeforeReceipt` restore, post-rollback restore failure separation, `RecoveryRequired` suppression, single shared deadline, no hidden policy retry |
| `disposition.rs` | 17 | Unix reference matrix, Windows reference matrix, unobservable-runtime refusal, ordering per disposition, zero mutation under `ForeignPreserved`, direct seam exclusivity, revalidation failure modes, `KeepInstalled` composition, `RecoveryRequired` suppression, no product terminology in public types, prepare-before-quiesce, rollback quiesce + old restore |
| `commit_engine.rs` | 1 | The single engine's own unit coverage |
| `windows_scm.rs` | 16 | Name/descriptor validation, access-denied remediation without escalation, Windows command-line parsing (quotes, backslashes, unicode) and its rejections, config-identity arity, explicit lifecycle mapping for every SCM state, absent/exact identity, spec validation, create/refresh, one-deadline transitions, restart safety, `MarkedForDelete` honesty, idempotent stop, and the typed-error contract for the five former panic sites |

The integration test targets under `crates/eggup-service/tests/`:

| Target | Tests | Demonstrates |
|---|---|---|
| `api_compatibility.rs` | 1 | The retained `LifecycleState` variant-pinning fixture. It is folded into `public_api_surface.rs` as a module, so it is compiled by both targets and is **not** deleted as redundant. |
| `public_api_surface.rs` | 4 | The public-API surface guard: every public item is bound to an explicit type, so a removed item or a changed signature is a **compilation** error. It also asserts that the crate's own sources still declare exactly the pinned number of top-level public items and `pub use` re-exports, and that the committed snapshot `tests/public-api-surface.txt` names every one of them. The snapshot is maintained by hand — the only tool that can emit a machine-readable public surface from Rust sources needs a nightly compiler, so the pinning test is the enforcement and the snapshot is the review artifact. No new dependency. |
| `windows_scm_live.rs` | 2 | The **real** Windows SCM backend: a disposable registration driven through install → inspect → ownership → start → stop → uninstall against the genuine Service Control Manager, with the `windows-service` mapping layer asserted explicitly. Windows-only (`#![cfg(windows)]`); **not run** on any other platform, and never inferred from one. |
| `errors_doc_sections.rs` | 4 | The `# Errors` / `# Panics` doc sweep for `lib.rs`, `disposition.rs`, and `lifecycle_update.rs`. The Windows SCM surface is deliberately excluded: milestone `014` owns it. |
| `manager_failure_kinds.rs` | 14 | The `ManagerFailureKind` taxonomy and its negative controls |
| `cron_single_listing_and_gate.rs` | 8 | The cron single-listing rule and the executable-containment gate |
| `commit_engine_policy.rs`, `commit_engine_parity.rs` | 7 | The single engine's policy inputs and the parity of the two orchestrators |
| `manager_deadline_budget.rs`, `manager_detail_byte_cap.rs` | 4 | The one-deadline budget and the one byte ceiling |
| `systemd_failed_service.rs` | 1 | A real systemd `failed` unit, stop, and quiescence |
| `bounded_executor_consolidation.rs` | 12 | The consolidation's observable outcomes, against a portable child fixture: the status and bounded output of a run, the stdin payload delivered byte-for-byte, the literal argv with no shell, the four pre-flight rejections with their messages, the typed missing-program and spawn failures, the timeout kind, the output-overflow kind, the nonzero exit, and the golden argv every manager builds. The child program is the test binary itself, re-executed with a mode argument, so the fixture is portable and needs no shell, no interpreter, and no manifest change. |
| `bounded_executor_environment.rs` | 2 | The child environment is cleared, and only the three session-bus variables are injected for user-scoped systemd. A dedicated binary, so `std::env::set_var` cannot race another test. The `--user` injection case is Unix-only, because only Unix can run a program named `systemctl`; its portable counterpart is the crate-private filter assertion in `lib.rs`. |

The three tests CI runs individually on Windows are pinned because they assert
UTF-8 boundary behavior that a differently-configured default run would not
exercise (`.github/workflows/ci.yml:124-126`):

```sh
cargo test -p eggup-service --lib diagnostic_byte_bounds_preserve_utf8_at_256_and_512_edges --locked
cargo test -p eggup-service --lib service_errors_output_and_permission_remediation_stay_utf8_bounded --locked
cargo test -p eggup-service --lib lifecycle_failure_detail_is_utf8_safe_bounded_and_control_free --locked
cargo test -p eggup-service --lib windows_scm::tests --locked
```

The first two are in `lib.rs:3430` and `lib.rs:3450`; the third is
`lifecycle_update.rs:420`. The fourth runs the whole SCM test module, whose
platform-independent tests (parser, ownership, lifecycle mapping) run on every
host — the `#[cfg(windows)]` backend is the only part that is Windows-only.

Note the shape choice: `no_gregg_strings_in_public_types`
(`disposition.rs:2008`) asserts that the M006 public types contain no reference
to the behavioral oracle the milestone was qualified against, keeping the
surface product-neutral. That is a real regression guard, not a naming
preference.

Focused commands for review:

```sh
cargo test -p eggup-service --locked
cargo test -p eggup-service --lib commit_with_disposition --locked
cargo test -p eggup-service --lib commit_with_lifecycle --locked
cargo test -p eggup-service --lib plan_ --locked
cargo test -p eggup-service --test public_api_surface --locked
cargo test -p eggup-service --test windows_scm_live --locked -- --nocapture
cargo clippy -p eggup-service --all-targets --locked -- -D warnings
```

The full gate is `./scripts/check-local.sh`; see
[tooling-governance.md](tooling-governance.md).

## Known doc/code drift

- **Three UTF-8 boundary tests, not two.** CI runs three individually
  (`ci.yml:124-126`) — the third, `lifecycle_failure_detail_is_utf8_safe_bounded_and_control_free`,
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
- **`ScmState::Unknown` is unreachable on every platform.** `windows-service`
  0.8.1 exposes exactly seven `ServiceState` variants and the SCM backend maps
  each of them, so no platform can produce `ScmState::Unknown`; the test
  helpers construct it only. Its `allow(dead_code)` is therefore
  unconditional, while every other `ScmState` and `ScmLookup` variant is
  constructed by the `#[cfg(windows)]` backend and carries a
  `#[cfg_attr(not(windows), allow(dead_code))]`. The variant sets are
  unchanged; a variant added later is not covered by any allowance and is
  reported.
- **The real-SCM fixture is not run off Windows.** `tests/windows_scm_live.rs`
  is `#![cfg(windows)]`, so on a non-Windows host the target compiles to zero
  tests. Its result on a non-Windows lane is **not run**, never a pass and
  never an inference from another platform.

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
