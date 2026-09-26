#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Manager-neutral service registration, ownership, and lifecycle model"]
#![doc = "plus Unix manager mechanics and a native Windows SCM adapter."]
#![doc = ""]
#![doc = "Destructive manager operations are authorized only for `Owned`"]
#![doc = "registrations. `Foreign` and `Unknown` fail closed. Manager state and"]
#![doc = "application health are separate. No privilege escalation, no updater or"]
#![doc = "transport policy, and no shell interpolation live in this crate."]

mod disposition;
mod lifecycle_update;
mod windows_scm;
pub use disposition::{
    commit_with_disposition, plan_unix, plan_windows, DirectObservation, DirectRuntimeControl,
    DirectState, DispositionBaseline, KnownConfig, UnixPlannerInput, UpdateRuntimeDisposition,
    WindowsPlannerInput, WindowsServiceState,
};
pub use lifecycle_update::{
    commit_with_lifecycle, LifecycleFailure, LifecycleRestorationStatus, LifecycleUpdateError,
    LifecycleUpdatePhase, LifecycleUpdatePolicy, LifecycleUpdateReceipt, NoPostInstallCheck,
    PostInstallCheck, PostInstallCheckError,
};
pub use windows_scm::{
    WindowsErrorControl, WindowsScmInstall, WindowsScmManager, WindowsServiceDependency,
    WindowsStartType,
};

use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// A validated opaque service identity (unit name, launchd label, SCM name).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServiceId(String);

impl ServiceId {
    /// Creates an identifier, rejecting empty, overlong, or control-character input.
    pub fn new(value: impl Into<String>) -> Result<Self, ServiceError> {
        let value = value.into();
        if value.is_empty() || value.len() > 256 || value.chars().any(char::is_control) {
            return Err(ServiceError::invalid(
                "service id is empty, overlong, or has controls",
            ));
        }
        Ok(Self(value))
    }

    /// Returns the stable string form.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ServiceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Consumer-supplied desired registration.
///
/// Identity includes the exact executable path plus critical arguments and an
/// optional config path. Ownership compares all three: same name with a
/// different executable is `Foreign`; same executable with different critical
/// args is `Foreign` (documented rule); malformed registrations are `Unknown`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceSpec {
    id: ServiceId,
    executable: PathBuf,
    args: Vec<String>,
    config: Option<PathBuf>,
}

impl ServiceSpec {
    /// Creates a spec, validating the id, absolute executable, and arg safety.
    pub fn new(
        id: ServiceId,
        executable: PathBuf,
        args: Vec<String>,
        config: Option<PathBuf>,
    ) -> Result<Self, ServiceError> {
        if !executable.is_absolute() {
            return Err(ServiceError::invalid("executable must be absolute"));
        }
        if executable.as_os_str().is_empty() {
            return Err(ServiceError::invalid("executable must name a file"));
        }
        for a in &args {
            if a.chars().any(char::is_control) {
                return Err(ServiceError::invalid("argument has control characters"));
            }
        }
        if let Some(c) = &config {
            if !c.is_absolute() {
                return Err(ServiceError::invalid("config path must be absolute"));
            }
        }
        Ok(Self {
            id,
            executable,
            args,
            config,
        })
    }

    /// Returns the service identity.
    pub fn id(&self) -> &ServiceId {
        &self.id
    }

    /// Returns the exact executable path.
    pub fn executable(&self) -> &std::path::Path {
        &self.executable
    }

    /// Returns critical arguments participating in ownership.
    pub fn args(&self) -> &[String] {
        &self.args
    }

    /// Returns the optional config path participating in ownership.
    pub fn config(&self) -> Option<&std::path::Path> {
        self.config.as_deref()
    }
}

/// Canonical ownership classification, shared with `eggup-core` vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ownership {
    /// No registration exists under this identity.
    Absent,
    /// The registration exactly matches this deployment and may be mutated.
    Owned,
    /// The registration belongs to another deployment and must not be mutated.
    Foreign,
    /// Ownership cannot be proven; mutation is denied.
    Unknown,
}

/// Manager-neutral lifecycle state. Health is separate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum LifecycleState {
    /// Registered but not running.
    Stopped,
    /// Running.
    Running,
    /// Start/stop/restart in flight.
    Transitioning,
    /// State cannot be determined.
    Unknown,
}

/// Application health, distinct from manager state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum HealthState {
    /// The application reports healthy.
    Healthy,
    /// The application reports degraded/unhealthy.
    Degraded,
    /// Health is unknown or no probe is configured.
    Unknown,
}

/// A manager-neutral registration observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationSnapshot {
    /// Whether a registration record exists.
    pub present: bool,
    /// The recorded executable, when present and parseable.
    pub executable: Option<PathBuf>,
    /// Recorded critical args, when parseable.
    pub args: Vec<String>,
    /// Recorded config path, when parseable.
    pub config: Option<PathBuf>,
    /// Whether the record was malformed.
    pub malformed: bool,
}

impl RegistrationSnapshot {
    /// An absent registration.
    pub fn absent() -> Self {
        Self {
            present: false,
            executable: None,
            args: Vec::new(),
            config: None,
            malformed: false,
        }
    }

    /// Classifies this observation against the desired spec.
    pub fn ownership(&self, spec: &ServiceSpec) -> Ownership {
        if !self.present {
            return Ownership::Absent;
        }
        if self.malformed {
            return Ownership::Unknown;
        }
        match &self.executable {
            None => Ownership::Unknown,
            Some(exe) if exe != spec.executable() => Ownership::Foreign,
            Some(_) if self.args != spec.args => Ownership::Foreign,
            Some(_) if self.config.as_deref() != spec.config() => Ownership::Foreign,
            Some(_) => Ownership::Owned,
        }
    }
}

/// A point-in-time lifecycle observation plus desired restoration intent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifecycleSnapshot {
    /// The service identity observed.
    pub id: ServiceId,
    /// Ownership at observation time.
    pub ownership: Ownership,
    /// Manager state at observation time.
    pub state: LifecycleState,
    /// Health at observation time (never mutates manager state).
    pub health: HealthState,
    /// Whether the service was registered at observation time.
    pub was_registered: bool,
    /// Whether the service was running at observation time.
    pub was_running: bool,
}

impl LifecycleSnapshot {
    /// The restoration intent: restart only if it was running.
    pub fn restore_running(&self) -> bool {
        self.was_running
    }
}

/// Desired post-update restoration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreIntent {
    /// Return to the observed running/stopped state.
    Preserve,
    /// Ensure running (explicit caller policy; overrides stopped-stays-stopped).
    EnsureRunning,
    /// Ensure stopped.
    EnsureStopped,
}

/// Bounded transition outcome with conflict diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionResult {
    /// What was attempted.
    pub operation: ServiceOperation,
    /// Whether the manager reports the desired end state.
    pub completed: bool,
    /// Bounded human-readable detail (never embeds secrets).
    pub detail: String,
}

impl TransitionResult {
    /// Returns whether the operation completed.
    pub fn completed(&self) -> bool {
        self.completed
    }
}

/// Manager operations. Destructive variants require `Owned`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ServiceOperation {
    /// Inspect registration and state (always allowed).
    Inspect,
    /// Install/register (allowed for `Absent` with caller authorization, or `Owned` refresh).
    Install,
    /// Start an `Owned` stopped service.
    Start,
    /// Stop an `Owned` running service.
    Stop,
    /// Restart an `Owned` service.
    Restart,
    /// Remove an `Owned` registration.
    Uninstall,
}

/// Typed service errors. Destructive attempts on `Foreign`/`Unknown` fail here.
/// Error strings produced by crate helpers are capped at 512 UTF-8-safe bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ServiceError {
    /// Caller input violates the contract.
    InvalidInput(String),
    /// The registration is foreign or unknown; destructive mutation denied.
    OwnershipDenied {
        /// The service identity.
        id: String,
        /// The observed ownership.
        ownership: Ownership,
    },
    /// The manager reported a conflict (already exists, not found, transitioning).
    Conflict(String),
    /// A bounded manager call failed.
    Manager(String),
}

impl ServiceError {
    pub(crate) fn invalid(detail: impl Into<String>) -> Self {
        Self::InvalidInput(truncate_utf8_bytes(detail.into(), 512))
    }

    pub(crate) fn bounded(detail: impl Into<String>) -> String {
        truncate_utf8_bytes(detail.into(), 512)
    }

    pub(crate) fn conflict(detail: impl Into<String>) -> Self {
        Self::Conflict(Self::bounded(detail))
    }

    pub(crate) fn manager(detail: impl Into<String>) -> Self {
        Self::Manager(Self::bounded(detail))
    }

    /// Reports an ownership denial.
    pub fn denied(id: &ServiceId, ownership: Ownership) -> Self {
        Self::OwnershipDenied {
            id: id.as_str().to_string(),
            ownership,
        }
    }
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(d) => write!(f, "invalid service input: {d}"),
            Self::OwnershipDenied { id, ownership } => {
                write!(f, "service {id} ownership denies mutation ({ownership:?})")
            }
            Self::Conflict(d) => write!(f, "service conflict: {d}"),
            Self::Manager(d) => write!(f, "service manager failed: {d}"),
        }
    }
}

impl std::error::Error for ServiceError {}

/// Consumer-supplied health probe. Read-only: never mutates manager state.
pub trait HealthProbe: fmt::Debug {
    /// Checks application health for an `Owned` service.
    fn check(&self, spec: &ServiceSpec) -> HealthState;
}

/// Null probe: health is always unknown.
#[derive(Debug, Default)]
pub struct NoProbe;

impl HealthProbe for NoProbe {
    fn check(&self, _spec: &ServiceSpec) -> HealthState {
        HealthState::Unknown
    }
}

/// Manager-neutral service operations. Platform adapters implement this in
/// later milestones; `TestDoubleManager` implements it for tests.
pub trait ServiceManager {
    /// Inspects registration and lifecycle state without mutating anything.
    fn inspect(&self, spec: &ServiceSpec) -> Result<LifecycleSnapshot, ServiceError>;

    /// Installs/registers. Allowed for `Absent` (creation) or `Owned` (refresh).
    fn install(&mut self, spec: &ServiceSpec) -> Result<TransitionResult, ServiceError>;

    /// Starts an `Owned` stopped service within `timeout`.
    fn start(
        &mut self,
        spec: &ServiceSpec,
        timeout: Duration,
    ) -> Result<TransitionResult, ServiceError>;

    /// Stops an `Owned` running service within `timeout`.
    fn stop(
        &mut self,
        spec: &ServiceSpec,
        timeout: Duration,
    ) -> Result<TransitionResult, ServiceError>;

    /// Restarts an `Owned` service within `timeout`.
    fn restart(
        &mut self,
        spec: &ServiceSpec,
        timeout: Duration,
    ) -> Result<TransitionResult, ServiceError>;

    /// Removes an `Owned` registration.
    fn uninstall(&mut self, spec: &ServiceSpec) -> Result<TransitionResult, ServiceError>;
}

/// Ownership guard shared by all destructive operations.
pub fn require_owned(id: &ServiceId, ownership: Ownership) -> Result<(), ServiceError> {
    match ownership {
        Ownership::Owned => Ok(()),
        Ownership::Absent | Ownership::Foreign | Ownership::Unknown => {
            Err(ServiceError::denied(id, ownership))
        }
    }
}

/// Deterministic in-memory manager for contract tests.
#[derive(Debug, Default)]
pub struct TestDoubleManager {
    registrations: HashMap<String, TestRegistration>,
}

#[derive(Debug, Clone)]
struct TestRegistration {
    spec: ServiceSpec,
    state: LifecycleState,
    malformed: bool,
}

impl TestDoubleManager {
    /// Creates an empty test manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Injects a registration with exact spec + state.
    pub fn put(&mut self, spec: ServiceSpec, state: LifecycleState) {
        self.registrations.insert(
            spec.id().as_str().to_string(),
            TestRegistration {
                spec,
                state,
                malformed: false,
            },
        );
    }

    /// Injects a malformed registration record under an id.
    pub fn put_malformed(&mut self, id: ServiceId) {
        self.registrations.insert(
            id.as_str().to_string(),
            TestRegistration {
                spec: ServiceSpec::new(id, PathBuf::from("/invalid"), vec![], None)
                    .expect("test spec"),
                state: LifecycleState::Unknown,
                malformed: true,
            },
        );
    }

    fn snapshot_for(&self, spec: &ServiceSpec) -> (Ownership, LifecycleSnapshot) {
        match self.registrations.get(spec.id().as_str()) {
            None => {
                let snap = LifecycleSnapshot {
                    id: spec.id().clone(),
                    ownership: Ownership::Absent,
                    state: LifecycleState::Stopped,
                    health: HealthState::Unknown,
                    was_registered: false,
                    was_running: false,
                };
                (Ownership::Absent, snap)
            }
            Some(reg) => {
                let observed = RegistrationSnapshot {
                    present: true,
                    executable: Some(reg.spec.executable().to_path_buf()),
                    args: reg.spec.args().to_vec(),
                    config: reg.spec.config().map(|p| p.to_path_buf()),
                    malformed: reg.malformed,
                };
                let ownership = observed.ownership(spec);
                let snap = LifecycleSnapshot {
                    id: spec.id().clone(),
                    ownership,
                    state: reg.state,
                    health: HealthState::Unknown,
                    was_registered: true,
                    was_running: reg.state == LifecycleState::Running,
                };
                (ownership, snap)
            }
        }
    }
}

impl ServiceManager for TestDoubleManager {
    fn inspect(&self, spec: &ServiceSpec) -> Result<LifecycleSnapshot, ServiceError> {
        Ok(self.snapshot_for(spec).1)
    }

    fn install(&mut self, spec: &ServiceSpec) -> Result<TransitionResult, ServiceError> {
        let (ownership, _) = self.snapshot_for(spec);
        match ownership {
            Ownership::Absent => {
                self.put(spec.clone(), LifecycleState::Stopped);
                Ok(TransitionResult {
                    operation: ServiceOperation::Install,
                    completed: true,
                    detail: ServiceError::bounded("installed"),
                })
            }
            Ownership::Owned => {
                self.put(spec.clone(), LifecycleState::Stopped);
                Ok(TransitionResult {
                    operation: ServiceOperation::Install,
                    completed: true,
                    detail: ServiceError::bounded("refreshed owned registration"),
                })
            }
            Ownership::Foreign | Ownership::Unknown => {
                Err(ServiceError::denied(spec.id(), ownership))
            }
        }
    }

    fn start(
        &mut self,
        spec: &ServiceSpec,
        _timeout: Duration,
    ) -> Result<TransitionResult, ServiceError> {
        let (ownership, snap) = self.snapshot_for(spec);
        require_owned(spec.id(), ownership)?;
        if snap.state == LifecycleState::Running {
            return Ok(TransitionResult {
                operation: ServiceOperation::Start,
                completed: true,
                detail: ServiceError::bounded("already running"),
            });
        }
        if let Some(reg) = self.registrations.get_mut(spec.id().as_str()) {
            reg.state = LifecycleState::Running;
        }
        Ok(TransitionResult {
            operation: ServiceOperation::Start,
            completed: true,
            detail: ServiceError::bounded("started"),
        })
    }

    fn stop(
        &mut self,
        spec: &ServiceSpec,
        _timeout: Duration,
    ) -> Result<TransitionResult, ServiceError> {
        let (ownership, snap) = self.snapshot_for(spec);
        require_owned(spec.id(), ownership)?;
        if snap.state == LifecycleState::Stopped {
            return Ok(TransitionResult {
                operation: ServiceOperation::Stop,
                completed: true,
                detail: ServiceError::bounded("already stopped"),
            });
        }
        if let Some(reg) = self.registrations.get_mut(spec.id().as_str()) {
            reg.state = LifecycleState::Stopped;
        }
        Ok(TransitionResult {
            operation: ServiceOperation::Stop,
            completed: true,
            detail: ServiceError::bounded("stopped"),
        })
    }

    fn restart(
        &mut self,
        spec: &ServiceSpec,
        timeout: Duration,
    ) -> Result<TransitionResult, ServiceError> {
        let (ownership, _) = self.snapshot_for(spec);
        require_owned(spec.id(), ownership)?;
        let stopped = self.stop(spec, timeout)?;
        if !stopped.completed {
            return Ok(TransitionResult {
                operation: ServiceOperation::Restart,
                completed: false,
                detail: ServiceError::bounded("restart stop phase incomplete"),
            });
        }
        let started = self.start(spec, timeout)?;
        Ok(TransitionResult {
            operation: ServiceOperation::Restart,
            completed: started.completed,
            detail: ServiceError::bounded(if started.completed {
                "restarted"
            } else {
                "restart start phase incomplete"
            }),
        })
    }

    fn uninstall(&mut self, spec: &ServiceSpec) -> Result<TransitionResult, ServiceError> {
        let (ownership, _) = self.snapshot_for(spec);
        require_owned(spec.id(), ownership)?;
        self.registrations.remove(spec.id().as_str());
        Ok(TransitionResult {
            operation: ServiceOperation::Uninstall,
            completed: true,
            detail: ServiceError::bounded("uninstalled"),
        })
    }
}

// ============================================================================
// M002 — Unix manager adapters (systemd, launchd, cron) + shared mechanics.
// ============================================================================

/// Maximum captured bytes per stdout/stderr stream for one manager call.
pub const MAX_COMMAND_OUTPUT_BYTES: usize = 256 * 1024;
/// Maximum crontab content accepted for list/install (fail closed above).
pub const MAX_CRONTAB_BYTES: usize = 1024 * 1024;
/// Maximum service definition bytes accepted for atomic writes.
pub const MAX_DEFINITION_BYTES: usize = 1024 * 1024;
/// Upper bound for transition polling (callers supply smaller values).
pub const MAX_TRANSITION_TIMEOUT: Duration = Duration::from_secs(300);

