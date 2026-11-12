//! Bounded candidate execution: the wall-clock deadline is an unconditional
//! maximum bound on the whole bounded execution.
//!
//! The discriminating fixture needs a descendant process that inherits the
//! stdout and stderr pipes and holds them open after the direct child exits.
//! It re-executes this very test binary as its helper, so it needs no shell,
//! no `cmd.exe`, and no external tool, and it runs natively on Linux, macOS,
//! and Windows. The helper picks its role from its own argv, because
//! `run_bounded` clears the inherited environment.
//!
//! The discriminating assertion is that the bounded call *returns*, not that
//! it returns quickly: the call runs on a worker thread and the parent waits
//! for it with a generous bound whose expiry is a failure. A loaded host can
//! only delay the fixed implementation; it cannot make it hang.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use eggup_core::{
    hash_file, run_bounded, ArtifactMember, ArtifactSet, CommandSpec, Error,
    ExactIdentityValidator, InstallPlan, IntegrityRequirement, MemberId, ProductId, ReleaseId,
};

/// The fixture root, used for the release handshake and the helper markers.
const HELPER_ROOT: &str = "EGGUP_CORE_CANDIDATE_HELPER_ROOT";
/// The exit code the direct child reports.
const HELPER_EXIT: &str = "EGGUP_CORE_CANDIDATE_HELPER_EXIT";
/// The bytes the writing helper emits.
const HELPER_WRITE: &str = "EGGUP_CORE_CANDIDATE_HELPER_WRITE";

/// The test name each helper role is selected by.
const DESCENDANT_TEST: &str = "candidate_deadline_helper_descendant_leaves_a_holder";
const HOLDER_TEST: &str = "candidate_deadline_helper_holder_keeps_the_pipe_open";
const SLEEPER_TEST: &str = "candidate_deadline_helper_sleeper_outlives_its_deadline";
const WRITE_TEST: &str = "candidate_deadline_helper_writes_output_and_exits";

/// The exit code the direct child of the descendant fixture reports.
const DESCENDANT_EXIT_CODE: i32 = 7;
/// The exit code the non-zero-exit fixture reports.
const NON_ZERO_EXIT_CODE: i32 = 3;

/// The deadline the descendant fixture gives the bounded execution.
const DESCENDANT_DEADLINE: Duration = Duration::from_secs(2);
/// The deadline the timeout fixtures give the bounded execution.
const SHORT_DEADLINE: Duration = Duration::from_millis(300);

/// The generous bound the parent allows a bounded call to take.
///
/// It is far above the fixture's own deadline on purpose: the discriminating
/// assertion is that the call returns at all, not that it returns quickly.
const GENEROUS_BOUND: Duration = Duration::from_secs(30);
/// The generous ceiling the timeout fixtures allow a bounded call to take.
const GENEROUS_ELAPSED_CEILING: Duration = Duration::from_secs(20);

/// How long the descendant helper may hold the pipes open.
const HOLDER_LIFETIME: Duration = Duration::from_secs(120);
/// How long the sleeper helper sleeps before it finishes on its own.
const SLEEPER_LIFETIME: Duration = Duration::from_secs(30);

const STARTED_MARKER: &str = "started";
const FINISHED_MARKER: &str = "finished";
const RELEASED_MARKER: &str = "released";
const HOLDER_STARTED_MARKER: &str = "holder-started";
const HOLDER_EXITED_MARKER: &str = "holder-exited";

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
            "eggup-core-candidate-{tag}-{nanos}-{sequence}-{}",
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

    /// Releases the descendant helper, which then lets the pipes close.
    fn release(&self) {
        let _ = fs::write(self.marker(RELEASED_MARKER), b"released");
    }

    /// Waits for the descendant helper to report that it ended.
    fn wait_for_holder_exit(&self) -> bool {
        let ceiling = Instant::now() + Duration::from_secs(10);
        while Instant::now() < ceiling {
            if self.exists(HOLDER_EXITED_MARKER) {
                return true;
            }
            thread::sleep(Duration::from_millis(10));
        }
        false
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        // Release first, so a helper that is still holding the pipes ends
        // before the directory is removed.
        self.release();
        if self.exists(HOLDER_STARTED_MARKER) {
            self.wait_for_holder_exit();
        }
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).expect("chmod");
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) {}

/// Reports whether this process was invoked as the named helper role.
fn invoked_as(helper_test: &str) -> bool {
    std::env::args().any(|argument| argument == helper_test)
}

/// The helper root, when the caller supplied one.
fn helper_root() -> Option<PathBuf> {
    std::env::var(HELPER_ROOT).ok().map(PathBuf::from)
}

/// The exit code the helper should report.
fn helper_exit_code() -> i32 {
    std::env::var(HELPER_EXIT)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0)
}

