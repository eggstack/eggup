#![cfg(windows)]
//! M014 — the **real** Windows SCM backend, executed against the genuine
//! Windows Service Control Manager.
//!
//! The `windows_scm.rs` test module drives a fake backend, so the half of the
//! adapter that crosses the OS boundary — the `windows-service` mapping layer
//! and the SCM calls themselves — had never executed on any platform. This
//! fixture closes that gap: it registers a **disposable** service through the
//! crate's **public** API (`WindowsScmInstall` + `WindowsScmManager::new`,
//! never a crate-internal back door) and drives the observable lifecycle
//! against the real SCM:
//!
//! install → inspect → ownership → start → stop → uninstall,
//!
//! plus the `windows-service` mapping round-trip (start type, error control,
//! service dependencies, and the `ServiceInfo` round-trip).
//!
//! # Rules this fixture obeys
//!
//! * **Disposable and self-cleaning.** It touches only the registrations it
//!   creates, and it removes them on every exit path — including a failing
//!   assertion, through a `Drop` guard.
//! * **Bounded waits.** Every wait polls to a target state under an explicit
//!   deadline and fails with the observed state on timeout. It never asserts
//!   that a transition happened "within N milliseconds".
//! * **Skippable with a recorded reason.** Without the administrator rights a
//!   real SCM registration requires, the fixture records the skip and its
//!   reason and returns. It never elevates, prompts, or calls `runas`.
//! * **No `unsafe`.** The workspace denies `unsafe_code` for every target.
//!
//! On a non-Windows host the whole file is compiled out, so the fixture is
//! **not run** there — it is never inferred from another platform.

use std::ffi::OsString;
use std::io::Write as _;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use eggup_service::{
    LifecycleState, Ownership, ServiceId, ServiceManager, ServiceSpec, WindowsErrorControl,
    WindowsScmInstall, WindowsScmManager, WindowsServiceDependency, WindowsStartType,
};
use windows_service::service::{
    ServiceAccess, ServiceControl, ServiceControlAccept, ServiceDependency, ServiceErrorControl,
    ServiceExitCode, ServiceStartType, ServiceState, ServiceStatus, ServiceType,
};
use windows_service::service_control_handler::{
    self, ServiceControlHandlerResult, ServiceStatusHandle,
};
use windows_service::service_dispatcher;
use windows_service::service_manager::{ServiceManager as ScmServiceManager, ServiceManagerAccess};

/// The name of the test that acts as the service entry point when the Service
/// Control Manager starts this test binary as a service.
///
/// The SCM starts the binary with `--exact --test-threads=1 <marker> <service
/// name>`; the marker is also this test's name, so the harness selects exactly
/// this test and runs it on the main thread.
const SERVICE_MARKER: &str = "eggup_scm_live_service_entrypoint";

/// The poll interval of the fixture's own bounded waits.
const POLL_INTERVAL: Duration = Duration::from_millis(50);

/// The deadline of the fixture's own bounded waits.
const POLL_DEADLINE: Duration = Duration::from_secs(60);

/// The transition timeout handed to the adapter and to the adapter's own
/// bounded polling.
const TRANSITION_TIMEOUT: Duration = Duration::from_secs(60);

/// The Service Control Manager's callback into this process.
///
/// The SCM passes the service arguments; the fixture does not need them,
/// because the service name travels in the process command line.
extern "system" fn scm_service_main(_argc: u32, _argv: *mut *mut u16) {
    let Some(name) = service_name_from_args() else {
        return;
    };
    run_as_service(&name);
}

/// Reads the service name the SCM started this process with.
///
/// The name is the argument that follows [`SERVICE_MARKER`].
fn service_name_from_args() -> Option<String> {
    let mut marker_seen = false;
    for arg in std::env::args().skip(1) {
        if marker_seen {
            return Some(arg);
        }
        if arg == SERVICE_MARKER {
            marker_seen = true;
        }
    }
    None
}

/// Registers the control handler and reports the running state.
fn run_as_service(name: &str) {
    let slot: Arc<Mutex<Option<ServiceStatusHandle>>> = Arc::new(Mutex::new(None));
    let handler_slot = Arc::clone(&slot);
    let handler = move |control: ServiceControl| {
        if matches!(
            control,
            ServiceControl::Stop | ServiceControl::Shutdown | ServiceControl::Preshutdown
        ) {
            report_stopped(&handler_slot);
        }
        ServiceControlHandlerResult::NoError
    };
    let Ok(handle) = service_control_handler::register(name, handler) else {
        return;
    };
    if let Ok(mut guard) = slot.lock() {
        *guard = Some(handle);
    }
    let _ = handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::from_secs(1),
        process_id: None,
    });
}

