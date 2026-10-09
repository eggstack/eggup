#![forbid(unsafe_code)]

use eggup_service::{
    CommandExecutor, CommandOutput, LifecycleState, ServiceError, ServiceId, ServiceManager,
    ServiceSpec, SystemdInstall, SystemdManager, SystemdScope,
};
#[cfg(target_os = "linux")]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::Mutex;
use std::time::Duration;

const SHOW_PROPERTIES: &str =
    "LoadState,ActiveState,SubState,ExecStart,MainPID,ControlPID,ControlGroup,Job,Result,NRestarts";

struct UnitFixture {
    name: String,
    path: PathBuf,
}

#[derive(Debug, Default)]
struct RecordingExecutor {
    inner: eggup_service::SystemExecutor,
    calls: Mutex<Vec<(Vec<String>, Option<i32>)>>,
}

impl RecordingExecutor {
    fn calls(&self) -> Vec<(Vec<String>, Option<i32>)> {
        self.calls.lock().expect("call log mutex").clone()
    }
}

impl CommandExecutor for RecordingExecutor {
    fn run(
        &self,
        argv: &[String],
        stdin_data: Option<&[u8]>,
        timeout: Duration,
    ) -> Result<CommandOutput, ServiceError> {
        let output = self.inner.run(argv, stdin_data, timeout)?;
        self.calls
            .lock()
            .expect("call log mutex")
            .push((argv.to_vec(), output.status));
        Ok(output)
    }
}

impl UnitFixture {
    fn new(label: &str, definition: &str) -> Self {
        let name = format!("eggup-m010-{}-{label}.service", std::process::id());
        let path = Path::new("/run/systemd/system").join(&name);
        std::fs::write(&path, definition).expect("write disposable unit");
        let output = systemctl(&["daemon-reload"]);
        assert_success(&output, "daemon-reload");
        Self { name, path }
    }

    fn show(&self) -> String {
        let output = systemctl(&["show", &self.name, "-p", SHOW_PROPERTIES]);
        assert_success(&output, "systemctl show");
        String::from_utf8(output.stdout).expect("systemd show output is UTF-8")
    }

    fn is_active(&self) -> (String, Option<i32>) {
        let output = systemctl(&["is-active", &self.name]);
        (
            String::from_utf8_lossy(&output.stdout).trim().to_owned(),
            output.status.code(),
        )
    }

    fn start(&self) -> Output {
        systemctl(&["start", &self.name])
    }

