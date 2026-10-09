#![forbid(unsafe_code)]

use eggup_service::{
    CommandExecutor, CommandOutput, LifecycleState, Ownership, ServiceError, ServiceId,
    ServiceManager, ServiceSpec, SystemdInstall, SystemdManager, SystemdScope,
};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug)]
struct FixedExecutor(&'static str);

impl CommandExecutor for FixedExecutor {
    fn run(
        &self,
        _argv: &[String],
        _stdin_data: Option<&[u8]>,
        _timeout: Duration,
    ) -> Result<CommandOutput, ServiceError> {
        Ok(CommandOutput {
            status: Some(0),
            stdout: self.0.as_bytes().to_vec(),
            stderr: Vec::new(),
        })
    }
}

fn inspect_fixture(show: &'static str) -> eggup_service::LifecycleSnapshot {
    let unit_name = "eggup-m010-control.service";
    let install = SystemdInstall::new(
        unit_name.to_string(),
        SystemdScope::System,
        PathBuf::from(format!("/run/systemd/system/{unit_name}")),
        b"[Service]\nExecStart=/usr/bin/false\n".to_vec(),
        false,
        false,
        Duration::from_secs(2),
    )
    .expect("install descriptor");
    let manager = SystemdManager::new(FixedExecutor(show), install);
    let spec = ServiceSpec::new(
        ServiceId::new(unit_name).expect("service id"),
        PathBuf::from("/usr/bin/false"),
        Vec::new(),
        None,
    )
    .expect("service spec");
    manager.inspect(&spec).expect("inspect fixture output")
}

#[test]
fn registry_0_1_3_classifies_exact_failed_registration_as_owned_unknown_state() {
    let snapshot = inspect_fixture(
        "LoadState=loaded\nActiveState=failed\nExecStart={ path=/usr/bin/false ; argv[]=/usr/bin/false ; ignore_errors=no ; }\n",
    );
    assert_eq!(snapshot.ownership, Ownership::Owned);
    assert_eq!(snapshot.state, LifecycleState::Unknown);
}

#[test]
fn registry_0_1_3_marks_changed_exec_start_foreign() {
    let snapshot = inspect_fixture(
        "LoadState=loaded\nActiveState=active\nExecStart={ path=/usr/bin/sleep ; argv[]=/usr/bin/sleep 120 ; ignore_errors=no ; }\n",
    );
    assert_eq!(snapshot.ownership, Ownership::Foreign);
    assert_eq!(snapshot.state, LifecycleState::Running);
}

#[test]
fn registry_0_1_3_malformed_exec_start_is_unknown() {
    let snapshot =
        inspect_fixture("LoadState=loaded\nActiveState=failed\nExecStart=ambiguous output\n");
    assert_eq!(snapshot.ownership, Ownership::Unknown);
    assert_eq!(snapshot.state, LifecycleState::Unknown);
}
