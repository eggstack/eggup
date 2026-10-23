//! M005 — the published cleanup contract, proven from outside the crate.
//!
//! `eggup-archive` 0.1.3 shipped four `cleanup()` entry points whose shared
//! implementation emptied the owned root and then unconditionally returned
//! `Err(ExtractionErrorKind::CleanupFailed)`. The `Ok` path was therefore
//! unreachable, and the rustdoc promised a "removal" the crate never performed.
//!
//! M005 makes the contract truthful and observable, for every entry point:
//!
//! * `Ok(())` — the owned root object was emptied through the retained root
//!   handle, so the owned tree held no entries at the final handle-relative
//!   listing.
//! * `Err(..)` with `kind() == ExtractionErrorKind::CleanupFailed` — the owned
//!   root could **not** be fully emptied. The residue may be partially
//!   emptied and its emptiness is not guaranteed.
//!
//! The crate never removes the root directory itself, on any path. The
//! residual empty directory is the caller's to remove, and the caller must
//! capture its path from `root()` **before** the consuming cleanup call.

use eggup_archive::{
    extract, ArchiveFormat, ArchiveLimits, ArchiveMember, ArchivePlan, ExtractionErrorKind,
};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_DIR_ID: AtomicU64 = AtomicU64::new(1);

/// Creates an isolated scratch directory for one fixture.
fn test_dir(tag: &str) -> PathBuf {
    let id = NEXT_DIR_ID.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "eggup-archive-m005-{}-{tag}-{id}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Writes a single-entry zip archive with one regular file member.
fn write_zip(path: &Path, name: &str, body: &[u8]) {
    let file = fs::File::create(path).unwrap();
    let mut writer = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(0o644);
    writer.start_file(name, options).unwrap();
    writer.write_all(body).unwrap();
    writer.finish().unwrap();
}

/// Builds a plan that declares exactly one member with no expected facts.
fn single_member_plan(archive: &Path) -> ArchivePlan {
    let limits = ArchiveLimits::strict_default();
    ArchivePlan::new(
        archive.to_path_buf(),
        ArchiveFormat::Zip,
        vec![ArchiveMember::new("app", "app", None, None).unwrap()],
        limits,
    )
    .unwrap()
}

/// Asserts the caller-visible state of a cleaned root: the crate emptied the
/// owned tree but left the root directory itself in place for the caller.
fn assert_root_left_empty_and_removable(root: &Path) {
    assert!(
        root.exists(),
        "the crate must not remove the root directory"
    );
    assert!(root.is_dir(), "the residual root must still be a directory");
    assert_eq!(
        fs::read_dir(root).unwrap().count(),
        0,
        "the residual root must be empty after a successful cleanup"
    );
    // The caller owns removing the residual directory.
    fs::remove_dir(root).unwrap();
    assert!(!root.exists());
}

#[test]
fn cleanup_reports_ok_and_leaves_an_empty_root_for_all_four_entry_points() {
    let dir = test_dir("four-entry-points");

    // 1. `ExtractedArchive::cleanup`.
    let archive = dir.join("one.zip");
    write_zip(&archive, "app", b"payload");
    let plan = single_member_plan(&archive);
    let extracted = extract(&plan, &dir).unwrap();
    let root = extracted.root().to_path_buf();
    extracted
        .cleanup()
        .expect("cleanup must report success when the owned root was emptied");
    assert_root_left_empty_and_removable(&root);

    // 2. `PersistedExtraction::cleanup`.
    let archive = dir.join("two.zip");
    write_zip(&archive, "app", b"payload");
    let plan = single_member_plan(&archive);
    let persisted = extract(&plan, &dir).unwrap().persist();
    let root = persisted.root().to_path_buf();
    persisted
        .cleanup()
        .expect("cleanup must report success when the owned root was emptied");
    assert_root_left_empty_and_removable(&root);

    // 3. `BoundExtraction::cleanup`.
    let archive = dir.join("three.zip");
    write_zip(&archive, "app", b"payload");
    let plan = single_member_plan(&archive);
    let bound = extract(&plan, &dir)
        .unwrap()
        .persist()
        .into_bound_sources()
        .unwrap();
    let root = bound.root().to_path_buf();
    bound
        .cleanup()
        .expect("cleanup must report success when the owned root was emptied");
    assert_root_left_empty_and_removable(&root);

    // 4. `DeferredCleanup::cleanup`.
    let archive = dir.join("four.zip");
    write_zip(&archive, "app", b"payload");
    let plan = single_member_plan(&archive);
    let (members, deferred) = extract(&plan, &dir)
        .unwrap()
        .persist()
        .into_bound_sources()
        .unwrap()
        .into_members();
    let root = deferred.root().to_path_buf();
    drop(members);
    deferred
        .cleanup()
        .expect("cleanup must report success when the owned root was emptied");
    assert_root_left_empty_and_removable(&root);

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn cleanup_reports_no_residue_path_on_the_success_path() {
    let dir = test_dir("no-residue-on-success");
    let archive = dir.join("app.zip");
    write_zip(&archive, "app", b"payload");
    let plan = single_member_plan(&archive);
    let persisted = extract(&plan, &dir).unwrap().persist();
    let root = persisted.root().to_path_buf();

    // The success path yields `Ok(())`: there is no error value at all, so no
    // residue path is reported. The caller captures the residual path from
    // `root()` before the consuming cleanup call instead.
    let result = persisted.cleanup();
    assert!(
        result.is_ok(),
        "cleanup must be able to report success, not only failure"
    );
    assert_root_left_empty_and_removable(&root);

    fs::remove_dir_all(&dir).unwrap();
}

/// The crate never traverses the root pathname, so a foreign replacement
/// installed there survives cleanup untouched.
///
/// The replacement is produced by renaming the operation-owned root aside and
/// installing a foreign directory at the recorded pathname. That rename is
/// refused by Windows while the root directory handle is held, so the
/// replacement fixture is Unix-scoped; the crate's own inline suite carries
/// the Windows reparse-point equivalent.
#[cfg(unix)]
#[test]
fn foreign_replacement_at_the_recorded_pathname_survives_cleanup_untouched() {
    let dir = test_dir("foreign-replacement");
    let archive = dir.join("app.zip");
    write_zip(&archive, "app", b"payload");
    let plan = single_member_plan(&archive);
    let persisted = extract(&plan, &dir).unwrap().persist();
    let root = persisted.root().to_path_buf();
    let moved = dir.join("moved-root");

    fs::rename(&root, &moved).unwrap();
    fs::create_dir(&root).unwrap();
    for index in 0..6 {
        fs::write(root.join(format!("foreign-{index}")), b"keep").unwrap();
    }

    // The owned root object was emptied through the retained handle, so the
    // cleanup succeeds; the foreign directory at the recorded pathname is
    // never traversed and keeps every file.
    persisted
        .cleanup()
        .expect("cleanup must report success when the owned root was emptied");
    assert!(root.exists());
    for index in 0..6 {
        assert_eq!(
            fs::read(root.join(format!("foreign-{index}"))).unwrap(),
            b"keep"
        );
    }
    // The renamed owned root was emptied through the handle.
    assert_eq!(fs::read_dir(&moved).unwrap().count(), 0);
    fs::remove_dir_all(&root).unwrap();
    fs::remove_dir_all(&moved).unwrap();
    fs::remove_dir_all(&dir).unwrap();
}

/// The failure path is fail-closed: `Err` with `CleanupFailed` and a residue
/// path naming the recorded root.
///
/// The crate exposes no portable public failure trigger, so this assertion
/// lives in the crate's own inline suite
/// (`cleanup_that_cannot_empty_the_root_fails_closed_with_residue_evidence`),
/// where the private cleanup bounds are reachable. What is observable from
/// outside the crate is that the failure category is a distinct, documented
/// value rather than the success path.
#[test]
fn cleanup_failure_category_is_distinct_from_success() {
    let dir = test_dir("failure-category");
    let archive = dir.join("app.zip");
    write_zip(&archive, "app", b"payload");
    let plan = single_member_plan(&archive);
    let persisted = extract(&plan, &dir).unwrap().persist();
    let root = persisted.root().to_path_buf();

    let result = persisted.cleanup();
    match result {
        Ok(()) => assert_eq!(fs::read_dir(&root).unwrap().count(), 0),
        Err(error) => {
            assert_eq!(error.kind(), ExtractionErrorKind::CleanupFailed);
            assert_eq!(error.residue_path(), Some(root.as_path()));
        }
    }
    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&dir);
}
