#![forbid(unsafe_code)]

//! The bounded-executor consolidation fixtures for service-lifecycle M015.
//!
//! `SystemExecutor`'s bounded runner now delegates to
//! `eggup_core::run_bounded`. These tests pin the observable outcomes that
//! delegation must preserve: the status and the bounded output of a run, the
//! delivery of the stdin payload, the pre-flight rejections, the typed spawn
//! and wait failures, the timeout, the output overflow, the nonzero exit, and
//! the argv every manager builds.
//!
//! The child program is the test binary itself, re-executed with a mode
//! argument; see `common/fixture_child.rs`. That keeps every fixture
//! portable: the Windows lane runs this whole suite.

#[path = "common/fixture_child.rs"]
mod fixture_child;

use eggup_service::{
    CommandExecutor, CronManager, FakeExecutor, LaunchdDomain, LaunchdInstall, LaunchdManager,
    ManagerCommandOutput, ManagerFailureKind, ServiceError, ServiceId, ServiceManager, ServiceSpec,
    SystemExecutor, SystemdInstall, SystemdManager, SystemdScope, MAX_CRONTAB_BYTES,
    MAX_TRANSITION_TIMEOUT,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// The fixture child's entry point.
///
/// When this process is the ordinary test run there is no fixture argument,
/// so the test does nothing. When this process was started as the fixture
/// child, the mode argument selects the behaviour and the process ends with
/// the child's exit code.
#[test]
fn eggup_fixture_child() {
    if let Some(code) = fixture_child::run_child() {
        std::process::exit(code);
    }
}

/// A generous allowance over the budget a test exercises.
///
/// The rule is the budget plus a wide margin: no assertion here depends on a
/// sub-second wall-clock window.
const ALLOWANCE: Duration = Duration::from_secs(20);

/// Builds the argv that runs the fixture child in `mode`.
fn fixture_argv(mode: &str, options: &[(&str, &str)]) -> Vec<String> {
    let mut argv = vec![fixture_child::executable().to_string_lossy().into_owned()];
    argv.extend(fixture_child::child_argv(mode, options));
    argv
}

/// A successful manager response with the given status and stdout.
fn out(status: i32, stdout: &str) -> ManagerCommandOutput {
    ManagerCommandOutput {
        status: Some(status),
        stdout: stdout.as_bytes().to_vec(),
        stderr: Vec::new(),
    }
}

/// Creates a fresh temporary directory and returns it.
fn temp_dir(prefix: &str) -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let path = std::env::temp_dir().join(format!(
        "eggup-m015-{prefix}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&path).expect("the temporary directory is created");
    path
}

/// The platform-appropriate absolute executable path used by the specs.
///
/// `ServiceSpec` requires an absolute executable, and a bare POSIX path is
/// not absolute on Windows.
fn spec_executable() -> PathBuf {
    if cfg!(windows) {
        PathBuf::from(r"C:\opt\app\bin")
    } else {
        PathBuf::from("/opt/app/bin")
    }
}

/// The spec executable as the text the manager observations carry.
fn spec_executable_text() -> String {
    spec_executable().to_string_lossy().into_owned()
}

/// Builds the representative service spec.
fn spec(id: &str) -> ServiceSpec {
    ServiceSpec::new(
        ServiceId::new(id).expect("the service id is valid"),
        spec_executable(),
        expected(&["--serve"]),
        None,
    )
    .expect("the service spec is valid")
}

/// The `systemctl show` observation for an absent unit.
fn show_absent() -> String {
    "LoadState=not-found\nActiveState=inactive\n".to_string()
}

/// The `systemctl show` observation for an exactly owned unit.
fn show_owned(active: &str) -> String {
    format!(
        "LoadState=loaded\nActiveState={active}\nExecStart={{ path={exe} ; argv[]={exe} --serve ; ignore_errors=no }}\n",
        exe = spec_executable_text()
    )
}

/// The launchd plist that matches the representative spec.
fn plist() -> Vec<u8> {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<plist version=\"1.0\">\n<dict>\n  <key>Label</key>\n  <string>com.example.daemon</string>\n  <key>ProgramArguments</key>\n  <array>\n    <string>{exe}</string>\n    <string>--serve</string>\n  </array>\n</dict>\n</plist>\n",
        exe = spec_executable_text()
    )
    .into_bytes()
}

/// The observed argv of every recorded call.
fn observed(fake: &FakeExecutor) -> Vec<Vec<String>> {
    fake.calls().into_iter().map(|call| call.argv).collect()
}

/// The golden argv vector, as the strings the executor observes.
fn expected(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| item.to_string()).collect()
}

