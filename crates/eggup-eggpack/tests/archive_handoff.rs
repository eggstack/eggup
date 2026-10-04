//! M002 archive extraction handoff tests.
//!
//! These tests exercise the helpers that translate `ManifestProjection::Archive`
//! into typed inputs for `eggup-archive` and `eggup-core`. They build archives
//! in-memory, run the full bound-source flow, and assert exact size/digest
//! evidence plus advisory-path invariants.
//!
//! The checked-in `archive-manifest.json` fixture carries placeholder member
//! digests, so these tests patch the member `bytes` facts to match the
//! in-memory entry contents before projecting. The patch only rewrites
//! `size`/`sha256` facts for the two known sources; identities, targets, and
//! schema stay exactly as published.

use eggpack_manifest::ReleaseManifest;
use eggup_archive::{
    extract, ArchiveFormat, ArchiveLimits, BoundExtraction, BoundMember, DeferredCleanup,
    ExtractionErrorKind,
};
use eggup_core::{MemberId, PermissionsIntent};
use eggup_eggpack::{
    archive_format_for_name, archive_plan_for, bind_archive_members, core_plan_for_archive,
    project, AdapterError, ManifestProjection,
};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const ARCHIVE_MANIFEST: &str = include_str!("fixtures/archive-manifest.json");
const LINUX_TARGET: &str = "x86_64-unknown-linux-gnu";

