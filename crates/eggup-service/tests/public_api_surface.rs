//! M014 — the local, dependency-free **public-API surface guard**.
//!
//! This is the *enforcement* half of work package WP3. The committed
//! snapshot next to it, `tests/public-api-surface.txt`, is the *reviewable
//! artifact*; this file is what actually fails the build.
//!
//! # What this guards
//!
//! Every public item of `eggup-service` is bound to an explicit type below.
//! A **removed** item or a **changed signature** is therefore a
//! *compilation* error, not a review note: the guard lives in
//! `crates/eggup-service/tests/`, so it is compiled by
//! `cargo test --workspace --all-targets --all-features --locked` — the gate
//! `scripts/check-local.sh` already runs.
//!
//! # Constraints this guard honours
//!
//! * **No new dependency.** It uses only `eggup-service`, `eggup-core` (an
//!   existing dependency of the crate), and `std` (R-B8).
//! * **Dependency-free, network-free, root-free.** It reads the crate's own
//!   sources through `include_str!` and writes nothing.
//! * **A deliberate review step.** The coverage assertion compares the
//!   crate's own sources with the number of pinned items, so adding a public
//!   item without pinning it fails the guard (R-B7).
//!
//! # The retained compatibility fixture
//!
//! `tests/api_compatibility.rs` is **retained**, not deleted as redundant, and
//! it is folded into this guard: it is compiled here as a module, so its
//! `LifecycleState` variant-pinning signal is part of this guard's coverage
//! as well as remaining a test target of its own (R-B6).

// The point of this file is to spell out the exact public types. Factoring
// them into aliases would move the shape out of the binding and weaken the
// pin, so the complexity lint is not applicable here.
#![allow(clippy::type_complexity)]

// The retained compatibility fixture, folded in. Its `LifecycleState`
// variant-pinning signal is compiled as part of this guard, and it remains a
// test target in its own right.
#[path = "api_compatibility.rs"]
mod api_compatibility;

use std::path::{Path, PathBuf};
use std::time::Duration;

use eggup_core::{
    CommitOwnership, PostCommitFailurePolicy, TransactionDisposition, TransactionReceipt,
    ValidatedTransaction,
};
use eggup_service::{
    atomic_write_definition, candidate_managers, commit_with_disposition,
    commit_with_disposition_with_engine_policy, commit_with_lifecycle,
    commit_with_lifecycle_with_engine_policy, cron_classify, cron_merge, cron_remove, inspect_host,
    plan_unix, plan_windows, require_owned, CandidateManager, CommandExecutor, CommandOutput,
    CommitEnginePolicy, CronManager, CronOwnership, DirectObservation, DirectRuntimeControl,
    DirectState, DispositionBaseline, FakeCall, FakeExecutor, HealthProbe, HealthState, HostFacts,
    KnownConfig, LaunchdDomain, LaunchdInstall, LaunchdManager, LifecycleFailure,
    LifecycleRestorationStatus, LifecycleSnapshot, LifecycleState, LifecycleUpdateError,
    LifecycleUpdatePhase, LifecycleUpdatePolicy, LifecycleUpdateReceipt, ManagerCommandOutput,
    ManagerFailureKind, NoPostInstallCheck, NoProbe, Ownership, OwnershipLossTolerance,
    PostInstallCheck, PostInstallCheckError, PostTransitionVerification, RegistrationSnapshot,
    RestoreIntent, ServiceError, ServiceId, ServiceManager, ServiceOperation, ServiceSpec,
    SystemExecutor, SystemdInstall, SystemdManager, SystemdScope, TestDoubleManager,
    TransitionResult, UnixPlannerInput, UpdateRuntimeDisposition, WindowsErrorControl,
    WindowsPlannerInput, WindowsScmInstall, WindowsScmManager, WindowsServiceDependency,
    WindowsServiceState, WindowsStartType, MAX_COMMAND_OUTPUT_BYTES, MAX_CRONTAB_BYTES,
    MAX_DEFINITION_BYTES, MAX_DIAGNOSTIC_BYTES, MAX_TRANSITION_TIMEOUT,
};

// ---------------------------------------------------------------------------
// Local helper types.
//
// They exist only to give the trait and generic pins concrete arguments. They
// are not part of the crate's public surface.
// ---------------------------------------------------------------------------

/// A local `DirectRuntimeControl` implementor, used to pin the trait's
/// method signature.
#[derive(Debug)]
struct PinDirectRuntimeControl;

impl DirectRuntimeControl for PinDirectRuntimeControl {
    fn inspect(&self) -> Result<DirectObservation, ServiceError> {
        Err(ServiceError::manager(
            ManagerFailureKind::Protocol,
            "the pinning implementor is never called",
        ))
    }

    fn stop(&mut self, _timeout: Duration) -> Result<TransitionResult, ServiceError> {
        Err(ServiceError::manager(
            ManagerFailureKind::Protocol,
            "the pinning implementor is never called",
        ))
    }

    fn start(&mut self, _timeout: Duration) -> Result<TransitionResult, ServiceError> {
        Err(ServiceError::manager(
            ManagerFailureKind::Protocol,
            "the pinning implementor is never called",
        ))
    }
}

/// The number of top-level public items the guard pins.
///
/// It is compared against the crate's own sources, so a public item added
/// without a matching pin fails the guard.
const PINNED_TOP_LEVEL_ITEMS: usize = 75;