/// A recorder that observes every argv and succeeds on every call.
///
/// `CronManager` keeps its executor private, so the cron scenario uses this
/// recorder to compare the observed argv with the golden table directly.
#[derive(Debug, Default, Clone)]
struct RecordingExecutor {
    calls: std::sync::Arc<std::sync::Mutex<Vec<Vec<String>>>>,
}

impl RecordingExecutor {
    /// The observed argv of every recorded call.
    fn observed(&self) -> Vec<Vec<String>> {
        self.calls.lock().expect("call log mutex").clone()
    }
}

impl CommandExecutor for RecordingExecutor {
    fn run(
        &self,
        argv: &[String],
        _stdin_data: Option<&[u8]>,
        _timeout: Duration,
    ) -> Result<ManagerCommandOutput, ServiceError> {
        self.calls
            .lock()
            .expect("call log mutex")
            .push(argv.to_vec());
        Ok(out(0, ""))
    }
}

/// T1 — a platform-neutral run returns the status and the bounded output.
#[test]
fn runs_a_command_and_returns_its_status_and_bounded_output() {
    let executor = SystemExecutor::new();
    let out = executor
        .run(
            &fixture_argv("emit-bytes", &[("bytes", "64")]),
            None,
            Duration::from_secs(30),
        )
        .expect("the fixture child runs");
    assert_eq!(out.status, Some(0));
    assert_eq!(fixture_child::payload(&out.stdout), vec![b'x'; 64]);
    // The captured stream stays inside the configured bound.
    assert!(out.stdout.len() <= executor.max_output_bytes);
    assert!(out.stderr.len() <= executor.max_output_bytes);
}

/// T2 — the `crontab -` shape: the stdin payload arrives byte for byte.
#[test]
fn delivers_the_stdin_payload_to_the_child() {
    let executor = SystemExecutor::new();
    let payload = b"* * * * * root /opt/app/bin --serve\n# trailing bytes\n".to_vec();
    let mut argv = fixture_argv("echo-stdin", &[]);
    // The `"-"` argument mirrors the real `crontab -` argv.
    argv.push("-".to_string());
    let out = executor
        .run(&argv, Some(&payload), Duration::from_secs(30))
        .expect("the fixture child runs");
    assert_eq!(out.status, Some(0));
    assert_eq!(fixture_child::payload(&out.stdout), payload);
}

/// The argv reaches the child literally: no shell, no word splitting.
///
/// This is the portable counterpart of the Unix-only fixture that proved the
/// same property with a POSIX path.
#[test]
fn passes_the_argv_literally_without_a_shell() {
    let executor = SystemExecutor::new();
    // A shell would split this argument on whitespace and would redirect its
    // output. Neither may happen.
    let literal = "$HOME; echo pwned > /tmp/eggup-m015-must-not-exist";
    let marker = std::path::Path::new("/tmp/eggup-m015-must-not-exist");
    let _ = std::fs::remove_file(marker);
    let mut argv = fixture_argv("print-argv", &[]);
    argv.push(literal.to_string());
    let out = executor
        .run(&argv, None, Duration::from_secs(30))
        .expect("the fixture child runs");
    assert_eq!(out.status, Some(0));
    let payload = String::from_utf8_lossy(&fixture_child::payload(&out.stdout)).into_owned();
    let lines: Vec<&str> = payload.lines().collect();
    assert!(
        lines.contains(&literal),
        "the argument must arrive literally, got {payload:?}"
    );
    assert!(!marker.exists(), "no shell may have run the argument");
}

