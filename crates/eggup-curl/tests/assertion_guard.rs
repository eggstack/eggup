//! M013, Defect A — assertion guards for the curl adapter.
//!
//! Before M013, nine `eggup-curl` tests returned early from an `else` block
//! when their fixture could not be constructed, so they reported a pass while
//! asserting nothing. These guards make that class of defect impossible to
//! reintroduce silently:
//!
//! * **pattern guard** — the crate's `src/` contains no vacuous early exit
//!   (an `else` block that returns from a test instead of asserting);
//! * **reason guard** — every ignored test in the crate records a reason, so
//!   an environment-conditional test is reported as ignored, never as passed;
//! * **behavior witness** — each behavior M013 requires has a named test whose
//!   body asserts something and cannot exit early.
//!
//! These live in `tests/` so the sources they scan can be copied into a
//! pristine baseline worktree; see the M013 closure record.

const CRATE_SOURCE: &str = include_str!("../src/lib.rs");

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
    vec![("src/lib.rs", CRATE_SOURCE)]
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
            "metadata success, not-found and server-error classification",
            &["metadata_success_not_found_and_server_error"],
        ),
        (
            "artifact success, not-found and server-error classification",
            &["artifact_success_not_found_and_server_error"],
        ),
        (
            "a truncated transfer is a hard failure that promotes nothing",
            &["truncated_transfer_is_hard_failure_without_promotion"],
        ),
        (
            "metadata and artifact size overflow",
            &["metadata_and_artifact_size_overflow"],
        ),
        (
            "connect and total timeouts are typed",
            &[
                "total_phase_timeout_is_typed_end_to_end",
                "connect_phase_timeout_is_typed_end_to_end",
            ],
        ),
        (
            "a sub-second total timeout kills the child promptly",
            &["sub_second_total_timeout_kills_child_promptly"],
        ),
        (
            "cancellation before spawn cleans the temporary file",
            &["cancellation_before_spawn_cleans_temp"],
        ),
        (
            "cancellation during the body stream kills and reaps the child",
            &["cancellation_during_body_stream_kills_and_reaps_child"],
        ),
        (
            "redirects are followed or rejected as configured",
            &["redirect_follow_and_reject"],
        ),
        (
            "the connect/total timeout label is proven without a subprocess",
            &["classify_curl_result_labels_connect_and_total_without_a_subprocess"],
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