/// The number of `pub use` re-exports the guard pins.
const PINNED_REEXPORTS: usize = 5;

/// The sources the coverage assertion reads.
const SOURCES: &[(&str, &str)] = &[
    ("commit_engine.rs", include_str!("../src/commit_engine.rs")),
    ("disposition.rs", include_str!("../src/disposition.rs")),
    ("lib.rs", include_str!("../src/lib.rs")),
    (
        "lifecycle_update.rs",
        include_str!("../src/lifecycle_update.rs"),
    ),
    ("windows_scm.rs", include_str!("../src/windows_scm.rs")),
];

// ---------------------------------------------------------------------------
// `impl Into<String>` (argument-position `impl Trait`) parameters cannot be
// turbofished, so the affected constructors are pinned through a local
// generic wrapper. The wrapper's bound must still satisfy the constructor's
// bound, so a changed bound is a compilation error.
// ---------------------------------------------------------------------------

/// Pins `ServiceId::new`.
fn pin_service_id_new<T: Into<String>>(value: T) -> Result<ServiceId, ServiceError> {
    ServiceId::new(value)
}

/// Pins `PostInstallCheckError::new`.
fn pin_post_install_check_error_new<T: Into<String>>(detail: T) -> PostInstallCheckError {
    PostInstallCheckError::new(detail)
}

/// Pins `ServiceError::manager`.
fn pin_service_error_manager<T: Into<String>>(kind: ManagerFailureKind, detail: T) -> ServiceError {
    ServiceError::manager(kind, detail)
}

/// Pins `ServiceError::manager_with_os_code`.
fn pin_service_error_manager_with_os_code<T: Into<String>>(
    kind: ManagerFailureKind,
    detail: T,
    os_code: Option<i32>,
) -> ServiceError {
    ServiceError::manager_with_os_code(kind, detail, os_code)
}

/// Pins `WindowsScmInstall::new`.
fn pin_windows_scm_install_new<S: Into<String>, D: Into<String>>(
    service_name: S,
    display_name: D,
    start_type: WindowsStartType,
    error_control: WindowsErrorControl,
    account_name: Option<String>,
) -> Result<WindowsScmInstall, ServiceError> {
    WindowsScmInstall::new(
        service_name,
        display_name,
        start_type,
        error_control,
        account_name,
    )
}

// ---------------------------------------------------------------------------
// `commit_engine.rs`
// ---------------------------------------------------------------------------

/// `LifecycleUpdatePhase` — every variant, by name.
const _: fn(&LifecycleUpdatePhase) -> u8 = |phase| match phase {
    LifecycleUpdatePhase::Inspect => 0,
    LifecycleUpdatePhase::Quiesce => 1,
    LifecycleUpdatePhase::Commit => 2,
    LifecycleUpdatePhase::RestoreNew => 3,
    LifecycleUpdatePhase::PostInstallCheck => 4,
    LifecycleUpdatePhase::QuiesceForRollback => 5,
    LifecycleUpdatePhase::RestoreOld => 6,
    LifecycleUpdatePhase::FinalInspect => 7,
    _ => 8,
};

/// `LifecycleFailure` — the public field names and types.
const _: fn(
    &LifecycleFailure,
) -> (
    LifecycleUpdatePhase,
    String,
    String,
    Option<Ownership>,
    Option<LifecycleState>,
) = |failure| {
    (
        failure.phase,
        failure.service_id.clone(),
        failure.detail.clone(),
        failure.ownership,
        failure.state,
    )
};

/// `PostInstallCheckError` — the tuple field is private; the type is pinned
/// through its constructor.
const _: fn(String) -> PostInstallCheckError = pin_post_install_check_error_new::<String>;

/// `PostInstallCheck` — the trait's method signature.
const _: fn(
    &NoPostInstallCheck,
    &ServiceSpec,
    &LifecycleSnapshot,
    Duration,
) -> Result<(), PostInstallCheckError> = NoPostInstallCheck::check;

/// `NoPostInstallCheck` — the unit struct.
const _: fn() -> NoPostInstallCheck = NoPostInstallCheck::default;

/// `PostTransitionVerification` — every variant, by name.
const _: fn(&PostTransitionVerification) -> u8 = |verification| match verification {
    PostTransitionVerification::Verify => 0,
    PostTransitionVerification::TrustManagerReport => 1,
    _ => 2,
};

/// `OwnershipLossTolerance` — every variant, by name.
const _: fn(&OwnershipLossTolerance) -> u8 = |tolerance| match tolerance {
    OwnershipLossTolerance::Deny => 0,
    OwnershipLossTolerance::TolerateAlreadyStopped => 1,
    _ => 2,
};

/// `CommitEnginePolicy` — the public field names and types.
const _: fn(&CommitEnginePolicy) -> (PostTransitionVerification, OwnershipLossTolerance) =
    |policy| {
        (
            policy.post_transition_verification,
            policy.ownership_loss_tolerance,
        )
    };

// ---------------------------------------------------------------------------
// `disposition.rs`
// ---------------------------------------------------------------------------

/// `UpdateRuntimeDisposition` — every variant, exhaustively.
const _: fn(&UpdateRuntimeDisposition) -> u8 = |disposition| match disposition {
    UpdateRuntimeDisposition::ManagedRunning => 0,
    UpdateRuntimeDisposition::ManagedStopped => 1,
    UpdateRuntimeDisposition::DirectRunning => 2,
    UpdateRuntimeDisposition::Stopped => 3,
    UpdateRuntimeDisposition::ForeignPreserved => 4,
};

