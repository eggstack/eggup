#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Bounded allowlisted extraction of already verified local archives."]
#![doc = ""]
#![doc = "The crate supports regular-file members from tar.gz and zip archives"]
#![doc = "without transport, authenticity, or live-installation policy."]

use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use zip::ZipArchive;

const ROOT_ATTEMPTS: u32 = 32;
const IO_CHUNK_SIZE: usize = 16 * 1024;
const TAR_TRAILING_PADDING_LIMIT: u64 = 1024 * 1024;
const CLEANUP_MAX_PASSES: u32 = 8;
const CLEANUP_MAX_ENTRIES_PER_PASS: usize = 10_000;
const CLEANUP_MAX_DEPTH: usize = 32;
static NEXT_ROOT_ID: AtomicU64 = AtomicU64::new(1);

/// Supported local archive formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveFormat {
    /// A single-member gzip stream containing a tar archive.
    TarGz,
    /// A zip archive using stored or deflate-compressed entries.
    Zip,
}

/// Finite resource limits applied to one extraction operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArchiveLimits {
    max_archive_bytes: u64,
    max_entries: usize,
    max_path_bytes: usize,
    max_member_bytes: u64,
    max_total_uncompressed_bytes: u64,
}

impl ArchiveLimits {
    /// Creates validated, finite limits.
    pub fn new(
        max_archive_bytes: u64,
        max_entries: usize,
        max_path_bytes: usize,
        max_member_bytes: u64,
        max_total_uncompressed_bytes: u64,
    ) -> Result<Self, ExtractionError> {
        if max_archive_bytes == 0
            || max_entries == 0
            || max_entries > 100_000
            || max_path_bytes == 0
            || max_path_bytes > 4096
            || max_member_bytes == 0
            || max_total_uncompressed_bytes == 0
            || max_member_bytes > max_total_uncompressed_bytes
        {
            return Err(ExtractionError::new(ExtractionErrorKind::InvalidPlan));
        }
        Ok(Self {
            max_archive_bytes,
            max_entries,
            max_path_bytes,
            max_member_bytes,
            max_total_uncompressed_bytes,
        })
    }

    /// Conservative finite limits suitable as a starting point for consumers.
    pub fn strict_default() -> Self {
        Self {
            max_archive_bytes: 512 * 1024 * 1024,
            max_entries: 10_000,
            max_path_bytes: 1024,
            max_member_bytes: 256 * 1024 * 1024,
            max_total_uncompressed_bytes: 512 * 1024 * 1024,
        }
    }
}

/// One declared archive path and its single-component extraction filename.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveMember {
    source_path: String,
    output_name: String,
    expected_size: Option<u64>,
    expected_sha256: Option<[u8; 32]>,
}

impl ArchiveMember {
    /// Declares one member. Expected size and SHA-256 are checked when present.
    pub fn new(
        source_path: impl Into<String>,
        output_name: impl Into<String>,
        expected_size: Option<u64>,
        expected_sha256: Option<[u8; 32]>,
    ) -> Result<Self, ExtractionError> {
        let source_path = source_path.into();
        let output_name = output_name.into();
        validate_archive_path(&source_path, false, 4096)?;
        validate_output_name(&output_name, 4096)?;
        Ok(Self {
            source_path,
            output_name,
            expected_size,
            expected_sha256,
        })
    }

    /// Returns the exact normalized member path expected in the archive.
    pub fn source_path(&self) -> &str {
        &self.source_path
    }

    /// Returns the output filename inside the private extraction root.
    pub fn output_name(&self) -> &str {
        &self.output_name
    }
}

/// A validated declaration for one local archive extraction.
#[derive(Debug, Clone)]
pub struct ArchivePlan {
    archive_path: PathBuf,
    format: ArchiveFormat,
    members: Vec<ArchiveMember>,
    limits: ArchiveLimits,
}

impl ArchivePlan {
    /// Creates a plan for an archive whose integrity/authenticity decisions are
    /// already complete in the caller's policy layer.
    pub fn new(
        archive_path: impl Into<PathBuf>,
        format: ArchiveFormat,
        members: Vec<ArchiveMember>,
        limits: ArchiveLimits,
    ) -> Result<Self, ExtractionError> {
        if members.is_empty() || members.len() > limits.max_entries {
            return Err(ExtractionError::new(ExtractionErrorKind::InvalidPlan));
        }
        let mut sources = HashSet::new();
        let mut outputs = HashSet::new();
        let mut expected_total = 0u64;
        for member in &members {
            validate_archive_path(&member.source_path, false, limits.max_path_bytes)?;
            validate_output_name(&member.output_name, limits.max_path_bytes)?;
            if !sources.insert(member.source_path.clone())
                || !outputs.insert(member.output_name.to_ascii_lowercase())
            {
                return Err(ExtractionError::new(ExtractionErrorKind::InvalidPlan));
            }
            if let Some(size) = member.expected_size {
                if size > limits.max_member_bytes {
                    return Err(ExtractionError::new(ExtractionErrorKind::InvalidPlan));
                }
                expected_total = expected_total
                    .checked_add(size)
                    .ok_or_else(|| ExtractionError::new(ExtractionErrorKind::InvalidPlan))?;
            }
        }
        if expected_total > limits.max_total_uncompressed_bytes {
            return Err(ExtractionError::new(ExtractionErrorKind::InvalidPlan));
        }
        Ok(Self {
            archive_path: archive_path.into(),
            format,
            members,
            limits,
        })
    }
}

/// Stable error categories for extraction and cleanup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractionErrorKind {
    /// The plan or a caller-selected directory is invalid.
    InvalidPlan,
    /// The source archive or private output could not be read or written.
    Io,
    /// The archive structure, compression stream, or checksum is malformed.
    MalformedArchive,
    /// A member path is ambiguous, unsupported, or unsafe.
    InvalidPath,
    /// The archive contains more entries than the declared bound.
    TooManyEntries,
    /// The compressed archive file exceeds its declared byte limit.
    ArchiveTooLarge,
    /// A declared member is a link, directory, or special file.
    NonRegularMember,
    /// Two archive entries have the same normalized member path.
    DuplicateArchiveMember,
    /// A required declared member does not occur in the archive.
    MissingMember,
    /// A member exceeds its finite decompressed-byte limit.
    MemberTooLarge,
    /// The operation exceeds its aggregate decompressed-byte limit.
    TotalTooLarge,
    /// A member's observed size differs from its declared exact size.
    SizeMismatch,
    /// A member's observed SHA-256 differs from its declared digest.
    DigestMismatch,
    /// Cleanup failed; the residue path is available from the error.
    CleanupFailed,
}

/// An extraction failure with optional bounded recovery location evidence.
#[derive(Debug)]
pub struct ExtractionError {
    kind: ExtractionErrorKind,
    residue_path: Option<PathBuf>,
}

impl ExtractionError {
    fn new(kind: ExtractionErrorKind) -> Self {
        Self {
            kind,
            residue_path: None,
        }
    }

    fn with_residue(mut self, path: PathBuf) -> Self {
        self.kind = ExtractionErrorKind::CleanupFailed;
        self.residue_path = Some(path);
        self
    }

    fn with_empty_residue(mut self, path: PathBuf) -> Self {
        self.residue_path = Some(path);
        self
    }

    /// Returns the stable failure category.
    pub fn kind(&self) -> ExtractionErrorKind {
        self.kind
    }

    /// Returns the private-root residue location when cleanup failed.
    pub fn residue_path(&self) -> Option<&Path> {
        self.residue_path.as_deref()
    }
}

impl fmt::Display for ExtractionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "archive extraction failed ({:?})", self.kind)
    }
}

impl std::error::Error for ExtractionError {}

/// Verified output evidence for one extracted regular file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedMember {
    source_path: String,
    output_name: String,
    path: PathBuf,
    bytes_written: u64,
    sha256: [u8; 32],
}

impl ExtractedMember {
    /// Returns the normalized archive path that supplied these bytes.
    pub fn source_path(&self) -> &str {
        &self.source_path
    }

    /// Returns the output filename inside the extraction root.
    pub fn output_name(&self) -> &str {
        &self.output_name
    }