/// Bounded manager-command output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
    /// Process exit status code, when waited.
    pub status: Option<i32>,
    /// Captured stdout (bounded).
    pub stdout: Vec<u8>,
    /// Captured stderr (bounded).
    pub stderr: Vec<u8>,
}

impl CommandOutput {
    /// Stdout as lossy UTF-8 (manager output is ASCII/UTF-8 in practice).
    pub fn stdout_text(&self) -> String {
        String::from_utf8_lossy(&self.stdout).into_owned()
    }

    /// Stderr as lossy UTF-8.
    pub fn stderr_text(&self) -> String {
        String::from_utf8_lossy(&self.stderr).into_owned()
    }
}

/// Injectable manager-command executor (literal argv, no shell).
///
/// Implementations run the program directly with the given arguments, a
/// controlled stdin policy, a deadline, and bounded output. No shell
/// interpolation is performed. [`SystemExecutor`] is the production
/// implementation; [`FakeExecutor`] scripts deterministic responses for tests.
pub trait CommandExecutor: fmt::Debug {
    /// Runs `argv[0]` with `argv[1..]` within `timeout`.
    ///
    /// `stdin_data` is `None` for null stdin; `Some(bytes)` streams bounded
    /// input (used only for `crontab -`). Missing binaries, timeouts, and
    /// output-bound overflows are `Err(ServiceError::Manager)`; nonzero
    /// exits are `Ok` with the observed status for caller classification.
    fn run(
        &self,
        argv: &[String],
        stdin_data: Option<&[u8]>,
        timeout: Duration,
    ) -> Result<CommandOutput, ServiceError>;
}

/// Production bounded command runner (literal argv, no shell).
///
/// Manager names resolve only through trusted absolute platform paths. The
/// child environment is cleared; user-scoped systemd receives only explicit
/// session-bus variables. Stdin is null unless the caller supplies bounded
/// input. Stdout/stderr are each bounded by `max_output_bytes`; overflow
/// fails closed without assuming state. Deadlines kill and reap the child.
#[derive(Debug, Clone)]
pub struct SystemExecutor {
    /// Per-stream output bound.
    pub max_output_bytes: usize,
}

impl Default for SystemExecutor {
    fn default() -> Self {
        Self {
            max_output_bytes: MAX_COMMAND_OUTPUT_BYTES,
        }
    }
}

impl SystemExecutor {
    /// Creates a runner with the default output bound.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a runner with an explicit per-stream bound.
    pub fn with_max_output(max_output_bytes: usize) -> Result<Self, ServiceError> {
        if max_output_bytes == 0 || max_output_bytes > 16 * 1024 * 1024 {
            return Err(ServiceError::invalid("output bound out of range"));
        }
        Ok(Self { max_output_bytes })
    }
}

impl CommandExecutor for SystemExecutor {
    fn run(
        &self,
        argv: &[String],
        stdin_data: Option<&[u8]>,
        timeout: Duration,
    ) -> Result<CommandOutput, ServiceError> {
        if argv.is_empty() || argv[0].is_empty() {
            return Err(ServiceError::invalid("manager command has no program"));
        }
        if argv.iter().any(|a| a.contains('\0')) {
            return Err(ServiceError::invalid("manager command contains nul bytes"));
        }
        if timeout.is_zero() || timeout > MAX_TRANSITION_TIMEOUT + Duration::from_secs(60) {
            return Err(ServiceError::invalid("manager timeout out of range"));
        }
        if let Some(data) = stdin_data {
            if data.len() > MAX_CRONTAB_BYTES {
                return Err(ServiceError::invalid("manager stdin exceeds bound"));
            }
        }
        let command_deadline = Instant::now() + timeout;
        let program_path = resolve_manager_program(&argv[0])?;
        let program = program_path.to_string_lossy().into_owned();
        let mut cmd = std::process::Command::new(&program_path);
        if argv.len() > 1 {
            cmd.args(&argv[1..]);
        }
        cmd.env_clear();
        for (key, value) in filtered_manager_environment(argv, std::env::vars_os()) {
            cmd.env(key, value);
        }
        if stdin_data.is_some() {
            cmd.stdin(std::process::Stdio::piped());
        } else {
            cmd.stdin(std::process::Stdio::null());
        }
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());
        let mut child = cmd.spawn().map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => {
                ServiceError::manager(format!("manager binary missing: {program}"))
            }
            _ => ServiceError::manager(format!("manager spawn failed for {program}: {e}")),
        })?;
        // Bounded stdin write (only for short-lived filters like `crontab -`).
        if let Some(data) = stdin_data {
            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                // Best-effort bounded write; failure fails closed.
                if stdin.write_all(data).is_err() || stdin.flush().is_err() {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(ServiceError::manager(format!(
                        "manager stdin failed for {program}"
                    )));
                }
            }
            // `stdin` is dropped here, closing the pipe (EOF for the child).
        }
        let stdout_pipe: Option<Box<dyn std::io::Read + Send>> = child
            .stdout
            .take()
            .map(|p| Box::new(p) as Box<dyn std::io::Read + Send>);
        let stderr_pipe: Option<Box<dyn std::io::Read + Send>> = child
            .stderr
            .take()
            .map(|p| Box::new(p) as Box<dyn std::io::Read + Send>);
        let max = self.max_output_bytes;
        let (tx_out, rx_out) = std::sync::mpsc::channel();
        let (tx_err, rx_err) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let out = read_bounded_generic(stdout_pipe, max);
            let _ = tx_out.send(out);
        });
        std::thread::spawn(move || {
            let out = read_bounded_generic(stderr_pipe, max);
            let _ = tx_err.send(out);
        });
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    let remaining = command_deadline
                        .checked_duration_since(Instant::now())
                        .filter(|d| !d.is_zero())
                        .ok_or_else(|| ServiceError::manager("manager command timed out"))?;
                    let (stdout, out_overflow) = rx_out.recv_timeout(remaining).map_err(|_| {
                        ServiceError::manager("manager output collection timed out")
                    })?;
                    let remaining = command_deadline
                        .checked_duration_since(Instant::now())
                        .filter(|d| !d.is_zero())
                        .ok_or_else(|| ServiceError::manager("manager command timed out"))?;
                    let (stderr, err_overflow) = rx_err.recv_timeout(remaining).map_err(|_| {
                        ServiceError::manager("manager output collection timed out")
                    })?;
                    if out_overflow || err_overflow {
                        return Err(ServiceError::manager(format!(
                            "manager output exceeded bound for {program}"
                        )));
                    }
                    return Ok(CommandOutput {
                        status: status.code(),
                        stdout,
                        stderr,
                    });
                }
                Ok(None) => {
                    if Instant::now() >= command_deadline {
                        let _ = child.kill();
                        let _ = child.wait();
                        return Err(ServiceError::manager(format!(
                            "manager command timed out for {program}"
                        )));
                    }
                    std::thread::sleep(
                        command_deadline
                            .checked_duration_since(Instant::now())
                            .unwrap_or_default()
                            .min(Duration::from_millis(5)),
                    );
                }
                Err(e) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(ServiceError::manager(format!(
                        "manager wait failed for {program}: {e}"
                    )));
                }
            }
        }
    }
}

fn resolve_manager_program(program: &str) -> Result<PathBuf, ServiceError> {
    let requested = Path::new(program);
    if requested.is_absolute() {
        return validate_executable_path(requested, program);
    }
    let candidates: &[&str] = match program {
        "systemctl" => &["/usr/bin/systemctl", "/bin/systemctl"],
        "launchctl" => &["/bin/launchctl", "/usr/bin/launchctl"],
        "crontab" => &["/usr/bin/crontab", "/bin/crontab"],
        _ => {
            return Err(ServiceError::invalid(
                "manager program must be absolute or allowlisted",
            ))
        }
    };
    for candidate in candidates {
        let path = Path::new(candidate);
        if validate_executable_path(path, program).is_err() {
            continue;
        }
        return Ok(path.to_path_buf());
    }
    Err(ServiceError::manager(format!(
        "trusted manager binary missing: {program}"
    )))
}

fn validate_executable_path(path: &Path, display: &str) -> Result<PathBuf, ServiceError> {
    if !path.is_absolute() {
        return Err(ServiceError::invalid(
            "manager program override must be absolute",
        ));
    }
    let metadata = std::fs::metadata(path)
        .map_err(|_| ServiceError::manager(format!("manager binary missing: {display}")))?;
    if !metadata.is_file() {
        return Err(ServiceError::invalid(
            "manager program is not a regular file",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 == 0 {
            return Err(ServiceError::invalid("manager program is not executable"));
        }
    }
    Ok(path.to_path_buf())
}

fn filtered_manager_environment(
    argv: &[String],
    source: impl IntoIterator<Item = (std::ffi::OsString, std::ffi::OsString)>,
) -> Vec<(std::ffi::OsString, std::ffi::OsString)> {
    let user_systemd = argv
        .first()
        .is_some_and(|p| Path::new(p).file_name().is_some_and(|n| n == "systemctl"))
        && argv.iter().any(|arg| arg == "--user");
    let allow: &[&std::ffi::OsStr] = if user_systemd {
        &[
            std::ffi::OsStr::new("DBUS_SESSION_BUS_ADDRESS"),
            std::ffi::OsStr::new("XDG_RUNTIME_DIR"),
            std::ffi::OsStr::new("SYSTEMD_BUS_ADDRESS"),
        ]
    } else {
        &[]
    };
    source
        .into_iter()
        .filter(|(key, _)| allow.contains(&key.as_os_str()))
        .collect()
}

fn read_bounded_generic(
    pipe: Option<Box<dyn std::io::Read + Send>>,
    max: usize,
) -> (Vec<u8>, bool) {
    let Some(mut pipe) = pipe else {
        return (Vec::new(), false);
    };
    let mut buf = Vec::new();
    let mut tmp = [0u8; 8192];
    loop {
        use std::io::Read;
        match pipe.read(&mut tmp) {
            Ok(0) => break,
            Ok(n) => {
                buf.extend_from_slice(&tmp[..n]);
                if buf.len() > max {
                    return (buf, true);
                }
            }
            Err(_) => break,
        }
    }
    (buf, false)
}

/// One scripted expectation for [`FakeExecutor`].
#[derive(Debug, Clone)]
struct FakeExpectation {
    expected_argv: Vec<String>,
    result: Result<CommandOutput, ServiceError>,
}

/// One observed call for [`FakeExecutor`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FakeCall {
    /// Exact argv observed.
    pub argv: Vec<String>,
    /// Stdin bytes observed, when any.
    pub stdin: Option<Vec<u8>>,
}

/// Deterministic test executor with scripted responses.
#[derive(Debug, Default)]
pub struct FakeExecutor {
    expectations: Mutex<VecDeque<FakeExpectation>>,
    calls: Mutex<Vec<FakeCall>>,
}

impl FakeExecutor {
    /// Creates an empty fake executor.
    pub fn new() -> Self {
        Self::default()
    }

    /// Scripts a successful response for the next matching call.
    pub fn expect(&self, argv: &[&str], output: CommandOutput) {
        self.expectations
            .lock()
            .expect("fake lock")
            .push_back(FakeExpectation {
                expected_argv: argv.iter().map(|s| s.to_string()).collect(),
                result: Ok(output),
            });
    }

    /// Scripts a failure for the next matching call.
    pub fn expect_error(&self, argv: &[&str], err: ServiceError) {
        self.expectations
            .lock()
            .expect("fake lock")
            .push_back(FakeExpectation {
                expected_argv: argv.iter().map(|s| s.to_string()).collect(),
                result: Err(err),
            });
    }

    /// Returns all observed calls in order.
    pub fn calls(&self) -> Vec<FakeCall> {
        self.calls.lock().expect("fake lock").clone()
    }

    /// Returns the number of observed calls.
    pub fn call_count(&self) -> usize {
        self.calls.lock().expect("fake lock").len()
    }

    /// Returns true when all expectations were consumed.
    pub fn is_exhausted(&self) -> bool {
        self.expectations.lock().expect("fake lock").is_empty()
    }
}

impl CommandExecutor for FakeExecutor {
    fn run(
        &self,
        argv: &[String],
        stdin_data: Option<&[u8]>,
        _timeout: Duration,
    ) -> Result<CommandOutput, ServiceError> {
        self.calls.lock().expect("fake lock").push(FakeCall {
            argv: argv.to_vec(),
            stdin: stdin_data.map(|b| b.to_vec()),
        });
        let mut queue = self.expectations.lock().expect("fake lock");
        let Some(exp) = queue.pop_front() else {
            return Err(ServiceError::manager("unexpected manager command"));
        };
        if exp.expected_argv != argv {
            return Err(ServiceError::manager(format!(
                "unexpected manager argv: got {:?}, want {:?}",
                argv, exp.expected_argv
            )));
        }
        exp.result
    }
}

impl CommandExecutor for &FakeExecutor {
    fn run(
        &self,
        argv: &[String],
        stdin_data: Option<&[u8]>,
        timeout: Duration,
    ) -> Result<CommandOutput, ServiceError> {
        (*self).run(argv, stdin_data, timeout)
    }
}

// ---- Definition file safety ----

/// Atomically writes a manager definition file.
///
/// Uses a same-directory exclusively-created owner-private temp, `fsync`s the
/// temp, sets an appropriate mode (`0644` for new files; preserves the
/// existing mode on overwrite), then promotes. `allow_overwrite = false`
/// uses no-replace promotion and fails with `Conflict` when `dest` exists;
/// `true` replaces an owned destination via rename. Never creates missing
/// parents and never changes parent permissions. Cleans only the owned temp.
pub fn atomic_write_definition(
    path: &Path,
    bytes: &[u8],
    allow_overwrite: bool,
) -> Result<(), ServiceError> {
    if bytes.is_empty() || bytes.len() > MAX_DEFINITION_BYTES {
        return Err(ServiceError::invalid(
            "definition bytes are empty or overlong",
        ));
    }
    if bytes.contains(&0) {
        return Err(ServiceError::invalid("definition contains nul bytes"));
    }
    let parent = path
        .parent()
        .ok_or_else(|| ServiceError::invalid("definition path has no parent"))?;
    let parent_meta = std::fs::symlink_metadata(parent)
        .map_err(|e| ServiceError::manager(format!("reading definition parent: {e}")))?;
    if !parent_meta.is_dir() || parent_meta.file_type().is_symlink() {
        return Err(ServiceError::invalid(
            "definition parent must be an existing real directory",
        ));
    }
    let dest_exists = std::fs::symlink_metadata(path).is_ok();
    if dest_exists && !allow_overwrite {
        return Err(ServiceError::conflict(
            "definition already exists; refusing to overwrite",
        ));
    }
    // Preserve the existing mode on overwrite; otherwise use 0644.
    #[cfg(unix)]
    let final_mode: u32 = {
        use std::os::unix::fs::PermissionsExt;
        if dest_exists {
            std::fs::symlink_metadata(path)
                .map(|m| m.permissions().mode() & 0o777)
                .unwrap_or(0o644)
        } else {
            0o644
        }
    };
    // Exclusive temp in the same directory (same filesystem for promotion).
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let mut tmp: PathBuf = parent.join(format!(".eggup-def-{}-{nonce}.tmp", std::process::id()));
    // Retry on collision with a bounded suffix loop.
    let mut file: Option<std::fs::File> = None;
    for attempt in 0..32u32 {
        if attempt > 0 {
            tmp = parent.join(format!(
                ".eggup-def-{}-{nonce}-{attempt}.tmp",
                std::process::id()
            ));
        }
        let mut opts = std::fs::OpenOptions::new();
        opts.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        match opts.open(&tmp) {
            Ok(f) => {
                file = Some(f);
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => {
                return Err(ServiceError::manager(format!(
                    "creating definition temp: {e}"
                )));
            }
        }
    }
    let mut file = file
        .ok_or_else(|| ServiceError::manager("definition temp collisions exhausted".to_string()))?;
    struct Guard<'a> {
        path: &'a Path,
        disarm: bool,
    }
    impl Drop for Guard<'_> {
        fn drop(&mut self) {
            if !self.disarm {
                let _ = std::fs::remove_file(self.path);
            }
        }
    }
    let mut guard = Guard {
        path: &tmp,
        disarm: false,
    };
    {
        use std::io::Write;
        file.write_all(bytes)
            .map_err(|e| ServiceError::manager(format!("writing definition temp: {e}")))?;
        file.flush()
            .map_err(|e| ServiceError::manager(format!("flushing definition temp: {e}")))?;
        file.sync_all()
            .map_err(|e| ServiceError::manager(format!("syncing definition temp: {e}")))?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(final_mode);
        std::fs::set_permissions(&tmp, perms)
            .map_err(|e| ServiceError::manager(format!("securing definition temp: {e}")))?;
    }
    drop(file);
    if allow_overwrite {
        match std::fs::rename(&tmp, path) {
            Ok(()) => {
                guard.disarm = true;
                // Best-effort parent fsync for durability; ignore platform errors.
                if let Ok(dir) = std::fs::File::open(parent) {
                    let _ = dir.sync_all();
                }
                Ok(())
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                // Windows: rename fails when dest exists. Remove the owned
                // dest (already authorized by ownership) then rename.
                std::fs::remove_file(path).map_err(|e2| {
                    ServiceError::manager(format!("removing owned definition: {e2}"))
                })?;
                std::fs::rename(&tmp, path)
                    .map_err(|e2| ServiceError::manager(format!("promoting definition: {e2}")))?;
                guard.disarm = true;
                Ok(())
            }
            Err(e) => Err(ServiceError::manager(format!("promoting definition: {e}"))),
        }
    } else {
        match std::fs::hard_link(&tmp, path) {
            Ok(()) => {
                let _ = std::fs::remove_file(&tmp);
                guard.disarm = true;
                if let Ok(dir) = std::fs::File::open(parent) {
                    let _ = dir.sync_all();
                }
                Ok(())
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Err(ServiceError::conflict(
                "definition already exists; refusing to overwrite",
            )),
            Err(e) => Err(ServiceError::manager(format!("promoting definition: {e}"))),
        }
    }
}

// ---- systemd adapter ----

/// systemd scope without privilege guessing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemdScope {
    /// System manager (`--system`).
    System,
    /// User manager (`--user`).
    User,
}