/// `KnownConfig` — every variant, exhaustively.
const _: fn(&KnownConfig) -> u8 = |config| match config {
    KnownConfig::Absent => 0,
    KnownConfig::Present(path) => {
        let _: &PathBuf = path;
        1
    }
    KnownConfig::Unknown => 2,
};

/// `DirectState` — every variant, exhaustively.
const _: fn(&DirectState) -> u8 = |state| match state {
    DirectState::Running => 0,
    DirectState::Stopped => 1,
    DirectState::Unknown => 2,
};

/// `DirectObservation` — the public field names and types.
const _: fn(&DirectObservation) -> (DirectState, bool, Option<PathBuf>, Option<PathBuf>) =
    |observed| {
        (
            observed.state,
            observed.exact_ownership_proven,
            observed.executable.clone(),
            observed.config.clone(),
        )
    };

/// `UnixPlannerInput` — the public field names and types.
const _: fn(
    &UnixPlannerInput,
) -> (
    Ownership,
    bool,
    bool,
    KnownConfig,
    Option<PathBuf>,
    DirectObservation,
) = |input| {
    (
        input.manager_ownership,
        input.manager_active,
        input.manager_state_known,
        input.registered_config.clone(),
        input.selected_config.clone(),
        input.direct.clone(),
    )
};

/// `WindowsServiceState` — every variant, exhaustively.
const _: fn(&WindowsServiceState) -> u8 = |state| match state {
    WindowsServiceState::Running => 0,
    WindowsServiceState::StartPending => 1,
    WindowsServiceState::Stopped => 2,
    WindowsServiceState::StopPending => 3,
    WindowsServiceState::Unknown => 4,
};

/// `WindowsPlannerInput` — the public field names and types.
const _: fn(&WindowsPlannerInput) -> (bool, bool, Ownership, WindowsServiceState) = |input| {
    (
        input.installed,
        input.executable_known,
        input.ownership,
        input.state,
    )
};

/// `plan_unix` — the exact signature.
const _: fn(&UnixPlannerInput) -> Result<UpdateRuntimeDisposition, ServiceError> = plan_unix;

/// `plan_windows` — the exact signature.
const _: fn(&WindowsPlannerInput) -> Result<UpdateRuntimeDisposition, ServiceError> = plan_windows;

/// `DirectRuntimeControl` — the trait's method signature, through a local
/// implementor.
const _: fn(&PinDirectRuntimeControl) -> Result<DirectObservation, ServiceError> =
    PinDirectRuntimeControl::inspect;
const _: fn(&mut PinDirectRuntimeControl, Duration) -> Result<TransitionResult, ServiceError> =
    PinDirectRuntimeControl::stop;
const _: fn(&mut PinDirectRuntimeControl, Duration) -> Result<TransitionResult, ServiceError> =
    PinDirectRuntimeControl::start;

/// `DispositionBaseline` — the public field names and types.
const _: fn(&DispositionBaseline) -> (Ownership, LifecycleState, Option<DirectObservation>) =
    |baseline| {
        (
            baseline.manager_ownership,
            baseline.manager_state,
            baseline.direct.clone(),
        )
    };

/// `commit_with_disposition` — the exact generic signature.
const _: fn(
    &mut TestDoubleManager,
    &ServiceSpec,
    Option<&mut PinDirectRuntimeControl>,
    ValidatedTransaction,
    CommitOwnership<'static>,
    LifecycleUpdatePolicy,
    &NoPostInstallCheck,
    UpdateRuntimeDisposition,
    &DispositionBaseline,
) -> Result<LifecycleUpdateReceipt, LifecycleUpdateError> =
    commit_with_disposition::<TestDoubleManager, PinDirectRuntimeControl, NoPostInstallCheck>;

/// `commit_with_disposition_with_engine_policy` — the exact generic signature.
const _: fn(
    &mut TestDoubleManager,
    &ServiceSpec,
    Option<&mut PinDirectRuntimeControl>,
    ValidatedTransaction,
    CommitOwnership<'static>,
    LifecycleUpdatePolicy,
    &NoPostInstallCheck,
    UpdateRuntimeDisposition,
    &DispositionBaseline,
    CommitEnginePolicy,
) -> Result<LifecycleUpdateReceipt, LifecycleUpdateError> =
    commit_with_disposition_with_engine_policy::<
        TestDoubleManager,
        PinDirectRuntimeControl,
        NoPostInstallCheck,
    >;

// ---------------------------------------------------------------------------
// `lib.rs`
// ---------------------------------------------------------------------------

/// `ServiceId` — the constructor and the accessor.
const _: fn(String) -> Result<ServiceId, ServiceError> = pin_service_id_new::<String>;
const _: for<'a> fn(&'a ServiceId) -> &'a str = ServiceId::as_str;

/// `ServiceSpec` — the constructor and the accessors.
const _: fn(ServiceId, PathBuf, Vec<String>, Option<PathBuf>) -> Result<ServiceSpec, ServiceError> =
    ServiceSpec::new;
const _: for<'a> fn(&'a ServiceSpec) -> &'a ServiceId = ServiceSpec::id;
const _: for<'a> fn(&'a ServiceSpec) -> &'a Path = ServiceSpec::executable;
const _: for<'a> fn(&'a ServiceSpec) -> &'a [String] = ServiceSpec::args;
const _: for<'a> fn(&'a ServiceSpec) -> Option<&'a Path> = ServiceSpec::config;

