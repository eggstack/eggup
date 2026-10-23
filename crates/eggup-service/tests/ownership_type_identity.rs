//! Compile-time type-identity fixture for the unified ownership vocabulary
//! (M012).
//!
//! `eggup_service::Ownership`, `eggup_service::CronOwnership`, and
//! `eggup_core::Ownership` must denote **one** type. The proof is a
//! compile-time bidirectional assignment, so a future divergence is a build
//! failure rather than a runtime assertion.
//!
//! This fixture lives in its own file because its baseline evidence is a
//! *compile* failure: at the pre-M012 baseline the service crate defined its
//! own structurally identical `Ownership` enum, which is exactly the
//! divergence this fixture rules out.

/// Bidirectional assignment between the canonical core type and the service
/// crate's re-export.
const _: () = {
    let core: eggup_core::Ownership = eggup_service::Ownership::Owned;
    let service: eggup_service::Ownership = core;
    let _ = service;
};

/// The same identity holds for the cron alias.
const _: () = {
    let cron: eggup_service::CronOwnership = eggup_service::Ownership::Foreign;
    let core: eggup_core::Ownership = cron;
    let _ = core;
};