/// Runs the bounded call on a worker thread and waits for it.
///
/// The wait is generous: its expiry is the failure, because the defect this
/// fixture covers is a call that never returns at all.
fn run_bounded_within(spec: &CommandSpec, bound: Duration) -> eggup_core::CommandOutput {
    let spec = spec.clone();
    let (sender, receiver) = mpsc::channel();
    let worker = thread::spawn(move || {
        let _ = sender.send(run_bounded(&spec));
    });
    match receiver.recv_timeout(bound) {
        Ok(Ok(output)) => {
            let _ = worker.join();
            output
        }
        Ok(Err(error)) => panic!("the bounded call returned an error: {error:?}"),
        Err(_elapsed) => panic!("the bounded call did not return within {bound:?}"),
    }
}

/// Runs the writing helper with the given payload and exit code.
///
/// The helper is this very test binary, so the child prints the test harness's
/// own banner on its stdout before the payload. The banner is therefore not
/// hard-coded anywhere: a fixture that needs the child's exact stdout compares
/// two runs that differ only in the payload.
fn run_writer(fixture: &Fixture, payload: &str, exit_code: i32) -> eggup_core::CommandOutput {
    let spec = CommandSpec::new(std::env::current_exe().expect("current exe"))
        .args(["--exact", WRITE_TEST, "--quiet"])
        .environment(HELPER_ROOT, fixture.root.to_string_lossy().into_owned())
        .environment(HELPER_WRITE, payload)
        .environment(HELPER_EXIT, exit_code.to_string())
        .timeout(Duration::from_secs(10));
    run_bounded_within(&spec, GENEROUS_BOUND)
}

// ---------------------------------------------------------------------------
// Helper roles. Each one is inert unless the parent invoked this binary with
// the matching `--exact <name>` selection.
// ---------------------------------------------------------------------------

/// The direct child of the descendant fixture: it leaves a holder behind and
/// exits immediately with a known code.
#[test]
fn candidate_deadline_helper_descendant_leaves_a_holder() {
    if !invoked_as(DESCENDANT_TEST) {
        return;
    }
    let root = helper_root().expect("helper root");
    // The holder inherits this process's stdout and stderr, which are the
    // pipes the bounded runner is reading. It therefore keeps both pipes open
    // after this direct child exits.
    let mut holder = Command::new(std::env::current_exe().expect("current exe"));
    holder
        .arg("--exact")
        .arg(HOLDER_TEST)
        .arg("--nocapture")
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("spawn holder");
    let _ = fs::write(root.join(STARTED_MARKER), b"started");
    let _ = fs::write(root.join(HOLDER_STARTED_MARKER), b"started");
    // The holder deliberately outlives this process.
    drop(holder);
    std::process::exit(helper_exit_code());
}