    /// Returns the local regular-file path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the exact bytes written and verified.
    pub fn bytes_written(&self) -> u64 {
        self.bytes_written
    }

    /// Returns the SHA-256 digest of the extracted bytes.
    pub fn sha256(&self) -> [u8; 32] {
        self.sha256
    }
}

/// Extraction output whose private directory is removed when dropped.
#[derive(Debug)]
pub struct ExtractedArchive {
    guard: Option<DirectoryGuard>,
    members: Vec<ExtractedMember>,
}

impl ExtractedArchive {
    /// Returns the owner-private extraction root.
    pub fn root(&self) -> &Path {
        self.guard.as_ref().expect("active extraction guard").path()
    }

    /// Returns member evidence in declaration order.
    pub fn members(&self) -> &[ExtractedMember] {
        &self.members
    }

    /// Transfers cleanup responsibility to the caller after transaction
    /// preparation or another explicit ownership handoff.
    pub fn persist(mut self) -> PersistedExtraction {
        let guard = self.guard.take().expect("active extraction guard");
        let (root, handle) = guard.disarm();
        PersistedExtraction {
            root,
            handle,
            members: self.members,
        }
    }

    /// Removes the operation-owned extraction directory and reports residue
    /// evidence if cleanup fails.
    pub fn cleanup(mut self) -> Result<(), ExtractionError> {
        let guard = self.guard.take().expect("active extraction guard");
        guard.cleanup()
    }
}

/// Extracted output whose cleanup responsibility has been transferred.
pub struct PersistedExtraction {
    root: PathBuf,
    handle: File,
    members: Vec<ExtractedMember>,
}

impl std::fmt::Debug for PersistedExtraction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PersistedExtraction")
            .field("root", &self.root)
            .field("members", &self.members)
            .finish_non_exhaustive()
    }
}

impl PersistedExtraction {
    /// Returns the retained private extraction root.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns member evidence in declaration order.
    pub fn members(&self) -> &[ExtractedMember] {
        &self.members
    }

    /// Removes the retained directory when the caller no longer needs it.
    pub fn cleanup(self) -> Result<(), ExtractionError> {
        remove_owned_root_via_handle(self.root, self.handle)
    }
}

/// Extracts the exact declared members from an already verified local archive.
///
/// `output_parent` must already be an existing real directory. The function
/// creates an exclusive child directory there and never touches the live
/// installation. Dropping a successful [`ExtractedArchive`] best-effort empties
/// only that operation-owned child via its retained handle and leaves the
/// now-empty directory as residue; call [`ExtractedArchive::persist`] to
/// transfer that cleanup responsibility.
pub fn extract(
    plan: &ArchivePlan,
    output_parent: &Path,
) -> Result<ExtractedArchive, ExtractionError> {
    let archive_meta = fs::symlink_metadata(&plan.archive_path)
        .map_err(|_| ExtractionError::new(ExtractionErrorKind::Io))?;
    if !archive_meta.is_file() || archive_meta.file_type().is_symlink() {
        return Err(ExtractionError::new(ExtractionErrorKind::InvalidPlan));
    }
    if archive_meta.len() > plan.limits.max_archive_bytes {
        return Err(ExtractionError::new(ExtractionErrorKind::ArchiveTooLarge));
    }
    let (root, handle) = create_private_root(output_parent)?;
    match extract_inner(plan, &root) {
        Ok(members) => Ok(ExtractedArchive {
            guard: Some(DirectoryGuard::new(root, handle)),
            members,
        }),
        Err(error) => match remove_owned_root_with_hook(root.clone(), handle, None) {
            Ok(()) => Err(error),
            Err(_) => Err(error.with_empty_residue(root)),
        },
    }
}

fn extract_inner(plan: &ArchivePlan, root: &Path) -> Result<Vec<ExtractedMember>, ExtractionError> {
    let mut total = 0u64;
    let mut seen = HashSet::new();
    let mut extracted = HashMap::new();
    match plan.format {
        ArchiveFormat::TarGz => extract_tar_gz(plan, root, &mut total, &mut seen, &mut extracted)?,
        ArchiveFormat::Zip => extract_zip(plan, root, &mut total, &mut seen, &mut extracted)?,
    }
    let mut ordered = Vec::with_capacity(plan.members.len());
    for member in &plan.members {
        ordered.push(
            extracted
                .remove(&member.source_path)
                .ok_or_else(|| ExtractionError::new(ExtractionErrorKind::MissingMember))?,
        );
    }
    Ok(ordered)
}

fn extract_tar_gz(
    plan: &ArchivePlan,
    root: &Path,
    total: &mut u64,
    seen: &mut HashSet<String>,
    extracted: &mut HashMap<String, ExtractedMember>,
) -> Result<(), ExtractionError> {
    let input = File::open(&plan.archive_path)
        .map_err(|_| ExtractionError::new(ExtractionErrorKind::Io))?;
    let decoder = GzDecoder::new(input);
    let mut archive = tar::Archive::new(decoder);
    let entries = archive
        .entries()
        .map_err(|_| ExtractionError::new(ExtractionErrorKind::MalformedArchive))?;
    for (index, entry) in entries.enumerate() {
        if index >= plan.limits.max_entries {
            return Err(ExtractionError::new(ExtractionErrorKind::TooManyEntries));
        }
        let mut entry =
            entry.map_err(|_| ExtractionError::new(ExtractionErrorKind::MalformedArchive))?;
        if entry.header().path_bytes().contains(&b'\\') {
            return Err(ExtractionError::new(ExtractionErrorKind::InvalidPath));
        }
        let path_bytes = entry.path_bytes().into_owned();
        let path = std::str::from_utf8(&path_bytes)
            .map_err(|_| ExtractionError::new(ExtractionErrorKind::InvalidPath))?;
        let entry_type = entry.header().entry_type();
        let normalized =
            validate_archive_path(path, entry_type.is_dir(), plan.limits.max_path_bytes)?;
        if !seen.insert(normalized.clone()) {
            return Err(ExtractionError::new(
                ExtractionErrorKind::DuplicateArchiveMember,
            ));
        }
        let declared = plan
            .members
            .iter()
            .find(|member| member.source_path == normalized);
        let is_regular = matches!(entry_type.as_byte(), 0 | b'0');
        if declared.is_some() && !is_regular {
            return Err(ExtractionError::new(ExtractionErrorKind::NonRegularMember));
        }
        if !is_regular {
            drain_bounded(&mut entry, plan.limits, total)?;
            continue;
        }
        if let Some(member) = declared {
            let path = root.join(&member.output_name);
            let mut output = create_private_file(&path)?;
            let (bytes_written, digest) =
                copy_bounded(&mut entry, Some(&mut output), plan.limits, total)?;
            output
                .flush()
                .map_err(|_| ExtractionError::new(ExtractionErrorKind::Io))?;
            validate_expected(member, bytes_written, digest)?;
            extracted.insert(
                normalized.clone(),
                ExtractedMember {
                    source_path: normalized,
                    output_name: member.output_name.clone(),
                    path,
                    bytes_written,
                    sha256: digest,
                },
            );
        } else {
            drain_bounded(&mut entry, plan.limits, total)?;
        }
    }
    // The tar iterator can stop at the tar end marker before the gzip footer
    // is consumed. Drain a strictly bounded zero-padding tail to validate gzip
    // integrity without accepting a concatenated second tar or an inflate tail.
    let mut decoder = archive.into_inner();
    let mut trailing = 0u64;
    let mut buffer = [0u8; IO_CHUNK_SIZE];
    loop {
        let n = decoder
            .read(&mut buffer)
            .map_err(|_| ExtractionError::new(ExtractionErrorKind::MalformedArchive))?;
        if n == 0 {
            break;
        }
        trailing = trailing
            .checked_add(n as u64)
            .ok_or_else(|| ExtractionError::new(ExtractionErrorKind::MalformedArchive))?;
        if trailing > TAR_TRAILING_PADDING_LIMIT || buffer[..n].iter().any(|byte| *byte != 0) {
            return Err(ExtractionError::new(ExtractionErrorKind::MalformedArchive));
        }
    }
    Ok(())
}

