//! M013 — one byte cap for manager-derived free text.
//!
//! A manager error detail longer than 512 bytes must be bounded at 512 bytes
//! on a UTF-8 character boundary, including a multi-byte code point that
//! crosses the boundary.
//!
//! This file uses only the **pre-M013 public API surface**, so it compiles
//! unchanged against the pre-M013 baseline
//! (`c4805411f74b85158e0ae5d0a219a32c4209d98e`) and fails there by
//! assertion: before M013 the manager's stderr excerpt was capped at 256
//! bytes, so the composed detail carried only the first 256 bytes of the
//! manager's own output.
//!
//! The fixture observes the composed detail through `Display`, which is the
//! one rendering both the pre-M013 and the post-M013 error shapes share.

use eggup_service::{
    CommandExecutor, CommandOutput, CronManager, ServiceError, ServiceId, ServiceManager,
    ServiceSpec,
};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const MARKER: &str = "eggup-m013-bytecap";
const DESIRED: &str = "*/5 * * * * /opt/app/bin --serve";

/// The declared ceiling for manager-derived free text.
const DECLARED_CEILING: usize = 512;

/// The offset at which the multi-byte code point is planted in the manager's
/// stderr, so that it straddles the composed detail's 512-byte boundary.
const CRAB_OFFSET: usize = 490;

/// The platform's "absolute path" spelling of the fixture executable.
fn platform_exe() -> &'static str {
    if cfg!(windows) {
        r"C:\opt\app\bin.exe"
    } else {
        "/opt/app/bin"
    }
}

fn fixture_spec() -> ServiceSpec {
    ServiceSpec::new(
        ServiceId::new("eggup-m013-bytecap").unwrap(),
        PathBuf::from(platform_exe()),
        vec!["--serve".to_string()],
        None,
    )
    .unwrap()
}

/// The manager's stderr: long ASCII, a multi-byte code point straddling the
/// boundary, then more ASCII.
fn long_stderr() -> String {
    let mut out = String::with_capacity(1024);
    for _ in 0..CRAB_OFFSET {
        out.push('e');
    }
    out.push('\u{1F980}'); // four UTF-8 bytes
    while out.len() < 1000 {
        out.push('e');
    }
    out
}

/// The predicate the fixture asserts: the rendered message must be bounded at
/// the declared ceiling, and must not carry the code point that straddles the
/// boundary.
fn is_bounded_at_the_ceiling(rendered: &str) -> bool {
    rendered.len() <= DECLARED_CEILING + 64 && !rendered.contains('\u{1F980}')
}

/// A recording executor that answers `crontab -l` with an empty crontab and
/// fails `crontab -` with a long stderr.
#[derive(Debug, Clone)]
struct FailingWriteExecutor {
    stderr: String,
    calls: Arc<Mutex<Vec<Vec<String>>>>,
}

impl CommandExecutor for FailingWriteExecutor {
    fn run(
        &self,
        argv: &[String],
        _stdin_data: Option<&[u8]>,
        _timeout: Duration,
    ) -> Result<CommandOutput, ServiceError> {
        self.calls.lock().unwrap().push(argv.to_vec());
        let is_list = argv.iter().any(|a| a == "-l");
        if is_list {
            return Ok(CommandOutput {
                status: Some(0),
                stdout: Vec::new(),
                stderr: Vec::new(),
            });
        }
        Ok(CommandOutput {
            status: Some(1),
            stdout: Vec::new(),
            stderr: self.stderr.clone().into_bytes(),
        })
    }
}

#[test]
fn a_long_manager_detail_is_bounded_at_the_declared_ceiling() {
    let stderr = long_stderr();
    assert!(
        stderr.len() > DECLARED_CEILING,
        "the fixture's stderr must exceed the ceiling"
    );
    let executor = FailingWriteExecutor {
        stderr: stderr.clone(),
        calls: Arc::new(Mutex::new(Vec::new())),
    };
    let mut m = CronManager::new(
        executor,
        MARKER.to_string(),
        DESIRED.to_string(),
        Duration::from_secs(5),
    )
    .unwrap();
    let err = m
        .install(&fixture_spec())
        .expect_err("a failing crontab write must fail");
    let rendered = format!("{err}");

    // The detail is bounded: the manager's whole stderr is not carried.
    assert!(
        !rendered.contains(&stderr),
        "the composed detail was not bounded at {DECLARED_CEILING} bytes"
    );
    // The truncation landed on a UTF-8 character boundary: the multi-byte
    // code point that straddles the boundary is absent.
    assert!(
        !rendered.contains('\u{1F980}'),
        "the truncation split a multi-byte code point: {rendered}"
    );
    // The manager's own output is carried up to the declared ceiling, not up
    // to a smaller, second cap.
    assert!(
        rendered.contains(&stderr[0..300]),
        "the composed detail carried fewer than 300 bytes of the manager's output"
    );
    assert!(
        is_bounded_at_the_ceiling(&rendered),
        "the composed detail exceeded the declared ceiling"
    );
}

/// Negative control: an over-long detail must fail the fixture's own
/// predicate, so the fixture can detect a missing bound.
#[test]
fn an_over_long_detail_fails_the_fixture_predicate() {
    let stderr = long_stderr();
    assert!(
        !is_bounded_at_the_ceiling(&stderr),
        "the fixture predicate must reject an unbounded detail"
    );
    // And a bounded detail must pass it.
    let bounded: String = stderr.chars().take(64).collect();
    assert!(is_bounded_at_the_ceiling(&bounded));
}