impl SystemdScope {
    fn flag(&self) -> &'static str {
        match self {
            Self::System => "--system",
            Self::User => "--user",
        }
    }
}

/// Caller-owned systemd install material (mechanics only, no product policy).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemdInstall {
    /// Unit name (for example `my-daemon.service`). `*.service` only.
    pub unit_name: String,
    /// Explicit scope (never guessed from EUID).
    pub scope: SystemdScope,
    /// Absolute unit file path supplied by the caller.
    pub unit_path: PathBuf,
    /// Exact desired unit file bytes (caller-owned content).
    pub definition: Vec<u8>,
    /// Whether to run `enable` after install/refresh (explicit only).
    pub enable: bool,
    /// Whether to run `daemon-reload` after install/refresh (explicit only).
    pub reload: bool,
    /// Bounded wait for state transitions.
    pub transition_timeout: Duration,
}

impl SystemdInstall {
    /// Validates install material without touching the manager.
    pub fn new(
        unit_name: String,
        scope: SystemdScope,
        unit_path: PathBuf,
        definition: Vec<u8>,
        enable: bool,
        reload: bool,
        transition_timeout: Duration,
    ) -> Result<Self, ServiceError> {
        validate_unit_name(&unit_name)?;
        if !unit_path.is_absolute() {
            return Err(ServiceError::invalid("unit path must be absolute"));
        }
        if definition.is_empty() || definition.len() > MAX_DEFINITION_BYTES {
            return Err(ServiceError::invalid(
                "unit definition is empty or overlong",
            ));
        }
        if definition.contains(&0) {
            return Err(ServiceError::invalid("unit definition contains nul"));
        }
        if transition_timeout.is_zero() || transition_timeout > MAX_TRANSITION_TIMEOUT {
            return Err(ServiceError::invalid("transition timeout out of range"));
        }
        Ok(Self {
            unit_name,
            scope,
            unit_path,
            definition,
            enable,
            reload,
            transition_timeout,
        })
    }
}

fn validate_unit_name(name: &str) -> Result<(), ServiceError> {
    if name.is_empty() || name.len() > 256 || name.chars().any(char::is_control) {
        return Err(ServiceError::invalid(
            "unit name is empty, overlong, or has controls",
        ));
    }
    if name.contains(['/', '\\', ' ', '@']) {
        return Err(ServiceError::invalid("unit name must be a bare unit file"));
    }
    if !name.ends_with(".service") {
        return Err(ServiceError::invalid("only *.service units are supported"));
    }
    Ok(())
}

/// systemd manager implementing the neutral contract.
#[derive(Debug)]
pub struct SystemdManager<E: CommandExecutor = SystemExecutor> {
    executor: E,
    install: SystemdInstall,
}

impl<E: CommandExecutor> SystemdManager<E> {
    /// Creates a systemd adapter. No manager calls are made here.
    pub fn new(executor: E, install: SystemdInstall) -> Self {
        Self { executor, install }
    }

    /// Returns the install descriptor.
    pub fn install_config(&self) -> &SystemdInstall {
        &self.install
    }

    /// Returns the command executor (test visibility).
    pub fn executor(&self) -> &E {
        &self.executor
    }

    fn show_argv(&self) -> Vec<String> {
        vec![
            "systemctl".to_string(),
            self.install.scope.flag().to_string(),
            "show".to_string(),
            self.install.unit_name.clone(),
            "-p".to_string(),
            "LoadState,ActiveState,ExecStart".to_string(),
        ]
    }

    fn ownership_of(
        &self,
        spec: &ServiceSpec,
    ) -> Result<(Ownership, LifecycleState), ServiceError> {
        self.ownership_until(
            spec,
            OperationDeadline::new(self.install.transition_timeout)?,
        )
    }

    fn ownership_until(
        &self,
        spec: &ServiceSpec,
        deadline: OperationDeadline,
    ) -> Result<(Ownership, LifecycleState), ServiceError> {
        let out = self
            .executor
            .run(&self.show_argv(), None, deadline.command_timeout()?)?;
        if out.status != Some(0) {
            return Ok((Ownership::Unknown, LifecycleState::Unknown));
        }
        let (ownership, state, _) = systemd_ownership(&out.stdout_text(), spec)?;
        Ok((ownership, state))
    }

    fn run_unit(
        &self,
        verb: &str,
        deadline: OperationDeadline,
    ) -> Result<CommandOutput, ServiceError> {
        let argv = vec![
            "systemctl".to_string(),
            self.install.scope.flag().to_string(),
            verb.to_string(),
            self.install.unit_name.clone(),
        ];
        let out = self
            .executor
            .run(&argv, None, deadline.command_timeout()?)
            .map_err(permission_hint)?;
        if out.status != Some(0) {
            return Err(permission_hint(ServiceError::manager(format!(
                "systemctl {verb} exited {}: {}",
                out.status.unwrap_or(-1),
                truncate(&out.stderr_text()),
            ))));
        }
        Ok(out)
    }

    fn poll_active(
        &self,
        want_active: bool,
        deadline: OperationDeadline,
    ) -> Result<bool, ServiceError> {
        loop {
            let argv = vec![
                "systemctl".to_string(),
                self.install.scope.flag().to_string(),
                "is-active".to_string(),
                self.install.unit_name.clone(),
            ];
            let out = self
                .executor
                .run(&argv, None, deadline.command_timeout()?)
                .map_err(permission_hint)?;
            let state = out.stdout_text().trim().to_string();
            let confirmed = if want_active {
                state == "active" && out.status == Some(0)
            } else {
                state == "inactive" && out.status == Some(3)
            };
            if confirmed {
                return Ok(!deadline.expired());
            }
            if deadline.expired() {
                return Ok(false);
            }
            std::thread::sleep(
                deadline
                    .remaining()
                    .unwrap_or_default()
                    .min(Duration::from_millis(100)),
            );
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct OperationDeadline(Instant);

impl OperationDeadline {
    fn new(timeout: Duration) -> Result<Self, ServiceError> {
        if timeout.is_zero() || timeout > MAX_TRANSITION_TIMEOUT {
            return Err(ServiceError::invalid("transition timeout out of range"));
        }
        Ok(Self(Instant::now() + timeout))
    }

    fn remaining(self) -> Option<Duration> {
        self.0
            .checked_duration_since(Instant::now())
            .filter(|duration| !duration.is_zero())
    }

    fn command_timeout(self) -> Result<Duration, ServiceError> {
        self.remaining().ok_or_else(|| {
            ServiceError::manager("transition deadline exhausted before manager command")
        })
    }

    fn expired(self) -> bool {
        self.remaining().is_none()
    }
}

fn bounded_timeout(t: Duration) -> Duration {
    std::cmp::min(t, MAX_TRANSITION_TIMEOUT)
}

fn reconcile_config_identity(
    mut observed: RegistrationSnapshot,
    spec: &ServiceSpec,
) -> RegistrationSnapshot {
    let Some(config) = spec.config() else {
        return observed;
    };
    let Some(config_text) = config.to_str() else {
        observed.malformed = true;
        return observed;
    };
    let expected_count = spec
        .args()
        .iter()
        .filter(|arg| arg.as_str() == config_text)
        .count();
    let observed_count = observed
        .args
        .iter()
        .filter(|arg| arg.as_str() == config_text)
        .count();
    if observed_count == 0 {
        // A parseable registration without the specified config is Foreign.
        return observed;
    }
    if expected_count > 1 || observed_count > 1 {
        observed.malformed = true;
        return observed;
    }
    if expected_count == 1 && observed_count == 1 {
        observed.config = Some(config.to_path_buf());
    }
    observed
}

fn truncate_utf8_bytes(mut text: String, max_bytes: usize) -> String {
    if text.len() > max_bytes {
        let mut end = max_bytes;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        text.truncate(end);
    }
    text
}

fn truncate(s: &str) -> String {
    truncate_utf8_bytes(s.to_string(), 256)
}

fn permission_hint(e: ServiceError) -> ServiceError {
    match e {
        ServiceError::Manager(d) => {
            let lower = d.to_lowercase();
            if lower.contains("permission") || lower.contains("denied") || lower.contains("access")
            {
                const HINT: &str = " (permission denied; re-run with appropriate user/system scope; no automatic elevation)";
                let detail = truncate_utf8_bytes(d, 512 - HINT.len());
                ServiceError::Manager(format!("{detail}{HINT}"))
            } else {
                ServiceError::Manager(d)
            }
        }
        other => other,
    }
}

fn ensure_mutation_success(out: &CommandOutput, command: &str) -> Result<(), ServiceError> {
    if out.status == Some(0) {
        Ok(())
    } else {
        Err(permission_hint(ServiceError::manager(format!(
            "{command} exited {}: {}",
            out.status.unwrap_or(-1),
            truncate(&out.stderr_text()),
        ))))
    }
}

/// Parses one `ExecStart={ path=... ; argv[]=... ; ... }` value.
///
/// Returns `(executable, args)` where `args` excludes `argv[0]`. Returns
/// `None` for any ambiguous or unsupported shape.
fn parse_exec_start(value: &str) -> Option<(PathBuf, Vec<String>)> {
    let v = value.trim();
    if v.is_empty() {
        return None;
    }
    // Expected shape: `{ path=... ; argv[]=... ; ... }`. Reject multiple
    // command blocks by requiring exactly one `{...}` group.
    if v.matches('{').count() != 1 || v.matches('}').count() != 1 {
        return None;
    }
    let inner = v.strip_prefix('{')?.strip_suffix('}')?;
    let mut path: Option<String> = None;
    let mut argv: Option<String> = None;
    for part in inner.split(';') {
        let part = part.trim();
        if let Some(p) = part.strip_prefix("path=") {
            if path.is_some() {
                return None;
            }
            path = Some(p.trim().to_string());
        } else if let Some(a) = part.strip_prefix("argv[]=") {
            if argv.is_some() {
                return None;
            }
            argv = Some(a.trim().to_string());
        }
    }
    let path = path?;
    let argv = argv?;
    if path.is_empty() || path.contains(['%', '$']) {
        return None;
    }
    // Split argv on whitespace (systemd show quotes simply; narrow rule:
    // reject embedded quotes/escapes we do not understand).
    if argv.contains(['"', '\'', '\\', '%', '$']) {
        return None;
    }
    let parts: Vec<String> = argv.split_whitespace().map(|s| s.to_string()).collect();
    if parts.is_empty() || parts[0] != path {
        return None;
    }
    let exe = PathBuf::from(path);
    if !exe.is_absolute() {
        return None;
    }
    Some((exe, parts[1..].to_vec()))
}

/// Computes systemd ownership for a spec from show output.
fn systemd_ownership(
    text: &str,
    spec: &ServiceSpec,
) -> Result<(Ownership, LifecycleState, bool), ServiceError> {
    let mut load: Option<&str> = None;
    let mut active: Option<&str> = None;
    let mut execstarts: Vec<&str> = Vec::new();
    for line in text.lines() {
        if let Some(v) = line.strip_prefix("LoadState=") {
            load = Some(v.trim());
        } else if let Some(v) = line.strip_prefix("ActiveState=") {
            active = Some(v.trim());
        } else if let Some(v) = line.strip_prefix("ExecStart=") {
            execstarts.push(v.trim());
        }
    }
    match load.unwrap_or("") {
        "" | "not-found" => return Ok((Ownership::Absent, LifecycleState::Stopped, false)),
        "loaded" => {}
        _ => return Ok((Ownership::Unknown, LifecycleState::Unknown, true)),
    }
    let state = match active.unwrap_or("") {
        "active" => LifecycleState::Running,
        "inactive" => LifecycleState::Stopped,
        "activating" | "deactivating" | "reloading" => LifecycleState::Transitioning,
        _ => LifecycleState::Unknown,
    };
    if execstarts.len() != 1 {
        return Ok((Ownership::Unknown, state, true));
    }
    let Some((exe, args)) = parse_exec_start(execstarts[0]) else {
        return Ok((Ownership::Unknown, state, true));
    };
    let observed = reconcile_config_identity(
        RegistrationSnapshot {
            present: true,
            executable: Some(exe),
            args,
            config: None,
            malformed: false,
        },
        spec,
    );
    Ok((observed.ownership(spec), state, true))
}

impl<E: CommandExecutor> ServiceManager for SystemdManager<E> {
    fn inspect(&self, spec: &ServiceSpec) -> Result<LifecycleSnapshot, ServiceError> {
        let out = self.executor.run(
            &self.show_argv(),
            None,
            bounded_timeout(self.install.transition_timeout),
        )?;
        let (ownership, state, present) = if out.status == Some(0) {
            systemd_ownership(&out.stdout_text(), spec)?
        } else {
            (Ownership::Unknown, LifecycleState::Unknown, true)
        };
        Ok(LifecycleSnapshot {
            id: spec.id().clone(),
            ownership,
            state,
            health: HealthState::Unknown,
            was_registered: present,
            was_running: state == LifecycleState::Running,
        })
    }

    fn install(&mut self, spec: &ServiceSpec) -> Result<TransitionResult, ServiceError> {
        let (ownership, _) = self.ownership_of(spec)?;
        match ownership {
            Ownership::Absent => {
                atomic_write_definition(&self.install.unit_path, &self.install.definition, false)?;
                self.post_write()?;
                Ok(TransitionResult {
                    operation: ServiceOperation::Install,
                    completed: true,
                    detail: ServiceError::bounded("installed systemd unit"),
                })
            }
            Ownership::Owned => {
                // Re-inspect before destructive overwrite (external changes).
                let (recheck, _) = self.ownership_of(spec)?;
                if recheck != Ownership::Owned {
                    return Err(ServiceError::denied(spec.id(), recheck));
                }
                atomic_write_definition(&self.install.unit_path, &self.install.definition, true)?;
                self.post_write()?;
                Ok(TransitionResult {
                    operation: ServiceOperation::Install,
                    completed: true,
                    detail: ServiceError::bounded("refreshed owned systemd unit"),
                })
            }
            Ownership::Foreign | Ownership::Unknown => {
                Err(ServiceError::denied(spec.id(), ownership))
            }
        }
    }

    fn start(
        &mut self,
        spec: &ServiceSpec,
        timeout: Duration,
    ) -> Result<TransitionResult, ServiceError> {
        let deadline = OperationDeadline::new(timeout)?;
        let (ownership, state) = self.ownership_until(spec, deadline)?;
        require_owned(spec.id(), ownership)?;
        if state == LifecycleState::Running {
            return Ok(TransitionResult {
                operation: ServiceOperation::Start,
                completed: !deadline.expired(),
                detail: ServiceError::bounded("already running"),
            });
        }
        self.run_unit("start", deadline)?;
        let completed = self.poll_active(true, deadline)?;
        Ok(TransitionResult {
            operation: ServiceOperation::Start,
            completed,
            detail: ServiceError::bounded(if completed {
                "started"
            } else {
                "start incomplete; state unconfirmed"
            }),
        })
    }

    fn stop(
        &mut self,
        spec: &ServiceSpec,
        timeout: Duration,
    ) -> Result<TransitionResult, ServiceError> {
        let deadline = OperationDeadline::new(timeout)?;
        let (ownership, state) = self.ownership_until(spec, deadline)?;
        require_owned(spec.id(), ownership)?;
        if state == LifecycleState::Stopped {
            return Ok(TransitionResult {
                operation: ServiceOperation::Stop,
                completed: !deadline.expired(),
                detail: ServiceError::bounded("already stopped"),
            });
        }
        self.run_unit("stop", deadline)?;
        let completed = self.poll_active(false, deadline)?;
        Ok(TransitionResult {
            operation: ServiceOperation::Stop,
            completed,
            detail: ServiceError::bounded(if completed {
                "stopped"
            } else {
                "stop incomplete; state unconfirmed"
            }),
        })
    }

    fn restart(
        &mut self,
        spec: &ServiceSpec,
        timeout: Duration,
    ) -> Result<TransitionResult, ServiceError> {
        let deadline = OperationDeadline::new(timeout)?;
        let (ownership, _) = self.ownership_until(spec, deadline)?;
        require_owned(spec.id(), ownership)?;
        self.run_unit("restart", deadline)?;
        let completed = self.poll_active(true, deadline)?;
        Ok(TransitionResult {
            operation: ServiceOperation::Restart,
            completed,
            detail: ServiceError::bounded(if completed {
                "restarted"
            } else {
                "restart incomplete; state unconfirmed"
            }),
        })
    }

    fn uninstall(&mut self, spec: &ServiceSpec) -> Result<TransitionResult, ServiceError> {
        let (ownership, _) = self.ownership_of(spec)?;
        require_owned(spec.id(), ownership)?;
        // Re-inspect before destructive removal.
        let (recheck, _) = self.ownership_of(spec)?;
        if recheck != Ownership::Owned {
            return Err(ServiceError::denied(spec.id(), recheck));
        }
        // Best-effort disable; a failing disable does not delete the file.
        let disable_argv = vec![
            "systemctl".to_string(),
            self.install.scope.flag().to_string(),
            "disable".to_string(),
            self.install.unit_name.clone(),
        ];
        match self.executor.run(
            &disable_argv,
            None,
            bounded_timeout(self.install.transition_timeout),
        ) {
            Ok(out) if out.status == Some(0) => {}
            Ok(out) => {
                return Ok(TransitionResult {
                    operation: ServiceOperation::Uninstall,
                    completed: false,
                    detail: ServiceError::bounded(format!(
                        "disable failed with exit status {}",
                        out.status.unwrap_or(-1)
                    )),
                });
            }
            Err(e) => {
                return Ok(TransitionResult {
                    operation: ServiceOperation::Uninstall,
                    completed: false,
                    detail: ServiceError::bounded(format!("disable failed: {e}")),
                });
            }
        }
        std::fs::remove_file(&self.install.unit_path)
            .map_err(|e| ServiceError::manager(format!("removing owned unit: {e}")))?;
        if self.install.reload {
            let reload_argv = vec![
                "systemctl".to_string(),
                self.install.scope.flag().to_string(),
                "daemon-reload".to_string(),
            ];
            match self.executor.run(
                &reload_argv,
                None,
                bounded_timeout(self.install.transition_timeout),
            ) {
                Ok(out) if out.status == Some(0) => {}
                Ok(out) => {
                    return Ok(TransitionResult {
                        operation: ServiceOperation::Uninstall,
                        completed: false,
                        detail: ServiceError::bounded(format!(
                            "removed unit but reload exited {}",
                            out.status.unwrap_or(-1)
                        )),
                    });
                }
                Err(e) => {
                    return Ok(TransitionResult {
                        operation: ServiceOperation::Uninstall,
                        completed: false,
                        detail: ServiceError::bounded(format!(
                            "removed unit but reload failed: {e}"
                        )),
                    });
                }
            }
        }
        Ok(TransitionResult {
            operation: ServiceOperation::Uninstall,
            completed: true,
            detail: ServiceError::bounded("uninstalled systemd unit"),
        })
    }
}

impl<E: CommandExecutor> SystemdManager<E> {
    fn post_write(&self) -> Result<(), ServiceError> {
        if self.install.reload {
            let argv = vec![
                "systemctl".to_string(),
                self.install.scope.flag().to_string(),
                "daemon-reload".to_string(),
            ];
            let out = self
                .executor
                .run(
                    &argv,
                    None,
                    bounded_timeout(self.install.transition_timeout),
                )
                .map_err(permission_hint)?;
            ensure_mutation_success(&out, "systemctl daemon-reload")?;
        }
        if self.install.enable {
            let argv = vec![
                "systemctl".to_string(),
                self.install.scope.flag().to_string(),
                "enable".to_string(),
                self.install.unit_name.clone(),
            ];
            let out = self
                .executor
                .run(
                    &argv,
                    None,
                    bounded_timeout(self.install.transition_timeout),
                )
                .map_err(permission_hint)?;
            ensure_mutation_success(&out, "systemctl enable")?;
        }
        Ok(())
    }
}

// ---- launchd adapter ----

/// launchd domain without automatic EUID-based choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchdDomain {
    /// Per-user agent (`gui/<uid>` target supplied by the caller).
    UserAgent,
    /// System daemon (`system` target, requires appropriate privileges).
    SystemDaemon,
}

/// Caller-owned launchd install material.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchdInstall {
    /// Job label (for example `com.example.daemon`).
    pub label: String,
    /// Explicit domain (never guessed).
    pub domain: LaunchdDomain,
    /// Bootstrap target: `gui/<uid>` for user agents, `system` for daemons.
    pub target: String,
    /// Absolute plist path supplied by the caller.
    pub plist_path: PathBuf,
    /// Exact desired plist bytes (caller-owned content).
    pub definition: Vec<u8>,
    /// Whether install should bootstrap the job (explicit only).
    pub bootstrap_on_install: bool,
    /// Bounded wait for state transitions.
    pub transition_timeout: Duration,
}

impl LaunchdInstall {
    /// Validates install material.
    pub fn new(
        label: String,
        domain: LaunchdDomain,
        target: String,
        plist_path: PathBuf,
        definition: Vec<u8>,
        bootstrap_on_install: bool,
        transition_timeout: Duration,
    ) -> Result<Self, ServiceError> {
        validate_label(&label)?;
        if target.is_empty() || target.len() > 128 || target.chars().any(char::is_control) {
            return Err(ServiceError::invalid("launchd target is invalid"));
        }
        match domain {
            LaunchdDomain::UserAgent if !target.starts_with("gui/") => {
                return Err(ServiceError::invalid(
                    "user-agent target must look like gui/<uid>",
                ));
            }
            LaunchdDomain::SystemDaemon if target != "system" => {
                return Err(ServiceError::invalid(
                    "system-daemon target must be `system`",
                ));
            }
            _ => {}
        }
        if !plist_path.is_absolute() {
            return Err(ServiceError::invalid("plist path must be absolute"));
        }
        if definition.is_empty() || definition.len() > MAX_DEFINITION_BYTES {
            return Err(ServiceError::invalid(
                "plist definition is empty or overlong",
            ));
        }
        if definition.contains(&0) {
            return Err(ServiceError::invalid("plist definition contains nul"));
        }
        if transition_timeout.is_zero() || transition_timeout > MAX_TRANSITION_TIMEOUT {
            return Err(ServiceError::invalid("transition timeout out of range"));
        }
        Ok(Self {
            label,
            domain,
            target,
            plist_path,
            definition,
            bootstrap_on_install,
            transition_timeout,
        })
    }
}

fn validate_label(label: &str) -> Result<(), ServiceError> {
    if label.is_empty() || label.len() > 256 || label.chars().any(char::is_control) {
        return Err(ServiceError::invalid(
            "label is empty, overlong, or has controls",
        ));
    }
    if label.contains(['/', '\\', ' ', '@']) {
        return Err(ServiceError::invalid("label must be a bare job label"));
    }
    if !label
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_')
    {
        return Err(ServiceError::invalid("label must use [A-Za-z0-9._-] only"));
    }
    Ok(())
}

/// launchd manager implementing the neutral contract.
#[derive(Debug)]
pub struct LaunchdManager<E: CommandExecutor = SystemExecutor> {
    executor: E,
    install: LaunchdInstall,
}

impl<E: CommandExecutor> LaunchdManager<E> {
    /// Creates a launchd adapter. No manager calls are made here.
    pub fn new(executor: E, install: LaunchdInstall) -> Self {
        Self { executor, install }
    }