fn extract_zip(
    plan: &ArchivePlan,
    root: &Path,
    total: &mut u64,
    seen: &mut HashSet<String>,
    extracted: &mut HashMap<String, ExtractedMember>,
) -> Result<(), ExtractionError> {
    let input = File::open(&plan.archive_path)
        .map_err(|_| ExtractionError::new(ExtractionErrorKind::Io))?;
    let mut archive = ZipArchive::new(input)
        .map_err(|_| ExtractionError::new(ExtractionErrorKind::MalformedArchive))?;
    if archive.len() > plan.limits.max_entries {
        return Err(ExtractionError::new(ExtractionErrorKind::TooManyEntries));
    }
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|_| ExtractionError::new(ExtractionErrorKind::MalformedArchive))?;
        let path = std::str::from_utf8(entry.name_raw())
            .map_err(|_| ExtractionError::new(ExtractionErrorKind::InvalidPath))?;
        let normalized = validate_archive_path(path, entry.is_dir(), plan.limits.max_path_bytes)?;
        if !seen.insert(normalized.clone()) {
            return Err(ExtractionError::new(
                ExtractionErrorKind::DuplicateArchiveMember,
            ));
        }
        let declared = plan
            .members
            .iter()
            .find(|member| member.source_path == normalized);
        let regular = zip_entry_is_regular(&entry);
        if declared.is_some() && !regular {
            return Err(ExtractionError::new(ExtractionErrorKind::NonRegularMember));
        }
        if !regular {
            drain_bounded(&mut entry, plan.limits, total)?;
            continue;
        }
        if let Some(member) = declared {
            let path = root.join(&member.output_name);
            let mut output = create_private_file(&path)?;
            let (bytes_written, digest) =
                copy_bounded(&mut entry, Some(&mut output), plan.limits, total)?;
            output
                .flush()
                .map_err(|_| ExtractionError::new(ExtractionErrorKind::Io))?;
            validate_expected(member, bytes_written, digest)?;
            extracted.insert(
                normalized.clone(),
                ExtractedMember {
                    source_path: normalized,
                    output_name: member.output_name.clone(),
                    path,
                    bytes_written,
                    sha256: digest,
                },
            );
        } else {
            drain_bounded(&mut entry, plan.limits, total)?;
        }
    }
    Ok(())
}

fn zip_entry_is_regular<R: Read>(entry: &zip::read::ZipFile<'_, R>) -> bool {
    if !entry.is_file() || entry.is_symlink() || entry.is_dir() || entry.enclosed_name().is_none() {
        return false;
    }
    entry.unix_mode().is_none_or(|mode| {
        let file_type = mode & 0o170000;
        file_type == 0 || file_type == 0o100000
    })
}

fn copy_bounded<R: Read>(
    input: &mut R,
    mut output: Option<&mut File>,
    limits: ArchiveLimits,
    total: &mut u64,
) -> Result<(u64, [u8; 32]), ExtractionError> {
    let mut member_bytes = 0u64;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; IO_CHUNK_SIZE];
    loop {
        let n = input
            .read(&mut buffer)
            .map_err(|_| ExtractionError::new(ExtractionErrorKind::MalformedArchive))?;
        if n == 0 {
            break;
        }
        let next_member = member_bytes
            .checked_add(n as u64)
            .ok_or_else(|| ExtractionError::new(ExtractionErrorKind::MemberTooLarge))?;
        let next_total = total
            .checked_add(n as u64)
            .ok_or_else(|| ExtractionError::new(ExtractionErrorKind::TotalTooLarge))?;
        if next_member > limits.max_member_bytes {
            return Err(ExtractionError::new(ExtractionErrorKind::MemberTooLarge));
        }
        if next_total > limits.max_total_uncompressed_bytes {
            return Err(ExtractionError::new(ExtractionErrorKind::TotalTooLarge));
        }
        if let Some(file) = output.as_deref_mut() {
            file.write_all(&buffer[..n])
                .map_err(|_| ExtractionError::new(ExtractionErrorKind::Io))?;
        }
        hasher.update(&buffer[..n]);
        member_bytes = next_member;
        *total = next_total;
    }
    let digest: [u8; 32] = hasher.finalize().into();
    Ok((member_bytes, digest))
}

fn drain_bounded<R: Read>(
    input: &mut R,
    limits: ArchiveLimits,
    total: &mut u64,
) -> Result<(), ExtractionError> {
    copy_bounded(input, None, limits, total).map(|_| ())
}

fn validate_expected(
    member: &ArchiveMember,
    observed_size: u64,
    observed_sha256: [u8; 32],
) -> Result<(), ExtractionError> {
    if member
        .expected_size
        .is_some_and(|expected| expected != observed_size)
    {
        return Err(ExtractionError::new(ExtractionErrorKind::SizeMismatch));
    }
    if member
        .expected_sha256
        .is_some_and(|expected| expected != observed_sha256)
    {
        return Err(ExtractionError::new(ExtractionErrorKind::DigestMismatch));
    }
    Ok(())
}

fn validate_archive_path(
    path: &str,
    directory: bool,
    max_path_bytes: usize,
) -> Result<String, ExtractionError> {
    if path.is_empty()
        || path.len() > max_path_bytes
        || path.starts_with('/')
        || path.contains('\\')
        || path.contains(':')
        || path.chars().any(|ch| ch.is_control())
    {
        return Err(ExtractionError::new(ExtractionErrorKind::InvalidPath));
    }
    let candidate = if directory {
        path.strip_suffix('/').unwrap_or(path)
    } else {
        path
    };
    if candidate.is_empty() {
        return Err(ExtractionError::new(ExtractionErrorKind::InvalidPath));
    }
    let path_obj = Path::new(candidate);
    for component in path_obj.components() {
        match component {
            Component::Normal(_) => {}
            _ => return Err(ExtractionError::new(ExtractionErrorKind::InvalidPath)),
        }
    }
    if candidate
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(ExtractionError::new(ExtractionErrorKind::InvalidPath));
    }
    Ok(candidate.to_string())
}

fn validate_output_name(name: &str, max_path_bytes: usize) -> Result<(), ExtractionError> {
    if name.is_empty()
        || name.len() > max_path_bytes
        || name == "."
        || name == ".."
        || name.contains('/')
        || name.contains('\\')
        || name.contains(':')
        || name.ends_with('.')
        || name.ends_with(' ')
        || !name.is_ascii()
        || name
            .chars()
            .any(|ch| ch.is_control() || "<>\"|?*".contains(ch))
    {
        return Err(ExtractionError::new(ExtractionErrorKind::InvalidPath));
    }
    let stem = name.split('.').next().unwrap_or(name).to_ascii_uppercase();
    let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || ["COM", "LPT"].iter().any(|prefix| {
            stem.strip_prefix(prefix).is_some_and(|suffix| {
                suffix.len() == 1 && (b'1'..=b'9').contains(&suffix.as_bytes()[0])
            })
        });
    if reserved {
        return Err(ExtractionError::new(ExtractionErrorKind::InvalidPath));
    }
    Ok(())
}

fn open_dir_handle(path: &Path) -> std::io::Result<File> {
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
        std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
            .open(path)
    }
    #[cfg(not(windows))]
    {
        File::open(path)
    }
}

fn create_private_root(parent: &Path) -> Result<(PathBuf, File), ExtractionError> {
    let meta = fs::symlink_metadata(parent)
        .map_err(|_| ExtractionError::new(ExtractionErrorKind::InvalidPlan))?;
    if !meta.is_dir() || meta.file_type().is_symlink() {
        return Err(ExtractionError::new(ExtractionErrorKind::InvalidPlan));
    }
    let parent_handle =
        open_dir_handle(parent).map_err(|_| ExtractionError::new(ExtractionErrorKind::Io))?;
    for _ in 0..ROOT_ATTEMPTS {
        let id = NEXT_ROOT_ID.fetch_add(1, Ordering::Relaxed);
        let file_name = format!(".eggup-extract-{}-{id:016x}", std::process::id());
        let mut options = fs_at::OpenOptions::default();
        #[cfg(unix)]
        {
            use fs_at::os::unix::OpenOptionsExt;
            options.mode(0o700);
        }
        match options.mkdir_at(&parent_handle, &file_name) {
            Ok(root_handle) => {
                let path = parent.join(&file_name);
                return Ok((path, root_handle));
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(_) => return Err(ExtractionError::new(ExtractionErrorKind::Io)),
        }
    }
    Err(ExtractionError::new(ExtractionErrorKind::Io))
}

fn create_private_file(path: &Path) -> Result<File, ExtractionError> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
        .open(path)
        .map_err(|_| ExtractionError::new(ExtractionErrorKind::Io))
}

