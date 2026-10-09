#![forbid(unsafe_code)]

use eggup_service::{
    ServiceId, ServiceManager, ServiceSpec, SystemExecutor, SystemdInstall, SystemdManager,
    SystemdScope,
};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, Instant};

struct UnitFixture {
    name: String,
    path: PathBuf,
}

impl UnitFixture {
    fn new() -> Self {
        let name = format!("eggup-m010-012-{}.service", std::process::id());
        let path = Path::new("/run/systemd/system").join(&name);
        std::fs::write(
            &path,
            "[Unit]\nDescription=Eggup published 0.1.2 negative control\n[Service]\nType=exec\nExecStart=/usr/bin/false\nRestart=no\n",
        )
        .expect("write disposable unit");
        assert_success(&systemctl(&["daemon-reload"]), "daemon-reload");
        Self { name, path }
    }

    fn start(&self) -> Output {
        systemctl(&["start", &self.name])
    }

    fn show(&self) -> String {
        let output = systemctl(&[
            "show",
            &self.name,
            "-p",
            "LoadState,ActiveState,ExecStart,MainPID,ControlPID,ControlGroup,Job",
        ]);
        assert_success(&output, "systemctl show");
        String::from_utf8(output.stdout).expect("show output is UTF-8")
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

fn assert_success(output: &Output, operation: &str) {
    assert!(
        output.status.success(),
        "{operation} failed: status={:?}, stderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn published_0_1_2_cannot_complete_stop_for_owned_failed_unit() {
    if std::env::var_os("EGGUP_SYSTEMD_INTEGRATION").is_none() {
        eprintln!("skipped: set EGGUP_SYSTEMD_INTEGRATION=1 in a disposable systemd VM");
        return;
    }
    assert_eq!(
        std::fs::read_to_string("/proc/1/comm")
            .expect("read init process")
            .trim(),
        "systemd",
        "negative control requires systemd as PID 1"
    );
    assert!(Path::new("/sys/fs/cgroup/cgroup.controllers").exists());

    let fixture = UnitFixture::new();
    let start = fixture.start();
    eprintln!("published_0_1_2_start_status={:?}", start.status.code());
    let deadline = Instant::now() + Duration::from_secs(5);
    let before = loop {
        let show = fixture.show();
        if show.lines().any(|line| line == "ActiveState=failed") {
            break show;
        }
        assert!(Instant::now() < deadline, "unit did not fail: {show}");
        std::thread::sleep(Duration::from_millis(10));
    };

    let spec = ServiceSpec::new(
        ServiceId::new(fixture.name.clone()).expect("service id"),
        PathBuf::from("/usr/bin/false"),
        Vec::new(),
        None,
    )
    .expect("service spec");
    let install = SystemdInstall::new(
        fixture.name.clone(),
        SystemdScope::System,
        fixture.path.clone(),
        b"[Unit]\nDescription=Eggup published 0.1.2 negative control\n[Service]\nType=exec\nExecStart=/usr/bin/false\nRestart=no\n".to_vec(),
        false,
        false,
        Duration::from_secs(5),
    )
    .expect("systemd install descriptor");
    let mut manager = SystemdManager::new(SystemExecutor::new(), install);
    let before_stop = manager.inspect(&spec).expect("inspect failed unit");
    assert_eq!(before_stop.ownership, eggup_service::Ownership::Owned);
    assert_eq!(before_stop.state, eggup_service::LifecycleState::Unknown);
    let stop = manager
        .stop(&spec, Duration::from_secs(5))
        .expect("0.1.2 returns a bounded stop result");
    let after = fixture.show();
    eprintln!("published_0_1_2_before={before:?} stop={stop:?} after={after:?}");
    for expected in [
        "ActiveState=failed",
        "MainPID=0",
        "ControlPID=0",
        "ControlGroup=",
        "Job=",
    ] {
        assert!(
            after.lines().any(|line| line == expected),
            "negative control did not establish {expected:?}: {after}"
        );
    }
    assert!(
        !stop.completed,
        "published 0.1.2 unexpectedly completed an owned failed unit stop"
    );
    assert!(after.lines().any(|line| line == "ActiveState=failed"));
}
