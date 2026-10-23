//! Shared scaffolding for the M012 commit-engine fixtures.
//!
//! Everything here uses only the **pre-M012 public API surface**, so the
//! module compiles unchanged against the pre-M012 baseline
//! (`c4805411f74b85158e0ae5d0a219a32c4209d98e`).

#![allow(dead_code)]

use eggup_core::{
    AllValidators, ArtifactMember, ArtifactSet, CommitOwnership, InstallPlan, IntegrityRequirement,
    MemberId, OwnershipVerifier, PermissionsIntent, PostCommitFailurePolicy, ProductId, ReleaseId,
    ValidatedTransaction,
};
use eggup_service::{
    DirectObservation, DirectRuntimeControl, HealthState, LifecycleSnapshot, LifecycleState,
    LifecycleUpdateError, LifecycleUpdatePolicy, ManagerFailureKind, Ownership, RestoreIntent,
    ServiceError, ServiceId, ServiceManager, ServiceOperation, ServiceSpec, TransitionResult,
};
use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

static NEXT: AtomicU64 = AtomicU64::new(0);

/// How `inspect` behaves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectMode {
    /// Returns the scripted observation.
    Ok,
    /// Returns an error.
    Error,
    /// Panics.
    Panic,
}

/// How a `start`/`stop` call behaves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionMode {
    /// Reports `completed == true` and moves the unit to the target state.
    Complete,
    /// Reports `completed == false`.
    Incomplete,
    /// Returns an error.
    Error,
    /// Panics.
    Panic,
}

/// A scriptable manager double.
///
/// The same script drives both orchestrators, so any difference in their
/// observable outcome is a difference in the engine, not in the input.
#[derive(Debug)]
pub struct ScriptedManager {
    ownership: RefCell<Ownership>,
    state: RefCell<LifecycleState>,
    inspect_mode: InspectMode,
    stop_mode: TransitionMode,
    start_mode: TransitionMode,
    /// When set, the state reported by `inspect` after a transition attempt.
    /// The transition itself never moves the unit.
    post_transition_state: Option<LifecycleState>,
    /// The 1-based `inspect` call at which ownership is lost.
    ///
    /// Both orchestrators perform exactly one `inspect` before the artifact
    /// mutation and exactly one inside the post-commit transition, so the
    /// ownership is lost at the same point of the sequence for both.
    lose_ownership_after_inspect: Option<usize>,
    inspect_count: Cell<usize>,
    transitions: Cell<usize>,
}

impl ScriptedManager {
    /// Creates a manager double with the given script.
    pub fn new(
        ownership: Ownership,
        state: LifecycleState,
        inspect_mode: InspectMode,
        stop_mode: TransitionMode,
        start_mode: TransitionMode,
    ) -> Self {
        Self {
            ownership: RefCell::new(ownership),
            state: RefCell::new(state),
            inspect_mode,
            stop_mode,
            start_mode,
            post_transition_state: None,
            lose_ownership_after_inspect: None,
            inspect_count: Cell::new(0),
            transitions: Cell::new(0),
        }
    }

    /// Pins the state reported after a transition attempt.
    pub fn with_post_transition_state(mut self, state: LifecycleState) -> Self {
        self.post_transition_state = Some(state);
        self
    }

    /// Loses ownership at the given 1-based `inspect` call.
    pub fn losing_ownership_after_inspect(mut self, call: usize) -> Self {
        self.lose_ownership_after_inspect = Some(call);
        self
    }

    fn transition(
        &self,
        mode: TransitionMode,
        target: LifecycleState,
    ) -> Result<TransitionResult, ServiceError> {
        match mode {
            TransitionMode::Panic => panic!("scripted manager panicked"),
            TransitionMode::Error => Err(ServiceError::manager(
                ManagerFailureKind::Protocol,
                "scripted manager failure",
            )),
            TransitionMode::Incomplete => Ok(TransitionResult {
                operation: ServiceOperation::Stop,
                completed: false,
                detail: String::new(),
            }),
            TransitionMode::Complete => {
                self.transitions.set(self.transitions.get() + 1);
                if self.post_transition_state.is_none() {
                    *self.state.borrow_mut() = target;
                }
                Ok(TransitionResult {
                    operation: ServiceOperation::Stop,
                    completed: true,
                    detail: String::new(),
                })
            }
        }
    }
}