/// The descendant: it holds the inherited pipes open until it is released.
#[test]
fn candidate_deadline_helper_holder_keeps_the_pipe_open() {
    if !invoked_as(HOLDER_TEST) {
        return;
    }
    let released = helper_root().expect("helper root").join(RELEASED_MARKER);
    let ceiling = Instant::now() + HOLDER_LIFETIME;
    while Instant::now() < ceiling {
        if released.exists() {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
    if let Some(root) = helper_root() {
        let _ = fs::write(root.join(HOLDER_EXITED_MARKER), b"exited");
    }
}

/// A candidate that outlives its own deadline.
#[test]
fn candidate_deadline_helper_sleeper_outlives_its_deadline() {
    if !invoked_as(SLEEPER_TEST) {
        return;
    }
    if let Some(root) = helper_root() {
        let _ = fs::write(root.join(STARTED_MARKER), b"started");
    }
    thread::sleep(SLEEPER_LIFETIME);
    if let Some(root) = helper_root() {
        let _ = fs::write(root.join(FINISHED_MARKER), b"finished");
    }
}

/// A candidate that writes a known string and exits with a known code.
#[test]
fn candidate_deadline_helper_writes_output_and_exits() {
    if !invoked_as(WRITE_TEST) {
        return;
    }
    let text = std::env::var(HELPER_WRITE).unwrap_or_default();
    {
        use std::io::Write as _;
        let mut stdout = std::io::stdout();
        stdout.write_all(text.as_bytes()).expect("write stdout");
        stdout.flush().expect("flush stdout");
    }
    std::process::exit(helper_exit_code());
}

// ---------------------------------------------------------------------------
// The fixtures.
// ---------------------------------------------------------------------------

/// The discriminating fixture: a descendant holds the pipes open after the
/// direct child exited, so the bounded call must still return at the deadline.
#[test]
fn candidate_deadline_descendant_holding_the_pipe_returns_at_the_deadline() {
    let fixture = Fixture::new("descendant");
    let spec = CommandSpec::new(std::env::current_exe().expect("current exe"))
        .args(["--exact", DESCENDANT_TEST, "--nocapture"])
        .environment(HELPER_ROOT, fixture.root.to_string_lossy().into_owned())
        .environment(HELPER_EXIT, DESCENDANT_EXIT_CODE.to_string())
        .timeout(DESCENDANT_DEADLINE);

    let output = run_bounded_within(&spec, GENEROUS_BOUND);

    assert!(
        fixture.exists(STARTED_MARKER),
        "the descendant holder was never spawned, so the fixture did not exercise the defect"
    );
    assert!(
        output.timed_out(),
        "the deadline must be reported: {output:?}"
    );
    assert!(
        !output.output_limited(),
        "no output bound was exceeded: {output:?}"
    );
    assert_eq!(
        output.exit_code(),
        Some(DESCENDANT_EXIT_CODE),
        "the direct child's exit code must stay observable: {output:?}"
    );
    assert!(
        !output.success(),
        "a deadline expiry is never a success: {output:?}"
    );
    assert!(
        output.stdout().is_empty(),
        "the stream the descendant still holds contributes no bytes: {output:?}"
    );
}

/// A child that outlives its deadline is killed and reaped.
#[test]
fn candidate_deadline_timeout_kills_and_reaps_the_direct_child() {
    let fixture = Fixture::new("timeout");
    let spec = CommandSpec::new(std::env::current_exe().expect("current exe"))
        .args(["--exact", SLEEPER_TEST, "--nocapture"])
        .environment(HELPER_ROOT, fixture.root.to_string_lossy().into_owned())
        .timeout(SHORT_DEADLINE);

    let started = Instant::now();
    let output = run_bounded_within(&spec, GENEROUS_BOUND);
    let elapsed = started.elapsed();

    assert!(
        fixture.exists(STARTED_MARKER),
        "the candidate never started, so the fixture did not exercise the timeout"
    );
    assert!(
        !fixture.exists(FINISHED_MARKER),
        "the candidate must not have run to completion"
    );
    assert!(
        output.timed_out(),
        "the deadline must be reported: {output:?}"
    );
    assert!(
        !output.success(),
        "a deadline expiry is never a success: {output:?}"
    );
    assert!(
        output.exit_code() != Some(0),
        "a killed child must not report success: {output:?}"
    );
    assert!(
        elapsed < GENEROUS_ELAPSED_CEILING,
        "the bounded call must return within the generous ceiling: {elapsed:?}"
    );
}

/// A stream that exceeds the output bound is reported and its bytes discarded.
#[test]
fn candidate_deadline_output_overflow_is_reported_and_discards_bytes() {
    let fixture = Fixture::new("overflow");
    let spec = CommandSpec::new(std::env::current_exe().expect("current exe"))
        .args(["--exact", WRITE_TEST, "--quiet"])
        .environment(HELPER_ROOT, fixture.root.to_string_lossy().into_owned())
        .environment(HELPER_WRITE, "0123456789")
        .environment(HELPER_EXIT, "0")
        .max_output_bytes(4)
        .timeout(Duration::from_secs(10));

    let output = run_bounded_within(&spec, GENEROUS_BOUND);

    assert!(
        output.output_limited(),
        "the output bound must be reported: {output:?}"
    );
    assert!(
        !output.timed_out(),
        "an overflow is not a deadline expiry: {output:?}"
    );
    assert!(
        !output.success(),
        "an overflow is never a success: {output:?}"
    );
    assert!(
        output.stdout().is_empty(),
        "the overflowing stream's bytes are discarded: {output:?}"
    );
    assert!(
        output.stdout().len() <= 4,
        "captured output never exceeds the bound: {output:?}"
    );
}

/// A spawn failure is an error, and names the operation that failed.
#[test]
fn candidate_deadline_spawn_failure_reports_the_operation() {
    let fixture = Fixture::new("spawn");
    let missing = fixture.root.join("no-such-candidate-program");
    let error = run_bounded(&CommandSpec::new(missing)).expect_err("the spawn must fail");
    match error {
        Error::Io { operation, .. } => assert_eq!(operation, "spawning bounded candidate"),
        other => panic!("unexpected error: {other:?}"),
    }
}

/// A non-zero exit stays distinguishable from a timeout and from an overflow.
#[test]
fn candidate_deadline_non_zero_exit_is_distinguishable() {
    let fixture = Fixture::new("non-zero");
    let output = run_writer(&fixture, "", NON_ZERO_EXIT_CODE);

    assert_eq!(
        output.exit_code(),
        Some(NON_ZERO_EXIT_CODE),
        "the exit code must be observable: {output:?}"
    );
    assert!(
        !output.timed_out(),
        "a non-zero exit is not a deadline expiry: {output:?}"
    );
    assert!(
        !output.output_limited(),
        "a non-zero exit is not an overflow: {output:?}"
    );
    assert!(
        !output.success(),
        "a non-zero exit is never a success: {output:?}"
    );
}

/// A normal exit is unchanged.
#[test]
fn candidate_deadline_normal_exit_is_unchanged() {
    let fixture = Fixture::new("normal");
    let expected = "eggup candidate identity\n";
    // The candidate is this test binary, so the child prints the test harness's
    // own banner on its stdout ahead of the payload. The banner is taken from
    // a baseline run that differs only in the payload, so the assertion stays
    // exact without hard-coding what the harness prints.
    let baseline = run_writer(&fixture, "", 0);
    let output = run_writer(&fixture, expected, 0);

    assert_eq!(output.exit_code(), Some(0), "the exit code must be zero");
    assert!(output.success(), "a clean exit is a success: {output:?}");
    assert_eq!(
        output.stdout(),
        [baseline.stdout(), expected.as_bytes()].concat(),
        "the candidate's exact stdout must be captured"
    );
    assert!(output.stderr().is_empty());
    assert!(
        !output.timed_out(),
        "a clean exit is not a timeout: {output:?}"
    );
    assert!(
        !output.output_limited(),
        "a clean exit is not an overflow: {output:?}"
    );
}

/// The validator path still fails closed when a candidate exceeds its
/// deadline.
#[test]
fn candidate_deadline_validator_fails_closed_on_a_candidate_that_overruns() {
    let fixture = Fixture::new("validator");
    // The staged member is a copy of this test binary, so the candidate is a
    // real program on every platform without a shell.
    let program = fixture.root.join("candidate-program");
    fs::copy(std::env::current_exe().expect("current exe"), &program).expect("copy test binary");
    make_executable(&program);
    let digest = hash_file(&program).expect("hash candidate");
    let member_id = MemberId::new("main").expect("member id");
    let member = ArtifactMember::new(member_id.clone(), program, "bin/main")
        .expect("member")
        .with_integrity(IntegrityRequirement::Sha256(digest));
    let install = fixture.root.join("install");
    fs::create_dir_all(&install).expect("install root");
    let verified = InstallPlan::new(
        ProductId::new("eggup").expect("product id"),
        ReleaseId::new("r1").expect("release id"),
        &install,
        ArtifactSet::single(member).expect("artifact set"),
    )
    .expect("install plan")
    .prepare()
    .expect("prepare")
    .verify_integrity()
    .expect("verify integrity");

    let validator = ExactIdentityValidator::new(member_id, "eggup candidate identity\n")
        .args(["--exact", SLEEPER_TEST, "--nocapture"])
        .timeout(SHORT_DEADLINE);

    let error = verified
        .validate(&validator)
        .expect_err("a candidate that overruns must fail closed");
    assert!(
        matches!(error, Error::CandidateExecution(_)),
        "the validator must report a candidate failure: {error:?}"
    );
}

/// After a timeout with no descendant holding the pipe, the bounded call
/// returns and the runner stays usable.
///
/// With no descendant, the killed child's pipe write end was the only holder,
/// so each reader reaches EOF and ends on its own. Core cannot join a reader
/// that was still blocked when the deadline was reached — joining it could
/// wait past the deadline — so the reclamation itself is not observable from
/// a test without a platform-specific thread counter. What is observable is
/// that the call returns and that the runner still works afterwards.
#[test]
fn candidate_deadline_timeout_returns_and_leaves_the_runner_usable() {
    let fixture = Fixture::new("readers");
    let spec = CommandSpec::new(std::env::current_exe().expect("current exe"))
        .args(["--exact", SLEEPER_TEST, "--nocapture"])
        .environment(HELPER_ROOT, fixture.root.to_string_lossy().into_owned())
        .timeout(SHORT_DEADLINE);

    let started = Instant::now();
    let output = run_bounded_within(&spec, GENEROUS_BOUND);
    let elapsed = started.elapsed();

    assert!(
        output.timed_out(),
        "the deadline must be reported: {output:?}"
    );
    assert!(
        elapsed < GENEROUS_ELAPSED_CEILING,
        "the bounded call must return within the generous ceiling: {elapsed:?}"
    );

    // The runner is still usable: a later bounded call still completes.
    let later_output = run_writer(&fixture, "later\n", 0);
    assert!(
        later_output.success(),
        "a later bounded call must still succeed: {later_output:?}"
    );
    assert!(
        later_output.stdout().ends_with(b"later\n"),
        "a later bounded call must still capture its output: {later_output:?}"
    );
}
