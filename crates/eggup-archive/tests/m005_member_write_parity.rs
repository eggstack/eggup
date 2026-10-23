//! M005 — one member-write path, and per-format parity between tar.gz and zip.
//!
//! M005 moved the member-write body out of `extract_tar_gz` and `extract_zip`
//! into a single shared helper, so the two formats cannot drift apart in what
//! they materialize. This file proves that from outside the crate:
//!
//! * `member_write_path_exists_exactly_once_in_non_test_code` — a source-level
//!   guard that the member creation primitive, the bounded copy with a
//!   populated output, and the expected-value validation each have exactly
//!   one call site in non-test code. It fails if the path is duplicated.
//! * `every_case_produces_identical_outcomes_in_both_formats` — one shared
//!   case table driven through **both** formats, asserting identical
//!   observable outcomes per case.
//! * `windows_device_aliases_are_rejected_and_near_misses_remain_usable` —
//!   the pre-existing device-name regression guard. It is **not** an M005
//!   behavior change and passes before and after M005.

use eggup_archive::{
    extract, ArchiveFormat, ArchiveLimits, ArchiveMember, ArchivePlan, ExtractionErrorKind,
};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_DIR_ID: AtomicU64 = AtomicU64::new(1);

fn test_dir(tag: &str) -> PathBuf {
    let id = NEXT_DIR_ID.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "eggup-archive-m005-parity-{}-{tag}-{id}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn digest(body: &[u8]) -> [u8; 32] {
    Sha256::digest(body).into()
}

/// One archive entry, in the shape both format writers can express.
enum Entry {
    File(&'static str, &'static [u8]),
    Directory(&'static str),
}

/// The observable evidence of one extracted member.
#[derive(Debug, Clone, PartialEq, Eq)]
struct MemberEvidence {
    output_name: String,
    bytes_written: u64,
    sha256: [u8; 32],
}

/// The observable outcome of one case.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Outcome {
    /// The declared members, in declaration order.
    Members(Vec<MemberEvidence>),
    /// The extraction failed with this category.
    Failure(ExtractionErrorKind),
}

/// One row of the shared case table.
struct Case {
    name: &'static str,
    entries: Vec<Entry>,
    members: Vec<ArchiveMember>,
    limits: ArchiveLimits,
    expected: Outcome,
}

fn limits() -> ArchiveLimits {
    ArchiveLimits::new(1024 * 1024, 64, 256, 256 * 1024, 1024 * 1024).unwrap()
}

/// Tight member bound, used by the over-limit case.
fn tight_limits() -> ArchiveLimits {
    ArchiveLimits::new(1024 * 1024, 64, 256, 4, 1024 * 1024).unwrap()
}

/// The shared case table. Every row runs through both formats.
fn cases() -> Vec<Case> {
    vec![
        Case {
            name: "declared-member-evidence",
            entries: vec![Entry::File("app", b"payload")],
            members: vec![
                ArchiveMember::new("app", "app", Some(7), Some(digest(b"payload"))).unwrap(),
            ],
            limits: limits(),
            expected: Outcome::Members(vec![MemberEvidence {
                output_name: "app".to_string(),
                bytes_written: 7,
                sha256: digest(b"payload"),
            }]),
        },
        Case {
            name: "declaration-order-is-preserved",
            entries: vec![Entry::File("b", b"second"), Entry::File("a", b"first")],
            members: vec![
                ArchiveMember::new("a", "a", Some(5), Some(digest(b"first"))).unwrap(),
                ArchiveMember::new("b", "b", Some(6), Some(digest(b"second"))).unwrap(),
            ],
            limits: limits(),
            expected: Outcome::Members(vec![
                MemberEvidence {
                    output_name: "a".to_string(),
                    bytes_written: 5,
                    sha256: digest(b"first"),
                },
                MemberEvidence {
                    output_name: "b".to_string(),
                    bytes_written: 6,
                    sha256: digest(b"second"),
                },
            ]),
        },
        Case {
            name: "missing-member",
            entries: vec![Entry::File("other", b"payload")],
            members: vec![ArchiveMember::new("app", "app", None, None).unwrap()],
            limits: limits(),
            expected: Outcome::Failure(ExtractionErrorKind::MissingMember),
        },
        Case {
            // Two entries that normalize to the same member path: a regular
            // file "app" and a directory "app/". The `zip` writer refuses two
            // identical file names, so the duplicate is expressed through the
            // normalization both formats share.
            name: "duplicate-normalized-member-path",
            entries: vec![Entry::File("app", b"first"), Entry::Directory("app")],
            members: vec![ArchiveMember::new("app", "app", None, None).unwrap()],
            limits: limits(),
            expected: Outcome::Failure(ExtractionErrorKind::DuplicateArchiveMember),
        },
        Case {
            name: "size-mismatch",
            entries: vec![Entry::File("app", b"payload")],
            members: vec![ArchiveMember::new("app", "app", Some(99), None).unwrap()],
            limits: limits(),
            expected: Outcome::Failure(ExtractionErrorKind::SizeMismatch),
        },
        Case {
            name: "digest-mismatch",
            entries: vec![Entry::File("app", b"payload")],
            members: vec![ArchiveMember::new("app", "app", None, Some([0u8; 32])).unwrap()],
            limits: limits(),
            expected: Outcome::Failure(ExtractionErrorKind::DigestMismatch),
        },
        Case {
            name: "non-regular-declared-member",
            entries: vec![Entry::Directory("dir")],
            members: vec![ArchiveMember::new("dir", "dir", None, None).unwrap()],
            limits: limits(),
            expected: Outcome::Failure(ExtractionErrorKind::NonRegularMember),
        },
        Case {
            name: "over-limit-member",
            entries: vec![Entry::File("app", b"0123456789abcdef")],
            members: vec![ArchiveMember::new("app", "app", None, None).unwrap()],
            limits: tight_limits(),
            expected: Outcome::Failure(ExtractionErrorKind::MemberTooLarge),
        },
    ]
}