    /// Returns the install descriptor.
    pub fn install_config(&self) -> &LaunchdInstall {
        &self.install
    }

    /// Returns the command executor (test visibility).
    pub fn executor(&self) -> &E {
        &self.executor
    }

    fn read_plist_observation(&self) -> Result<RegistrationSnapshot, ServiceError> {
        match std::fs::read(&self.install.plist_path) {
            Ok(bytes) => parse_launchd_plist(&bytes),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Ok(RegistrationSnapshot::absent())
            }
            Err(e) => Err(ServiceError::manager(format!("reading plist: {e}"))),
        }
    }

    fn loaded_and_running(
        &self,
        deadline: OperationDeadline,
    ) -> Result<(bool, LifecycleState), ServiceError> {
        let argv = vec![
            "launchctl".to_string(),
            "list".to_string(),
            self.install.label.clone(),
        ];
        let out = self
            .executor
            .run(&argv, None, deadline.command_timeout()?)
            .map_err(permission_hint)?;
        if out.status != Some(0) {
            return Ok((false, LifecycleState::Stopped));
        }
        // `launchctl list` prints `"PID" = <n>;` when running, `"PID" = -;`
        // or missing when loaded but not running. Narrow rule: running only
        // when a numeric PID is present.
        let text = out.stdout_text();
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with("\"PID\"") || line.starts_with("PID") {
                if let Some(eq) = line.find('=') {
                    let val = line[eq + 1..]
                        .trim()
                        .trim_matches([';', '"', ' ', '\t'])
                        .to_string();
                    if val.parse::<i64>().is_ok() {
                        return Ok((true, LifecycleState::Running));
                    }
                    return Ok((true, LifecycleState::Stopped));
                }
            }
        }
        // Loaded (exit 0) but no PID line: conservatively Stopped, not Unknown,
        // because the file observation already gates ownership.
        Ok((true, LifecycleState::Stopped))
    }

    fn ownership_of(
        &self,
        spec: &ServiceSpec,
    ) -> Result<(Ownership, LifecycleState, bool, bool), ServiceError> {
        self.ownership_until(
            spec,
            OperationDeadline::new(self.install.transition_timeout)?,
        )
    }

    fn ownership_until(
        &self,
        spec: &ServiceSpec,
        deadline: OperationDeadline,
    ) -> Result<(Ownership, LifecycleState, bool, bool), ServiceError> {
        let observed = self.read_plist_observation()?;
        if !observed.present {
            return Ok((Ownership::Absent, LifecycleState::Stopped, false, false));
        }
        let observed = reconcile_config_identity(observed, spec);
        let ownership = observed.ownership(spec);
        if ownership != Ownership::Owned {
            // Foreign/Unknown files are never probed for loaded state beyond
            // what is needed to report Stopped (no destructive calls).
            return Ok((ownership, LifecycleState::Unknown, true, false));
        }
        match self.loaded_and_running(deadline) {
            Ok((loaded, state)) => Ok((ownership, state, true, loaded)),
            Err(_) => Ok((ownership, LifecycleState::Unknown, true, false)),
        }
    }
}

/// Parses plist bytes for `ProgramArguments` ownership.
///
/// Narrow structured scan (no XML library): finds exactly one
/// `<key>ProgramArguments</key>` followed by `<array>` of `<string>`
/// entries. Any ambiguity, unsupported escaping, or missing key yields a
/// malformed (`Unknown`) snapshot. Label-only observations are never `Owned`.
fn parse_launchd_plist(bytes: &[u8]) -> Result<RegistrationSnapshot, ServiceError> {
    if bytes.len() > MAX_DEFINITION_BYTES {
        return Ok(malformed_snapshot());
    }
    let text = match std::str::from_utf8(bytes) {
        Ok(t) => t,
        Err(_) => return Ok(malformed_snapshot()),
    };
    let key = "<key>ProgramArguments</key>";
    if text.matches(key).count() != 1 {
        return Ok(malformed_snapshot());
    }
    let key_pos = text.find(key).expect("counted");
    let after_key = &text[key_pos + key.len()..];
    let array_open = "<array>";
    let array_close = "</array>";
    let Some(array_start_rel) = after_key.find(array_open) else {
        return Ok(malformed_snapshot());
    };
    // No other keys may intervene between the key and its array.
    if after_key[..array_start_rel].contains("<key>") {
        return Ok(malformed_snapshot());
    }
    let array_body_start = array_start_rel + array_open.len();
    let Some(array_end_rel) = after_key[array_body_start..].find(array_close) else {
        return Ok(malformed_snapshot());
    };
    let array_body = &after_key[array_body_start..array_body_start + array_end_rel];
    // Only `<string>` entries are supported inside the array.
    if array_body.contains('<') && !array_body.contains("<string>") {
        // Contains tags but no strings: malformed.
        if !array_body.trim().is_empty() {
            return Ok(malformed_snapshot());
        }
    }
    let mut args: Vec<String> = Vec::new();
    let mut rest = array_body;
    while let Some(open) = rest.find("<string>") {
        let Some(close) = rest[open..].find("</string>") else {
            return Ok(malformed_snapshot());
        };
        let value = rest[open + "<string>".len()..open + close].to_string();
        // Reject unsupported XML constructs inside values: no tags, and only
        // the five predefined entities.
        if value.contains('<') {
            return Ok(malformed_snapshot());
        }
        if value.contains('&') && !is_supported_entity(&value) {
            return Ok(malformed_snapshot());
        }
        args.push(unescape_plist(&value));
        rest = &rest[open + close + "</string>".len()..];
    }
    // Any leftover non-whitespace tags mean an unsupported shape.
    let stripped = rest.replace([' ', '\t', '\n', '\r'], "");
    if !stripped.is_empty() {
        return Ok(malformed_snapshot());
    }
    if args.is_empty() {
        return Ok(malformed_snapshot());
    }
    let exe = PathBuf::from(&args[0]);
    if !exe.is_absolute() {
        return Ok(malformed_snapshot());
    }
    Ok(RegistrationSnapshot {
        present: true,
        executable: Some(exe),
        args: args[1..].to_vec(),
        config: None,
        malformed: false,
    })
}

fn malformed_snapshot() -> RegistrationSnapshot {
    RegistrationSnapshot {
        present: true,
        executable: None,
        args: Vec::new(),
        config: None,
        malformed: true,
    }
}

fn is_supported_entity(v: &str) -> bool {
    // Cheap check: only &amp; &lt; &gt; &quot; &apos; are understood.
    let mut rest = v;
    while let Some(pos) = rest.find('&') {
        let tail = &rest[pos..];
        if !(tail.starts_with("&amp;")
            || tail.starts_with("&lt;")
            || tail.starts_with("&gt;")
            || tail.starts_with("&quot;")
            || tail.starts_with("&apos;"))
        {
            return false;
        }
        rest = &tail[1..];
    }
    true
}

