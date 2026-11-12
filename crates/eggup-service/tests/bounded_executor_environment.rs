#![forbid(unsafe_code)]

//! T9 — the child environment is cleared, and only the session-bus variables
//! are injected.
//!
//! This is a **dedicated test binary**: it sets variables in its own process
//! environment, so `std::env::set_var` cannot race another test in another
//! target. The variables are the three session-bus names the service
//! allowlists for user-scoped systemd, plus a sentinel that must never reach
//! a child.
//!
//! The `--user` injection case needs a child whose argv carries `--user` and
//! whose program is named `systemctl`. The test harness that hosts the
//! portable fixture rejects an unknown `--user` option, so the Unix case uses
//! a small shell program named `systemctl` instead; the portable counterpart
//! of the same policy is the crate-private assertion in the crate's own
//! `mod tests`.

#[path = "common/fixture_child.rs"]
mod fixture_child;

use eggup_service::{CommandExecutor, SystemExecutor};
use std::time::Duration;

/// The sentinel that must never reach a child.
const SENTINEL: &str = "EGGUP_EXECUTOR_SENTINEL";

/// The three names the service allowlists for user-scoped systemd.
#[cfg(unix)]
const SESSION_BUS: [&str; 3] = [
    "DBUS_SESSION_BUS_ADDRESS",
    "SYSTEMD_BUS_ADDRESS",
    "XDG_RUNTIME_DIR",
];

/// The fixture child's entry point.
#[test]
fn eggup_fixture_child() {
    if let Some(code) = fixture_child::run_child() {
        std::process::exit(code);
    }
}

/// Builds the argv that runs the fixture child in `mode`.
fn fixture_argv(mode: &str) -> Vec<String> {
    let mut argv = vec![fixture_child::executable().to_string_lossy().into_owned()];
    argv.extend(fixture_child::child_argv(mode, &[]));
    argv
}

/// The names of the variables the child reported, sorted.
#[cfg(unix)]
fn reported_names(listing: &str) -> Vec<String> {
    let mut names: Vec<String> = listing
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.split_once('=')
                .map(|(name, _)| name.to_string())
                .unwrap_or_else(|| line.to_string())
        })
        .collect();
    names.sort();
    names
}

/// T9 — the environment is cleared, and only the session-bus variables are
/// injected for user-scoped systemd.
#[test]
fn the_child_environment_is_cleared_and_only_the_session_bus_variables_are_injected() {
    // The parent environment carries the three session-bus names and the
    // sentinel. The sentinel's presence is asserted, so a silently ineffective
    // `set_var` cannot make the rest of the test vacuous.
    std::env::set_var("DBUS_SESSION_BUS_ADDRESS", "unix:path=/run/user/1/bus");
    std::env::set_var("XDG_RUNTIME_DIR", "/run/user/1");
    std::env::set_var(
        "SYSTEMD_BUS_ADDRESS",
        "unix:path=/run/user/1/systemd/private",
    );
    std::env::set_var(SENTINEL, "secret");
    assert_eq!(
        std::env::var(SENTINEL).ok(),
        Some("secret".to_string()),
        "the sentinel is set in the parent environment"
    );

    let executor = SystemExecutor::new();

    // (a) A command that is not user-scoped systemd sees an empty
    // environment, and (c) never sees the sentinel.
    let out = executor
        .run(&fixture_argv("print-env"), None, Duration::from_secs(30))
        .expect("the fixture child runs");
    assert_eq!(out.status, Some(0));
    let listing = String::from_utf8_lossy(&fixture_child::payload(&out.stdout)).into_owned();
    assert!(
        listing.trim().is_empty(),
        "the child environment must be empty, got {listing:?}"
    );
    assert!(!listing.contains(SENTINEL));

    // (b) A user-scoped systemd command sees exactly the three session-bus
    // variables, and not the sentinel.
    //
    // The child program must be named `systemctl` for the service's filter to
    // recognise it, and its argv must carry `--user`. Only Unix can run a
    // program whose name has no extension, so this case is Unix-only; the
    // portable counterpart of the same policy is the crate-private assertion
    // in the crate's own `mod tests`.
    #[cfg(unix)]
    {
        let directory = std::env::temp_dir().join(format!(
            "eggup-m015-systemctl-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|since| since.as_nanos())
                .unwrap_or_default()
        ));
        std::fs::create_dir_all(&directory).expect("the temporary directory is created");
        let program = directory.join("systemctl");
        // A copy, not a symlink, so the fixture does not depend on the host's
        // `systemctl` being present. The program is a POSIX shell that prints
        // its environment; the two names the shell adds to its own
        // environment are dropped, so what the test sees is exactly what the
        // service handed to the child.
        std::fs::write(&program, b"#!/bin/sh\nexec env -u SHLVL -u PWD\n")
            .expect("the fixture program is written");
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755))
                .expect("the fixture program is executable");
        }

        let argv = vec![program.to_string_lossy().into_owned(), "--user".to_string()];
        let out = executor
            .run(&argv, None, Duration::from_secs(30))
            .expect("the fixture program runs");
        assert_eq!(out.status, Some(0));
        let listing = String::from_utf8_lossy(&out.stdout).into_owned();
        assert_eq!(
            reported_names(&listing),
            SESSION_BUS
                .iter()
                .map(|name| name.to_string())
                .collect::<Vec<String>>()
        );
        assert!(
            !listing.contains(SENTINEL),
            "the sentinel must not be injected"
        );
        let _ = std::fs::remove_dir_all(&directory);
    }
}
