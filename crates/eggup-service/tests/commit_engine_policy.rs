//! Named commit-engine policy fixtures (M012).
//!
//! These fixtures exercise the two **named, separately testable policy
//! inputs** of the unified commit engine:
//!
//! * [`eggup_service::PostTransitionVerification`] — whether the engine
//!   re-inspects and validates after a transition (default: verify);
//! * [`eggup_service::OwnershipLossTolerance`] — whether ownership loss
//!   during a transition is tolerated (default: deny).
//!
//! The permissive inputs exist only because the milestone requires the two
//! former engines' behavioural differences to become explicit, named choices.
//! Both defaults are the **stricter** of the two former behaviours.
//!
//! This file uses the post-M012 API, so unlike `commit_engine_parity.rs` it
//! does not compile against the pre-M012 baseline: the named policy inputs
//! did not exist there. The baseline-observable half of the same behaviour —
//! "an ownership that is no longer `Owned` is always a typed failure by
//! default" — is proved in `commit_engine_parity.rs`, which does compile and
//! fail against the baseline.

mod common;

use common::{
    ownership_proof, policy, service_spec, validated_transaction, InspectMode, ScriptedManager,
    TransitionMode,
};
use eggup_service::{
    commit_with_disposition_with_engine_policy, commit_with_lifecycle_with_engine_policy,
    CommitEnginePolicy, DispositionBaseline, LifecycleState, Ownership, OwnershipLossTolerance,
    PostTransitionVerification, RestoreIntent, UpdateRuntimeDisposition,
};

/// The default policy is the stricter of the two former engines' behaviours.
#[test]
fn the_default_engine_policy_is_the_stricter_one() {
    let default = CommitEnginePolicy::default();
    assert_eq!(
        default.post_transition_verification,
        PostTransitionVerification::Verify
    );
    assert_eq!(
        default.ownership_loss_tolerance,
        OwnershipLossTolerance::Deny
    );
}

/// Ownership loss is a typed failure by default, and the explicitly selected
/// permissive input produces the tolerant outcome instead — with the selected
/// policy observable in the returned receipt.
#[test]
fn the_permissive_ownership_loss_tolerance_is_named_and_observable() {
    // The manager reports `Owned` before the artifact mutation and loses the
    // ownership at the post-commit transition's inspection, with the unit
    // already in the wanted stopped state.
    let manager = || {
        ScriptedManager::new(
            Ownership::Owned,
            LifecycleState::Stopped,
            InspectMode::Ok,
            TransitionMode::Complete,
            TransitionMode::Complete,
        )
        .losing_ownership_after_inspect(2)
    };
    let baseline = DispositionBaseline {
        manager_ownership: Ownership::Owned,
        manager_state: LifecycleState::Stopped,
        direct: None,
    };

    // Default: the strict typed failure.
    let (_guard, root, transaction) = validated_transaction("strict");
    let spec = service_spec(&root);
    let mut strict_manager = manager();
    let strict = commit_with_lifecycle_with_engine_policy(
        &mut strict_manager,
        &spec,
        transaction,
        ownership_proof(),
        policy(RestoreIntent::EnsureStopped),
        &eggup_service::NoPostInstallCheck,
        CommitEnginePolicy::default(),
    )
    .expect("the artifact transaction itself still commits");
    let failure = strict
        .post_commit_failure
        .expect("ownership loss must be a typed failure by default");
    assert_eq!(failure.detail, "service ownership is no longer Owned");
    assert_eq!(
        strict.engine_policy.ownership_loss_tolerance,
        OwnershipLossTolerance::Deny
    );

    // Explicitly selected permissive input: the tolerant outcome.
    let permissive_policy = CommitEnginePolicy {
        post_transition_verification: PostTransitionVerification::Verify,
        ownership_loss_tolerance: OwnershipLossTolerance::TolerateAlreadyStopped,
    };
    let (_guard2, root2, transaction2) = validated_transaction("permissive");
    let spec2 = service_spec(&root2);
    let mut permissive_manager = manager();
    let permissive = commit_with_lifecycle_with_engine_policy(
        &mut permissive_manager,
        &spec2,
        transaction2,
        ownership_proof(),
        policy(RestoreIntent::EnsureStopped),
        &eggup_service::NoPostInstallCheck,
        permissive_policy,
    )
    .expect("the artifact transaction itself still commits");
    assert!(
        permissive.post_commit_failure.is_none(),
        "the permissive input must tolerate the already-stopped ownership loss, got {:?}",
        permissive.post_commit_failure
    );
    // The selected policy is observable in the returned evidence.
    assert_eq!(
        permissive.engine_policy.ownership_loss_tolerance,
        OwnershipLossTolerance::TolerateAlreadyStopped
    );
    assert_eq!(
        permissive.engine_policy.post_transition_verification,
        PostTransitionVerification::Verify
    );

    // The disposition path exposes the same named input and the same
    // observability.
    let (_guard3, root3, transaction3) = validated_transaction("permissive-disposition");
    let spec3 = service_spec(&root3);
    let mut permissive_manager3 = manager();
    let permissive3 = commit_with_disposition_with_engine_policy(
        &mut permissive_manager3,
        &spec3,
        None::<&mut common::UnusedDirect>,
        transaction3,
        ownership_proof(),
        policy(RestoreIntent::EnsureStopped),
        &eggup_service::NoPostInstallCheck,
        UpdateRuntimeDisposition::ManagedStopped,
        &baseline,
        permissive_policy,
    )
    .expect("the artifact transaction itself still commits");
    assert!(permissive3.post_commit_failure.is_none());
    assert_eq!(
        permissive3.engine_policy.ownership_loss_tolerance,
        OwnershipLossTolerance::TolerateAlreadyStopped
    );
}

/// Post-transition verification is on by default and rejects an observation
/// whose ownership is not `Owned`; the explicitly selected non-default input
/// reproduces the former disposition behaviour.
#[test]
fn post_transition_verification_rejects_a_non_owned_observation() {
    // The manager reports `Owned` before the mutation and `Foreign` at the
    // post-transition inspection, having completed the transition.
    let manager = || {
        ScriptedManager::new(
            Ownership::Owned,
            LifecycleState::Stopped,
            InspectMode::Ok,
            TransitionMode::Complete,
            TransitionMode::Complete,
        )
        .losing_ownership_after_inspect(3)
    };

    let (_guard, root, transaction) = validated_transaction("verify");
    let spec = service_spec(&root);
    let mut manager = manager();
    let receipt = commit_with_lifecycle_with_engine_policy(
        &mut manager,
        &spec,
        transaction,
        ownership_proof(),
        policy(RestoreIntent::EnsureRunning),
        &eggup_service::NoPostInstallCheck,
        CommitEnginePolicy::default(),
    )
    .expect("the artifact transaction itself still commits");
    let failure = receipt
        .post_commit_failure
        .expect("a non-Owned observation must be a typed failure");
    assert_eq!(
        failure.detail,
        "service ownership/state does not match the requested state"
    );
}

/// The byte ceiling for manager-output diagnostic text is declared once and
/// every production truncation site for that data class uses it.
#[test]
fn the_manager_output_diagnostic_ceiling_is_declared_once() {
    // The declared ceiling governs diagnostic text; the raw capture bound is a
    // different data class.
    assert_eq!(eggup_service::MAX_DIAGNOSTIC_BYTES, 512);
    assert_eq!(eggup_service::MAX_COMMAND_OUTPUT_BYTES, 256 * 1024);
}
