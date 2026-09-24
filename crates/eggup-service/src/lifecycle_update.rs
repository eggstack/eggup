use crate::{
    LifecycleSnapshot, LifecycleState, Ownership, RestoreIntent, ServiceManager, ServiceSpec,
};
use eggup_core::{
    CommitOwnership, Error as CoreError, PostCommitFailurePolicy, TransactionDisposition,
    TransactionReceipt, ValidatedTransaction,
};
use std::{
    fmt,
    panic::{catch_unwind, AssertUnwindSafe},
    time::{Duration, Instant},
};

/// Phase that produced lifecycle-specific update evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum LifecycleUpdatePhase {
    /// Initial state observation.
    Inspect,
    /// Stop the prior generation before artifact mutation.
    Quiesce,
    /// Core transaction returned before a terminal receipt.
    Commit,
    /// Restore the successful new generation's desired state.
    RestoreNew,
    /// Caller-owned post-install validation.
    PostInstallCheck,
    /// Stop the new generation before Core rollback.
    QuiesceForRollback,
    /// Restore the prior generation's lifecycle state after rollback.
    RestoreOld,
    /// Optional final read-only observation.
    FinalInspect,
}

/// Bounded service-specific failure evidence, separate from Core artifact evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifecycleFailure {
    /// Orchestration phase.
    pub phase: LifecycleUpdatePhase,
    /// Service identifier.
    pub service_id: String,
    /// Bounded detail. Implementations must not include secrets.
    pub detail: String,
    /// Ownership observed when applicable.
    pub ownership: Option<Ownership>,
    /// State observed when applicable.
    pub state: Option<LifecycleState>,
}

impl LifecycleFailure {
    fn new(
        spec: &ServiceSpec,
        phase: LifecycleUpdatePhase,
        detail: impl fmt::Display,
        snapshot: Option<&LifecycleSnapshot>,
    ) -> Self {
        Self {
            phase,
            service_id: spec.id().as_str().to_owned(),
            detail: bounded_detail(detail.to_string()),
            ownership: snapshot.map(|s| s.ownership),
            state: snapshot.map(|s| s.state),
        }
    }
}

/// Terminal service restoration status for a composite update.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleRestorationStatus {
    /// Service lifecycle was restored and confirmed for the terminal artifact generation.
    Restored,
    /// A lifecycle transition or confirmation failed.
    Failed,
    /// Artifact state is uncertain, so automatic restoration was suppressed.
    NotAttemptedRecoveryRequired,
}

/// Mechanism-level choices and bounds for one service-aware transaction.
#[derive(Debug, Clone, Copy)]
pub struct LifecycleUpdatePolicy {
    /// Desired service state after a successful artifact update.
    pub restore: RestoreIntent,
    /// Core behavior if post-commit lifecycle/check work fails.
    pub post_commit_failure: PostCommitFailurePolicy,
    /// Maximum wait for stopping the prior service generation.
    pub quiesce_timeout: Duration,
    /// Shared budget for post-commit transition and caller check work.
    pub post_commit_timeout: Duration,
    /// Maximum wait to restore prior lifecycle state after rollback or Core pre-receipt error.
    pub rollback_restore_timeout: Duration,
}

impl LifecycleUpdatePolicy {
    /// Rejects zero or overlong timeouts before any manager mutation.
    pub fn validate(&self) -> Result<(), LifecycleUpdateError> {
        for (field, value) in [
            ("quiesce_timeout", self.quiesce_timeout),
            ("post_commit_timeout", self.post_commit_timeout),
            ("rollback_restore_timeout", self.rollback_restore_timeout),
        ] {
            if value.is_zero() || value > crate::MAX_TRANSITION_TIMEOUT {
                return Err(LifecycleUpdateError::InvalidPolicy { field });
            }
        }
        Ok(())
    }
}

/// Bounded caller-owned post-install check failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostInstallCheckError(String);

impl PostInstallCheckError {
    /// Creates bounded, non-secret failure evidence.
    pub fn new(detail: impl Into<String>) -> Self {
        Self(bounded_detail(detail.into()))
    }
}

impl fmt::Display for PostInstallCheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for PostInstallCheckError {}

/// Caller-controlled post-install check. Implementations must honor `remaining`.
pub trait PostInstallCheck: fmt::Debug {
    /// Check the installed generation within the supplied remaining-time budget.
    ///
    /// Synchronous user code cannot be forcibly cancelled safely. This method
    /// must return within `remaining` and must not include secrets in its error.
    fn check(
        &self,
        spec: &ServiceSpec,
        snapshot: &LifecycleSnapshot,
        remaining: Duration,
    ) -> Result<(), PostInstallCheckError>;
}

/// No-op check for consumers that require lifecycle restoration only.
#[derive(Debug, Default)]
pub struct NoPostInstallCheck;

impl PostInstallCheck for NoPostInstallCheck {
    fn check(
        &self,
        _spec: &ServiceSpec,
        _snapshot: &LifecycleSnapshot,
        _remaining: Duration,
    ) -> Result<(), PostInstallCheckError> {
        Ok(())
    }
}

/// Composite result that retains the exact Core transaction evidence.
#[derive(Debug)]
pub struct LifecycleUpdateReceipt {
    /// Immutable pre-update observation.
    pub before: LifecycleSnapshot,
    /// Final read-only observation when available.
    pub final_snapshot: Option<LifecycleSnapshot>,
    /// Exact Core artifact transaction receipt.
    pub transaction: TransactionReceipt,
    /// Whether a previously running service was quiesced before Core mutation.
    pub pre_commit_quiesced: bool,
    /// Lifecycle restoration state at the terminal artifact disposition.
    pub restoration: LifecycleRestorationStatus,
    /// Post-commit lifecycle or caller-check failure, when one occurred.
    pub post_commit_failure: Option<LifecycleFailure>,
    /// Failure stopping a possibly running new generation before Core rollback.
    pub rollback_quiesce_failure: Option<LifecycleFailure>,
    /// Failure restoring the old service state after successful artifact rollback.
    pub rollback_restore_failure: Option<LifecycleFailure>,
    /// Failure to obtain the optional final read-only observation.
    pub observation_failure: Option<LifecycleFailure>,
}