fn write_tar_gz(path: &Path, entries: &[Entry]) {
    let file = fs::File::create(path).unwrap();
    let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
    let mut builder = tar::Builder::new(encoder);
    for entry in entries {
        match entry {
            Entry::File(name, body) => {
                let mut header = tar::Header::new_gnu();
                header.set_entry_type(tar::EntryType::Regular);
                header.set_mode(0o644);
                header.set_size(body.len() as u64);
                header.set_cksum();
                builder.append_data(&mut header, name, *body).unwrap();
            }
            Entry::Directory(name) => {
                let mut header = tar::Header::new_gnu();
                header.set_entry_type(tar::EntryType::Directory);
                header.set_mode(0o755);
                header.set_size(0);
                header.set_cksum();
                builder
                    .append_data(&mut header, format!("{name}/"), &b""[..])
                    .unwrap();
            }
        }
    }
    builder.into_inner().unwrap().finish().unwrap();
}

fn write_zip(path: &Path, entries: &[Entry]) {
    let file = fs::File::create(path).unwrap();
    let mut writer = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(0o644);
    for entry in entries {
        match entry {
            Entry::File(name, body) => {
                writer.start_file(*name, options).unwrap();
                writer.write_all(body).unwrap();
            }
            Entry::Directory(name) => {
                writer.add_directory(*name, options).unwrap();
            }
        }
    }
    writer.finish().unwrap();
}

/// Runs one case in one format and returns its observable outcome.
///
/// The cleanup outcome is part of the observable: a successful case must clean
/// up with `Ok`, leave the root directory in place, and leave it empty.
fn run_case(case: &Case, dir: &Path, format: ArchiveFormat) -> Outcome {
    let archive = dir.join(match format {
        ArchiveFormat::TarGz => "case.tar.gz",
        ArchiveFormat::Zip => "case.zip",
    });
    match format {
        ArchiveFormat::TarGz => write_tar_gz(&archive, &case.entries),
        ArchiveFormat::Zip => write_zip(&archive, &case.entries),
    }
    let plan = ArchivePlan::new(archive, format, case.members.clone(), case.limits).unwrap();
    match extract(&plan, dir) {
        Ok(extracted) => {
            let evidence = extracted
                .members()
                .iter()
                .map(|member| MemberEvidence {
                    output_name: member.output_name().to_string(),
                    bytes_written: member.bytes_written(),
                    sha256: member.sha256(),
                })
                .collect();
            let root = extracted.root().to_path_buf();
            extracted
                .cleanup()
                .expect("cleanup must report success when the owned root was emptied");
            assert!(
                root.exists(),
                "the crate must not remove the root directory"
            );
            assert_eq!(
                fs::read_dir(&root).unwrap().count(),
                0,
                "the residual root must be empty after a successful cleanup"
            );
            let _ = fs::remove_dir_all(&root);
            Outcome::Members(evidence)
        }
        Err(error) => Outcome::Failure(error.kind()),
    }
}

#[test]
fn every_case_produces_identical_outcomes_in_both_formats() {
    for case in cases() {
        let dir = test_dir(case.name);
        let tar_outcome = run_case(&case, &dir, ArchiveFormat::TarGz);
        let zip_outcome = run_case(&case, &dir, ArchiveFormat::Zip);
        assert_eq!(
            tar_outcome, zip_outcome,
            "case {} diverged between tar.gz and zip",
            case.name
        );
        assert_eq!(
            tar_outcome, case.expected,
            "case {} did not produce the expected outcome",
            case.name
        );
        let _ = fs::remove_dir_all(&dir);
    }
}

/// The member-write path must exist in exactly one place in non-test code.
///
/// The assertion is whitespace-insensitive so that a `rustfmt` line break
/// cannot silently disarm it, and it fails if the path is duplicated.
#[test]
fn member_write_path_exists_exactly_once_in_non_test_code() {
    let source = include_str!("../src/lib.rs");
    let production = source
        .split_once("\n#[cfg(test)]\nmod tests {")
        .map(|(head, _)| head)
        .expect("the inline test module marker must be present in src/lib.rs");
    let dense: String = production
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();

    assert_eq!(
        dense.matches("scope.create_member(").count(),
        1,
        "the member creation primitive must have exactly one call site in non-test code"
    );
    assert_eq!(
        dense.matches("Some(&mutoutput)").count(),
        1,
        "the bounded copy with a populated output must have exactly one call site in non-test code"
    );
    assert_eq!(
        dense.matches("validate_expected(member,").count(),
        1,
        "the expected-value validation must have exactly one call site in non-test code"
    );
    assert_eq!(
        dense.matches("fnwrite_declared_member<").count(),
        1,
        "the shared member-write helper must be defined exactly once"
    );
}

/// Regression guard, not an M005 behavior change: the Windows device-name set
/// stays rejected and its near-misses stay usable.
#[test]
fn windows_device_aliases_are_rejected_and_near_misses_remain_usable() {
    for name in ["COM0", "LPT0", "CONIN$", "CONOUT$", "CLOCK$"] {
        let error = ArchiveMember::new("app", name, None, None).unwrap_err();
        assert_eq!(
            error.kind(),
            ExtractionErrorKind::InvalidPath,
            "name {name:?} must be rejected as a Windows device alias"
        );
    }
    for name in ["com10", "console", "auxiliary", "clock"] {
        assert!(
            ArchiveMember::new("app", name, None, None).is_ok(),
            "name {name:?} must remain a usable output name"
        );
    }
}