impl ServiceManager for ScriptedManager {
    fn inspect(&self, spec: &ServiceSpec) -> Result<LifecycleSnapshot, ServiceError> {
        match self.inspect_mode {
            InspectMode::Error => {
                return Err(ServiceError::manager(
                    ManagerFailureKind::Protocol,
                    "scripted inspect failure",
                ))
            }
            InspectMode::Panic => panic!("scripted manager panicked while inspecting"),
            InspectMode::Ok => {}
        }
        let call = self.inspect_count.get() + 1;
        self.inspect_count.set(call);
        if self.lose_ownership_after_inspect == Some(call) {
            *self.ownership.borrow_mut() = Ownership::Foreign;
        }
        let state = match self.post_transition_state {
            Some(post) if self.transitions.get() > 0 => post,
            _ => *self.state.borrow(),
        };
        Ok(LifecycleSnapshot {
            id: spec.id().clone(),
            ownership: *self.ownership.borrow(),
            state,
            health: HealthState::Unknown,
            was_registered: true,
            was_running: state == LifecycleState::Running,
        })
    }

    fn install(&mut self, _spec: &ServiceSpec) -> Result<TransitionResult, ServiceError> {
        Err(ServiceError::manager(
            ManagerFailureKind::Protocol,
            "install is not exercised",
        ))
    }

    fn start(
        &mut self,
        _spec: &ServiceSpec,
        _timeout: Duration,
    ) -> Result<TransitionResult, ServiceError> {
        self.transition(self.start_mode, LifecycleState::Running)
    }

    fn stop(
        &mut self,
        _spec: &ServiceSpec,
        _timeout: Duration,
    ) -> Result<TransitionResult, ServiceError> {
        self.transition(self.stop_mode, LifecycleState::Stopped)
    }

    fn restart(
        &mut self,
        _spec: &ServiceSpec,
        _timeout: Duration,
    ) -> Result<TransitionResult, ServiceError> {
        Err(ServiceError::manager(
            ManagerFailureKind::Protocol,
            "restart is not exercised",
        ))
    }

    fn uninstall(&mut self, _spec: &ServiceSpec) -> Result<TransitionResult, ServiceError> {
        Err(ServiceError::manager(
            ManagerFailureKind::Protocol,
            "uninstall is not exercised",
        ))
    }
}

/// A direct-runtime seam that is never exercised by these fixtures.
#[derive(Debug)]
pub struct UnusedDirect;

impl DirectRuntimeControl for UnusedDirect {
    fn inspect(&self) -> Result<DirectObservation, ServiceError> {
        Err(ServiceError::manager(
            ManagerFailureKind::Protocol,
            "direct control is not exercised",
        ))
    }

    fn stop(&mut self, _timeout: Duration) -> Result<TransitionResult, ServiceError> {
        Err(ServiceError::manager(
            ManagerFailureKind::Protocol,
            "direct control is not exercised",
        ))
    }

    fn start(&mut self, _timeout: Duration) -> Result<TransitionResult, ServiceError> {
        Err(ServiceError::manager(
            ManagerFailureKind::Protocol,
            "direct control is not exercised",
        ))
    }
}

/// The observable outcome class of one orchestrator run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutcomeClass {
    /// The artifact transaction committed and the post-commit work succeeded.
    CommittedClean,
    /// The artifact transaction committed, but the post-commit lifecycle work
    /// failed.
    CommittedWithPostCommitFailure,
    /// The artifact transaction rolled back.
    RolledBack,
    /// Core retained recovery evidence.
    RecoveryRequired,
    /// Preflight rejected the request before any mutation.
    PreflightError,
    /// Pre-commit quiescence failed.
    QuiesceError,
    /// Core returned before a terminal receipt.
    CoreBeforeReceiptError,
    /// Any other error variant (the error type is `#[non_exhaustive]`).
    OtherError,
}