/// Recursively deletes only the directory object referenced by the retained
/// handle. All deletions are `*at` operations relative to that handle (or a
/// child handle derived from it); the replacement pathname is never traversed.
/// Symlinks and reparse points are unlinked, never followed.
fn empty_dir_contents(dir: &mut File) -> Result<(), ExtractionError> {
    empty_dir_contents_at_depth(dir, 0)
}

fn empty_dir_contents_at_depth(dir: &mut File, depth: usize) -> Result<(), ExtractionError> {
    if depth > CLEANUP_MAX_DEPTH {
        return Err(ExtractionError::new(ExtractionErrorKind::Io));
    }
    for _ in 0..CLEANUP_MAX_PASSES {
        let names: Vec<std::ffi::OsString> = {
            let iter =
                fs_at::read_dir(dir).map_err(|_| ExtractionError::new(ExtractionErrorKind::Io))?;
            let mut names = Vec::new();
            for entry in iter {
                let entry = entry.map_err(|_| ExtractionError::new(ExtractionErrorKind::Io))?;
                let name = entry.name();
                if name == OsStr::new(".") || name == OsStr::new("..") {
                    continue;
                }
                names.push(name.to_os_string());
                if names.len() > CLEANUP_MAX_ENTRIES_PER_PASS {
                    return Err(ExtractionError::new(ExtractionErrorKind::Io));
                }
            }
            names
        };
        if names.is_empty() {
            return Ok(());
        }
        let mut deleted_any = false;
        for name in &names {
            let name_path = Path::new(name);
            match fs_at::OpenOptions::default().open_dir_at(dir, name_path) {
                Ok(mut child) => {
                    let is_dir = child
                        .metadata()
                        .map(|metadata| metadata.is_dir())
                        .unwrap_or(false);
                    let is_symlink = child
                        .metadata()
                        .map(|metadata| metadata.is_symlink())
                        .unwrap_or(false);
                    if is_dir && !is_symlink {
                        empty_dir_contents_at_depth(&mut child, depth + 1)?;
                        drop(child);
                        match fs_at::OpenOptions::default().rmdir_at(dir, name_path) {
                            Ok(()) => deleted_any = true,
                            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                                deleted_any = true;
                            }
                            Err(_) => {
                                return Err(ExtractionError::new(ExtractionErrorKind::Io));
                            }
                        }
                    } else {
                        drop(child);
                        match fs_at::OpenOptions::default().unlink_at(dir, name_path) {
                            Ok(()) => deleted_any = true,
                            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                                deleted_any = true;
                            }
                            Err(_) => {
                                return Err(ExtractionError::new(ExtractionErrorKind::Io));
                            }
                        }
                    }
                }
                Err(error) => {
                    if error.kind() == std::io::ErrorKind::NotFound {
                        deleted_any = true;
                        continue;
                    }
                    match fs_at::OpenOptions::default().unlink_at(dir, name_path) {
                        Ok(()) => deleted_any = true,
                        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                            deleted_any = true;
                        }
                        Err(_) => {
                            return Err(ExtractionError::new(ExtractionErrorKind::Io));
                        }
                    }
                }
            }
        }
        if !deleted_any {
            return Err(ExtractionError::new(ExtractionErrorKind::Io));
        }
    }
    Err(ExtractionError::new(ExtractionErrorKind::Io))
}

fn remove_owned_root_via_handle(path: PathBuf, handle: File) -> Result<(), ExtractionError> {
    remove_owned_root_with_hook(path, handle, None)
}

fn remove_owned_root_with_hook(
    path: PathBuf,
    mut handle: File,
    hook: Option<&dyn Fn(&Path)>,
) -> Result<(), ExtractionError> {
    let empty_result = empty_dir_contents(&mut handle);
    if let Some(hook) = hook {
        hook(&path);
    }
    let _ = empty_result;
    Err(ExtractionError::new(ExtractionErrorKind::CleanupFailed).with_residue(path))
}

/// Retained directory authority for an extraction root.
///
/// The open [`File`] handle is captured atomically at creation time via
/// `mkdir_at` before any archive bytes are materialized. Recursive cleanup
/// operates only through that handle (and child handles derived from it) via
/// `fs_at` `*at` operations. The original pathname is used only as residue
/// evidence; it is never recursively traversed or deleted.
struct DirectoryGuard {
    path: PathBuf,
    handle: Option<File>,
    active: bool,
}

impl std::fmt::Debug for DirectoryGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DirectoryGuard")
            .field("path", &self.path)
            .field("active", &self.active)
            .finish_non_exhaustive()
    }
}

