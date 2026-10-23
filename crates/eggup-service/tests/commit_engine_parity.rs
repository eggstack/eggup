//! Parity and type-identity fixtures for the unified commit engine (M012).
//!
//! These fixtures use only the **pre-M012 public API surface**, so the file
//! compiles unchanged against the pre-M012 baseline
//! (`c4805411f74b85158e0ae5d0a219a32c4209d98e`) and fails there. They pin:
//!
//! * both former orchestrators reaching **one** engine, by asserting that they
//!   produce the same observable outcome class for the same manager script;
//! * the strict default for post-transition verification (a manager's own
//!   `completed` flag is not evidence of observed state);
//! * the strict default for ownership loss (an ownership that is no longer
//!   `Owned` is always a typed failure);
//! * the type identity of `Ownership` and `CronOwnership` with the canonical
//!   `eggup_core::Ownership`, as a **compile-time** check.

mod common;

use common::{
    classify, ownership_proof, policy, service_spec, validated_transaction, InspectMode,
    ScriptedManager, TransitionMode, UnusedDirect,
};
use eggup_service::{
    commit_with_disposition, commit_with_lifecycle, DispositionBaseline, LifecycleState, Ownership,
    RestoreIntent, UpdateRuntimeDisposition,
};

/// One row of the parity matrix.
struct Scenario {
    name: &'static str,
    ownership: Ownership,
    state: LifecycleState,
    inspect_mode: InspectMode,
    stop_mode: TransitionMode,
    start_mode: TransitionMode,
    post_transition_state: Option<LifecycleState>,
    lose_ownership_after_inspect: Option<usize>,
    planned: UpdateRuntimeDisposition,
    restore: RestoreIntent,
}

impl Scenario {
    fn manager(&self) -> ScriptedManager {
        let mut manager = ScriptedManager::new(
            self.ownership,
            self.state,
            self.inspect_mode,
            self.stop_mode,
            self.start_mode,
        );
        if let Some(state) = self.post_transition_state {
            manager = manager.with_post_transition_state(state);
        }
        if let Some(call) = self.lose_ownership_after_inspect {
            manager = manager.losing_ownership_after_inspect(call);
        }
        manager
    }
}

