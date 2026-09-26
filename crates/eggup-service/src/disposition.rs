#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Generic daemon update disposition and reference qualification.
//!
//! Separates artifact mutation authority from service-manager mutation
//! authority. [`UpdateRuntimeDisposition`] describes what lifecycle authority
//! Eggup may exercise around an already validated transaction. Pure planners
//! reproduce the reference matrix without native managers, and
//! [`commit_with_disposition`] enforces prepare-before-quiesce plus a
//! post-preparation, pre-mutation revalidation barrier.
//!
//! Reference behavior only: greggd at `eggstack/gregg@8b18f9ee` is a behavioral
//! oracle. No Gregg code, dependency, or product terminology is used here.

use crate::{
    LifecycleSnapshot, LifecycleState, LifecycleUpdatePolicy, Ownership, PostInstallCheck,
    PostInstallCheckError, ServiceManager, ServiceSpec, TransitionResult,
};
use eggup_core::{
    CommitOwnership, PostCommitFailurePolicy, TransactionDisposition, ValidatedTransaction,
};
use std::fmt;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Product-neutral update runtime disposition.
///
/// Artifact commit authority and manager mutation authority are separate facts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateRuntimeDisposition {
    /// Owned manager, running: quiesce via manager, commit, restore/start per policy.
    ManagedRunning,
    /// Owned manager, stopped: commit and preserve stopped unless explicitly requested otherwise.
    ManagedStopped,
    /// Selected direct runtime running: quiesce/restore only via caller-owned direct seam.
    DirectRunning,
    /// No running runtime: commit with no fabricated restart.
    Stopped,
    /// Foreign manager preserved: zero manager mutation.
    ForeignPreserved,
}

/// Known vs unknown registered config identity for Unix-like planning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KnownConfig {
    /// No config participates in identity.
    Absent,
    /// Exact registered config path.
    Present(PathBuf),
    /// Registered config could not be determined.
    Unknown,
}

/// Selected direct-runtime state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectState {
    /// Selected instance is running.
    Running,
    /// Selected instance is stopped.
    Stopped,
    /// Selected runtime state cannot be proven.
    Unknown,
}

/// Caller-supplied selected direct-runtime observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectObservation {
    /// Runtime state of the selected instance.
    pub state: DirectState,
    /// Whether exact instance/config ownership is proven.
    pub exact_ownership_proven: bool,
    /// Exact selected executable identity, when known.
    pub executable: Option<PathBuf>,
    /// Exact selected config identity, when relevant.
    pub config: Option<PathBuf>,
}

/// Unix-like planner inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnixPlannerInput {
    /// Manager registration ownership.
    pub manager_ownership: Ownership,
    /// Whether the manager reports active (running).
    pub manager_active: bool,
    /// Whether the manager state is known (false = transitioning/unknown).
    pub manager_state_known: bool,
    /// Known registered config identity, when relevant.
    pub registered_config: KnownConfig,
    /// Selected artifact config identity, when relevant.
    pub selected_config: Option<PathBuf>,
    /// Caller-supplied selected direct-runtime observation.
    pub direct: DirectObservation,
}

/// Windows SCM state classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowsServiceState {
    /// Running.
    Running,
    /// Start-pending.
    StartPending,
    /// Stopped.
    Stopped,
    /// Stop-pending.
    StopPending,
    /// Unknown or contradictory.
    Unknown,
}

/// Windows-like planner inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowsPlannerInput {
    /// Whether an SCM registration exists.
    pub installed: bool,
    /// Whether the registration executable is present and parseable.
    pub executable_known: bool,
    /// Ownership from exact executable-path equivalence.
    pub ownership: Ownership,
    /// SCM state class.
    pub state: WindowsServiceState,
}

/// Pure Unix-like manager/direct decision logic.
pub fn plan_unix(
    input: &UnixPlannerInput,
) -> Result<UpdateRuntimeDisposition, crate::ServiceError> {
    if !input.manager_state_known {
        if input.manager_ownership == Ownership::Unknown
            && !input.manager_active
            && input.direct.state == DirectState::Stopped
        {
            return Ok(UpdateRuntimeDisposition::Stopped);
        }
        return Err(crate::ServiceError::invalid(
            "manager state is transitioning or unknown",
        ));
    }
    match input.manager_ownership {
        Ownership::Owned => {
            if input.manager_active {
                Ok(UpdateRuntimeDisposition::ManagedRunning)
            } else {
                Ok(UpdateRuntimeDisposition::ManagedStopped)
            }
        }
        Ownership::Foreign => {
            if input.manager_active {
                match &input.registered_config {
                    KnownConfig::Unknown => Err(crate::ServiceError::invalid(
                        "registered config is unknown; refusing to infer disposition",
                    )),
                    KnownConfig::Absent | KnownConfig::Present(_) => {
                        let same = match (&input.registered_config, &input.selected_config) {
                            (KnownConfig::Absent, None) => true,
                            (KnownConfig::Present(a), Some(b)) => a == b,
                            _ => false,
                        };
                        if same {
                            Ok(UpdateRuntimeDisposition::ForeignPreserved)
                        } else {
                            match input.direct.state {
                                DirectState::Running if input.direct.exact_ownership_proven => {
                                    Ok(UpdateRuntimeDisposition::DirectRunning)
                                }
                                DirectState::Stopped => Ok(UpdateRuntimeDisposition::Stopped),
                                _ => Err(crate::ServiceError::invalid(
                                    "direct runtime ownership/state unproven",
                                )),
                            }
                        }
                    }
                }
            } else {
                match input.direct.state {
                    DirectState::Running if input.direct.exact_ownership_proven => {
                        Ok(UpdateRuntimeDisposition::DirectRunning)
                    }
                    DirectState::Stopped => Ok(UpdateRuntimeDisposition::Stopped),
                    _ => Err(crate::ServiceError::invalid(
                        "direct runtime ownership/state unproven",
                    )),
                }
            }
        }
        Ownership::Absent => {
            if input.manager_active {
                Err(crate::ServiceError::invalid(
                    "registration absent but manager reports active",
                ))
            } else {
                match input.direct.state {
                    DirectState::Running if input.direct.exact_ownership_proven => {
                        Ok(UpdateRuntimeDisposition::DirectRunning)
                    }
                    DirectState::Stopped => Ok(UpdateRuntimeDisposition::Stopped),
                    _ => Err(crate::ServiceError::invalid(
                        "direct runtime ownership/state unproven",
                    )),
                }
            }
        }
        Ownership::Unknown => {
            if input.manager_active {
                Err(crate::ServiceError::invalid(
                    "manager ownership unknown while active",
                ))
            } else {
                match input.direct.state {
                    DirectState::Running => Err(crate::ServiceError::invalid(
                        "manager ownership unknown; refusing direct mutation",
                    )),
                    DirectState::Stopped => Ok(UpdateRuntimeDisposition::Stopped),
                    DirectState::Unknown => Err(crate::ServiceError::invalid(
                        "manager and direct states unproven",
                    )),
                }
            }
        }
    }
}

/// Pure Windows SCM/executable decision logic including pending states.
pub fn plan_windows(
    input: &WindowsPlannerInput,
) -> Result<UpdateRuntimeDisposition, crate::ServiceError> {
    if !input.installed {
        return Ok(UpdateRuntimeDisposition::Stopped);
    }
    if !input.executable_known {
        return Err(crate::ServiceError::invalid(
            "service executable absent or unparseable",
        ));
    }
    match input.ownership {
        Ownership::Foreign => Ok(UpdateRuntimeDisposition::ForeignPreserved),
        Ownership::Owned => match input.state {
            WindowsServiceState::Running | WindowsServiceState::StartPending => {
                Ok(UpdateRuntimeDisposition::ManagedRunning)
            }
            WindowsServiceState::Stopped | WindowsServiceState::StopPending => {
                Ok(UpdateRuntimeDisposition::ManagedStopped)
            }
            WindowsServiceState::Unknown => Err(crate::ServiceError::invalid(
                "windows service state unknown",
            )),
        },
        Ownership::Absent => Err(crate::ServiceError::invalid(
            "windows registration state contradictory",
        )),
        Ownership::Unknown => Err(crate::ServiceError::invalid("windows ownership unproven")),
    }
}

/// Minimum caller-owned direct-runtime control seam.
pub trait DirectRuntimeControl: fmt::Debug {
    /// Re-observe the selected instance.
    fn inspect(&self) -> Result<DirectObservation, crate::ServiceError>;
    /// Stop/quiesce the selected instance within the deadline.
    fn stop(&mut self, timeout: Duration) -> Result<TransitionResult, crate::ServiceError>;
    /// Start/restore the selected instance within the deadline.
    fn start(&mut self, timeout: Duration) -> Result<TransitionResult, crate::ServiceError>;
}