const MAIN_SOURCE: &str = "egress";
const MAIN_INSTALL: &str = "egress";
const MAIN_BYTES: &[u8] = b"abc";
const HELPER_SOURCE: &str = "bin/egress-helper";
const HELPER_INSTALL: &str = "egress-helper";
const HELPER_BYTES: &[u8] = b"helper-bytes";

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(label: &str) -> Self {
        let id = TEMP_COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "eggup-eggpack-m002-{}-{}-{}",
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

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// Project the archive fixture with member facts patched to `contents`.
///
/// `contents` maps archive `source` -> exact bytes that will be placed in the
/// test archive. Every patched member keeps its published identity; only the
/// `bytes.size`/`bytes.sha256` facts change.
fn patched_archive_projection(contents: &[(&str, &[u8])]) -> ManifestProjection {
    let mut value: serde_json::Value =
        serde_json::from_str(ARCHIVE_MANIFEST).expect("fixture must be JSON");
    let members = value
        .pointer_mut("/targets/0/form/members")
        .expect("fixture must have archive members")
        .as_array_mut()
        .expect("members must be an array");
    for member in members.iter_mut() {
        let source = member
            .get("source")
            .and_then(|s| s.as_str())
            .expect("member must have a source");
        let (_, bytes) = contents
            .iter()
            .find(|(name, _)| *name == source)
            .expect("test must supply bytes for every fixture member");
        let facts = member
            .get_mut("bytes")
            .expect("member must have byte facts");
        facts["size"] = serde_json::Value::from(bytes.len() as u64);
        facts["sha256"] = serde_json::Value::from(sha256_hex(bytes));
    }
    let manifest =
        ReleaseManifest::from_json(&value.to_string()).expect("patched manifest must validate");
    project(&manifest, LINUX_TARGET).expect("Linux target must be present")
}

/// The standard fixture with every member identity rewritten, so it has the
/// same member count but provably different `source`/`output_name` identities.
fn renamed_archive_projection() -> ManifestProjection {
    let mut value: serde_json::Value =
        serde_json::from_str(ARCHIVE_MANIFEST).expect("fixture must be JSON");
    let members = value
        .pointer_mut("/targets/0/form/members")
        .expect("fixture must have archive members")
        .as_array_mut()
        .expect("members must be an array");
    for member in members.iter_mut() {
        let source = member
            .get("source")
            .and_then(|s| s.as_str())
            .expect("member must have a source")
            .to_string();
        member["source"] = serde_json::Value::from(format!("foreign/{source}"));
        // Install names must be plain filenames, so flatten the basename.
        let basename = source.rsplit('/').next().unwrap_or(source.as_str());
        member["install"] = serde_json::Value::from(format!("foreign-{basename}"));
        // Identical content to the standard fixture: the substitution is
        // digest-invisible.
        let bytes = if source == MAIN_SOURCE {
            MAIN_BYTES
        } else {
            HELPER_BYTES
        };
        member["bytes"]["size"] = serde_json::Value::from(bytes.len() as u64);
        member["bytes"]["sha256"] = serde_json::Value::from(sha256_hex(bytes));
    }
    let manifest =
        ReleaseManifest::from_json(&value.to_string()).expect("renamed manifest must validate");
    project(&manifest, LINUX_TARGET).expect("Linux target must be present")
}

fn standard_projection() -> ManifestProjection {
    patched_archive_projection(&[(MAIN_SOURCE, MAIN_BYTES), (HELPER_SOURCE, HELPER_BYTES)])
}

fn projection_members(projection: &ManifestProjection) -> (Vec<u64>, Vec<[u8; 32]>) {
    match projection {
        ManifestProjection::Archive { members, .. } => (
            members.iter().map(|m| m.exact_size).collect(),
            members.iter().map(|m| m.sha256).collect(),
        ),
        _ => panic!("expected archive projection"),
    }
}

fn permissions_for(projection: &ManifestProjection) -> HashMap<MemberId, PermissionsIntent> {
    match projection {
        ManifestProjection::Archive { members, .. } => members
            .iter()
            .map(|m| {
                (
                    MemberId::new(m.install.clone()).unwrap(),
                    PermissionsIntent::Preserve,
                )
            })
            .collect(),
        _ => panic!("expected archive projection"),
    }
}

fn build_tar_gz(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut bytes = Vec::new();
    {
        let gz = flate2::write::GzEncoder::new(&mut bytes, flate2::Compression::default());
        let mut tar = tar::Builder::new(gz);
        for (name, data) in entries {
            let mut header = tar::Header::new_gnu();
            header.set_size(data.len() as u64);
            header.set_mode(0o644);
            header.set_entry_type(tar::EntryType::Regular);
            tar.append_data(&mut header, name, *data).unwrap();
        }
        tar.into_inner().unwrap().finish().unwrap();
    }
    bytes
}

/// Drain deferred cleanup per the M001b/M001d residue contract.
///
/// Explicit cleanup empties the owned tree via the retained handle and then
/// reports the now-empty directory as `CleanupFailed` residue (no portable
/// object-bound root unlink exists). Accept `Ok` or that exact residue shape;
/// anything else fails the test.
fn finish_cleanup(cleanup: DeferredCleanup, extraction_root: &Path) {
    match cleanup.cleanup() {
        Ok(()) => assert!(!extraction_root.exists()),
        Err(error) => {
            assert_eq!(error.kind(), ExtractionErrorKind::CleanupFailed);
            let residue = error.residue_path().map(|p| p.to_path_buf());
            assert_eq!(residue.as_deref(), Some(extraction_root));
            assert_eq!(fs::read_dir(extraction_root).unwrap().count(), 0);
            fs::remove_dir(extraction_root).unwrap();
        }
    }
}

fn build_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut bytes = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut bytes));
        let options: zip::write::FileOptions<'_, ()> =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
        for (name, data) in entries {
            zip.start_file(*name, options).unwrap();
            zip.write_all(data).unwrap();
        }
        zip.finish().unwrap();
    }
    bytes
}