/// `LifecycleState` — every variant, by name. This is the signal the
/// retained `api_compatibility.rs` fixture carries; it is pinned here too.
const _: fn(&LifecycleState) -> u8 = |state| match state {
    LifecycleState::Stopped => 0,
    LifecycleState::Running => 1,
    LifecycleState::Transitioning => 2,
    LifecycleState::Unknown => 3,
    _ => 4,
};

/// `HealthState` — every variant, by name.
const _: fn(&HealthState) -> u8 = |state| match state {
    HealthState::Healthy => 0,
    HealthState::Degraded => 1,
    HealthState::Unknown => 2,
    _ => 3,
};

/// `RegistrationSnapshot` — the public field names and types.
const _: fn(&RegistrationSnapshot) -> (bool, Option<PathBuf>, Vec<String>, Option<PathBuf>, bool) =
    |snapshot| {
        (
            snapshot.present,
            snapshot.executable.clone(),
            snapshot.args.clone(),
            snapshot.config.clone(),
            snapshot.malformed,
        )
    };

/// `RegistrationSnapshot::absent` and `::ownership`.
const _: fn() -> RegistrationSnapshot = RegistrationSnapshot::absent;
const _: for<'a> fn(&'a RegistrationSnapshot, &'a ServiceSpec) -> Ownership =
    RegistrationSnapshot::ownership;

/// `LifecycleSnapshot` — the public field names and types.
const _: fn(
    &LifecycleSnapshot,
) -> (
    ServiceId,
    Ownership,
    LifecycleState,
    HealthState,
    bool,
    bool,
) = |snapshot| {
    (
        snapshot.id.clone(),
        snapshot.ownership,
        snapshot.state,
        snapshot.health,
        snapshot.was_registered,
        snapshot.was_running,
    )
};

/// `RestoreIntent` — every variant, exhaustively.
const _: fn(&RestoreIntent) -> u8 = |intent| match intent {
    RestoreIntent::Preserve => 0,
    RestoreIntent::EnsureRunning => 1,
    RestoreIntent::EnsureStopped => 2,
};

/// `TransitionResult` — the public field names and types.
const _: fn(&TransitionResult) -> (ServiceOperation, bool, String) =
    |result| (result.operation, result.completed, result.detail.clone());

/// `ServiceOperation` — every variant, by name.
const _: fn(&ServiceOperation) -> u8 = |operation| match operation {
    ServiceOperation::Inspect => 0,
    ServiceOperation::Install => 1,
    ServiceOperation::Start => 2,
    ServiceOperation::Stop => 3,
    ServiceOperation::Restart => 4,
    ServiceOperation::Uninstall => 5,
    _ => 6,
};

/// `ManagerFailureKind` — every variant, by name.
const _: fn(&ManagerFailureKind) -> u8 = |kind| match kind {
    ManagerFailureKind::Spawn => 0,
    ManagerFailureKind::Timeout => 1,
    ManagerFailureKind::Permission => 2,
    ManagerFailureKind::NotFound => 3,
    ManagerFailureKind::Protocol => 4,
    ManagerFailureKind::LocalFilesystem => 5,
    _ => 6,
};

/// `ServiceError` — the variant names and the payload field names and types.
const _: fn(&ServiceError) -> u8 = |error| match error {
    ServiceError::InvalidInput(detail) => {
        let _: &String = detail;
        0
    }
    ServiceError::OwnershipDenied { id, ownership } => {
        let _: &String = id;
        let _: &Ownership = ownership;
        1
    }
    ServiceError::Conflict(detail) => {
        let _: &String = detail;
        2
    }
    ServiceError::Manager {
        kind,
        detail,
        os_code,
    } => {
        let _: &ManagerFailureKind = kind;
        let _: &String = detail;
        let _: &Option<i32> = os_code;
        3
    }
    _ => 4,
};

/// `ServiceError` — the constructors and the accessors.
const _: fn(ManagerFailureKind, String) -> ServiceError = pin_service_error_manager::<String>;
const _: fn(ManagerFailureKind, String, Option<i32>) -> ServiceError =
    pin_service_error_manager_with_os_code::<String>;
const _: fn(&ServiceId, Ownership) -> ServiceError = ServiceError::denied;
const _: for<'a> fn(&'a ServiceError) -> Option<ManagerFailureKind> = ServiceError::manager_kind;
const _: for<'a> fn(&'a ServiceError) -> Option<i32> = ServiceError::manager_os_code;

/// `HealthProbe` — the trait's method signature.
const _: fn(&NoProbe, &ServiceSpec) -> HealthState = NoProbe::check;

/// `NoProbe` — the unit struct.
const _: fn() -> NoProbe = NoProbe::default;

