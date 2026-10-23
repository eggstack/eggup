//! M013 — `# Errors` / `# Panics` presence check.
//!
//! Plan §6.5 requires every public `Result`-returning item in the required
//! set to carry a `# Errors` rustdoc section, and every `FakeExecutor` method
//! that can panic to carry a `# Panics` section. The Windows SCM public
//! surface is deliberately **excluded**: service milestone `014` owns it, and
//! the two milestones must not edit the same doc line.
//!
//! The check reads the crate's own sources at compile time and asserts that
//! each required item's rustdoc block carries the section. It compiles
//! against the pre-M013 baseline and fails there by assertion, because the
//! baseline carries no `# Errors` section at all.

const LIB: &str = include_str!("../src/lib.rs");
const DISPOSITION: &str = include_str!("../src/disposition.rs");
const LIFECYCLE_UPDATE: &str = include_str!("../src/lifecycle_update.rs");

/// Collects the contiguous rustdoc block that starts at `anchor`.
///
/// Returns `None` when the anchor is absent.
fn doc_block_from(source: &str, anchor: &str) -> Option<String> {
    let start = source.find(anchor)?;
    let mut block: Vec<&str> = Vec::new();
    for line in source[start..].lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("///") {
            block.push(trimmed);
        } else {
            break;
        }
    }
    Some(block.join("\n"))
}

/// The predicate the fixture asserts: the required item carries the section.
fn carries_errors_section(source: &str, anchor: &str) -> bool {
    doc_block_from(source, anchor)
        .map(|block| block.contains("# Errors"))
        .unwrap_or(false)
}

/// The predicate the fixture asserts: the required item carries the panic
/// section.
fn carries_panics_section(source: &str, anchor: &str) -> bool {
    doc_block_from(source, anchor)
        .map(|block| block.contains("# Panics"))
        .unwrap_or(false)
}

#[test]
fn every_required_result_returning_item_carries_an_errors_section() {
    let required: &[(&str, &str, &str)] = &[
        // (label, source, first doc line of the item)
        ("ServiceId::new", LIB, "/// Creates an identifier, rejecting empty, overlong, or control-character input."),
        ("ServiceSpec::new", LIB, "/// Creates a spec, validating the id, absolute executable, and arg safety."),
        ("require_owned", LIB, "/// Ownership guard shared by all destructive operations."),
        ("SystemExecutor::with_max_output", LIB, "/// Creates a runner with an explicit per-stream bound."),
        ("atomic_write_definition", LIB, "/// Atomically writes a manager definition file."),
        ("inspect_host", LIB, "/// Inspects host manager facts without choosing policy."),
        ("cron_merge", LIB, "/// Merges one managed block idempotently, preserving unrelated bytes."),
        ("cron_remove", LIB, "/// Removes the owned managed block, preserving unrelated bytes."),
        ("SystemdInstall::new", LIB, "/// Validates install material without touching the manager."),
        ("LaunchdInstall::new", LIB, "/// Validates install material.\n"),
        ("CronManager::new", LIB, "/// Creates a cron adapter with a unique marker and exact desired block."),
        // The six ServiceManager trait methods.
        ("ServiceManager::inspect", LIB, "/// Inspects registration and lifecycle state without mutating anything."),
        ("ServiceManager::install", LIB, "/// Installs/registers. Allowed for `Absent` (creation) or `Owned` (refresh)."),
        ("ServiceManager::start", LIB, "/// Starts an `Owned` stopped service within `timeout`."),
        ("ServiceManager::stop", LIB, "/// Stops an `Owned` service within `timeout`."),
        ("ServiceManager::restart", LIB, "/// Restarts an `Owned` service within `timeout`."),
        ("ServiceManager::uninstall", LIB, "/// Removes an `Owned` registration."),
        // The disposition and lifecycle-update entry points.
        ("disposition::plan_unix", DISPOSITION, "/// Pure Unix-like manager/direct decision logic."),
        ("disposition::plan_windows", DISPOSITION, "/// Pure Windows SCM/executable decision logic including pending states."),
        ("disposition::commit_with_disposition", DISPOSITION, "/// Disposition-aware orchestration over an existing `ValidatedTransaction`."),
        ("disposition::commit_with_disposition_with_engine_policy", DISPOSITION, "/// Disposition-aware orchestration under an explicitly selected commit-engine"),
        ("LifecycleUpdatePolicy::validate", LIFECYCLE_UPDATE, "/// Rejects zero or overlong timeouts before any manager mutation."),
        ("commit_with_lifecycle", LIFECYCLE_UPDATE, "/// Quiesce an owned service, compose Core M007, and restore lifecycle state based on the artifact result."),
        ("commit_with_lifecycle_with_engine_policy", LIFECYCLE_UPDATE, "/// Quiesce an owned service under an explicitly selected commit-engine policy."),
    ];

    for (label, source, anchor) in required {
        assert!(
            doc_block_from(source, anchor).is_some(),
            "{label}: the required item was not found at its anchor"
        );
        assert!(
            carries_errors_section(source, anchor),
            "{label}: the required item carries no `# Errors` section"
        );
    }
}

#[test]
fn every_fake_executor_method_that_can_panic_carries_a_panics_section() {
    let required: &[(&str, &str)] = &[
        (
            "FakeExecutor::expect",
            "/// Scripts a successful response for the next matching call.",
        ),
        (
            "FakeExecutor::expect_error",
            "/// Scripts a failure for the next matching call.",
        ),
        (
            "FakeExecutor::calls",
            "/// Returns all observed calls in order.",
        ),
        (
            "FakeExecutor::call_count",
            "/// Returns the number of observed calls.",
        ),
        (
            "FakeExecutor::is_exhausted",
            "/// Returns true when all expectations were consumed.",
        ),
    ];

    for (label, anchor) in required {
        assert!(
            doc_block_from(LIB, anchor).is_some(),
            "{label}: the required item was not found at its anchor"
        );
        assert!(
            carries_panics_section(LIB, anchor),
            "{label}: the required item carries no `# Panics` section"
        );
    }
}

#[test]
fn at_least_one_errors_section_shows_how_to_distinguish_two_failure_kinds() {
    // Plan §6.5.4: at least one `# Errors` section must carry a fenced code
    // example that distinguishes two failure kinds without parsing text.
    let block = doc_block_from(
        LIB,
        "/// Inspects host manager facts without choosing policy.",
    )
    .expect("inspect_host is documented");
    assert!(
        block.contains("```"),
        "the example section must carry a fenced code block"
    );
    assert!(
        block.contains("manager_kind()"),
        "the example must discriminate on the typed kind"
    );
    assert!(
        block.contains("ManagerFailureKind::Timeout")
            && block.contains("ManagerFailureKind::NotFound"),
        "the example must distinguish at least two failure kinds"
    );
}

/// Negative control: an item that is *not* in the required set must fail the
/// predicate, so the check can detect a missing section.
#[test]
fn an_undocumented_item_fails_the_presence_predicate() {
    // `ServiceError::manager_kind` is a helper, not a `Result`-returning
    // item; it deliberately carries no `# Errors` section.
    assert!(
        !carries_errors_section(
            LIB,
            "/// Returns the manager failure kind, when this is a manager failure."
        ),
        "the predicate must reject an item without an `# Errors` section"
    );
    assert!(
        !carries_errors_section(LIB, "/// This anchor does not exist in the crate."),
        "the predicate must reject an absent item"
    );
}