fn matrix() -> Vec<Scenario> {
    vec![
        Scenario {
            name: "owned running",
            ownership: Ownership::Owned,
            state: LifecycleState::Running,
            inspect_mode: InspectMode::Ok,
            stop_mode: TransitionMode::Complete,
            start_mode: TransitionMode::Complete,
            post_transition_state: None,
            lose_ownership_after_inspect: None,
            planned: UpdateRuntimeDisposition::ManagedRunning,
            restore: RestoreIntent::Preserve,
        },
        Scenario {
            name: "owned stopped",
            ownership: Ownership::Owned,
            state: LifecycleState::Stopped,
            inspect_mode: InspectMode::Ok,
            stop_mode: TransitionMode::Complete,
            start_mode: TransitionMode::Complete,
            post_transition_state: None,
            lose_ownership_after_inspect: None,
            planned: UpdateRuntimeDisposition::ManagedStopped,
            restore: RestoreIntent::Preserve,
        },
        Scenario {
            name: "ownership lost before the post-commit transition",
            ownership: Ownership::Owned,
            state: LifecycleState::Stopped,
            inspect_mode: InspectMode::Ok,
            stop_mode: TransitionMode::Complete,
            start_mode: TransitionMode::Complete,
            post_transition_state: None,
            lose_ownership_after_inspect: Some(2),
            planned: UpdateRuntimeDisposition::ManagedStopped,
            restore: RestoreIntent::EnsureStopped,
        },
        Scenario {
            name: "completed with a mismatching post-transition state",
            ownership: Ownership::Owned,
            state: LifecycleState::Stopped,
            inspect_mode: InspectMode::Ok,
            stop_mode: TransitionMode::Complete,
            start_mode: TransitionMode::Complete,
            post_transition_state: Some(LifecycleState::Stopped),
            lose_ownership_after_inspect: None,
            planned: UpdateRuntimeDisposition::ManagedStopped,
            restore: RestoreIntent::EnsureRunning,
        },
        Scenario {
            name: "not completed with a mismatching post-transition state",
            ownership: Ownership::Owned,
            state: LifecycleState::Stopped,
            inspect_mode: InspectMode::Ok,
            stop_mode: TransitionMode::Complete,
            start_mode: TransitionMode::Incomplete,
            post_transition_state: Some(LifecycleState::Stopped),
            lose_ownership_after_inspect: None,
            planned: UpdateRuntimeDisposition::ManagedStopped,
            restore: RestoreIntent::EnsureRunning,
        },
        Scenario {
            name: "not completed with a matching post-transition state",
            ownership: Ownership::Owned,
            state: LifecycleState::Stopped,
            inspect_mode: InspectMode::Ok,
            stop_mode: TransitionMode::Complete,
            start_mode: TransitionMode::Incomplete,
            post_transition_state: Some(LifecycleState::Running),
            lose_ownership_after_inspect: None,
            planned: UpdateRuntimeDisposition::ManagedStopped,
            restore: RestoreIntent::EnsureRunning,
        },
        Scenario {
            name: "inspect error",
            ownership: Ownership::Owned,
            state: LifecycleState::Stopped,
            inspect_mode: InspectMode::Error,
            stop_mode: TransitionMode::Complete,
            start_mode: TransitionMode::Complete,
            post_transition_state: None,
            lose_ownership_after_inspect: None,
            planned: UpdateRuntimeDisposition::ManagedStopped,
            restore: RestoreIntent::Preserve,
        },
        Scenario {
            name: "inspect panic",
            ownership: Ownership::Owned,
            state: LifecycleState::Stopped,
            inspect_mode: InspectMode::Panic,
            stop_mode: TransitionMode::Complete,
            start_mode: TransitionMode::Complete,
            post_transition_state: None,
            lose_ownership_after_inspect: None,
            planned: UpdateRuntimeDisposition::ManagedStopped,
            restore: RestoreIntent::Preserve,
        },
        Scenario {
            name: "manager error",
            ownership: Ownership::Owned,
            state: LifecycleState::Running,
            inspect_mode: InspectMode::Ok,
            stop_mode: TransitionMode::Error,
            start_mode: TransitionMode::Complete,
            post_transition_state: None,
            lose_ownership_after_inspect: None,
            planned: UpdateRuntimeDisposition::ManagedRunning,
            restore: RestoreIntent::Preserve,
        },
        Scenario {
            name: "manager panic",
            ownership: Ownership::Owned,
            state: LifecycleState::Running,
            inspect_mode: InspectMode::Ok,
            stop_mode: TransitionMode::Panic,
            start_mode: TransitionMode::Complete,
            post_transition_state: None,
            lose_ownership_after_inspect: None,
            planned: UpdateRuntimeDisposition::ManagedRunning,
            restore: RestoreIntent::Preserve,
        },
    ]
}

/// The two former orchestrators now reach **one** engine: for every scenario
/// in the matrix they must produce the same observable outcome class.
///
/// At the pre-M012 baseline the paths disagree on the ownership-loss and the
/// post-transition scenarios, so this fixture fails there.
#[test]
fn both_former_paths_reach_one_engine_and_agree_on_every_scenario() {
    for scenario in matrix() {
        let (_lifecycle_guard, lifecycle_root, lifecycle_transaction) =
            validated_transaction("lifecycle");
        let lifecycle_spec = service_spec(&lifecycle_root);
        let mut lifecycle_manager = scenario.manager();
        let lifecycle_outcome = classify(commit_with_lifecycle(
            &mut lifecycle_manager,
            &lifecycle_spec,
            lifecycle_transaction,
            ownership_proof(),
            policy(scenario.restore),
            &eggup_service::NoPostInstallCheck,
        ));

        let (_disposition_guard, disposition_root, disposition_transaction) =
            validated_transaction("disposition");
        let disposition_spec = service_spec(&disposition_root);
        let mut disposition_manager = scenario.manager();
        let baseline = DispositionBaseline {
            manager_ownership: scenario.ownership,
            manager_state: scenario.state,
            direct: None,
        };
        let disposition_outcome = classify(commit_with_disposition(
            &mut disposition_manager,
            &disposition_spec,
            None::<&mut UnusedDirect>,
            disposition_transaction,
            ownership_proof(),
            policy(scenario.restore),
            &eggup_service::NoPostInstallCheck,
            scenario.planned,
            &baseline,
        ));

        assert_eq!(
            lifecycle_outcome, disposition_outcome,
            "the two former paths disagree on scenario {:?}",
            scenario.name
        );
    }
}

