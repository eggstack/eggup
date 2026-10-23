//! M013 — typed manager failure kinds.
//!
//! These fixtures drive the crate's *pure* classification functions and the
//! public error surface. They reference [`eggup_service::ManagerFailureKind`],
//! which does not exist in the pre-M013 baseline: the baseline proof for this
//! file is therefore a **compile failure**, which the plan (§10.9) accepts for
//! the fixtures that require the new typed API.
//!
//! Nothing here sleeps on the wall clock and nothing needs a real privilege
//! failure.
//!
//! Platform gate note: the gate is applied **per test** so this module still
//! compiles and still runs its platform-independent fixtures on Windows. Two
//! of the fixtures below depend on POSIX path spelling: the allowlist probe
//! needs an absolute path (`/nonexistent-...` is not absolute on Windows), and
//! the systemd `ExecStart` payload carries the spec executable verbatim —
//! while the crate's narrow `ExecStart` grammar rejects a backslash, a
//! Windows executable path contains one, so the payload could not be
//! classified as `Owned` there. The remaining fixtures in this file are
//! platform-independent and still run everywhere.

use eggup_service::{
    CommandExecutor, CommandOutput, ManagerFailureKind, ServiceError, ServiceId, ServiceManager,
    ServiceSpec, SystemdInstall, SystemdManager, SystemdScope,
};
// `SystemExecutor` is used only by the Unix-gated allowlist probes: the
// missing-binary path they exercise is spelled as a POSIX absolute path.
#[cfg(unix)]
use eggup_service::SystemExecutor;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

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
        ServiceId::new("eggup-m013-kinds").unwrap(),
        PathBuf::from(platform_exe()),
        vec!["--serve".to_string()],
        None,
    )
    .unwrap()
}

/// A `systemctl show` payload that the crate classifies as `Owned`.
fn owned_show() -> String {
    format!(
        "LoadState=loaded\nActiveState=inactive\nExecStart={{ path={exe} ; argv[]={exe} --serve ; ignore_errors=no }}\n",
        exe = platform_exe()
    )
}

/// A recording executor: it captures every argv it is asked to run and
/// answers with scripted outputs.
#[derive(Debug, Default)]
struct RecordingExecutor {
    calls: Mutex<Vec<Vec<String>>>,
    show: String,
    mutation_status: i32,
}

impl RecordingExecutor {
    fn new(show: String, mutation_status: i32) -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            show,
            mutation_status,
        }
    }
}

impl CommandExecutor for RecordingExecutor {
    fn run(
        &self,
        argv: &[String],
        _stdin_data: Option<&[u8]>,
        _timeout: Duration,
    ) -> Result<CommandOutput, ServiceError> {
        self.calls.lock().unwrap().push(argv.to_vec());
        let is_show = argv.iter().any(|a| a == "show");
        let status = if is_show { 0 } else { self.mutation_status };
        let stdout = if is_show {
            self.show.clone()
        } else {
            String::new()
        };
        Ok(CommandOutput {
            status: Some(status),
            stdout: stdout.into_bytes(),
            stderr: Vec::new(),
        })
    }
}

/// Builds a systemd manager whose unit file lives in a unique temp path.
fn systemd_manager(
    executor: RecordingExecutor,
    timeout: Duration,
) -> (SystemdManager<RecordingExecutor>, PathBuf) {
    let mut unit_path = std::env::temp_dir();
    unit_path.push(format!("eggup-m013-kinds-{}.service", std::process::id()));
    let install = SystemdInstall::new(
        "eggup-m013-kinds.service".to_string(),
        SystemdScope::User,
        unit_path.clone(),
        b"[Unit]\nDescription=m013\n".to_vec(),
        false,
        true,
        timeout,
    )
    .unwrap();
    (SystemdManager::new(executor, install), unit_path)
}

// ---- Spawn ----