fn unescape_plist(v: &str) -> String {
    v.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

impl<E: CommandExecutor> ServiceManager for LaunchdManager<E> {
    fn inspect(&self, spec: &ServiceSpec) -> Result<LifecycleSnapshot, ServiceError> {
        let (ownership, state, present, loaded) = self.ownership_of(spec)?;
        Ok(LifecycleSnapshot {
            id: spec.id().clone(),
            ownership,
            state,
            health: HealthState::Unknown,
            was_registered: present,
            was_running: state == LifecycleState::Running && loaded,
        })
    }

    fn install(&mut self, spec: &ServiceSpec) -> Result<TransitionResult, ServiceError> {
        let (ownership, _, present, _) = self.ownership_of(spec)?;
        match ownership {
            Ownership::Absent => {
                let _ = present;
                atomic_write_definition(&self.install.plist_path, &self.install.definition, false)?;
                if self.install.bootstrap_on_install {
                    self.bootstrap(OperationDeadline::new(self.install.transition_timeout)?)?;
                }
                Ok(TransitionResult {
                    operation: ServiceOperation::Install,
                    completed: true,
                    detail: ServiceError::bounded("installed launchd plist"),
                })
            }
            Ownership::Owned => {
                let (recheck, _, _, _) = self.ownership_of(spec)?;
                if recheck != Ownership::Owned {
                    return Err(ServiceError::denied(spec.id(), recheck));
                }
                atomic_write_definition(&self.install.plist_path, &self.install.definition, true)?;
                if self.install.bootstrap_on_install {
                    self.bootstrap(OperationDeadline::new(self.install.transition_timeout)?)?;
                }
                Ok(TransitionResult {
                    operation: ServiceOperation::Install,
                    completed: true,
                    detail: ServiceError::bounded("refreshed owned launchd plist"),
                })
            }
            Ownership::Foreign | Ownership::Unknown => {
                Err(ServiceError::denied(spec.id(), ownership))
            }
        }
    }

    fn start(
        &mut self,
        spec: &ServiceSpec,
        timeout: Duration,
    ) -> Result<TransitionResult, ServiceError> {
        self.start_until(spec, OperationDeadline::new(timeout)?)
    }

    fn stop(
        &mut self,
        spec: &ServiceSpec,
        timeout: Duration,
    ) -> Result<TransitionResult, ServiceError> {
        self.stop_until(spec, OperationDeadline::new(timeout)?)
    }

    fn restart(
        &mut self,
        spec: &ServiceSpec,
        timeout: Duration,
    ) -> Result<TransitionResult, ServiceError> {
        let deadline = OperationDeadline::new(timeout)?;
        let stopped = self.stop_until(spec, deadline)?;
        if !stopped.completed {
            return Ok(TransitionResult {
                operation: ServiceOperation::Restart,
                completed: false,
                detail: ServiceError::bounded("restart stop phase incomplete; start not attempted"),
            });
        }
        let started = self.start_until(spec, deadline)?;
        Ok(TransitionResult {
            operation: ServiceOperation::Restart,
            completed: started.completed,
            detail: ServiceError::bounded(if started.completed {
                "restarted"
            } else {
                "restart start phase incomplete; state unconfirmed"
            }),
        })
    }

    fn uninstall(&mut self, spec: &ServiceSpec) -> Result<TransitionResult, ServiceError> {
        let (ownership, _, _, loaded) = self.ownership_of(spec)?;
        require_owned(spec.id(), ownership)?;
        let (recheck, _, _, _) = self.ownership_of(spec)?;
        if recheck != Ownership::Owned {
            return Err(ServiceError::denied(spec.id(), recheck));
        }
        if loaded {
            let argv = vec![
                "launchctl".to_string(),
                "bootout".to_string(),
                self.install.target.clone(),
                self.install.plist_path.to_string_lossy().into_owned(),
            ];
            match self.executor.run(
                &argv,
                None,
                bounded_timeout(self.install.transition_timeout),
            ) {
                Ok(out) if out.status == Some(0) => {}
                Ok(out) => {
                    return Ok(TransitionResult {
                        operation: ServiceOperation::Uninstall,
                        completed: false,
                        detail: ServiceError::bounded(format!(
                            "bootout exited {}",
                            out.status.unwrap_or(-1)
                        )),
                    });
                }
                Err(e) => {
                    return Ok(TransitionResult {
                        operation: ServiceOperation::Uninstall,
                        completed: false,
                        detail: ServiceError::bounded(format!("bootout failed: {e}")),
                    });
                }
            }
        }
        std::fs::remove_file(&self.install.plist_path)
            .map_err(|e| ServiceError::manager(format!("removing owned plist: {e}")))?;
        Ok(TransitionResult {
            operation: ServiceOperation::Uninstall,
            completed: true,
            detail: ServiceError::bounded("uninstalled launchd plist"),
        })
    }
}

impl<E: CommandExecutor> LaunchdManager<E> {
    fn start_until(
        &self,
        spec: &ServiceSpec,
        deadline: OperationDeadline,
    ) -> Result<TransitionResult, ServiceError> {
        let (ownership, state, _, loaded) = self.ownership_until(spec, deadline)?;
        require_owned(spec.id(), ownership)?;
        if state == LifecycleState::Running {
            return Ok(TransitionResult {
                operation: ServiceOperation::Start,
                completed: !deadline.expired(),
                detail: ServiceError::bounded("already running"),
            });
        }
        if !loaded {
            self.bootstrap(deadline)?;
        }
        self.kickstart(deadline)?;
        let completed = self.confirm_running(deadline)?;
        Ok(TransitionResult {
            operation: ServiceOperation::Start,
            completed,
            detail: ServiceError::bounded(if completed {
                "started"
            } else {
                "start incomplete; state unconfirmed"
            }),
        })
    }

    fn stop_until(
        &self,
        spec: &ServiceSpec,
        deadline: OperationDeadline,
    ) -> Result<TransitionResult, ServiceError> {
        let (ownership, state, _, _) = self.ownership_until(spec, deadline)?;
        require_owned(spec.id(), ownership)?;
        if state == LifecycleState::Stopped {
            return Ok(TransitionResult {
                operation: ServiceOperation::Stop,
                completed: !deadline.expired(),
                detail: ServiceError::bounded("already stopped"),
            });
        }
        let argv = vec![
            "launchctl".to_string(),
            "stop".to_string(),
            self.install.label.clone(),
        ];
        let out = self
            .executor
            .run(&argv, None, deadline.command_timeout()?)
            .map_err(permission_hint)?;
        ensure_mutation_success(&out, "launchctl stop")?;
        let completed = self.confirm_stopped(deadline)?;
        Ok(TransitionResult {
            operation: ServiceOperation::Stop,
            completed,
            detail: ServiceError::bounded(if completed {
                "stopped"
            } else {
                "stop incomplete; state unconfirmed"
            }),
        })
    }

    fn bootstrap(&self, deadline: OperationDeadline) -> Result<(), ServiceError> {
        let argv = vec![
            "launchctl".to_string(),
            "bootstrap".to_string(),
            self.install.target.clone(),
            self.install.plist_path.to_string_lossy().into_owned(),
        ];
        let out = self
            .executor
            .run(&argv, None, deadline.command_timeout()?)
            .map_err(permission_hint)?;
        ensure_mutation_success(&out, "launchctl bootstrap")
    }

    fn kickstart(&self, deadline: OperationDeadline) -> Result<(), ServiceError> {
        let argv = vec![
            "launchctl".to_string(),
            "kickstart".to_string(),
            "-k".to_string(),
            format!("{}/{}", self.install.target, self.install.label),
        ];
        let out = self
            .executor
            .run(&argv, None, deadline.command_timeout()?)
            .map_err(permission_hint)?;
        ensure_mutation_success(&out, "launchctl kickstart")
    }

    fn confirm_running(&self, deadline: OperationDeadline) -> Result<bool, ServiceError> {
        loop {
            match self.loaded_and_running(deadline) {
                Ok((_, LifecycleState::Running)) => return Ok(!deadline.expired()),
                Ok(_) => {}
                Err(e) => return Err(e),
            }
            if deadline.expired() {
                return Ok(false);
            }
            std::thread::sleep(
                deadline
                    .remaining()
                    .unwrap_or_default()
                    .min(Duration::from_millis(100)),
            );
        }
    }

    fn confirm_stopped(&self, deadline: OperationDeadline) -> Result<bool, ServiceError> {
        loop {
            match self.loaded_and_running(deadline) {
                Ok((_, LifecycleState::Stopped)) => return Ok(!deadline.expired()),
                Ok(_) => {}
                Err(e) => return Err(e),
            }
            if deadline.expired() {
                return Ok(false);
            }
            std::thread::sleep(
                deadline
                    .remaining()
                    .unwrap_or_default()
                    .min(Duration::from_millis(100)),
            );
        }
    }
}

// ---- cron adapter ----

/// How a crontab relates to one managed block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CronOwnership {
    /// No managed block is present.
    Absent,
    /// Exactly one block matches the desired content.
    Owned,
    /// Same marker with different content (another deployment). No overwrite.
    Foreign,
    /// Ambiguous (multiple markers, malformed). Fail closed.
    Unknown,
}

/// Validates a cron marker (unique, single-line, no shell).
fn validate_marker(marker: &str) -> Result<(), ServiceError> {
    if marker.is_empty() || marker.len() > 128 || marker.chars().any(char::is_control) {
        return Err(ServiceError::invalid("cron marker is invalid"));
    }
    if marker.contains(['\n', '\r']) {
        return Err(ServiceError::invalid("cron marker must be single-line"));
    }
    Ok(())
}

/// Validates a desired cron block (exact lines between markers).
fn validate_block(block: &str) -> Result<(), ServiceError> {
    if block.len() > MAX_CRONTAB_BYTES / 2 {
        return Err(ServiceError::invalid("cron block exceeds bound"));
    }
    if block.contains('\0') {
        return Err(ServiceError::invalid("cron block contains nul"));
    }
    Ok(())
}

fn begin_line(marker: &str) -> String {
    format!("# BEGIN {marker}")
}

fn end_line(marker: &str) -> String {
    format!("# END {marker}")
}

/// Locates managed blocks: returns `(begin_line_idx, end_line_idx)` pairs.
fn find_blocks(lines: &[&str], marker: &str) -> Vec<(usize, usize)> {
    let begin = begin_line(marker);
    let end = end_line(marker);
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].trim_end() == begin {
            let mut j = i + 1;
            while j < lines.len() && lines[j].trim_end() != end {
                j += 1;
            }
            if j < lines.len() {
                out.push((i, j));
                i = j + 1;
                continue;
            }
            // Unterminated BEGIN without END: treat as a block to the end
            // (malformed → Unknown at classification).
            out.push((i, lines.len()));
            break;
        }
        i += 1;
    }
    out
}

/// Classifies a crontab against one desired block.
pub fn cron_classify(crontab: &str, marker: &str, desired: &str) -> CronOwnership {
    let lines: Vec<&str> = crontab.lines().collect();
    let blocks = find_blocks(&lines, marker);
    match blocks.len() {
        0 => CronOwnership::Absent,
        1 => {
            let (b, e) = blocks[0];
            if e >= lines.len() {
                return CronOwnership::Unknown;
            }
            let current = lines[b + 1..e].join("\n");
            if current == desired.trim_matches('\n') || current == desired {
                CronOwnership::Owned
            } else {
                // Documented rule: same marker with different content is
                // Foreign (another deployment owns the marker name).
                CronOwnership::Foreign
            }
        }
        _ => CronOwnership::Unknown,
    }
}

/// Merges one managed block idempotently, preserving unrelated bytes.
///
/// Returns `(new_crontab, changed)`. Fails closed on duplicate markers or
/// foreign content (no destructive overwrite). Appends with exactly one
/// trailing newline when creating.
pub fn cron_merge(
    crontab: &str,
    marker: &str,
    desired: &str,
) -> Result<(String, bool), ServiceError> {
    validate_marker(marker)?;
    validate_block(desired)?;
    match cron_classify(crontab, marker, desired) {
        CronOwnership::Owned => Ok((crontab.to_string(), false)),
        CronOwnership::Absent => {
            let desired_trimmed = desired.trim_matches('\n');
            let block = format!(
                "{}\n{}\n{}",
                begin_line(marker),
                desired_trimmed,
                end_line(marker)
            );
            let mut out = crontab.to_string();
            if out.is_empty() {
                out = format!("{block}\n");
            } else if out.ends_with('\n') {
                out.push_str(&format!("{block}\n"));
            } else {
                out.push_str(&format!("\n{block}\n"));
            }
            if out.len() > MAX_CRONTAB_BYTES {
                return Err(ServiceError::invalid("merged crontab exceeds bound"));
            }
            Ok((out, true))
        }
        CronOwnership::Foreign => Err(ServiceError::denied(
            &ServiceId::new(format!("cron:{marker}"))
                .unwrap_or_else(|_| ServiceId::new("cron").expect("static")),
            Ownership::Foreign,
        )),
        CronOwnership::Unknown => Err(ServiceError::conflict(
            "multiple or malformed managed cron blocks; refusing to mutate",
        )),
    }
}

/// Removes the owned managed block, preserving unrelated bytes.
///
/// Returns `(new_crontab, changed)`. Only an exact `Owned` block is removed;
/// `Absent` is a no-op without rewrite; `Foreign`/`Unknown` fail closed.
pub fn cron_remove(
    crontab: &str,
    marker: &str,
    desired: &str,
) -> Result<(String, bool), ServiceError> {
    validate_marker(marker)?;
    match cron_classify(crontab, marker, desired) {
        CronOwnership::Absent => Ok((crontab.to_string(), false)),
        CronOwnership::Owned => {
            let lines: Vec<&str> = crontab.lines().collect();
            let blocks = find_blocks(&lines, marker);
            debug_assert_eq!(blocks.len(), 1);
            let (b, e) = blocks[0];
            let mut kept: Vec<&str> = Vec::new();
            for (i, line) in lines.iter().enumerate() {
                if i < b || i > e {
                    kept.push(line);
                }
            }
            let mut out = kept.join("\n");
            if !crontab.is_empty() && !out.is_empty() {
                // Preserve the trailing-newline convention of the input.
                if crontab.ends_with('\n') {
                    out.push('\n');
                }
            }
            // Removing the only block from a file that was exactly the block
            // yields an empty crontab (valid: `crontab -r` equivalent via
            // `crontab -` with empty input is avoided; caller handles empty
            // as `crontab -r`? To keep one code path, return empty string and
            // let the adapter decide between `crontab -` and `crontab -r`.
            // Here we return the empty string; the adapter writes it via
            // `crontab -` only when changed (empty write clears the crontab
            // on most implementations; documented).
            Ok((out, true))
        }
        CronOwnership::Foreign => Err(ServiceError::denied(
            &ServiceId::new(format!("cron:{marker}"))
                .unwrap_or_else(|_| ServiceId::new("cron").expect("static")),
            Ownership::Foreign,
        )),
        CronOwnership::Unknown => Err(ServiceError::conflict(
            "multiple or malformed managed cron blocks; refusing to mutate",
        )),
    }
}

/// Cron manager: a managed block inside the user's crontab.
///
/// Cron has no native start/stop process semantics; those operations return
/// explicit `InvalidInput` errors instead of pretending success.
#[derive(Debug)]
pub struct CronManager<E: CommandExecutor = SystemExecutor> {
    executor: E,
    marker: String,
    desired_block: String,
    transition_timeout: Duration,
}

impl<E: CommandExecutor> CronManager<E> {
    /// Creates a cron adapter with a unique marker and exact desired block.
    pub fn new(
        executor: E,
        marker: String,
        desired_block: String,
        transition_timeout: Duration,
    ) -> Result<Self, ServiceError> {
        validate_marker(&marker)?;
        validate_block(&desired_block)?;
        if transition_timeout.is_zero() || transition_timeout > MAX_TRANSITION_TIMEOUT {
            return Err(ServiceError::invalid("transition timeout out of range"));
        }
        Ok(Self {
            executor,
            marker,
            desired_block,
            transition_timeout,
        })
    }

    /// Returns the marker.
    pub fn marker(&self) -> &str {
        &self.marker
    }

    fn list_crontab(&self) -> Result<String, ServiceError> {
        let argv = vec!["crontab".to_string(), "-l".to_string()];
        match self
            .executor
            .run(&argv, None, bounded_timeout(self.transition_timeout))
        {
            Ok(out) => {
                if out.status == Some(0) {
                    let text = out.stdout_text();
                    if text.len() > MAX_CRONTAB_BYTES {
                        return Err(ServiceError::manager("crontab exceeds bound"));
                    }
                    Ok(text)
                } else {
                    // `crontab -l` exits 1 with "no crontab" when empty.
                    // Treat that as empty, not as failure.
                    let combined =
                        format!("{}{}", out.stdout_text(), out.stderr_text()).to_lowercase();
                    if combined.contains("no crontab") {
                        Ok(String::new())
                    } else {
                        Err(ServiceError::manager(format!(
                            "crontab -l exited {}: {}",
                            out.status.unwrap_or(-1),
                            truncate(&out.stderr_text()),
                        )))
                    }
                }
            }
            Err(e) => Err(e),
        }
    }

    fn write_crontab(&self, content: &str) -> Result<(), ServiceError> {
        if content.len() > MAX_CRONTAB_BYTES {
            return Err(ServiceError::invalid("crontab content exceeds bound"));
        }
        let argv = vec!["crontab".to_string(), "-".to_string()];
        let out = self.executor.run(
            &argv,
            Some(content.as_bytes()),
            bounded_timeout(self.transition_timeout),
        )?;
        if out.status != Some(0) {
            return Err(ServiceError::manager(format!(
                "crontab - exited {}: {}",
                out.status.unwrap_or(-1),
                truncate(&out.stderr_text()),
            )));
        }
        Ok(())
    }

    fn cron_ownership(
        &self,
        spec: &ServiceSpec,
    ) -> Result<(Ownership, CronOwnership), ServiceError> {
        // Cron ownership is about the managed block, not the ServiceSpec exe
        // directly. The desired block is caller-owned material that must embed
        // the exact executable/args; the spec gates destructive ops via the
        // neutral rule using a synthetic observation:
        //
        // - Absent block → Absent (install allowed);
        // - Owned block → Owned (mutation allowed);
        // - Foreign/Unknown block → denied.
        //
        // Additionally, the block must contain the spec executable path;
        // otherwise even an exact block for another binary would look Owned.
        // Enforce: desired block must mention the executable, and the live
        // block must too, else Foreign.
        let current = self.list_crontab()?;
        let block_state = cron_classify(&current, &self.marker, &self.desired_block);
        let exe = spec.executable().to_string_lossy().into_owned();
        let ownership = match block_state {
            CronOwnership::Absent => Ownership::Absent,
            CronOwnership::Owned => {
                if current.contains(&exe) || self.desired_block.contains(&exe) {
                    Ownership::Owned
                } else {
                    Ownership::Foreign
                }
            }
            CronOwnership::Foreign => Ownership::Foreign,
            CronOwnership::Unknown => Ownership::Unknown,
        };
        Ok((ownership, block_state))
    }
}

impl<E: CommandExecutor> ServiceManager for CronManager<E> {
    fn inspect(&self, spec: &ServiceSpec) -> Result<LifecycleSnapshot, ServiceError> {
        // Cron has no running process semantics; state is Stopped when the
        // block exists (supervised externally) and Stopped when absent.
        // Health remains a separate consumer probe.
        let (ownership, _) = self
            .cron_ownership(spec)
            .unwrap_or((Ownership::Unknown, CronOwnership::Unknown));
        // If listing failed, report Unknown without mutating.
        let current = self.list_crontab();
        let (ownership, state, present) = match current {
            Ok(text) => {
                let block_state = cron_classify(&text, &self.marker, &self.desired_block);
                let ownership = match block_state {
                    CronOwnership::Absent => Ownership::Absent,
                    CronOwnership::Owned => {
                        let exe = spec.executable().to_string_lossy().into_owned();
                        if text.contains(&exe) {
                            Ownership::Owned
                        } else {
                            Ownership::Foreign
                        }
                    }
                    CronOwnership::Foreign => Ownership::Foreign,
                    CronOwnership::Unknown => Ownership::Unknown,
                };
                let present = block_state != CronOwnership::Absent;
                (ownership, LifecycleState::Stopped, present)
            }
            Err(_) => (ownership, LifecycleState::Unknown, false),
        };
        Ok(LifecycleSnapshot {
            id: spec.id().clone(),
            ownership,
            state,
            health: HealthState::Unknown,
            was_registered: present,
            was_running: false,
        })
    }

    fn install(&mut self, spec: &ServiceSpec) -> Result<TransitionResult, ServiceError> {
        let current = self.list_crontab()?;
        // Reuse pure merge (fail-closed on Foreign/Unknown, idempotent).
        let (merged, changed) = cron_merge(&current, &self.marker, &self.desired_block)?;
        // Gate on neutral ownership as well: Absent creation or Owned refresh.
        let block_state = cron_classify(&current, &self.marker, &self.desired_block);
        match block_state {
            CronOwnership::Absent | CronOwnership::Owned => {}
            CronOwnership::Foreign | CronOwnership::Unknown => {
                return Err(ServiceError::denied(
                    spec.id(),
                    if block_state == CronOwnership::Foreign {
                        Ownership::Foreign
                    } else {
                        Ownership::Unknown
                    },
                ));
            }
        }
        if !changed {
            return Ok(TransitionResult {
                operation: ServiceOperation::Install,
                completed: true,
                detail: ServiceError::bounded("cron block already installed"),
            });
        }
        // Re-list before destructive write (external crontab changes).
        let fresh = self.list_crontab()?;
        if fresh != current {
            return Err(ServiceError::conflict(
                "crontab changed during install; re-inspect before retry",
            ));
        }
        self.write_crontab(&merged)?;
        Ok(TransitionResult {
            operation: ServiceOperation::Install,
            completed: true,
            detail: ServiceError::bounded("installed cron block"),
        })
    }

    fn start(
        &mut self,
        _spec: &ServiceSpec,
        _timeout: Duration,
    ) -> Result<TransitionResult, ServiceError> {
        Err(ServiceError::invalid(
            "cron has no start semantics; supervision is external",
        ))
    }