impl LifecycleUpdateReceipt {
    /// Core's authoritative artifact disposition.
    pub fn artifact_disposition(&self) -> TransactionDisposition {
        self.transaction.disposition()
    }

    /// Whether Core retained recovery evidence requiring manual artifact recovery.
    pub fn manual_artifact_recovery_required(&self) -> bool {
        self.transaction.disposition() == TransactionDisposition::RecoveryRequired
    }

    /// Whether the new artifact generation remains installed.
    pub fn new_generation_retained(&self) -> bool {
        self.transaction.disposition() == TransactionDisposition::Committed
    }

    /// Whether service lifecycle was restored and confirmed for the terminal generation.
    pub fn service_state_restored(&self) -> bool {
        self.restoration == LifecycleRestorationStatus::Restored
    }
}

/// Failure before a composite terminal transaction receipt could be returned.
#[derive(Debug)]
#[non_exhaustive]
pub enum LifecycleUpdateError {
    /// One timeout is zero or exceeds the service transition limit.
    InvalidPolicy {
        /// Name of the invalid field.
        field: &'static str,
    },
    /// Preflight failed before service or artifact mutation.
    Preflight {
        /// Structured failure evidence.
        failure: LifecycleFailure,
    },
    /// Pre-commit service quiescence failed; attempted restoration is retained separately.
    Quiesce {
        /// Original quiescence failure.
        failure: LifecycleFailure,
        /// Failure restoring the pre-update state after the stop attempt, if any.
        restore_failure: Option<LifecycleFailure>,
    },
    /// Core returned an error before a terminal receipt; service restoration was attempted.
    CoreBeforeReceipt {
        /// Exact Core error.
        core: CoreError,
        /// Failure restoring pre-update lifecycle state, if any.
        restore_failure: Option<LifecycleFailure>,
    },
}

impl fmt::Display for LifecycleUpdateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPolicy { field } => write!(f, "invalid lifecycle update timeout: {field}"),
            Self::Preflight { failure } => write!(f, "lifecycle preflight failed: {failure:?}"),
            Self::Quiesce { failure, .. } => write!(f, "service quiescence failed: {failure:?}"),
            Self::CoreBeforeReceipt { core, .. } => {
                write!(f, "Core returned before a transaction receipt: {core}")
            }
        }
    }
}
impl std::error::Error for LifecycleUpdateError {}

/// Quiesce an owned service, compose Core M007, and restore lifecycle state based on the artifact result.
///
/// The existing registration must be `Owned` and either `Running` or `Stopped`.
/// `Preserve` keeps a stopped service stopped after success. A rolled-back
/// transaction restores the pre-update lifecycle state regardless of `restore`.
/// `RecoveryRequired` suppresses automatic service starts. No registration
/// install/refresh is performed. The service adapter uses Core's transaction
/// rollback rather than maintaining artifact backups of its own.
pub fn commit_with_lifecycle<M: ServiceManager, C: PostInstallCheck>(
    manager: &mut M,
    spec: &ServiceSpec,
    transaction: ValidatedTransaction,
    ownership: CommitOwnership<'_>,
    policy: LifecycleUpdatePolicy,
    check: &C,
) -> Result<LifecycleUpdateReceipt, LifecycleUpdateError> {
    policy.validate()?;
    let before = safe_inspect(manager, spec, LifecycleUpdatePhase::Inspect)
        .map_err(|failure| LifecycleUpdateError::Preflight { failure })?;
    if before.ownership != Ownership::Owned
        || !matches!(
            before.state,
            LifecycleState::Running | LifecycleState::Stopped
        )
    {
        return Err(LifecycleUpdateError::Preflight {
            failure: LifecycleFailure::new(
                spec,
                LifecycleUpdatePhase::Inspect,
                "service must be Owned and Running or Stopped",
                Some(&before),
            ),
        });
    }

    let mut pre_commit_quiesced = false;
    if before.state == LifecycleState::Running {
        let stop_result = catch_unwind(AssertUnwindSafe(|| {
            manager.stop(spec, policy.quiesce_timeout)
        }));
        let stop_failure = match stop_result {
            Ok(Err(e)) => Some(LifecycleFailure::new(
                spec,
                LifecycleUpdatePhase::Quiesce,
                e,
                Some(&before),
            )),
            Err(_) => Some(LifecycleFailure::new(
                spec,
                LifecycleUpdatePhase::Quiesce,
                "service manager panicked while stopping",
                Some(&before),
            )),
            Ok(Ok(result)) if !result.completed => Some(LifecycleFailure::new(
                spec,
                LifecycleUpdatePhase::Quiesce,
                "stop did not complete",
                Some(&before),
            )),
            Ok(Ok(_)) => match safe_inspect(manager, spec, LifecycleUpdatePhase::Quiesce) {
                Ok(after)
                    if after.ownership == Ownership::Owned
                        && after.state == LifecycleState::Stopped =>
                {
                    pre_commit_quiesced = true;
                    None
                }
                Ok(after) => Some(LifecycleFailure::new(
                    spec,
                    LifecycleUpdatePhase::Quiesce,
                    "ownership/state changed after stop",
                    Some(&after),
                )),
                Err(failure) => Some(failure),
            },
        };
        if let Some(failure) = stop_failure {
            let restore_failure = safe_restore_state(
                manager,
                spec,
                LifecycleState::Running,
                policy.rollback_restore_timeout,
                LifecycleUpdatePhase::Quiesce,
            )
            .err();
            return Err(LifecycleUpdateError::Quiesce {
                failure,
                restore_failure,
            });
        }
    }

    let mut post_commit_failure = None;
    let mut rollback_quiesce_failure = None;
    let core_result =
        transaction.commit_with_post_commit(ownership, policy.post_commit_failure, || {
            let deadline = Instant::now() + policy.post_commit_timeout;
            let work = catch_unwind(AssertUnwindSafe(|| {
                restore_success_state(manager, spec, &before, policy.restore, check, deadline)
            }));
            let failure = match work {
                Ok(Ok(())) => return Ok(()),
                Ok(Err(failure)) => failure,
                Err(_) => LifecycleFailure::new(
                    spec,
                    LifecycleUpdatePhase::PostInstallCheck,
                    "post-commit service work panicked",
                    None,
                ),
            };
            post_commit_failure = Some(failure.clone());

            if policy.post_commit_failure == PostCommitFailurePolicy::RollBack {
                rollback_quiesce_failure = match catch_unwind(AssertUnwindSafe(|| {
                    quiesce_for_rollback(manager, spec, deadline)
                })) {
                    Ok(failure) => failure,
                    Err(_) => Some(LifecycleFailure::new(
                        spec,
                        LifecycleUpdatePhase::QuiesceForRollback,
                        "service manager panicked while quiescing before artifact rollback",
                        None,
                    )),
                };
            }
            Err(PostInstallCheckError::new(failure.detail))
        });

    let transaction = match core_result {
        Ok(receipt) => receipt,
        Err(core) => {
            let restore_failure = if pre_commit_quiesced {
                safe_restore_state(
                    manager,
                    spec,
                    before.state,
                    policy.rollback_restore_timeout,
                    LifecycleUpdatePhase::RestoreOld,
                )
                .err()
            } else {
                None
            };
            return Err(LifecycleUpdateError::CoreBeforeReceipt {
                core,
                restore_failure,
            });
        }
    };

    let mut rollback_restore_failure = None;
    let mut restoration = LifecycleRestorationStatus::Restored;
    match transaction.disposition() {
        TransactionDisposition::Committed => {
            if post_commit_failure
                .as_ref()
                .is_some_and(|f| f.phase == LifecycleUpdatePhase::RestoreNew)
            {
                restoration = LifecycleRestorationStatus::Failed;
            }
        }
        TransactionDisposition::RolledBack => {
            if let Err(failure) = safe_restore_state(
                manager,
                spec,
                before.state,
                policy.rollback_restore_timeout,
                LifecycleUpdatePhase::RestoreOld,
            ) {
                rollback_restore_failure = Some(failure);
                restoration = LifecycleRestorationStatus::Failed;
            }
        }
        TransactionDisposition::RecoveryRequired => {
            restoration = LifecycleRestorationStatus::NotAttemptedRecoveryRequired;
        }
    }

    let final_snapshot = match safe_inspect(manager, spec, LifecycleUpdatePhase::FinalInspect) {
        Ok(snapshot) => Some(snapshot),
        Err(observation_failure) => {
            // A final read-only observation is nonessential and never changes Core disposition.
            return Ok(LifecycleUpdateReceipt {
                before,
                final_snapshot: None,
                transaction,
                pre_commit_quiesced,
                restoration,
                post_commit_failure,
                rollback_quiesce_failure,
                rollback_restore_failure,
                observation_failure: Some(observation_failure),
            });
        }
    };

    Ok(LifecycleUpdateReceipt {
        before,
        final_snapshot,
        transaction,
        pre_commit_quiesced,
        restoration,
        post_commit_failure,
        rollback_quiesce_failure,
        rollback_restore_failure,
        observation_failure: None,
    })
}