#[test]
fn spawn_kind_is_reachable_from_a_constructed_io_error() {
    // No real privilege failure and no real spawn: the classification is a
    // pure function of the operating-system error kind.
    let denied = std::io::Error::from(std::io::ErrorKind::PermissionDenied);
    let missing = std::io::Error::from(std::io::ErrorKind::NotFound);
    let other = std::io::Error::from(std::io::ErrorKind::BrokenPipe);

    assert_eq!(
        ManagerFailureKind::from_spawn_error(&denied),
        ManagerFailureKind::Permission
    );
    assert_eq!(
        ManagerFailureKind::from_spawn_error(&missing),
        ManagerFailureKind::NotFound
    );
    assert_eq!(
        ManagerFailureKind::from_spawn_error(&other),
        ManagerFailureKind::Spawn
    );
}

#[test]
fn spawn_kind_negative_control_rejects_a_wrong_expectation() {
    // Negative control: the classification must discriminate. If the
    // mapping were vacuous, a wrong expectation would also "hold".
    let denied = std::io::Error::from(std::io::ErrorKind::PermissionDenied);
    assert_ne!(
        ManagerFailureKind::from_spawn_error(&denied),
        ManagerFailureKind::Spawn
    );
    assert_ne!(
        ManagerFailureKind::from_spawn_error(&denied),
        ManagerFailureKind::NotFound
    );
}

// ---- Permission ----

#[test]
fn permission_kind_is_constructible_on_every_lane() {
    let denied = std::io::Error::from(std::io::ErrorKind::PermissionDenied);
    assert_eq!(
        ManagerFailureKind::from_spawn_error(&denied),
        ManagerFailureKind::Permission
    );
}

// ---- NotFound ----

// The absolute-path branch of the program allowlist validates the path and
// reports a missing binary as `NotFound`. No process is spawned.
//
// Unix-scoped: the fixture's program path is a POSIX absolute path, and a
// Windows path is not absolute for the allowlist's `is_absolute` check.
#[cfg(unix)]
#[test]
fn not_found_kind_is_reachable_through_the_allowlist_path() {
    let executor = SystemExecutor::new();
    let err = executor
        .run(
            &["/nonexistent-eggup-m013-manager".to_string()],
            None,
            Duration::from_secs(5),
        )
        .expect_err("a missing manager binary must fail");
    assert_eq!(err.manager_kind(), Some(ManagerFailureKind::NotFound));
}

#[cfg(unix)]
#[test]
fn not_found_kind_negative_control_rejects_a_wrong_expectation() {
    let executor = SystemExecutor::new();
    let err = executor
        .run(
            &["/nonexistent-eggup-m013-manager".to_string()],
            None,
            Duration::from_secs(5),
        )
        .expect_err("a missing manager binary must fail");
    assert_ne!(err.manager_kind(), Some(ManagerFailureKind::Spawn));
    assert_ne!(err.manager_kind(), Some(ManagerFailureKind::Timeout));
}

// ---- Timeout ----

#[test]
fn timeout_kind_is_reachable_without_a_wall_clock_sleep() {
    // A one-nanosecond budget is already exhausted by the time the first
    // manager command is issued. Whichever manager call observes the
    // exhausted deadline, the operation reports `Timeout` — never a
    // successful transition and never an assumed state.
    let (mut manager, unit_path) = systemd_manager(
        RecordingExecutor::new(owned_show(), 0),
        Duration::from_secs(5),
    );
    let err = manager
        .start(&fixture_spec(), Duration::from_nanos(1))
        .expect_err("an exhausted deadline must fail");
    assert_eq!(err.manager_kind(), Some(ManagerFailureKind::Timeout));
    let _ = std::fs::remove_file(&unit_path);
}

#[test]
fn timeout_kind_negative_control_rejects_a_wrong_expectation() {
    let (mut manager, unit_path) = systemd_manager(
        RecordingExecutor::new(owned_show(), 0),
        Duration::from_secs(5),
    );
    let err = manager
        .start(&fixture_spec(), Duration::from_nanos(1))
        .expect_err("an exhausted deadline must fail");
    assert_ne!(err.manager_kind(), Some(ManagerFailureKind::Spawn));
    assert_ne!(err.manager_kind(), Some(ManagerFailureKind::Protocol));
    let _ = std::fs::remove_file(&unit_path);
}

