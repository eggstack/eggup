//! Clean external fixture for the eggup-core 0.1.2 + eggup-archive 0.1.2 pair.
//!
//! This crate lives outside the Eggup workspace and depends on the packaged
//! tarballs via path. It exercises the M001d flow:
//!
//! `PersistedExtraction::into_bound_sources -> BoundExtraction -> BoundSources
//!  -> InstallPlan::prepare_with_bound_sources`
//!
//! on tar.gz and zip archives built from in-memory bytes.

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use eggup_archive::{
    extract, ArchiveFormat, ArchiveLimits, ArchiveMember, ArchivePlan, ExtractionErrorKind,
};
use eggup_core::{
    ArtifactMember, ArtifactSet, BoundSources, InstallPlan, MemberId, PermissionsIntent, ProductId,
    ReleaseId,
};

fn unique_dir(label: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "eggup-pkg-consumer-{}-{}-{}",
        label,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
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

fn sha256(bytes: &[u8]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().into()
}

fn run_extraction_through_bound_sources(archive_bytes: &[u8], format: ArchiveFormat) {
    let workdir = unique_dir("bound");
    let archive_path = workdir.join(format!(
        "archive.{}",
        match format {
            ArchiveFormat::TarGz => "tar.gz",
            ArchiveFormat::Zip => "zip",
        }
    ));
    fs::write(&archive_path, archive_bytes).unwrap();

    let member_name = "egress";
    let member_bytes = b"#!/bin/sh\necho egress\n".to_vec();
    let digest = sha256(&member_bytes);
    let plan = ArchivePlan::new(
        &archive_path,
        format,
        vec![ArchiveMember::new(
            format!("archive/{member_name}"),
            member_name,
            Some(member_bytes.len() as u64),
            Some(digest),
        )
        .unwrap()],
        ArchiveLimits::new(
            archive_bytes.len() as u64 + 1,
            16,
            4096,
            member_bytes.len() as u64 + 1,
            member_bytes.len() as u64 + 1,
        )
        .unwrap(),
    )
    .unwrap();

    let extracted = extract(&plan, &workdir).unwrap();
    let persisted = extracted.persist();
    let bound = persisted.into_bound_sources().unwrap();
    let (members, _cleanup) = bound.into_members();
    assert_eq!(members.len(), 1);
    assert_eq!(members[0].bytes_written(), member_bytes.len() as u64);
    assert_eq!(members[0].sha256(), digest);
    let advisory = members[0].advisory_path().to_path_buf();

    let mut sources = BoundSources::new();
    for m in members {
        let id = MemberId::new(m.output_name().to_string()).unwrap();
        sources.insert(id, m.into_open_object());
    }

    let install_root = workdir.join("install-root");
    fs::create_dir_all(&install_root).unwrap();
    let set = ArtifactSet::new(vec![ArtifactMember::new(
        MemberId::new(member_name.to_string()).unwrap(),
        advisory,
        member_name,
    )
    .unwrap()
    .with_permissions(PermissionsIntent::Executable)
    .with_integrity(eggup_core::IntegrityRequirement::Sha256(digest))])
    .unwrap();

    let plan = InstallPlan::new(
        ProductId::new("egress-fixture").unwrap(),
        ReleaseId::new("fixture-1").unwrap(),
        &install_root,
        set,
    )
    .unwrap();

    let prepared = plan.prepare_with_bound_sources(sources).unwrap();
    assert_eq!(prepared.artifacts().len(), 1);
}

#[test]
fn m001d_flow_through_packaged_core_and_archive_tar_gz() {
    let archive_bytes = build_tar_gz(&[("archive/egress", b"#!/bin/sh\necho egress\n")]);
    run_extraction_through_bound_sources(&archive_bytes, ArchiveFormat::TarGz);
}

#[test]
fn m001d_flow_through_packaged_core_and_archive_zip() {
    let archive_bytes = build_zip(&[("archive/egress", b"#!/bin/sh\necho egress\n")]);
    run_extraction_through_bound_sources(&archive_bytes, ArchiveFormat::Zip);
}

#[test]
fn tar_gz_mismatch_is_rejected() {
    let archive_bytes = build_tar_gz(&[("archive/wrong", b"abcdef")]);
    let workdir = unique_dir("mismatch");
    let archive_path = workdir.join("archive.tar.gz");
    fs::write(&archive_path, &archive_bytes).unwrap();
    let plan = ArchivePlan::new(
        &archive_path,
        ArchiveFormat::TarGz,
        vec![ArchiveMember::new(
            "archive/expected",
            "expected",
            Some(6),
            Some(sha256(b"abcdef")),
        )
        .unwrap()],
        ArchiveLimits::new(
            archive_bytes.len() as u64 + 1,
            16,
            4096,
            1024,
            1024,
        )
        .unwrap(),
    )
    .unwrap();
    let err = extract(&plan, &workdir).unwrap_err();
    assert_eq!(err.kind(), ExtractionErrorKind::MissingMember);
}
