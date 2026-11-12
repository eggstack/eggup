//! M013 — one timeout model: one deadline per operation.
//!
//! A recording [`CommandExecutor`] captures the `timeout` argument of every
//! `run` call and drives a scripted systemd `install` on an `Owned` unit.
//!
//! This file uses only the **pre-M013 public API surface**, so it compiles
//! unchanged against the pre-M013 baseline
//! (`c4805411f74b85158e0ae5d0a219a32c4209d98e`) and fails there by
//! assertion: before M013 each manager call received a fresh full budget.
//!
//! The proof is made on the budgets handed to the executor and on the call
//! count. Nothing here sleeps on the wall clock.
//!
//! Platform gate note: the gate is applied **per test** so this module still
//! compiles and still runs its platform-independent fixture on Windows. The
//! scripted install needs a unit the crate classifies as `Owned`, and that
//! classification comes from the `ExecStart` payload, which carries the spec
//! executable verbatim; the crate's narrow `ExecStart` grammar rejects a
//! backslash, and a Windows executable path contains one. The Unix-only
//! helpers carry the same gate, so the Windows build stays warning-free.

#[cfg(unix)]
use eggup_service::{
    CommandExecutor, CommandOutput, ServiceError, ServiceId, ServiceManager, ServiceSpec,
    SystemdInstall, SystemdManager, SystemdScope,
};
#[cfg(unix)]
use std::path::PathBuf;
#[cfg(unix)]
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// The platform's "absolute path" spelling of the fixture executable.
#[cfg(unix)]
fn platform_exe() -> &'static str {
    if cfg!(windows) {
        r"C:\opt\app\bin.exe"
    } else {
        "/opt/app/bin"
    }
}

#[cfg(unix)]
fn fixture_spec() -> ServiceSpec {
    ServiceSpec::new(
        ServiceId::new("eggup-m013-budget").unwrap(),
        PathBuf::from(platform_exe()),
        vec!["--serve".to_string()],
        None,
    )
    .unwrap()
}

/// A `systemctl show` payload that the crate classifies as `Owned`.
#[cfg(unix)]
fn owned_show() -> String {
    format!(
        "LoadState=loaded\nActiveState=inactive\nExecStart={{ path={exe} ; argv[]={exe} --serve ; ignore_errors=no }}\n",
        exe = platform_exe()
    )
}

/// The shared record of every manager invocation.
#[cfg(unix)]
type Calls = Arc<Mutex<Vec<(Vec<String>, Duration)>>>;

/// Records the argv and the timeout budget of every manager invocation.
#[cfg(unix)]
#[derive(Debug, Clone)]
struct RecordingExecutor {
    calls: Calls,
    show: String,
}

#[cfg(unix)]
impl RecordingExecutor {
    fn new(show: String) -> (Self, Calls) {
        let calls: Calls = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                calls: Arc::clone(&calls),
                show,
            },
            calls,
        )
    }
}

#[cfg(unix)]
impl CommandExecutor for RecordingExecutor {
    fn run(
        &self,
        argv: &[String],
        _stdin_data: Option<&[u8]>,
        timeout: Duration,
    ) -> Result<CommandOutput, ServiceError> {
        self.calls.lock().unwrap().push((argv.to_vec(), timeout));
        let is_show = argv.iter().any(|a| a == "show");
        let stdout = if is_show {
            self.show.clone()
        } else {
            String::new()
        };
        Ok(CommandOutput {
            status: Some(0),
            stdout: stdout.into_bytes(),
            stderr: Vec::new(),
        })
    }
}

/// The fixture's own predicate: no manager invocation may be granted a fresh
/// full budget. Every recorded budget must be *the remaining* budget of the
/// operation's single deadline.
fn no_invocation_gets_a_fresh_full_budget(budgets: &[Duration], budget: Duration) -> bool {
    budgets.iter().all(|b| *b < budget)
}

#[cfg(unix)]
fn budgets_of(calls: &Calls) -> Vec<Duration> {
    calls
        .lock()
        .unwrap()
        .iter()
        .map(|(_, budget)| *budget)
        .collect()
}

#[cfg(unix)]
fn argvs_of(calls: &Calls) -> Vec<Vec<String>> {
    calls
        .lock()
        .unwrap()
        .iter()
        .map(|(argv, _)| argv.clone())
        .collect()
}

/// Unix-scoped: the scripted install needs a unit the crate classifies as
/// `Owned`, which the scripted `ExecStart` payload cannot express on Windows
/// (see the platform gate note above).
#[cfg(unix)]
#[test]
fn a_scripted_systemd_install_shares_one_budget_across_every_manager_call() {
    let budget = Duration::from_secs(30);
    let (executor, calls) = RecordingExecutor::new(owned_show());
    let mut unit_path = std::env::temp_dir();
    unit_path.push(format!("eggup-m013-budget-{}.service", std::process::id()));
    let install = SystemdInstall::new(
        "eggup-m013-budget.service".to_string(),
        SystemdScope::User,
        unit_path.clone(),
        b"[Unit]\nDescription=m013\n".to_vec(),
        true,
        true,
        budget,
    )
    .unwrap();
    let mut manager = SystemdManager::new(executor, install);

    let result = manager.install(&fixture_spec());
    assert!(
        result.is_ok(),
        "the scripted install must succeed: {:?}",
        result.err()
    );

    let budgets = budgets_of(&calls);
    // The expected number of manager calls: the ownership probe, the
    // pre-write ownership recheck, `daemon-reload`, and `enable`.
    assert_eq!(
        budgets.len(),
        4,
        "expected four manager calls, observed {:?}",
        argvs_of(&calls)
    );
    // Every recorded budget is at most the operation budget.
    for (index, recorded) in budgets.iter().enumerate() {
        assert!(
            *recorded <= budget,
            "call {index} was granted {recorded:?}, above the operation budget {budget:?}"
        );
    }
    // No invocation is granted a fresh full budget: every recorded budget is
    // the *remaining* budget of the operation's single deadline.
    assert!(
        no_invocation_gets_a_fresh_full_budget(&budgets, budget),
        "a manager invocation was granted a fresh full budget: {budgets:?}"
    );
    // The shared budget only shrinks; it never grows.
    for pair in budgets.windows(2) {
        assert!(
            pair[1] <= pair[0],
            "the shared budget grew between calls: {budgets:?}"
        );
    }
    let _ = std::fs::remove_file(&unit_path);
}

/// Negative control: the pre-M013 schedule — every call granted a fresh full
/// budget — must fail the fixture's own predicate. If it did not, the fixture
/// could not detect the defect it exists to detect.
#[test]
fn a_fresh_full_budget_per_call_fails_the_fixture_predicate() {
    let budget = Duration::from_secs(30);
    let pre_m013_schedule = vec![budget; 4];
    assert!(
        !no_invocation_gets_a_fresh_full_budget(&pre_m013_schedule, budget),
        "the fixture predicate must reject the pre-M013 schedule"
    );
    // And a schedule that really does draw from one budget must pass.
    let shared = vec![
        budget - Duration::from_millis(1),
        budget - Duration::from_millis(2),
        budget - Duration::from_millis(3),
        budget - Duration::from_millis(4),
    ];
    assert!(no_invocation_gets_a_fresh_full_budget(&shared, budget));
}