/// Reports the stopped state when the SCM asks this service to stop.
fn report_stopped(slot: &Arc<Mutex<Option<ServiceStatusHandle>>>) {
    let Ok(guard) = slot.lock() else {
        return;
    };
    let Some(handle) = guard.as_ref() else {
        return;
    };
    let _ = handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::ZERO,
        process_id: None,
    });
}

/// The service entry point, selected by the Service Control Manager.
///
/// In an ordinary test run there is no marker in the command line, so this
/// test returns immediately. When the SCM starts this binary as a service, the
/// marker is present and the dispatcher blocks until the SCM stops the
/// service.
#[test]
fn eggup_scm_live_service_entrypoint() {
    if !std::env::args().any(|arg| arg == SERVICE_MARKER) {
        return;
    }
    let Some(name) = service_name_from_args() else {
        return;
    };
    let _ = service_dispatcher::start(name, scm_service_main);
}

/// Writes a diagnostic that must survive the test harness's output capture.
///
/// `libtest` captures `println!`/`eprintln!` and only replays the capture for
/// a failing test, so a recorded skip reason would be invisible in a passing
/// run. Writing straight to the process's stdout keeps it in the test output.
fn record(message: &str) {
    let mut stdout = std::io::stdout().lock();
    let _ = writeln!(stdout, "{message}");
    let _ = stdout.flush();
}

/// A unique per-run suffix, so a disposable registration never collides with
/// another run or with a pre-existing service.
fn unique_suffix() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or_default();
    format!("{}-{nanos}", std::process::id())
}

/// Polls `probe` until it reports convergence or the deadline expires.
///
/// The probe returns `(converged, observed)`. On timeout the error names the
/// last observed state, so a failed wait is a diagnosis and not a bare
/// "took too long".
fn poll_until<F>(deadline: Instant, mut probe: F) -> Result<(), String>
where
    F: FnMut() -> Result<(bool, String), String>,
{
    loop {
        let (converged, observed) = probe()?;
        if converged {
            return Ok(());
        }
        let now = Instant::now();
        if now >= deadline {
            return Err(format!(
                "the SCM state did not converge before the deadline; observed: {observed}"
            ));
        }
        std::thread::sleep((deadline - now).min(POLL_INTERVAL));
    }
}

/// Reads the configuration the SCM holds for `name`.
fn query_config(name: &str) -> Result<windows_service::service::ServiceConfig, String> {
    let manager = ScmServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)
        .map_err(|error| error.to_string())?;
    let service = manager
        .open_service(name, ServiceAccess::QUERY_CONFIG)
        .map_err(|error| error.to_string())?;
    service.query_config().map_err(|error| error.to_string())
}

/// Reads the state the SCM holds for `name`.
fn query_status(name: &str) -> Result<ServiceState, String> {
    let manager = ScmServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)
        .map_err(|error| error.to_string())?;
    let service = manager
        .open_service(name, ServiceAccess::QUERY_STATUS)
        .map_err(|error| error.to_string())?;
    service
        .query_status()
        .map(|status| status.current_state)
        .map_err(|error| error.to_string())
}

/// Removes a registration directly, so a failed assertion still leaves the
/// runner without an orphaned registration.
///
/// The wait for absence is bounded and reports the observed state.
fn remove_registration(name: &str) -> Result<(), String> {
    let manager = ScmServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)
        .map_err(|error| error.to_string())?;
    let service = match manager.open_service(
        name,
        ServiceAccess::DELETE | ServiceAccess::QUERY_STATUS | ServiceAccess::STOP,
    ) {
        Ok(service) => service,
        Err(windows_service::Error::Winapi(error)) if error.raw_os_error() == Some(1060) => {
            return Ok(());
        }
        Err(error) => return Err(error.to_string()),
    };
    let _ = service.stop();
    let deadline = Instant::now() + POLL_DEADLINE;
    poll_until(deadline, || match service.query_status() {
        Ok(status) => Ok((
            status.current_state == ServiceState::Stopped,
            format!("{:?}", status.current_state),
        )),
        Err(error) => Err(error.to_string()),
    })?;
    service.delete().map_err(|error| error.to_string())?;
    let deadline = Instant::now() + POLL_DEADLINE;
    poll_until(deadline, || {
        match manager.open_service(name, ServiceAccess::QUERY_STATUS) {
            Ok(_) => Ok((false, "the registration is still present".to_string())),
            Err(windows_service::Error::Winapi(error)) if error.raw_os_error() == Some(1060) => {
                Ok((true, "absent".to_string()))
            }
            Err(error) => Err(error.to_string()),
        }
    })
}