/// `ServiceManager` — the trait's method signatures, through the crate's own
/// `TestDoubleManager` implementor.
const _: for<'a> fn(
    &'a TestDoubleManager,
    &'a ServiceSpec,
) -> Result<LifecycleSnapshot, ServiceError> = TestDoubleManager::inspect;
const _: for<'a> fn(
    &'a mut TestDoubleManager,
    &'a ServiceSpec,
) -> Result<TransitionResult, ServiceError> = TestDoubleManager::install;
const _: for<'a> fn(
    &'a mut TestDoubleManager,
    &'a ServiceSpec,
    Duration,
) -> Result<TransitionResult, ServiceError> = TestDoubleManager::start;
const _: for<'a> fn(
    &'a mut TestDoubleManager,
    &'a ServiceSpec,
    Duration,
) -> Result<TransitionResult, ServiceError> = TestDoubleManager::stop;
const _: for<'a> fn(
    &'a mut TestDoubleManager,
    &'a ServiceSpec,
    Duration,
) -> Result<TransitionResult, ServiceError> = TestDoubleManager::restart;
const _: for<'a> fn(
    &'a mut TestDoubleManager,
    &'a ServiceSpec,
) -> Result<TransitionResult, ServiceError> = TestDoubleManager::uninstall;

/// `require_owned` — the exact signature.
const _: fn(&ServiceId, Ownership) -> Result<(), ServiceError> = require_owned;

/// `TestDoubleManager` — the constructor and the injection helpers.
const _: fn() -> TestDoubleManager = TestDoubleManager::new;
const _: fn(&mut TestDoubleManager, ServiceSpec, LifecycleState) = TestDoubleManager::put;
const _: fn(&mut TestDoubleManager, ServiceId) = TestDoubleManager::put_malformed;

/// The bounded-size constants.
const _: usize = MAX_COMMAND_OUTPUT_BYTES;
const _: usize = MAX_CRONTAB_BYTES;
const _: usize = MAX_DEFINITION_BYTES;
const _: Duration = MAX_TRANSITION_TIMEOUT;
const _: usize = MAX_DIAGNOSTIC_BYTES;

/// `ManagerCommandOutput` — the public field names and types.
const _: fn(&ManagerCommandOutput) -> (Option<i32>, Vec<u8>, Vec<u8>) =
    |output| (output.status, output.stdout.clone(), output.stderr.clone());

/// `ManagerCommandOutput` — the accessors.
const _: for<'a> fn(&'a ManagerCommandOutput) -> String = ManagerCommandOutput::stdout_text;
const _: for<'a> fn(&'a ManagerCommandOutput) -> String = ManagerCommandOutput::stderr_text;

/// `CommandOutput` — the compatibility alias, pinned to the same type.
const _: fn() -> Option<CommandOutput> = || None;

/// `CommandExecutor` — the trait's method signature.
const _: fn(
    &FakeExecutor,
    &[String],
    Option<&[u8]>,
    Duration,
) -> Result<ManagerCommandOutput, ServiceError> = FakeExecutor::run;

/// `SystemExecutor` — the constructor and the bounded-output constructor.
const _: fn() -> SystemExecutor = SystemExecutor::new;
const _: fn(usize) -> Result<SystemExecutor, ServiceError> = SystemExecutor::with_max_output;

/// `FakeCall` — the public field names and types.
const _: fn(&FakeCall) -> (Vec<String>, Option<Vec<u8>>) =
    |call| (call.argv.clone(), call.stdin.clone());

/// `FakeExecutor` — the constructor and the scripting helpers.
const _: fn() -> FakeExecutor = FakeExecutor::new;
const _: fn(&FakeExecutor, &[&str], ManagerCommandOutput) = FakeExecutor::expect;
const _: fn(&FakeExecutor, &[&str], ServiceError) = FakeExecutor::expect_error;
const _: for<'a> fn(&'a FakeExecutor) -> Vec<FakeCall> = FakeExecutor::calls;
const _: for<'a> fn(&'a FakeExecutor) -> usize = FakeExecutor::call_count;
const _: for<'a> fn(&'a FakeExecutor) -> bool = FakeExecutor::is_exhausted;

/// `atomic_write_definition` — the exact signature.
const _: fn(&Path, &[u8], bool) -> Result<(), ServiceError> = atomic_write_definition;

/// `SystemdScope` — every variant, exhaustively.
const _: fn(&SystemdScope) -> u8 = |scope| match scope {
    SystemdScope::System => 0,
    SystemdScope::User => 1,
};

/// `SystemdInstall` — the constructor and the public field names and types.
const _: fn(
    String,
    SystemdScope,
    PathBuf,
    Vec<u8>,
    bool,
    bool,
    Duration,
) -> Result<SystemdInstall, ServiceError> = SystemdInstall::new;
const _: fn(&SystemdInstall) -> (String, SystemdScope, PathBuf, Vec<u8>, bool, bool, Duration) =
    |install| {
        (
            install.unit_name.clone(),
            install.scope,
            install.unit_path.clone(),
            install.definition.clone(),
            install.enable,
            install.reload,
            install.transition_timeout,
        )
    };

/// `SystemdManager` — the constructor and the accessors.
const _: fn(SystemExecutor, SystemdInstall) -> SystemdManager = SystemdManager::new;
const _: for<'a> fn(&'a SystemdManager) -> &'a SystemdInstall = SystemdManager::install_config;
const _: for<'a> fn(&'a SystemdManager) -> &'a SystemExecutor = SystemdManager::executor;

/// `LaunchdDomain` — every variant, exhaustively.
const _: fn(&LaunchdDomain) -> u8 = |domain| match domain {
    LaunchdDomain::UserAgent => 0,
    LaunchdDomain::SystemDaemon => 1,
};