fn restore_success_state<M: ServiceManager, C: PostInstallCheck>(
    manager: &mut M,
    spec: &ServiceSpec,
    before: &LifecycleSnapshot,
    intent: RestoreIntent,
    check: &C,
    deadline: Instant,
) -> Result<(), LifecycleFailure> {
    let wanted = match intent {
        RestoreIntent::Preserve => before.state,
        RestoreIntent::EnsureRunning => LifecycleState::Running,
        RestoreIntent::EnsureStopped => LifecycleState::Stopped,
    };
    transition_to(
        manager,
        spec,
        wanted,
        remaining(deadline),
        LifecycleUpdatePhase::RestoreNew,
    )?;
    let snapshot = safe_inspect(manager, spec, LifecycleUpdatePhase::RestoreNew)?;
    validate_observed(spec, &snapshot, wanted, LifecycleUpdatePhase::RestoreNew)?;
    let budget = remaining(deadline);
    if budget.is_zero() {
        return Err(LifecycleFailure::new(
            spec,
            LifecycleUpdatePhase::PostInstallCheck,
            "post-commit deadline exhausted before post-install check",
            Some(&snapshot),
        ));
    }
    check.check(spec, &snapshot, budget).map_err(|e| {
        LifecycleFailure::new(
            spec,
            LifecycleUpdatePhase::PostInstallCheck,
            e,
            Some(&snapshot),
        )
    })
}

fn transition_to<M: ServiceManager>(
    manager: &mut M,
    spec: &ServiceSpec,
    wanted: LifecycleState,
    timeout: Duration,
    phase: LifecycleUpdatePhase,
) -> Result<(), LifecycleFailure> {
    if timeout.is_zero() {
        return Err(LifecycleFailure::new(
            spec,
            phase,
            "lifecycle operation deadline exhausted",
            None,
        ));
    }
    let current = manager
        .inspect(spec)
        .map_err(|e| LifecycleFailure::new(spec, phase, e, None))?;
    if current.ownership != Ownership::Owned {
        return Err(LifecycleFailure::new(
            spec,
            phase,
            "service ownership is no longer Owned",
            Some(&current),
        ));
    }
    let result = match (wanted, current.state) {
        (LifecycleState::Running, LifecycleState::Running)
        | (LifecycleState::Stopped, LifecycleState::Stopped) => return Ok(()),
        (LifecycleState::Running, LifecycleState::Stopped) => manager.start(spec, timeout),
        (LifecycleState::Stopped, LifecycleState::Running) => manager.stop(spec, timeout),
        (_, LifecycleState::Transitioning | LifecycleState::Unknown) => {
            return Err(LifecycleFailure::new(
                spec,
                phase,
                "service state is Unknown or Transitioning",
                Some(&current),
            ))
        }
        _ => unreachable!("only Running and Stopped are valid desired states"),
    }
    .map_err(|e| LifecycleFailure::new(spec, phase, e, Some(&current)))?;
    if !result.completed {
        return Err(LifecycleFailure::new(
            spec,
            phase,
            "service transition did not complete",
            Some(&current),
        ));
    }
    let after = manager
        .inspect(spec)
        .map_err(|e| LifecycleFailure::new(spec, phase, e, None))?;
    validate_observed(spec, &after, wanted, phase)
}

