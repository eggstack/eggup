#[cfg(test)]
use std::collections::VecDeque;
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::{
    reconcile_config_identity, require_owned, HealthState, LifecycleSnapshot, LifecycleState,
    OperationDeadline, Ownership, RegistrationSnapshot, ServiceError, ServiceId, ServiceManager,
    ServiceOperation, ServiceSpec, TransitionResult, MAX_TRANSITION_TIMEOUT,
};

const MAX_SCM_NAME_CHARS: usize = 256;
const MAX_SCM_COMMAND_CHARS: usize = 8192;
const DEFAULT_SCM_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_POLL_INTERVAL: Duration = Duration::from_millis(50);
const MAX_POLL_INTERVAL: Duration = Duration::from_millis(500);

/// Caller-selected Windows service startup mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowsStartType {
    /// Start automatically when Windows starts.
    Automatic,
    /// Start only when requested by an operator or manager.
    Manual,
    /// Leave registered but prevent start requests from succeeding.
    Disabled,
}

/// Caller-selected SCM error-control setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowsErrorControl {
    /// Log the error and continue startup.
    Ignore,
    /// Log the error and continue startup.
    Normal,
    /// Log the error; restart using the last-known-good configuration.
    Severe,
    /// Log the error; startup fails using the last-known-good configuration.
    Critical,
}

/// Caller-owned settings used when creating or refreshing an SCM registration.
///
/// The descriptor intentionally omits failure actions, service descriptions,
/// dependencies, custom passwords, and service-entrypoint configuration. A
/// refresh preserves existing fields that this descriptor does not own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowsScmInstall {
    service_id: ServiceId,
    display_name: String,
    start_type: WindowsStartType,
    error_control: WindowsErrorControl,
    account_name: Option<String>,
    transition_timeout: Duration,
}

impl WindowsScmInstall {
    /// Creates a caller-owned SCM install descriptor.
    pub fn new(
        service_name: impl Into<String>,
        display_name: impl Into<String>,
        start_type: WindowsStartType,
        error_control: WindowsErrorControl,
        account_name: Option<String>,
    ) -> Result<Self, ServiceError> {
        let service_name = service_name.into();
        validate_service_name(&service_name)?;
        let display_name = display_name.into();
        validate_display_name(&display_name)?;
        if let Some(account) = &account_name {
            validate_account_name(account)?;
        }
        Ok(Self {
            service_id: ServiceId::new(service_name)?,
            display_name,
            start_type,
            error_control,
            account_name,
            transition_timeout: DEFAULT_SCM_TIMEOUT,
        })
    }

    /// Sets the bounded timeout used by install/uninstall and SCM polling.
    pub fn with_transition_timeout(mut self, timeout: Duration) -> Result<Self, ServiceError> {
        if timeout.is_zero() || timeout > MAX_TRANSITION_TIMEOUT {
            return Err(ServiceError::invalid("transition timeout out of range"));
        }
        self.transition_timeout = timeout;
        Ok(self)
    }

    /// The Windows service key name.
    pub fn service_id(&self) -> &ServiceId {
        &self.service_id
    }

    /// The caller-owned display name.
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    /// The caller-owned startup mode.
    pub fn start_type(&self) -> WindowsStartType {
        self.start_type
    }

    /// The caller-owned error-control setting.
    pub fn error_control(&self) -> WindowsErrorControl {
        self.error_control
    }

    /// Optional caller-selected account name; no account password is accepted or stored.
    pub fn account_name(&self) -> Option<&str> {
        self.account_name.as_deref()
    }

    /// The bounded timeout used by install/uninstall and SCM polling.
    pub fn transition_timeout(&self) -> Duration {
        self.transition_timeout
    }
}

fn validate_service_name(value: &str) -> Result<(), ServiceError> {
    if value.is_empty()
        || value.encode_utf16().count() > MAX_SCM_NAME_CHARS
        || value.chars().any(|c| c.is_control() || c == '\0')
        || value.contains(['/', '\\'])
    {
        return Err(ServiceError::invalid(
            "Windows service name is empty, overlong, path-like, or has controls",
        ));
    }
    Ok(())
}

fn validate_display_name(value: &str) -> Result<(), ServiceError> {
    if value.is_empty()
        || value.encode_utf16().count() > MAX_SCM_NAME_CHARS
        || value.chars().any(|c| c.is_control() || c == '\0')
    {
        return Err(ServiceError::invalid(
            "Windows service display name is empty, overlong, or has controls",
        ));
    }
    Ok(())
}

fn validate_account_name(value: &str) -> Result<(), ServiceError> {
    if value.is_empty()
        || value.encode_utf16().count() > MAX_SCM_NAME_CHARS
        || value.chars().any(|c| c.is_control() || c == '\0')
    {
        return Err(ServiceError::invalid(
            "Windows service account name is empty, overlong, or has controls",
        ));
    }
    Ok(())
}

/// Native Windows SCM adapter implementing the manager-neutral contract.
///
/// `new` is available on Windows. The ownership, command-line parser, and
/// deterministic backend tests are platform-independent. No `sc.exe`, shell,
/// ambient `PATH`, or automatic elevation is used.
#[derive(Debug)]
pub struct WindowsScmManager {
    backend: Box<dyn ScmBackend>,
    install: WindowsScmInstall,
}

impl WindowsScmManager {
    /// Creates an adapter using the local Windows Service Control Manager.
    #[cfg(windows)]
    pub fn new(install: WindowsScmInstall) -> Self {
        Self {
            backend: Box::<WindowsBackend>::default(),
            install,
        }
    }

    /// Returns the caller-owned descriptor.
    pub fn install_config(&self) -> &WindowsScmInstall {
        &self.install
    }

    #[cfg(test)]
    fn with_backend(backend: impl ScmBackend + 'static, install: WindowsScmInstall) -> Self {
        Self {
            backend: Box::new(backend),
            install,
        }
    }