/// Removes a disposable registration when the fixture exits, including on a
/// failing assertion.
struct Disposable {
    name: String,
    armed: bool,
}

impl Disposable {
    fn new(name: String) -> Self {
        Self { name, armed: false }
    }

    fn arm(&mut self) {
        self.armed = true;
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for Disposable {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        if let Err(error) = remove_registration(&self.name) {
            record(&format!(
                "EGGUP-SCM-LIVE-CLEANUP-FAILED: {}: {error}",
                self.name
            ));
        }
    }
}

/// The whole observable lifecycle of a disposable registration, against the
/// real Windows Service Control Manager.
#[test]
fn real_scm_registers_observes_starts_stops_and_unregisters_a_disposable_service() {
    // R-A5: the fixture is skippable with a recorded reason, never a silent
    // pass. The probe is the SCM's own access check: without the rights to
    // create a service, no real registration is possible. The fixture does
    // not elevate, prompt, or shell out.
    if let Err(error) =
        ScmServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CREATE_SERVICE)
    {
        record(&format!(
            "EGGUP-SCM-LIVE-SKIP: administrator rights for a real SCM registration are unavailable: {error}"
        ));
        return;
    }

    let suffix = unique_suffix();
    let service_name = format!("EggupScmLive-{suffix}");
    let dependency_name = format!("EggupScmLiveDep-{suffix}");
    let executable = std::env::current_exe().expect("the test binary path");
    let executable_text = executable
        .to_str()
        .expect("the test binary path must be representable as UTF-8")
        .to_string();

    let service_args = vec![
        "--exact".to_string(),
        "--test-threads=1".to_string(),
        SERVICE_MARKER.to_string(),
        service_name.clone(),
    ];
    let dependency_args = vec![
        "--exact".to_string(),
        "--test-threads=1".to_string(),
        SERVICE_MARKER.to_string(),
        dependency_name.clone(),
    ];

    let spec = ServiceSpec::new(
        ServiceId::new(service_name.clone()).expect("a valid service id"),
        executable.clone(),
        service_args.clone(),
        None,
    )
    .expect("a valid service spec");
    let dependency_spec = ServiceSpec::new(
        ServiceId::new(dependency_name.clone()).expect("a valid service id"),
        executable.clone(),
        dependency_args.clone(),
        None,
    )
    .expect("a valid dependency spec");

    let mut dependency_guard = Disposable::new(dependency_name.clone());
    let mut primary_guard = Disposable::new(service_name.clone());

    // The dependency target is a second disposable registration of this
    // fixture's own, so no pre-existing service is ever named.
    let dependency_install = WindowsScmInstall::new(
        dependency_name.clone(),
        format!("Eggup SCM live dependency {suffix}"),
        WindowsStartType::Manual,
        WindowsErrorControl::Normal,
        None,
    )
    .expect("a valid dependency install")
    .with_transition_timeout(TRANSITION_TIMEOUT)
    .expect("a valid transition timeout");
    let mut dependency_manager = WindowsScmManager::new(dependency_install);
    let dependency_created = dependency_manager
        .install(&dependency_spec)
        .expect("the dependency registration must be created");
    assert!(
        dependency_created.completed,
        "the dependency registration was not confirmed: {}",
        dependency_created.detail
    );
    dependency_guard.arm();

    let install = WindowsScmInstall::new(
        service_name.clone(),
        format!("Eggup SCM live {suffix}"),
        WindowsStartType::Manual,
        WindowsErrorControl::Severe,
        None,
    )
    .expect("a valid install")
    .with_transition_timeout(TRANSITION_TIMEOUT)
    .expect("a valid transition timeout");
    let mut manager = WindowsScmManager::new(install);

    // 1. install — the registration is created, and creation does not start
    //    the service.
    let installed = manager
        .install(&spec)
        .expect("the registration must be created");
    assert!(
        installed.completed,
        "the registration was not confirmed: {}",
        installed.detail
    );
    primary_guard.arm();
    let after_install = manager
        .inspect(&spec)
        .expect("the registration must be observable");
    assert!(
        after_install.was_registered,
        "the registration must be present"
    );
    assert_eq!(
        after_install.state,
        LifecycleState::Stopped,
        "creation must not start the service"
    );

    // 2. inspect — the registration is observed as present, with the exact
    //    parsed command line the SCM holds.
    assert_eq!(after_install.ownership, Ownership::Owned);
    let stored = query_config(&service_name)
        .expect("the configuration must be readable")
        .executable_path
        .to_str()
        .expect("the stored command line must be representable as UTF-8")
        .to_string();
    let parsed: Vec<String> = windows_args::Args::parse_cmd(&stored).collect();
    assert_eq!(
        parsed.first().map(String::as_str),
        Some(executable_text.as_str()),
        "the SCM must hold the exact executable: {parsed:?}"
    );
    assert_eq!(
        &parsed[1..],
        &service_args[..],
        "the SCM must hold the exact arguments: {parsed:?}"
    );

