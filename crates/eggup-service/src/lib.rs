#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Manager-neutral service registration, ownership, and lifecycle model."]
#![doc = ""]
#![doc = "Destructive manager operations are authorized only for `Owned`"]
#![doc = "registrations. `Foreign` and `Unknown` fail closed. Manager state and"]
#![doc = "application health are separate. No privilege escalation, no updater or"]
#![doc = "transport policy, and no real manager calls live in this crate."]

use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

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
        let mut d = detail.into();
        if d.len() > 512 {
            d.truncate(512);
        }
        Self::InvalidInput(d)
    }

    fn bounded(detail: impl Into<String>) -> String {
        let mut d = detail.into();
        if d.len() > 512 {
            d.truncate(512);
        }
        d
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
        self.stop(spec, timeout)?;
        self.start(spec, timeout)?;
        Ok(TransitionResult {
            operation: ServiceOperation::Restart,
            completed: true,
            detail: ServiceError::bounded("restarted"),
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