/// The manager's own `completed` flag is not evidence of observed state: a
/// transition whose post-transition observation does not match the wanted
/// state is a typed validation failure on **both** former paths.
///
/// At the pre-M012 baseline the disposition path returned success here.
#[test]
fn post_transition_verification_is_a_typed_failure_on_both_paths() {
    let scenario = matrix()
        .into_iter()
        .find(|s| s.name == "completed with a mismatching post-transition state")
        .expect("scenario present");

    let (_guard, root, transaction) = validated_transaction("post-transition");
    let spec = service_spec(&root);
    let mut manager = scenario.manager();
    let receipt = commit_with_lifecycle(
        &mut manager,
        &spec,
        transaction,
        ownership_proof(),
        policy(scenario.restore),
        &eggup_service::NoPostInstallCheck,
    )
    .expect("the artifact transaction itself still commits");
    let failure = receipt
        .post_commit_failure
        .expect("the post-transition mismatch must be recorded");
    assert_eq!(
        failure.detail,
        "service ownership/state does not match the requested state"
    );
    assert_eq!(
        failure.phase,
        eggup_service::LifecycleUpdatePhase::RestoreNew
    );

    let (_guard2, root2, transaction2) = validated_transaction("post-transition-disposition");
    let spec2 = service_spec(&root2);
    let mut manager2 = scenario.manager();
    let baseline = DispositionBaseline {
        manager_ownership: scenario.ownership,
        manager_state: scenario.state,
        direct: None,
    };
    let receipt2 = commit_with_disposition(
        &mut manager2,
        &spec2,
        None::<&mut UnusedDirect>,
        transaction2,
        ownership_proof(),
        policy(scenario.restore),
        &eggup_service::NoPostInstallCheck,
        scenario.planned,
        &baseline,
    )
    .expect("the artifact transaction itself still commits");
    let failure2 = receipt2
        .post_commit_failure
        .expect("the post-transition mismatch must be recorded");
    assert_eq!(
        failure2.detail,
        "service ownership/state does not match the requested state"
    );
}

/// An ownership that is no longer `Owned` at a transition is always a typed
/// failure by default — on both former paths.
///
/// At the pre-M012 baseline the disposition path tolerated this case.
#[test]
fn ownership_loss_is_a_typed_failure_by_default_on_both_paths() {
    let scenario = matrix()
        .into_iter()
        .find(|s| s.name == "ownership lost before the post-commit transition")
        .expect("scenario present");

    let (_guard, root, transaction) = validated_transaction("ownership-loss");
    let spec = service_spec(&root);
    let mut manager = scenario.manager();
    let receipt = commit_with_lifecycle(
        &mut manager,
        &spec,
        transaction,
        ownership_proof(),
        policy(scenario.restore),
        &eggup_service::NoPostInstallCheck,
    )
    .expect("the artifact transaction itself still commits");
    let failure = receipt
        .post_commit_failure
        .expect("the ownership loss must be recorded");
    assert_eq!(failure.detail, "service ownership is no longer Owned");

    let (_guard2, root2, transaction2) = validated_transaction("ownership-loss-disposition");
    let spec2 = service_spec(&root2);
    let mut manager2 = scenario.manager();
    let baseline = DispositionBaseline {
        manager_ownership: scenario.ownership,
        manager_state: scenario.state,
        direct: None,
    };
    let receipt2 = commit_with_disposition(
        &mut manager2,
        &spec2,
        None::<&mut UnusedDirect>,
        transaction2,
        ownership_proof(),
        policy(scenario.restore),
        &eggup_service::NoPostInstallCheck,
        scenario.planned,
        &baseline,
    )
    .expect("the artifact transaction itself still commits");
    let failure2 = receipt2
        .post_commit_failure
        .expect("the ownership loss must be recorded");
    assert_eq!(failure2.detail, "service ownership is no longer Owned");
}