    fn validate_spec(&self, spec: &ServiceSpec) -> Result<(), ServiceError> {
        if spec.id() != &self.install.service_id {
            return Err(ServiceError::invalid(
                "service spec id does not match Windows install descriptor",
            ));
        }
        if !is_windows_absolute_path(spec.executable()) {
            return Err(ServiceError::invalid(
                "Windows service executable must be an absolute Windows path",
            ));
        }
        let executable = spec.executable().to_str().ok_or_else(|| {
            ServiceError::invalid("Windows service executable must be representable as UTF-8")
        })?;
        if let Some(config) = spec.config() {
            if config.to_str().is_none() {
                return Err(ServiceError::invalid(
                    "Windows service config path must be representable as UTF-8",
                ));
            }
        }
        let command_budget = executable
            .encode_utf16()
            .count()
            .saturating_mul(2)
            .saturating_add(2)
            .saturating_add(
                spec.args()
                    .iter()
                    .map(|arg| {
                        arg.encode_utf16()
                            .count()
                            .saturating_mul(2)
                            .saturating_add(3)
                    })
                    .sum::<usize>(),
            );
        if command_budget > MAX_SCM_COMMAND_CHARS {
            return Err(ServiceError::invalid(
                "Windows service command line exceeds the supported bound",
            ));
        }
        Ok(())
    }

    fn observe(
        &mut self,
        spec: &ServiceSpec,
        deadline: OperationDeadline,
    ) -> Result<(Ownership, ScmLookup), ServiceError> {
        self.validate_spec(spec)?;
        let lookup = self.backend.query(spec.id(), deadline.command_timeout()?)?;
        if deadline.expired() {
            return Err(ServiceError::manager(
                "SCM query exceeded transition deadline",
            ));
        }
        let ownership = match &lookup {
            ScmLookup::Absent => Ownership::Absent,
            ScmLookup::MarkedForDelete => Ownership::Unknown,
            ScmLookup::Present(record) => registration_ownership(record, spec),
        };
        Ok((ownership, lookup))
    }

    fn require_owned_observation(
        &mut self,
        spec: &ServiceSpec,
        deadline: OperationDeadline,
    ) -> Result<ScmRecord, ServiceError> {
        let (ownership, lookup) = self.observe(spec, deadline)?;
        require_owned(spec.id(), ownership)?;
        match lookup {
            ScmLookup::Present(record) => Ok(record),
            ScmLookup::Absent | ScmLookup::MarkedForDelete => {
                Err(ServiceError::denied(spec.id(), ownership))
            }
        }
    }

    fn wait_for_state(
        &mut self,
        spec: &ServiceSpec,
        target: ScmState,
        deadline: OperationDeadline,
    ) -> Result<bool, ServiceError> {
        loop {
            let record = self.require_owned_observation(spec, deadline)?;
            if record.state == target {
                return Ok(!deadline.expired());
            }
            if deadline.expired() {
                return Ok(false);
            }
            let wait = if record.wait_hint.is_zero() {
                DEFAULT_POLL_INTERVAL
            } else {
                record.wait_hint.min(MAX_POLL_INTERVAL)
            };
            let delay = deadline.remaining().unwrap_or_default().min(wait);
            if delay.is_zero() {
                return Ok(false);
            }
            self.backend.sleep(delay);
        }
    }

    fn stop_phase_for_restart(
        &mut self,
        spec: &ServiceSpec,
        deadline: OperationDeadline,
    ) -> Result<bool, ServiceError> {
        loop {
            let record = self.require_owned_observation(spec, deadline)?;
            match record.state {
                ScmState::Stopped => return Ok(!deadline.expired()),
                ScmState::Running | ScmState::Paused => {
                    self.backend.stop(spec.id(), deadline.command_timeout()?)?;
                    return self.wait_for_state(spec, ScmState::Stopped, deadline);
                }
                ScmState::Unknown => return Ok(false),
                ScmState::StartPending
                | ScmState::StopPending
                | ScmState::ContinuePending
                | ScmState::PausePending => {}
            }
            if deadline.expired() {
                return Ok(false);
            }
            let wait = if record.wait_hint.is_zero() {
                DEFAULT_POLL_INTERVAL
            } else {
                record.wait_hint.min(MAX_POLL_INTERVAL)
            };
            let delay = deadline.remaining().unwrap_or_default().min(wait);
            if delay.is_zero() {
                return Ok(false);
            }
            self.backend.sleep(delay);
        }
    }

    fn transition(
        operation: ServiceOperation,
        completed: bool,
        detail: &'static str,
    ) -> TransitionResult {
        TransitionResult {
            operation,
            completed,
            detail: ServiceError::bounded(detail),
        }
    }
}