/// T3 — the pre-flight rejections keep their messages.
#[test]
fn rejects_a_zero_timeout_an_over_range_timeout_and_an_over_bound_stdin_payload() {
    let executor = SystemExecutor::new();
    let program = fixture_child::executable().to_string_lossy().into_owned();

    let zero = executor
        .run(std::slice::from_ref(&program), None, Duration::ZERO)
        .expect_err("a zero timeout is refused");
    assert_invalid(&zero, "manager timeout out of range");

    let over_range = executor
        .run(
            std::slice::from_ref(&program),
            None,
            MAX_TRANSITION_TIMEOUT + Duration::from_secs(61),
        )
        .expect_err("an over-range timeout is refused");
    assert_invalid(&over_range, "manager timeout out of range");

    let over_bound = vec![b'x'; MAX_CRONTAB_BYTES + 1];
    let stdin = executor
        .run(
            std::slice::from_ref(&program),
            Some(&over_bound),
            Duration::from_secs(30),
        )
        .expect_err("an over-bound stdin payload is refused");
    assert_invalid(&stdin, "manager stdin exceeds bound");

    let no_program = executor
        .run(&[], None, Duration::from_secs(30))
        .expect_err("an empty argv is refused");
    assert_invalid(&no_program, "manager command has no program");

    let nul = executor
        .run(
            &[program, "bad\0argument".to_string()],
            None,
            Duration::from_secs(30),
        )
        .expect_err("a NUL byte in the argv is refused");
    assert_invalid(&nul, "manager command contains nul bytes");
}

/// T4 — a missing program is still the typed `NotFound` failure.
#[test]
fn classifies_a_missing_program_as_not_found_with_the_current_detail() {
    let executor = SystemExecutor::new();
    let missing = temp_dir("missing").join("no-such-manager-binary");
    let requested = missing.to_string_lossy().into_owned();
    let err = executor
        .run(
            std::slice::from_ref(&requested),
            None,
            Duration::from_secs(30),
        )
        .expect_err("a missing program fails closed");
    assert_eq!(err.manager_kind(), Some(ManagerFailureKind::NotFound));
    match err {
        ServiceError::Manager { detail, .. } => {
            // The pre-flight names the **requested** program.
            assert_eq!(detail, format!("manager binary missing: {requested}"));
        }
        other => panic!("expected a manager failure, got {other:?}"),
    }
}

