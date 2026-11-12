//! A portable child fixture for the bounded-executor tests.
//!
//! The child program is **the test binary itself**, re-executed with a mode
//! argument. That keeps the fixture portable: `std::env::current_exe()` is an
//! absolute path to a real executable on every platform, so it passes
//! `resolve_manager_program` → `validate_executable_path` everywhere — on
//! Unix it needs one execute bit, which a built test binary has — and no
//! shell, no interpreter, and no platform-specific program name is involved.
//!
//! The mode is selected from **argv**, never from the environment: the
//! environment is cleared for the child, which is the property under test.
//!
//! # Wire protocol
//!
//! The child is invoked as
//!
//! ```text
//! <test-binary> --exact --nocapture --test-threads=1 \
//!     eggup_fixture_child eggup-fixture:mode=<mode> eggup-fixture:<key>=<value> ...
//! ```
//!
//! The leading arguments are the test harness's own: the exact test filter
//! selects the single entry-point test, `--nocapture` keeps the payload on the
//! real stdout instead of the harness's capture buffer, and the remaining
//! positional arguments are harness filters that match no test. The
//! `eggup-fixture:` prefixed arguments carry the mode and its options.
//!
//! The payload is framed on stdout between two NUL-delimited markers, so it
//! is recovered byte-exactly regardless of what the harness writes around it.
//! The markers are NUL-delimited so that no payload byte can be confused with
//! a delimiter.

#![allow(dead_code)]

use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::Duration;

/// The name of the test that is the fixture child's entry point.
///
/// The harness's exact-name filter selects this test and no other.
pub const ENTRY_TEST: &str = "eggup_fixture_child";

/// The argument prefix that marks a fixture-child invocation.
const MARKER: &str = "eggup-fixture:";

/// The payload's opening delimiter.
const BEGIN: &[u8] = b"EGGUP-FIXTURE-BEGIN\0";

/// The payload's closing delimiter.
const END: &[u8] = b"\0EGGUP-FIXTURE-END";

/// The absolute path of the running test binary, which is also the child
/// program.
pub fn executable() -> PathBuf {
    std::env::current_exe().expect("the running test binary has an absolute path")
}

/// Builds the harness arguments that turn this test binary into the fixture
/// child running `mode`.
///
/// `options` are extra `key=value` pairs the mode may read.
pub fn child_argv(mode: &str, options: &[(&str, &str)]) -> Vec<String> {
    let mut argv = vec![
        "--exact".to_string(),
        "--nocapture".to_string(),
        "--test-threads=1".to_string(),
        ENTRY_TEST.to_string(),
        format!("{MARKER}mode={mode}"),
    ];
    for (key, value) in options {
        argv.push(format!("{MARKER}{key}={value}"));
    }
    argv
}

/// Recovers the fixture payload from a captured stdout stream.
///
/// The payload is the byte range between the two markers. The harness's own
/// output, which surrounds it, is not part of the payload.
pub fn payload(stdout: &[u8]) -> Vec<u8> {
    let start =
        find(stdout, BEGIN).expect("the fixture payload begin marker is present") + BEGIN.len();
    let end =
        find(&stdout[start..], END).expect("the fixture payload end marker is present") + start;
    stdout[start..end].to_vec()
}

/// Runs the fixture child when this process was started as one.
///
/// Returns the exit code the process must end with, or `None` when this is an
/// ordinary test run and the fixture must do nothing at all.
pub fn run_child() -> Option<i32> {
    let arguments: Vec<String> = std::env::args().collect();
    let mut mode = None;
    let mut options: Vec<(String, String)> = Vec::new();
    for argument in &arguments {
        let Some(rest) = argument.strip_prefix(MARKER) else {
            continue;
        };
        match rest.split_once('=') {
            Some(("mode", value)) => mode = Some(value.to_string()),
            Some((key, value)) => options.push((key.to_string(), value.to_string())),
            None => {}
        }
    }
    let mode = mode?;

    let payload = match mode.as_str() {
        // Echo the whole stdin payload back on stdout.
        "echo-stdin" => {
            let mut buffer = Vec::new();
            let _ = std::io::stdin().read_to_end(&mut buffer);
            buffer
        }
        // Emit a fixed number of filler bytes, so a caller can exceed an
        // output bound.
        "emit-bytes" => {
            let count = option(&options, "bytes")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(0);
            vec![b'x'; count]
        }
        // Print the arguments the child received, one per line.
        "print-argv" => {
            let arguments: Vec<String> = std::env::args().skip(1).collect();
            arguments.join("\n").into_bytes()
        }
        // Print the whole environment, one `name=value` line per variable.
        "print-env" => {
            let mut listing = String::new();
            for (name, value) in std::env::vars() {
                listing.push_str(&name);
                listing.push('=');
                listing.push_str(&value);
                listing.push('\n');
            }
            listing.into_bytes()
        }
        // Create a marker file, so a caller can prove that no child was
        // started at all.
        "touch" => {
            let path = option(&options, "path").unwrap_or_default();
            let _ = std::fs::write(path, b"started");
            Vec::new()
        }
        // Sleep well past any caller's budget, so the caller's deadline ends
        // the run.
        "sleep-ms" => {
            let millis = option(&options, "ms")
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(0);
            std::thread::sleep(Duration::from_millis(millis));
            Vec::new()
        }
        // Exit with a chosen code and no payload.
        "exit-code" => {
            let code = option(&options, "code")
                .and_then(|value| value.parse::<i32>().ok())
                .unwrap_or(0);
            write_payload(&[]);
            return Some(code);
        }
        other => {
            write_payload(other.as_bytes());
            return Some(90);
        }
    };
    write_payload(&payload);
    Some(0)
}

/// Reads one option value.
fn option(options: &[(String, String)], key: &str) -> Option<String> {
    options
        .iter()
        .find(|(name, _)| name == key)
        .map(|(_, value)| value.clone())
}

/// Writes one framed payload to stdout and flushes it.
///
/// The flush matters: the caller reads the payload through a pipe, and the
/// process may end immediately afterwards.
fn write_payload(payload: &[u8]) {
    let mut stdout = std::io::stdout();
    let _ = stdout.write_all(BEGIN);
    let _ = stdout.write_all(payload);
    let _ = stdout.write_all(END);
    let _ = stdout.flush();
}

/// Finds `needle` in `haystack`.
fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