fn run_handoff(format: ArchiveFormat, archive_bytes: &[u8]) {
    let projection = standard_projection();
    let (exact_sizes, exact_digests) = projection_members(&projection);
    let workdir = TempDir::new("handoff");
    let archive_path = workdir.path().join("archive.bin");
    fs::write(&archive_path, archive_bytes).unwrap();

    let limits = ArchiveLimits::new(
        archive_bytes.len() as u64 + 1,
        16,
        4096,
        exact_sizes.iter().copied().max().unwrap() + 1,
        exact_sizes.iter().sum::<u64>() + 1,
    )
    .unwrap();

    let plan = archive_plan_for(&projection, format, &archive_path, limits).unwrap();

    let extracted = extract(&plan, workdir.path()).unwrap();
    let extraction_root = extracted.root().to_path_buf();
    let bound = extracted.persist().into_bound_sources().unwrap();

    let install_root = workdir.path().join("install");
    fs::create_dir_all(&install_root).unwrap();
    let core_plan = core_plan_for_archive(
        &projection,
        &extraction_root,
        &install_root,
        permissions_for(&projection),
    )
    .unwrap();

    let (members, cleanup): (Vec<BoundMember>, DeferredCleanup) = bound.into_members();
    assert_eq!(members.len(), exact_sizes.len());
    for ((member, size), digest) in members
        .iter()
        .zip(exact_sizes.iter())
        .zip(exact_digests.iter())
    {
        assert_eq!(member.bytes_written(), *size);
        assert_eq!(member.sha256(), *digest);
    }

    let sources = bind_archive_members(&projection, members).unwrap();
    let prepared = core_plan.prepare_with_bound_sources(sources).unwrap();
    assert_eq!(prepared.artifacts().len(), exact_sizes.len());

    // `prepare` stages into an Eggup-owned private stage, never into the
    // install root: prove staged bytes through the prepared transaction.
    let main_id = MemberId::new(MAIN_INSTALL.to_string()).unwrap();
    let helper_id = MemberId::new(HELPER_INSTALL.to_string()).unwrap();
    assert_eq!(
        fs::read(prepared.staged_path(&main_id).unwrap()).unwrap(),
        MAIN_BYTES
    );
    assert_eq!(
        fs::read(prepared.staged_path(&helper_id).unwrap()).unwrap(),
        HELPER_BYTES
    );

    finish_cleanup(cleanup, &extraction_root);
}

#[test]
fn handoff_translates_archive_projection_to_tar_gz_plan_and_stages_bound() {
    let archive_bytes = build_tar_gz(&[(MAIN_SOURCE, MAIN_BYTES), (HELPER_SOURCE, HELPER_BYTES)]);
    run_handoff(ArchiveFormat::TarGz, &archive_bytes);
}

#[test]
fn handoff_translates_archive_projection_to_zip_plan_and_stages_bound() {
    let archive_bytes = build_zip(&[(MAIN_SOURCE, MAIN_BYTES), (HELPER_SOURCE, HELPER_BYTES)]);
    run_handoff(ArchiveFormat::Zip, &archive_bytes);
}

#[test]
fn supported_format_mapping_accepts_tar_gz_variants_and_zip() {
    use eggup_archive::ArchiveFormat;
    assert_eq!(
        archive_format_for_name("egress-3.1.0-x86_64-unknown-linux-gnu.tar.gz").unwrap(),
        ArchiveFormat::TarGz
    );
    assert_eq!(
        archive_format_for_name("EGRESS.TGZ").unwrap(),
        ArchiveFormat::TarGz
    );
    assert_eq!(
        archive_format_for_name("egress-3.1.0.Zip").unwrap(),
        ArchiveFormat::Zip
    );
    for bad in [
        "",
        "egress-3.1.0.tar.bz2",
        "egress-3.1.0.tar",
        "egress-3.1.0.7z",
        "README",
        "egress.zip.exe",
        "egress-\u{e9}.zip",
    ] {
        assert!(
            matches!(
                archive_format_for_name(bad),
                Err(AdapterError::UnsupportedArchiveFormat)
            ),
            "unsupported name must fail closed: {bad:?}"
        );
    }
}