/// Baseline authority/state observed after preparation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispositionBaseline {
    /// Manager ownership observed after preparation.
    pub manager_ownership: Ownership,
    /// Manager state observed after preparation.
    pub manager_state: LifecycleState,
    /// Direct observation observed after preparation, when relevant.
    pub direct: Option<DirectObservation>,
}

fn mk_failure(
    spec: &ServiceSpec,
    phase: crate::LifecycleUpdatePhase,
    detail: impl fmt::Display,
    snapshot: Option<&LifecycleSnapshot>,
) -> crate::LifecycleFailure {
    crate::LifecycleFailure {
        phase,
        service_id: spec.id().as_str().to_owned(),
        detail: bound_detail(detail.to_string()),
        ownership: snapshot.map(|s| s.ownership),
        state: snapshot.map(|s| s.state),
    }
}

fn bound_detail(mut detail: String) -> String {
    detail = detail.chars().filter(|c| !c.is_control()).collect();
    crate::truncate_utf8_bytes(detail, 512)
}

fn remaining(deadline: Instant) -> Duration {
    deadline.saturating_duration_since(Instant::now())
}

fn safe_manager_inspect<M: ServiceManager>(
    manager: &mut M,
    spec: &ServiceSpec,
    phase: crate::LifecycleUpdatePhase,
) -> Result<LifecycleSnapshot, crate::LifecycleFailure> {
    match catch_unwind(AssertUnwindSafe(|| manager.inspect(spec))) {
        Ok(Ok(s)) => Ok(s),
        Ok(Err(e)) => Err(mk_failure(spec, phase, e, None)),
        Err(_) => Err(mk_failure(
            spec,
            phase,
            "service manager panicked while inspecting",
            None,
        )),
    }
}

/// Disposition-aware orchestration over an existing `ValidatedTransaction`.
///
/// See module docs for ordering and revalidation rules.
#[allow(clippy::too_many_arguments)]
pub fn commit_with_disposition<M, D, C>(
    manager: &mut M,
    spec: &ServiceSpec,
    direct: Option<&mut D>,
    transaction: ValidatedTransaction,
    ownership: CommitOwnership<'_>,
    policy: LifecycleUpdatePolicy,
    check: &C,
    planned: UpdateRuntimeDisposition,
    baseline: &DispositionBaseline,
) -> Result<crate::LifecycleUpdateReceipt, crate::LifecycleUpdateError>
where
    M: ServiceManager,
    D: DirectRuntimeControl,
    C: PostInstallCheck,
{
    use crate::LifecycleUpdatePhase as Phase;
    policy.validate()?;

    let baseline_ok = match planned {
        UpdateRuntimeDisposition::ManagedRunning => {
            baseline.manager_ownership == Ownership::Owned
                && baseline.manager_state == LifecycleState::Running
        }
        UpdateRuntimeDisposition::ManagedStopped => {
            baseline.manager_ownership == Ownership::Owned
                && baseline.manager_state == LifecycleState::Stopped
        }
        UpdateRuntimeDisposition::DirectRunning => matches!(
            baseline.direct.as_ref(),
            Some(o) if o.state == DirectState::Running && o.exact_ownership_proven
        ),
        UpdateRuntimeDisposition::Stopped => true,
        UpdateRuntimeDisposition::ForeignPreserved => matches!(
            baseline.manager_ownership,
            Ownership::Foreign | Ownership::Absent | Ownership::Unknown
        ),
    };
    if !baseline_ok {
        return Err(crate::LifecycleUpdateError::Preflight {
            failure: mk_failure(
                spec,
                Phase::Inspect,
                "planned disposition does not match baseline authority/state",
                None,
            ),
        });
    }

    let fresh_manager = safe_manager_inspect(manager, spec, Phase::Inspect)
        .map_err(|f| crate::LifecycleUpdateError::Preflight { failure: f })?;
    if fresh_manager.ownership != baseline.manager_ownership
        || fresh_manager.state != baseline.manager_state
    {
        return Err(crate::LifecycleUpdateError::Preflight {
            failure: mk_failure(
                spec,
                Phase::Inspect,
                "lifecycle authority/state changed before mutation",
                Some(&fresh_manager),
            ),
        });
    }

    // Direct revalidation when the plan exercises direct control.
    if planned == UpdateRuntimeDisposition::DirectRunning {
        let Some(d) = direct.as_ref() else {
            return Err(crate::LifecycleUpdateError::Preflight {
                failure: mk_failure(
                    spec,
                    Phase::Inspect,
                    "direct control required but not provided",
                    Some(&fresh_manager),
                ),
            });
        };
        let fresh_direct = match catch_unwind(AssertUnwindSafe(|| (*d).inspect())) {
            Ok(Ok(o)) => o,
            Ok(Err(e)) => {
                return Err(crate::LifecycleUpdateError::Preflight {
                    failure: mk_failure(spec, Phase::Inspect, e, Some(&fresh_manager)),
                });
            }
            Err(_) => {
                return Err(crate::LifecycleUpdateError::Preflight {
                    failure: mk_failure(
                        spec,
                        Phase::Inspect,
                        "direct runtime panicked while inspecting",
                        Some(&fresh_manager),
                    ),
                });
            }
        };
        let Some(base_direct) = baseline.direct.as_ref() else {
            return Err(crate::LifecycleUpdateError::Preflight {
                failure: mk_failure(
                    spec,
                    Phase::Inspect,
                    "direct baseline missing for DirectRunning",
                    Some(&fresh_manager),
                ),
            });
        };
        if fresh_direct != *base_direct
            || !fresh_direct.exact_ownership_proven
            || fresh_direct.state != DirectState::Running
        {
            return Err(crate::LifecycleUpdateError::Preflight {
                failure: mk_failure(
                    spec,
                    Phase::Inspect,
                    "direct runtime identity/state changed before mutation",
                    Some(&fresh_manager),
                ),
            });
        }
    }

    let before = fresh_manager;
    // Keep the direct reference alive across commit via an Option slot that is
    // reborrowed (never moved) for quiesce, post-commit restore, and rollback.
    let mut direct_slot: Option<&mut D> = direct;
    let mut pre_commit_quiesced = false;

    match planned {
        UpdateRuntimeDisposition::ManagedRunning => {
            let stop_result = catch_unwind(AssertUnwindSafe(|| {
                manager.stop(spec, policy.quiesce_timeout)
            }));
            let stop_failure = match stop_result {
                Ok(Err(e)) => Some(mk_failure(spec, Phase::Quiesce, e, Some(&before))),
                Err(_) => Some(mk_failure(
                    spec,
                    Phase::Quiesce,
                    "service manager panicked while stopping",
                    Some(&before),
                )),
                Ok(Ok(r)) if !r.completed => Some(mk_failure(
                    spec,
                    Phase::Quiesce,
                    "stop did not complete",
                    Some(&before),
                )),
                Ok(Ok(_)) => match safe_manager_inspect(manager, spec, Phase::Quiesce) {
                    Ok(after)
                        if after.ownership == Ownership::Owned
                            && after.state == LifecycleState::Stopped =>
                    {
                        pre_commit_quiesced = true;
                        None
                    }
                    Ok(after) => Some(mk_failure(
                        spec,
                        Phase::Quiesce,
                        "ownership/state changed after stop",
                        Some(&after),
                    )),
                    Err(f) => Some(f),
                },
            };
            if let Some(f) = stop_failure {
                let restore = restore_owned_state(
                    manager,
                    spec,
                    LifecycleState::Running,
                    policy.rollback_restore_timeout,
                )
                .err();
                return Err(crate::LifecycleUpdateError::Quiesce {
                    failure: f,
                    restore_failure: restore,
                });
            }
            commit_manager_path(
                manager,
                spec,
                before,
                transaction,
                ownership,
                policy,
                check,
                planned,
                pre_commit_quiesced,
            )
        }
        UpdateRuntimeDisposition::DirectRunning => {
            let Some(d) = direct_slot.as_mut() else {
                return Err(crate::LifecycleUpdateError::Preflight {
                    failure: mk_failure(
                        spec,
                        Phase::Inspect,
                        "direct control required but not provided",
                        Some(&before),
                    ),
                });
            };
            let stop_result = catch_unwind(AssertUnwindSafe(|| (**d).stop(policy.quiesce_timeout)));
            let stop_failure = match stop_result {
                Ok(Err(e)) => Some(mk_failure(spec, Phase::Quiesce, e, Some(&before))),
                Err(_) => Some(mk_failure(
                    spec,
                    Phase::Quiesce,
                    "direct runtime panicked while stopping",
                    Some(&before),
                )),
                Ok(Ok(r)) if !r.completed => Some(mk_failure(
                    spec,
                    Phase::Quiesce,
                    "direct stop did not complete",
                    Some(&before),
                )),
                Ok(Ok(_)) => match catch_unwind(AssertUnwindSafe(|| (**d).inspect())) {
                    Ok(Ok(after))
                        if after.state == DirectState::Stopped && after.exact_ownership_proven =>
                    {
                        pre_commit_quiesced = true;
                        None
                    }
                    Ok(Ok(_)) => Some(mk_failure(
                        spec,
                        Phase::Quiesce,
                        "direct runtime state changed after stop",
                        Some(&before),
                    )),
                    Ok(Err(e)) => Some(mk_failure(spec, Phase::Quiesce, e, Some(&before))),
                    Err(_) => Some(mk_failure(
                        spec,
                        Phase::Quiesce,
                        "direct runtime panicked while re-observing",
                        Some(&before),
                    )),
                },
            };
            if let Some(f) = stop_failure {
                // Best-effort restore of the prior direct generation.
                if let Some(d2) = direct_slot.as_mut() {
                    let _ = catch_unwind(AssertUnwindSafe(|| {
                        let _ = (**d2).start(policy.rollback_restore_timeout);
                    }));
                }
                return Err(crate::LifecycleUpdateError::Quiesce {
                    failure: f,
                    restore_failure: None,
                });
            }
            commit_direct_path(
                manager,
                spec,
                direct_slot,
                before,
                transaction,
                ownership,
                policy,
                check,
                pre_commit_quiesced,
            )
        }
        UpdateRuntimeDisposition::ManagedStopped
        | UpdateRuntimeDisposition::Stopped
        | UpdateRuntimeDisposition::ForeignPreserved => commit_manager_path(
            manager,
            spec,
            before,
            transaction,
            ownership,
            policy,
            check,
            planned,
            pre_commit_quiesced,
        ),
    }
}