impl ServiceManager for WindowsScmManager {
    fn inspect(&self, spec: &ServiceSpec) -> Result<LifecycleSnapshot, ServiceError> {
        self.validate_spec(spec)?;
        let deadline = OperationDeadline::new(self.install.transition_timeout)?;
        let lookup = self.backend.query(spec.id(), deadline.command_timeout()?)?;
        if deadline.expired() {
            return Err(ServiceError::manager(
                "SCM inspection exceeded transition deadline",
            ));
        }
        let (ownership, state, present) = match lookup {
            ScmLookup::Absent => (Ownership::Absent, LifecycleState::Stopped, false),
            ScmLookup::MarkedForDelete => (Ownership::Unknown, LifecycleState::Transitioning, true),
            ScmLookup::Present(record) => (
                registration_ownership(&record, spec),
                lifecycle_state(record.state),
                true,
            ),
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
        self.validate_spec(spec)?;
        let deadline = OperationDeadline::new(self.install.transition_timeout)?;
        let (ownership, _) = self.observe(spec, deadline)?;
        match ownership {
            Ownership::Absent => {
                self.backend
                    .create(&self.install, spec, deadline.command_timeout()?)?;
                if deadline.expired() {
                    return Ok(Self::transition(
                        ServiceOperation::Install,
                        false,
                        "create accepted; registration confirmation exceeded deadline",
                    ));
                }
                let (after, _) = self.observe(spec, deadline)?;
                if after != Ownership::Owned {
                    return Err(ServiceError::denied(spec.id(), after));
                }
                Ok(Self::transition(
                    ServiceOperation::Install,
                    true,
                    "created owned Windows service registration",
                ))
            }
            Ownership::Owned => {
                let (rechecked, _) = self.observe(spec, deadline)?;
                if rechecked != Ownership::Owned {
                    return Err(ServiceError::denied(spec.id(), rechecked));
                }
                self.backend
                    .refresh(&self.install, spec, deadline.command_timeout()?)?;
                if deadline.expired() {
                    return Ok(Self::transition(
                        ServiceOperation::Install,
                        false,
                        "refresh accepted; registration confirmation exceeded deadline",
                    ));
                }
                let (after, _) = self.observe(spec, deadline)?;
                if after != Ownership::Owned {
                    return Err(ServiceError::denied(spec.id(), after));
                }
                Ok(Self::transition(
                    ServiceOperation::Install,
                    true,
                    "refreshed owned Windows service settings",
                ))
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
        self.validate_spec(spec)?;
        let deadline = OperationDeadline::new(timeout)?;
        let record = self.require_owned_observation(spec, deadline)?;
        match record.state {
            ScmState::Running => Ok(Self::transition(
                ServiceOperation::Start,
                !deadline.expired(),
                "already running",
            )),
            ScmState::Stopped => {
                self.backend.start(spec.id(), deadline.command_timeout()?)?;
                let completed = self.wait_for_state(spec, ScmState::Running, deadline)?;
                Ok(Self::transition(
                    ServiceOperation::Start,
                    completed,
                    if completed {
                        "started"
                    } else {
                        "start incomplete; running state unconfirmed"
                    },
                ))
            }
            ScmState::StartPending | ScmState::ContinuePending => {
                let completed = self.wait_for_state(spec, ScmState::Running, deadline)?;
                Ok(Self::transition(
                    ServiceOperation::Start,
                    completed,
                    if completed {
                        "start transition completed"
                    } else {
                        "start incomplete; running state unconfirmed"
                    },
                ))
            }
            ScmState::StopPending
            | ScmState::PausePending
            | ScmState::Paused
            | ScmState::Unknown => Ok(Self::transition(
                ServiceOperation::Start,
                false,
                "start not issued from an unsupported or conflicting SCM state",
            )),
        }
    }

    fn stop(
        &mut self,
        spec: &ServiceSpec,
        timeout: Duration,
    ) -> Result<TransitionResult, ServiceError> {
        self.validate_spec(spec)?;
        let deadline = OperationDeadline::new(timeout)?;
        let record = self.require_owned_observation(spec, deadline)?;
        match record.state {
            ScmState::Stopped => Ok(Self::transition(
                ServiceOperation::Stop,
                !deadline.expired(),
                "already stopped",
            )),
            ScmState::Running | ScmState::Paused => {
                self.backend.stop(spec.id(), deadline.command_timeout()?)?;
                let completed = self.wait_for_state(spec, ScmState::Stopped, deadline)?;
                Ok(Self::transition(
                    ServiceOperation::Stop,
                    completed,
                    if completed {
                        "stopped"
                    } else {
                        "stop incomplete; stopped state unconfirmed"
                    },
                ))
            }
            ScmState::StopPending => {
                let completed = self.wait_for_state(spec, ScmState::Stopped, deadline)?;
                Ok(Self::transition(
                    ServiceOperation::Stop,
                    completed,
                    if completed {
                        "stop transition completed"
                    } else {
                        "stop incomplete; stopped state unconfirmed"
                    },
                ))
            }
            ScmState::StartPending
            | ScmState::ContinuePending
            | ScmState::PausePending
            | ScmState::Unknown => Ok(Self::transition(
                ServiceOperation::Stop,
                false,
                "stop not issued from an unsupported or conflicting SCM state",
            )),
        }
    }

    fn restart(
        &mut self,
        spec: &ServiceSpec,
        timeout: Duration,
    ) -> Result<TransitionResult, ServiceError> {
        self.validate_spec(spec)?;
        let deadline = OperationDeadline::new(timeout)?;
        let stopped = self.stop_phase_for_restart(spec, deadline)?;
        if !stopped || deadline.expired() {
            return Ok(Self::transition(
                ServiceOperation::Restart,
                false,
                "restart stop phase incomplete; start was not issued",
            ));
        }
        let stopped_record = self.require_owned_observation(spec, deadline)?;
        if stopped_record.state != ScmState::Stopped {
            return Ok(Self::transition(
                ServiceOperation::Restart,
                false,
                "restart stop phase changed before start; start was not issued",
            ));
        }
        self.backend.start(spec.id(), deadline.command_timeout()?)?;
        let completed = self.wait_for_state(spec, ScmState::Running, deadline)?;
        Ok(Self::transition(
            ServiceOperation::Restart,
            completed,
            if completed {
                "restarted"
            } else {
                "restart start phase incomplete; running state unconfirmed"
            },
        ))
    }

    fn uninstall(&mut self, spec: &ServiceSpec) -> Result<TransitionResult, ServiceError> {
        self.validate_spec(spec)?;
        let deadline = OperationDeadline::new(self.install.transition_timeout)?;
        let record = self.require_owned_observation(spec, deadline)?;
        match record.state {
            ScmState::Running | ScmState::Paused => {
                self.backend.stop(spec.id(), deadline.command_timeout()?)?;
                if !self.wait_for_state(spec, ScmState::Stopped, deadline)? {
                    return Ok(Self::transition(
                        ServiceOperation::Uninstall,
                        false,
                        "stop incomplete; service was not deleted",
                    ));
                }
            }
            ScmState::StopPending => {
                if !self.wait_for_state(spec, ScmState::Stopped, deadline)? {
                    return Ok(Self::transition(
                        ServiceOperation::Uninstall,
                        false,
                        "stop incomplete; service was not deleted",
                    ));
                }
            }
            ScmState::Stopped => {}
            ScmState::StartPending
            | ScmState::ContinuePending
            | ScmState::PausePending
            | ScmState::Unknown => {
                return Ok(Self::transition(
                    ServiceOperation::Uninstall,
                    false,
                    "service state is not safely stopped; registration was not deleted",
                ));
            }
        }
        let final_check = self.require_owned_observation(spec, deadline)?;
        if final_check.state != ScmState::Stopped {
            return Ok(Self::transition(
                ServiceOperation::Uninstall,
                false,
                "service changed before delete; registration was not deleted",
            ));
        }
        self.backend
            .delete(spec.id(), deadline.command_timeout()?)?;
        loop {
            if deadline.expired() {
                return Ok(Self::transition(
                    ServiceOperation::Uninstall,
                    false,
                    "delete accepted; service remains present or marked for deletion",
                ));
            }
            match self.backend.query(spec.id(), deadline.command_timeout()?)? {
                ScmLookup::Absent => {
                    return Ok(Self::transition(
                        ServiceOperation::Uninstall,
                        !deadline.expired(),
                        "service deletion confirmed",
                    ));
                }
                ScmLookup::MarkedForDelete => {}
                ScmLookup::Present(record) => {
                    let ownership = registration_ownership(&record, spec);
                    require_owned(spec.id(), ownership)?;
                }
            }
            let delay = deadline
                .remaining()
                .unwrap_or_default()
                .min(DEFAULT_POLL_INTERVAL);
            if delay.is_zero() {
                return Ok(Self::transition(
                    ServiceOperation::Uninstall,
                    false,
                    "delete accepted; service remains present or marked for deletion",
                ));
            }
            self.backend.sleep(delay);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Several SCM states are only constructible on Windows, but tested cross-platform.
enum ScmState {
    Stopped,
    StartPending,
    StopPending,
    Running,
    ContinuePending,
    PausePending,
    Paused,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScmRecord {
    /// `None` means the API returned a command that cannot be represented as UTF-8.
    command_line: Option<String>,
    state: ScmState,
    wait_hint: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // Production variants are constructed by the Windows backend.
enum ScmLookup {
    Absent,
    MarkedForDelete,
    Present(ScmRecord),
}

trait ScmBackend: fmt::Debug {
    fn query(&self, id: &ServiceId, remaining: Duration) -> Result<ScmLookup, ServiceError>;
    fn create(
        &mut self,
        install: &WindowsScmInstall,
        spec: &ServiceSpec,
        remaining: Duration,
    ) -> Result<(), ServiceError>;
    fn refresh(
        &mut self,
        install: &WindowsScmInstall,
        spec: &ServiceSpec,
        remaining: Duration,
    ) -> Result<(), ServiceError>;
    fn start(&mut self, id: &ServiceId, remaining: Duration) -> Result<(), ServiceError>;
    fn stop(&mut self, id: &ServiceId, remaining: Duration) -> Result<(), ServiceError>;
    fn delete(&mut self, id: &ServiceId, remaining: Duration) -> Result<(), ServiceError>;
    #[cfg(test)]
    fn test_calls(&self) -> Vec<&'static str> {
        Vec::new()
    }
    fn sleep(&mut self, delay: Duration) {
        std::thread::sleep(delay);
    }
}

fn registration_ownership(record: &ScmRecord, spec: &ServiceSpec) -> Ownership {
    let Some(command_line) = record.command_line.as_deref() else {
        return Ownership::Unknown;
    };
    let Some((executable, args)) = parse_service_command(command_line) else {
        return Ownership::Unknown;
    };
    reconcile_config_identity(
        RegistrationSnapshot {
            present: true,
            executable: Some(executable),
            args,
            config: None,
            malformed: false,
        },
        spec,
    )
    .ownership(spec)
}

fn lifecycle_state(state: ScmState) -> LifecycleState {
    match state {
        ScmState::Stopped => LifecycleState::Stopped,
        ScmState::Running => LifecycleState::Running,
        ScmState::StartPending
        | ScmState::StopPending
        | ScmState::ContinuePending
        | ScmState::PausePending => LifecycleState::Transitioning,
        ScmState::Paused | ScmState::Unknown => LifecycleState::Unknown,
    }
}

fn parse_service_command(command_line: &str) -> Option<(PathBuf, Vec<String>)> {
    if command_line.is_empty()
        || command_line.encode_utf16().count() > MAX_SCM_COMMAND_CHARS
        || command_line.chars().any(|c| c.is_control() || c == '\0')
        || !has_balanced_windows_quotes(command_line)
    {
        return None;
    }
    let mut arguments: Vec<String> = windows_args::Args::parse_cmd(command_line).collect();
    if arguments.is_empty() || arguments[0].is_empty() || !is_windows_absolute_str(&arguments[0]) {
        return None;
    }
    if arguments[0].chars().any(char::is_whitespace) && !command_line.starts_with('"') {
        return None;
    }
    if !command_line.starts_with('"') {
        let first = command_line.split_whitespace().next()?;
        if !first.to_ascii_lowercase().ends_with(".exe") {
            return None;
        }
    }
    let executable = PathBuf::from(arguments.remove(0));
    Some((executable, arguments))
}

fn has_balanced_windows_quotes(command_line: &str) -> bool {
    let mut quoted = false;
    let mut backslashes = 0usize;
    for ch in command_line.chars() {
        match ch {
            '\\' => backslashes = backslashes.saturating_add(1),
            '"' => {
                if backslashes & 1 == 0 {
                    quoted = !quoted;
                }
                backslashes = 0;
            }
            _ => backslashes = 0,
        }
    }
    !quoted
}

fn is_windows_absolute_path(path: &Path) -> bool {
    path.to_str().is_some_and(is_windows_absolute_str)
}

fn is_windows_absolute_str(path: &str) -> bool {
    let bytes = path.as_bytes();
    (bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && matches!(bytes[2], b'\\' | b'/'))
        || (path.starts_with(r"\\")
            && path[2..]
                .split(['\\', '/'])
                .filter(|part| !part.is_empty())
                .count()
                >= 2)
}

#[cfg(windows)]
fn start_type(value: WindowsStartType) -> windows_service::service::ServiceStartType {
    use windows_service::service::ServiceStartType;
    match value {
        WindowsStartType::Automatic => ServiceStartType::AutoStart,
        WindowsStartType::Manual => ServiceStartType::OnDemand,
        WindowsStartType::Disabled => ServiceStartType::Disabled,
    }
}

#[cfg(windows)]
fn error_control(value: WindowsErrorControl) -> windows_service::service::ServiceErrorControl {
    use windows_service::service::ServiceErrorControl;
    match value {
        WindowsErrorControl::Ignore => ServiceErrorControl::Ignore,
        WindowsErrorControl::Normal => ServiceErrorControl::Normal,
        WindowsErrorControl::Severe => ServiceErrorControl::Severe,
        WindowsErrorControl::Critical => ServiceErrorControl::Critical,
    }
}

#[cfg(windows)]
#[derive(Debug, Default)]
struct WindowsBackend;

#[cfg(windows)]
impl WindowsBackend {
    fn map_error(error: windows_service::Error, operation: &'static str) -> ServiceError {
        let code = match &error {
            windows_service::Error::Winapi(io_error) => io_error.raw_os_error(),
            _ => None,
        };
        let detail = if code == Some(5) {
            format!("{operation} denied by Windows SCM; run with an account authorized for this service operation")
        } else {
            format!(
                "{operation} failed in Windows SCM (code {})",
                code.unwrap_or_default()
            )
        };
        ServiceError::manager(detail)
    }

    fn manager(
        access: windows_service::service_manager::ServiceManagerAccess,
        operation: &'static str,
    ) -> Result<windows_service::service_manager::ServiceManager, ServiceError> {
        windows_service::service_manager::ServiceManager::local_computer(None::<&str>, access)
            .map_err(|error| Self::map_error(error, operation))
    }

    fn open(
        id: &ServiceId,
        access: windows_service::service::ServiceAccess,
    ) -> Result<Result<windows_service::service::Service, u32>, ServiceError> {
        use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};
        let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)
            .map_err(|error| Self::map_error(error, "open SCM"))?;
        match manager.open_service(id.as_str(), access) {
            Ok(service) => Ok(Ok(service)),
            Err(windows_service::Error::Winapi(error)) if error.raw_os_error() == Some(1060) => {
                Ok(Err(1060))
            }
            Err(windows_service::Error::Winapi(error)) if error.raw_os_error() == Some(1072) => {
                Ok(Err(1072))
            }
            Err(error) => Err(Self::map_error(error, "open service")),
        }
    }

    fn info(
        install: &WindowsScmInstall,
        spec: &ServiceSpec,
    ) -> windows_service::service::ServiceInfo {
        use std::ffi::OsString;
        use windows_service::service::{ServiceDependency, ServiceType};
        windows_service::service::ServiceInfo {
            name: OsString::from(install.service_id.as_str()),
            display_name: OsString::from(&install.display_name),
            service_type: ServiceType::OWN_PROCESS,
            start_type: start_type(install.start_type),
            error_control: error_control(install.error_control),
            executable_path: spec.executable().to_path_buf(),
            launch_arguments: spec.args().iter().map(OsString::from).collect(),
            dependencies: Vec::<ServiceDependency>::new(),
            account_name: install.account_name.as_deref().map(OsString::from),
            account_password: None,
        }
    }
}

#[cfg(windows)]
impl ScmBackend for WindowsBackend {
    fn query(&self, id: &ServiceId, _remaining: Duration) -> Result<ScmLookup, ServiceError> {
        use windows_service::service::ServiceAccess;
        match Self::open(
            id,
            ServiceAccess::QUERY_CONFIG | ServiceAccess::QUERY_STATUS,
        )? {
            Err(1060) => Ok(ScmLookup::Absent),
            Err(1072) => Ok(ScmLookup::MarkedForDelete),
            Err(_) => unreachable!(),
            Ok(service) => {
                let config = service
                    .query_config()
                    .map_err(|error| Self::map_error(error, "query service config"))?;
                let status = service
                    .query_status()
                    .map_err(|error| Self::map_error(error, "query service status"))?;
                let command_line = config.executable_path.to_str().map(str::to_owned);
                Ok(ScmLookup::Present(ScmRecord {
                    command_line,
                    state: match status.current_state {
                        windows_service::service::ServiceState::Stopped => ScmState::Stopped,
                        windows_service::service::ServiceState::StartPending => {
                            ScmState::StartPending
                        }
                        windows_service::service::ServiceState::StopPending => {
                            ScmState::StopPending
                        }
                        windows_service::service::ServiceState::Running => ScmState::Running,
                        windows_service::service::ServiceState::ContinuePending => {
                            ScmState::ContinuePending
                        }
                        windows_service::service::ServiceState::PausePending => {
                            ScmState::PausePending
                        }
                        windows_service::service::ServiceState::Paused => ScmState::Paused,
                    },
                    wait_hint: status.wait_hint,
                }))
            }
        }
    }

    fn create(
        &mut self,
        install: &WindowsScmInstall,
        spec: &ServiceSpec,
        _remaining: Duration,
    ) -> Result<(), ServiceError> {
        use windows_service::service::{ServiceAccess, ServiceType};
        use windows_service::service_manager::ServiceManagerAccess;
        let manager = Self::manager(ServiceManagerAccess::CREATE_SERVICE, "create service")?;
        let mut info = Self::info(install, spec);
        info.service_type = ServiceType::OWN_PROCESS;
        manager
            .create_service(
                &info,
                ServiceAccess::QUERY_CONFIG | ServiceAccess::QUERY_STATUS,
            )
            .map(drop)
            .map_err(|error| Self::map_error(error, "create service"))
    }

    fn refresh(
        &mut self,
        install: &WindowsScmInstall,
        spec: &ServiceSpec,
        _remaining: Duration,
    ) -> Result<(), ServiceError> {
        use windows_service::service::ServiceAccess;
        let (service, config) = match Self::open(
            spec.id(),
            ServiceAccess::QUERY_CONFIG | ServiceAccess::CHANGE_CONFIG,
        )? {
            Ok(service) => {
                let config = service
                    .query_config()
                    .map_err(|error| Self::map_error(error, "query config before refresh"))?;
                (service, config)
            }
            Err(1060 | 1072) => {
                return Err(ServiceError::conflict("service disappeared before refresh"))
            }
            Err(_) => unreachable!(),
        };
        let mut info = Self::info(install, spec);
        // Preserve fields outside this adapter's ownership. In particular, dependencies and
        // service type are queried and carried forward; account None means ChangeServiceConfig
        // leaves the existing account unchanged. Load-order group/tag are passed as null by the
        // wrapper, which likewise means no change.
        info.service_type = config.service_type;
        info.dependencies = config.dependencies;
        info.account_name = None;
        service
            .change_config(&info)
            .map_err(|error| Self::map_error(error, "refresh service config"))
    }

    fn start(&mut self, id: &ServiceId, _remaining: Duration) -> Result<(), ServiceError> {
        use windows_service::service::ServiceAccess;
        match Self::open(id, ServiceAccess::START)? {
            Ok(service) => service
                .start::<&std::ffi::OsStr>(&[])
                .map_err(|error| Self::map_error(error, "start service")),
            Err(1060 | 1072) => Err(ServiceError::conflict("service disappeared before start")),
            Err(_) => unreachable!(),
        }
    }

    fn stop(&mut self, id: &ServiceId, _remaining: Duration) -> Result<(), ServiceError> {
        use windows_service::service::ServiceAccess;
        match Self::open(id, ServiceAccess::STOP)? {
            Ok(service) => service
                .stop()
                .map(drop)
                .map_err(|error| Self::map_error(error, "stop service")),
            Err(1060 | 1072) => Err(ServiceError::conflict("service disappeared before stop")),
            Err(_) => unreachable!(),
        }
    }

    fn delete(&mut self, id: &ServiceId, _remaining: Duration) -> Result<(), ServiceError> {
        use windows_service::service::ServiceAccess;
        match Self::open(id, ServiceAccess::DELETE)? {
            Ok(service) => service
                .delete()
                .map_err(|error| Self::map_error(error, "delete service")),
            Err(1060 | 1072) => Err(ServiceError::conflict("service disappeared before delete")),
            Err(_) => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[derive(Debug)]
    struct FakeBackend {
        lookup: RefCell<VecDeque<ScmLookup>>,
        current: RefCell<ScmLookup>,
        calls: RefCell<Vec<&'static str>>,
    }

    impl FakeBackend {
        fn new(lookup: ScmLookup) -> Self {
            Self {
                lookup: RefCell::new(VecDeque::new()),
                current: RefCell::new(lookup),
                calls: RefCell::new(Vec::new()),
            }
        }

        fn scripted(initial: ScmLookup, states: impl IntoIterator<Item = ScmLookup>) -> Self {
            Self {
                lookup: RefCell::new(states.into_iter().collect()),
                current: RefCell::new(initial),
                calls: RefCell::new(Vec::new()),
            }
        }
    }

    impl ScmBackend for FakeBackend {
        fn query(&self, _id: &ServiceId, _remaining: Duration) -> Result<ScmLookup, ServiceError> {
            if let Some(next) = self.lookup.borrow_mut().pop_front() {
                *self.current.borrow_mut() = next;
            }
            Ok(self.current.borrow().clone())
        }
        fn create(
            &mut self,
            _: &WindowsScmInstall,
            _: &ServiceSpec,
            _: Duration,
        ) -> Result<(), ServiceError> {
            self.calls.borrow_mut().push("create");
            *self.current.borrow_mut() = present(
                r#""C:\Program Files\Eggup\egg.exe" --service"#,
                ScmState::Stopped,
            );
            Ok(())
        }
        fn refresh(
            &mut self,
            _: &WindowsScmInstall,
            _: &ServiceSpec,
            _: Duration,
        ) -> Result<(), ServiceError> {
            self.calls.borrow_mut().push("refresh");
            Ok(())
        }
        fn start(&mut self, _: &ServiceId, _: Duration) -> Result<(), ServiceError> {
            self.calls.borrow_mut().push("start");
            Ok(())
        }
        fn stop(&mut self, _: &ServiceId, _: Duration) -> Result<(), ServiceError> {
            self.calls.borrow_mut().push("stop");
            Ok(())
        }
        fn delete(&mut self, _: &ServiceId, _: Duration) -> Result<(), ServiceError> {
            self.calls.borrow_mut().push("delete");
            *self.current.borrow_mut() = ScmLookup::MarkedForDelete;
            Ok(())
        }
        fn sleep(&mut self, delay: Duration) {
            std::thread::sleep(delay);
        }
        fn test_calls(&self) -> Vec<&'static str> {
            self.calls.borrow().clone()
        }
    }

    fn present(command_line: &str, state: ScmState) -> ScmLookup {
        ScmLookup::Present(record(command_line, state))
    }

    fn record(command_line: &str, state: ScmState) -> ScmRecord {
        ScmRecord {
            command_line: Some(command_line.to_owned()),
            state,
            wait_hint: Duration::from_millis(1),
        }
    }

    fn install() -> WindowsScmInstall {
        WindowsScmInstall::new(
            "EggupTest",
            "Eggup Test",
            WindowsStartType::Manual,
            WindowsErrorControl::Normal,
            None,
        )
        .unwrap()
        .with_transition_timeout(Duration::from_secs(1))
        .unwrap()
    }

    #[test]
    fn install_descriptor_rejects_invalid_scm_names() {
        for name in ["", "group/service", "embedded\0nul"] {
            assert!(WindowsScmInstall::new(
                name,
                "Valid display",
                WindowsStartType::Manual,
                WindowsErrorControl::Normal,
                None,
            )
            .is_err());
        }
        assert!(WindowsScmInstall::new(
            "valid-name",
            "",
            WindowsStartType::Manual,
            WindowsErrorControl::Normal,
            None,
        )
        .is_err());
    }

    fn spec() -> ServiceSpec {
        ServiceSpec {
            id: ServiceId::new("EggupTest").unwrap(),
            executable: PathBuf::from(r"C:\Program Files\Eggup\egg.exe"),
            args: vec!["--service".into()],
            config: None,
        }
    }

    fn manager(backend: FakeBackend) -> WindowsScmManager {
        WindowsScmManager::with_backend(backend, install())
    }

    #[test]
    fn parser_handles_quoted_executable_arguments_backslashes_and_unicode() {
        let parsed = parse_service_command(
            r#""C:\Program Files\Eggup\工具.exe" --path "C:\a b\\" --name "é""#,
        )
        .unwrap();
        assert_eq!(parsed.0, PathBuf::from(r"C:\Program Files\Eggup\工具.exe"));
        assert_eq!(parsed.1, ["--path", r"C:\a b\", "--name", "é"]);
        assert!(parse_service_command(r#"C:\Program Files\Eggup\egg.exe --service"#).is_none());
        assert!(parse_service_command(r#""C:\Program Files\egg.exe --service"#).is_none());
    }

    #[test]
    fn parser_rejects_ambiguous_or_relative_command_lines() {
        for bad in ["", "egg.exe --service", r#""C:\egg.exe" "unterminated"#] {
            assert!(parse_service_command(bad).is_none(), "{bad:?}");
        }
    }

    #[test]
    fn config_identity_requires_exactly_one_config_argument() {
        let mut configured = spec();
        configured.config = Some(PathBuf::from(r"C:\ProgramData\Eggup\cfg.toml"));
        configured
            .args
            .push(r"C:\ProgramData\Eggup\cfg.toml".into());
        let first_record = record(
            r#""C:\Program Files\Eggup\egg.exe" --service C:\ProgramData\Eggup\cfg.toml"#,
            ScmState::Stopped,
        );
        assert_eq!(
            registration_ownership(&first_record, &configured),
            Ownership::Owned
        );
        configured
            .args
            .push(r"C:\ProgramData\Eggup\cfg.toml".into());
        let second_record = record(
            r#""C:\Program Files\Eggup\egg.exe" --service C:\ProgramData\Eggup\cfg.toml C:\ProgramData\Eggup\cfg.toml"#,
            ScmState::Stopped,
        );
        assert_eq!(
            registration_ownership(&second_record, &configured),
            Ownership::Unknown
        );
    }

    #[test]
    fn lifecycle_mapping_is_explicit_for_all_scm_states() {
        for (scm, neutral) in [
            (ScmState::Stopped, LifecycleState::Stopped),
            (ScmState::Running, LifecycleState::Running),
            (ScmState::StartPending, LifecycleState::Transitioning),
            (ScmState::StopPending, LifecycleState::Transitioning),
            (ScmState::ContinuePending, LifecycleState::Transitioning),
            (ScmState::PausePending, LifecycleState::Transitioning),
            (ScmState::Paused, LifecycleState::Unknown),
            (ScmState::Unknown, LifecycleState::Unknown),
        ] {
            assert_eq!(lifecycle_state(scm), neutral);
        }
    }

    #[test]
    fn inspect_maps_absent_and_requires_exact_command_identity() {
        let absent = manager(FakeBackend::new(ScmLookup::Absent))
            .inspect(&spec())
            .unwrap();
        assert_eq!(absent.ownership, Ownership::Absent);
        let owned = manager(FakeBackend::new(present(
            r#""C:\Program Files\Eggup\egg.exe" --service"#,
            ScmState::Running,
        )))
        .inspect(&spec())
        .unwrap();
        assert_eq!(owned.ownership, Ownership::Owned);
        let foreign = manager(FakeBackend::new(present(
            r#""C:\Other\egg.exe" --service"#,
            ScmState::Stopped,
        )))
        .inspect(&spec())
        .unwrap();
        assert_eq!(foreign.ownership, Ownership::Foreign);
        let argv_foreign = manager(FakeBackend::new(present(
            r#""C:\Program Files\Eggup\egg.exe" --other"#,
            ScmState::Stopped,
        )))
        .inspect(&spec())
        .unwrap();
        assert_eq!(argv_foreign.ownership, Ownership::Foreign);
        let unknown = manager(FakeBackend::new(present(
            "malformed cmd",
            ScmState::Stopped,
        )))
        .inspect(&spec())
        .unwrap();
        assert_eq!(unknown.ownership, Ownership::Unknown);
    }

    #[test]
    fn validate_spec_rejects_identity_mismatch_and_oversized_command() {
        let manager = manager(FakeBackend::new(ScmLookup::Absent));
        let mut wrong_id = spec();
        wrong_id.id = ServiceId::new("Different").unwrap();
        assert!(manager.validate_spec(&wrong_id).is_err());
        let mut oversized = spec();
        oversized.args = vec!["x".repeat(MAX_SCM_COMMAND_CHARS)];
        assert!(manager.validate_spec(&oversized).is_err());
    }

    #[test]
    fn install_creates_absent_and_denies_foreign() {
        let mut m = manager(FakeBackend::new(ScmLookup::Absent));
        assert!(m.install(&spec()).unwrap().completed);
        assert!(manager(FakeBackend::new(present(
            r#""C:\other.exe" --service"#,
            ScmState::Stopped
        )))
        .install(&spec())
        .is_err());
    }

    #[test]
    fn refresh_reinspects_owned_registration() {
        let owned = present(
            r#""C:\Program Files\Eggup\egg.exe" --service"#,
            ScmState::Stopped,
        );
        let mut refresh = manager(FakeBackend::scripted(owned.clone(), [owned.clone(), owned]));
        assert!(refresh.install(&spec()).unwrap().completed);
        assert!(refresh.backend.test_calls().contains(&"refresh"));

        let backend = FakeBackend::scripted(
            present(
                r#""C:\Program Files\Eggup\egg.exe" --service"#,
                ScmState::Stopped,
            ),
            [
                present(
                    r#""C:\Program Files\Eggup\egg.exe" --service"#,
                    ScmState::Stopped,
                ),
                present(r#""C:\other.exe" --service"#, ScmState::Stopped),
            ],
        );
        assert!(manager(backend).install(&spec()).is_err());
    }

    #[test]
    fn start_stop_restart_use_confirmed_states_and_one_deadline() {
        let running = present(
            r#""C:\Program Files\Eggup\egg.exe" --service"#,
            ScmState::Running,
        );
        assert!(
            manager(FakeBackend::new(running.clone()))
                .start(&spec(), Duration::from_secs(1))
                .unwrap()
                .completed
        );

        let stopped = present(
            r#""C:\Program Files\Eggup\egg.exe" --service"#,
            ScmState::Stopped,
        );
        let start_manager = manager(FakeBackend::scripted(
            stopped.clone(),
            [
                present(
                    r#""C:\Program Files\Eggup\egg.exe" --service"#,
                    ScmState::Stopped,
                ),
                present(
                    r#""C:\Program Files\Eggup\egg.exe" --service"#,
                    ScmState::StartPending,
                ),
                present(
                    r#""C:\Program Files\Eggup\egg.exe" --service"#,
                    ScmState::Running,
                ),
            ],
        ));
        let mut start_manager = start_manager;
        assert!(
            start_manager
                .start(&spec(), Duration::from_secs(1))
                .unwrap()
                .completed
        );

        let restart_manager = manager(FakeBackend::scripted(
            running.clone(),
            [
                present(
                    r#""C:\Program Files\Eggup\egg.exe" --service"#,
                    ScmState::Running,
                ),
                present(
                    r#""C:\Program Files\Eggup\egg.exe" --service"#,
                    ScmState::Stopped,
                ),
                present(
                    r#""C:\Program Files\Eggup\egg.exe" --service"#,
                    ScmState::Stopped,
                ),
                present(
                    r#""C:\Program Files\Eggup\egg.exe" --service"#,
                    ScmState::Stopped,
                ),
                present(
                    r#""C:\Program Files\Eggup\egg.exe" --service"#,
                    ScmState::Running,
                ),
            ],
        ));
        let mut restart_manager = restart_manager;
        assert!(
            restart_manager
                .restart(&spec(), Duration::from_secs(1))
                .unwrap()
                .completed
        );

        let stop_backend = FakeBackend::scripted(
            running,
            [
                present(
                    r#""C:\Program Files\Eggup\egg.exe" --service"#,
                    ScmState::Running,
                ),
                present(
                    r#""C:\Program Files\Eggup\egg.exe" --service"#,
                    ScmState::Stopped,
                ),
            ],
        );
        let mut stop_manager = manager(stop_backend);
        assert!(
            stop_manager
                .stop(&spec(), Duration::from_secs(1))
                .unwrap()
                .completed
        );
    }

    #[test]
    fn restart_does_not_start_if_stop_does_not_complete() {
        let backend = FakeBackend::scripted(
            present(
                r#""C:\Program Files\Eggup\egg.exe" --service"#,
                ScmState::Running,
            ),
            std::iter::once(present(
                r#""C:\Program Files\Eggup\egg.exe" --service"#,
                ScmState::Running,
            ))
            .chain(std::iter::repeat_n(
                present(
                    r#""C:\Program Files\Eggup\egg.exe" --service"#,
                    ScmState::StopPending,
                ),
                5,
            )),
        );
        let mut manager = manager(backend);
        let result = manager.restart(&spec(), Duration::from_millis(20));
        assert!(
            matches!(&result, Ok(result) if !result.completed)
                || matches!(&result, Err(ServiceError::Manager(detail)) if detail.contains("deadline"))
        );
        assert!(!manager.backend.test_calls().contains(&"start"));
    }

    #[test]
    fn uninstall_reports_marked_for_delete_until_absent() {
        let backend = FakeBackend::scripted(
            present(
                r#""C:\Program Files\Eggup\egg.exe" --service"#,
                ScmState::Stopped,
            ),
            [
                present(
                    r#""C:\Program Files\Eggup\egg.exe" --service"#,
                    ScmState::Stopped,
                ),
                present(
                    r#""C:\Program Files\Eggup\egg.exe" --service"#,
                    ScmState::Stopped,
                ),
                present(
                    r#""C:\Program Files\Eggup\egg.exe" --service"#,
                    ScmState::Stopped,
                ),
                ScmLookup::MarkedForDelete,
                ScmLookup::Absent,
            ],
        );
        let mut manager = manager(backend);
        assert!(manager.uninstall(&spec()).unwrap().completed);
    }

    #[test]
    fn uninstall_does_not_claim_completion_while_marked_for_delete() {
        let stopped = present(
            r#""C:\Program Files\Eggup\egg.exe" --service"#,
            ScmState::Stopped,
        );
        let backend = FakeBackend::scripted(stopped.clone(), [stopped.clone(), stopped]);
        let install = install()
            .with_transition_timeout(Duration::from_millis(20))
            .unwrap();
        let mut manager = WindowsScmManager::with_backend(backend, install);
        let result = manager.uninstall(&spec()).unwrap();
        assert!(!result.completed);
        assert!(manager.backend.test_calls().contains(&"delete"));
    }

    #[test]
    fn stop_of_already_stopped_registration_is_complete_without_a_control() {
        let mut manager = manager(FakeBackend::new(present(
            r#""C:\Program Files\Eggup\egg.exe" --service"#,
            ScmState::Stopped,
        )));
        assert!(
            manager
                .stop(&spec(), Duration::from_secs(1))
                .unwrap()
                .completed
        );
        assert!(!manager.backend.test_calls().contains(&"stop"));
    }
}