// ---- Protocol ----

// Unix-scoped: the scripted `ExecStart` payload carries the spec executable
// verbatim, and the crate's narrow `ExecStart` grammar rejects a backslash —
// a Windows executable path contains one, so the unit could not be classified
// as `Owned` there and the install would be denied instead of failing with a
// protocol error.
#[cfg(unix)]
#[test]
fn protocol_kind_is_reported_for_a_nonzero_manager_exit() {
    // The manager ran and reported a failure the crate cannot accept: a
    // non-zero exit on a mutation.
    let (mut manager, unit_path) = systemd_manager(
        RecordingExecutor::new(owned_show(), 7),
        Duration::from_secs(5),
    );
    let err = manager
        .install(&fixture_spec())
        .expect_err("a nonzero mutation exit must fail");
    assert_eq!(err.manager_kind(), Some(ManagerFailureKind::Protocol));
    let _ = std::fs::remove_file(&unit_path);
}

#[cfg(unix)]
#[test]
fn protocol_kind_negative_control_rejects_a_wrong_expectation() {
    let (mut manager, unit_path) = systemd_manager(
        RecordingExecutor::new(owned_show(), 7),
        Duration::from_secs(5),
    );
    let err = manager
        .install(&fixture_spec())
        .expect_err("a nonzero mutation exit must fail");
    assert_ne!(err.manager_kind(), Some(ManagerFailureKind::Timeout));
    assert_ne!(err.manager_kind(), Some(ManagerFailureKind::NotFound));
    let _ = std::fs::remove_file(&unit_path);
}

// ---- Windows SCM OS code mapping (platform-independent) ----

#[test]
fn scm_os_code_5_maps_to_permission() {
    assert_eq!(
        ManagerFailureKind::from_scm_os_code(Some(5)),
        ManagerFailureKind::Permission
    );
}

#[test]
fn scm_os_code_1060_maps_to_not_found() {
    assert_eq!(
        ManagerFailureKind::from_scm_os_code(Some(1060)),
        ManagerFailureKind::NotFound
    );
}

#[test]
fn other_scm_os_codes_map_to_protocol() {
    assert_eq!(
        ManagerFailureKind::from_scm_os_code(Some(1)),
        ManagerFailureKind::Protocol
    );
    assert_eq!(
        ManagerFailureKind::from_scm_os_code(Some(1072)),
        ManagerFailureKind::Protocol
    );
    assert_eq!(
        ManagerFailureKind::from_scm_os_code(None),
        ManagerFailureKind::Protocol
    );
}

#[test]
fn scm_os_code_negative_control_rejects_a_wrong_expectation() {
    assert_ne!(
        ManagerFailureKind::from_scm_os_code(Some(5)),
        ManagerFailureKind::NotFound
    );
    assert_ne!(
        ManagerFailureKind::from_scm_os_code(Some(1060)),
        ManagerFailureKind::Permission
    );
    assert_ne!(
        ManagerFailureKind::from_scm_os_code(Some(1)),
        ManagerFailureKind::Permission
    );
}

// ---- The kind, not the text ----

// Unix-scoped: same scripted `ExecStart` payload as the protocol fixture
// above, and the same backslash constraint.
#[cfg(unix)]
#[test]
fn the_kind_is_reachable_without_reading_the_detail_text() {
    let (mut manager, unit_path) = systemd_manager(
        RecordingExecutor::new(owned_show(), 7),
        Duration::from_secs(5),
    );
    let err = manager
        .install(&fixture_spec())
        .expect_err("a nonzero mutation exit must fail");
    // The consumer discriminates on the kind alone. The detail text is free
    // prose and may change; the kind may not.
    let kind = err.manager_kind().expect("a manager failure has a kind");
    assert_eq!(kind, ManagerFailureKind::Protocol);
    let _ = std::fs::remove_file(&unit_path);
}
