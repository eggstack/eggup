use eggup_service::LifecycleState;

/// Classifies a lifecycle state by name.
///
/// The helper is `pub` so that the M014 public-API surface guard
/// (`tests/public_api_surface.rs`) can fold this fixture in as a module: the
/// file is retained as a test target of its own *and* compiled as part of the
/// guard, so its `LifecycleState` variant-pinning signal is enforced by both.
pub fn classify(state: LifecycleState) -> &'static str {
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
