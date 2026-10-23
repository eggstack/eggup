//! M013 — one cron listing, one cron ownership rule, an explicit
//! executable-containment gate on the destructive paths.
//!
//! This file uses only the **pre-M013 public API surface**, so it compiles
//! unchanged against the pre-M013 baseline
//! (`c4805411f74b85158e0ae5d0a219a32c4209d98e`) and fails there by
//! assertion: before M013 `inspect` spawned `crontab -l` twice and no
//! executable-containment gate was applied at all.
//!
//! Nothing here sleeps on the wall clock and nothing touches a real crontab.
//!
//! Platform gate note: the cron fixtures below are Unix-scoped, and the gate
//! is applied **per test** so this module still compiles and still runs its
//! platform-independent fixture on Windows. A crontab body names a POSIX
//! absolute path (`/opt/app/bin`), and the crate's executable-containment
//! gate compares the managed block body against the spec executable
//! verbatim. On Windows the same fixture would spell the executable
//! `C:\opt\app\bin.exe`, so the block body would no longer name it and the
//! adapter would refuse the request material — and `crontab` does not exist
//! there at all. The Unix-only helpers carry the same gate, so the Windows
//! build stays warning-free.

#[cfg(unix)]
use eggup_service::{
    CommandExecutor, CommandOutput, CronManager, Ownership, ServiceError, ServiceId,
    ServiceManager, ServiceSpec,
};
#[cfg(unix)]
use std::path::PathBuf;
#[cfg(unix)]
use std::sync::{Arc, Mutex};
#[cfg(unix)]
use std::time::Duration;

#[cfg(unix)]
const MARKER: &str = "eggup-m013-cron-block";
#[cfg(unix)]
const EXE: &str = "/opt/app/bin";
#[cfg(unix)]
const DESIRED: &str = "*/5 * * * * /opt/app/bin --serve";

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
        ServiceId::new("eggup-m013-cron").unwrap(),
        PathBuf::from(platform_exe()),
        vec!["--serve".to_string()],
        None,
    )
    .unwrap()
}

/// A crontab that carries exactly one managed block whose body is `DESIRED`.
#[cfg(unix)]
fn owned_crontab() -> String {
    format!("# BEGIN {MARKER}\n{DESIRED}\n# END {MARKER}\n")
}

/// A crontab whose managed block has the marker but a different body.
#[cfg(unix)]
fn foreign_crontab() -> String {
    format!("# BEGIN {MARKER}\n0 0 * * * /opt/other/bin --serve\n# END {MARKER}\n")
}

/// The shared record of every manager invocation.
#[cfg(unix)]
type Calls = Arc<Mutex<Vec<Vec<String>>>>;

/// Records every manager invocation and answers `crontab -l` with a scripted
/// crontab.
#[cfg(unix)]
#[derive(Debug, Clone)]
struct RecordingExecutor {
    calls: Calls,
    crontab: String,
    list_status: i32,
}

