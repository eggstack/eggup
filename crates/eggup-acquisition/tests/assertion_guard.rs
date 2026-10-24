//! M013 — assertion guards for the acquisition seam.
//!
//! Three guards, all reading crate sources at compile time so that no
//! filesystem discovery and no network is involved:
//!
//! * **pattern guard** — the crate's `src/` contains no vacuous early exit
//!   (an `else` block that returns from a test instead of asserting);
//! * **reason guard** — every ignored test in the crate records a reason;
//! * **behavior witness** — each behavior M013 requires has a named test whose
//!   body asserts something and cannot exit early.
//!
//! These live in `tests/` so the sources they scan can be copied into a
//! pristine baseline worktree; see the M013 closure record.

const CRATE_SOURCE: &str = include_str!("../src/lib.rs");
const TYPED_IO_TESTS: &str = include_str!("typed_io_contract.rs");
const PUBLICATION_COUPLING_TESTS: &str = include_str!("publication_coupling.rs");

/// The vacuous-exit needle, assembled at runtime so this guard's own source
/// never contains the literal pattern it forbids.
fn early_exit_needle() -> String {
    ["else", "{", "return"].join(" ")
}

/// The ignored-test marker, assembled at runtime for the same reason.
fn ignored_marker() -> String {
    ["#[", "ignore"].join("")
}

/// Every source file the guards read.
fn sources() -> Vec<(&'static str, &'static str)> {
    vec![
        ("src/lib.rs", CRATE_SOURCE),
        ("tests/typed_io_contract.rs", TYPED_IO_TESTS),
        ("tests/publication_coupling.rs", PUBLICATION_COUPLING_TESTS),
    ]
}

/// Extracts the body of the named function by brace matching.
fn function_body<'a>(source: &'a str, name: &str) -> Option<&'a str> {
    let signature = format!("fn {name}(");
    let start = source.find(&signature)?;
    let open = start + source[start..].find('{')?;
    let mut depth = 0usize;
    for (offset, byte) in source.as_bytes()[open..].iter().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&source[open..=open + offset]);
                }
            }
            _ => {}
        }
    }
    None
}

/// True when the named function carries a test attribute and is not shadowed by
/// another function declared after that attribute.
fn is_test_function(source: &str, name: &str) -> bool {
    let signature = format!("fn {name}(");
    let Some(start) = source.find(&signature) else {
        return false;
    };
    let head = &source[..start];
    match head.rfind("#[test]") {
        Some(index) => !head[index..].contains("fn "),
        None => false,
    }
}

/// Locates a named test and returns the file it was found in plus its body.
fn locate_test<'a>(
    sources: &[(&'static str, &'a str)],
    name: &str,
) -> Option<(&'static str, &'a str)> {
    for (file, source) in sources {
        if is_test_function(source, name) {
            if let Some(body) = function_body(source, name) {
                return Some((file, body));
            }
        }
    }
    None
}

/// The pattern guard: no vacuous early exit anywhere in the crate's sources.
#[test]
fn crate_sources_have_no_vacuous_early_exit() {
    let needle = early_exit_needle();
    for (file, source) in sources() {
        assert!(
            !source.contains(&needle),
            "{file} contains a vacuous early exit: a test returns from an `else` block \
             without asserting anything, so it reports a pass while proving nothing"
        );
    }
}

/// The reason guard: every ignored test records why it is ignored.
#[test]
fn every_ignored_test_records_a_reason() {
    let marker = ignored_marker();
    let attributed = format!("{marker} = \"");
    for (file, source) in sources() {
        let mut rest = source;
        while let Some(index) = rest.find(&marker) {
            let tail = &rest[index..];
            assert!(
                tail.starts_with(&attributed),
                "{file} contains an ignored test without a recorded reason; every ignored test \
                 must carry an explicit reason and be reported as ignored, never as passed"
            );
            rest = &rest[index + marker.len()..];
        }
    }
}

/// The behavior witness: every behavior M013 requires has a named test that
/// asserts something and cannot exit early.
#[test]
fn required_behaviors_have_named_asserting_tests() {
    const REQUIRED: &[(&str, &[&str])] = &[
        (
            "the typed I/O error preserves the underlying io error",
            &["typed_io_preserves_the_underlying_error"],
        ),
        (
            "the typed I/O detail keeps the 512-byte UTF-8-safe bound",
            &["typed_io_detail_is_bounded_and_utf8_safe"],
        ),
        (
            "the typed I/O path never surfaces URL material",
            &["typed_io_failure_does_not_surface_url_material"],
        ),
        (
            "a typed I/O failure is terminal under composition",
            &["typed_io_is_terminal_under_composition"],
        ),
        (
            "the redaction caps are byte-identical after consolidation",
            &["redaction_caps_are_byte_identical_after_consolidation"],
        ),
        (
            "the seam's changelog declares exactly one Unreleased section with the next release",
            &["seam_changelog_declares_exactly_one_unreleased_section_with_a_next_release"],
        ),
        (
            "every in-workspace dependent either accepts the declared next release or records the required widening",
            &["dependent_requirements_accept_the_next_release_or_record_the_widening"],
        ),
        (
            "the published adapters record the required widening and claim no contradicted republication version",
            &[
                "adapter_changelogs_record_the_widening_and_assert_no_contradicted_republication_version",
            ],
        ),
    ];

    let sources = sources();
    for (behavior, names) in REQUIRED {
        for name in *names {
            let (file, body) = locate_test(&sources, name)
                .unwrap_or_else(|| panic!("{behavior} has no named test `{name}`"));
            assert!(
                !body.contains("return"),
                "{file}: `{name}` can exit early, so it may report a pass while asserting nothing"
            );
            assert!(
                body.contains("assert"),
                "{file}: `{name}` contains no assertion, so it proves nothing"
            );
        }
    }
}
