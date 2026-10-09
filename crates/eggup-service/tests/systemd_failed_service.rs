#![forbid(unsafe_code)]

use eggup_service::{
    LifecycleState, ServiceId, ServiceManager, ServiceSpec, SystemExecutor, SystemdInstall,
    SystemdManager, SystemdScope,
};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::Duration;

const SHOW_PROPERTIES: &str =
    "LoadState,ActiveState,SubState,ExecStart,MainPID,ControlPID,ControlGroup,Job,Result";

struct UnitFixture {
    name: String,
    path: PathBuf,
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
}

impl Drop for UnitFixture {
    fn drop(&mut self) {
        let _ = systemctl(&["stop", &self.name]);
        let _ = systemctl(&["reset-failed", &self.name]);
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

fn manager(fixture: &UnitFixture, definition: &str) -> SystemdManager {
    SystemdManager::new(
        SystemExecutor::new(),
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
    assert!(
        events.lines().any(|line| line == "populated 0"),
        "cgroup still populated: {events}"
    );
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
    assert!(
        !start.status.success(),
        "false fixture unexpectedly succeeded"
    );
    let before = failed.show();
    let (active_before, active_status_before) = failed.is_active();
    eprintln!(
        "systemd={} before_show={before:?} before_is_active=({active_before:?}, {active_status_before:?})",
        String::from_utf8_lossy(&systemctl(&["--version"]).stdout)
            .lines()
            .next()
            .unwrap_or("unknown")
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
    assert!(
        stop.completed,
        "owned failed unit was not proven quiescent: {stop:?}"
    );
    assert_eq!(active_after, "inactive");
    assert_eq!(active_status_after, Some(3));
    assert_no_cgroup_tasks(&after);

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
