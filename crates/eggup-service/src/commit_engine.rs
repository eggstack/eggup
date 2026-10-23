#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! The single commit-and-rollback engine shared by every `eggup-service`
//! orchestrator.
//!
//! Before this module existed, `disposition.rs` and `lifecycle_update.rs` each
//! carried a full orchestrator **and** their own copies of the transition,
//! rollback-quiesce, panic-safe-inspect, panic-safe-restore, deadline, and
//! detail-truncation mechanics. The copies had already drifted apart in
//! safety-relevant ways. This module is the single home of those mechanics;
//! both public orchestrators reach them here.
//!
//! # What the engine guarantees
//!
//! * **No panic on manager- or caller-supplied state.** Every manager-reported
//!   state and every desired state is handled by a typed arm. A desired state
//!   outside `{Running, Stopped}` is a typed, bounded failure — never a panic,
//!   never a fabricated success.
//! * **Fail closed.** Ownership that is not `Owned` is a typed failure unless
//!   the caller explicitly selects
//!   [`OwnershipLossTolerance::TolerateAlreadyStopped`]. `Owned` is never
//!   inferred: it is returned only by a caller-proven ownership check.
//! * **One catch boundary per mechanic.** A panicking manager is caught at the
//!   mechanic that is executing and recorded under the phase actually
//!   executing, so a panicking `start`/`stop` is never reported as a
//!   post-install check failure.
//! * **One truncation rule.** Failure detail is control-character free and
//!   bounded by [`crate::MAX_DIAGNOSTIC_BYTES`].
//!
//! # Named policy inputs
//!
//! The two behavioural differences between the former copies are explicit,
//! named, separately testable inputs on [`CommitEnginePolicy`]:
//!
//! | Input | Default | Fail-closed direction |
//! |---|---|---|
//! | [`CommitEnginePolicy::post_transition_verification`] | [`PostTransitionVerification::Verify`] | a manager's own `completed` flag is a self-report, not evidence of observed state, so the engine re-inspects and validates |
//! | [`CommitEnginePolicy::ownership_loss_tolerance`] | [`OwnershipLossTolerance::Deny`] | unproven ownership must not be able to produce a successful outcome |
//!
//! Both defaults are the **stricter** of the two former behaviours.

use crate::{LifecycleSnapshot, LifecycleState, Ownership, ServiceManager, ServiceSpec};
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
    /// Builds bounded failure evidence for one phase and observation.
    pub(crate) fn new(
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

/// Whether the engine re-inspects the manager after a transition.
///
/// A manager's `completed` flag is a **self-report**, not evidence of observed
/// state: the M010 quiescence work already established that a nominal
/// completion is not proof. `Verify` therefore re-inspects the manager and
/// validates the observed ownership and state against the wanted state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum PostTransitionVerification {
    /// Re-inspect the manager and validate ownership and state after every
    /// transition. This is the default and the fail-closed direction.
    #[default]
    Verify,
    /// Accept the manager's own completion report without re-inspecting.
    ///
    /// This is the former disposition-path behaviour. It is available only as
    /// an explicitly selected, non-default input, because it can report a
    /// successful transition whose observed state or ownership never changed.
    TrustManagerReport,
}

/// Whether ownership loss during a transition is tolerated.
///
/// `Owned` is never inferred, so an ownership that is no longer `Owned` is
/// unproven. `Deny` (the default) makes that a typed failure; only the
/// explicitly selected `TolerateAlreadyStopped` input lets an already-stopped
/// unit pass a transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum OwnershipLossTolerance {
    /// Ownership that is not `Owned` is always a typed failure. This is the
    /// default and the fail-closed direction.
    #[default]
    Deny,
    /// Tolerate ownership loss when the unit is already in the wanted stopped
    /// state.
    ///
    /// This is the former disposition-path behaviour. It is available only as
    /// an explicitly selected, non-default input, because it lets an unproven
    /// ownership produce a successful outcome.
    TolerateAlreadyStopped,
}

/// The two named, separately testable inputs that distinguish the unified
/// engine's behaviour.
///
/// Both fields default to the **stricter** of the two former engine copies'
/// behaviours, and both public orchestrators run the same engine under this
/// default policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CommitEnginePolicy {
    /// Whether the engine re-inspects and validates after a transition.
    pub post_transition_verification: PostTransitionVerification,
    /// Whether ownership loss during a transition is tolerated.
    pub ownership_loss_tolerance: OwnershipLossTolerance,
}