/// Project the fixture with member facts patched to `contents` and
/// archive-level facts patched to `archive_bytes`.
fn fully_patched_projection(
    contents: &[(&str, &[u8])],
    archive_bytes: &[u8],
) -> ManifestProjection {
    let mut value: serde_json::Value =
        serde_json::from_str(ARCHIVE_MANIFEST).expect("fixture must be JSON");
    let form = value
        .pointer_mut("/targets/0/form")
        .expect("fixture must have an archive form");
    form["artifact"]["size"] = serde_json::Value::from(archive_bytes.len() as u64);
    form["artifact"]["sha256"] = serde_json::Value::from(sha256_hex(archive_bytes));
    let members = form
        .get_mut("members")
        .and_then(|m| m.as_array_mut())
        .expect("form must have members");
    for member in members.iter_mut() {
        let source = member["source"].as_str().unwrap().to_string();
        let (_, bytes) = contents
            .iter()
            .find(|(name, _)| *name == source)
            .expect("test must supply bytes for every fixture member");
        member["bytes"]["size"] = serde_json::Value::from(bytes.len() as u64);
        member["bytes"]["sha256"] = serde_json::Value::from(sha256_hex(bytes));
    }
    let manifest =
        ReleaseManifest::from_json(&value.to_string()).expect("patched manifest validates");
    project(&manifest, LINUX_TARGET).expect("Linux target must be present")
}

#[test]
fn acquired_archive_continuity_gate_accepts_exact_bytes_only() {
    let contents = [(MAIN_SOURCE, MAIN_BYTES), (HELPER_SOURCE, HELPER_BYTES)];
    let archive_bytes = build_tar_gz(&contents);
    let full = fully_patched_projection(&contents, &archive_bytes);
    let tmp = TempDir::new("continuity");
    let archive_path = tmp.path().join("archive.tar.gz");
    fs::write(&archive_path, &archive_bytes).unwrap();
    eggup_eggpack::validate_acquired_archive(&full, &archive_path).unwrap();

    // Wrong size fails.
    fs::write(&archive_path, b"short").unwrap();
    assert!(matches!(
        eggup_eggpack::validate_acquired_archive(&full, &archive_path),
        Err(AdapterError::InvalidAcquiredFile(_))
    ));

    // Same size, tampered bytes fail.
    let mut tampered = archive_bytes.clone();
    tampered[10] ^= 0xff;
    fs::write(&archive_path, &tampered).unwrap();
    assert!(matches!(
        eggup_eggpack::validate_acquired_archive(&full, &archive_path),
        Err(AdapterError::InvalidAcquiredFile(_))
    ));

    // Relative path fails without touching the filesystem.
    assert!(matches!(
        eggup_eggpack::validate_acquired_archive(&full, Path::new("relative/archive.tar.gz")),
        Err(AdapterError::InvalidAcquiredFile(_))
    ));

    // Missing file fails.
    assert!(matches!(
        eggup_eggpack::validate_acquired_archive(&full, &tmp.path().join("absent.tar.gz")),
        Err(AdapterError::InvalidAcquiredFile(_))
    ));

    // Installable projections stay on the extraction-required path.
    let direct = ReleaseManifest::from_json(include_str!("fixtures/direct-manifest.json")).unwrap();
    let direct_projection = project(&direct, LINUX_TARGET).unwrap();
    assert!(matches!(
        eggup_eggpack::validate_acquired_archive(&direct_projection, &archive_path),
        Err(AdapterError::ArchiveExtractionRequired)
    ));
}

#[test]
fn crossed_member_contents_fail_before_staging() {
    let projection = standard_projection();
    let tmp = TempDir::new("crossed");
    // Entries swapped between the two declared members: sizes and digests
    // both disagree with the manifest.
    let archive_bytes = build_tar_gz(&[(MAIN_SOURCE, HELPER_BYTES), (HELPER_SOURCE, MAIN_BYTES)]);
    let archive_path = tmp.path().join("archive.tar.gz");
    fs::write(&archive_path, &archive_bytes).unwrap();
    let limits = ArchiveLimits::new(archive_bytes.len() as u64 + 1, 16, 4096, 64, 64).unwrap();
    let plan = archive_plan_for(&projection, ArchiveFormat::TarGz, &archive_path, limits).unwrap();
    assert!(extract(&plan, tmp.path()).is_err());
}