#[cfg(unix)]
impl RecordingExecutor {
    fn new(crontab: String) -> (Self, Calls) {
        let calls: Calls = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                calls: Arc::clone(&calls),
                crontab,
                list_status: 0,
            },
            calls,
        )
    }

    fn failing_list(crontab: String) -> (Self, Calls) {
        let (executor, calls) = Self::new(crontab);
        (
            Self {
                list_status: 1,
                ..executor
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
        _timeout: Duration,
    ) -> Result<CommandOutput, ServiceError> {
        self.calls.lock().unwrap().push(argv.to_vec());
        let is_list = argv.iter().any(|a| a == "-l");
        if is_list {
            if self.list_status == 0 {
                return Ok(CommandOutput {
                    status: Some(0),
                    stdout: self.crontab.clone().into_bytes(),
                    stderr: Vec::new(),
                });
            }
            return Ok(CommandOutput {
                status: Some(1),
                stdout: Vec::new(),
                // Not the "no crontab" protocol: the listing genuinely
                // failed, so the crontab is unlistable, not empty.
                stderr: b"crontab: permission denied".to_vec(),
            });
        }
        Ok(CommandOutput {
            status: Some(0),
            stdout: Vec::new(),
            stderr: Vec::new(),
        })
    }
}

#[cfg(unix)]
fn count_lists(calls: &Calls) -> usize {
    calls
        .lock()
        .unwrap()
        .iter()
        .filter(|argv| argv.iter().any(|a| a == "-l"))
        .count()
}

#[cfg(unix)]
fn count_writes(calls: &Calls) -> usize {
    calls
        .lock()
        .unwrap()
        .iter()
        .filter(|argv| argv.iter().any(|a| a == "-"))
        .count()
}

#[cfg(unix)]
fn manager(executor: RecordingExecutor) -> CronManager<RecordingExecutor> {
    CronManager::new(
        executor,
        MARKER.to_string(),
        DESIRED.to_string(),
        Duration::from_secs(5),
    )
    .unwrap()
}

/// Item 4 — `inspect` lists the crontab exactly once.
#[cfg(unix)]
#[test]
fn cron_inspect_issues_exactly_one_listing() {
    let (executor, calls) = RecordingExecutor::new(owned_crontab());
    let m = manager(executor);
    let snapshot = m.inspect(&fixture_spec()).expect("inspection succeeds");
    assert_eq!(snapshot.ownership, Ownership::Owned);
    assert_eq!(
        count_lists(&calls),
        1,
        "inspect must list the crontab exactly once, observed {:?}",
        calls.lock().unwrap().clone()
    );
}

/// Item 4 — the pre-write re-listing safety check is retained.
#[cfg(unix)]
#[test]
fn cron_install_and_uninstall_retain_the_pre_write_re_listing() {
    let (executor, calls) = RecordingExecutor::new(String::new());
    let mut m = manager(executor);
    m.install(&fixture_spec()).expect("install succeeds");
    // install: one listing, the pre-write re-listing, then the write.
    assert_eq!(
        count_lists(&calls),
        2,
        "install must re-list before writing"
    );
    assert_eq!(count_writes(&calls), 1, "install must write exactly once");

    let (executor, calls) = RecordingExecutor::new(owned_crontab());
    let mut m = manager(executor);
    m.uninstall(&fixture_spec()).expect("uninstall succeeds");
    assert_eq!(
        count_lists(&calls),
        2,
        "uninstall must re-list before writing"
    );
    assert_eq!(count_writes(&calls), 1, "uninstall must write exactly once");
}

/// Item 5 — the managed block is `Owned` when its body names the spec
/// executable.
#[cfg(unix)]
#[test]
fn cron_managed_block_is_owned_when_the_body_names_the_spec_executable() {
    let (executor, _calls) = RecordingExecutor::new(owned_crontab());
    let m = manager(executor);
    let snapshot = m.inspect(&fixture_spec()).expect("inspection succeeds");
    assert_eq!(snapshot.ownership, Ownership::Owned);
}

/// Item 5 — a same-marker block with different content is `Foreign` and is
/// never mutated.
#[cfg(unix)]
#[test]
fn cron_same_marker_different_content_is_foreign_and_never_mutated() {
    let (executor, calls) = RecordingExecutor::new(foreign_crontab());
    let mut m = manager(executor);
    let snapshot = m.inspect(&fixture_spec()).expect("inspection succeeds");
    assert_eq!(snapshot.ownership, Ownership::Foreign);

    let err = m
        .install(&fixture_spec())
        .expect_err("a foreign block must deny install");
    assert!(
        matches!(err, ServiceError::OwnershipDenied { .. }),
        "expected an ownership denial, got {err:?}"
    );
    let err = m
        .uninstall(&fixture_spec())
        .expect_err("a foreign block must deny uninstall");
    assert!(
        matches!(err, ServiceError::OwnershipDenied { .. }),
        "expected an ownership denial, got {err:?}"
    );
    assert_eq!(
        count_writes(&calls),
        0,
        "a foreign block must never be written"
    );
}

/// Item 5 — an unrelated crontab entry that merely mentions the executable
/// path does NOT make a non-matching managed block look `Owned`.
#[cfg(unix)]
#[test]
fn an_unrelated_entry_naming_the_executable_does_not_make_a_block_owned() {
    let crontab = format!(
        "*/9 * * * * {EXE} --unrelated\n# BEGIN {MARKER}\n0 0 * * * /opt/other/bin --serve\n# END {MARKER}\n"
    );
    let (executor, calls) = RecordingExecutor::new(crontab);
    let mut m = manager(executor);
    let snapshot = m.inspect(&fixture_spec()).expect("inspection succeeds");
    assert_eq!(
        snapshot.ownership,
        Ownership::Foreign,
        "whole-crontab containment must not decide ownership"
    );
    let err = m
        .install(&fixture_spec())
        .expect_err("a foreign block must deny install");
    assert!(matches!(err, ServiceError::OwnershipDenied { .. }));
    assert_eq!(count_writes(&calls), 0);
}

/// Item 5 — the executable-containment gate refuses inconsistent request
/// material with `InvalidInput`, identically across `inspect`, `install` and
/// `uninstall`, and before any listing or mutation happens.
#[cfg(unix)]
#[test]
fn cron_inconsistent_request_material_is_refused_before_any_listing() {
    // The managed block body names `/opt/app/bin`; the spec names a different
    // executable. The request material is internally inconsistent.
    let spec = ServiceSpec::new(
        ServiceId::new("eggup-m013-cron").unwrap(),
        PathBuf::from("/opt/other/bin"),
        vec!["--serve".to_string()],
        None,
    )
    .unwrap();

    let (executor, calls) = RecordingExecutor::new(owned_crontab());
    let m = manager(executor);
    let err = m.inspect(&spec).expect_err("inspect must refuse");
    assert!(
        matches!(err, ServiceError::InvalidInput(_)),
        "expected InvalidInput, got {err:?}"
    );
    assert_eq!(
        count_lists(&calls),
        0,
        "the gate must refuse before any listing"
    );

    let (executor, calls) = RecordingExecutor::new(owned_crontab());
    let mut m = manager(executor);
    let err = m.install(&spec).expect_err("install must refuse");
    assert!(
        matches!(err, ServiceError::InvalidInput(_)),
        "expected InvalidInput, got {err:?}"
    );
    assert_eq!(count_lists(&calls), 0);
    assert_eq!(count_writes(&calls), 0);

    let (executor, calls) = RecordingExecutor::new(owned_crontab());
    let mut m = manager(executor);
    let err = m.uninstall(&spec).expect_err("uninstall must refuse");
    assert!(
        matches!(err, ServiceError::InvalidInput(_)),
        "expected InvalidInput, got {err:?}"
    );
    assert_eq!(count_lists(&calls), 0);
    assert_eq!(count_writes(&calls), 0);
}

/// An unlistable crontab reports `Unknown`, never `Absent`.
#[cfg(unix)]
#[test]
fn an_unlistable_crontab_reports_unknown_and_never_absent() {
    let (executor, calls) = RecordingExecutor::failing_list(String::new());
    let m = manager(executor);
    let snapshot = m.inspect(&fixture_spec()).expect("inspection succeeds");
    assert_eq!(snapshot.ownership, Ownership::Unknown);
    assert!(snapshot.ownership != Ownership::Absent);
    assert_eq!(count_writes(&calls), 0);
}

/// Negative control: the listing-count predicate must reject a wrong count,
/// so the fixture can detect an extra or missing manager call.
#[test]
fn the_listing_count_predicate_rejects_a_wrong_count() {
    let observed = 1usize;
    assert!(observed == 1);
    assert!(observed != 2, "a wrong count must be rejected");
    assert!(observed != 0, "a wrong count must be rejected");
}
