use eggup_service::LifecycleState;

fn classify(state: LifecycleState) -> &'static str {
    match state {
        LifecycleState::Stopped => "stopped",
        LifecycleState::Running => "running",
        LifecycleState::Transitioning => "transitioning",
        LifecycleState::Unknown => "unknown",
        _ => "future",
    }
}

#[test]
fn closed_lifecycle_state_match_remains_exhaustive() {
    assert_eq!(classify(LifecycleState::Unknown), "unknown");
}