#[test]
fn archive_plan_for_rejects_installable_projection() {
    let manifest = ReleaseManifest::from_json(include_str!("fixtures/direct-manifest.json"))
        .expect("direct must parse");
    let projection = project(&manifest, LINUX_TARGET).unwrap();
    let tmp = TempDir::new("not-archive");
    let archive_path = tmp.path().join("archive.tar.gz");
    fs::write(&archive_path, b"abc").unwrap();
    let limits = ArchiveLimits::new(8, 16, 4096, 8, 8).unwrap();
    let err =
        archive_plan_for(&projection, ArchiveFormat::TarGz, &archive_path, limits).unwrap_err();
    assert!(matches!(err, AdapterError::ArchiveExtractionRequired));
}

#[test]
fn core_plan_and_bind_reject_installable_projection() {
    let manifest = ReleaseManifest::from_json(include_str!("fixtures/direct-manifest.json"))
        .expect("direct must parse");
    let projection = project(&manifest, LINUX_TARGET).unwrap();
    let tmp = TempDir::new("not-archive-core");
    let extraction_root = tmp.path().join("root");
    let install_root = tmp.path().join("install");
    fs::create_dir_all(&install_root).unwrap();
    let err = core_plan_for_archive(&projection, &extraction_root, &install_root, HashMap::new())
        .unwrap_err();
    assert!(matches!(err, AdapterError::ArchiveExtractionRequired));
    let err = bind_archive_members(&projection, Vec::new()).unwrap_err();
    assert!(matches!(err, AdapterError::ArchiveExtractionRequired));
}

#[test]
fn bind_archive_members_fails_closed_on_count_mismatch() {
    let projection = standard_projection();
    let tmp = TempDir::new("bind-mismatch");
    let archive_bytes = build_tar_gz(&[(MAIN_SOURCE, MAIN_BYTES), (HELPER_SOURCE, HELPER_BYTES)]);
    let archive_path = tmp.path().join("archive.tar.gz");
    fs::write(&archive_path, &archive_bytes).unwrap();
    let limits = ArchiveLimits::new(archive_bytes.len() as u64 + 1, 16, 4096, 64, 64).unwrap();
    let plan = archive_plan_for(&projection, ArchiveFormat::TarGz, &archive_path, limits).unwrap();
    let extracted = extract(&plan, tmp.path()).unwrap();
    let bound = extracted.persist().into_bound_sources().unwrap();
    let (members, _cleanup): (Vec<BoundMember>, DeferredCleanup) = bound.into_members();
    assert_eq!(members.len(), 2);
    let truncated: Vec<BoundMember> = members.into_iter().take(1).collect();

    let err = bind_archive_members(&projection, truncated).unwrap_err();
    assert!(matches!(err, AdapterError::MapMismatch(_)));
}

