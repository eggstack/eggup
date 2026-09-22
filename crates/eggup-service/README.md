# eggup-service

Manager-neutral service-registration, ownership, state, and
lifecycle-snapshot model. No systemd/launchd/cron/Windows-SCM calls live
here; platform adapters implement the `ServiceManager` trait in later
milestones.

- `ServiceId` / `ServiceSpec` identify desired registrations with exact
  executable, arguments, and config evidence.
- `Ownership::{Absent, Owned, Foreign, Unknown}` reuses the canonical Eggup
  vocabulary: destructive manager operations are authorized only for `Owned`.
- `LifecycleState::{Stopped, Running, Transitioning, Unknown}` is distinct
  from application health (`HealthState::{Healthy, Degraded, Unknown}`).
- `LifecycleSnapshot` preserves pre-update intent (`was_running`,
  `was_registered`) so orchestration can restore "was running" versus "was
  registered but stopped". Stopped services remain stopped by default.
- `ServiceManager` trait plus `TestDoubleManager` for deterministic tests;
  `HealthProbe` seam is consumer-supplied and never mutates manager state.
- Bounded `TransitionResult` with conflict diagnostics; no automatic privilege
  elevation; no updater/release/network policy.
