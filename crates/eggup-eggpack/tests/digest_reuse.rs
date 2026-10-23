//! M006 canonical digest reuse fixtures.
//!
//! These fixtures pin the behavior that `eggup-eggpack`'s archive continuity
//! gate must keep across the move from an inline SHA-256 loop to
//! `eggup_core::hash_file`:
//!
//! - the digest **value** is unchanged for a known artifact;
//! - the symlink rejection and the exact-size check still happen, and still
//!   happen **before** the digest computation;
//! - a digest mismatch still fails closed;
//! - the core routine's open failure is mapped to the adapter's own
//!   `AdapterError::InvalidAcquiredFile` variant, not leaked as another
//!   variant;
//! - the adapter's manifest carries **no direct runtime `sha2` dependency**.
//!
//! Fixtures 1-7 are pass-to-pass: they assert behavior that is correct both
//! before and after M006, so they are regression guards against the refactor
//! moving or weakening the ordering and the fail-closed path. They are not
//! red-first proofs. The **only** red-first fixture is the manifest assertion
//! at the end of this file, which fails against the pre-M006 manifest.

use eggup_core::{hash_file, ProductId, ReleaseId};
use eggup_eggpack::{validate_acquired_archive, AdapterError, ManifestProjection};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// The published SHA-256 constant of `b"abc"`.
const ABC_DIGEST: [u8; 32] = [
    0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22, 0x23,
    0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00, 0x15, 0xad,
];
const ABC_BYTES: &[u8] = b"abc";
const ARTIFACT_NAME: &str = "egress.tar.gz";

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(label: &str) -> Self {
        let id = TEMP_COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "eggup-eggpack-m006-{}-{}-{}",
            label,
            std::process::id(),
            id
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// Build an archive projection with exactly the declared size and digest.
fn archive_projection(exact_size: u64, sha256: [u8; 32]) -> ManifestProjection {
    ManifestProjection::Archive {
        product: ProductId::new("egress").unwrap(),
        release: ReleaseId::new("1.0.0").unwrap(),
        target: "x86_64-unknown-linux-gnu".to_string(),
        artifact_name: ARTIFACT_NAME.to_string(),
        exact_size,
        sha256,
        members: Vec::new(),
    }
}

/// Read the dependency keys declared in one manifest table.
///
/// The manifest is a flat list of `key = value` entries per table, so a
/// section scan is exact here and needs no TOML parser dependency — which
/// would itself add a package to the crate's dependency graph.
fn manifest_table_keys(manifest: &str, table: &str) -> Vec<String> {
    let mut keys = Vec::new();
    let mut in_table = false;
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_table = line == table;
            continue;
        }
        if !in_table || line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, _)) = line.split_once('=') {
            keys.push(key.trim().to_string());
        }
    }
    keys
}

fn crate_manifest() -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("the crate manifest must be readable")
}

/// Fixture 1 - the digest value for a known artifact is unchanged.
#[test]
fn known_artifact_digest_matches_the_published_constant_and_the_core_routine() {
    let tmp = TempDir::new("known");
    let path = tmp.path().join(ARTIFACT_NAME);
    fs::write(&path, ABC_BYTES).unwrap();

    // The routine the adapter now delegates to produces the published
    // constant for these bytes.
    assert_eq!(hash_file(&path).unwrap(), ABC_DIGEST);

    // The adapter accepts the file when the projection declares exactly that
    // size and digest, which proves the digest it compared is that value.
    let projection = archive_projection(ABC_BYTES.len() as u64, ABC_DIGEST);
    validate_acquired_archive(&projection, &path).expect("exact bytes must validate");

    // And the two agree on the same file.
    assert_eq!(hash_file(&path).unwrap(), ABC_DIGEST);
}

/// Fixture 2 - a symlinked acquired archive is rejected even when the link
/// target's size and digest both match the manifest exactly.
#[cfg(unix)]
#[test]
fn symlinked_acquired_archive_is_rejected_even_when_the_target_matches() {
    use std::os::unix::fs::symlink;

    let tmp = TempDir::new("symlink");
    let target = tmp.path().join("target.bin");
    fs::write(&target, ABC_BYTES).unwrap();
    let link = tmp.path().join(ARTIFACT_NAME);
    symlink(&target, &link).unwrap();

    // The target's size and digest match the projection exactly, so the only
    // reason for rejection is that the acquired path is a symlink.
    let projection = archive_projection(ABC_BYTES.len() as u64, ABC_DIGEST);
    assert!(matches!(
        validate_acquired_archive(&projection, &link),
        Err(AdapterError::InvalidAcquiredFile(_))
    ));
}

/// Fixture 3 - ordering proof. The symlink target is a FIFO: if the path were
/// opened before the guard ran, the open would block until a writer appeared.
/// The call runs on a worker thread behind a bounded wait, so a blocked
/// implementation fails this fixture instead of hanging the lane.
#[cfg(unix)]
#[test]
fn symlink_guard_precedes_the_open_so_a_fifo_target_never_blocks() {
    use std::os::unix::fs::symlink;
    use std::sync::mpsc;
    use std::time::Duration;

    let tmp = TempDir::new("fifo");
    let fifo = tmp.path().join("target.fifo");
    let status = std::process::Command::new("mkfifo")
        .arg(&fifo)
        .status()
        .expect("mkfifo must be available to build the ordering fixture");
    assert!(status.success(), "mkfifo must create the FIFO");

    let link = tmp.path().join(ARTIFACT_NAME);
    symlink(&fifo, &link).unwrap();

    let projection = archive_projection(0, ABC_DIGEST);
    let (tx, rx) = mpsc::channel();
    let acquired = link.clone();
    std::thread::spawn(move || {
        let accepted = validate_acquired_archive(&projection, &acquired).is_ok();
        let _ = tx.send(accepted);
    });

    match rx.recv_timeout(Duration::from_secs(10)) {
        Ok(accepted) => assert!(
            !accepted,
            "a symlinked FIFO must be rejected by the guard, not opened"
        ),
        Err(mpsc::RecvTimeoutError::Timeout) => panic!(
            "validate_acquired_archive blocked on the FIFO: the symlink guard must run \
             before any open of the acquired path"
        ),
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            panic!("the worker thread ended without producing a result")
        }
    }
}