/// T4 — the spawn-failure arm is still reached and still typed.
///
/// A file that exists and passes the pre-flight but cannot be executed makes
/// the spawn itself fail, which is the arm the pre-flight never reaches. The
/// platform decides how that failure looks, so each platform's own shape is
/// asserted: Unix refuses the spawn with "not found" when the script's
/// interpreter is missing, and Windows refuses it with a bad-image error.
#[test]
fn classifies_a_spawn_failure_as_a_typed_manager_failure() {
    let executor = SystemExecutor::new();
    let dir = temp_dir("spawn-failure");
    #[cfg(unix)]
    let program = dir.join("no-such-interpreter");
    #[cfg(windows)]
    let program = dir.join("not-a-program.exe");
    #[cfg(unix)]
    std::fs::write(&program, b"#!/nonexistent/eggup-interpreter\n").expect("the file is written");
    #[cfg(windows)]
    std::fs::write(&program, b"this is not a program image\n").expect("the file is written");
    make_executable(&program);
    let resolved = program.to_string_lossy().into_owned();
    let err = executor
        .run(
            std::slice::from_ref(&resolved),
            None,
            Duration::from_secs(30),
        )
        .expect_err("a program that cannot be executed fails closed");
    match err {
        ServiceError::Manager { kind, detail, .. } => {
            #[cfg(unix)]
            {
                assert_eq!(kind, ManagerFailureKind::NotFound);
                assert_eq!(detail, format!("manager binary missing: {resolved}"));
            }
            #[cfg(windows)]
            {
                assert_eq!(kind, ManagerFailureKind::Spawn);
                assert!(
                    detail.starts_with(&format!("manager spawn failed for {resolved}: ")),
                    "{detail}"
                );
            }
        }
        other => panic!("expected a manager failure, got {other:?}"),
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// T5 — a child that outlives the budget yields the timeout kind.
#[test]
fn a_timeout_yields_the_timeout_kind() {
    let executor = SystemExecutor::new();
    let started = Instant::now();
    let err = executor
        .run(
            &fixture_argv("sleep-ms", &[("ms", "60000")]),
            None,
            Duration::from_millis(300),
        )
        .expect_err("a child that outlives the budget fails closed");
    assert_eq!(err.manager_kind(), Some(ManagerFailureKind::Timeout));
    assert!(
        started.elapsed() < ALLOWANCE,
        "the call ended at {:?}",
        started.elapsed()
    );
}

/// T6 — an output overflow yields the protocol kind and discards the stream.
#[test]
fn an_output_overflow_yields_the_protocol_kind_and_discards_the_stream() {
    let executor = SystemExecutor::with_max_output(1024).expect("the bound is in range");
    let program = fixture_child::executable().to_string_lossy().into_owned();
    let err = executor
        .run(
            &fixture_argv("emit-bytes", &[("bytes", "8192")]),
            None,
            Duration::from_secs(30),
        )
        .expect_err("an output overflow fails closed");
    assert_eq!(err.manager_kind(), Some(ManagerFailureKind::Protocol));
    match err {
        ServiceError::Manager { detail, .. } => {
            assert_eq!(
                detail,
                format!("manager output exceeded bound for {program}")
            );
        }
        other => panic!("expected a manager failure, got {other:?}"),
    }
}

/// T7 — a nonzero exit is still `Ok` with the observed status.
#[test]
fn a_nonzero_exit_is_ok_with_the_observed_status() {
    let executor = SystemExecutor::new();
    let out = executor
        .run(
            &fixture_argv("exit-code", &[("code", "3")]),
            None,
            Duration::from_secs(30),
        )
        .expect("a nonzero exit is not an error");
    assert_eq!(out.status, Some(3));
}

/// T8 — the argv every manager builds is byte-identical.
///
/// The guard is a golden table: the argv the adapters build is compared with
/// the exact vectors below. The `FakeExecutor` refuses any argv that differs
/// from the scripted one, so a drift fails the operation as well as this
/// comparison.
#[test]
fn the_managers_argv_is_byte_identical() {
    let show = [
        "systemctl",
        "--user",
        "show",
        "my-daemon.service",
        "-p",
        "LoadState,ActiveState,ExecStart",
    ];

    // ---- systemctl ----

    // `show`.
    let fake = FakeExecutor::new();
    fake.expect(&show, out(0, &show_absent()));
    let dir = temp_dir("argv-systemd-show");
    let manager = systemd_manager(fake, dir.join("my-daemon.service"), false, false);
    manager
        .inspect(&spec("my-daemon.service"))
        .expect("the unit is inspected");
    assert_eq!(observed(manager.executor()), vec![expected(&show)]);

    // `start` and `is-active`.
    let fake = FakeExecutor::new();
    fake.expect(&show, out(0, &show_owned("inactive")));
    fake.expect(
        &["systemctl", "--user", "start", "my-daemon.service"],
        out(0, ""),
    );
    fake.expect(
        &["systemctl", "--user", "is-active", "my-daemon.service"],
        out(0, "active\n"),
    );
    let dir = temp_dir("argv-systemd-start");
    let mut manager = systemd_manager(fake, dir.join("my-daemon.service"), false, false);
    manager
        .start(&spec("my-daemon.service"), Duration::from_secs(5))
        .expect("the unit is started");
    assert_eq!(
        observed(manager.executor()),
        vec![
            expected(&show),
            vec![
                "systemctl".to_string(),
                "--user".to_string(),
                "start".to_string(),
                "my-daemon.service".to_string()
            ],
            vec![
                "systemctl".to_string(),
                "--user".to_string(),
                "is-active".to_string(),
                "my-daemon.service".to_string()
            ],
        ]
    );

    // `stop` and `is-active`.
    let fake = FakeExecutor::new();
    fake.expect(&show, out(0, &show_owned("active")));
    fake.expect(
        &["systemctl", "--user", "stop", "my-daemon.service"],
        out(0, ""),
    );
    fake.expect(
        &["systemctl", "--user", "is-active", "my-daemon.service"],
        out(3, "inactive\n"),
    );
    let dir = temp_dir("argv-systemd-stop");
    let mut manager = systemd_manager(fake, dir.join("my-daemon.service"), false, false);
    manager
        .stop(&spec("my-daemon.service"), Duration::from_secs(5))
        .expect("the unit is stopped");
    assert_eq!(
        observed(manager.executor()),
        vec![
            expected(&show),
            vec![
                "systemctl".to_string(),
                "--user".to_string(),
                "stop".to_string(),
                "my-daemon.service".to_string()
            ],
            vec![
                "systemctl".to_string(),
                "--user".to_string(),
                "is-active".to_string(),
                "my-daemon.service".to_string()
            ],
        ]
    );

    // `enable` and `daemon-reload`, through the install path.
    let fake = FakeExecutor::new();
    fake.expect(&show, out(0, &show_absent()));
    fake.expect(&["systemctl", "--user", "daemon-reload"], out(0, ""));
    fake.expect(
        &["systemctl", "--user", "enable", "my-daemon.service"],
        out(0, ""),
    );
    let dir = temp_dir("argv-systemd-install");
    let mut manager = systemd_manager(fake, dir.join("my-daemon.service"), true, true);
    manager
        .install(&spec("my-daemon.service"))
        .expect("the unit is installed");
    assert_eq!(
        observed(manager.executor()),
        vec![
            expected(&show),
            vec![
                "systemctl".to_string(),
                "--user".to_string(),
                "daemon-reload".to_string()
            ],
            vec![
                "systemctl".to_string(),
                "--user".to_string(),
                "enable".to_string(),
                "my-daemon.service".to_string()
            ],
        ]
    );

    // `disable` and `daemon-reload`, through the uninstall path.
    let fake = FakeExecutor::new();
    fake.expect(&show, out(0, &show_owned("inactive")));
    fake.expect(&show, out(0, &show_owned("inactive")));
    fake.expect(
        &["systemctl", "--user", "disable", "my-daemon.service"],
        out(0, ""),
    );
    fake.expect(&["systemctl", "--user", "daemon-reload"], out(0, ""));
    let dir = temp_dir("argv-systemd-uninstall");
    let unit = dir.join("my-daemon.service");
    std::fs::write(&unit, b"[Unit]\n").expect("the unit file is written");
    let mut manager = systemd_manager(fake, unit.clone(), true, true);
    manager
        .uninstall(&spec("my-daemon.service"))
        .expect("the unit is uninstalled");
    assert_eq!(
        observed(manager.executor()),
        vec![
            expected(&show),
            expected(&show),
            vec![
                "systemctl".to_string(),
                "--user".to_string(),
                "disable".to_string(),
                "my-daemon.service".to_string()
            ],
            vec![
                "systemctl".to_string(),
                "--user".to_string(),
                "daemon-reload".to_string()
            ],
        ]
    );
    assert!(!unit.exists());

    // ---- launchctl ----

    let list = ["launchctl", "list", "com.example.daemon"];

    // `list`.
    let fake = FakeExecutor::new();
    fake.expect(&list, out(0, "\"PID\" = 123;\n"));
    let dir = temp_dir("argv-launchd-list");
    let plist_path = dir.join("com.example.daemon.plist");
    std::fs::write(&plist_path, plist()).expect("the plist is written");
    let manager = launchd_manager(fake, plist_path.clone(), false);
    manager
        .inspect(&spec("com.example.daemon"))
        .expect("the label is inspected");
    assert_eq!(observed(manager.executor()), vec![expected(&list)]);

    // `bootstrap`.
    let dir = temp_dir("argv-launchd-bootstrap");
    let plist_path = dir.join("com.example.daemon.plist");
    let fake = FakeExecutor::new();
    fake.expect(
        &[
            "launchctl",
            "bootstrap",
            "gui/501",
            &plist_path.to_string_lossy(),
        ],
        out(0, ""),
    );
    let mut manager = launchd_manager(fake, plist_path.clone(), true);
    manager
        .install(&spec("com.example.daemon"))
        .expect("the plist is installed");
    assert_eq!(
        observed(manager.executor()),
        vec![vec![
            "launchctl".to_string(),
            "bootstrap".to_string(),
            "gui/501".to_string(),
            manager
                .install_config()
                .plist_path
                .to_string_lossy()
                .into_owned(),
        ]]
    );

    // `kickstart`.
    let fake = FakeExecutor::new();
    fake.expect(&list, out(0, "\"PID\" = -;\n"));
    fake.expect(
        &["launchctl", "kickstart", "-k", "gui/501/com.example.daemon"],
        out(0, ""),
    );
    fake.expect(&list, out(0, "\"PID\" = 42;\n"));
    let dir = temp_dir("argv-launchd-kickstart");
    let plist_path = dir.join("com.example.daemon.plist");
    std::fs::write(&plist_path, plist()).expect("the plist is written");
    let mut manager = launchd_manager(fake, plist_path.clone(), false);
    manager
        .start(&spec("com.example.daemon"), Duration::from_secs(5))
        .expect("the label is started");
    assert_eq!(
        observed(manager.executor()),
        vec![
            expected(&list),
            vec![
                "launchctl".to_string(),
                "kickstart".to_string(),
                "-k".to_string(),
                "gui/501/com.example.daemon".to_string()
            ],
            expected(&list),
        ]
    );

    // `stop`.
    let fake = FakeExecutor::new();
    fake.expect(&list, out(0, "\"PID\" = 123;\n"));
    fake.expect(&["launchctl", "stop", "com.example.daemon"], out(0, ""));
    fake.expect(&list, out(0, "\"PID\" = -;\n"));
    let dir = temp_dir("argv-launchd-stop");
    let plist_path = dir.join("com.example.daemon.plist");
    std::fs::write(&plist_path, plist()).expect("the plist is written");
    let mut manager = launchd_manager(fake, plist_path.clone(), false);
    manager
        .stop(&spec("com.example.daemon"), Duration::from_secs(5))
        .expect("the label is stopped");
    assert_eq!(
        observed(manager.executor()),
        vec![
            expected(&list),
            vec![
                "launchctl".to_string(),
                "stop".to_string(),
                "com.example.daemon".to_string()
            ],
            expected(&list),
        ]
    );

    // `bootout`.
    let dir = temp_dir("argv-launchd-bootout");
    let plist_path = dir.join("com.example.daemon.plist");
    let fake = FakeExecutor::new();
    fake.expect(&list, out(0, "\"PID\" = 123;\n"));
    fake.expect(&list, out(0, "\"PID\" = 123;\n"));
    fake.expect(
        &[
            "launchctl",
            "bootout",
            "gui/501",
            &plist_path.to_string_lossy(),
        ],
        out(0, ""),
    );
    std::fs::write(&plist_path, plist()).expect("the plist is written");
    let mut manager = launchd_manager(fake, plist_path.clone(), false);
    manager
        .uninstall(&spec("com.example.daemon"))
        .expect("the plist is uninstalled");
    assert_eq!(
        observed(manager.executor()),
        vec![
            expected(&list),
            expected(&list),
            vec![
                "launchctl".to_string(),
                "bootout".to_string(),
                "gui/501".to_string(),
                manager
                    .install_config()
                    .plist_path
                    .to_string_lossy()
                    .into_owned(),
            ],
        ]
    );

    // ---- crontab ----

    // `-l` and `-`.
    let recorder = RecordingExecutor::default();
    let observer = recorder.clone();
    let mut manager = CronManager::new(
        recorder,
        "eggup-m015-marker".to_string(),
        format!("*/5 * * * * {}", spec_executable_text()),
        Duration::from_secs(5),
    )
    .expect("the cron manager is valid");
    manager
        .install(&spec("cron-job"))
        .expect("the cron block is installed");
    assert_eq!(
        observer.observed(),
        vec![
            expected(&["crontab", "-l"]),
            expected(&["crontab", "-l"]),
            expected(&["crontab", "-"]),
        ]
    );
}

/// T10 — the pre-flight rejections are service errors, and no child starts.
#[test]
fn the_pre_flight_rejections_are_service_errors_not_core_errors() {
    let executor = SystemExecutor::new();
    let dir = temp_dir("preflight");
    let marker = dir.join("child-started");
    let program = fixture_child::executable().to_string_lossy().into_owned();
    let argv = fixture_argv("touch", &[("path", &marker.to_string_lossy())]);

    // No program at all.
    let err = executor
        .run(&[], None, Duration::from_secs(30))
        .expect_err("an empty argv is refused");
    assert!(matches!(err, ServiceError::InvalidInput(_)));
    assert!(err.manager_kind().is_none());

    // A NUL byte in the argv.
    let err = executor
        .run(
            &[program.clone(), "bad\0argument".to_string()],
            None,
            Duration::from_secs(30),
        )
        .expect_err("a NUL byte in the argv is refused");
    assert!(matches!(err, ServiceError::InvalidInput(_)));

    // A zero timeout.
    let err = executor
        .run(&argv, None, Duration::ZERO)
        .expect_err("a zero timeout is refused");
    assert!(matches!(err, ServiceError::InvalidInput(_)));

    // An over-range timeout.
    let err = executor
        .run(
            &argv,
            None,
            MAX_TRANSITION_TIMEOUT + Duration::from_secs(61),
        )
        .expect_err("an over-range timeout is refused");
    assert!(matches!(err, ServiceError::InvalidInput(_)));

    // An over-bound stdin payload.
    let over_bound = vec![b'x'; MAX_CRONTAB_BYTES + 1];
    let err = executor
        .run(&argv, Some(&over_bound), Duration::from_secs(30))
        .expect_err("an over-bound stdin payload is refused");
    assert!(matches!(err, ServiceError::InvalidInput(_)));

    // Not one of those refusals started a child.
    assert!(!marker.exists());
    let _ = std::fs::remove_dir_all(&dir);
}

/// Asserts that an error is an invalid-input failure with the given detail.
fn assert_invalid(error: &ServiceError, detail: &str) {
    assert_eq!(error.manager_kind(), None);
    match error {
        ServiceError::InvalidInput(actual) => assert_eq!(actual, detail),
        other => panic!("expected an invalid-input failure, got {other:?}"),
    }
}

/// Makes a file executable on Unix; a no-op elsewhere.
fn make_executable(path: &std::path::Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(path)
            .expect("the file exists")
            .permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(path, permissions).expect("the permissions are set");
    }
    #[cfg(not(unix))]
    let _ = path;
}

/// Builds a systemd manager over the fake executor.
fn systemd_manager(
    fake: FakeExecutor,
    unit_path: PathBuf,
    enable: bool,
    reload: bool,
) -> SystemdManager<FakeExecutor> {
    let install = SystemdInstall::new(
        "my-daemon.service".to_string(),
        SystemdScope::User,
        unit_path,
        b"[Unit]\nDescription=test\n[Service]\nExecStart=/opt/app/bin --serve\n".to_vec(),
        enable,
        reload,
        Duration::from_secs(5),
    )
    .expect("the systemd install descriptor is valid");
    SystemdManager::new(fake, install)
}

/// Builds a launchd manager over the fake executor.
fn launchd_manager(
    fake: FakeExecutor,
    plist_path: PathBuf,
    bootstrap_on_install: bool,
) -> LaunchdManager<FakeExecutor> {
    let install = LaunchdInstall::new(
        "com.example.daemon".to_string(),
        LaunchdDomain::UserAgent,
        "gui/501".to_string(),
        plist_path,
        plist(),
        bootstrap_on_install,
        Duration::from_secs(5),
    )
    .expect("the launchd install descriptor is valid");
    LaunchdManager::new(fake, install)
}