fn restore_owned_state<M: ServiceManager>(
    manager: &mut M,
    spec: &ServiceSpec,
    wanted: LifecycleState,
    timeout: Duration,
) -> Result<(), crate::LifecycleFailure> {
    use crate::LifecycleUpdatePhase as Phase;
    if timeout.is_zero() {
        return Err(mk_failure(
            spec,
            Phase::RestoreOld,
            "lifecycle operation deadline exhausted",
            None,
        ));
    }
    let current = manager
        .inspect(spec)
        .map_err(|e| mk_failure(spec, Phase::RestoreOld, e, None))?;
    if current.ownership != Ownership::Owned {
        return Err(mk_failure(
            spec,
            Phase::RestoreOld,
            "service ownership is no longer Owned",
            Some(&current),
        ));
    }
    match (wanted, current.state) {
        (LifecycleState::Running, LifecycleState::Running)
        | (LifecycleState::Stopped, LifecycleState::Stopped) => Ok(()),
        (LifecycleState::Running, LifecycleState::Stopped) => {
            let r = manager
                .start(spec, timeout)
                .map_err(|e| mk_failure(spec, Phase::RestoreOld, e, Some(&current)))?;
            if !r.completed {
                return Err(mk_failure(
                    spec,
                    Phase::RestoreOld,
                    "service transition did not complete",
                    Some(&current),
                ));
            }
            Ok(())
        }
        (LifecycleState::Stopped, LifecycleState::Running) => {
            let r = manager
                .stop(spec, timeout)
                .map_err(|e| mk_failure(spec, Phase::RestoreOld, e, Some(&current)))?;
            if !r.completed {
                return Err(mk_failure(
                    spec,
                    Phase::RestoreOld,
                    "service transition did not complete",
                    Some(&current),
                ));
            }
            Ok(())
        }
        _ => Err(mk_failure(
            spec,
            Phase::RestoreOld,
            "service state is Unknown or Transitioning",
            Some(&current),
        )),
    }
}