/// Fixture 4 - a size mismatch is rejected before the digest comparison.
#[test]
fn size_mismatch_is_rejected_before_the_digest_comparison() {
    let tmp = TempDir::new("size");
    let path = tmp.path().join(ARTIFACT_NAME);
    fs::write(&path, ABC_BYTES).unwrap();

    // The content digest matches the manifest exactly; only the declared
    // size is wrong, so the size check is the only possible reason for the
    // rejection.
    let projection = archive_projection(ABC_BYTES.len() as u64 + 1, ABC_DIGEST);
    assert!(matches!(
        validate_acquired_archive(&projection, &path),
        Err(AdapterError::InvalidAcquiredFile(_))
    ));
}

/// Fixture 5 - a digest mismatch still fails closed.
#[test]
fn digest_mismatch_still_fails_closed() {
    let tmp = TempDir::new("mismatch");
    let path = tmp.path().join(ARTIFACT_NAME);
    fs::write(&path, ABC_BYTES).unwrap();

    let mut other = ABC_DIGEST;
    other[0] ^= 0xff;

    // The declared size is exact; only the manifest digest does not match.
    let projection = archive_projection(ABC_BYTES.len() as u64, other);
    assert!(matches!(
        validate_acquired_archive(&projection, &path),
        Err(AdapterError::InvalidAcquiredFile(_))
    ));
}

/// Fixture 6 - the core routine's open failure is mapped to the adapter's own
/// variant instead of being leaked as `AdapterError::Eggup` or any other
/// variant.
#[cfg(unix)]
#[test]
fn digest_open_failure_is_mapped_to_invalid_acquired_file() {
    use std::fs::File;
    use std::os::unix::fs::PermissionsExt;

    let tmp = TempDir::new("open-failure");
    let path = tmp.path().join(ARTIFACT_NAME);
    fs::write(&path, ABC_BYTES).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();

    // A privileged runner can read a mode-000 file, in which case the open
    // failure this fixture needs cannot be produced on this host.
    if File::open(&path).is_ok() {
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        eprintln!(
            "SKIPPED: this host opens a mode-000 file (privileged runner); the \
             open-failure mapping is not exercisable here"
        );
        return;
    }

    let projection = archive_projection(ABC_BYTES.len() as u64, ABC_DIGEST);
    let outcome = validate_acquired_archive(&projection, &path);
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();

    assert!(
        matches!(outcome, Err(AdapterError::InvalidAcquiredFile(_))),
        "the core open failure must be mapped to InvalidAcquiredFile, not leaked as another \
         variant: {outcome:?}"
    );
}

/// Fixture 7 (partial) - the read-failure branch of the same
/// `Err(core_error) -> InvalidAcquiredFile` mapping.
///
/// No deterministic read-failure fixture exists: a truncated file yields EOF
/// rather than an error, and a directory is rejected earlier by the type
/// check. The evidence for the read path is therefore source inspection of
/// the core routine, asserted here when the core source tree is present next
/// to this crate. When it is absent (a packaged crate), the check cannot run
/// and says so.
#[test]
fn core_read_failure_is_reported_as_an_io_error_the_adapter_can_map() {
    let core_source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../eggup-core/src/integrity.rs");
    if !core_source.exists() {
        eprintln!(
            "SKIPPED: the sibling eggup-core source tree is not present at {}; the \
             read-failure branch is evidenced by source inspection in the closure record",
            core_source.display()
        );
        return;
    }
    let source = fs::read_to_string(&core_source).expect("the core source must be readable");
    assert!(
        source.contains("reading file for hashing"),
        "the core routine must report a read failure as an Io error with the \
         \"reading file for hashing\" operation"
    );
}

/// Fixture 8 - red first. The adapter's manifest must not declare a direct
/// runtime `sha2` dependency. This fixture FAILS against the pre-M006
/// manifest, where `sha2 = "0.10.9"` is a `[dependencies]` entry.
#[test]
fn the_adapter_manifest_carries_no_direct_runtime_sha2_dependency() {
    let manifest = crate_manifest();

    let runtime = manifest_table_keys(&manifest, "[dependencies]");
    assert!(
        !runtime.iter().any(|key| key == "sha2"),
        "the adapter must not declare a direct runtime `sha2` dependency; found {runtime:?}"
    );

    // The dev-dependency is deliberately retained: the integration fixtures
    // compute their digests independently of the crate under test. Removing
    // it is a separate decision with its own evidence.
    let development = manifest_table_keys(&manifest, "[dev-dependencies]");
    assert!(
        development.iter().any(|key| key == "sha2"),
        "the `sha2` dev-dependency is retained by M006 for independent fixture digests; \
         found {development:?}"
    );
}
