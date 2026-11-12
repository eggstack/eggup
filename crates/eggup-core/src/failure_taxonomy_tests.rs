//! Fixtures that pin the published `FailureCategory` taxonomy.
//!
//! The published taxonomy is only useful if every member of it describes a
//! failure that can actually occur. This module pins that property from both
//! sides:
//!
//! - the **declared** member set, read from this crate's own source, and
//! - the **producible** member set, derived empirically from the crate's real
//!   error-to-category mapping and from a real failing post-commit check.
//!
//! The two sets must be equal in both directions.
//!
//! ## Why the declared set is read from source
//!
//! Safe Rust cannot enumerate the variants of a `#[non_exhaustive]` enum
//! without naming each one, and naming a variant that a later milestone
//! removed would make this fixture fail to *compile* against the tree that
//! still has it. A compile error is not acceptable evidence for a taxonomy
//! change: it does not distinguish "the taxonomy changed" from "the fixture
//! stopped compiling for any other reason". The declared set is therefore read
//! from the source text, which keeps this fixture compilable on both sides of
//! the change and lets it *detect* the difference at runtime instead.
//!
//! The scan is corroborated from two directions so it cannot drift silently:
//! [`category_string`] is an exhaustive match the compiler checks against the
//! enum's real variant set, and [`declared_members`] is cross-checked against
//! the values that match actually returns.

use std::collections::BTreeSet;

use crate::transaction::report_for_error;
use crate::{
    AbsentPolicy, AllValidators, ArtifactMember, ArtifactSet, CommitOwnership, Error,
    ExistingAsOwnedVerifier, FailureCategory, FailurePhase, InstallPlan, IntegrityRequirement,
    MemberId, PostCommitFailurePolicy, ProductId, ReleaseId,
};

/// The category members the published taxonomy declares.
///
/// This is the set every produced category must belong to. It is a `const`
/// list of names rather than a match over the enum so that this fixture
/// compiles unchanged against a tree whose enum has a different member set.
const EXPECTED_MEMBERS: [&str; 7] = [
    "LockContention",
    "OwnershipConflict",
    "Verification",
    "Filesystem",
    "InvalidInput",
    "Injected",
    "PostCommitCheck",
];

/// The published `as_str()` string of every category in [`EXPECTED_MEMBERS`].
///
/// These strings are a published contract: a consumer may persist or
/// aggregate them, so they are pinned byte for byte.
const EXPECTED_STRINGS: [&str; 7] = [
    "lock-contention",
    "ownership-conflict",
    "verification",
    "filesystem",
    "invalid-input",
    "injected",
    "post-commit-check",
];

/// Maps a category to its published string through an exhaustive match.
///
/// The binding arm is unreachable exactly while the taxonomy has exactly the
/// seven members this milestone pins. If a member is added or removed, the
/// compiler reports it here, so the published taxonomy cannot drift away from
/// the fixture unnoticed.
#[expect(
    unreachable_patterns,
    reason = "the taxonomy has exactly the seven members pinned by this milestone"
)]
fn category_string(category: &FailureCategory) -> &'static str {
    match category {
        FailureCategory::LockContention => "lock-contention",
        FailureCategory::OwnershipConflict => "ownership-conflict",
        FailureCategory::Verification => "verification",
        FailureCategory::Filesystem => "filesystem",
        FailureCategory::InvalidInput => "invalid-input",
        FailureCategory::Injected => "injected",
        FailureCategory::PostCommitCheck => "post-commit-check",
        other => other.as_str(),
    }
}

/// Every member of the enum as this build knows it.
fn compiled_categories() -> Vec<FailureCategory> {
    vec![
        FailureCategory::LockContention,
        FailureCategory::OwnershipConflict,
        FailureCategory::Verification,
        FailureCategory::Filesystem,
        FailureCategory::InvalidInput,
        FailureCategory::Injected,
        FailureCategory::PostCommitCheck,
    ]
}

/// Reads this crate's `transaction.rs` source.
fn transaction_source() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/transaction.rs");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("reading {}: {error}", path.display()))
}