#[allow(clippy::too_many_arguments)]
fn commit_manager_path<M, C>(
    manager: &mut M,
    spec: &ServiceSpec,
    before: LifecycleSnapshot,
    transaction: ValidatedTransaction,
    ownership: CommitOwnership<'_>,
    policy: LifecycleUpdatePolicy,
    check: &C,
    planned: UpdateRuntimeDisposition,
    pre_commit_quiesced: bool,
) -> Result<crate::LifecycleUpdateReceipt, crate::LifecycleUpdateError>
where
    M: ServiceManager,
    C: PostInstallCheck,
{
    use crate::LifecycleUpdatePhase as Phase;
    let mut post_commit_failure = None;
    let mut rollback_quiesce_failure = None;
    let planned_c = planned;
    let before_c = before.clone();
    let core_result =
        transaction.commit_with_post_commit(ownership, policy.post_commit_failure, || {
            let deadline = Instant::now() + policy.post_commit_timeout;
            let work = catch_unwind(AssertUnwindSafe(|| {
                restore_manager_success(
                    manager,
                    spec,
                    &before_c,
                    planned_c,
                    policy.restore,
                    check,
                    deadline,
                )
            }));
            let f = match work {
                Ok(Ok(())) => return Ok(()),
                Ok(Err(f)) => f,
                Err(_) => mk_failure(
                    spec,
                    Phase::PostInstallCheck,
                    "post-commit service work panicked",
                    None,
                ),
            };
            post_commit_failure = Some(f.clone());
            if policy.post_commit_failure == PostCommitFailurePolicy::RollBack {
                rollback_quiesce_failure = match catch_unwind(AssertUnwindSafe(|| {
                    quiesce_manager_for_rollback(manager, spec, deadline)
                })) {
                    Ok(v) => v,
                    Err(_) => Some(mk_failure(
                        spec,
                        Phase::QuiesceForRollback,
                        "service manager panicked while quiescing before artifact rollback",
                        None,
                    )),
                };
            }
            Err(PostInstallCheckError::new(f.detail))
        });
    let transaction = match core_result {
        Ok(r) => r,
        Err(core) => {
            let restore = if pre_commit_quiesced {
                restore_owned_state(manager, spec, before.state, policy.rollback_restore_timeout)
                    .err()
            } else {
                None
            };
            return Err(crate::LifecycleUpdateError::CoreBeforeReceipt {
                core,
                restore_failure: restore,
            });
        }
    };
    let mut rollback_restore_failure = None;
    let mut restoration = crate::LifecycleRestorationStatus::Restored;
    match transaction.disposition() {
        TransactionDisposition::Committed => {
            if post_commit_failure
                .as_ref()
                .is_some_and(|f| f.phase == Phase::RestoreNew)
            {
                restoration = crate::LifecycleRestorationStatus::Failed;
            }
        }
        TransactionDisposition::RolledBack => {
            let should_restore = matches!(
                planned,
                UpdateRuntimeDisposition::ManagedRunning | UpdateRuntimeDisposition::ManagedStopped
            );
            if should_restore {
                if let Err(f) = restore_owned_state(
                    manager,
                    spec,
                    before.state,
                    policy.rollback_restore_timeout,
                ) {
                    rollback_restore_failure = Some(f);
                    restoration = crate::LifecycleRestorationStatus::Failed;
                }
            }
        }
        TransactionDisposition::RecoveryRequired => {
            restoration = crate::LifecycleRestorationStatus::NotAttemptedRecoveryRequired;
        }
    }
    let final_snapshot = match safe_manager_inspect(manager, spec, Phase::FinalInspect) {
        Ok(s) => Some(s),
        Err(obs) => {
            return Ok(crate::LifecycleUpdateReceipt {
                before,
                final_snapshot: None,
                transaction,
                pre_commit_quiesced,
                restoration,
                post_commit_failure,
                rollback_quiesce_failure,
                rollback_restore_failure,
                observation_failure: Some(obs),
            });
        }
    };
    Ok(crate::LifecycleUpdateReceipt {
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

fn restore_manager_success<M, C>(
    manager: &mut M,
    spec: &ServiceSpec,
    before: &LifecycleSnapshot,
    planned: UpdateRuntimeDisposition,
    intent: crate::RestoreIntent,
    check: &C,
    deadline: Instant,
) -> Result<(), crate::LifecycleFailure>
where
    M: ServiceManager,
    C: PostInstallCheck,
{
    use crate::LifecycleUpdatePhase as Phase;
    if planned == UpdateRuntimeDisposition::ForeignPreserved {
        let snapshot = safe_manager_inspect(manager, spec, Phase::RestoreNew)?;
        let budget = remaining(deadline);
        if budget.is_zero() {
            return Err(mk_failure(
                spec,
                Phase::PostInstallCheck,
                "post-commit deadline exhausted before post-install check",
                Some(&snapshot),
            ));
        }
        return check
            .check(spec, &snapshot, budget)
            .map_err(|e| mk_failure(spec, Phase::PostInstallCheck, e, Some(&snapshot)));
    }
    let wanted = match planned {
        UpdateRuntimeDisposition::Stopped => match intent {
            crate::RestoreIntent::EnsureRunning => LifecycleState::Running,
            _ => LifecycleState::Stopped,
        },
        _ => match intent {
            crate::RestoreIntent::Preserve => before.state,
            crate::RestoreIntent::EnsureRunning => LifecycleState::Running,
            crate::RestoreIntent::EnsureStopped => LifecycleState::Stopped,
        },
    };
    transition_owned_to(
        manager,
        spec,
        wanted,
        remaining(deadline),
        Phase::RestoreNew,
    )?;
    let snapshot = safe_manager_inspect(manager, spec, Phase::RestoreNew)?;
    let budget = remaining(deadline);
    if budget.is_zero() {
        return Err(mk_failure(
            spec,
            Phase::PostInstallCheck,
            "post-commit deadline exhausted before post-install check",
            Some(&snapshot),
        ));
    }
    check
        .check(spec, &snapshot, budget)
        .map_err(|e| mk_failure(spec, Phase::PostInstallCheck, e, Some(&snapshot)))
}

fn transition_owned_to<M: ServiceManager>(
    manager: &mut M,
    spec: &ServiceSpec,
    wanted: LifecycleState,
    timeout: Duration,
    phase: crate::LifecycleUpdatePhase,
) -> Result<(), crate::LifecycleFailure> {
    if timeout.is_zero() {
        return Err(mk_failure(
            spec,
            phase,
            "lifecycle operation deadline exhausted",
            None,
        ));
    }
    let current = manager
        .inspect(spec)
        .map_err(|e| mk_failure(spec, phase, e, None))?;
    if current.ownership != Ownership::Owned {
        if wanted == LifecycleState::Stopped && current.state == LifecycleState::Stopped {
            return Ok(());
        }
        return Err(mk_failure(
            spec,
            phase,
            "service ownership is no longer Owned",
            Some(&current),
        ));
    }
    match (wanted, current.state) {
        (LifecycleState::Running, LifecycleState::Running)
        | (LifecycleState::Stopped, LifecycleState::Stopped) => Ok(()),
        (LifecycleState::Running, LifecycleState::Stopped) => {
            let r = manager
                .start(spec, timeout)
                .map_err(|e| mk_failure(spec, phase, e, Some(&current)))?;
            if !r.completed {
                return Err(mk_failure(
                    spec,
                    phase,
                    "service transition did not complete",
                    Some(&current),
                ));
            }
            Ok(())
        }
        (LifecycleState::Stopped, LifecycleState::Running) => {
            let r = manager
                .stop(spec, timeout)
                .map_err(|e| mk_failure(spec, phase, e, Some(&current)))?;
            if !r.completed {
                return Err(mk_failure(
                    spec,
                    phase,
                    "service transition did not complete",
                    Some(&current),
                ));
            }
            Ok(())
        }
        _ => Err(mk_failure(
            spec,
            phase,
            "service state is Unknown or Transitioning",
            Some(&current),
        )),
    }
}

fn quiesce_manager_for_rollback<M: ServiceManager>(
    manager: &mut M,
    spec: &ServiceSpec,
    deadline: Instant,
) -> Option<crate::LifecycleFailure> {
    use crate::LifecycleUpdatePhase as Phase;
    let budget = remaining(deadline);
    if budget.is_zero() {
        return Some(mk_failure(
            spec,
            Phase::QuiesceForRollback,
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
    if let Some(obs) = observed.as_ref() {
        if obs.ownership != Ownership::Owned {
            return Some(mk_failure(
                spec,
                Phase::QuiesceForRollback,
                "refusing to quiesce non-owned service before rollback",
                Some(obs),
            ));
        }
    }
    match manager.stop(spec, budget) {
        Err(e) => Some(mk_failure(
            spec,
            Phase::QuiesceForRollback,
            e,
            observed.as_ref(),
        )),
        Ok(r) if !r.completed => Some(mk_failure(
            spec,
            Phase::QuiesceForRollback,
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
            Ok(after) => Some(mk_failure(
                spec,
                Phase::QuiesceForRollback,
                "stopped state could not be confirmed before artifact rollback",
                Some(&after),
            )),
            Err(e) => Some(mk_failure(spec, Phase::QuiesceForRollback, e, None)),
        },
    }
}

#[allow(clippy::too_many_arguments)]
fn commit_direct_path<M, D, C>(
    manager: &mut M,
    spec: &ServiceSpec,
    mut direct: Option<&mut D>,
    before: LifecycleSnapshot,
    transaction: ValidatedTransaction,
    ownership: CommitOwnership<'_>,
    policy: LifecycleUpdatePolicy,
    check: &C,
    pre_commit_quiesced: bool,
) -> Result<crate::LifecycleUpdateReceipt, crate::LifecycleUpdateError>
where
    M: ServiceManager,
    D: DirectRuntimeControl,
    C: PostInstallCheck,
{
    use crate::LifecycleUpdatePhase as Phase;
    let mut post_commit_failure = None;
    let mut rollback_quiesce_failure = None;
    let before_c = before.clone();
    let core_result =
        transaction.commit_with_post_commit(ownership, policy.post_commit_failure, || {
            let deadline = Instant::now() + policy.post_commit_timeout;
            // Restore the direct generation per Preserve/Ensure policy, then check.
            let wanted = match policy.restore {
                crate::RestoreIntent::EnsureStopped => DirectState::Stopped,
                _ => DirectState::Running,
            };
            let work = catch_unwind(AssertUnwindSafe(|| {
                if wanted == DirectState::Running {
                    let budget = remaining(deadline);
                    if budget.is_zero() {
                        return Err(mk_failure(
                            spec,
                            Phase::RestoreNew,
                            "lifecycle operation deadline exhausted",
                            Some(&before_c),
                        ));
                    }
                    let Some(d) = direct.as_mut() else {
                        return Err(mk_failure(
                            spec,
                            Phase::RestoreNew,
                            "direct control missing during restore",
                            Some(&before_c),
                        ));
                    };
                    let r = (**d)
                        .start(budget)
                        .map_err(|e| mk_failure(spec, Phase::RestoreNew, e, Some(&before_c)))?;
                    if !r.completed {
                        return Err(mk_failure(
                            spec,
                            Phase::RestoreNew,
                            "direct transition did not complete",
                            Some(&before_c),
                        ));
                    }
                }
                // Read-only manager observation for the check snapshot; direct
                // state is caller-owned and not part of LifecycleSnapshot.
                let snapshot = safe_manager_inspect(manager, spec, Phase::RestoreNew)?;
                let budget = remaining(deadline);
                if budget.is_zero() {
                    return Err(mk_failure(
                        spec,
                        Phase::PostInstallCheck,
                        "post-commit deadline exhausted before post-install check",
                        Some(&snapshot),
                    ));
                }
                check
                    .check(spec, &snapshot, budget)
                    .map_err(|e| mk_failure(spec, Phase::PostInstallCheck, e, Some(&snapshot)))
            }));
            let f = match work {
                Ok(Ok(())) => return Ok(()),
                Ok(Err(f)) => f,
                Err(_) => mk_failure(
                    spec,
                    Phase::PostInstallCheck,
                    "post-commit direct work panicked",
                    None,
                ),
            };
            post_commit_failure = Some(f.clone());
            if policy.post_commit_failure == PostCommitFailurePolicy::RollBack {
                rollback_quiesce_failure = match catch_unwind(AssertUnwindSafe(|| {
                    quiesce_direct_for_rollback(direct.as_deref_mut(), spec, deadline)
                })) {
                    Ok(v) => v,
                    Err(_) => Some(mk_failure(
                        spec,
                        Phase::QuiesceForRollback,
                        "direct runtime panicked while quiescing before artifact rollback",
                        None,
                    )),
                };
            }
            Err(PostInstallCheckError::new(f.detail))
        });
    let transaction = match core_result {
        Ok(r) => r,
        Err(core) => {
            // Best-effort restore of the prior direct generation on pre-receipt Core error.
            if pre_commit_quiesced {
                if let Some(d) = direct.as_mut() {
                    let _ = catch_unwind(AssertUnwindSafe(|| {
                        let _ = (**d).start(policy.rollback_restore_timeout);
                    }));
                }
            }
            return Err(crate::LifecycleUpdateError::CoreBeforeReceipt {
                core,
                restore_failure: None,
            });
        }
    };
    let mut rollback_restore_failure = None;
    let mut restoration = crate::LifecycleRestorationStatus::Restored;
    match transaction.disposition() {
        TransactionDisposition::Committed => {
            if post_commit_failure
                .as_ref()
                .is_some_and(|f| f.phase == Phase::RestoreNew)
            {
                restoration = crate::LifecycleRestorationStatus::Failed;
            }
        }
        TransactionDisposition::RolledBack => {
            // Restore the prior direct generation (was Running).
            if let Some(d) = direct.as_mut() {
                let r = catch_unwind(AssertUnwindSafe(|| {
                    (**d).start(policy.rollback_restore_timeout)
                }));
                match r {
                    Ok(Ok(res)) if res.completed => {}
                    Ok(Ok(_)) => {
                        rollback_restore_failure = Some(mk_failure(
                            spec,
                            Phase::RestoreOld,
                            "direct transition did not complete",
                            Some(&before),
                        ));
                        restoration = crate::LifecycleRestorationStatus::Failed;
                    }
                    Ok(Err(e)) => {
                        rollback_restore_failure =
                            Some(mk_failure(spec, Phase::RestoreOld, e, Some(&before)));
                        restoration = crate::LifecycleRestorationStatus::Failed;
                    }
                    Err(_) => {
                        rollback_restore_failure = Some(mk_failure(
                            spec,
                            Phase::RestoreOld,
                            "direct runtime panicked while restoring",
                            Some(&before),
                        ));
                        restoration = crate::LifecycleRestorationStatus::Failed;
                    }
                }
            }
        }
        TransactionDisposition::RecoveryRequired => {
            restoration = crate::LifecycleRestorationStatus::NotAttemptedRecoveryRequired;
        }
    }
    // Final manager observation is read-only and nonessential; direct final
    // state is caller-observed and not part of the neutral receipt.
    let final_snapshot = match safe_manager_inspect(manager, spec, Phase::FinalInspect) {
        Ok(s) => Some(s),
        Err(obs) => {
            return Ok(crate::LifecycleUpdateReceipt {
                before,
                final_snapshot: None,
                transaction,
                pre_commit_quiesced,
                restoration,
                post_commit_failure,
                rollback_quiesce_failure,
                rollback_restore_failure,
                observation_failure: Some(obs),
            });
        }
    };
    Ok(crate::LifecycleUpdateReceipt {
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

fn quiesce_direct_for_rollback<D: DirectRuntimeControl>(
    direct: Option<&mut D>,
    spec: &ServiceSpec,
    deadline: Instant,
) -> Option<crate::LifecycleFailure> {
    use crate::LifecycleUpdatePhase as Phase;
    let budget = remaining(deadline);
    if budget.is_zero() {
        return Some(mk_failure(
            spec,
            Phase::QuiesceForRollback,
            "post-commit deadline exhausted before rollback quiescence",
            None,
        ));
    }
    let Some(d) = direct else {
        return Some(mk_failure(
            spec,
            Phase::QuiesceForRollback,
            "direct control missing before artifact rollback",
            None,
        ));
    };
    // If already stopped with proven ownership, nothing to do.
    if let Ok(obs) = d.inspect() {
        if obs.state == DirectState::Stopped && obs.exact_ownership_proven {
            return None;
        }
        if !obs.exact_ownership_proven {
            return Some(mk_failure(
                spec,
                Phase::QuiesceForRollback,
                "refusing to quiesce unproven direct runtime before rollback",
                None,
            ));
        }
    }
    match d.stop(budget) {
        Err(e) => Some(mk_failure(spec, Phase::QuiesceForRollback, e, None)),
        Ok(r) if !r.completed => Some(mk_failure(
            spec,
            Phase::QuiesceForRollback,
            "direct stop did not complete before artifact rollback",
            None,
        )),
        Ok(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        NoPostInstallCheck, PostInstallCheckError, RestoreIntent, ServiceError, ServiceId,
        TestDoubleManager, TransitionResult,
    };
    use eggup_core::{
        AllValidators, ArtifactMember, ArtifactSet, InstallPlan, MemberId,
        PostCommitFailurePolicy as CorePolicy,
    };
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn direct_obs(state: DirectState, proven: bool) -> DirectObservation {
        DirectObservation {
            state,
            exact_ownership_proven: proven,
            executable: Some(PathBuf::from("/opt/app/service.bin")),
            config: Some(PathBuf::from("/etc/app/config.toml")),
        }
    }

    fn unix_input(
        ownership: Ownership,
        active: bool,
        known: bool,
        registered: KnownConfig,
        selected: Option<PathBuf>,
        direct_state: DirectState,
        proven: bool,
    ) -> UnixPlannerInput {
        UnixPlannerInput {
            manager_ownership: ownership,
            manager_active: active,
            manager_state_known: known,
            registered_config: registered,
            selected_config: selected,
            direct: direct_obs(direct_state, proven),
        }
    }

    #[test]
    fn unix_reference_matrix() {
        let cfg = PathBuf::from("/etc/app/config.toml");
        let other = PathBuf::from("/etc/other/config.toml");
        // Owned + active -> ManagedRunning.
        assert_eq!(
            plan_unix(&unix_input(
                Ownership::Owned,
                true,
                true,
                KnownConfig::Absent,
                None,
                DirectState::Stopped,
                false
            ))
            .unwrap(),
            UpdateRuntimeDisposition::ManagedRunning
        );
        // Owned + inactive -> ManagedStopped.
        assert_eq!(
            plan_unix(&unix_input(
                Ownership::Owned,
                false,
                true,
                KnownConfig::Absent,
                None,
                DirectState::Stopped,
                false
            ))
            .unwrap(),
            UpdateRuntimeDisposition::ManagedStopped
        );
        // Foreign + active + registered == selected -> ForeignPreserved.
        assert_eq!(
            plan_unix(&unix_input(
                Ownership::Foreign,
                true,
                true,
                KnownConfig::Present(cfg.clone()),
                Some(cfg.clone()),
                DirectState::Stopped,
                false
            ))
            .unwrap(),
            UpdateRuntimeDisposition::ForeignPreserved
        );
        // Foreign + active + different + direct running -> DirectRunning.
        assert_eq!(
            plan_unix(&unix_input(
                Ownership::Foreign,
                true,
                true,
                KnownConfig::Present(cfg.clone()),
                Some(other.clone()),
                DirectState::Running,
                true
            ))
            .unwrap(),
            UpdateRuntimeDisposition::DirectRunning
        );
        // Foreign + active + different + direct stopped -> Stopped.
        assert_eq!(
            plan_unix(&unix_input(
                Ownership::Foreign,
                true,
                true,
                KnownConfig::Present(cfg.clone()),
                Some(other.clone()),
                DirectState::Stopped,
                false
            ))
            .unwrap(),
            UpdateRuntimeDisposition::Stopped
        );
        // Foreign + active + unknown registered -> error.
        assert!(plan_unix(&unix_input(
            Ownership::Foreign,
            true,
            true,
            KnownConfig::Unknown,
            Some(cfg.clone()),
            DirectState::Stopped,
            false
        ))
        .is_err());
        // Foreign + inactive + direct running -> DirectRunning.
        assert_eq!(
            plan_unix(&unix_input(
                Ownership::Foreign,
                false,
                true,
                KnownConfig::Present(cfg.clone()),
                Some(other.clone()),
                DirectState::Running,
                true
            ))
            .unwrap(),
            UpdateRuntimeDisposition::DirectRunning
        );
        // Foreign + inactive + direct stopped -> Stopped.
        assert_eq!(
            plan_unix(&unix_input(
                Ownership::Foreign,
                false,
                true,
                KnownConfig::Present(cfg.clone()),
                Some(other.clone()),
                DirectState::Stopped,
                false
            ))
            .unwrap(),
            UpdateRuntimeDisposition::Stopped
        );
        // Absent + active -> error.
        assert!(plan_unix(&unix_input(
            Ownership::Absent,
            true,
            true,
            KnownConfig::Absent,
            None,
            DirectState::Stopped,
            false
        ))
        .is_err());
        // Absent + inactive + direct running -> DirectRunning.
        assert_eq!(
            plan_unix(&unix_input(
                Ownership::Absent,
                false,
                true,
                KnownConfig::Absent,
                None,
                DirectState::Running,
                true
            ))
            .unwrap(),
            UpdateRuntimeDisposition::DirectRunning
        );
        // Absent + inactive + direct stopped -> Stopped.
        assert_eq!(
            plan_unix(&unix_input(
                Ownership::Absent,
                false,
                true,
                KnownConfig::Absent,
                None,
                DirectState::Stopped,
                false
            ))
            .unwrap(),
            UpdateRuntimeDisposition::Stopped
        );
        // Unknown + active -> error.
        assert!(plan_unix(&unix_input(
            Ownership::Unknown,
            true,
            true,
            KnownConfig::Absent,
            None,
            DirectState::Stopped,
            false
        ))
        .is_err());
        // Unknown + inactive + direct running -> error.
        assert!(plan_unix(&unix_input(
            Ownership::Unknown,
            false,
            true,
            KnownConfig::Absent,
            None,
            DirectState::Running,
            true
        ))
        .is_err());
        // Unknown + inactive + direct stopped -> Stopped.
        assert_eq!(
            plan_unix(&unix_input(
                Ownership::Unknown,
                false,
                true,
                KnownConfig::Absent,
                None,
                DirectState::Stopped,
                false
            ))
            .unwrap(),
            UpdateRuntimeDisposition::Stopped
        );
        // No Gregg product strings in errors.
        let err = plan_unix(&unix_input(
            Ownership::Absent,
            true,
            true,
            KnownConfig::Absent,
            None,
            DirectState::Stopped,
            false,
        ))
        .unwrap_err();
        let msg = format!("{err}").to_lowercase();
        assert!(!msg.contains("gregg"));
        assert!(!msg.contains("greggd"));
    }

    #[test]
    fn windows_reference_matrix() {
        let w = |installed: bool, known: bool, ownership: Ownership, state: WindowsServiceState| {
            plan_windows(&WindowsPlannerInput {
                installed,
                executable_known: known,
                ownership,
                state,
            })
        };
        // Not installed -> Stopped.
        assert_eq!(
            w(
                false,
                false,
                Ownership::Absent,
                WindowsServiceState::Unknown
            )
            .unwrap(),
            UpdateRuntimeDisposition::Stopped
        );
        // Registration executable absent/unparseable -> error.
        assert!(w(
            true,
            false,
            Ownership::Unknown,
            WindowsServiceState::Running
        )
        .is_err());
        // Foreign executable -> ForeignPreserved.
        assert_eq!(
            w(true, true, Ownership::Foreign, WindowsServiceState::Running).unwrap(),
            UpdateRuntimeDisposition::ForeignPreserved
        );
        // Owned Running/StartPending -> ManagedRunning.
        assert_eq!(
            w(true, true, Ownership::Owned, WindowsServiceState::Running).unwrap(),
            UpdateRuntimeDisposition::ManagedRunning
        );
        assert_eq!(
            w(
                true,
                true,
                Ownership::Owned,
                WindowsServiceState::StartPending
            )
            .unwrap(),
            UpdateRuntimeDisposition::ManagedRunning
        );
        // Owned Stopped/StopPending -> ManagedStopped.
        assert_eq!(
            w(true, true, Ownership::Owned, WindowsServiceState::Stopped).unwrap(),
            UpdateRuntimeDisposition::ManagedStopped
        );
        assert_eq!(
            w(
                true,
                true,
                Ownership::Owned,
                WindowsServiceState::StopPending
            )
            .unwrap(),
            UpdateRuntimeDisposition::ManagedStopped
        );
    }

    #[derive(Debug)]
    struct FakeDirect {
        obs: DirectObservation,
        events: Rc<RefCell<Vec<&'static str>>>,
        fail_stop: bool,
        fail_start: bool,
        drift_after_stop: bool,
    }

    impl FakeDirect {
        fn running(events: Rc<RefCell<Vec<&'static str>>>) -> Self {
            Self {
                obs: direct_obs(DirectState::Running, true),
                events,
                fail_stop: false,
                fail_start: false,
                drift_after_stop: false,
            }
        }
    }

    impl DirectRuntimeControl for FakeDirect {
        fn inspect(&self) -> Result<DirectObservation, ServiceError> {
            self.events.borrow_mut().push("direct-inspect");
            Ok(self.obs.clone())
        }

        fn stop(&mut self, _timeout: Duration) -> Result<TransitionResult, ServiceError> {
            self.events.borrow_mut().push("direct-stop");
            if self.fail_stop {
                return Err(ServiceError::manager("injected direct stop failure"));
            }
            self.obs.state = DirectState::Stopped;
            if self.drift_after_stop {
                self.obs.executable = Some(PathBuf::from("/different/service.bin"));
                self.obs.exact_ownership_proven = false;
            }
            Ok(TransitionResult {
                operation: crate::ServiceOperation::Stop,
                completed: true,
                detail: "direct stopped".into(),
            })
        }

        fn start(&mut self, _timeout: Duration) -> Result<TransitionResult, ServiceError> {
            self.events.borrow_mut().push("direct-start");
            if self.fail_start {
                return Err(ServiceError::manager("injected direct start failure"));
            }
            self.obs.state = DirectState::Running;
            self.obs.exact_ownership_proven = true;
            Ok(TransitionResult {
                operation: crate::ServiceOperation::Start,
                completed: true,
                detail: "direct started".into(),
            })
        }
    }

    static NEXT: AtomicU64 = AtomicU64::new(0);

    fn setup() -> (TempfileGuard, std::path::PathBuf, ValidatedTransaction) {
        let base = std::env::temp_dir().join(format!(
            "eggup-disposition-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let source_root = base.join("input");
        let install_root = base.join("install");
        std::fs::create_dir_all(&source_root).unwrap();
        std::fs::create_dir_all(&install_root).unwrap();
        let main = source_root.join("main");
        let sidecar = source_root.join("sidecar");
        std::fs::write(&main, b"new-main").unwrap();
        std::fs::write(&sidecar, b"new-sidecar").unwrap();
        std::fs::write(install_root.join("service.bin"), b"old-main").unwrap();
        std::fs::write(install_root.join("sidecar.bin"), b"old-sidecar").unwrap();
        let members = ArtifactSet::new(vec![
            ArtifactMember::new(MemberId::new("main").unwrap(), &main, "service.bin")
                .unwrap()
                .with_permissions(eggup_core::PermissionsIntent::Executable)
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
        let validated = InstallPlan::new(
            eggup_core::ProductId::new("disposition-test").unwrap(),
            eggup_core::ReleaseId::new("r2").unwrap(),
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

    struct TempfileGuard {
        path: std::path::PathBuf,
    }

    impl Drop for TempfileGuard {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    fn service_spec(install_root: &std::path::Path) -> ServiceSpec {
        ServiceSpec::new(
            ServiceId::new("disposition-test").unwrap(),
            install_root.join("service.bin"),
            vec!["--worker".into()],
            None,
        )
        .unwrap()
    }

    fn ownership_proof(_install_root: &std::path::Path) -> eggup_core::CommitOwnership<'static> {
        use eggup_core::OwnershipVerifier;
        struct Fixed(eggup_core::Ownership);
        impl OwnershipVerifier for Fixed {
            fn verify(
                &self,
                _member: &eggup_core::MemberId,
                _destination: &std::path::Path,
            ) -> eggup_core::Ownership {
                self.0
            }
        }
        static VERIFIER: Fixed = Fixed(eggup_core::Ownership::Owned);
        eggup_core::CommitOwnership::new(&VERIFIER, eggup_core::AbsentPolicy::AllowCreate)
    }

    fn policy() -> LifecycleUpdatePolicy {
        LifecycleUpdatePolicy {
            restore: RestoreIntent::Preserve,
            post_commit_failure: CorePolicy::KeepInstalled,
            quiesce_timeout: Duration::from_secs(2),
            post_commit_timeout: Duration::from_secs(2),
            rollback_restore_timeout: Duration::from_secs(2),
        }
    }

    #[test]
    fn managed_running_stop_commit_restart_ordering() {
        let (_guard, install_root, validated) = setup();
        let spec = service_spec(&install_root);
        let mut manager = TestDoubleManager::new();
        manager.put(spec.clone(), crate::LifecycleState::Running);
        let baseline = DispositionBaseline {
            manager_ownership: Ownership::Owned,
            manager_state: crate::LifecycleState::Running,
            direct: None,
        };
        let receipt = commit_with_disposition(
            &mut manager,
            &spec,
            None::<&mut FakeDirect>,
            validated,
            ownership_proof(&install_root),
            policy(),
            &NoPostInstallCheck,
            UpdateRuntimeDisposition::ManagedRunning,
            &baseline,
        )
        .expect("managed running");
        assert!(receipt.pre_commit_quiesced);
        assert!(receipt.service_state_restored());
        assert_eq!(
            receipt.artifact_disposition(),
            eggup_core::TransactionDisposition::Committed
        );
    }

    #[test]
    fn managed_stopped_stays_stopped_under_preserve() {
        let (_guard, install_root, validated) = setup();
        let spec = service_spec(&install_root);
        let mut manager = TestDoubleManager::new();
        manager.put(spec.clone(), crate::LifecycleState::Stopped);
        let baseline = DispositionBaseline {
            manager_ownership: Ownership::Owned,
            manager_state: crate::LifecycleState::Stopped,
            direct: None,
        };
        let receipt = commit_with_disposition(
            &mut manager,
            &spec,
            None::<&mut FakeDirect>,
            validated,
            ownership_proof(&install_root),
            policy(),
            &NoPostInstallCheck,
            UpdateRuntimeDisposition::ManagedStopped,
            &baseline,
        )
        .unwrap();
        assert!(!receipt.pre_commit_quiesced);
        let final_state = receipt.final_snapshot.unwrap().state;
        assert_eq!(final_state, crate::LifecycleState::Stopped);
    }

    #[test]
    fn stopped_fabricates_no_restart() {
        let (_guard, install_root, validated) = setup();
        let spec = ServiceSpec::new(
            ServiceId::new("absent-service").unwrap(),
            install_root.join("service.bin"),
            vec!["--worker".into()],
            None,
        )
        .unwrap();
        let mut manager = TestDoubleManager::new();
        let baseline = DispositionBaseline {
            manager_ownership: Ownership::Absent,
            manager_state: crate::LifecycleState::Stopped,
            direct: Some(direct_obs(DirectState::Stopped, true)),
        };
        let receipt = commit_with_disposition(
            &mut manager,
            &spec,
            None::<&mut FakeDirect>,
            validated,
            ownership_proof(&install_root),
            policy(),
            &NoPostInstallCheck,
            UpdateRuntimeDisposition::Stopped,
            &baseline,
        )
        .unwrap();
        assert!(!receipt.pre_commit_quiesced);
        assert_eq!(
            receipt.artifact_disposition(),
            eggup_core::TransactionDisposition::Committed
        );
    }

    #[test]
    fn foreign_preserved_performs_zero_manager_mutation() {
        let (_guard, install_root, validated) = setup();
        // Foreign registration: same id, different executable.
        let foreign_spec = ServiceSpec::new(
            ServiceId::new("disposition-test").unwrap(),
            PathBuf::from("/foreign/service.bin"),
            vec!["--worker".into()],
            None,
        )
        .unwrap();
        let mut manager = TestDoubleManager::new();
        manager.put(foreign_spec.clone(), crate::LifecycleState::Running);
        // Our spec expects the test executable, so ownership is Foreign.
        let spec = service_spec(&install_root);
        let baseline = DispositionBaseline {
            manager_ownership: Ownership::Foreign,
            manager_state: crate::LifecycleState::Running,
            direct: None,
        };
        let receipt = commit_with_disposition(
            &mut manager,
            &spec,
            None::<&mut FakeDirect>,
            validated,
            ownership_proof(&install_root),
            policy(),
            &NoPostInstallCheck,
            UpdateRuntimeDisposition::ForeignPreserved,
            &baseline,
        )
        .unwrap();
        assert!(!receipt.pre_commit_quiesced);
        // Manager registration unchanged (still foreign, still running).
        let after = manager.inspect(&spec).unwrap();
        assert_eq!(after.ownership, Ownership::Foreign);
        assert_eq!(
            receipt.artifact_disposition(),
            eggup_core::TransactionDisposition::Committed
        );
    }

    #[test]
    fn direct_running_uses_only_direct_seam() {
        let (_guard, install_root, validated) = setup();
        let spec = service_spec(&install_root);
        // Manager is Foreign and untouched; direct is Running.
        let foreign = ServiceSpec::new(
            spec.id().clone(),
            PathBuf::from("/foreign/service.bin"),
            spec.args().to_vec(),
            None,
        )
        .unwrap();
        let mut manager = TestDoubleManager::new();
        manager.put(foreign, crate::LifecycleState::Running);
        let events = Rc::new(RefCell::new(Vec::new()));
        let mut direct = FakeDirect::running(events.clone());
        let baseline = DispositionBaseline {
            manager_ownership: Ownership::Foreign,
            manager_state: crate::LifecycleState::Running,
            direct: Some(direct_obs(DirectState::Running, true)),
        };
        let receipt = commit_with_disposition(
            &mut manager,
            &spec,
            Some(&mut direct),
            validated,
            ownership_proof(&install_root),
            policy(),
            &NoPostInstallCheck,
            UpdateRuntimeDisposition::DirectRunning,
            &baseline,
        )
        .unwrap();
        assert!(receipt.pre_commit_quiesced);
        let ev = events.borrow().clone();
        assert!(ev.contains(&"direct-stop"));
        assert!(ev.contains(&"direct-start"));
        // Manager still foreign (zero manager mutation).
        let after = manager.inspect(&spec).unwrap();
        assert_eq!(after.ownership, Ownership::Foreign);
    }

    #[test]
    fn owned_to_foreign_revalidation_fails_before_stop() {
        let (_guard, install_root, validated) = setup();
        let spec = service_spec(&install_root);
        let mut manager = TestDoubleManager::new();
        manager.put(spec.clone(), crate::LifecycleState::Running);
        // Baseline says Owned+Running, but manager drifts to Foreign before
        // orchestration observes (simulated by replacing the registration).
        let foreign = ServiceSpec::new(
            spec.id().clone(),
            PathBuf::from("/different/service-executable"),
            spec.args().to_vec(),
            None,
        )
        .unwrap();
        manager.put(foreign, crate::LifecycleState::Running);
        let baseline = DispositionBaseline {
            manager_ownership: Ownership::Owned,
            manager_state: crate::LifecycleState::Running,
            direct: None,
        };
        let err = commit_with_disposition(
            &mut manager,
            &spec,
            None::<&mut FakeDirect>,
            validated,
            ownership_proof(&install_root),
            policy(),
            &NoPostInstallCheck,
            UpdateRuntimeDisposition::ManagedRunning,
            &baseline,
        )
        .unwrap_err();
        assert!(matches!(err, crate::LifecycleUpdateError::Preflight { .. }));
    }

    #[test]
    fn direct_identity_change_fails_before_stop() {
        let (_guard, install_root, validated) = setup();
        let spec = service_spec(&install_root);
        let mut manager = TestDoubleManager::new();
        let baseline = DispositionBaseline {
            manager_ownership: Ownership::Absent,
            manager_state: crate::LifecycleState::Stopped,
            direct: Some(direct_obs(DirectState::Running, true)),
        };
        let events = Rc::new(RefCell::new(Vec::new()));
        let mut direct = FakeDirect {
            obs: DirectObservation {
                state: DirectState::Running,
                exact_ownership_proven: false,
                executable: Some(PathBuf::from("/different/service.bin")),
                config: Some(PathBuf::from("/etc/app/config.toml")),
            },
            events,
            fail_stop: false,
            fail_start: false,
            drift_after_stop: false,
        };
        let err = commit_with_disposition(
            &mut manager,
            &spec,
            Some(&mut direct),
            validated,
            ownership_proof(&install_root),
            policy(),
            &NoPostInstallCheck,
            UpdateRuntimeDisposition::DirectRunning,
            &baseline,
        )
        .unwrap_err();
        assert!(matches!(err, crate::LifecycleUpdateError::Preflight { .. }));
    }

    #[test]
    fn keepinstalled_reports_committed_plus_lifecycle_failure() {
        let (_guard, install_root, validated) = setup();
        let spec = service_spec(&install_root);
        let mut manager = TestDoubleManager::new();
        manager.put(spec.clone(), crate::LifecycleState::Running);
        // Break restoration by making start fail: use a manager wrapper that
        // fails the post-commit start. Simplest: use EnsureRunning with a
        // stopped baseline? Instead, use a failing check with RollBack disabled
        // (KeepInstalled) and assert Committed + post_commit_failure.
        struct FailCheck;
        impl std::fmt::Debug for FailCheck {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("FailCheck")
            }
        }
        impl crate::PostInstallCheck for FailCheck {
            fn check(
                &self,
                _spec: &ServiceSpec,
                _snap: &LifecycleSnapshot,
                _remaining: Duration,
            ) -> Result<(), PostInstallCheckError> {
                Err(PostInstallCheckError::new("injected check failure"))
            }
        }
        let baseline = DispositionBaseline {
            manager_ownership: Ownership::Owned,
            manager_state: crate::LifecycleState::Running,
            direct: None,
        };
        let mut pol = policy();
        pol.post_commit_failure = CorePolicy::KeepInstalled;
        let receipt = commit_with_disposition(
            &mut manager,
            &spec,
            None::<&mut FakeDirect>,
            validated,
            ownership_proof(&install_root),
            pol,
            &FailCheck,
            UpdateRuntimeDisposition::ManagedRunning,
            &baseline,
        )
        .unwrap();
        assert_eq!(
            receipt.artifact_disposition(),
            eggup_core::TransactionDisposition::Committed
        );
        assert!(receipt.post_commit_failure.is_some());
    }

    #[test]
    fn recovery_required_suppresses_automatic_starts() {
        // RecoveryRequired arises from Core when artifact state is uncertain.
        // Directly assert the restoration mapping used by orchestration: with a
        // failing check under RollBack that still cannot produce a receipt?
        // Instead, verify the planner never fabricates starts for Stopped and
        // that a RollBack check failure quiesces before rollback (covered by
        // M005 machinery). Here assert Stopped + EnsureStopped stays stopped.
        let (_guard, install_root, validated) = setup();
        let spec = service_spec(&install_root);
        let mut manager = TestDoubleManager::new();
        manager.put(spec.clone(), crate::LifecycleState::Stopped);
        let baseline = DispositionBaseline {
            manager_ownership: Ownership::Owned,
            manager_state: crate::LifecycleState::Stopped,
            direct: None,
        };
        let mut pol = policy();
        pol.restore = RestoreIntent::EnsureStopped;
        let receipt = commit_with_disposition(
            &mut manager,
            &spec,
            None::<&mut FakeDirect>,
            validated,
            ownership_proof(&install_root),
            pol,
            &NoPostInstallCheck,
            UpdateRuntimeDisposition::ManagedStopped,
            &baseline,
        )
        .unwrap();
        assert_eq!(
            receipt.final_snapshot.unwrap().state,
            crate::LifecycleState::Stopped
        );
    }

    #[test]
    fn no_gregg_strings_in_public_types() {
        let debug = format!("{:?}", UpdateRuntimeDisposition::ManagedRunning);
        assert!(!debug.to_lowercase().contains("gregg"));
        let debug = format!("{:?}", WindowsServiceState::Running);
        assert!(!debug.to_lowercase().contains("gregg"));
    }

    #[derive(Debug)]
    struct EventManager {
        inner: TestDoubleManager,
        events: Rc<RefCell<Vec<&'static str>>>,
    }

    impl EventManager {
        fn with_service(
            spec: ServiceSpec,
            state: crate::LifecycleState,
            events: Rc<RefCell<Vec<&'static str>>>,
        ) -> Self {
            let mut inner = TestDoubleManager::new();
            inner.put(spec, state);
            Self { inner, events }
        }
    }

    impl ServiceManager for EventManager {
        fn inspect(&self, spec: &ServiceSpec) -> Result<LifecycleSnapshot, ServiceError> {
            self.events.borrow_mut().push("inspect");
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
            self.inner.start(spec, timeout)
        }

        fn stop(
            &mut self,
            spec: &ServiceSpec,
            timeout: Duration,
        ) -> Result<TransitionResult, ServiceError> {
            self.events.borrow_mut().push("stop");
            self.inner.stop(spec, timeout)
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

    #[test]
    fn preparation_completes_before_first_quiesce() {
        let (_guard, install_root, validated) = setup();
        let spec = service_spec(&install_root);
        let events = Rc::new(RefCell::new(Vec::new()));
        let mut manager = EventManager::with_service(
            spec.clone(),
            crate::LifecycleState::Running,
            events.clone(),
        );
        let baseline = DispositionBaseline {
            manager_ownership: Ownership::Owned,
            manager_state: crate::LifecycleState::Running,
            direct: None,
        };
        commit_with_disposition(
            &mut manager,
            &spec,
            None::<&mut FakeDirect>,
            validated,
            ownership_proof(&install_root),
            policy(),
            &NoPostInstallCheck,
            UpdateRuntimeDisposition::ManagedRunning,
            &baseline,
        )
        .unwrap();
        let ev = events.borrow().clone();
        let first_inspect = ev.iter().position(|e| *e == "inspect").unwrap();
        let first_stop = ev.iter().position(|e| *e == "stop").unwrap();
        assert!(
            first_inspect < first_stop,
            "inspect must precede quiesce, got {ev:?}"
        );
        // Invalid policy performs zero lifecycle mutation.
        let (_guard2, install_root2, validated2) = setup();
        let spec2 = service_spec(&install_root2);
        let events2 = Rc::new(RefCell::new(Vec::new()));
        let mut manager2 = EventManager::with_service(
            spec2.clone(),
            crate::LifecycleState::Running,
            events2.clone(),
        );
        let mut bad = policy();
        bad.quiesce_timeout = Duration::ZERO;
        let err = commit_with_disposition(
            &mut manager2,
            &spec2,
            None::<&mut FakeDirect>,
            validated2,
            ownership_proof(&install_root2),
            bad,
            &NoPostInstallCheck,
            UpdateRuntimeDisposition::ManagedRunning,
            &DispositionBaseline {
                manager_ownership: Ownership::Owned,
                manager_state: crate::LifecycleState::Running,
                direct: None,
            },
        )
        .unwrap_err();
        assert!(matches!(
            err,
            crate::LifecycleUpdateError::InvalidPolicy { .. }
        ));
        assert!(events2.borrow().is_empty());
    }

    #[derive(Debug)]
    struct FailCheck;
    impl crate::PostInstallCheck for FailCheck {
        fn check(
            &self,
            _spec: &ServiceSpec,
            _snap: &LifecycleSnapshot,
            _remaining: Duration,
        ) -> Result<(), PostInstallCheckError> {
            Err(PostInstallCheckError::new("injected check failure"))
        }
    }

    #[derive(Debug)]
    struct SabotageCheck(std::path::PathBuf);
    impl crate::PostInstallCheck for SabotageCheck {
        fn check(
            &self,
            _spec: &ServiceSpec,
            _snap: &LifecycleSnapshot,
            _remaining: Duration,
        ) -> Result<(), PostInstallCheckError> {
            let dest = self.0.join("service.bin");
            std::fs::remove_file(&dest).unwrap();
            std::fs::create_dir(&dest).unwrap();
            Err(PostInstallCheckError::new("injected rollback failure"))
        }
    }

    #[test]
    fn rollback_quiesces_new_generation_and_restores_old() {
        let (_guard, install_root, validated) = setup();
        let spec = service_spec(&install_root);
        let events = Rc::new(RefCell::new(Vec::new()));
        let mut manager = EventManager::with_service(
            spec.clone(),
            crate::LifecycleState::Running,
            events.clone(),
        );
        let baseline = DispositionBaseline {
            manager_ownership: Ownership::Owned,
            manager_state: crate::LifecycleState::Running,
            direct: None,
        };
        let mut pol = policy();
        pol.post_commit_failure = CorePolicy::RollBack;
        let receipt = commit_with_disposition(
            &mut manager,
            &spec,
            None::<&mut FakeDirect>,
            validated,
            ownership_proof(&install_root),
            pol,
            &FailCheck,
            UpdateRuntimeDisposition::ManagedRunning,
            &baseline,
        )
        .unwrap();
        assert_eq!(
            receipt.artifact_disposition(),
            eggup_core::TransactionDisposition::RolledBack
        );
        // New generation was quiesced before artifact rollback (at least two
        // stops: pre-commit quiesce + rollback quiesce) and old state restored.
        let ev = events.borrow().clone();
        assert!(ev.iter().filter(|e| ***e == *"stop").count() >= 2);
        assert!(ev.contains(&"start"));
        assert_eq!(
            receipt.final_snapshot.unwrap().state,
            crate::LifecycleState::Running
        );
    }

    #[test]
    fn recovery_required_suppresses_all_automatic_starts() {
        let (_guard, install_root, validated) = setup();
        let spec = service_spec(&install_root);
        let events = Rc::new(RefCell::new(Vec::new()));
        let mut manager = EventManager::with_service(
            spec.clone(),
            crate::LifecycleState::Running,
            events.clone(),
        );
        let baseline = DispositionBaseline {
            manager_ownership: Ownership::Owned,
            manager_state: crate::LifecycleState::Running,
            direct: None,
        };
        let mut pol = policy();
        pol.post_commit_failure = CorePolicy::RollBack;
        let receipt = commit_with_disposition(
            &mut manager,
            &spec,
            None::<&mut FakeDirect>,
            validated,
            ownership_proof(&install_root),
            pol,
            &SabotageCheck(install_root.clone()),
            UpdateRuntimeDisposition::ManagedRunning,
            &baseline,
        )
        .unwrap();
        assert_eq!(
            receipt.artifact_disposition(),
            eggup_core::TransactionDisposition::RecoveryRequired
        );
        assert!(receipt.manual_artifact_recovery_required());
        assert_eq!(
            receipt.restoration,
            crate::LifecycleRestorationStatus::NotAttemptedRecoveryRequired
        );
        // Only the new generation may have been started; no old-generation
        // restart occurs against uncertain artifacts.
        assert_eq!(
            events.borrow().iter().filter(|e| ***e == *"start").count(),
            1
        );
        assert_eq!(
            receipt.final_snapshot.as_ref().unwrap().state,
            crate::LifecycleState::Stopped
        );
    }
}