#[test]
fn bind_archive_members_cross_checks_bound_identity_not_just_count() {
    let projection = standard_projection();
    let tmp = TempDir::new("bind-identity");
    let archive_bytes = build_tar_gz(&[(MAIN_SOURCE, MAIN_BYTES), (HELPER_SOURCE, HELPER_BYTES)]);
    let archive_path = tmp.path().join("archive.tar.gz");
    fs::write(&archive_path, &archive_bytes).unwrap();
    let limits = ArchiveLimits::new(archive_bytes.len() as u64 + 1, 16, 4096, 64, 64).unwrap();
    let plan = archive_plan_for(&projection, ArchiveFormat::TarGz, &archive_path, limits).unwrap();
    let extracted = extract(&plan, tmp.path()).unwrap();
    let bound = extracted.persist().into_bound_sources().unwrap();
    let (members, cleanup): (Vec<BoundMember>, DeferredCleanup) = bound.into_members();
    assert_eq!(members.len(), 2);

    // A re-projection that reorders members still satisfies the count check, so
    // only an identity cross-check can reject it. Positional labelling would
    // commit each member's bytes under the other member's declared identity.
    let reversed: Vec<BoundMember> = members.into_iter().rev().collect();
    let err = bind_archive_members(&projection, reversed).unwrap_err();
    assert!(
        matches!(err, AdapterError::MapMismatch(_)),
        "count-matching but identity-mismatched members must fail closed, got {err:?}"
    );
    cleanup.cleanup().ok();

    // Declaration order is the correct pairing and must still succeed, so the
    // cross-check is not over-strict.
    let plan2 = archive_plan_for(
        &projection,
        ArchiveFormat::TarGz,
        &archive_path,
        ArchiveLimits::new(archive_bytes.len() as u64 + 1, 16, 4096, 64, 64).unwrap(),
    )
    .unwrap();
    let extracted2 = extract(&plan2, tmp.path()).unwrap();
    let extraction_root2 = extracted2.root().to_path_buf();
    let bound2 = extracted2.persist().into_bound_sources().unwrap();
    let (ordered, cleanup2): (Vec<BoundMember>, DeferredCleanup) = bound2.into_members();
    let sources = bind_archive_members(&projection, ordered).unwrap();
    let install_root = tmp.path().join("install");
    fs::create_dir_all(&install_root).unwrap();
    let core_plan = core_plan_for_archive(
        &projection,
        &extraction_root2,
        &install_root,
        permissions_for(&projection),
    )
    .unwrap();
    let prepared = core_plan
        .prepare_with_bound_sources(sources)
        .expect("the correct declaration-order pairing must still stage");
    assert_eq!(
        fs::read(
            prepared
                .staged_path(&MemberId::new(MAIN_INSTALL).unwrap())
                .unwrap()
        )
        .unwrap(),
        MAIN_BYTES
    );
    assert_eq!(
        fs::read(
            prepared
                .staged_path(&MemberId::new(HELPER_INSTALL).unwrap())
                .unwrap()
        )
        .unwrap(),
        HELPER_BYTES
    );
    cleanup2.cleanup().ok();
}

#[test]
fn bind_archive_members_rejects_an_extraction_of_a_different_projection() {
    let declared = standard_projection();
    // Same member count, different identities, and identical member content:
    // only an identity cross-check can catch this substitution.
    let foreign = renamed_archive_projection();
    let tmp = TempDir::new("bind-foreign");
    let archive_bytes = build_tar_gz(&[
        ("foreign/egress", MAIN_BYTES),
        ("foreign/bin/egress-helper", HELPER_BYTES),
    ]);
    let archive_path = tmp.path().join("archive.tar.gz");
    fs::write(&archive_path, &archive_bytes).unwrap();
    let limits = ArchiveLimits::new(archive_bytes.len() as u64 + 1, 16, 4096, 64, 64).unwrap();
    let plan = archive_plan_for(&foreign, ArchiveFormat::TarGz, &archive_path, limits).unwrap();
    let extracted = extract(&plan, tmp.path()).unwrap();
    let bound = extracted.persist().into_bound_sources().unwrap();
    let (members, cleanup): (Vec<BoundMember>, DeferredCleanup) = bound.into_members();
    assert_eq!(
        members.len(),
        2,
        "counts must match for the test to be meaningful"
    );

    let err = bind_archive_members(&declared, members).unwrap_err();
    assert!(
        matches!(err, AdapterError::MapMismatch(_)),
        "an extraction of a different projection must fail closed, got {err:?}"
    );
    cleanup.cleanup().ok();
}