/// Extracts the variant names declared by the `FailureCategory` enum.
fn declared_member_names(source: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut inside = false;
    let mut found = false;
    for line in source.lines() {
        let trimmed = line.trim();
        if !inside {
            if trimmed.starts_with("pub enum FailureCategory {") {
                inside = true;
                found = true;
            }
            continue;
        }
        if line.trim_end() == "}" {
            break;
        }
        if trimmed.is_empty() || trimmed.starts_with("///") || trimmed.starts_with("#[") {
            continue;
        }
        let Some(name) = trimmed.strip_suffix(',') else {
            continue;
        };
        let name = name.trim();
        if !name.is_empty()
            && name.starts_with(|c: char| c.is_ascii_uppercase())
            && name.chars().all(|c| c.is_ascii_alphanumeric())
        {
            names.push(name.to_string());
        }
    }
    assert!(
        found,
        "the FailureCategory enum was not found in transaction.rs"
    );
    names
}

/// Extracts the `Self::Name => "value"` arms of `FailureCategory::as_str`.
fn declared_member_strings(source: &str) -> Vec<(String, String)> {
    let mut arms = Vec::new();
    let mut inside = false;
    for line in source.lines() {
        let trimmed = line.trim();
        if !inside {
            if trimmed.starts_with("impl FailureCategory {") {
                inside = true;
            }
            continue;
        }
        if line.trim_end() == "}" {
            break;
        }
        let Some(rest) = trimmed.strip_prefix("Self::") else {
            continue;
        };
        let Some((name, value)) = rest.split_once("=>") else {
            continue;
        };
        let value = value.trim().trim_end_matches(',').trim();
        let Some(value) = value.strip_prefix('"').and_then(|v| v.strip_suffix('"')) else {
            continue;
        };
        arms.push((name.trim().to_string(), value.to_string()));
    }
    assert!(
        arms.len() >= EXPECTED_MEMBERS.len(),
        "the FailureCategory::as_str arms were not found in transaction.rs"
    );
    arms
}

/// The categories this crate's source declares, with their published strings.
fn declared_members() -> Vec<(String, String)> {
    let source = transaction_source();
    let strings = declared_member_strings(&source);
    declared_member_names(&source)
        .into_iter()
        .map(|name| {
            let string = strings
                .iter()
                .find(|(candidate, _)| *candidate == name)
                .map(|(_, value)| value.clone())
                .unwrap_or_else(|| panic!("FailureCategory::{name} has no as_str() arm"));
            (name, string)
        })
        .collect()
}

/// Every published `Error` variant, one instance each.
///
/// The fault-harness variant is compiled only in this crate's own test
/// builds, which is exactly why it is present here and absent from a shipped
/// build.
fn every_error_variant() -> Vec<Error> {
    vec![
        Error::InvalidInput("caller supplied an invalid value".into()),
        Error::UnknownMember("member".into()),
        Error::Io {
            operation: "reading a local file",
            source: std::io::Error::other("injected io failure"),
        },
        Error::UpdateInProgress {
            lock: std::path::PathBuf::from("/tmp/eggup-mutation.lock"),
        },
        Error::DestinationConflict {
            destination: std::path::PathBuf::from("/tmp/eggup-destination"),
        },
        Error::VerificationFailed("integrity evidence did not pass".into()),
        Error::CandidateExecution("candidate execution failed".into()),
        Error::Injected("injected failure".into()),
    ]
}

