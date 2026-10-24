//! Bounded candidate execution: the bounded stdin capability.
//!
//! The fixtures re-execute this very test binary as their helper, so they need
//! no shell, no `cmd.exe`, and no external tool, and they run natively on
//! Linux, macOS, and Windows. The helper picks its role from its own argv,
//! because `run_bounded` clears the inherited environment.
//!
//! Every fixture name contains the substring `candidate`, so
//! `cargo test -p eggup-core --locked candidate` selects them all.
//!
//! The write-failure and the deadline fixtures use a payload of exactly
//! `MAX_STDIN_BYTES`, which is larger than the OS pipe buffer on every
//! supported platform, so the write cannot silently succeed against a child
//! that never reads its stdin.

use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use eggup_core::{run_bounded, CommandOutput, CommandSpec, Error, MAX_STDIN_BYTES};

/// The fixture root, used for the helper markers.
const HELPER_ROOT: &str = "EGGUP_CORE_CANDIDATE_STDIN_HELPER_ROOT";

/// The test name each helper role is selected by.
const ECHO_TEST: &str = "candidate_stdin_helper_echoes_stdin";
const LENGTH_TEST: &str = "candidate_stdin_helper_reports_stdin_length";
const IGNORE_TEST: &str = "candidate_stdin_helper_exits_without_reading_stdin";
const SLEEPER_TEST: &str = "candidate_stdin_helper_sleeper_ignores_stdin";

/// The marker the helper writes once it has started.
const STARTED_MARKER: &str = "started";

/// The deadline the deadline fixture gives the bounded execution.
const SHORT_DEADLINE: Duration = Duration::from_millis(300);
/// The deadline the other fixtures give the bounded execution.
const GENEROUS_DEADLINE: Duration = Duration::from_secs(20);

/// The generous bound the parent allows a bounded call to take.
///
/// It is far above the fixtures' own deadlines on purpose: the discriminating
/// assertion is that the call returns at all, not that it returns quickly.
const GENEROUS_BOUND: Duration = Duration::from_secs(60);
/// The generous ceiling the deadline fixture allows a bounded call to take.
const GENEROUS_ELAPSED_CEILING: Duration = Duration::from_secs(20);

/// How long the sleeper helper sleeps before it finishes on its own.
const SLEEPER_LIFETIME: Duration = Duration::from_secs(30);

/// How long the "no child was spawned" fixture waits before it checks the
/// marker.
///
/// The check is in the absence direction: a child that was spawned would write
/// the marker within a few milliseconds, so a longer wait can only make the
/// absence more convincing.
const NO_SPAWN_SETTLE: Duration = Duration::from_millis(500);

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