    // 3. ownership — the exact spec is owned, a different executable is
    //    foreign and is never mutated.
    let foreign_spec = ServiceSpec::new(
        ServiceId::new(service_name.clone()).expect("a valid service id"),
        PathBuf::from(r"C:\Other\egg.exe"),
        service_args.clone(),
        None,
    )
    .expect("a valid foreign spec");
    let foreign = manager
        .inspect(&foreign_spec)
        .expect("the registration must be observable");
    assert_eq!(
        foreign.ownership,
        Ownership::Foreign,
        "a different executable must be foreign"
    );
    assert!(
        manager.start(&foreign_spec, TRANSITION_TIMEOUT).is_err(),
        "a foreign registration must deny the mutation"
    );

    // 4. start — the service reaches Running.
    let started = manager
        .start(&spec, TRANSITION_TIMEOUT)
        .expect("the service must start");
    assert!(
        started.completed,
        "the service did not reach Running: {}",
        started.detail
    );
    let deadline = Instant::now() + POLL_DEADLINE;
    poll_until(deadline, || match query_status(&service_name) {
        Ok(state) => Ok((state == ServiceState::Running, format!("{state:?}"))),
        Err(error) => Err(error),
    })
    .expect("the SCM must report Running before the deadline");

    // 5. stop — the service reaches Stopped.
    let stopped = manager
        .stop(&spec, TRANSITION_TIMEOUT)
        .expect("the service must stop");
    assert!(
        stopped.completed,
        "the service did not reach Stopped: {}",
        stopped.detail
    );
    let deadline = Instant::now() + POLL_DEADLINE;
    poll_until(deadline, || match query_status(&service_name) {
        Ok(state) => Ok((state == ServiceState::Stopped, format!("{state:?}"))),
        Err(error) => Err(error),
    })
    .expect("the SCM must report Stopped before the deadline");

    // The `windows-service` mapping layer, asserted explicitly and not only
    // through the end state.
    let config = query_config(&service_name).expect("the configuration must be readable");
    assert_eq!(
        config.start_type,
        ServiceStartType::OnDemand,
        "the start type must round-trip through the SCM"
    );
    assert_eq!(
        config.error_control,
        ServiceErrorControl::Severe,
        "the error control must round-trip through the SCM"
    );
    assert_eq!(
        config.service_type,
        ServiceType::OWN_PROCESS,
        "the service type must round-trip through the SCM"
    );

    // The dependency mapping, against the fixture's own disposable
    // registration.
    let install_with_dependency = WindowsScmInstall::new(
        service_name.clone(),
        format!("Eggup SCM live {suffix}"),
        WindowsStartType::Manual,
        WindowsErrorControl::Severe,
        None,
    )
    .expect("a valid install")
    .with_dependencies(vec![WindowsServiceDependency::Service(
        dependency_name.clone(),
    )])
    .expect("a valid dependency list")
    .with_transition_timeout(TRANSITION_TIMEOUT)
    .expect("a valid transition timeout");
    manager = WindowsScmManager::new(install_with_dependency);
    let refreshed = manager
        .install(&spec)
        .expect("the refresh must be accepted");
    assert!(
        refreshed.completed,
        "the refresh was not confirmed: {}",
        refreshed.detail
    );
    let config = query_config(&service_name).expect("the configuration must be readable");
    assert_eq!(
        config.dependencies,
        vec![ServiceDependency::Service(OsString::from(
            dependency_name.clone()
        ))],
        "the dependency must round-trip through the SCM: {:?}",
        config.dependencies
    );

    // 6. uninstall — the registration is deleted, and the adapter reports
    //    completion only after the SCM confirms the absence.
    let uninstalled = manager
        .uninstall(&spec)
        .expect("the registration must be removed");
    assert!(
        uninstalled.completed,
        "the removal was not confirmed: {}",
        uninstalled.detail
    );
    primary_guard.disarm();
    let after_uninstall = manager
        .inspect(&spec)
        .expect("the absence must be observable");
    assert_eq!(
        after_uninstall.ownership,
        Ownership::Absent,
        "the registration must be gone"
    );

    let dependency_removed = dependency_manager
        .uninstall(&dependency_spec)
        .expect("the dependency registration must be removed");
    assert!(
        dependency_removed.completed,
        "the dependency removal was not confirmed: {}",
        dependency_removed.detail
    );
    dependency_guard.disarm();
}