/// The category a real failing post-commit check produces.
///
/// The check runs through a real transaction so the category is read from a
/// real [`crate::TransactionReceipt::post_commit_failure`] report rather than
/// asserted by hand.
fn post_commit_check_category() -> &'static str {
    use crate::test_support::InstallationRoot;

    let inputs = InstallationRoot::new().expect("inputs fixture");
    let install = InstallationRoot::new().expect("install fixture");
    let source = inputs
        .write_file("main", b"new-main")
        .expect("write source");
    let digest = crate::hash_file(&source).expect("digest the source");
    // Core never creates a live destination parent, so the fixture creates it.
    std::fs::create_dir_all(install.path().join("bin")).expect("create destination parent");
    let member = ArtifactMember::new(
        MemberId::new("main").expect("member identity"),
        source.clone(),
        "bin/main",
    )
    .expect("artifact member")
    .with_integrity(IntegrityRequirement::Sha256(digest));
    let plan = InstallPlan::new(
        ProductId::new("taxonomy").expect("product identity"),
        ReleaseId::new("r1").expect("release identity"),
        install.path(),
        ArtifactSet::single(member).expect("artifact set"),
    )
    .expect("install plan");
    let prepared = plan.prepare().expect("prepare");
    let validated = prepared
        .verify_integrity()
        .expect("verify integrity")
        .validate(&AllValidators::new())
        .expect("validate");
    let receipt = validated
        .commit_with_post_commit(
            CommitOwnership::new(&ExistingAsOwnedVerifier, AbsentPolicy::AllowCreate),
            PostCommitFailurePolicy::KeepInstalled,
            || Err::<(), &str>("post-commit check failed"),
        )
        .expect("commit with a failing post-commit check");
    receipt
        .post_commit_failure()
        .expect("a failed post-commit check is reported")
        .category()
        .as_str()
}

/// The categories this crate can actually produce, derived empirically.
///
/// One call into the error-to-category mapping per published `Error` variant,
/// plus the category of a real failing post-commit check.
fn producible_categories() -> BTreeSet<&'static str> {
    let mut categories = BTreeSet::new();
    for error in every_error_variant() {
        let report = report_for_error(FailurePhase::Ownership, None, &error);
        categories.insert(report.category().as_str());
    }
    categories.insert(post_commit_check_category());
    categories
}

#[test]
fn declared_taxonomy_has_exactly_the_published_members() {
    let declared = declared_members();
    let declared_names: Vec<&str> = declared
        .iter()
        .map(|(name, _)| name.as_str())
        .collect::<Vec<_>>();

    let unexpected: Vec<&str> = declared_names
        .iter()
        .copied()
        .filter(|name| !EXPECTED_MEMBERS.contains(name))
        .collect();
    assert!(
        unexpected.is_empty(),
        "declared category {} is not part of the published taxonomy; declared members: {declared_names:?}",
        unexpected.join(", ")
    );
    assert_eq!(
        declared_names, EXPECTED_MEMBERS,
        "the published taxonomy must declare exactly the categories that can be produced"
    );
}

#[test]
fn every_declared_category_has_a_producer() {
    let declared = declared_members();
    let producible = producible_categories();

    let unproduced: Vec<&str> = declared
        .iter()
        .filter(|(_, string)| !producible.contains(string.as_str()))
        .map(|(name, _)| name.as_str())
        .collect();
    assert!(
        unproduced.is_empty(),
        "declared category {} has no producer; declared members: {:?}",
        unproduced.join(", "),
        declared
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>()
    );

    let undeclared: Vec<&str> = producible
        .iter()
        .copied()
        .filter(|string| !declared.iter().any(|(_, candidate)| candidate == string))
        .collect();
    assert!(
        undeclared.is_empty(),
        "produced category {} is not a declared member of the published taxonomy",
        undeclared.join(", ")
    );
}

#[test]
fn category_strings_are_byte_identical() {
    let compiled: Vec<&str> = compiled_categories()
        .iter()
        .map(|category| category_string(category))
        .collect();
    assert_eq!(
        compiled, EXPECTED_STRINGS,
        "the published as_str() strings must not change"
    );
    for category in compiled_categories() {
        assert_eq!(
            category_string(&category),
            category.as_str(),
            "the exhaustive match and as_str() must agree"
        );
    }

    // The source scan must agree with the compiled values, so the declared
    // set this module reasons about is the set the crate really has.
    let declared = declared_members();
    for (name, string) in &declared {
        if let Some(expected) = compiled
            .iter()
            .zip(EXPECTED_MEMBERS.iter())
            .find(|(_, candidate)| *candidate == name)
            .map(|(value, _)| *value)
        {
            assert_eq!(
                string, expected,
                "the source-declared string for {name} must match the compiled value"
            );
        }
    }
}