fn validate_observed(
    spec: &ServiceSpec,
    snapshot: &LifecycleSnapshot,
    wanted: LifecycleState,
    phase: LifecycleUpdatePhase,
) -> Result<(), LifecycleFailure> {
    if snapshot.ownership != Ownership::Owned || snapshot.state != wanted {
        return Err(LifecycleFailure::new(
            spec,
            phase,
            "service ownership/state does not match the requested state",
            Some(snapshot),
        ));
    }
    Ok(())
}

fn restore_state<M: ServiceManager>(
    manager: &mut M,
    spec: &ServiceSpec,
    wanted: LifecycleState,
    timeout: Duration,
    phase: LifecycleUpdatePhase,
) -> Result<(), LifecycleFailure> {
    transition_to(manager, spec, wanted, timeout, phase)
}

fn safe_inspect<M: ServiceManager>(
    manager: &mut M,
    spec: &ServiceSpec,
    phase: LifecycleUpdatePhase,
) -> Result<LifecycleSnapshot, LifecycleFailure> {
    match catch_unwind(AssertUnwindSafe(|| manager.inspect(spec))) {
        Ok(Ok(snapshot)) => Ok(snapshot),
        Ok(Err(error)) => Err(LifecycleFailure::new(spec, phase, error, None)),
        Err(_) => Err(LifecycleFailure::new(
            spec,
            phase,
            "service manager panicked while inspecting",
            None,
        )),
    }
}

fn safe_restore_state<M: ServiceManager>(
    manager: &mut M,
    spec: &ServiceSpec,
    wanted: LifecycleState,
    timeout: Duration,
    phase: LifecycleUpdatePhase,
) -> Result<(), LifecycleFailure> {
    match catch_unwind(AssertUnwindSafe(|| {
        restore_state(manager, spec, wanted, timeout, phase)
    })) {
        Ok(result) => result,
        Err(_) => Err(LifecycleFailure::new(
            spec,
            phase,
            "service manager panicked while restoring state",
            None,
        )),
    }
}

fn quiesce_for_rollback<M: ServiceManager>(
    manager: &mut M,
    spec: &ServiceSpec,
    deadline: Instant,
) -> Option<LifecycleFailure> {
    let budget = remaining(deadline);
    if budget.is_zero() {
        return Some(LifecycleFailure::new(
            spec,
            LifecycleUpdatePhase::QuiesceForRollback,
            "post-commit deadline exhausted before rollback quiescence",
            None,
        ));
    }
    let observed = manager.inspect(spec).ok();
    if observed
        .as_ref()
        .is_some_and(|s| s.ownership == Ownership::Owned && s.state == LifecycleState::Stopped)
    {
        return None;
    }
    let stop = manager.stop(spec, budget);
    match stop {
        Err(error) => Some(LifecycleFailure::new(
            spec,
            LifecycleUpdatePhase::QuiesceForRollback,
            error,
            observed.as_ref(),
        )),
        Ok(result) if !result.completed => Some(LifecycleFailure::new(
            spec,
            LifecycleUpdatePhase::QuiesceForRollback,
            "stop did not complete before artifact rollback",
            observed.as_ref(),
        )),
        Ok(_) => match manager.inspect(spec) {
            Ok(after)
                if after.ownership == Ownership::Owned
                    && after.state == LifecycleState::Stopped =>
            {
                None
            }
            Ok(after) => Some(LifecycleFailure::new(
                spec,
                LifecycleUpdatePhase::QuiesceForRollback,
                "stopped state could not be confirmed before artifact rollback",
                Some(&after),
            )),
            Err(error) => Some(LifecycleFailure::new(
                spec,
                LifecycleUpdatePhase::QuiesceForRollback,
                error,
                None,
            )),
        },
    }
}

fn remaining(deadline: Instant) -> Duration {
    deadline.saturating_duration_since(Instant::now())
}