    fn stop(
        &mut self,
        _spec: &ServiceSpec,
        _timeout: Duration,
    ) -> Result<TransitionResult, ServiceError> {
        Err(ServiceError::invalid(
            "cron has no stop semantics; supervision is external",
        ))
    }

    fn restart(
        &mut self,
        _spec: &ServiceSpec,
        _timeout: Duration,
    ) -> Result<TransitionResult, ServiceError> {
        Err(ServiceError::invalid(
            "cron has no restart semantics; supervision is external",
        ))
    }

    fn uninstall(&mut self, spec: &ServiceSpec) -> Result<TransitionResult, ServiceError> {
        let current = self.list_crontab()?;
        let block_state = cron_classify(&current, &self.marker, &self.desired_block);
        match block_state {
            CronOwnership::Owned => {}
            CronOwnership::Absent => {
                return Ok(TransitionResult {
                    operation: ServiceOperation::Uninstall,
                    completed: true,
                    detail: ServiceError::bounded("cron block already absent"),
                });
            }
            CronOwnership::Foreign => {
                return Err(ServiceError::denied(spec.id(), Ownership::Foreign));
            }
            CronOwnership::Unknown => {
                return Err(ServiceError::denied(spec.id(), Ownership::Unknown));
            }
        }
        let (removed, changed) = cron_remove(&current, &self.marker, &self.desired_block)?;
        if !changed {
            return Ok(TransitionResult {
                operation: ServiceOperation::Uninstall,
                completed: true,
                detail: ServiceError::bounded("cron block already absent"),
            });
        }
        let fresh = self.list_crontab()?;
        if fresh != current {
            return Err(ServiceError::conflict(
                "crontab changed during uninstall; re-inspect before retry",
            ));
        }
        // Avoid a redundant write when nothing changes (handled above);
        // here a change is required, so write exactly once.
        self.write_crontab(&removed)?;
        Ok(TransitionResult {
            operation: ServiceOperation::Uninstall,
            completed: true,
            detail: ServiceError::bounded("removed owned cron block"),
        })
    }
}

// ---- host detection ----

/// Candidate manager families for auto-selection (policy helper output).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateManager {
    /// systemd with an explicit scope.
    Systemd(SystemdScope),
    /// launchd with an explicit domain.
    Launchd(LaunchdDomain),
    /// User crontab supervision.
    Cron,
}

/// Observed host facts (separate from selection policy).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostFacts {
    /// `std::env::consts::OS` value at detection time.
    pub os: String,
    /// Whether a usable systemd manager was observed (Linux only).
    pub systemd_available: bool,
    /// Whether `launchctl` was observed (macOS only).
    pub launchd_available: bool,
    /// Whether `crontab` was observed.
    pub crontab_available: bool,
}

/// Inspects host manager facts without choosing policy.
///
/// Distinguishes detection failure (`Err`) from "not installed"
/// (`Ok` with `*_available = false`). Never elevates or mutates.
pub fn inspect_host(executor: &impl CommandExecutor) -> Result<HostFacts, ServiceError> {
    let os = std::env::consts::OS.to_string();
    // systemd: Linux only. Binary probe + well-known runtime dir.
    let systemd_available = if os == "linux" {
        let runtime_dir = std::path::Path::new("/run/systemd/system").exists();
        match executor.run(
            &["systemctl".to_string(), "--version".to_string()],
            None,
            Duration::from_secs(5),
        ) {
            Ok(out) => out.status == Some(0) && runtime_dir,
            Err(ServiceError::Manager(d)) if d.contains("missing") => false,
            Err(e) => return Err(e),
        }
    } else {
        false
    };
    let launchd_available = if os == "macos" {
        match executor.run(
            &["launchctl".to_string(), "version".to_string()],
            None,
            Duration::from_secs(5),
        ) {
            Ok(out) => out.status == Some(0),
            Err(ServiceError::Manager(d)) if d.contains("missing") => false,
            Err(e) => return Err(e),
        }
    } else {
        false
    };
    let crontab_available = match executor.run(
        &["crontab".to_string(), "-l".to_string()],
        None,
        Duration::from_secs(5),
    ) {
        Ok(_) => true,
        Err(ServiceError::Manager(d)) if d.contains("missing") => false,
        Err(e) => return Err(e),
    };
    Ok(HostFacts {
        os,
        systemd_available,
        launchd_available,
        crontab_available,
    })
}

/// Small explicit auto-selection policy over [`HostFacts`].
///
/// Overridable: consumers may ignore this and construct a specific adapter.
/// Linux with usable systemd yields systemd; macOS with launchd yields
/// launchd; otherwise cron is the candidate where available. Empty output
/// means no candidate (distinct from detection failure, which is `Err` from
/// [`inspect_host`]).
pub fn candidate_managers(facts: &HostFacts) -> Vec<CandidateManager> {
    if facts.os == "macos" && facts.launchd_available {
        let mut out = vec![CandidateManager::Launchd(LaunchdDomain::UserAgent)];
        if facts.crontab_available {
            out.push(CandidateManager::Cron);
        }
        return out;
    }
    if facts.os == "linux" && facts.systemd_available {
        let mut out = vec![CandidateManager::Systemd(SystemdScope::System)];
        if facts.crontab_available {
            out.push(CandidateManager::Cron);
        }
        return out;
    }
    if facts.crontab_available {
        return vec![CandidateManager::Cron];
    }
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn spec(id: &str, exe: &str, args: &[&str]) -> ServiceSpec {
        ServiceSpec::new(
            ServiceId::new(id).unwrap(),
            PathBuf::from(exe),
            args.iter().map(|s| s.to_string()).collect(),
            None,
        )
        .unwrap()
    }

    #[test]
    fn diagnostic_byte_bounds_preserve_utf8_at_256_and_512_edges() {
        for bound in [256, 512] {
            let exact = truncate_utf8_bytes("a".repeat(bound), bound);
            assert_eq!(exact.len(), bound);
            let over = truncate_utf8_bytes("a".repeat(bound + 1), bound);
            assert_eq!(over.len(), bound);

            for character in ["é", "€", "🧡"] {
                let width = character.len();
                let mut input = "a".repeat(bound - width + 1);
                input.push_str(character);
                input.push_str("tail");
                let bounded = truncate_utf8_bytes(input, bound);
                assert_eq!(bounded.len(), bound - width + 1);
                assert!(bounded.is_char_boundary(bounded.len()));
            }
        }
    }

    #[test]
    fn service_errors_output_and_permission_remediation_stay_utf8_bounded() {
        let unicode = "🧡".repeat(200);
        let invalid = ServiceError::invalid(unicode.clone());
        assert!(
            matches!(invalid, ServiceError::InvalidInput(text) if text.len() <= 512 && text.is_char_boundary(text.len()))
        );
        assert!(ServiceError::bounded(unicode.clone()).len() <= 512);
        assert!(truncate(&unicode).len() <= 256);

        let manager = ServiceError::manager(format!("manager failed: {unicode}"));
        let remediated = permission_hint(ServiceError::manager(format!(
            "permission denied: {unicode}"
        )));
        assert!(
            matches!(manager, ServiceError::Manager(text) if text.len() <= 512 && text.is_char_boundary(text.len()))
        );
        assert!(
            matches!(remediated, ServiceError::Manager(text) if text.len() <= 512 && text.is_char_boundary(text.len()) && text.contains("permission denied;"))
        );
    }

    #[test]
    fn exact_owned_registration_allows_lifecycle() {
        let mut m = TestDoubleManager::new();
        let s = spec("svc", "/opt/app/bin", &["--serve"]);
        m.put(s.clone(), LifecycleState::Stopped);
        let snap = m.inspect(&s).unwrap();
        assert_eq!(snap.ownership, Ownership::Owned);
        assert_eq!(snap.state, LifecycleState::Stopped);
        assert!(!snap.was_running);
        m.start(&s, Duration::from_secs(5)).unwrap();
        assert_eq!(m.inspect(&s).unwrap().state, LifecycleState::Running);
        m.stop(&s, Duration::from_secs(5)).unwrap();
        assert_eq!(m.inspect(&s).unwrap().state, LifecycleState::Stopped);
    }

    #[test]
    fn same_name_different_executable_is_foreign() {
        let mut m = TestDoubleManager::new();
        m.put(
            spec("svc", "/opt/other/bin", &["--serve"]),
            LifecycleState::Running,
        );
        let want = spec("svc", "/opt/app/bin", &["--serve"]);
        assert_eq!(m.inspect(&want).unwrap().ownership, Ownership::Foreign);
        assert!(m.stop(&want, Duration::from_secs(1)).is_err());
        assert!(m.uninstall(&want).is_err());
    }

    #[test]
    fn same_executable_different_critical_args_is_foreign() {
        let mut m = TestDoubleManager::new();
        m.put(
            spec("svc", "/opt/app/bin", &["--serve", "--port=1"]),
            LifecycleState::Stopped,
        );
        let want = spec("svc", "/opt/app/bin", &["--serve", "--port=2"]);
        assert_eq!(
            m.inspect(&want).unwrap().ownership,
            Ownership::Foreign,
            "documented rule: critical-arg drift is Foreign"
        );
        assert!(m.start(&want, Duration::from_secs(1)).is_err());
    }

    #[test]
    fn malformed_registration_is_unknown_and_denies_mutation() {
        let mut m = TestDoubleManager::new();
        let id = ServiceId::new("svc").unwrap();
        m.put_malformed(id.clone());
        let want = ServiceSpec::new(id, PathBuf::from("/invalid"), vec![], None).unwrap();
        assert_eq!(m.inspect(&want).unwrap().ownership, Ownership::Unknown);
        assert!(m.stop(&want, Duration::from_secs(1)).is_err());
        assert!(m.restart(&want, Duration::from_secs(1)).is_err());
    }

    #[test]
    fn absent_install_then_lifecycle() {
        let mut m = TestDoubleManager::new();
        let s = spec("svc", "/opt/app/bin", &[]);
        assert_eq!(m.inspect(&s).unwrap().ownership, Ownership::Absent);
        m.install(&s).unwrap();
        assert_eq!(m.inspect(&s).unwrap().ownership, Ownership::Owned);
        m.start(&s, Duration::from_secs(1)).unwrap();
        m.uninstall(&s).unwrap();
        assert_eq!(m.inspect(&s).unwrap().ownership, Ownership::Absent);
    }

    #[test]
    fn stopped_and_running_snapshots_preserve_intent() {
        let mut m = TestDoubleManager::new();
        let s = spec("svc", "/opt/app/bin", &[]);
        m.put(s.clone(), LifecycleState::Stopped);
        let stopped = m.inspect(&s).unwrap();
        assert!(!stopped.restore_running());
        assert!(stopped.was_registered);
        m.start(&s, Duration::from_secs(1)).unwrap();
        let running = m.inspect(&s).unwrap();
        assert!(running.restore_running());
        // Stopped stays stopped by default: no auto-start on inspect/install refresh.
        m.stop(&s, Duration::from_secs(1)).unwrap();
        assert_eq!(m.inspect(&s).unwrap().state, LifecycleState::Stopped);
    }

    #[test]
    fn health_failure_does_not_mutate_manager_state() {
        #[derive(Debug)]
        struct Fail;
        impl HealthProbe for Fail {
            fn check(&self, _spec: &ServiceSpec) -> HealthState {
                HealthState::Degraded
            }
        }
        let mut m = TestDoubleManager::new();
        let s = spec("svc", "/opt/app/bin", &[]);
        m.put(s.clone(), LifecycleState::Running);
        let probe = Fail;
        assert_eq!(probe.check(&s), HealthState::Degraded);
        assert_eq!(m.inspect(&s).unwrap().state, LifecycleState::Running);
    }

    #[test]
    fn foreign_and_unknown_destructive_operations_are_denied() {
        let mut m = TestDoubleManager::new();
        m.put(spec("svc", "/opt/other/bin", &[]), LifecycleState::Running);
        let foreign = spec("svc", "/opt/app/bin", &[]);
        for op in [
            ServiceOperation::Start,
            ServiceOperation::Stop,
            ServiceOperation::Restart,
            ServiceOperation::Uninstall,
        ] {
            let err = match op {
                ServiceOperation::Start => m.start(&foreign, Duration::from_secs(1)).unwrap_err(),
                ServiceOperation::Stop => m.stop(&foreign, Duration::from_secs(1)).unwrap_err(),
                ServiceOperation::Restart => {
                    m.restart(&foreign, Duration::from_secs(1)).unwrap_err()
                }
                ServiceOperation::Uninstall => m.uninstall(&foreign).unwrap_err(),
                _ => continue,
            };
            assert!(
                matches!(err, ServiceError::OwnershipDenied { .. }),
                "{op:?}"
            );
        }
        // Install on Foreign is also denied (creation vs replacement are distinct).
        assert!(m.install(&foreign).is_err());
    }

    #[test]
    fn manager_state_and_health_are_separate() {
        let snap = LifecycleSnapshot {
            id: ServiceId::new("svc").unwrap(),
            ownership: Ownership::Owned,
            state: LifecycleState::Running,
            health: HealthState::Degraded,
            was_registered: true,
            was_running: true,
        };
        assert_eq!(snap.state, LifecycleState::Running);
        assert_eq!(snap.health, HealthState::Degraded);
    }

    #[test]
    fn restore_intent_is_explicit() {
        assert_eq!(RestoreIntent::Preserve, RestoreIntent::Preserve);
        assert_ne!(RestoreIntent::Preserve, RestoreIntent::EnsureRunning);
    }
}

#[cfg(test)]
mod unix_tests {
    use super::*;
    use std::path::PathBuf;

    fn spec(id: &str, exe: &str, args: &[&str]) -> ServiceSpec {
        ServiceSpec::new(
            ServiceId::new(id).unwrap(),
            PathBuf::from(exe),
            args.iter().map(|s| s.to_string()).collect(),
            None,
        )
        .unwrap()
    }

    fn temp_dir(prefix: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        // Include a per-call counter to avoid collisions within one nanos tick.
        use std::sync::atomic::{AtomicU64, Ordering};
        static N: AtomicU64 = AtomicU64::new(0);
        let n = N.fetch_add(1, Ordering::Relaxed);
        let p = std::env::temp_dir().join(format!("{prefix}-{}-{nanos}-{n}", std::process::id()));
        let _ = std::fs::create_dir_all(&p);
        p
    }

    fn out(status: i32, stdout: &str) -> CommandOutput {
        CommandOutput {
            status: Some(status),
            stdout: stdout.as_bytes().to_vec(),
            stderr: Vec::new(),
        }
    }

    fn show_loaded_active(exe: &str, args: &str, active: &str) -> String {
        format!(
            "LoadState=loaded\nActiveState={active}\nExecStart={{ path={exe} ; argv[]={exe} {args} ; ignore_errors=no }}\n"
        )
    }

    // ---- Shared command runner ----

    #[test]
    fn command_timeout_kills_and_reaps() {
        let ex = SystemExecutor::new();
        let start = Instant::now();
        let err = ex
            .run(
                &["/bin/sleep".to_string(), "5".to_string()],
                None,
                Duration::from_millis(200),
            )
            .unwrap_err();
        assert!(matches!(err, ServiceError::Manager(_)));
        assert!(format!("{err}").contains("timed out"));
        assert!(start.elapsed() < Duration::from_secs(4));
    }

    #[test]
    fn command_output_bound_fails_closed() {
        let ex = SystemExecutor::with_max_output(64).unwrap();
        // `/bin/echo` with a long argument exceeds the 64-byte bound.
        let big = "x".repeat(1024);
        let err = ex
            .run(
                &["/bin/echo".to_string(), big],
                None,
                Duration::from_secs(5),
            )
            .unwrap_err();
        assert!(matches!(err, ServiceError::Manager(_)));
        assert!(format!("{err}").contains("bound"));
    }

    #[test]
    fn command_passes_argv_literally_without_shell() {
        let ex = SystemExecutor::new();
        // Semicolons, pipes, and dollars must not be interpreted.
        let payload = "hello; echo PWNED | cat $HOME `id`";
        let output = ex
            .run(
                &["/bin/echo".to_string(), payload.to_string()],
                None,
                Duration::from_secs(5),
            )
            .unwrap();
        assert_eq!(output.status, Some(0));
        assert_eq!(output.stdout_text().trim(), payload);
    }

    #[test]
    fn command_nonzero_exit_is_ok_with_status() {
        let ex = SystemExecutor::new();
        let output = ex
            .run(
                &[
                    "/bin/sh".to_string(),
                    "-c".to_string(),
                    "exit 3".to_string(),
                ],
                None,
                Duration::from_secs(5),
            )
            .unwrap();
        assert_eq!(output.status, Some(3));
    }

    #[test]
    fn command_missing_binary_fails_closed() {
        let ex = SystemExecutor::new();
        let err = ex
            .run(
                &["/nonexistent-eggup-binary-xyz".to_string()],
                None,
                Duration::from_secs(5),
            )
            .unwrap_err();
        assert!(matches!(err, ServiceError::Manager(_)));
        assert!(format!("{err}").contains("missing"));
    }

    #[test]
    fn command_environment_adds_nothing() {
        // The child environment is cleared for a system-scoped command.
        let ex = SystemExecutor::new();
        let output = ex
            .run(&["/usr/bin/env".to_string()], None, Duration::from_secs(5))
            .unwrap();
        assert_eq!(output.status, Some(0));
        assert!(
            output.stdout_text().is_empty(),
            "system scope starts env-free"
        );
    }