/// A private directory holding the fixture markers.
struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new(tag: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|elapsed| elapsed.as_nanos())
            .unwrap_or(0);
        // Tests run in parallel threads of one process, so the pid alone is
        // not unique; the sequence makes the name collision-free.
        let sequence = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "eggup-core-candidate-stdin-{tag}-{nanos}-{sequence}-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create fixture root");
        Self { root }
    }

    fn marker(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }

    fn exists(&self, name: &str) -> bool {
        self.marker(name).exists()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// Reports whether this process was invoked as the named helper role.
fn invoked_as(helper_test: &str) -> bool {
    std::env::args().any(|argument| argument == helper_test)
}

/// The helper root, when the caller supplied one.
fn helper_root() -> Option<PathBuf> {
    std::env::var(HELPER_ROOT).ok().map(PathBuf::from)
}

/// Reads this process's stdin to EOF.
///
/// The helper only reaches EOF once the bounded runner's writer thread has
/// written the whole payload and closed the pipe.
fn read_stdin_to_end() -> Vec<u8> {
    let mut buffer = Vec::new();
    std::io::stdin()
        .read_to_end(&mut buffer)
        .expect("read stdin");
    buffer
}

/// Writes bytes to this process's stdout and flushes them.
///
/// The bytes are written to the real stdout, not to the test harness's
/// captured output, so the bounded runner sees them on the stdout pipe.
fn write_stdout(bytes: &[u8]) {
    use std::io::Write as _;
    let mut stdout = std::io::stdout();
    stdout.write_all(bytes).expect("write stdout");
    stdout.flush().expect("flush stdout");
}

/// Runs the bounded call on a worker thread and waits for it.
///
/// The wait is generous: its expiry is the failure, because the defect this
/// fixture family covers is a call that never returns at all.
fn run_bounded_within(spec: &CommandSpec, bound: Duration) -> Result<CommandOutput, Error> {
    let spec = spec.clone();
    let (sender, receiver) = mpsc::channel();
    let worker = thread::spawn(move || {
        let _ = sender.send(run_bounded(&spec));
    });
    match receiver.recv_timeout(bound) {
        Ok(result) => {
            let _ = worker.join();
            result
        }
        Err(_elapsed) => panic!("the bounded call did not return within {bound:?}"),
    }
}

/// Builds a spec that runs this very test binary in the given helper role.
fn helper_spec(fixture: &Fixture, helper_test: &str, timeout: Duration) -> CommandSpec {
    CommandSpec::new(std::env::current_exe().expect("current exe"))
        .args(["--exact", helper_test, "--quiet"])
        .environment(HELPER_ROOT, fixture.root.to_string_lossy().into_owned())
        .timeout(timeout)
}

// ---------------------------------------------------------------------------
// Helper roles. Each one is inert unless the parent invoked this binary with
// the matching `--exact <name>` selection.
// ---------------------------------------------------------------------------

/// Reads all of stdin and echoes those exact bytes on stdout.
#[test]
fn candidate_stdin_helper_echoes_stdin() {
    if !invoked_as(ECHO_TEST) {
        return;
    }
    let payload = read_stdin_to_end();
    write_stdout(&payload);
    std::process::exit(0);
}

/// Reads all of stdin and reports how many bytes it read on stdout.
#[test]
fn candidate_stdin_helper_reports_stdin_length() {
    if !invoked_as(LENGTH_TEST) {
        return;
    }
    let payload = read_stdin_to_end();
    write_stdout(format!("stdin_bytes={}\n", payload.len()).as_bytes());
    std::process::exit(0);
}

/// Exits immediately, without reading its stdin at all.
#[test]
fn candidate_stdin_helper_exits_without_reading_stdin() {
    if !invoked_as(IGNORE_TEST) {
        return;
    }
    if let Some(root) = helper_root() {
        let _ = fs::write(root.join(STARTED_MARKER), b"started");
    }
    std::process::exit(0);
}

/// Sleeps past any deadline it is given, without reading its stdin.
#[test]
fn candidate_stdin_helper_sleeper_ignores_stdin() {
    if !invoked_as(SLEEPER_TEST) {
        return;
    }
    if let Some(root) = helper_root() {
        let _ = fs::write(root.join(STARTED_MARKER), b"started");
    }
    thread::sleep(SLEEPER_LIFETIME);
    std::process::exit(0);
}

// ---------------------------------------------------------------------------
// The fixtures.
// ---------------------------------------------------------------------------

/// The payload is delivered to the child, and the child's echo of it comes
/// back on stdout.
#[test]
fn candidate_stdin_payload_is_delivered_to_the_child() {
    let fixture = Fixture::new("delivery");
    // The payload contains a NUL byte and a non-ASCII byte, so an exact
    // stdout comparison also proves the payload was not transformed.
    let payload: Vec<u8> = (0_u16..4096)
        .map(|index| (index % 251) as u8)
        .chain([0_u8, 0xff, b'\n'])
        .collect();
    let spec = helper_spec(&fixture, ECHO_TEST, GENEROUS_DEADLINE).stdin_data(payload.clone());

    let output = run_bounded_within(&spec, GENEROUS_BOUND).expect("the bounded call must succeed");

    assert!(
        !output.timed_out(),
        "the deadline must not fire: {output:?}"
    );
    assert!(
        !output.output_limited(),
        "the output bound must not fire: {output:?}"
    );
    assert_eq!(
        output.exit_code(),
        Some(0),
        "the child must exit cleanly: {output:?}"
    );
    assert!(
        output.stdout().ends_with(&payload),
        "the child's echo of the payload must come back on stdout: {output:?}"
    );
}

/// Without a payload, stdin is null: the child observes EOF at once and reads
/// no bytes, and the bounded execution behaves exactly as it did before the
/// stdin capability existed.
#[test]
fn candidate_stdin_without_a_payload_leaves_stdin_null() {
    let fixture = Fixture::new("null");
    let spec = helper_spec(&fixture, LENGTH_TEST, GENEROUS_DEADLINE);

    let output = run_bounded_within(&spec, GENEROUS_BOUND).expect("the bounded call must succeed");

    assert!(
        !output.timed_out(),
        "the deadline must not fire: {output:?}"
    );
    assert!(
        !output.output_limited(),
        "the output bound must not fire: {output:?}"
    );
    assert_eq!(
        output.exit_code(),
        Some(0),
        "the child must exit cleanly: {output:?}"
    );
    assert!(
        output.success(),
        "the bounded execution must succeed: {output:?}"
    );
    assert!(
        output.stdout().ends_with(b"stdin_bytes=0\n"),
        "the child must observe EOF on a null stdin and read no bytes: {output:?}"
    );
}

/// The bound is inclusive: a payload of exactly `MAX_STDIN_BYTES` is accepted
/// and delivered in full.
#[test]
fn candidate_stdin_payload_at_the_bound_is_accepted() {
    let fixture = Fixture::new("at-bound");
    let payload = vec![b'x'; MAX_STDIN_BYTES];
    let spec = helper_spec(&fixture, LENGTH_TEST, GENEROUS_DEADLINE).stdin_data(payload);

    let output = run_bounded_within(&spec, GENEROUS_BOUND).expect("the bounded call must succeed");

    assert!(
        !output.timed_out(),
        "the deadline must not fire: {output:?}"
    );
    assert!(
        !output.output_limited(),
        "the output bound must not fire: {output:?}"
    );
    assert_eq!(
        output.exit_code(),
        Some(0),
        "the child must exit cleanly: {output:?}"
    );
    assert!(
        output
            .stdout()
            .ends_with(format!("stdin_bytes={MAX_STDIN_BYTES}\n").as_bytes()),
        "the child must read the whole payload: {output:?}"
    );
}

/// A payload larger than the bound is rejected before any child is spawned.
#[test]
fn candidate_stdin_oversized_payload_is_rejected_without_spawning() {
    let fixture = Fixture::new("oversized");
    let payload = vec![b'x'; MAX_STDIN_BYTES + 1];
    let spec = helper_spec(&fixture, IGNORE_TEST, GENEROUS_DEADLINE).stdin_data(payload);

    let error = run_bounded_within(&spec, GENEROUS_BOUND)
        .expect_err("an oversized payload must be rejected");

    match error {
        Error::InvalidInput(message) => assert_eq!(
            message, "stdin data exceeds the bounded stdin bound",
            "the rejection must name the bound"
        ),
        other => panic!("the rejection must be an invalid input: {other:?}"),
    }

    // No child may have been spawned: the helper writes its marker as its
    // first act, so an absent marker is the proof.
    thread::sleep(NO_SPAWN_SETTLE);
    assert!(
        !fixture.exists(STARTED_MARKER),
        "no child may be spawned for an oversized payload"
    );
}

/// A child that exits without reading its stdin yields the typed failure, and
/// the child is killed and reaped before the error is returned.
#[test]
fn candidate_stdin_write_failure_reports_the_operation() {
    let fixture = Fixture::new("write-failure");
    // The payload is larger than the OS pipe buffer, so the write cannot
    // succeed against a child that never reads its stdin.
    let payload = vec![b'x'; MAX_STDIN_BYTES];
    let spec = helper_spec(&fixture, IGNORE_TEST, GENEROUS_DEADLINE).stdin_data(payload);

    let error = run_bounded_within(&spec, GENEROUS_BOUND)
        .expect_err("a failed stdin write must fail closed");

    assert!(
        fixture.exists(STARTED_MARKER),
        "the child was never spawned, so the fixture did not exercise the defect"
    );
    match error {
        Error::Io { operation, source } => {
            assert_eq!(
                operation, "writing bounded stdin",
                "the operation string must classify the failure"
            );
            assert!(
                source.kind() != std::io::ErrorKind::Other,
                "the underlying I/O error must be preserved: {source:?}"
            );
        }
        other => panic!("the failure must be a typed I/O error: {other:?}"),
    }
}

/// The deadline still bounds a child that never reads a large stdin payload,
/// and the outcome is the documented one.
#[test]
fn candidate_stdin_deadline_bounds_a_child_that_never_reads_stdin() {
    let fixture = Fixture::new("deadline");
    let payload = vec![b'x'; MAX_STDIN_BYTES];
    let spec = helper_spec(&fixture, SLEEPER_TEST, SHORT_DEADLINE).stdin_data(payload);

    let started = Instant::now();
    let output = run_bounded_within(&spec, GENEROUS_BOUND).expect("the bounded call must return");
    let elapsed = started.elapsed();

    assert!(
        fixture.exists(STARTED_MARKER),
        "the child was never spawned, so the fixture did not exercise the defect"
    );
    assert!(
        output.timed_out(),
        "the deadline must be reported: {output:?}"
    );
    assert!(
        !output.output_limited(),
        "the output bound must not fire: {output:?}"
    );
    assert!(
        !output.success(),
        "a timed-out execution must not succeed: {output:?}"
    );
    assert!(
        elapsed < GENEROUS_ELAPSED_CEILING,
        "the bounded call must return within the generous ceiling: {elapsed:?}"
    );
}