fn bounded_detail(mut detail: String) -> String {
    detail = detail.chars().filter(|c| !c.is_control()).collect();
    if detail.len() > 512 {
        let mut end = 512;
        while !detail.is_char_boundary(end) {
            end -= 1;
        }
        detail.truncate(end);
    }
    detail
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ServiceError, ServiceId, ServiceOperation, TestDoubleManager, TransitionResult};
    use eggup_core::{
        AbsentOnlyVerifier, AbsentPolicy, AllValidators, ArtifactMember, ArtifactSet,
        CommitOwnership as CoreCommitOwnership, InstallPlan, MemberId, Ownership as CoreOwnership,
        PermissionsIntent, ProductId, ReleaseId,
    };
    use std::{
        cell::{Cell, RefCell},
        fs,
        path::{Path, PathBuf},
        rc::Rc,
        sync::atomic::{AtomicU64, Ordering},
        thread,
    };

    static NEXT: AtomicU64 = AtomicU64::new(0);

    struct FixedVerifier(CoreOwnership);
    impl eggup_core::OwnershipVerifier for FixedVerifier {
        fn verify(&self, _member: &MemberId, _destination: &Path) -> CoreOwnership {
            self.0
        }
    }

    #[derive(Default)]
    struct ScriptedManager {
        inner: TestDoubleManager,
        events: Rc<RefCell<Vec<&'static str>>>,
        fail_stop: bool,
        incomplete_stop: bool,
        fail_start_calls: Vec<usize>,
        start_calls: usize,
        stop_calls: usize,
        drift_to_foreign_after_stop: bool,
        fail_final_inspect: bool,
        check_finished: Option<Rc<Cell<bool>>>,
        final_inspect_failed: Cell<bool>,
        start_delay: Duration,
    }

    impl ScriptedManager {
        fn with_service(spec: ServiceSpec, state: LifecycleState) -> Self {
            let mut inner = TestDoubleManager::new();
            inner.put(spec, state);
            Self {
                inner,
                ..Self::default()
            }
        }
    }

    impl ServiceManager for ScriptedManager {
        fn inspect(&self, spec: &ServiceSpec) -> Result<LifecycleSnapshot, ServiceError> {
            self.events.borrow_mut().push("inspect");
            if self.fail_final_inspect
                && self.check_finished.as_ref().is_some_and(|f| f.get())
                && !self.final_inspect_failed.get()
            {
                // The final observation is the first inspect after the check returns.
                self.final_inspect_failed.set(true);
                return Err(ServiceError::manager("injected final inspection failure"));
            }
            self.inner.inspect(spec)
        }

        fn install(&mut self, spec: &ServiceSpec) -> Result<TransitionResult, ServiceError> {
            self.events.borrow_mut().push("install");
            self.inner.install(spec)
        }

        fn start(
            &mut self,
            spec: &ServiceSpec,
            timeout: Duration,
        ) -> Result<TransitionResult, ServiceError> {
            self.events.borrow_mut().push("start");
            self.start_calls += 1;
            if self.start_delay > Duration::ZERO {
                thread::sleep(self.start_delay);
            }
            if self.fail_start_calls.contains(&self.start_calls) {
                return Err(ServiceError::manager("injected start failure"));
            }
            self.inner.start(spec, timeout)
        }

        fn stop(
            &mut self,
            spec: &ServiceSpec,
            timeout: Duration,
        ) -> Result<TransitionResult, ServiceError> {
            self.events.borrow_mut().push("stop");
            self.stop_calls += 1;
            if self.fail_stop {
                return Err(ServiceError::manager("injected stop failure"));
            }
            if self.incomplete_stop {
                return Ok(TransitionResult {
                    operation: ServiceOperation::Stop,
                    completed: false,
                    detail: "injected incomplete stop".into(),
                });
            }
            let result = self.inner.stop(spec, timeout)?;
            if self.drift_to_foreign_after_stop {
                let foreign = ServiceSpec::new(
                    spec.id().clone(),
                    PathBuf::from("/different/service-executable"),
                    spec.args().to_vec(),
                    spec.config().map(Path::to_path_buf),
                )?;
                self.inner.put(foreign, LifecycleState::Stopped);
            }
            Ok(result)
        }

        fn restart(
            &mut self,
            spec: &ServiceSpec,
            timeout: Duration,
        ) -> Result<TransitionResult, ServiceError> {
            self.events.borrow_mut().push("restart");
            self.inner.restart(spec, timeout)
        }

        fn uninstall(&mut self, spec: &ServiceSpec) -> Result<TransitionResult, ServiceError> {
            self.events.borrow_mut().push("uninstall");
            self.inner.uninstall(spec)
        }
    }

    fn setup(old_generation: bool) -> (PathBuf, PathBuf, ValidatedTransaction) {
        let base = std::env::temp_dir().join(format!(
            "eggup-service-lifecycle-update-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let source_root = base.join("input");
        let install_root = base.join("install");
        fs::create_dir_all(&source_root).unwrap();
        fs::create_dir_all(&install_root).unwrap();
        let main = source_root.join("main");
        let sidecar = source_root.join("sidecar");
        fs::write(&main, b"new-main").unwrap();
        fs::write(&sidecar, b"new-sidecar").unwrap();
        if old_generation {
            fs::write(install_root.join("service.bin"), b"old-main").unwrap();
            fs::write(install_root.join("sidecar.bin"), b"old-sidecar").unwrap();
        }
        let members = ArtifactSet::new(vec![
            ArtifactMember::new(MemberId::new("main").unwrap(), &main, "service.bin")
                .unwrap()
                .with_permissions(PermissionsIntent::Executable)
                .with_integrity(eggup_core::IntegrityRequirement::Sha256(
                    eggup_core::hash_file(&main).unwrap(),
                )),
            ArtifactMember::new(MemberId::new("sidecar").unwrap(), &sidecar, "sidecar.bin")
                .unwrap()
                .with_integrity(eggup_core::IntegrityRequirement::Sha256(
                    eggup_core::hash_file(&sidecar).unwrap(),
                )),
        ])
        .unwrap();
        let prepared = InstallPlan::new(
            ProductId::new("service-test").unwrap(),
            ReleaseId::new("r2").unwrap(),
            &install_root,
            members,
        )
        .unwrap()
        .prepare()
        .unwrap();
        let validated = prepared
            .verify_integrity()
            .unwrap()
            .validate(&AllValidators::new())
            .unwrap();
        (base, install_root, validated)
    }

    fn spec(install_root: &Path) -> ServiceSpec {
        ServiceSpec::new(
            ServiceId::new("m005-test-service").unwrap(),
            install_root.join("service.bin"),
            vec!["--worker".into()],
            None,
        )
        .unwrap()
    }

    fn policy(restore: RestoreIntent, failure: PostCommitFailurePolicy) -> LifecycleUpdatePolicy {
        LifecycleUpdatePolicy {
            restore,
            post_commit_failure: failure,
            quiesce_timeout: Duration::from_secs(2),
            post_commit_timeout: Duration::from_secs(2),
            rollback_restore_timeout: Duration::from_secs(2),
        }
    }

    #[derive(Debug)]
    struct PassCheck {
        complete: Option<Rc<Cell<bool>>>,
        observed_remaining: Option<Rc<Cell<u128>>>,
        events: Option<Rc<RefCell<Vec<&'static str>>>>,
        fail: bool,
    }
    impl PostInstallCheck for PassCheck {
        fn check(
            &self,
            _spec: &ServiceSpec,
            _snapshot: &LifecycleSnapshot,
            remaining: Duration,
        ) -> Result<(), PostInstallCheckError> {
            if let Some(events) = &self.events {
                events.borrow_mut().push("check");
            }
            if let Some(value) = &self.observed_remaining {
                value.set(remaining.as_nanos());
            }
            if let Some(complete) = &self.complete {
                complete.set(true);
            }
            if self.fail {
                Err(PostInstallCheckError::new("injected health check failure"))
            } else {
                Ok(())
            }
        }
    }

    #[derive(Debug)]
    struct CompleteGenerationCheck;
    impl PostInstallCheck for CompleteGenerationCheck {
        fn check(
            &self,
            spec: &ServiceSpec,
            _snapshot: &LifecycleSnapshot,
            _remaining: Duration,
        ) -> Result<(), PostInstallCheckError> {
            let root = spec
                .executable()
                .parent()
                .expect("service executable parent");
            if fs::read(root.join("service.bin")).ok().as_deref() == Some(b"new-main")
                && fs::read(root.join("sidecar.bin")).ok().as_deref() == Some(b"new-sidecar")
            {
                Ok(())
            } else {
                Err(PostInstallCheckError::new(
                    "complete artifact generation is not visible",
                ))
            }
        }
    }

    #[derive(Debug)]
    struct PanicCheck {
        called: Rc<Cell<bool>>,
    }
    impl PostInstallCheck for PanicCheck {
        fn check(
            &self,
            _spec: &ServiceSpec,
            _snapshot: &LifecycleSnapshot,
            _remaining: Duration,
        ) -> Result<(), PostInstallCheckError> {
            self.called.set(true);
            panic!("test panic payload must not escape");
        }
    }

    #[derive(Debug)]
    struct SabotageCheck(PathBuf);
    impl PostInstallCheck for SabotageCheck {
        fn check(
            &self,
            _spec: &ServiceSpec,
            _snapshot: &LifecycleSnapshot,
            _remaining: Duration,
        ) -> Result<(), PostInstallCheckError> {
            let dest = self.0.join("service.bin");
            fs::remove_file(&dest).unwrap();
            fs::create_dir(&dest).unwrap();
            Err(PostInstallCheckError::new("injected rollback failure"))
        }
    }

    fn absent_ownership() -> CoreCommitOwnership<'static> {
        // The verifier is a static test value, so the returned ownership can be static.
        static VERIFIER: AbsentOnlyVerifier = AbsentOnlyVerifier;
        CoreCommitOwnership::new(&VERIFIER, AbsentPolicy::AllowCreate)
    }

    #[test]
    fn rejects_zero_and_overlong_budgets_before_mutation() {
        let mut invalid = policy(
            RestoreIntent::Preserve,
            PostCommitFailurePolicy::KeepInstalled,
        );
        invalid.quiesce_timeout = Duration::ZERO;
        assert!(matches!(
            invalid.validate(),
            Err(LifecycleUpdateError::InvalidPolicy { .. })
        ));
        invalid.quiesce_timeout = Duration::from_secs(1);
        invalid.post_commit_timeout = crate::MAX_TRANSITION_TIMEOUT + Duration::from_nanos(1);
        assert!(matches!(
            invalid.validate(),
            Err(LifecycleUpdateError::InvalidPolicy { .. })
        ));
    }

    #[test]
    fn preserve_and_explicit_restore_intents_produce_expected_success_state() {
        for (pre_state, intent, expected) in [
            (
                LifecycleState::Running,
                RestoreIntent::Preserve,
                LifecycleState::Running,
            ),
            (
                LifecycleState::Stopped,
                RestoreIntent::Preserve,
                LifecycleState::Stopped,
            ),
            (
                LifecycleState::Stopped,
                RestoreIntent::EnsureRunning,
                LifecycleState::Running,
            ),
            (
                LifecycleState::Running,
                RestoreIntent::EnsureStopped,
                LifecycleState::Stopped,
            ),
        ] {
            let (base, root, tx) = setup(false);
            let service = spec(&root);
            let mut manager = ScriptedManager::with_service(service.clone(), pre_state);
            let receipt = commit_with_lifecycle(
                &mut manager,
                &service,
                tx,
                absent_ownership(),
                policy(intent, PostCommitFailurePolicy::KeepInstalled),
                &NoPostInstallCheck,
            )
            .unwrap();
            assert_eq!(
                receipt.artifact_disposition(),
                TransactionDisposition::Committed
            );
            assert_eq!(receipt.final_snapshot.as_ref().unwrap().state, expected);
            assert_eq!(
                receipt.pre_commit_quiesced,
                pre_state == LifecycleState::Running
            );
            let events = manager.events.borrow();
            assert_eq!(events.iter().filter(|e| **e == "install").count(), 0);
            if pre_state == LifecycleState::Stopped && intent == RestoreIntent::Preserve {
                assert_eq!(events.iter().filter(|e| **e == "stop").count(), 0);
                assert_eq!(events.iter().filter(|e| **e == "start").count(), 0);
            }
            let _ = fs::remove_dir_all(base);
        }
    }

    #[test]
    fn post_install_check_observes_the_complete_new_multi_member_generation() {
        let (base, root, tx) = setup(false);
        let service = spec(&root);
        let mut manager = ScriptedManager::with_service(service.clone(), LifecycleState::Stopped);
        let receipt = commit_with_lifecycle(
            &mut manager,
            &service,
            tx,
            absent_ownership(),
            policy(
                RestoreIntent::Preserve,
                PostCommitFailurePolicy::KeepInstalled,
            ),
            &CompleteGenerationCheck,
        )
        .unwrap();
        assert_eq!(
            receipt.artifact_disposition(),
            TransactionDisposition::Committed
        );
        assert!(receipt.post_commit_failure.is_none());
        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn absent_foreign_unknown_and_transitioning_preconditions_fail_before_artifact_commit() {
        for kind in 0..4 {
            let (base, root, tx) = setup(false);
            let service = spec(&root);
            let mut manager = ScriptedManager::default();
            match kind {
                0 => {} // absent
                1 => {
                    let foreign = ServiceSpec::new(
                        service.id().clone(),
                        PathBuf::from("/foreign"),
                        service.args().to_vec(),
                        None,
                    )
                    .unwrap();
                    manager.inner.put(foreign, LifecycleState::Stopped);
                }
                2 => manager.inner.put_malformed(service.id().clone()),
                _ => manager
                    .inner
                    .put(service.clone(), LifecycleState::Transitioning),
            }
            assert!(matches!(
                commit_with_lifecycle(
                    &mut manager,
                    &service,
                    tx,
                    absent_ownership(),
                    policy(
                        RestoreIntent::Preserve,
                        PostCommitFailurePolicy::KeepInstalled
                    ),
                    &NoPostInstallCheck
                ),
                Err(LifecycleUpdateError::Preflight { .. })
            ));
            assert!(!root.join("service.bin").exists());
            assert_eq!(
                manager
                    .events
                    .borrow()
                    .iter()
                    .filter(|e| **e == "stop")
                    .count(),
                0
            );
            let _ = fs::remove_dir_all(base);
        }
    }

    #[test]
    fn stop_error_incomplete_result_and_ownership_drift_prevent_core_mutation() {
        for mode in 0..3 {
            let (base, root, tx) = setup(false);
            let service = spec(&root);
            let mut manager =
                ScriptedManager::with_service(service.clone(), LifecycleState::Running);
            match mode {
                0 => manager.fail_stop = true,
                1 => manager.incomplete_stop = true,
                _ => manager.drift_to_foreign_after_stop = true,
            }
            let result = commit_with_lifecycle(
                &mut manager,
                &service,
                tx,
                absent_ownership(),
                policy(
                    RestoreIntent::Preserve,
                    PostCommitFailurePolicy::KeepInstalled,
                ),
                &NoPostInstallCheck,
            );
            assert!(matches!(result, Err(LifecycleUpdateError::Quiesce { .. })));
            assert!(!root.join("service.bin").exists());
            let _ = fs::remove_dir_all(base);
        }
    }

    #[test]
    fn keep_installed_check_failure_retains_core_commit_and_service_evidence() {
        let (base, root, tx) = setup(false);
        let service = spec(&root);
        let mut manager = ScriptedManager::with_service(service.clone(), LifecycleState::Running);
        let check = PassCheck {
            complete: None,
            observed_remaining: None,
            events: None,
            fail: true,
        };
        let receipt = commit_with_lifecycle(
            &mut manager,
            &service,
            tx,
            absent_ownership(),
            policy(
                RestoreIntent::Preserve,
                PostCommitFailurePolicy::KeepInstalled,
            ),
            &check,
        )
        .unwrap();
        assert_eq!(
            receipt.artifact_disposition(),
            TransactionDisposition::Committed
        );
        assert!(receipt.transaction.post_commit_failure().is_some());
        assert_eq!(
            receipt.post_commit_failure.as_ref().unwrap().phase,
            LifecycleUpdatePhase::PostInstallCheck
        );
        assert!(receipt.new_generation_retained());
        assert_eq!(
            receipt.final_snapshot.as_ref().unwrap().state,
            LifecycleState::Running
        );
        assert!(root.join("service.bin").exists());
        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn rollback_check_failure_stops_new_generation_before_core_rollback_then_restores_old_state() {
        let (base, root, tx) = setup(true);
        let service = spec(&root);
        let mut manager = ScriptedManager::with_service(service.clone(), LifecycleState::Running);
        let events = manager.events.clone();
        let check = PassCheck {
            complete: None,
            observed_remaining: None,
            events: Some(events.clone()),
            fail: true,
        };
        let verifier = FixedVerifier(CoreOwnership::Owned);
        let receipt = commit_with_lifecycle(
            &mut manager,
            &service,
            tx,
            CoreCommitOwnership::new(&verifier, AbsentPolicy::AllowCreate),
            policy(RestoreIntent::Preserve, PostCommitFailurePolicy::RollBack),
            &check,
        )
        .unwrap();
        assert_eq!(
            receipt.artifact_disposition(),
            TransactionDisposition::RolledBack
        );
        assert_eq!(fs::read(root.join("service.bin")).unwrap(), b"old-main");
        assert_eq!(fs::read(root.join("sidecar.bin")).unwrap(), b"old-sidecar");
        assert_eq!(
            receipt.final_snapshot.as_ref().unwrap().state,
            LifecycleState::Running
        );
        assert!(receipt.post_commit_failure.is_some());
        let events = events.borrow();
        let check_i = events.iter().position(|e| *e == "check").unwrap();
        let stops_after_check = events
            .iter()
            .enumerate()
            .filter(|(i, e)| *i > check_i && **e == "stop")
            .count();
        assert_eq!(stops_after_check, 1);
        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn caller_check_panic_is_bounded_and_rollback_quiescence_is_attempted() {
        let (base, root, tx) = setup(true);
        let service = spec(&root);
        let mut manager = ScriptedManager::with_service(service.clone(), LifecycleState::Running);
        let called = Rc::new(Cell::new(false));
        let receipt = commit_with_lifecycle(
            &mut manager,
            &service,
            tx,
            CoreCommitOwnership::new(
                &FixedVerifier(CoreOwnership::Owned),
                AbsentPolicy::AllowCreate,
            ),
            policy(RestoreIntent::Preserve, PostCommitFailurePolicy::RollBack),
            &PanicCheck {
                called: called.clone(),
            },
        )
        .unwrap();
        assert!(called.get());
        assert_eq!(
            receipt.artifact_disposition(),
            TransactionDisposition::RolledBack
        );
        assert_eq!(
            receipt.post_commit_failure.as_ref().unwrap().phase,
            LifecycleUpdatePhase::PostInstallCheck
        );
        assert_eq!(
            receipt.final_snapshot.as_ref().unwrap().state,
            LifecycleState::Running
        );
        assert!(
            manager.stop_calls >= 2,
            "old and new generations should each be quiesced"
        );
        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn core_pre_receipt_error_restores_pre_update_state() {
        let (base, root, tx) = setup(false);
        let service = spec(&root);
        let mut manager = ScriptedManager::with_service(service.clone(), LifecycleState::Running);
        // Core propagates lock-acquisition failure as Err before it can issue a receipt.
        fs::write(root.join(".eggup-mutation.lock"), b"contended lock record").unwrap();
        let error = commit_with_lifecycle(
            &mut manager,
            &service,
            tx,
            CoreCommitOwnership::new(
                &FixedVerifier(CoreOwnership::Unknown),
                AbsentPolicy::AllowCreate,
            ),
            policy(
                RestoreIntent::EnsureStopped,
                PostCommitFailurePolicy::KeepInstalled,
            ),
            &NoPostInstallCheck,
        )
        .unwrap_err();
        assert!(matches!(
            error,
            LifecycleUpdateError::CoreBeforeReceipt {
                restore_failure: None,
                ..
            }
        ));
        assert_eq!(
            manager.inner.inspect(&service).unwrap().state,
            LifecycleState::Running
        );
        assert!(!root.join("service.bin").exists());
        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn post_rollback_restore_failure_remains_separate_from_core_rolled_back_receipt() {
        let (base, root, tx) = setup(true);
        let service = spec(&root);
        let mut manager = ScriptedManager::with_service(service.clone(), LifecycleState::Running);
        manager.fail_start_calls = vec![1, 2]; // new-generation start, then old-generation restoration
        let check = PassCheck {
            complete: None,
            observed_remaining: None,
            events: None,
            fail: true,
        };
        let receipt = commit_with_lifecycle(
            &mut manager,
            &service,
            tx,
            CoreCommitOwnership::new(
                &FixedVerifier(CoreOwnership::Owned),
                AbsentPolicy::AllowCreate,
            ),
            policy(RestoreIntent::Preserve, PostCommitFailurePolicy::RollBack),
            &check,
        )
        .unwrap();
        assert_eq!(
            receipt.artifact_disposition(),
            TransactionDisposition::RolledBack
        );
        assert!(receipt.rollback_restore_failure.is_some());
        assert_eq!(receipt.restoration, LifecycleRestorationStatus::Failed);
        assert_eq!(fs::read(root.join("service.bin")).unwrap(), b"old-main");
        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn recovery_required_suppresses_automatic_old_generation_restart() {
        let (base, root, tx) = setup(true);
        let service = spec(&root);
        let mut manager = ScriptedManager::with_service(service.clone(), LifecycleState::Running);
        let receipt = commit_with_lifecycle(
            &mut manager,
            &service,
            tx,
            CoreCommitOwnership::new(
                &FixedVerifier(CoreOwnership::Owned),
                AbsentPolicy::AllowCreate,
            ),
            policy(RestoreIntent::Preserve, PostCommitFailurePolicy::RollBack),
            &SabotageCheck(root.clone()),
        )
        .unwrap();
        assert_eq!(
            receipt.artifact_disposition(),
            TransactionDisposition::RecoveryRequired
        );
        assert!(receipt.manual_artifact_recovery_required());
        assert_eq!(
            receipt.restoration,
            LifecycleRestorationStatus::NotAttemptedRecoveryRequired
        );
        assert_eq!(
            manager.start_calls, 1,
            "only the new generation may have been started"
        );
        assert_eq!(
            receipt.final_snapshot.as_ref().unwrap().state,
            LifecycleState::Stopped
        );
        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn one_post_commit_deadline_is_passed_to_the_check_and_final_observation_failure_is_separate() {
        let (base, root, tx) = setup(false);
        let service = spec(&root);
        let complete = Rc::new(Cell::new(false));
        let remaining = Rc::new(Cell::new(0));
        let mut manager = ScriptedManager::with_service(service.clone(), LifecycleState::Stopped);
        manager.start_delay = Duration::from_millis(25);
        manager.fail_final_inspect = true;
        manager.check_finished = Some(complete.clone());
        let check = PassCheck {
            complete: Some(complete.clone()),
            observed_remaining: Some(remaining.clone()),
            events: None,
            fail: false,
        };
        let receipt = commit_with_lifecycle(
            &mut manager,
            &service,
            tx,
            absent_ownership(),
            policy(
                RestoreIntent::EnsureRunning,
                PostCommitFailurePolicy::KeepInstalled,
            ),
            &check,
        )
        .unwrap();
        assert!(remaining.get() < Duration::from_secs(2).as_nanos());
        assert!(receipt.observation_failure.is_some());
        assert!(receipt.final_snapshot.is_none());
        assert_eq!(
            receipt.artifact_disposition(),
            TransactionDisposition::Committed
        );
        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn service_transition_failure_keep_installed_does_not_retry_hidden_policy() {
        let (base, root, tx) = setup(false);
        let service = spec(&root);
        let mut manager = ScriptedManager::with_service(service.clone(), LifecycleState::Running);
        manager.fail_start_calls = vec![1];
        let receipt = commit_with_lifecycle(
            &mut manager,
            &service,
            tx,
            absent_ownership(),
            policy(
                RestoreIntent::Preserve,
                PostCommitFailurePolicy::KeepInstalled,
            ),
            &NoPostInstallCheck,
        )
        .unwrap();
        assert_eq!(
            receipt.artifact_disposition(),
            TransactionDisposition::Committed
        );
        assert_eq!(
            receipt.post_commit_failure.as_ref().unwrap().phase,
            LifecycleUpdatePhase::RestoreNew
        );
        assert_eq!(manager.start_calls, 1);
        assert_eq!(
            receipt.final_snapshot.as_ref().unwrap().state,
            LifecycleState::Stopped
        );
        assert!(root.join("service.bin").exists());
        let _ = fs::remove_dir_all(base);
    }
}