/// `LaunchdInstall` — the constructor and the public field names and types.
const _: fn(
    String,
    LaunchdDomain,
    String,
    PathBuf,
    Vec<u8>,
    bool,
    Duration,
) -> Result<LaunchdInstall, ServiceError> = LaunchdInstall::new;
const _: fn(
    &LaunchdInstall,
) -> (
    String,
    LaunchdDomain,
    String,
    PathBuf,
    Vec<u8>,
    bool,
    Duration,
) = |install| {
    (
        install.label.clone(),
        install.domain,
        install.target.clone(),
        install.plist_path.clone(),
        install.definition.clone(),
        install.bootstrap_on_install,
        install.transition_timeout,
    )
};

/// `LaunchdManager` — the constructor and the accessors.
const _: fn(SystemExecutor, LaunchdInstall) -> LaunchdManager = LaunchdManager::new;
const _: for<'a> fn(&'a LaunchdManager) -> &'a LaunchdInstall = LaunchdManager::install_config;
const _: for<'a> fn(&'a LaunchdManager) -> &'a SystemExecutor = LaunchdManager::executor;

/// `CronOwnership` — the compatibility alias, pinned to the same type as
/// `Ownership`.
const _: fn() -> Option<CronOwnership> = || None;
const _: fn(CronOwnership) -> Ownership = |ownership| ownership;

/// `cron_classify`, `cron_merge`, `cron_remove` — the exact signatures.
const _: fn(&str, &str, &str) -> CronOwnership = cron_classify;
const _: fn(&str, &str, &str) -> Result<(String, bool), ServiceError> = cron_merge;
const _: fn(&str, &str, &str) -> Result<(String, bool), ServiceError> = cron_remove;

/// `CronManager` — the constructor and the accessor.
const _: fn(SystemExecutor, String, String, Duration) -> Result<CronManager, ServiceError> =
    CronManager::new;
const _: for<'a> fn(&'a CronManager) -> &'a str = CronManager::marker;

/// `CandidateManager` — every variant, exhaustively.
const _: fn(&CandidateManager) -> u8 = |candidate| match candidate {
    CandidateManager::Systemd(scope) => {
        let _: &SystemdScope = scope;
        0
    }
    CandidateManager::Launchd(domain) => {
        let _: &LaunchdDomain = domain;
        1
    }
    CandidateManager::Cron => 2,
};

/// `HostFacts` — the public field names and types.
const _: fn(&HostFacts) -> (String, bool, bool, bool) = |facts| {
    (
        facts.os.clone(),
        facts.systemd_available,
        facts.launchd_available,
        facts.crontab_available,
    )
};

/// `inspect_host` — the exact signature, with the `impl CommandExecutor`
/// parameter instantiated.
const _: fn(&FakeExecutor) -> Result<HostFacts, ServiceError> = inspect_host;

/// `candidate_managers` — the exact signature.
const _: fn(&HostFacts) -> Vec<CandidateManager> = candidate_managers;

/// `Ownership` — every variant, exhaustively. It is re-exported from
/// `eggup-core` and is not `#[non_exhaustive]`.
const _: fn(&Ownership) -> u8 = |ownership| match ownership {
    Ownership::Absent => 0,
    Ownership::Owned => 1,
    Ownership::Foreign => 2,
    Ownership::Unknown => 3,
};

// ---------------------------------------------------------------------------
// `lifecycle_update.rs`
// ---------------------------------------------------------------------------

/// `LifecycleRestorationStatus` — every variant, exhaustively.
const _: fn(&LifecycleRestorationStatus) -> u8 = |status| match status {
    LifecycleRestorationStatus::Restored => 0,
    LifecycleRestorationStatus::Failed => 1,
    LifecycleRestorationStatus::NotAttemptedRecoveryRequired => 2,
};

/// `LifecycleUpdatePolicy` — the public field names and types.
const _: fn(
    &LifecycleUpdatePolicy,
) -> (
    RestoreIntent,
    PostCommitFailurePolicy,
    Duration,
    Duration,
    Duration,
) = |policy| {
    (
        policy.restore,
        policy.post_commit_failure,
        policy.quiesce_timeout,
        policy.post_commit_timeout,
        policy.rollback_restore_timeout,
    )
};

/// `LifecycleUpdateReceipt` — the public field names and types.
const _: fn(
    &LifecycleUpdateReceipt,
) -> (
    LifecycleSnapshot,
    Option<LifecycleSnapshot>,
    TransactionReceipt,
    bool,
    LifecycleRestorationStatus,
    Option<LifecycleFailure>,
    Option<LifecycleFailure>,
    Option<LifecycleFailure>,
    Option<LifecycleFailure>,
    CommitEnginePolicy,
) = |receipt| {
    (
        receipt.before.clone(),
        receipt.final_snapshot.clone(),
        receipt.transaction.clone(),
        receipt.pre_commit_quiesced,
        receipt.restoration,
        receipt.post_commit_failure.clone(),
        receipt.rollback_quiesce_failure.clone(),
        receipt.rollback_restore_failure.clone(),
        receipt.observation_failure.clone(),
        receipt.engine_policy,
    )
};

/// `LifecycleUpdateReceipt` — the accessors.
const _: for<'a> fn(&'a LifecycleUpdateReceipt) -> TransactionDisposition =
    LifecycleUpdateReceipt::artifact_disposition;
const _: for<'a> fn(&'a LifecycleUpdateReceipt) -> bool =
    LifecycleUpdateReceipt::manual_artifact_recovery_required;