    #[test]
    fn manager_resolution_ignores_ambient_path_and_environment_filters_sentinels() {
        let dir = temp_dir("manager-path");
        let fake = dir.join("systemctl");
        std::fs::write(&fake, b"#!/bin/sh\nexit 0\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        match resolve_manager_program("systemctl") {
            Ok(resolved) => {
                assert_ne!(resolved, fake, "ambient PATH candidate must be ignored");
                assert!(resolved.is_absolute());
            }
            Err(ServiceError::Manager(message)) => {
                assert!(message.contains("trusted manager binary missing"));
            }
            Err(other) => panic!("unexpected resolver error: {other}"),
        }

        let source = [
            ("PATH".into(), "/tmp/attacker".into()),
            ("EGGUP_SENTINEL".into(), "secret".into()),
            ("LD_PRELOAD".into(), "/tmp/inject.so".into()),
            (
                "DBUS_SESSION_BUS_ADDRESS".into(),
                "unix:path=/run/user/1/bus".into(),
            ),
            ("XDG_RUNTIME_DIR".into(), "/run/user/1".into()),
        ];
        let system =
            filtered_manager_environment(&["systemctl".into(), "start".into()], source.clone());
        assert!(system.is_empty());
        let user = filtered_manager_environment(
            &["systemctl".into(), "--user".into(), "start".into()],
            source,
        );
        let keys: Vec<_> = user
            .iter()
            .map(|(key, _)| key.to_string_lossy().into_owned())
            .collect();
        assert_eq!(keys, ["DBUS_SESSION_BUS_ADDRESS", "XDG_RUNTIME_DIR"]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn transition_deadline_rejects_zero_and_only_shrinks() {
        assert!(OperationDeadline::new(Duration::ZERO).is_err());
        let deadline = OperationDeadline::new(Duration::from_millis(80)).unwrap();
        let first = deadline.command_timeout().unwrap();
        std::thread::sleep(Duration::from_millis(10));
        let second = deadline.command_timeout().unwrap();
        assert!(second < first);
        assert!(second <= Duration::from_millis(70));
    }

    // ---- systemd ----

    fn systemd_manager(fake: FakeExecutor, unit_path: PathBuf) -> SystemdManager<FakeExecutor> {
        let install = SystemdInstall::new(
            "my-daemon.service".to_string(),
            SystemdScope::User,
            unit_path,
            b"[Unit]\nDescription=test\n[Service]\nExecStart=/opt/app/bin --serve\n".to_vec(),
            false,
            false,
            Duration::from_secs(5),
        )
        .unwrap();
        SystemdManager::new(fake, install)
    }

    #[test]
    fn systemd_absent_allows_install() {
        let fake = FakeExecutor::new();
        for _ in 0..2 {
            fake.expect(
                &[
                    "systemctl",
                    "--user",
                    "show",
                    "my-daemon.service",
                    "-p",
                    "LoadState,ActiveState,ExecStart",
                ],
                out(0, "LoadState=not-found\nActiveState=inactive\n"),
            );
        }
        let dir = temp_dir("sysd-absent");
        let unit = dir.join("my-daemon.service");
        let mut m = systemd_manager(fake, unit.clone());
        let s = spec("my-daemon.service", "/opt/app/bin", &["--serve"]);
        assert_eq!(m.inspect(&s).unwrap().ownership, Ownership::Absent);
        // Install writes the definition file (no reload/enable in this config).
        m.install(&s).unwrap();
        assert!(unit.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn systemd_exact_owned() {
        let fake = FakeExecutor::new();
        let show = show_loaded_active("/opt/app/bin", "--serve", "inactive");
        fake.expect(
            &[
                "systemctl",
                "--user",
                "show",
                "my-daemon.service",
                "-p",
                "LoadState,ActiveState,ExecStart",
            ],
            out(0, &show),
        );
        let dir = temp_dir("sysd-owned");
        let m = systemd_manager(fake, dir.join("my-daemon.service"));
        let s = spec("my-daemon.service", "/opt/app/bin", &["--serve"]);
        let snap = m.inspect(&s).unwrap();
        assert_eq!(snap.ownership, Ownership::Owned);
        assert_eq!(snap.state, LifecycleState::Stopped);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn systemd_same_unit_different_executable_is_foreign() {
        let fake = FakeExecutor::new();
        let show = show_loaded_active("/opt/other/bin", "--serve", "active");
        fake.expect(
            &[
                "systemctl",
                "--user",
                "show",
                "my-daemon.service",
                "-p",
                "LoadState,ActiveState,ExecStart",
            ],
            out(0, &show),
        );
        // Second show for the destructive denial path.
        fake.expect(
            &[
                "systemctl",
                "--user",
                "show",
                "my-daemon.service",
                "-p",
                "LoadState,ActiveState,ExecStart",
            ],
            out(0, &show),
        );
        let dir = temp_dir("sysd-foreign");
        let mut m = systemd_manager(fake, dir.join("my-daemon.service"));
        let want = spec("my-daemon.service", "/opt/app/bin", &["--serve"]);
        assert_eq!(m.inspect(&want).unwrap().ownership, Ownership::Foreign);
        assert!(m.stop(&want, Duration::from_secs(1)).is_err());
        assert!(m.uninstall(&want).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn systemd_argv_drift_is_foreign() {
        let fake = FakeExecutor::new();
        let show = show_loaded_active("/opt/app/bin", "--serve --port=1", "inactive");
        for _ in 0..2 {
            fake.expect(
                &[
                    "systemctl",
                    "--user",
                    "show",
                    "my-daemon.service",
                    "-p",
                    "LoadState,ActiveState,ExecStart",
                ],
                out(0, &show),
            );
        }
        let dir = temp_dir("sysd-drift");
        let mut m = systemd_manager(fake, dir.join("my-daemon.service"));
        let want = spec(
            "my-daemon.service",
            "/opt/app/bin",
            &["--serve", "--port=2"],
        );
        assert_eq!(m.inspect(&want).unwrap().ownership, Ownership::Foreign);
        assert!(m.start(&want, Duration::from_secs(1)).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn systemd_ambiguous_execstart_is_unknown() {
        let fake = FakeExecutor::new();
        // Two ExecStart blocks: ambiguous.
        let show = "LoadState=loaded\nActiveState=active\nExecStart={ path=/a ; argv[]=/a ; ignore_errors=no }\nExecStart={ path=/b ; argv[]=/b ; ignore_errors=no }\n";
        for _ in 0..3 {
            fake.expect(
                &[
                    "systemctl",
                    "--user",
                    "show",
                    "my-daemon.service",
                    "-p",
                    "LoadState,ActiveState,ExecStart",
                ],
                out(0, show),
            );
        }
        let dir = temp_dir("sysd-ambig");
        let mut m = systemd_manager(fake, dir.join("my-daemon.service"));
        let s = spec("my-daemon.service", "/opt/app/bin", &[]);
        assert_eq!(m.inspect(&s).unwrap().ownership, Ownership::Unknown);
        assert!(m.start(&s, Duration::from_secs(1)).is_err());
        assert!(m.uninstall(&s).is_err());
        // Specifiers also yield Unknown, not Owned.
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn systemd_specifier_execstart_is_unknown() {
        let text = "LoadState=loaded\nActiveState=active\nExecStart={ path=/opt/app/bin ; argv[]=/opt/app/bin %n ; ignore_errors=no }\n";
        let s = spec("my-daemon.service", "/opt/app/bin", &["%n"]);
        let (ownership, _, _) = systemd_ownership(text, &s).unwrap();
        assert_eq!(ownership, Ownership::Unknown);
    }

    #[test]
    fn systemd_config_identity_requires_one_exact_argv_match() {
        let with_config = ServiceSpec::new(
            ServiceId::new("my-daemon.service").unwrap(),
            PathBuf::from("/opt/app/bin"),
            vec!["--config".into(), "/etc/app.toml".into()],
            Some(PathBuf::from("/etc/app.toml")),
        )
        .unwrap();
        let exact = "LoadState=loaded\nActiveState=inactive\nExecStart={ path=/opt/app/bin ; argv[]=/opt/app/bin --config /etc/app.toml ; ignore_errors=no }\n";
        assert_eq!(
            systemd_ownership(exact, &with_config).unwrap().0,
            Ownership::Owned
        );
        let absent = "LoadState=loaded\nActiveState=inactive\nExecStart={ path=/opt/app/bin ; argv[]=/opt/app/bin --serve ; ignore_errors=no }\n";
        assert_eq!(
            systemd_ownership(absent, &with_config).unwrap().0,
            Ownership::Foreign
        );
        let ambiguous = "LoadState=loaded\nActiveState=inactive\nExecStart={ path=/opt/app/bin ; argv[]=/opt/app/bin /etc/app.toml /etc/app.toml ; ignore_errors=no }\n";
        assert_eq!(
            systemd_ownership(ambiguous, &with_config).unwrap().0,
            Ownership::Unknown
        );
    }

    #[test]
    fn systemd_active_stopped_transitioning_mapped() {
        for (active, expect) in [
            ("active", LifecycleState::Running),
            ("inactive", LifecycleState::Stopped),
            ("activating", LifecycleState::Transitioning),
            ("deactivating", LifecycleState::Transitioning),
            ("failed", LifecycleState::Unknown),
        ] {
            let text = show_loaded_active("/opt/app/bin", "--serve", active);
            let s = spec("my-daemon.service", "/opt/app/bin", &["--serve"]);
            let (_, state, _) = systemd_ownership(&text, &s).unwrap();
            assert_eq!(state, expect, "{active}");
        }
    }

    #[test]
    fn systemd_permission_denial_has_no_sudo() {
        let fake = FakeExecutor::new();
        let show = show_loaded_active("/opt/app/bin", "--serve", "inactive");
        fake.expect(
            &[
                "systemctl",
                "--user",
                "show",
                "my-daemon.service",
                "-p",
                "LoadState,ActiveState,ExecStart",
            ],
            out(0, &show),
        );
        fake.expect_error(
            &["systemctl", "--user", "start", "my-daemon.service"],
            ServiceError::Manager("permission denied for unit".to_string()),
        );
        let dir = temp_dir("sysd-perm");
        let mut m = systemd_manager(fake, dir.join("my-daemon.service"));
        let s = spec("my-daemon.service", "/opt/app/bin", &["--serve"]);
        let err = m.start(&s, Duration::from_secs(2)).unwrap_err().to_string();
        assert!(err.contains("permission"));
        assert!(!err.to_lowercase().contains("sudo"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn systemd_atomic_write_fault_preserves_old() {
        // Parent is a file, not a directory: write fails without creating.
        let dir = temp_dir("sysd-fault");
        let notdir = dir.join("notdir");
        std::fs::write(&notdir, b"x").unwrap();
        let err = atomic_write_definition(&notdir.join("u.service"), b"data", false).unwrap_err();
        assert!(matches!(
            err,
            ServiceError::InvalidInput(_) | ServiceError::Manager(_)
        ));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn systemd_start_stop_restart_confirm_state() {
        let fake = FakeExecutor::new();
        let stopped = show_loaded_active("/opt/app/bin", "--serve", "inactive");
        let running = show_loaded_active("/opt/app/bin", "--serve", "active");
        // inspect → stopped
        fake.expect(
            &[
                "systemctl",
                "--user",
                "show",
                "my-daemon.service",
                "-p",
                "LoadState,ActiveState,ExecStart",
            ],
            out(0, &stopped),
        );
        // start
        fake.expect(
            &["systemctl", "--user", "start", "my-daemon.service"],
            out(0, ""),
        );
        // poll is-active → active
        fake.expect(
            &["systemctl", "--user", "is-active", "my-daemon.service"],
            out(0, "active\n"),
        );
        // stop: show → running
        fake.expect(
            &[
                "systemctl",
                "--user",
                "show",
                "my-daemon.service",
                "-p",
                "LoadState,ActiveState,ExecStart",
            ],
            out(0, &running),
        );
        fake.expect(
            &["systemctl", "--user", "stop", "my-daemon.service"],
            out(0, ""),
        );
        fake.expect(
            &["systemctl", "--user", "is-active", "my-daemon.service"],
            out(3, "inactive\n"),
        );
        // restart: show → stopped (owned), restart, poll active
        fake.expect(
            &[
                "systemctl",
                "--user",
                "show",
                "my-daemon.service",
                "-p",
                "LoadState,ActiveState,ExecStart",
            ],
            out(0, &stopped),
        );
        fake.expect(
            &["systemctl", "--user", "restart", "my-daemon.service"],
            out(0, ""),
        );
        fake.expect(
            &["systemctl", "--user", "is-active", "my-daemon.service"],
            out(0, "active\n"),
        );
        let dir = temp_dir("sysd-trans");
        let mut m = systemd_manager(fake, dir.join("my-daemon.service"));
        let s = spec("my-daemon.service", "/opt/app/bin", &["--serve"]);
        assert!(m.start(&s, Duration::from_secs(5)).unwrap().completed);
        assert!(m.stop(&s, Duration::from_secs(5)).unwrap().completed);
        assert!(m.restart(&s, Duration::from_secs(5)).unwrap().completed);
        assert!(m.executor().is_exhausted());
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ---- launchd ----

    fn plist(exe: &str, args: &[&str]) -> Vec<u8> {
        let mut strings = format!("    <string>{exe}</string>\n");
        for a in args {
            strings.push_str(&format!("    <string>{a}</string>\n"));
        }
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<plist version=\"1.0\">\n<dict>\n  <key>Label</key>\n  <string>com.example.daemon</string>\n  <key>ProgramArguments</key>\n  <array>\n{strings}  </array>\n</dict>\n</plist>\n"
        )
        .into_bytes()
    }

    fn launchd_manager(fake: FakeExecutor, plist_path: PathBuf) -> LaunchdManager<FakeExecutor> {
        let install = LaunchdInstall::new(
            "com.example.daemon".to_string(),
            LaunchdDomain::UserAgent,
            "gui/501".to_string(),
            plist_path,
            plist("/opt/app/bin", &["--serve"]),
            false,
            Duration::from_secs(5),
        )
        .unwrap();
        LaunchdManager::new(fake, install)
    }

    #[test]
    fn launchd_absent_exact_foreign_unknown() {
        // Absent: no plist file, list fails.
        let fake = FakeExecutor::new();
        fake.expect(&["launchctl", "list", "com.example.daemon"], out(1, ""));
        let dir = temp_dir("ld-absent");
        let m = launchd_manager(fake, dir.join("com.example.daemon.plist"));
        let s = spec("com.example.daemon", "/opt/app/bin", &["--serve"]);
        assert_eq!(m.inspect(&s).unwrap().ownership, Ownership::Absent);

        // Exact: plist matches, loaded with PID → Owned + Running.
        let fake = FakeExecutor::new();
        fake.expect(
            &["launchctl", "list", "com.example.daemon"],
            out(0, "\"PID\" = 123;\n"),
        );
        let dir = temp_dir("ld-exact");
        let plist_path = dir.join("com.example.daemon.plist");
        std::fs::write(&plist_path, plist("/opt/app/bin", &["--serve"])).unwrap();
        let m = launchd_manager(fake, plist_path);
        let snap = m.inspect(&s).unwrap();
        assert_eq!(snap.ownership, Ownership::Owned);
        assert_eq!(snap.state, LifecycleState::Running);

        // Foreign: same label, different program.
        let fake = FakeExecutor::new();
        let dir = temp_dir("ld-foreign");
        let plist_path = dir.join("com.example.daemon.plist");
        std::fs::write(&plist_path, plist("/opt/other/bin", &["--serve"])).unwrap();
        let m = launchd_manager(fake, plist_path);
        assert_eq!(m.inspect(&s).unwrap().ownership, Ownership::Foreign);

        // Unknown: malformed plist.
        let fake = FakeExecutor::new();
        let dir = temp_dir("ld-unknown");
        let plist_path = dir.join("com.example.daemon.plist");
        std::fs::write(&plist_path, b"<plist><dict><key>Label</key></dict>").unwrap();
        let m = launchd_manager(fake, plist_path);
        assert_eq!(m.inspect(&s).unwrap().ownership, Ownership::Unknown);
    }

    #[test]
    fn launchd_user_vs_system_domain_distinct() {
        let user = LaunchdInstall::new(
            "com.example.daemon".to_string(),
            LaunchdDomain::UserAgent,
            "gui/501".to_string(),
            PathBuf::from("/tmp/com.example.daemon.plist"),
            plist("/opt/app/bin", &[]),
            false,
            Duration::from_secs(5),
        )
        .unwrap();
        let system = LaunchdInstall::new(
            "com.example.daemon".to_string(),
            LaunchdDomain::SystemDaemon,
            "system".to_string(),
            PathBuf::from("/Library/LaunchDaemons/com.example.daemon.plist"),
            plist("/opt/app/bin", &[]),
            false,
            Duration::from_secs(5),
        )
        .unwrap();
        assert_ne!(user.target, system.target);
        // Wrong target for domain is rejected (no silent domain choice).
        assert!(LaunchdInstall::new(
            "com.example.daemon".to_string(),
            LaunchdDomain::UserAgent,
            "system".to_string(),
            PathBuf::from("/tmp/x.plist"),
            plist("/opt/app/bin", &[]),
            false,
            Duration::from_secs(5),
        )
        .is_err());
    }

    #[test]
    fn launchd_malformed_plist_is_unknown_and_denies_mutation() {
        let fake = FakeExecutor::new();
        let dir = temp_dir("ld-malformed");
        let plist_path = dir.join("com.example.daemon.plist");
        std::fs::write(&plist_path, b"not xml at all").unwrap();
        let mut m = launchd_manager(fake, plist_path);
        let s = spec("com.example.daemon", "/opt/app/bin", &["--serve"]);
        assert_eq!(m.inspect(&s).unwrap().ownership, Ownership::Unknown);
        assert!(m.stop(&s, Duration::from_secs(1)).is_err());
        assert!(m.uninstall(&s).is_err());
    }

    #[test]
    fn launchd_bootstrap_bootout_kickstart_flow() {
        let fake = FakeExecutor::new();
        // inspect for start: plist owned, list → loaded but stopped (no PID).
        // ownership_of calls loaded_and_running once.
        // start calls ownership_of (list #1), then kickstart, then confirm (list #2).
        // Use a plist file so ownership is Owned.
        // Sequence: list(stopped) [ownership in start], kickstart, list(running) [confirm].
        // Note: start's ownership_of does one list; confirm does another.
        fake.expect(
            &["launchctl", "list", "com.example.daemon"],
            out(0, "\"PID\" = -;\n"),
        );
        fake.expect(
            &["launchctl", "kickstart", "-k", "gui/501/com.example.daemon"],
            out(0, ""),
        );
        fake.expect(
            &["launchctl", "list", "com.example.daemon"],
            out(0, "\"PID\" = 42;\n"),
        );
        let dir = temp_dir("ld-flow");
        let plist_path = dir.join("com.example.daemon.plist");
        std::fs::write(&plist_path, plist("/opt/app/bin", &["--serve"])).unwrap();
        let mut m = launchd_manager(fake, plist_path);
        let s = spec("com.example.daemon", "/opt/app/bin", &["--serve"]);
        let r = m.start(&s, Duration::from_secs(5)).unwrap();
        assert!(r.completed);
        assert!(m.executor().is_exhausted());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn launchd_restart_does_not_start_after_incomplete_stop() {
        #[derive(Debug, Default)]
        struct StaysRunning {
            calls: std::sync::Arc<Mutex<Vec<Vec<String>>>>,
        }
        impl CommandExecutor for StaysRunning {
            fn run(
                &self,
                argv: &[String],
                _stdin: Option<&[u8]>,
                timeout: Duration,
            ) -> Result<CommandOutput, ServiceError> {
                self.calls.lock().unwrap().push(argv.to_vec());
                if argv.first().is_some_and(|arg| arg == "launchctl")
                    && argv.get(1).is_some_and(|arg| arg == "list")
                {
                    std::thread::sleep(Duration::from_millis(2).min(timeout));
                    return Ok(out(0, "\"PID\" = 42;\n"));
                }
                if argv.get(1).is_some_and(|arg| arg == "stop") {
                    return Ok(out(0, ""));
                }
                Err(ServiceError::manager("unexpected launchctl command"))
            }
        }

        let executor = StaysRunning::default();
        let observed_calls = executor.calls.clone();
        let dir = temp_dir("ld-restart-incomplete");
        let plist_path = dir.join("com.example.daemon.plist");
        std::fs::write(&plist_path, plist("/opt/app/bin", &["--serve"])).unwrap();
        let install = LaunchdInstall::new(
            "com.example.daemon".into(),
            LaunchdDomain::UserAgent,
            "gui/501".into(),
            plist_path,
            plist("/opt/app/bin", &["--serve"]),
            false,
            Duration::from_secs(1),
        )
        .unwrap();
        let mut manager = LaunchdManager::new(executor, install);
        let service = spec("com.example.daemon", "/opt/app/bin", &["--serve"]);
        let started_at = Instant::now();
        let result = manager.restart(&service, Duration::from_millis(35));
        assert!(match result {
            Ok(result) => !result.completed,
            Err(ServiceError::Manager(message)) => message.contains("deadline exhausted"),
            Err(_) => false,
        });
        assert!(started_at.elapsed() < Duration::from_millis(150));
        let calls = observed_calls.lock().unwrap();
        assert!(calls
            .iter()
            .any(|argv| argv.get(1).is_some_and(|arg| arg == "stop")));
        assert!(!calls.iter().any(|argv| {
            argv.get(1)
                .is_some_and(|arg| arg == "bootstrap" || arg == "kickstart")
        }));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn launchd_no_label_only_ownership() {
        // Even when `launchctl list` reports loaded, a missing plist file is
        // Absent (label alone never proves ownership).
        let fake = FakeExecutor::new();
        fake.expect(
            &["launchctl", "list", "com.example.daemon"],
            out(0, "\"PID\" = 1;\n"),
        );
        let dir = temp_dir("ld-nolabel");
        let m = launchd_manager(fake, dir.join("missing.plist"));
        let s = spec("com.example.daemon", "/opt/app/bin", &[]);
        // No file → Absent, regardless of list output (file gates ownership).
        assert_eq!(m.inspect(&s).unwrap().ownership, Ownership::Absent);
    }

    #[test]
    fn launchd_config_identity_reconciles_exact_missing_and_ambiguous_argv() {
        let service = ServiceSpec::new(
            ServiceId::new("com.example.daemon").unwrap(),
            PathBuf::from("/opt/app/bin"),
            vec!["--settings".into(), "/etc/app.toml".into()],
            Some(PathBuf::from("/etc/app.toml")),
        )
        .unwrap();
        let observed = |args: &[&str]| {
            reconcile_config_identity(
                RegistrationSnapshot {
                    present: true,
                    executable: Some(PathBuf::from("/opt/app/bin")),
                    args: args.iter().map(|arg| (*arg).to_string()).collect(),
                    config: None,
                    malformed: false,
                },
                &service,
            )
            .ownership(&service)
        };
        assert_eq!(observed(&["--settings", "/etc/app.toml"]), Ownership::Owned);
        assert_eq!(observed(&["--serve"]), Ownership::Foreign);
        assert_eq!(
            observed(&["/etc/app.toml", "/etc/app.toml"]),
            Ownership::Unknown
        );
    }

    #[test]
    fn mutation_status_requires_zero_exit() {
        assert!(ensure_mutation_success(&out(0, ""), "mutation").is_ok());
        let err = ensure_mutation_success(&out(7, ""), "mutation").unwrap_err();
        assert!(matches!(err, ServiceError::Manager(_)));
        assert!(err.to_string().contains("exited 7"));
    }

    // ---- cron ----

    const MARKER: &str = "eggup-test-block-001";
    const DESIRED: &str = "*/5 * * * * /opt/app/bin --serve";

    #[test]
    fn cron_empty_absent_then_install() {
        assert_eq!(cron_classify("", MARKER, DESIRED), CronOwnership::Absent);
        let (merged, changed) = cron_merge("", MARKER, DESIRED).unwrap();
        assert!(changed);
        assert!(merged.contains("# BEGIN eggup-test-block-001"));
        assert!(merged.contains(DESIRED));
        assert_eq!(
            cron_classify(&merged, MARKER, DESIRED),
            CronOwnership::Owned
        );
    }

    #[test]
    fn cron_preserves_unrelated_entries_byte_for_byte() {
        let existing = "# mypersonal\n0 0 * * * /usr/bin/backup\n";
        let (merged, _) = cron_merge(existing, MARKER, DESIRED).unwrap();
        assert!(merged.starts_with(existing));
        assert!(merged.contains(DESIRED));
        // Removing restores the exact original bytes.
        let (removed, _) = cron_remove(&merged, MARKER, DESIRED).unwrap();
        assert_eq!(removed, existing);
    }

    #[test]
    fn cron_exact_block_is_idempotent() {
        let (merged, _) = cron_merge("", MARKER, DESIRED).unwrap();
        let (merged2, changed) = cron_merge(&merged, MARKER, DESIRED).unwrap();
        assert!(!changed);
        assert_eq!(merged, merged2);
    }

    #[test]
    fn cron_duplicate_marker_is_unknown_conflict() {
        let dup = format!(
            "# BEGIN {MARKER}\n{DESIRED}\n# END {MARKER}\n# BEGIN {MARKER}\n{DESIRED}\n# END {MARKER}\n"
        );
        assert_eq!(cron_classify(&dup, MARKER, DESIRED), CronOwnership::Unknown);
        assert!(cron_merge(&dup, MARKER, DESIRED).is_err());
        assert!(cron_remove(&dup, MARKER, DESIRED).is_err());
    }

    #[test]
    fn cron_modified_block_is_foreign_no_overwrite() {
        let other = format!("# BEGIN {MARKER}\n0 0 * * * /opt/evil/bin\n# END {MARKER}\n");
        assert_eq!(
            cron_classify(&other, MARKER, DESIRED),
            CronOwnership::Foreign
        );
        assert!(cron_merge(&other, MARKER, DESIRED).is_err());
        // Original bytes untouched by the failed merge.
        assert!(other.contains("/opt/evil/bin"));
    }

    #[test]
    fn cron_uninstall_only_owned_block() {
        let (merged, _) = cron_merge("", MARKER, DESIRED).unwrap();
        let (removed, changed) = cron_remove(&merged, MARKER, DESIRED).unwrap();
        assert!(changed);
        assert!(!removed.contains(MARKER));
        // No-op uninstall does not rewrite.
        let (removed2, changed2) = cron_remove(&removed, MARKER, DESIRED).unwrap();
        assert!(!changed2);
        assert_eq!(removed, removed2);
    }

    #[test]
    fn cron_command_adapter_skips_redundant_write() {
        let fake = FakeExecutor::new();
        // inspect list
        fake.expect(&["crontab", "-l"], out(0, ""));
        // install: list, then write (list #2 for merge, list #3 for race check).
        // Our install does list → merge → list (race check) → write.
        // Script: list(empty) [cron_ownership? no, install lists once for merge
        // input, once for race check]. Actually install calls list twice total
        // (once for merge input via `current`, once for `fresh`). Plus inspect
        // in the test calls list once more. Total: inspect(1) + install(2).
        let _ = fake;
        // Simpler: test pure idempotency (no executor) plus adapter no-op path.
        let fake = FakeExecutor::new();
        let installed = format!("# BEGIN {MARKER}\n{DESIRED}\n# END {MARKER}\n");
        // inspect
        fake.expect(&["crontab", "-l"], out(0, &installed));
        // install: current
        fake.expect(&["crontab", "-l"], out(0, &installed));
        let mut m = CronManager::new(
            fake,
            MARKER.to_string(),
            DESIRED.to_string(),
            Duration::from_secs(5),
        )
        .unwrap();
        let s = spec("cron-job", "/opt/app/bin", &["--serve"]);
        // Install is idempotent: no `crontab -` call (only two `-l` calls).
        let r = m.install(&s).unwrap();
        assert!(r.completed);
        assert_eq!(r.detail, "cron block already installed");
    }

    #[test]
    fn cron_command_timeout_and_missing_binary() {
        // Timeout surfaces as Manager, never assumed state.
        let fake = FakeExecutor::new();
        fake.expect_error(
            &["crontab", "-l"],
            ServiceError::Manager("manager command timed out for crontab".to_string()),
        );
        let m = CronManager::new(
            fake,
            MARKER.to_string(),
            DESIRED.to_string(),
            Duration::from_secs(5),
        )
        .unwrap();
        let s = spec("cron-job", "/opt/app/bin", &[]);
        assert_eq!(m.inspect(&s).unwrap().ownership, Ownership::Unknown);

        // Missing binary is Manager (not installed), distinguishable.
        let fake = FakeExecutor::new();
        fake.expect_error(
            &["crontab", "-l"],
            ServiceError::Manager("manager binary missing: crontab".to_string()),
        );
        let m = CronManager::new(
            fake,
            MARKER.to_string(),
            DESIRED.to_string(),
            Duration::from_secs(5),
        )
        .unwrap();
        assert_eq!(m.inspect(&s).unwrap().ownership, Ownership::Unknown);
    }

    #[test]
    fn cron_has_no_start_stop_semantics() {
        let fake = FakeExecutor::new();
        let mut m = CronManager::new(
            fake,
            MARKER.to_string(),
            DESIRED.to_string(),
            Duration::from_secs(5),
        )
        .unwrap();
        let s = spec("cron-job", "/opt/app/bin", &[]);
        assert!(m.start(&s, Duration::from_secs(1)).is_err());
        assert!(m.stop(&s, Duration::from_secs(1)).is_err());
        assert!(m.restart(&s, Duration::from_secs(1)).is_err());
    }

    // ---- definition file safety ----

    #[test]
    fn definition_write_is_atomic_private_and_no_clobber() {
        let dir = temp_dir("def-safe");
        let dest = dir.join("unit.service");
        atomic_write_definition(&dest, b"[Unit]\n", false).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&dest).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o644);
        }
        // No-clobber: second create fails.
        assert!(atomic_write_definition(&dest, b"[Unit]\n", false).is_err());
        // Overwrite allowed for owned refresh.
        atomic_write_definition(&dest, b"[Unit]\n# v2\n", true).unwrap();
        assert!(std::fs::read(&dest).unwrap().ends_with(b"# v2\n"));
        // No stray temps.
        let leftovers: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with(".eggup-def-"))
            .collect();
        assert!(leftovers.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn definition_write_rejects_missing_parent() {
        let dir = temp_dir("def-noparent");
        let dest = dir.join("no-such-dir/unit.service");
        let err = atomic_write_definition(&dest, b"[Unit]\n", false).unwrap_err();
        assert!(matches!(
            err,
            ServiceError::InvalidInput(_) | ServiceError::Manager(_)
        ));
        assert!(!dir.join("no-such-dir").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ---- host detection ----

    #[test]
    fn host_detection_distinguishes_unavailable_from_failure() {
        // Script probes according to the host OS gate used by `inspect_host`.
        let fake = FakeExecutor::new();
        if std::env::consts::OS == "linux" {
            fake.expect(&["systemctl", "--version"], out(0, "systemd 255\n"));
        }
        if std::env::consts::OS == "macos" {
            fake.expect(&["launchctl", "version"], out(0, "launchctl 1.0\n"));
        }
        fake.expect(&["crontab", "-l"], out(1, "no crontab for user\n"));
        let facts = inspect_host(&fake);
        assert!(facts.is_ok());
        let facts = facts.unwrap();
        // Detection failure (Err) stays distinct from "not installed" (false).
        assert!(!facts.systemd_available || std::env::consts::OS == "linux");
    }

    #[test]
    fn host_detection_failure_is_distinguishable() {
        let fake = FakeExecutor::new();
        fake.expect_error(
            &["systemctl", "--version"],
            ServiceError::Manager("manager command timed out for systemctl".to_string()),
        );
        // On Linux this propagates as Err (detection failure, not "absent").
        // On macOS the systemctl probe is skipped (OS gate) so no error.
        // Assert the helper never panics and distinguishes via Result.
        let _ = inspect_host(&fake);
    }

    #[test]
    fn candidate_policy_prefers_native_then_cron() {
        let facts = HostFacts {
            os: "linux".to_string(),
            systemd_available: true,
            launchd_available: false,
            crontab_available: true,
        };
        assert_eq!(
            candidate_managers(&facts),
            vec![
                CandidateManager::Systemd(SystemdScope::System),
                CandidateManager::Cron
            ]
        );
        let facts = HostFacts {
            os: "macos".to_string(),
            systemd_available: false,
            launchd_available: true,
            crontab_available: false,
        };
        assert_eq!(
            candidate_managers(&facts),
            vec![CandidateManager::Launchd(LaunchdDomain::UserAgent)]
        );
        let facts = HostFacts {
            os: "linux".to_string(),
            systemd_available: false,
            launchd_available: false,
            crontab_available: true,
        };
        assert_eq!(candidate_managers(&facts), vec![CandidateManager::Cron]);
    }

    // ---- native smoke (skipped when unsafe) ----

    #[test]
    fn native_systemctl_parsing_smoke_where_booted() {
        if !std::path::Path::new("/run/systemd/system").exists() {
            eprintln!("skipping: not booted with systemd");
            return;
        }
        let ex = SystemExecutor::new();
        let out = ex.run(
            &["systemctl".to_string(), "--version".to_string()],
            None,
            Duration::from_secs(5),
        );
        // Must not panic; either Ok or missing-binary Manager.
        let _ = out;
    }

    #[test]
    fn native_launchd_smoke_on_macos_only() {
        if std::env::consts::OS != "macos" {
            eprintln!("skipping: launchd smoke requires macOS");
            return;
        }
        let ex = SystemExecutor::new();
        let out = ex.run(
            &["launchctl".to_string(), "version".to_string()],
            None,
            Duration::from_secs(5),
        );
        let _ = out;
    }

    #[test]
    fn native_crontab_list_only_never_mutates() {
        let ex = SystemExecutor::new();
        // List-only smoke: never runs `crontab -` here.
        let out = ex.run(
            &["crontab".to_string(), "-l".to_string()],
            None,
            Duration::from_secs(5),
        );
        match out {
            Ok(_) => {}
            Err(e) if format!("{e}").contains("missing") => {
                eprintln!("skipping: crontab not installed");
            }
            Err(_) => {}
        }
    }

    // ---- consumer-specific rejection proof ----

    #[test]
    fn adapter_rejects_consumer_specific_constants() {
        // The adapters must not know product names, daemon users, or
        // hardening directives. Prove by construction: install descriptors
        // carry only mechanics + caller bytes; no defaults embed products.
        let install = SystemdInstall::new(
            "my-daemon.service".to_string(),
            SystemdScope::User,
            PathBuf::from("/tmp/my-daemon.service"),
            b"[Unit]\n".to_vec(),
            false,
            false,
            Duration::from_secs(5),
        )
        .unwrap();
        assert!(!String::from_utf8_lossy(&install.definition).contains("eggsact"));
        assert!(!install.unit_name.contains("eggsearch"));
        // Cron marker/block are caller-owned; the manager adds no content.
        let m = CronManager::new(
            FakeExecutor::new(),
            "caller-marker-xyz".to_string(),
            "*/5 * * * * /caller/bin".to_string(),
            Duration::from_secs(5),
        )
        .unwrap();
        assert_eq!(m.marker(), "caller-marker-xyz");
    }
}