/// Classifies one orchestrator run into its observable outcome class.
pub fn classify(
    result: Result<eggup_service::LifecycleUpdateReceipt, LifecycleUpdateError>,
) -> OutcomeClass {
    use eggup_core::TransactionDisposition;
    match result {
        Err(LifecycleUpdateError::Preflight { .. }) => OutcomeClass::PreflightError,
        Err(LifecycleUpdateError::Quiesce { .. }) => OutcomeClass::QuiesceError,
        Err(LifecycleUpdateError::CoreBeforeReceipt { .. }) => OutcomeClass::CoreBeforeReceiptError,
        Err(LifecycleUpdateError::InvalidPolicy { .. }) => OutcomeClass::PreflightError,
        Err(_) => OutcomeClass::OtherError,
        Ok(receipt) => match receipt.transaction.disposition() {
            TransactionDisposition::Committed => {
                if receipt.post_commit_failure.is_some() {
                    OutcomeClass::CommittedWithPostCommitFailure
                } else {
                    OutcomeClass::CommittedClean
                }
            }
            TransactionDisposition::RolledBack => OutcomeClass::RolledBack,
            TransactionDisposition::RecoveryRequired => OutcomeClass::RecoveryRequired,
        },
    }
}

/// Removes its directory when dropped.
pub struct TempfileGuard {
    path: PathBuf,
}

impl Drop for TempfileGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

/// Builds a validated transaction with one member, in a fresh install root.
pub fn validated_transaction(tag: &str) -> (TempfileGuard, PathBuf, ValidatedTransaction) {
    let base = std::env::temp_dir().join(format!(
        "eggup-m012-{}-{}-{}",
        tag,
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let source_root = base.join("input");
    let install_root = base.join("install");
    std::fs::create_dir_all(&source_root).unwrap();
    std::fs::create_dir_all(&install_root).unwrap();
    let main = source_root.join("main");
    std::fs::write(&main, b"new-main").unwrap();
    std::fs::write(install_root.join("service.bin"), b"old-main").unwrap();
    let members = ArtifactSet::new(vec![ArtifactMember::new(
        MemberId::new("main").unwrap(),
        &main,
        "service.bin",
    )
    .unwrap()
    .with_permissions(PermissionsIntent::Executable)
    .with_integrity(IntegrityRequirement::Sha256(
        eggup_core::hash_file(&main).unwrap(),
    ))])
    .unwrap();
    let validated = InstallPlan::new(
        ProductId::new("eggup-m012-test").unwrap(),
        ReleaseId::new("r2").unwrap(),
        &install_root,
        members,
    )
    .unwrap()
    .prepare()
    .unwrap()
    .verify_integrity()
    .unwrap()
    .validate(&AllValidators::new())
    .unwrap();
    (
        TempfileGuard { path: base.clone() },
        install_root,
        validated,
    )
}

/// The service spec used by these fixtures.
pub fn service_spec(install_root: &Path) -> ServiceSpec {
    ServiceSpec::new(
        ServiceId::new("eggup-m012-test").unwrap(),
        install_root.join("service.bin"),
        vec!["--worker".into()],
        None,
    )
    .unwrap()
}

/// A caller-proven ownership check that always reports `Owned`.
pub fn ownership_proof() -> CommitOwnership<'static> {
    struct Fixed(eggup_core::Ownership);
    impl OwnershipVerifier for Fixed {
        fn verify(
            &self,
            _member: &eggup_core::MemberId,
            _destination: &Path,
        ) -> eggup_core::Ownership {
            self.0
        }
    }
    static VERIFIER: Fixed = Fixed(eggup_core::Ownership::Owned);
    CommitOwnership::new(&VERIFIER, eggup_core::AbsentPolicy::AllowCreate)
}

/// The lifecycle-update policy used by these fixtures.
pub fn policy(restore: RestoreIntent) -> LifecycleUpdatePolicy {
    LifecycleUpdatePolicy {
        restore,
        post_commit_failure: PostCommitFailurePolicy::KeepInstalled,
        quiesce_timeout: Duration::from_secs(2),
        post_commit_timeout: Duration::from_secs(2),
        rollback_restore_timeout: Duration::from_secs(2),
    }
}