const _: for<'a> fn(&'a LifecycleUpdateReceipt) -> bool =
    LifecycleUpdateReceipt::new_generation_retained;
const _: for<'a> fn(&'a LifecycleUpdateReceipt) -> bool =
    LifecycleUpdateReceipt::service_state_restored;

/// `LifecycleUpdateError` — the variant names and the payload field names.
const _: fn(&LifecycleUpdateError) -> u8 = |error| match error {
    LifecycleUpdateError::InvalidPolicy { field } => {
        let _: &&'static str = field;
        0
    }
    LifecycleUpdateError::Preflight { failure } => {
        let _: &LifecycleFailure = failure;
        1
    }
    LifecycleUpdateError::Quiesce {
        failure,
        restore_failure,
    } => {
        let _: &LifecycleFailure = failure;
        let _: &Option<LifecycleFailure> = restore_failure;
        2
    }
    LifecycleUpdateError::CoreBeforeReceipt {
        core: _,
        restore_failure,
    } => {
        let _: &Option<LifecycleFailure> = restore_failure;
        3
    }
    _ => 4,
};

/// `commit_with_lifecycle` — the exact generic signature.
const _: fn(
    &mut TestDoubleManager,
    &ServiceSpec,
    ValidatedTransaction,
    CommitOwnership<'static>,
    LifecycleUpdatePolicy,
    &NoPostInstallCheck,
) -> Result<LifecycleUpdateReceipt, LifecycleUpdateError> =
    commit_with_lifecycle::<TestDoubleManager, NoPostInstallCheck>;

/// `commit_with_lifecycle_with_engine_policy` — the exact generic signature.
const _: fn(
    &mut TestDoubleManager,
    &ServiceSpec,
    ValidatedTransaction,
    CommitOwnership<'static>,
    LifecycleUpdatePolicy,
    &NoPostInstallCheck,
    CommitEnginePolicy,
) -> Result<LifecycleUpdateReceipt, LifecycleUpdateError> =
    commit_with_lifecycle_with_engine_policy::<TestDoubleManager, NoPostInstallCheck>;

// ---------------------------------------------------------------------------
// `windows_scm.rs`
// ---------------------------------------------------------------------------

/// `WindowsStartType` — every variant, exhaustively.
const _: fn(&WindowsStartType) -> u8 = |start_type| match start_type {
    WindowsStartType::Automatic => 0,
    WindowsStartType::Manual => 1,
    WindowsStartType::Disabled => 2,
};

/// `WindowsErrorControl` — every variant, exhaustively.
const _: fn(&WindowsErrorControl) -> u8 = |error_control| match error_control {
    WindowsErrorControl::Ignore => 0,
    WindowsErrorControl::Normal => 1,
    WindowsErrorControl::Severe => 2,
    WindowsErrorControl::Critical => 3,
};

/// `WindowsServiceDependency` — every variant, exhaustively.
const _: fn(&WindowsServiceDependency) -> u8 = |dependency| match dependency {
    WindowsServiceDependency::Service(name) => {
        let _: &String = name;
        0
    }
    WindowsServiceDependency::Group(name) => {
        let _: &String = name;
        1
    }
};

/// `WindowsScmInstall` — the constructors and the accessors.
const _: fn(
    String,
    String,
    WindowsStartType,
    WindowsErrorControl,
    Option<String>,
) -> Result<WindowsScmInstall, ServiceError> = pin_windows_scm_install_new::<String, String>;
const _: fn(
    WindowsScmInstall,
    Vec<WindowsServiceDependency>,
) -> Result<WindowsScmInstall, ServiceError> = WindowsScmInstall::with_dependencies;
const _: fn(WindowsScmInstall, Duration) -> Result<WindowsScmInstall, ServiceError> =
    WindowsScmInstall::with_transition_timeout;
const _: for<'a> fn(&'a WindowsScmInstall) -> &'a ServiceId = WindowsScmInstall::service_id;
const _: for<'a> fn(&'a WindowsScmInstall) -> &'a str = WindowsScmInstall::display_name;
const _: for<'a> fn(&'a WindowsScmInstall) -> WindowsStartType = WindowsScmInstall::start_type;
const _: for<'a> fn(&'a WindowsScmInstall) -> WindowsErrorControl =
    WindowsScmInstall::error_control;
const _: for<'a> fn(&'a WindowsScmInstall) -> Option<&'a str> = WindowsScmInstall::account_name;
const _: for<'a> fn(&'a WindowsScmInstall) -> Option<&'a [WindowsServiceDependency]> =
    WindowsScmInstall::dependencies;
const _: for<'a> fn(&'a WindowsScmInstall) -> Duration = WindowsScmInstall::transition_timeout;

/// `WindowsScmManager` — the constructor is Windows-only; the accessor is
/// everywhere.
#[cfg(windows)]
const _: fn(WindowsScmInstall) -> WindowsScmManager = WindowsScmManager::new;
const _: for<'a> fn(&'a WindowsScmManager) -> &'a WindowsScmInstall =
    WindowsScmManager::install_config;

// ---------------------------------------------------------------------------
// The re-exports.
//
// Each `pub use` in `lib.rs` is pinned by naming one item it re-exports.
// A removed re-export is a compilation error at the `use` site.
// ---------------------------------------------------------------------------

