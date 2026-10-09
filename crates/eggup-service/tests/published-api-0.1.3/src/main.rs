#![forbid(unsafe_code)]

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

fn main() {
    let _ = classify(LifecycleState::Unknown);
}