#[test]
fn advisory_member_entry_replacement_does_not_redirect_staged_bytes() {
    let projection = standard_projection();
    let tmp = TempDir::new("advisory-replace");
    let archive_bytes = build_tar_gz(&[(MAIN_SOURCE, MAIN_BYTES), (HELPER_SOURCE, HELPER_BYTES)]);
    let archive_path = tmp.path().join("archive.tar.gz");
    fs::write(&archive_path, &archive_bytes).unwrap();

    let limits = ArchiveLimits::new(archive_bytes.len() as u64 + 1, 16, 4096, 64, 64).unwrap();
    let plan = archive_plan_for(&projection, ArchiveFormat::TarGz, &archive_path, limits).unwrap();
    let extracted = extract(&plan, tmp.path()).unwrap();
    let extraction_root = extracted.root().to_path_buf();
    // Replace the advisory member entry after extraction: the bound handle
    // must still prove the owned bytes.
    let replaced = extraction_root.join(MAIN_INSTALL);
    let _ = fs::remove_file(&replaced);
    fs::write(&replaced, b"foreign").unwrap();

    let bound: BoundExtraction = extracted.persist().into_bound_sources().unwrap();
    let (members, cleanup) = bound.into_members();
    assert_eq!(members.len(), 2);
    let (exact_sizes, exact_digests) = projection_members(&projection);
    for ((member, size), digest) in members
        .iter()
        .zip(exact_sizes.iter())
        .zip(exact_digests.iter())
    {
        assert_eq!(member.bytes_written(), *size);
        assert_eq!(member.sha256(), *digest);
    }

    let install_root = tmp.path().join("install");
    fs::create_dir_all(&install_root).unwrap();
    let core_plan = core_plan_for_archive(
        &projection,
        &extraction_root,
        &install_root,
        permissions_for(&projection),
    )
    .unwrap();
    let sources = bind_archive_members(&projection, members).unwrap();
    let prepared = core_plan.prepare_with_bound_sources(sources).unwrap();
    let main_id = MemberId::new(MAIN_INSTALL.to_string()).unwrap();
    assert_eq!(
        fs::read(prepared.staged_path(&main_id).unwrap()).unwrap(),
        MAIN_BYTES
    );
    assert_eq!(fs::read(&replaced).unwrap(), b"foreign");
    cleanup.cleanup().ok();
}

#[test]
fn missing_declared_member_fails_before_staging() {
    let projection = standard_projection();
    let tmp = TempDir::new("missing-member");
    // The archive omits the helper entry the projection declares.
    let archive_bytes = build_tar_gz(&[(MAIN_SOURCE, MAIN_BYTES)]);
    let archive_path = tmp.path().join("archive.tar.gz");
    fs::write(&archive_path, &archive_bytes).unwrap();
    let limits = ArchiveLimits::new(archive_bytes.len() as u64 + 1, 16, 4096, 64, 64).unwrap();
    let plan = archive_plan_for(&projection, ArchiveFormat::TarGz, &archive_path, limits).unwrap();
    let err = extract(&plan, tmp.path()).unwrap_err();
    assert_eq!(err.kind(), ExtractionErrorKind::MissingMember);
}

#[test]
fn wrong_member_digest_fails_before_staging() {
    let projection = standard_projection();
    let tmp = TempDir::new("wrong-digest");
    // Same names and sizes, but the helper bytes differ from the manifest.
    let archive_bytes =
        build_tar_gz(&[(MAIN_SOURCE, MAIN_BYTES), (HELPER_SOURCE, b"tampered!!..")]);
    let archive_path = tmp.path().join("archive.tar.gz");
    fs::write(&archive_path, &archive_bytes).unwrap();
    let limits = ArchiveLimits::new(archive_bytes.len() as u64 + 1, 16, 4096, 64, 64).unwrap();
    let plan = archive_plan_for(&projection, ArchiveFormat::TarGz, &archive_path, limits).unwrap();
    let err = extract(&plan, tmp.path()).unwrap_err();
    assert_eq!(err.kind(), ExtractionErrorKind::DigestMismatch);
}