/// The one valid target state of a lifecycle transition.
///
/// `Running` and `Stopped` are the only desired states a transition may
/// request. Every other [`LifecycleState`] is rejected by the engine at the
/// point of use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Desired {
    Running,
    Stopped,
}

/// Detail text used when a caller supplies a desired state that is not a valid
/// `Running`/`Stopped` target.
const INVALID_DESIRED_STATE: &str =
    "desired lifecycle state is not a valid Running or Stopped target";

/// Remaining budget under a deadline.
///
/// One implementation for the whole crate: the deadline arithmetic is
/// unchanged by this engine, only its single home is.
pub(crate) fn remaining(deadline: Instant) -> Duration {
    deadline.saturating_duration_since(Instant::now())
}

/// Bounded, control-character-free failure detail.
///
/// One implementation for the whole crate, bounded by
/// [`crate::MAX_DIAGNOSTIC_BYTES`].
pub(crate) fn bounded_detail(mut detail: String) -> String {
    detail = detail.chars().filter(|c| !c.is_control()).collect();
    crate::truncate_utf8_bytes(detail, crate::MAX_DIAGNOSTIC_BYTES)
}

/// Panic-safe manager inspection.
///
/// The catch boundary lives at the mechanic that is executing, so a panicking
/// manager is recorded under the phase actually running.
pub(crate) fn safe_inspect<M: ServiceManager>(
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

/// Whether an ownership loss is tolerated under the selected policy.
///
/// One implementation for the whole engine: the named tolerance governs
/// **every** ownership check the engine performs — the check at the transition
/// point and the post-transition verification alike — so the input means the
/// same thing wherever it is observed.
fn ownership_tolerated(
    policy: &CommitEnginePolicy,
    wanted: LifecycleState,
    snapshot: &LifecycleSnapshot,
) -> bool {
    policy.ownership_loss_tolerance == OwnershipLossTolerance::TolerateAlreadyStopped
        && wanted == LifecycleState::Stopped
        && snapshot.state == LifecycleState::Stopped
}

/// The single transition mechanic.
///
/// The engine performs, in order:
///
/// 1. the deadline check;
/// 2. the **desired-state validity check at the point of use** — a desired
///    state outside `{Running, Stopped}` is a typed, bounded failure;
/// 3. the ownership check, which honours
///    [`CommitEnginePolicy::ownership_loss_tolerance`];
/// 4. the `start`/`stop` call and its `completed` check;
/// 5. the optional post-transition re-inspection and validation, governed by
///    [`CommitEnginePolicy::post_transition_verification`].
///
/// No `unreachable!`, `unwrap`, `expect`, or arithmetic panic is reachable from
/// any manager-reported state or caller-supplied desired state.
pub(crate) fn transition_to<M: ServiceManager>(
    manager: &mut M,
    spec: &ServiceSpec,
    wanted: LifecycleState,
    timeout: Duration,
    phase: LifecycleUpdatePhase,
    policy: &CommitEnginePolicy,
) -> Result<(), LifecycleFailure> {
    if timeout.is_zero() {
        return Err(LifecycleFailure::new(
            spec,
            phase,
            "lifecycle operation deadline exhausted",
            None,
        ));
    }
    let desired = match wanted {
        LifecycleState::Running => Desired::Running,
        LifecycleState::Stopped => Desired::Stopped,
        LifecycleState::Transitioning | LifecycleState::Unknown => {
            return Err(LifecycleFailure::new(
                spec,
                phase,
                INVALID_DESIRED_STATE,
                None,
            ))
        }
    };
    let current = manager
        .inspect(spec)
        .map_err(|e| LifecycleFailure::new(spec, phase, e, None))?;
    if current.ownership != Ownership::Owned {
        if ownership_tolerated(policy, wanted, &current) {
            return Ok(());
        }
        return Err(LifecycleFailure::new(
            spec,
            phase,
            "service ownership is no longer Owned",
            Some(&current),
        ));
    }
    let result = match (desired, current.state) {
        (Desired::Running, LifecycleState::Running)
        | (Desired::Stopped, LifecycleState::Stopped) => return Ok(()),
        (Desired::Running, LifecycleState::Stopped) => manager.start(spec, timeout),
        (Desired::Stopped, LifecycleState::Running) => manager.stop(spec, timeout),
        (_, LifecycleState::Transitioning | LifecycleState::Unknown) => {
            return Err(LifecycleFailure::new(
                spec,
                phase,
                "service state is Unknown or Transitioning",
                Some(&current),
            ))
        }
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
    if policy.post_transition_verification == PostTransitionVerification::Verify {
        let after = manager
            .inspect(spec)
            .map_err(|e| LifecycleFailure::new(spec, phase, e, None))?;
        validate_observed(spec, &after, wanted, phase, policy)?;
    }
    Ok(())
}

/// Panic-safe [`transition_to`].
///
/// The caller's own `catch_unwind` wraps a whole post-commit block, so a panic
/// escaping here would be labelled with the wrong phase. Catching it at the
/// point of execution records it under `phase` — so a panicking `start`/`stop`
/// cannot be reported as a post-install check failure.
pub(crate) fn safe_transition_to<M: ServiceManager>(
    manager: &mut M,
    spec: &ServiceSpec,
    wanted: LifecycleState,
    timeout: Duration,
    phase: LifecycleUpdatePhase,
    policy: &CommitEnginePolicy,
) -> Result<(), LifecycleFailure> {
    match catch_unwind(AssertUnwindSafe(|| {
        transition_to(manager, spec, wanted, timeout, phase, policy)
    })) {
        Ok(result) => result,
        Err(_) => Err(LifecycleFailure::new(
            spec,
            phase,
            "service manager panicked while transitioning",
            None,
        )),
    }
}

/// Validates one observation against the wanted state.
///
/// The ownership half of the check honours the selected
/// [`CommitEnginePolicy::ownership_loss_tolerance`], so the named tolerance
/// means the same thing at the transition point and in the post-transition
/// verification.
pub(crate) fn validate_observed(
    spec: &ServiceSpec,
    snapshot: &LifecycleSnapshot,
    wanted: LifecycleState,
    phase: LifecycleUpdatePhase,
    policy: &CommitEnginePolicy,
) -> Result<(), LifecycleFailure> {
    let ownership_ok =
        snapshot.ownership == Ownership::Owned || ownership_tolerated(policy, wanted, snapshot);
    if !ownership_ok || snapshot.state != wanted {
        return Err(LifecycleFailure::new(
            spec,
            phase,
            "service ownership/state does not match the requested state",
            Some(snapshot),
        ));
    }
    Ok(())
}

/// Stops an owned, running service before Core mutation.
///
/// One implementation for the whole crate. The stop is panic-guarded at the
/// mechanic that is executing, and the post-stop observation must prove
/// `Owned` and `Stopped` before the mutation is allowed to proceed.
pub(crate) fn quiesce_service<M: ServiceManager>(
    manager: &mut M,
    spec: &ServiceSpec,
    before: &LifecycleSnapshot,
    timeout: Duration,
) -> Result<(), LifecycleFailure> {
    let stop_result = catch_unwind(AssertUnwindSafe(|| manager.stop(spec, timeout)));
    match stop_result {
        Ok(Err(error)) => Err(LifecycleFailure::new(
            spec,
            LifecycleUpdatePhase::Quiesce,
            error,
            Some(before),
        )),
        Err(_) => Err(LifecycleFailure::new(
            spec,
            LifecycleUpdatePhase::Quiesce,
            "service manager panicked while stopping",
            Some(before),
        )),
        Ok(Ok(result)) if !result.completed => Err(LifecycleFailure::new(
            spec,
            LifecycleUpdatePhase::Quiesce,
            "stop did not complete",
            Some(before),
        )),
        Ok(Ok(_)) => {
            let after = safe_inspect(manager, spec, LifecycleUpdatePhase::Quiesce)?;
            if after.ownership == Ownership::Owned && after.state == LifecycleState::Stopped {
                Ok(())
            } else {
                Err(LifecycleFailure::new(
                    spec,
                    LifecycleUpdatePhase::Quiesce,
                    "ownership/state changed after stop",
                    Some(&after),
                ))
            }
        }
    }
}

/// Stops the new generation before Core rolls the artifact back.
///
/// One implementation for the whole crate, and the **fail-closed** one:
///
/// * an `inspect` failure is not "no information, proceed" — ownership could
///   not be established, so the destructive `stop` below would run against an
///   unproven target. It is refused instead;
/// * a service that is not `Owned` is refused as well.
///
/// The caller wraps this call in its own `catch_unwind`, because this
/// mechanic's boundary is the caller's post-commit block.
pub(crate) fn quiesce_for_rollback<M: ServiceManager>(
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
    let observed = match manager.inspect(spec) {
        Ok(snapshot) => Some(snapshot),
        Err(error) => {
            return Some(LifecycleFailure::new(
                spec,
                LifecycleUpdatePhase::QuiesceForRollback,
                format!("refusing to quiesce unobservable service before rollback: {error}"),
                None,
            ))
        }
    };
    if observed
        .as_ref()
        .is_some_and(|s| s.ownership == Ownership::Owned && s.state == LifecycleState::Stopped)
    {
        return None;
    }
    if let Some(observation) = observed.as_ref() {
        if observation.ownership != Ownership::Owned {
            return Some(LifecycleFailure::new(
                spec,
                LifecycleUpdatePhase::QuiesceForRollback,
                "refusing to quiesce non-owned service before rollback",
                Some(observation),
            ));
        }
    }
    match manager.stop(spec, budget) {
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

/// Restores the wanted lifecycle state after a successful artifact update and
/// then runs the caller's post-install check.
///
/// One implementation for the whole crate. The transition is panic-guarded at
/// its own boundary, so a panicking `start`/`stop` is recorded under
/// `RestoreNew` and never as a post-install check failure.
pub(crate) fn restore_and_check<M: ServiceManager, C: PostInstallCheck>(
    manager: &mut M,
    spec: &ServiceSpec,
    wanted: LifecycleState,
    check: &C,
    deadline: Instant,
    policy: &CommitEnginePolicy,
) -> Result<(), LifecycleFailure> {
    safe_transition_to(
        manager,
        spec,
        wanted,
        remaining(deadline),
        LifecycleUpdatePhase::RestoreNew,
        policy,
    )?;
    let snapshot = safe_inspect(manager, spec, LifecycleUpdatePhase::RestoreNew)?;
    if policy.post_transition_verification == PostTransitionVerification::Verify {
        validate_observed(
            spec,
            &snapshot,
            wanted,
            LifecycleUpdatePhase::RestoreNew,
            policy,
        )?;
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HealthState, ServiceError, ServiceId, TransitionResult};
    use std::time::Duration;

    fn spec() -> ServiceSpec {
        ServiceSpec::new(
            ServiceId::new("commit-engine").unwrap(),
            std::env::current_exe().unwrap(),
            Vec::new(),
            None,
        )
        .unwrap()
    }

    /// Manager double that always reports `Running` and `Owned`.
    #[derive(Debug)]
    struct RunningManager;

    impl ServiceManager for RunningManager {
        fn inspect(&self, spec: &ServiceSpec) -> Result<LifecycleSnapshot, ServiceError> {
            Ok(LifecycleSnapshot {
                id: spec.id().clone(),
                ownership: Ownership::Owned,
                state: LifecycleState::Running,
                health: HealthState::Unknown,
                was_registered: true,
                was_running: true,
            })
        }

        fn install(&mut self, _spec: &ServiceSpec) -> Result<TransitionResult, ServiceError> {
            unreachable!("not exercised")
        }

        fn start(
            &mut self,
            _spec: &ServiceSpec,
            _timeout: Duration,
        ) -> Result<TransitionResult, ServiceError> {
            unreachable!("not exercised")
        }

        fn stop(
            &mut self,
            _spec: &ServiceSpec,
            _timeout: Duration,
        ) -> Result<TransitionResult, ServiceError> {
            unreachable!("not exercised")
        }

        fn restart(
            &mut self,
            _spec: &ServiceSpec,
            _timeout: Duration,
        ) -> Result<TransitionResult, ServiceError> {
            unreachable!("not exercised")
        }

        fn uninstall(&mut self, _spec: &ServiceSpec) -> Result<TransitionResult, ServiceError> {
            unreachable!("not exercised")
        }
    }

    /// A desired state outside `{Running, Stopped}` is a typed, bounded
    /// failure — never a panic, never a fabricated success.
    #[test]
    fn non_running_stopped_desired_state_is_a_typed_failure() {
        for desired in [LifecycleState::Transitioning, LifecycleState::Unknown] {
            let mut manager = RunningManager;
            let spec = spec();
            let failure = transition_to(
                &mut manager,
                &spec,
                desired,
                Duration::from_secs(1),
                LifecycleUpdatePhase::RestoreNew,
                &CommitEnginePolicy::default(),
            )
            .expect_err("an invalid desired state must fail");
            assert_eq!(failure.phase, LifecycleUpdatePhase::RestoreNew);
            assert_eq!(failure.detail, INVALID_DESIRED_STATE);
            assert!(failure.detail.len() <= crate::MAX_DIAGNOSTIC_BYTES);
        }
    }
}