    fn wait_for_active_state(&self, wanted: &str) -> String {
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            let show = self.show();
            if show
                .lines()
                .any(|line| line == format!("ActiveState={wanted}"))
            {
                return show;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "unit did not reach ActiveState={wanted}: {show}"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    fn wait_for_failed_result(&self) -> String {
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            let show = self.show();
            if show.lines().any(|line| line == "ActiveState=failed") {
                return show;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "unit did not reach failed state: show={show}"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

impl Drop for UnitFixture {
    fn drop(&mut self) {
        let _ = systemctl(&["stop", &self.name]);
        let _ = std::fs::remove_file(&self.path);
        let _ = systemctl(&["daemon-reload"]);
    }
}

fn systemctl(args: &[&str]) -> Output {
    Command::new("/usr/bin/systemctl")
        .arg("--system")
        .args(args)
        .output()
        .expect("run systemctl")
}

fn journal(unit: &str) -> String {
    let output = Command::new("/usr/bin/journalctl")
        .args(["--no-pager", "--output=cat", "--lines=30", "--unit", unit])
        .output()
        .expect("run journalctl");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn assert_success(output: &Output, operation: &str) {
    assert!(
        output.status.success(),
        "{operation} failed: status={:?}, stderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn spec(name: &str, executable: &str) -> ServiceSpec {
    ServiceSpec::new(
        ServiceId::new(name).expect("service id"),
        PathBuf::from(executable),
        Vec::new(),
        None,
    )
    .expect("service spec")
}

fn manager(fixture: &UnitFixture, definition: &str) -> SystemdManager<RecordingExecutor> {
    SystemdManager::new(
        RecordingExecutor::default(),
        SystemdInstall::new(
            fixture.name.clone(),
            SystemdScope::System,
            fixture.path.clone(),
            definition.as_bytes().to_vec(),
            false,
            false,
            Duration::from_secs(5),
        )
        .expect("systemd install descriptor"),
    )
}

fn require_systemd_host() {
    assert_eq!(
        std::fs::read_to_string("/proc/1/comm")
            .expect("read init process")
            .trim(),
        "systemd",
        "M010 qualification requires a Linux VM with systemd as PID 1"
    );
    assert!(
        Path::new("/sys/fs/cgroup/cgroup.controllers").exists(),
        "M010 cgroup quiescence qualification requires cgroup v2"
    );
    eprintln!(
        "kernel={}",
        std::fs::read_to_string("/proc/sys/kernel/osrelease")
            .expect("read Linux kernel release")
            .trim()
    );
    let output = systemctl(&["show-environment"]);
    assert_success(&output, "systemctl show-environment");
}

fn assert_no_cgroup_tasks(show: &str) {
    let properties: std::collections::HashMap<_, _> = show
        .lines()
        .filter_map(|line| line.split_once('='))
        .collect();
    assert_eq!(properties.get("MainPID"), Some(&"0"), "{show}");
    assert_eq!(properties.get("ControlPID"), Some(&"0"), "{show}");
    let group = properties
        .get("ControlGroup")
        .expect("ControlGroup property");
    if group.is_empty() {
        eprintln!("cgroup_quiescence=ControlGroup absent");
        return;
    }
    assert!(group.starts_with('/'), "untrusted ControlGroup: {group:?}");
    assert!(
        group
            .split('/')
            .skip(1)
            .all(|component| !component.is_empty() && component != "." && component != ".."),
        "unsafe ControlGroup: {group:?}"
    );
    let path = Path::new("/sys/fs/cgroup").join(group.trim_start_matches('/'));
    if !path.exists() {
        return;
    }
    let processes =
        std::fs::read_to_string(path.join("cgroup.procs")).expect("read cgroup process list");
    assert!(
        processes.trim().is_empty(),
        "live cgroup processes: {processes}"
    );
    let events = std::fs::read_to_string(path.join("cgroup.events")).expect("read cgroup events");
    eprintln!("cgroup_quiescence=path={path:?} procs={processes:?} events={events:?}");
    assert!(
        events.lines().any(|line| line == "populated 0"),
        "cgroup still populated: {events}"
    );
}

fn cgroup_snapshot(show: &str) -> String {
    let group = show
        .lines()
        .find_map(|line| line.strip_prefix("ControlGroup="))
        .unwrap_or("<missing>");
    if group.is_empty() {
        return "ControlGroup absent".to_string();
    }
    if !group.starts_with('/') || group.split('/').any(|part| part == "..") {
        return format!("unsafe ControlGroup {group:?}");
    }
    let path = Path::new("/sys/fs/cgroup").join(group.trim_start_matches('/'));
    format!(
        "ControlGroup={group:?} cgroup.procs={:?} cgroup.events={:?}",
        std::fs::read_to_string(path.join("cgroup.procs")),
        std::fs::read_to_string(path.join("cgroup.events"))
    )
}

#[cfg(target_os = "linux")]
fn failed_unit_with_residual_cgroup_process_is_not_complete() {
    let directory =
        std::env::temp_dir().join(format!("eggup-m010-residual-{}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("create helper directory");
    let script = directory.join("fail-after-spawning-child");
    let pid_file = directory.join("child.pid");
    std::fs::write(
        &script,
        format!(
            "#!/bin/sh\n/usr/bin/sleep 120 &\necho $! > {}\nexit 23\n",
            pid_file.display()
        ),
    )
    .expect("write helper executable");
    let mut permissions = std::fs::metadata(&script)
        .expect("stat helper executable")
        .permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(&script, permissions).expect("make helper executable");

    let definition = format!(
        "[Unit]\nDescription=Eggup M010 residual cgroup fixture\n[Service]\nType=exec\nExecStart={}\nKillMode=process\nRestart=no\n",
        script.display()
    );
    let fixture = UnitFixture::new("residual", &definition);
    let start = fixture.start();
    assert_success(&start, "start residual-process fixture");
    let before = fixture.wait_for_active_state("failed");
    let pid: u32 = std::fs::read_to_string(&pid_file)
        .expect("read fixture child pid")
        .trim()
        .parse()
        .expect("fixture child pid is numeric");
    struct ChildCleanup(u32);
    impl Drop for ChildCleanup {
        fn drop(&mut self) {
            let _ = Command::new("/usr/bin/kill")
                .arg(self.0.to_string())
                .output();
        }
    }
    let _child_cleanup = ChildCleanup(pid);
    assert!(Path::new(&format!("/proc/{pid}")).exists());
    eprintln!("residual_process_before={before:?} pid={pid}");

    let mut manager = manager(&fixture, &definition);
    let want = spec(&fixture.name, script.to_str().expect("UTF-8 helper path"));
    let stop = manager
        .stop(&want, Duration::from_secs(5))
        .expect("owned failed unit stop should return a bounded result");
    let after = fixture.show();
    eprintln!(
        "residual_process_stop={stop:?} after_show={after:?} cgroup={}",
        cgroup_snapshot(&after)
    );
    eprintln!(
        "residual_process_stop_commands={:?}",
        manager
            .executor()
            .calls()
            .into_iter()
            .filter(|(argv, _)| argv.get(2).map(String::as_str) == Some("stop"))
            .collect::<Vec<_>>()
    );
    assert!(
        !stop.completed,
        "live process/cgroup cannot be called quiescent"
    );
    assert!(Path::new(&format!("/proc/{pid}")).exists());
    drop(_child_cleanup);
    drop(fixture);
    let _ = std::fs::remove_dir_all(directory);
}

#[test]
fn real_systemd_owned_failed_service_stop_is_observed_and_quiescent() {
    if std::env::var_os("EGGUP_SYSTEMD_INTEGRATION").is_none() {
        eprintln!("skipped: set EGGUP_SYSTEMD_INTEGRATION=1 in a disposable systemd VM");
        return;
    }
    require_systemd_host();

    let failed_definition = "[Unit]\nDescription=Eggup M010 failed-unit fixture\n[Service]\nType=exec\nExecStart=/usr/bin/false\nRestart=no\n";
    let failed = UnitFixture::new("failed", failed_definition);
    let start = failed.start();
    let before = failed.wait_for_active_state("failed");
    let (active_before, active_status_before) = failed.is_active();
    eprintln!(
        "systemd={} start_status={:?} before_show={before:?} before_is_active=({active_before:?}, {active_status_before:?})",
        String::from_utf8_lossy(&systemctl(&["--version"]).stdout)
            .lines()
            .next()
            .unwrap_or("unknown"),
        start.status.code()
    );
    assert!(before.lines().any(|line| line == "LoadState=loaded"));
    assert!(before.lines().any(|line| line == "ActiveState=failed"));
    assert_eq!(active_before, "failed");

    let mut failed_manager = manager(&failed, failed_definition);
    let failed_spec = spec(&failed.name, "/usr/bin/false");
    let observed = failed_manager
        .inspect(&failed_spec)
        .expect("inspect failed unit");
    assert_eq!(observed.ownership, eggup_service::Ownership::Owned);
    assert_eq!(observed.state, LifecycleState::Unknown);
    let stop = failed_manager
        .stop(&failed_spec, Duration::from_secs(5))
        .expect("owned failed-unit stop should return a bounded result");
    let after = failed.show();
    let (active_after, active_status_after) = failed.is_active();
    eprintln!(
        "failed_stop={stop:?} after_show={after:?} after_is_active=({active_after:?}, {active_status_after:?})"
    );
    eprintln!(
        "failed_stop_commands={:?}",
        failed_manager
            .executor()
            .calls()
            .into_iter()
            .filter(|(argv, _)| argv.get(2).map(String::as_str) == Some("stop"))
            .collect::<Vec<_>>()
    );
    assert!(
        stop.completed,
        "owned failed unit was not proven quiescent: {stop:?}"
    );
    assert!(matches!(active_after.as_str(), "failed" | "inactive"));
    assert_eq!(active_status_after, Some(3));
    assert!(after
        .lines()
        .any(|line| line == format!("ActiveState={active_after}")));
    assert_no_cgroup_tasks(&after);

    let recovered_definition = "[Unit]\nDescription=Eggup M010 caller-repaired fixture\n[Service]\nType=exec\nExecStart=/usr/bin/sleep 120\nRestart=no\n";
    std::fs::write(&failed.path, recovered_definition).expect("write caller-repaired unit");
    assert_success(
        &systemctl(&["daemon-reload"]),
        "reload caller-repaired unit",
    );
    let mut recovered_manager = manager(&failed, recovered_definition);
    let recovered_spec = ServiceSpec::new(
        ServiceId::new(failed.name.clone()).expect("repaired service id"),
        PathBuf::from("/usr/bin/sleep"),
        vec!["120".to_string()],
        None,
    )
    .expect("repaired service spec");
    let started = recovered_manager
        .start(&recovered_spec, Duration::from_secs(5))
        .expect("start caller-repaired unit");
    assert!(started.completed, "{started:?}");
    let stopped = recovered_manager
        .stop(&recovered_spec, Duration::from_secs(5))
        .expect("stop caller-repaired unit");
    assert!(stopped.completed, "{stopped:?}");

    let restart_definition = "[Unit]\nDescription=Eggup M010 restart-limit fixture\nStartLimitIntervalSec=60s\nStartLimitBurst=2\n[Service]\nType=exec\nExecStart=/usr/bin/false\nRestart=no\n";
    let restart = UnitFixture::new("restart-limit", restart_definition);
    let restart_start = restart.start();
    let restart_first = restart.wait_for_failed_result();
    let restart_second_start = restart.start();
    assert_success(&restart_second_start, "second start-limit fixture launch");
    let restart_second = restart.wait_for_failed_result();
    let rejected_start = restart.start();
    let restart_limited = restart.show();
    let first_exec = restart_first
        .lines()
        .find(|line| line.starts_with("ExecStart="))
        .expect("first failed invocation");
    let second_exec = restart_second
        .lines()
        .find(|line| line.starts_with("ExecStart="))
        .expect("second failed invocation");
    let limited_exec = restart_limited
        .lines()
        .find(|line| line.starts_with("ExecStart="))
        .expect("rate-limited invocation record");
    assert_ne!(first_exec, second_exec, "second launch must execute again");
    assert_eq!(
        second_exec, limited_exec,
        "rejected launch must not execute"
    );
    assert!(
        !rejected_start.status.success(),
        "third start must be rate limited"
    );
    eprintln!(
        "restart_limit_start_status={:?} second_status={:?} rejected_status={:?} first={restart_first:?} second={restart_second:?} limited={restart_limited:?} journal={:?}",
        restart_start.status.code(),
        restart_second_start.status.code(),
        rejected_start.status.code(),
        journal(&restart.name),
    );
    assert!(restart_limited
        .lines()
        .any(|line| line == "ActiveState=failed"));
    let mut restart_manager = manager(&restart, restart_definition);
    let restart_spec = spec(&restart.name, "/usr/bin/false");
    let restart_stop = restart_manager
        .stop(&restart_spec, Duration::from_secs(5))
        .expect("stop rate-limited failed unit");
    let restart_after = restart.show();
    eprintln!(
        "restart_limit_stop={restart_stop:?} show={restart_after:?} cgroup={}",
        cgroup_snapshot(&restart_after)
    );
    assert!(restart_stop.completed, "{restart_stop:?}");
    assert_no_cgroup_tasks(&restart_after);

    #[cfg(target_os = "linux")]
    failed_unit_with_residual_cgroup_process_is_not_complete();

    let running_definition = "[Unit]\nDescription=Eggup M010 foreign-unit fixture\n[Service]\nType=exec\nExecStart=/usr/bin/sleep 120\nRestart=no\n";
    let foreign = UnitFixture::new("foreign", running_definition);
    assert_success(&foreign.start(), "start foreign control unit");
    let mut foreign_manager = manager(&foreign, running_definition);
    let wrong_spec = spec(&foreign.name, "/usr/bin/false");
    let denied = foreign_manager
        .stop(&wrong_spec, Duration::from_secs(2))
        .expect_err("changed ExecStart must deny stop");
    assert!(denied.to_string().contains("ownership"));
    assert_eq!(foreign.is_active().0, "active");
    eprintln!("foreign_control_denied={denied}");
}