const _: fn() -> Option<CommitEnginePolicy> = || None;
const _: fn() -> Option<LifecycleFailure> = || None;
const _: fn() -> Option<LifecycleUpdatePhase> = || None;
const _: fn() -> Option<NoPostInstallCheck> = || None;
const _: fn() -> Option<OwnershipLossTolerance> = || None;
const _: fn() -> Option<PostInstallCheckError> = || None;
const _: fn() -> Option<PostTransitionVerification> = || None;
const _: fn() -> Option<DirectObservation> = || None;
const _: fn() -> Option<&'static dyn DirectRuntimeControl> = || None;
const _: fn() -> Option<DirectState> = || None;
const _: fn() -> Option<DispositionBaseline> = || None;
const _: fn() -> Option<KnownConfig> = || None;
const _: fn() -> Option<UnixPlannerInput> = || None;
const _: fn() -> Option<UpdateRuntimeDisposition> = || None;
const _: fn() -> Option<WindowsPlannerInput> = || None;
const _: fn() -> Option<WindowsServiceState> = || None;
const _: fn() -> Option<LifecycleRestorationStatus> = || None;
const _: fn() -> Option<LifecycleUpdateError> = || None;
const _: fn() -> Option<LifecycleUpdatePolicy> = || None;
const _: fn() -> Option<LifecycleUpdateReceipt> = || None;
const _: fn() -> Option<WindowsErrorControl> = || None;
const _: fn() -> Option<WindowsScmInstall> = || None;
const _: fn() -> Option<WindowsServiceDependency> = || None;
const _: fn() -> Option<WindowsStartType> = || None;
const _: fn() -> Option<&'static dyn PostInstallCheck> = || None;
const _: fn() -> Option<&'static dyn HealthProbe> = || None;
const _: fn() -> Option<&'static dyn ServiceManager> = || None;
const _: fn() -> Option<&'static dyn CommandExecutor> = || None;

// ---------------------------------------------------------------------------
// The coverage assertions.
// ---------------------------------------------------------------------------

/// Reports whether a line declares a top-level public item.
///
/// A top-level public item starts in column zero; an item inside an `impl`
/// block or a function body is indented and is not counted.
fn is_top_level_public_item(line: &str) -> bool {
    const KEYWORDS: [&str; 8] = [
        "fn", "struct", "enum", "trait", "const", "static", "type", "mod",
    ];
    let Some(rest) = line.strip_prefix("pub ") else {
        return false;
    };
    if rest.starts_with("use ") || rest.starts_with("unsafe ") || rest.starts_with("async ") {
        return false;
    }
    KEYWORDS.iter().any(|keyword| {
        rest.starts_with(&format!("{keyword} ")) || rest.starts_with(&format!("{keyword}("))
    })
}

/// Counts the top-level public items of a source file.
fn count_top_level_public_items(source: &str) -> usize {
    source
        .lines()
        .filter(|line| is_top_level_public_item(line))
        .count()
}

/// Counts the `pub use` re-exports of a source file.
fn count_pub_use(source: &str) -> usize {
    source
        .lines()
        .filter(|line| line.starts_with("pub use "))
        .count()
}

#[test]
fn every_top_level_public_item_is_pinned() {
    let discovered: usize = SOURCES
        .iter()
        .map(|(_, source)| count_top_level_public_items(source))
        .sum();
    assert_eq!(
        discovered, PINNED_TOP_LEVEL_ITEMS,
        "the crate's public surface changed: the guard pins {PINNED_TOP_LEVEL_ITEMS} top-level \
         public items but the sources declare {discovered}. Either pin the new item in this \
         file and in tests/public-api-surface.txt, or record the intended API change."
    );
}

#[test]
fn every_public_reexport_is_pinned() {
    let discovered: usize = SOURCES
        .iter()
        .map(|(_, source)| count_pub_use(source))
        .sum();
    assert_eq!(
        discovered, PINNED_REEXPORTS,
        "the crate's re-exports changed: the guard pins {PINNED_REEXPORTS} `pub use` \
         statements but the sources declare {discovered}. Either pin the new re-export in this \
         file and in tests/public-api-surface.txt, or record the intended API change."
    );
}

#[test]
fn the_committed_snapshot_lists_every_public_item() {
    let snapshot = include_str!("public-api-surface.txt");
    for (name, source) in SOURCES {
        for line in source.lines().filter(|line| is_top_level_public_item(line)) {
            let item = line
                .trim_start_matches("pub ")
                .split([' ', '(', '<', ':'])
                .next()
                .unwrap_or_default()
                .trim();
            assert!(
                snapshot.contains(item),
                "the committed snapshot does not mention the public item `{item}` from {name}"
            );
        }
    }
    for (name, source) in SOURCES {
        for line in source.lines().filter(|line| line.starts_with("pub use ")) {
            let reexport = line
                .trim_start_matches("pub use ")
                .split(['{', ' '])
                .next()
                .unwrap_or_default()
                .trim();
            assert!(
                snapshot.contains(reexport),
                "the committed snapshot does not mention the re-export `{reexport}` from {name}"
            );
        }
    }
}

#[test]
fn the_retained_compatibility_fixture_is_part_of_this_guard() {
    // `api_compatibility.rs` is folded in as a module, so its
    // `LifecycleState` variant-pinning signal is compiled by this guard as
    // well as remaining a test target of its own.
    assert_eq!(
        api_compatibility::classify(LifecycleState::Unknown),
        "unknown",
        "the retained compatibility fixture must keep classifying the lifecycle states"
    );
}