impl DirectoryGuard {
    fn new(path: PathBuf, handle: File) -> Self {
        Self {
            path,
            handle: Some(handle),
            active: true,
        }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn disarm(mut self) -> (PathBuf, File) {
        self.active = false;
        let handle = self.handle.take().expect("active extraction guard");
        (self.path.clone(), handle)
    }

    fn cleanup(mut self) -> Result<(), ExtractionError> {
        self.active = false;
        let handle = self.handle.take().expect("active extraction guard");
        remove_owned_root_via_handle(self.path.clone(), handle)
    }
}

impl Drop for DirectoryGuard {
    fn drop(&mut self) {
        if self.active {
            if let Some(handle) = self.handle.as_mut() {
                let _ = empty_dir_contents(handle);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eggup_core::{ArtifactMember, ArtifactSet, InstallPlan, MemberId, ProductId, ReleaseId};
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Cursor;
    use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
    use tar::{Builder as TarBuilder, EntryType as TarEntryType, Header};
    use zip::write::SimpleFileOptions;
    use zip::{CompressionMethod, ZipWriter};

    static NEXT_TEST_DIR: AtomicU64 = AtomicU64::new(1);

    struct TestDir(PathBuf);

    impl TestDir {
        fn new() -> Self {
            let id = NEXT_TEST_DIR.fetch_add(1, AtomicOrdering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "eggup-archive-test-{}-{id:016x}",
                std::process::id()
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    struct TarFixture<'a> {
        path: &'a str,
        kind: u8,
        body: &'a [u8],
        link: Option<&'a str>,
    }

    fn limits() -> ArchiveLimits {
        ArchiveLimits::new(2 * 1024 * 1024, 32, 256, 1024, 2048).unwrap()
    }

    fn member(path: &str, output: &str, bytes: &[u8]) -> ArchiveMember {
        ArchiveMember::new(
            path,
            output,
            Some(bytes.len() as u64),
            Some(Sha256::digest(bytes).into()),
        )
        .unwrap()
    }

    fn make_plan(
        archive: PathBuf,
        format: ArchiveFormat,
        members: Vec<ArchiveMember>,
        limits: ArchiveLimits,
    ) -> ArchivePlan {
        ArchivePlan::new(archive, format, members, limits).unwrap()
    }

    fn tar_file(dir: &Path, name: &str, entries: &[TarFixture<'_>]) -> PathBuf {
        let archive_path = dir.join(name);
        let gzip = GzEncoder::new(Vec::new(), Compression::default());
        let mut builder = TarBuilder::new(gzip);
        for entry in entries {
            let mut header = Header::new_gnu();
            header.set_entry_type(TarEntryType::new(entry.kind));
            header.set_size(entry.body.len() as u64);
            header.set_mode(0o644);
            header.set_path(entry.path).unwrap();
            if let Some(link) = entry.link {
                header.set_link_name(link).unwrap();
            }
            header.set_cksum();
            builder.append(&header, entry.body).unwrap();
        }
        let gzip = builder.into_inner().unwrap();
        let bytes = gzip.finish().unwrap();
        fs::write(&archive_path, bytes).unwrap();
        archive_path
    }

    fn zip_file(dir: &Path, name: &str, entries: &[(&str, &[u8])]) -> PathBuf {
        let path = dir.join(name);
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        for (entry_name, body) in entries {
            writer.start_file(*entry_name, options).unwrap();
            writer.write_all(body).unwrap();
        }
        let bytes = writer.finish().unwrap().into_inner();
        fs::write(&path, bytes).unwrap();
        path
    }

    fn extract_error(plan: &ArchivePlan, dir: &Path) -> ExtractionErrorKind {
        extract(plan, dir).unwrap_err().kind()
    }

    #[test]
    fn valid_tar_gz_extracts_allowlisted_members_in_declaration_order() {
        let dir = TestDir::new();
        let archive = tar_file(
            dir.path(),
            "bundle.tar.gz",
            &[
                TarFixture {
                    path: "bin/helper",
                    kind: b'0',
                    body: b"helper-v1",
                    link: None,
                },
                TarFixture {
                    path: "bin/main",
                    kind: b'0',
                    body: b"main-v1",
                    link: None,
                },
                TarFixture {
                    path: "notes.txt",
                    kind: b'0',
                    body: b"ignored",
                    link: None,
                },
            ],
        );
        let plan = make_plan(
            archive,
            ArchiveFormat::TarGz,
            vec![
                member("bin/main", "main", b"main-v1"),
                member("bin/helper", "helper", b"helper-v1"),
            ],
            limits(),
        );
        let extracted = extract(&plan, dir.path()).unwrap();
        assert_eq!(extracted.members()[0].source_path(), "bin/main");
        assert_eq!(extracted.members()[1].source_path(), "bin/helper");
        assert_eq!(fs::read(extracted.members()[0].path()).unwrap(), b"main-v1");
        assert!(!extracted.root().join("notes.txt").exists());
        assert_eq!(extracted.members()[0].bytes_written(), 7);
    }

    #[test]
    fn valid_zip_extracts_members_and_returns_digest_evidence() {
        let dir = TestDir::new();
        let archive = zip_file(
            dir.path(),
            "bundle.zip",
            &[("app.exe", b"windows-bin"), ("helper.dll", b"helper")],
        );
        let plan = make_plan(
            archive,
            ArchiveFormat::Zip,
            vec![
                member("app.exe", "app.exe", b"windows-bin"),
                member("helper.dll", "helper.dll", b"helper"),
            ],
            limits(),
        );
        let extracted = extract(&plan, dir.path()).unwrap();
        assert_eq!(
            fs::read(extracted.members()[0].path()).unwrap(),
            b"windows-bin"
        );
        assert_eq!(fs::read(extracted.members()[1].path()).unwrap(), b"helper");
        let expected_digest: [u8; 32] = Sha256::digest(b"windows-bin").into();
        assert_eq!(extracted.members()[0].sha256(), expected_digest);
    }

    #[cfg(unix)]
    #[test]
    fn extraction_root_and_files_are_owner_private() {
        use std::os::unix::fs::PermissionsExt;

        let dir = TestDir::new();
        let archive = zip_file(dir.path(), "bundle.zip", &[("app", b"binary")]);
        let plan = make_plan(
            archive,
            ArchiveFormat::Zip,
            vec![member("app", "app", b"binary")],
            limits(),
        );
        let extracted = extract(&plan, dir.path()).unwrap();
        assert_eq!(
            fs::metadata(extracted.root()).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(extracted.members()[0].path())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }

    #[test]
    fn archive_paths_reject_traversal_absolute_prefix_alias_and_overlong_names() {
        for path in [
            "",
            "../escape",
            "a/../../escape",
            "/etc/passwd",
            "C:/Windows/x",
            "a\\b",
            "a//b",
            "./app",
        ] {
            assert_eq!(
                ArchiveMember::new(path, "app", None, None)
                    .unwrap_err()
                    .kind(),
                ExtractionErrorKind::InvalidPath,
                "path {path:?}"
            );
        }
        assert_eq!(
            ArchiveMember::new("a".repeat(5000), "app", None, None)
                .unwrap_err()
                .kind(),
            ExtractionErrorKind::InvalidPath
        );
    }

    #[test]
    fn output_names_reject_windows_reserved_or_ambiguous_names() {
        for name in ["../app", "a/b", "a\\b", "CON", "nul.txt", "app.", "app "] {
            assert_eq!(
                ArchiveMember::new("app", name, None, None)
                    .unwrap_err()
                    .kind(),
                ExtractionErrorKind::InvalidPath,
                "name {name:?}"
            );
        }
        assert_eq!(
            ArchivePlan::new(
                "archive.zip",
                ArchiveFormat::Zip,
                vec![
                    ArchiveMember::new("one", "App", None, None).unwrap(),
                    ArchiveMember::new("two", "app", None, None).unwrap()
                ],
                limits(),
            )
            .unwrap_err()
            .kind(),
            ExtractionErrorKind::InvalidPlan
        );
    }

    #[test]
    fn plan_rejects_duplicate_members_and_nonpositive_or_inconsistent_limits() {
        let duplicate = ArchiveMember::new("bin/app", "app", None, None).unwrap();
        assert_eq!(
            ArchivePlan::new(
                "x",
                ArchiveFormat::Zip,
                vec![duplicate.clone(), duplicate],
                limits()
            )
            .unwrap_err()
            .kind(),
            ExtractionErrorKind::InvalidPlan
        );
        assert!(ArchiveLimits::new(0, 1, 10, 10, 10).is_err());
        assert!(ArchiveLimits::new(10, 0, 10, 10, 10).is_err());
        assert!(ArchiveLimits::new(10, 1, 0, 10, 10).is_err());
        assert!(ArchiveLimits::new(10, 1, 10, 11, 10).is_err());
    }

    #[test]
    fn missing_and_duplicate_archive_members_fail_closed() {
        let dir = TestDir::new();
        let missing = tar_file(dir.path(), "missing.tar.gz", &[]);
        let plan = make_plan(
            missing,
            ArchiveFormat::TarGz,
            vec![ArchiveMember::new("app", "app", None, None).unwrap()],
            limits(),
        );
        assert_eq!(
            extract_error(&plan, dir.path()),
            ExtractionErrorKind::MissingMember
        );

        let duplicate = tar_file(
            dir.path(),
            "duplicate.tar.gz",
            &[
                TarFixture {
                    path: "app",
                    kind: b'0',
                    body: b"first",
                    link: None,
                },
                TarFixture {
                    path: "app",
                    kind: b'0',
                    body: b"second",
                    link: None,
                },
            ],
        );
        let plan = make_plan(
            duplicate,
            ArchiveFormat::TarGz,
            vec![ArchiveMember::new("app", "app", None, None).unwrap()],
            limits(),
        );
        assert_eq!(
            extract_error(&plan, dir.path()),
            ExtractionErrorKind::DuplicateArchiveMember
        );
    }

    #[test]
    fn declared_tar_links_and_special_files_are_rejected() {
        for (name, kind, link) in [
            ("symlink", b'2', Some("target")),
            ("hardlink", b'1', Some("target")),
            ("fifo", b'6', None),
            ("device", b'3', None),
        ] {
            let dir = TestDir::new();
            let archive = tar_file(
                dir.path(),
                "bad.tar.gz",
                &[TarFixture {
                    path: name,
                    kind,
                    body: b"",
                    link,
                }],
            );
            let plan = make_plan(
                archive,
                ArchiveFormat::TarGz,
                vec![ArchiveMember::new(name, "output", None, None).unwrap()],
                limits(),
            );
            assert_eq!(
                extract_error(&plan, dir.path()),
                ExtractionErrorKind::NonRegularMember,
                "{name}"
            );
        }
    }

    #[test]
    fn declared_zip_symlink_is_rejected() {
        let dir = TestDir::new();
        let path = dir.path().join("symlink.zip");
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        writer
            .add_symlink("app", "target", SimpleFileOptions::default())
            .unwrap();
        fs::write(path, writer.finish().unwrap().into_inner()).unwrap();
        let plan = make_plan(
            dir.path().join("symlink.zip"),
            ArchiveFormat::Zip,
            vec![ArchiveMember::new("app", "app", None, None).unwrap()],
            limits(),
        );
        assert_eq!(
            extract_error(&plan, dir.path()),
            ExtractionErrorKind::NonRegularMember
        );
    }

    #[test]
    fn per_member_and_aggregate_decompression_limits_are_enforced() {
        let dir = TestDir::new();
        let archive = zip_file(dir.path(), "large.zip", &[("app", b"123456789")]);
        let member_limit = ArchiveLimits::new(1024, 10, 100, 8, 20).unwrap();
        let plan = make_plan(
            archive.clone(),
            ArchiveFormat::Zip,
            vec![ArchiveMember::new("app", "app", None, None).unwrap()],
            member_limit,
        );
        assert_eq!(
            extract_error(&plan, dir.path()),
            ExtractionErrorKind::MemberTooLarge
        );

        let archive = zip_file(
            dir.path(),
            "aggregate.zip",
            &[("one", b"12345678"), ("two", b"abcdefgh")],
        );
        let aggregate_limit = ArchiveLimits::new(2048, 10, 100, 8, 12).unwrap();
        let plan = make_plan(
            archive,
            ArchiveFormat::Zip,
            vec![
                ArchiveMember::new("one", "one", None, None).unwrap(),
                ArchiveMember::new("two", "two", None, None).unwrap(),
            ],
            aggregate_limit,
        );
        assert_eq!(
            extract_error(&plan, dir.path()),
            ExtractionErrorKind::TotalTooLarge
        );
    }

    #[test]
    fn entry_count_and_compressed_archive_size_are_bounded() {
        let dir = TestDir::new();
        let archive = zip_file(dir.path(), "entries.zip", &[("app", b"a"), ("extra", b"b")]);
        let entry_limit = ArchiveLimits::new(2048, 1, 100, 10, 10).unwrap();
        let plan = make_plan(
            archive.clone(),
            ArchiveFormat::Zip,
            vec![ArchiveMember::new("app", "app", None, None).unwrap()],
            entry_limit,
        );
        assert_eq!(
            extract_error(&plan, dir.path()),
            ExtractionErrorKind::TooManyEntries
        );

        let bytes = fs::metadata(&archive).unwrap().len();
        let archive_limit = ArchiveLimits::new(bytes - 1, 10, 100, 10, 10).unwrap();
        let plan = make_plan(
            archive,
            ArchiveFormat::Zip,
            vec![ArchiveMember::new("app", "app", None, None).unwrap()],
            archive_limit,
        );
        assert_eq!(
            extract_error(&plan, dir.path()),
            ExtractionErrorKind::ArchiveTooLarge
        );
    }

    #[test]
    fn exact_size_and_sha256_mismatches_fail() {
        let dir = TestDir::new();
        let archive = zip_file(dir.path(), "verify.zip", &[("app", b"correct")]);
        let wrong_size = ArchiveMember::new("app", "app", Some(99), None).unwrap();
        let plan = make_plan(
            archive.clone(),
            ArchiveFormat::Zip,
            vec![wrong_size],
            limits(),
        );
        assert_eq!(
            extract_error(&plan, dir.path()),
            ExtractionErrorKind::SizeMismatch
        );

        let wrong_digest = ArchiveMember::new("app", "app", None, Some([0; 32])).unwrap();
        let plan = make_plan(archive, ArchiveFormat::Zip, vec![wrong_digest], limits());
        assert_eq!(
            extract_error(&plan, dir.path()),
            ExtractionErrorKind::DigestMismatch
        );
    }

    #[test]
    fn corrupt_and_truncated_tar_gz_and_zip_fail() {
        let dir = TestDir::new();
        let tar = tar_file(
            dir.path(),
            "ok.tar.gz",
            &[TarFixture {
                path: "app",
                kind: b'0',
                body: b"ok",
                link: None,
            }],
        );
        let mut bytes = fs::read(&tar).unwrap();
        bytes.truncate(bytes.len() - 4);
        let broken_tar = dir.path().join("broken.tar.gz");
        fs::write(&broken_tar, bytes).unwrap();
        let plan = make_plan(
            broken_tar,
            ArchiveFormat::TarGz,
            vec![ArchiveMember::new("app", "app", None, None).unwrap()],
            limits(),
        );
        assert_ne!(extract_error(&plan, dir.path()), ExtractionErrorKind::Io);

        let zip = zip_file(dir.path(), "ok.zip", &[("app", b"ok")]);
        let mut bytes = fs::read(zip).unwrap();
        bytes.truncate(bytes.len() / 2);
        let broken_zip = dir.path().join("broken.zip");
        fs::write(&broken_zip, bytes).unwrap();
        let plan = make_plan(
            broken_zip,
            ArchiveFormat::Zip,
            vec![ArchiveMember::new("app", "app", None, None).unwrap()],
            limits(),
        );
        assert_eq!(
            extract_error(&plan, dir.path()),
            ExtractionErrorKind::MalformedArchive
        );
    }

    #[test]
    fn failure_cleans_only_its_owned_partial_root_and_drop_cleans_success() {
        let dir = TestDir::new();
        let sentinel = dir.path().join("foreign");
        fs::write(&sentinel, b"keep").unwrap();
        let archive = zip_file(dir.path(), "bad.zip", &[("app", b"123456789")]);
        let cap = ArchiveLimits::new(2048, 10, 100, 8, 10).unwrap();
        let plan = make_plan(
            archive,
            ArchiveFormat::Zip,
            vec![ArchiveMember::new("app", "app", None, None).unwrap()],
            cap,
        );
        let error = extract(&plan, dir.path()).unwrap_err();
        assert_eq!(error.kind(), ExtractionErrorKind::MemberTooLarge);
        assert_eq!(fs::read(&sentinel).unwrap(), b"keep");
        // Handle-bound failure cleanup empties the owned partial root via its
        // retained handle and leaves the now-empty directory as bounded
        // residue; it never touches the foreign sentinel.
        let residue = error.residue_path().expect("empty residue").to_path_buf();
        assert!(residue.exists());
        assert_eq!(fs::read_dir(&residue).unwrap().count(), 0);
        fs::remove_dir(&residue).unwrap();
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 2);

        let archive = zip_file(dir.path(), "good.zip", &[("app", b"ok")]);
        let plan = make_plan(
            archive,
            ArchiveFormat::Zip,
            vec![ArchiveMember::new("app", "app", None, None).unwrap()],
            limits(),
        );
        let root = {
            let output = extract(&plan, dir.path()).unwrap();
            output.root().to_path_buf()
        };
        // Drop best-effort empties the owned root via its handle but never
        // falls back to pathname recursion, so the now-empty directory
        // remains as residue.
        assert!(root.exists());
        assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
        fs::remove_dir(&root).unwrap();
    }

    #[test]
    fn persist_transfers_cleanup_responsibility_and_cleanup_is_explicit() {
        let dir = TestDir::new();
        let archive = zip_file(dir.path(), "good.zip", &[("app", b"ok")]);
        let plan = make_plan(
            archive,
            ArchiveFormat::Zip,
            vec![ArchiveMember::new("app", "app", None, None).unwrap()],
            limits(),
        );
        let persisted = extract(&plan, dir.path()).unwrap().persist();
        let root = persisted.root().to_path_buf();
        assert!(root.exists());
        // No object-bound root unlink exists on all platforms, so explicit
        // cleanup empties only the owned tree via its handle and reports the
        // now-empty directory as residue instead of recursively touching the
        // pathname.
        let error = persisted.cleanup().unwrap_err();
        assert_eq!(error.kind(), ExtractionErrorKind::CleanupFailed);
        assert_eq!(error.residue_path(), Some(root.as_path()));
        assert!(root.exists());
        assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
        fs::remove_dir(&root).unwrap();
    }

    #[test]
    fn cleanup_failure_reports_residue_without_deleting_replacement() {
        let dir = TestDir::new();
        let archive = zip_file(dir.path(), "bundle.zip", &[("app", b"binary")]);
        let plan = make_plan(
            archive,
            ArchiveFormat::Zip,
            vec![member("app", "app", b"binary")],
            limits(),
        );
        let extracted = extract(&plan, dir.path()).unwrap();
        let root = extracted.root().to_path_buf();
        let moved = dir.path().join("moved-root");
        fs::rename(&root, &moved).unwrap();
        fs::write(&root, b"foreign replacement").unwrap();

        let error = extracted.persist().cleanup().unwrap_err();
        assert_eq!(error.kind(), ExtractionErrorKind::CleanupFailed);
        assert_eq!(error.residue_path(), Some(root.as_path()));
        assert_eq!(fs::read(&root).unwrap(), b"foreign replacement");
        fs::remove_file(&root).unwrap();
        fs::remove_dir_all(moved).unwrap();
    }

    #[test]
    fn cleanup_preserves_foreign_non_empty_directory_at_original_path() {
        let dir = TestDir::new();
        let archive = zip_file(dir.path(), "bundle.zip", &[("app", b"binary")]);
        let plan = make_plan(
            archive,
            ArchiveFormat::Zip,
            vec![member("app", "app", b"binary")],
            limits(),
        );
        let extracted = extract(&plan, dir.path()).unwrap();
        let root = extracted.root().to_path_buf();
        let moved = dir.path().join("moved-root");
        fs::rename(&root, &moved).unwrap();
        // Replace the old pathname with a foreign non-empty directory that
        // would be catastrophic if any pathname-only recursive cleanup ran
        // against it.
        let foreign = root.clone();
        fs::create_dir(&foreign).unwrap();
        for i in 0..6 {
            fs::write(foreign.join(format!("foreign-{i}")), b"keep").unwrap();
        }

        let error = extracted.persist().cleanup().unwrap_err();
        assert_eq!(error.kind(), ExtractionErrorKind::CleanupFailed);
        assert_eq!(error.residue_path(), Some(foreign.as_path()));
        assert!(foreign.exists());
        for i in 0..6 {
            assert_eq!(
                fs::read(foreign.join(format!("foreign-{i}"))).unwrap(),
                b"keep"
            );
        }
        assert!(moved.exists());
        assert_eq!(fs::read_dir(&moved).unwrap().count(), 0);
        fs::remove_dir_all(&foreign).unwrap();
        fs::remove_dir_all(moved).unwrap();
    }

    #[test]
    fn cleanup_preserves_foreign_empty_directory_at_original_path() {
        let dir = TestDir::new();
        let archive = zip_file(dir.path(), "bundle.zip", &[("app", b"binary")]);
        let plan = make_plan(
            archive,
            ArchiveFormat::Zip,
            vec![member("app", "app", b"binary")],
            limits(),
        );
        let extracted = extract(&plan, dir.path()).unwrap();
        let root = extracted.root().to_path_buf();
        let moved = dir.path().join("moved-root");
        fs::rename(&root, &moved).unwrap();
        let foreign = root.clone();
        fs::create_dir(&foreign).unwrap();
        // Empty foreign directory: identity still differs, so cleanup must
        // fail closed rather than rmdir the foreign dir.
        let error = extracted.persist().cleanup().unwrap_err();
        assert_eq!(error.kind(), ExtractionErrorKind::CleanupFailed);
        assert_eq!(error.residue_path(), Some(foreign.as_path()));
        assert!(foreign.exists());
        fs::remove_dir(&foreign).unwrap();
        fs::remove_dir_all(moved).unwrap();
    }

    #[test]
    fn drop_cleanup_preserves_foreign_replacement_at_original_path() {
        let dir = TestDir::new();
        let archive = zip_file(dir.path(), "bundle.zip", &[("app", b"binary")]);
        let plan = make_plan(
            archive,
            ArchiveFormat::Zip,
            vec![member("app", "app", b"binary")],
            limits(),
        );
        let root = {
            let extracted = extract(&plan, dir.path()).unwrap();
            let root = extracted.root().to_path_buf();
            let moved = dir.path().join("moved-root");
            fs::rename(&root, &moved).unwrap();
            let foreign = root.clone();
            fs::create_dir(&foreign).unwrap();
            fs::write(foreign.join("foreign"), b"keep").unwrap();
            // Drop the extraction; best-effort drop cleanup empties only the
            // originally opened tree via its handle and never falls back to
            // `fs::remove_dir_all(path)`.
            drop(extracted);
            // The original owned root (now at `moved`) is emptied via its
            // handle; the foreign replacement is untouched.
            assert!(moved.exists());
            assert_eq!(fs::read_dir(&moved).unwrap().count(), 0);
            assert!(foreign.exists());
            assert_eq!(fs::read(foreign.join("foreign")).unwrap(), b"keep");
            moved
        };
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn cleanup_preserves_symlink_substitution_at_original_path() {
        use std::os::unix::fs::symlink;
        let dir = TestDir::new();
        let archive = zip_file(dir.path(), "bundle.zip", &[("app", b"binary")]);
        let plan = make_plan(
            archive,
            ArchiveFormat::Zip,
            vec![member("app", "app", b"binary")],
            limits(),
        );
        let extracted = extract(&plan, dir.path()).unwrap();
        let root = extracted.root().to_path_buf();
        let moved = dir.path().join("moved-root");
        fs::rename(&root, &moved).unwrap();
        // Replace the old pathname with a symlink pointing somewhere else;
        // recursive cleanup must not follow it.
        let target = dir.path().join("link-target");
        fs::create_dir(&target).unwrap();
        fs::write(target.join("keep"), b"keep").unwrap();
        symlink(&target, &root).unwrap();

        let error = extracted.persist().cleanup().unwrap_err();
        assert_eq!(error.kind(), ExtractionErrorKind::CleanupFailed);
        assert!(target.exists());
        assert_eq!(fs::read(target.join("keep")).unwrap(), b"keep");
        fs::remove_dir_all(moved).unwrap();
        fs::remove_dir_all(&target).unwrap();
        fs::remove_file(&root).unwrap();
    }

    #[test]
    fn explicit_cleanup_after_replacement_fails_closed_and_persists_identity() {
        let dir = TestDir::new();
        let archive = zip_file(dir.path(), "bundle.zip", &[("app", b"binary")]);
        let plan = make_plan(
            archive,
            ArchiveFormat::Zip,
            vec![member("app", "app", b"binary")],
            limits(),
        );
        let extracted = extract(&plan, dir.path()).unwrap();
        let persisted = extracted.persist();
        let root = persisted.root().to_path_buf();
        let moved = dir.path().join("moved-root");
        fs::rename(&root, &moved).unwrap();
        let foreign = root.clone();
        fs::create_dir(&foreign).unwrap();
        fs::write(foreign.join("foreign"), b"keep").unwrap();

        let error = persisted.cleanup().unwrap_err();
        assert_eq!(error.kind(), ExtractionErrorKind::CleanupFailed);
        assert_eq!(error.residue_path(), Some(foreign.as_path()));
        assert!(foreign.exists());
        assert!(moved.exists());
        assert_eq!(fs::read_dir(&moved).unwrap().count(), 0);
        fs::remove_dir_all(&foreign).unwrap();
        fs::remove_dir_all(moved).unwrap();
    }

    fn owned_root_with_files(parent: &Path, files: &[&str]) -> (PathBuf, File) {
        let (root, handle) = create_private_root(parent).unwrap();
        for name in files {
            fs::write(root.join(name), b"owned").unwrap();
        }
        (root, handle)
    }

    #[test]
    fn deterministic_race_foreign_non_empty_after_content_deletion_remains_intact() {
        let dir = TestDir::new();
        let (root, handle) = owned_root_with_files(dir.path(), &["owned"]);
        let moved = dir.path().join("moved-owned");
        let error = remove_owned_root_with_hook(
            root.clone(),
            handle,
            Some(&|path: &Path| {
                fs::rename(path, &moved).unwrap();
                fs::create_dir(path).unwrap();
                for i in 0..4 {
                    fs::write(path.join(format!("foreign-{i}")), b"keep").unwrap();
                }
            }),
        )
        .unwrap_err();
        assert_eq!(error.kind(), ExtractionErrorKind::CleanupFailed);
        assert_eq!(error.residue_path(), Some(root.as_path()));
        for i in 0..4 {
            assert_eq!(
                fs::read(root.join(format!("foreign-{i}"))).unwrap(),
                b"keep"
            );
        }
        assert!(moved.exists());
        assert_eq!(fs::read_dir(&moved).unwrap().count(), 0);
        fs::remove_dir_all(&root).unwrap();
        fs::remove_dir(&moved).unwrap();
    }

    #[test]
    fn deterministic_race_foreign_empty_after_content_deletion_remains_intact() {
        let dir = TestDir::new();
        let (root, handle) = owned_root_with_files(dir.path(), &["owned"]);
        let moved = dir.path().join("moved-owned");
        let error = remove_owned_root_with_hook(
            root.clone(),
            handle,
            Some(&|path: &Path| {
                fs::rename(path, &moved).unwrap();
                fs::create_dir(path).unwrap();
            }),
        )
        .unwrap_err();
        assert_eq!(error.kind(), ExtractionErrorKind::CleanupFailed);
        assert_eq!(error.residue_path(), Some(root.as_path()));
        assert!(root.exists());
        assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
        assert!(moved.exists());
        assert_eq!(fs::read_dir(&moved).unwrap().count(), 0);
        fs::remove_dir(&root).unwrap();
        fs::remove_dir(&moved).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn deterministic_race_symlink_after_content_deletion_is_not_followed() {
        use std::os::unix::fs::symlink;
        let dir = TestDir::new();
        let (root, handle) = owned_root_with_files(dir.path(), &["owned"]);
        let moved = dir.path().join("moved-owned");
        let target = dir.path().join("hook-target");
        fs::create_dir(&target).unwrap();
        fs::write(target.join("keep"), b"keep").unwrap();
        let error = remove_owned_root_with_hook(
            root.clone(),
            handle,
            Some(&|path: &Path| {
                fs::rename(path, &moved).unwrap();
                symlink(&target, path).unwrap();
            }),
        )
        .unwrap_err();
        assert_eq!(error.kind(), ExtractionErrorKind::CleanupFailed);
        assert_eq!(error.residue_path(), Some(root.as_path()));
        assert_eq!(fs::read(target.join("keep")).unwrap(), b"keep");
        assert!(moved.exists());
        assert_eq!(fs::read_dir(&moved).unwrap().count(), 0);
        fs::remove_dir_all(&moved).unwrap();
        fs::remove_dir_all(&target).unwrap();
        fs::remove_file(&root).unwrap();
    }

    #[test]
    fn deterministic_race_persisted_cleanup_uses_same_handle_authority() {
        let dir = TestDir::new();
        let (root, handle) = owned_root_with_files(dir.path(), &["owned"]);
        let moved = dir.path().join("moved-owned");
        let error = remove_owned_root_with_hook(
            root.clone(),
            handle,
            Some(&|path: &Path| {
                fs::rename(path, &moved).unwrap();
                fs::create_dir(path).unwrap();
                fs::write(path.join("foreign"), b"keep").unwrap();
            }),
        )
        .unwrap_err();
        assert_eq!(error.kind(), ExtractionErrorKind::CleanupFailed);
        assert_eq!(error.residue_path(), Some(root.as_path()));
        assert_eq!(fs::read(root.join("foreign")).unwrap(), b"keep");
        assert!(moved.exists());
        assert_eq!(fs::read_dir(&moved).unwrap().count(), 0);
        fs::remove_dir_all(&root).unwrap();
        fs::remove_dir(&moved).unwrap();
    }

    #[test]
    fn cleanup_remains_bounded_under_concurrent_file_creation() {
        let dir = TestDir::new();
        let (root, handle) = owned_root_with_files(dir.path(), &["owned"]);
        let root_clone = root.clone();
        let writer = std::thread::spawn(move || {
            for i in 0..200 {
                let _ = fs::write(root_clone.join(format!("race-{i}")), b"x");
            }
        });
        let result = remove_owned_root_with_hook(root.clone(), handle, None);
        assert_eq!(
            result.unwrap_err().kind(),
            ExtractionErrorKind::CleanupFailed
        );
        writer.join().unwrap();
        let _ = fs::remove_dir_all(&root);
    }

    #[cfg(windows)]
    #[test]
    fn deterministic_race_windows_reparse_after_authority_boundary_is_preserved() {
        let dir = TestDir::new();
        let (root, handle) = owned_root_with_files(dir.path(), &["owned"]);
        let moved = dir.path().join("moved-owned");
        let target = dir.path().join("hook-target");
        fs::create_dir(&target).unwrap();
        fs::write(target.join("keep"), b"keep").unwrap();
        let result = remove_owned_root_with_hook(
            root.clone(),
            handle,
            Some(&|path: &Path| {
                fs::rename(path, &moved).unwrap();
                let _ = std::os::windows::fs::symlink_dir(&target, path);
            }),
        );
        let error = result.unwrap_err();
        assert_eq!(error.kind(), ExtractionErrorKind::CleanupFailed);
        assert_eq!(error.residue_path(), Some(root.as_path()));
        assert_eq!(fs::read(target.join("keep")).unwrap(), b"keep");
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_file(&root);
        let _ = fs::remove_dir_all(&moved);
        let _ = fs::remove_dir(&moved);
        let _ = fs::remove_dir_all(&target);
    }

    #[test]
    fn normal_explicit_cleanup_empties_owned_root_and_reports_empty_residue() {
        let dir = TestDir::new();
        let archive = zip_file(dir.path(), "bundle.zip", &[("app", b"binary")]);
        let plan = make_plan(
            archive,
            ArchiveFormat::Zip,
            vec![member("app", "app", b"binary")],
            limits(),
        );
        let persisted = extract(&plan, dir.path()).unwrap().persist();
        let root = persisted.root().to_path_buf();
        let member_path = persisted.members()[0].path().to_path_buf();
        assert!(member_path.exists());
        let error = persisted.cleanup().unwrap_err();
        assert_eq!(error.kind(), ExtractionErrorKind::CleanupFailed);
        assert_eq!(error.residue_path(), Some(root.as_path()));
        assert!(!member_path.exists());
        assert!(root.exists());
        assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
        fs::remove_dir(&root).unwrap();
    }

    #[test]
    fn exclusive_output_creation_preserves_an_existing_file() {
        let dir = TestDir::new();
        let output = dir.path().join("existing");
        fs::write(&output, b"foreign").unwrap();
        assert_eq!(
            create_private_file(&output).unwrap_err().kind(),
            ExtractionErrorKind::Io
        );
        assert_eq!(fs::read(output).unwrap(), b"foreign");
    }

    #[test]
    fn extracted_file_can_be_prepared_as_an_ordinary_core_artifact() {
        let dir = TestDir::new();
        let archive = zip_file(dir.path(), "release.zip", &[("main", b"new binary")]);
        let plan = make_plan(
            archive,
            ArchiveFormat::Zip,
            vec![member("main", "main", b"new binary")],
            limits(),
        );
        let extracted = extract(&plan, dir.path()).unwrap().persist();
        let install_root = dir.path().join("install");
        fs::create_dir(&install_root).unwrap();
        let artifacts = ArtifactSet::new(vec![ArtifactMember::new(
            MemberId::new("main").unwrap(),
            extracted.members()[0].path(),
            "bin/main",
        )
        .unwrap()])
        .unwrap();
        let prepared = InstallPlan::new(
            ProductId::new("example").unwrap(),
            ReleaseId::new("1.0.0").unwrap(),
            &install_root,
            artifacts,
        )
        .unwrap()
        .prepare()
        .unwrap();
        assert!(prepared.stage_root().exists());
        let root = extracted.root().to_path_buf();
        let error = extracted.cleanup().unwrap_err();
        assert_eq!(error.kind(), ExtractionErrorKind::CleanupFailed);
        assert_eq!(error.residue_path(), Some(root.as_path()));
        assert!(root.exists());
        assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
        fs::remove_dir(&root).unwrap();
    }
}
