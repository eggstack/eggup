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
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
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
    ///
    /// This pathname is advisory diagnostics only, never staging authority:
    /// a root rename/replacement after extraction can leave it stale or
    /// foreign while the bytes remain in the handle-owned directory. Stage
    /// archive members through [`BoundMember`] (see
    /// [`PersistedExtraction::into_bound_sources`]) so staged bytes come from
    /// the already-open member object instead of re-resolving this path.
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
    /// Already-open readable member objects, paired by index with `members`.
    /// Retained so the object-bound handoff never re-resolves a pathname.
    handles: Vec<File>,
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
        let handles = std::mem::take(&mut self.handles);
        PersistedExtraction {
            root,
            handle,
            members: self.members,
            handles,
        }
    }

    /// Removes the operation-owned extraction directory and reports residue
    /// evidence if cleanup fails.
    pub fn cleanup(mut self) -> Result<(), ExtractionError> {
        let guard = self.guard.take().expect("active extraction guard");
        // Close member objects before emptying the root: cleanup never races
        // a still-open member, and no pathname fallback is consulted.
        drop(std::mem::take(&mut self.handles));
        guard.cleanup()
    }
}

/// Extracted output whose cleanup responsibility has been transferred.
pub struct PersistedExtraction {
    root: PathBuf,
    handle: File,
    members: Vec<ExtractedMember>,
    /// Already-open readable member objects, paired by index with `members`.
    handles: Vec<File>,
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
        let Self {
            root,
            handle,
            members: _,
            handles,
        } = self;
        // Close member objects before emptying the root (see
        // `ExtractedArchive::cleanup` for the ordering rationale).
        drop(handles);
        remove_owned_root_via_handle(root, handle)
    }

    /// Converts every member into an object-bound staging source.
    ///
    /// This is the handle-backed handoff: each returned [`BoundMember`] owns
    /// the already-open readable member object created by the atomic
    /// handle-relative write, rewound to byte zero, so later staging reads
    /// the verified object instead of re-resolving the recorded pathname or
    /// member name. Call this before any staging; a root rename or member
    /// replacement afterwards cannot redirect the bound objects.
    ///
    /// Member handles are closed when their [`BoundMember`] is consumed or
    /// dropped. Run cleanup only after bound handles are consumed or closed
    /// (see [`BoundExtraction::into_members`]); residue remains acceptable
    /// under the same semantics as [`cleanup`](Self::cleanup).
    pub fn into_bound_sources(self) -> Result<BoundExtraction, ExtractionError> {
        let Self {
            root,
            handle: root_handle,
            members,
            handles,
        } = self;
        assert_eq!(
            members.len(),
            handles.len(),
            "extraction members and retained handles must stay paired"
        );
        let mut bound = Vec::with_capacity(members.len());
        for (evidence, mut file) in members.into_iter().zip(handles.into_iter()) {
            file.flush()
                .map_err(|_| ExtractionError::new(ExtractionErrorKind::Io))?;
            file.seek(SeekFrom::Start(0))
                .map_err(|_| ExtractionError::new(ExtractionErrorKind::Io))?;
            bound.push(BoundMember {
                source_path: evidence.source_path,
                output_name: evidence.output_name,
                advisory_path: evidence.path,
                bytes_written: evidence.bytes_written,
                sha256: evidence.sha256,
                handle: file,
            });
        }
        Ok(BoundExtraction {
            members: bound,
            root,
            root_handle,
        })
    }
}

/// One extracted member proven by its already-open member object.
///
/// A bound member carries the same size/digest evidence as
/// [`ExtractedMember`], but its bytes are proven by object identity: `handle`
/// is the exact object created by the atomic handle-relative write and
/// verified during extraction, rewound to byte zero at handoff. Staging from
/// this object performs no name lookup after the handoff boundary, so a
/// concurrent member-entry replacement cannot redirect staged bytes.
///
/// The handle is single-owner and move-only. [`File`] cursors are shared
/// across clones, so this type is deliberately not [`Clone`]: move the object
/// into staging (see [`into_open_object`](Self::into_open_object)), which
/// rewinds it again before reading. Never [`try_clone`](File::try_clone) a
/// bound handle and assume an independent offset.
///
/// `advisory_path` is diagnostics only and explicitly non-authoritative: it
/// may be stale (root renamed) or foreign (member entry replaced) while the
/// handle still proves the owned bytes. Never stage by opening it.
#[derive(Debug)]
pub struct BoundMember {
    source_path: String,
    output_name: String,
    advisory_path: PathBuf,
    bytes_written: u64,
    sha256: [u8; 32],
    handle: File,
}

impl BoundMember {
    /// Returns the normalized archive path that supplied these bytes.
    pub fn source_path(&self) -> &str {
        &self.source_path
    }

    /// Returns the output filename inside the extraction root.
    pub fn output_name(&self) -> &str {
        &self.output_name
    }

    /// Returns the recorded member pathname for diagnostics only.
    ///
    /// Non-authoritative: may be stale or foreign after a root
    /// rename/replacement. Never open this path to obtain staged bytes; move
    /// [`into_open_object`](Self::into_open_object) into staging instead.
    pub fn advisory_path(&self) -> &Path {
        &self.advisory_path
    }

    /// Returns the exact bytes written and verified.
    pub fn bytes_written(&self) -> u64 {
        self.bytes_written
    }

    /// Returns the SHA-256 digest of the extracted bytes.
    pub fn sha256(&self) -> [u8; 32] {
        self.sha256
    }

    /// Borrows the open member object, for example to prove cursor ownership
    /// in tests. The holder owns the shared cursor: any read/seek moves it
    /// for all borrowers, and staging rewinds to byte zero before reading.
    pub fn handle_mut(&mut self) -> &mut File {
        &mut self.handle
    }

    /// Releases the already-open readable member object for staging.
    ///
    /// The object arrives rewound to byte zero by
    /// [`into_bound_sources`](PersistedExtraction::into_bound_sources);
    /// staging must still rewind before reading (it does) so a cursor moved
    /// after handoff cannot truncate staged bytes.
    pub fn into_open_object(self) -> File {
        self.handle
    }
}

/// Object-bound extraction output awaiting staging and deferred cleanup.
///
/// Created by [`PersistedExtraction::into_bound_sources`]. Members stage from
/// their open objects; the retained root handle stays owned here until
/// cleanup so directory authority is never lost while bound sources are
/// outstanding.
#[derive(Debug)]
pub struct BoundExtraction {
    members: Vec<BoundMember>,
    root: PathBuf,
    root_handle: File,
}

impl BoundExtraction {
    /// Returns the bound members in declaration order.
    pub fn members(&self) -> &[BoundMember] {
        &self.members
    }

    /// Mutably borrows the bound members, for example to prove cursor
    /// ownership before staging. The holder owns every shared cursor.
    pub fn members_mut(&mut self) -> &mut [BoundMember] {
        &mut self.members
    }

    /// Returns the retained private extraction root.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Splits bound staging sources from deferred root cleanup.
    ///
    /// The returned members own their open objects; stage them first, then
    /// let their handles close (by consumption into staging or drop) before
    /// running [`cleanup`](DeferredCleanup::cleanup). Cleanup ordered before
    /// staging completes fails closed on platforms where open handles pin
    /// directory entries, and is never pathname-recursive.
    pub fn into_members(self) -> (Vec<BoundMember>, DeferredCleanup) {
        let Self {
            members,
            root,
            root_handle,
        } = self;
        (members, DeferredCleanup { root, root_handle })
    }

    /// Removes the retained directory without staging, for abort paths.
    ///
    /// Member objects are closed first (see
    /// [`ExtractedArchive::cleanup`]); residue semantics match
    /// [`PersistedExtraction::cleanup`].
    pub fn cleanup(self) -> Result<(), ExtractionError> {
        let Self {
            members,
            root,
            root_handle,
        } = self;
        drop(members);
        remove_owned_root_via_handle(root, root_handle)
    }
}

/// Deferred extraction-root cleanup paired with outstanding bound sources.
///
/// Returned by [`BoundExtraction::into_members`]. Run this only after bound
/// handles are consumed or closed; cleanup empties the owned root solely via
/// the retained handle, never by traversing a pathname.
pub struct DeferredCleanup {
    root: PathBuf,
    root_handle: File,
}

impl std::fmt::Debug for DeferredCleanup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeferredCleanup")
            .field("root", &self.root)
            .finish_non_exhaustive()
    }
}

impl DeferredCleanup {
    /// Returns the retained private extraction root for residue diagnostics.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Empties the owned root via the retained handle and reports residue
    /// evidence if cleanup fails.
    pub fn cleanup(self) -> Result<(), ExtractionError> {
        remove_owned_root_via_handle(self.root, self.root_handle)
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
    match extract_with_hook_inner(plan, &root, &handle, None) {
        Ok((members, handles)) => Ok(ExtractedArchive {
            guard: Some(DirectoryGuard::new(root, handle)),
            members,
            handles,
        }),
        Err(error) => match remove_owned_root_with_hook(root.clone(), handle, None) {
            Ok(()) => Err(error),
            Err(_) => Err(error.with_empty_residue(root)),
        },
    }
}

/// Test seam for deterministic root rename/replacement races.
/// Deterministic pre-write hook for root rename/replacement races.
///
/// Runs synchronously on the extraction thread before each declared-member
/// file creation with the zero-based declared-write sequence number and the
/// recorded root path. Production passes `None`.
type MaterializationHook<'a> = Option<&'a dyn Fn(usize, &Path)>;

/// Retained root authority borrowed by both format handlers.
///
/// Carries the recorded root pathname together with the open root directory
/// object that authorizes every member write, plus the declared-write
/// sequence counter and optional deterministic test hook. This keeps tar and
/// zip on one shared authority path instead of duplicating handle logic.
struct ExtractionScope<'a> {
    root_path: &'a Path,
    root_handle: &'a File,
    write_seq: usize,
    hook: MaterializationHook<'a>,
}

impl<'a> ExtractionScope<'a> {
    /// Runs the deterministic hook (if any) before the next declared write.
    fn run_hook(&self) {
        if let Some(hook) = self.hook {
            hook(self.write_seq, self.root_path);
        }
    }

    /// Creates the next declared member file through the retained handle.
    fn create_member(&mut self, output_name: &str) -> Result<File, ExtractionError> {
        self.run_hook();
        let file = create_private_file_at(self.root_handle, Path::new(output_name))?;
        self.write_seq += 1;
        Ok(file)
    }

    /// Records the handoff pathname for a member written via the handle.
    ///
    /// The returned path is evidence only, never write authority (see the
    /// M001c stop record for why it may be stale after a rename).
    fn member_path(&self, output_name: &str) -> PathBuf {
        self.root_path.join(output_name)
    }
}

///
/// The optional `hook` runs synchronously on the extraction thread before
/// each declared-member file creation, with the zero-based declared-write
/// sequence number and the recorded root path. Production passes `None`.
#[cfg(test)]
fn extract_with_hook(
    plan: &ArchivePlan,
    output_parent: &Path,
    hook: MaterializationHook<'_>,
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
    match extract_with_hook_inner(plan, &root, &handle, hook) {
        Ok((members, handles)) => Ok(ExtractedArchive {
            guard: Some(DirectoryGuard::new(root, handle)),
            members,
            handles,
        }),
        Err(error) => match remove_owned_root_with_hook(root.clone(), handle, None) {
            Ok(()) => Err(error),
            Err(_) => Err(error.with_empty_residue(root)),
        },
    }
}

fn extract_with_hook_inner(
    plan: &ArchivePlan,
    root: &Path,
    root_handle: &File,
    hook: MaterializationHook<'_>,
) -> Result<(Vec<ExtractedMember>, Vec<File>), ExtractionError> {
    let mut total = 0u64;
    let mut seen = HashSet::new();
    let mut extracted: HashMap<String, (ExtractedMember, File)> = HashMap::new();
    let mut scope = ExtractionScope {
        root_path: root,
        root_handle,
        write_seq: 0,
        hook,
    };
    match plan.format {
        ArchiveFormat::TarGz => {
            extract_tar_gz(plan, &mut scope, &mut total, &mut seen, &mut extracted)?
        }
        ArchiveFormat::Zip => extract_zip(plan, &mut scope, &mut total, &mut seen, &mut extracted)?,
    }
    let mut ordered = Vec::with_capacity(plan.members.len());
    let mut handles = Vec::with_capacity(plan.members.len());
    for member in &plan.members {
        let (evidence, handle) = extracted
            .remove(&member.source_path)
            .ok_or_else(|| ExtractionError::new(ExtractionErrorKind::MissingMember))?;
        ordered.push(evidence);
        handles.push(handle);
    }
    Ok((ordered, handles))
}

fn extract_tar_gz(
    plan: &ArchivePlan,
    scope: &mut ExtractionScope<'_>,
    total: &mut u64,
    seen: &mut HashSet<String>,
    extracted: &mut HashMap<String, (ExtractedMember, File)>,
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
            let mut output = scope.create_member(&member.output_name)?;
            // The recorded member path is handoff evidence only; the write
            // itself was authorized by the retained root handle, never by
            // this pathname. If the root namespace entry is renamed/replaced
            // during extraction, this pathname may be stale/foreign (see
            // M001c stop record); the bytes remain in the handle-owned
            // directory.
            let path = scope.member_path(&member.output_name);
            let (bytes_written, digest) =
                copy_bounded(&mut entry, Some(&mut output), plan.limits, total)?;
            output
                .flush()
                .map_err(|_| ExtractionError::new(ExtractionErrorKind::Io))?;
            validate_expected(member, bytes_written, digest)?;
            extracted.insert(
                normalized.clone(),
                (
                    ExtractedMember {
                        source_path: normalized,
                        output_name: member.output_name.clone(),
                        path,
                        bytes_written,
                        sha256: digest,
                    },
                    output,
                ),
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
    scope: &mut ExtractionScope<'_>,
    total: &mut u64,
    seen: &mut HashSet<String>,
    extracted: &mut HashMap<String, (ExtractedMember, File)>,
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
            let mut output = scope.create_member(&member.output_name)?;
            // Recorded path is handoff evidence only; the write was
            // authorized by the retained root handle (see tar path note).
            let path = scope.member_path(&member.output_name);
            let (bytes_written, digest) =
                copy_bounded(&mut entry, Some(&mut output), plan.limits, total)?;
            output
                .flush()
                .map_err(|_| ExtractionError::new(ExtractionErrorKind::Io))?;
            validate_expected(member, bytes_written, digest)?;
            extracted.insert(
                normalized.clone(),
                (
                    ExtractedMember {
                        source_path: normalized,
                        output_name: member.output_name.clone(),
                        path,
                        bytes_written,
                        sha256: digest,
                    },
                    output,
                ),
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

/// Creates one owner-private member file relative to the retained
/// extraction-root handle.
///
/// The write authority is the open root directory object, never the recorded
/// root pathname: `fs_at::OpenOptions::open_at` resolves only relative to
/// `root`, `create_new(true)` rejects any existing file/link/directory
/// atomically where the filesystem supports it, and `follow(false)` refuses
/// to follow a final-component symlink/reparse point. Unix creation mode is
/// `0600`, preserving the previous owner-private intent.
///
/// The returned object carries read authority from the outset on the same
/// file description: no later staging step may regain readability by
/// reopening the member by pathname/name. The object-backed handoff rewinds
/// it to byte zero before staging consumption.
///
/// `name` must be an already-validated single-component `output_name`; no
/// multi-component archive-controlled path is accepted here.
fn create_private_file_at(root: &File, name: &Path) -> Result<File, ExtractionError> {
    let mut options = fs_at::OpenOptions::default();
    options
        .read(true)
        .write(fs_at::OpenOptionsWriteMode::Write)
        .create_new(true)
        .follow(false);
    #[cfg(unix)]
    {
        use fs_at::os::unix::OpenOptionsExt;
        options.mode(0o600);
    }
    options
        .open_at(root, name)
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
    use eggup_core::{
        ArtifactMember, ArtifactSet, BoundSources, InstallPlan, MemberId, ProductId, ReleaseId,
    };
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
        let (root, root_handle) = create_private_root(dir.path()).unwrap();
        fs::write(root.join("existing"), b"foreign").unwrap();
        assert_eq!(
            create_private_file_at(&root_handle, Path::new("existing"))
                .unwrap_err()
                .kind(),
            ExtractionErrorKind::Io
        );
        assert_eq!(fs::read(root.join("existing")).unwrap(), b"foreign");
        drop(root_handle);
        fs::remove_dir_all(&root).unwrap();
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
        // M001d handoff: the member stages from its already-open object, not
        // by re-resolving the recorded pathname.
        let bound = extract(&plan, dir.path())
            .unwrap()
            .persist()
            .into_bound_sources()
            .unwrap();
        assert_eq!(bound.members()[0].bytes_written(), 10);
        let install_root = dir.path().join("install");
        fs::create_dir(&install_root).unwrap();
        let root = bound.root().to_path_buf();
        let core_plan = bound_plan(&bound, &install_root);
        let (prepared, cleanup) = stage_bound_members(bound, core_plan);
        assert_eq!(
            fs::read(
                prepared
                    .staged_path(&MemberId::new("main").unwrap())
                    .unwrap()
            )
            .unwrap(),
            b"new binary"
        );
        assert!(prepared.stage_root().exists());
        let error = cleanup.cleanup().unwrap_err();
        assert_eq!(error.kind(), ExtractionErrorKind::CleanupFailed);
        assert_eq!(error.residue_path(), Some(root.as_path()));
        assert!(root.exists());
        assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
        fs::remove_dir(&root).unwrap();
    }

    // ---- M001c handle-relative materialization races ----
    //
    // Each test renames the operation-owned root synchronously on the
    // extraction thread (deterministic hook, no sleeps) before or between
    // declared-member creations, then installs a foreign replacement at the
    // recorded pathname. Writes must remain in the renamed original through
    // the retained handle; the foreign replacement must stay untouched. The
    // recorded `ExtractedMember::path()` is expected to be stale here — that
    // handoff gap is the documented M001c Section 14 stop (see closure
    // record), so these tests read owned bytes via the known moved location
    // rather than the returned path.

    #[test]
    fn tar_handle_relative_write_survives_root_rename_before_first_member() {
        let dir = TestDir::new();
        let archive = tar_file(
            dir.path(),
            "race.tar.gz",
            &[TarFixture {
                path: "app",
                kind: b'0',
                body: b"owned-bytes",
                link: None,
            }],
        );
        let plan = make_plan(
            archive,
            ArchiveFormat::TarGz,
            vec![member("app", "app", b"owned-bytes")],
            limits(),
        );
        let moved = dir.path().join("moved-owned-tar");
        let extracted = extract_with_hook(
            &plan,
            dir.path(),
            Some(&|seq: usize, root: &Path| {
                assert_eq!(seq, 0);
                fs::rename(root, &moved).unwrap();
                fs::create_dir(root).unwrap();
                for i in 0..3 {
                    fs::write(root.join(format!("foreign-{i}")), b"keep").unwrap();
                }
            }),
        )
        .unwrap();
        let root = extracted.root().to_path_buf();
        // Owned bytes landed in the renamed original via the handle.
        assert_eq!(fs::read(moved.join("app")).unwrap(), b"owned-bytes");
        // Foreign replacement is byte-for-byte untouched: only attacker files.
        for i in 0..3 {
            assert_eq!(
                fs::read(root.join(format!("foreign-{i}"))).unwrap(),
                b"keep"
            );
        }
        assert!(!root.join("app").exists());
        // Recorded path is stale (points into the foreign replacement).
        assert_eq!(extracted.members()[0].path(), root.join("app").as_path());
        drop(extracted);
        fs::remove_dir_all(&moved).unwrap();
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn zip_handle_relative_write_survives_root_rename_before_first_member() {
        let dir = TestDir::new();
        let archive = zip_file(dir.path(), "race.zip", &[("app", b"owned-bytes")]);
        let plan = make_plan(
            archive,
            ArchiveFormat::Zip,
            vec![member("app", "app", b"owned-bytes")],
            limits(),
        );
        let moved = dir.path().join("moved-owned-zip");
        let extracted = extract_with_hook(
            &plan,
            dir.path(),
            Some(&|seq: usize, root: &Path| {
                assert_eq!(seq, 0);
                fs::rename(root, &moved).unwrap();
                fs::create_dir(root).unwrap();
                for i in 0..3 {
                    fs::write(root.join(format!("foreign-{i}")), b"keep").unwrap();
                }
            }),
        )
        .unwrap();
        let root = extracted.root().to_path_buf();
        assert_eq!(fs::read(moved.join("app")).unwrap(), b"owned-bytes");
        for i in 0..3 {
            assert_eq!(
                fs::read(root.join(format!("foreign-{i}"))).unwrap(),
                b"keep"
            );
        }
        assert!(!root.join("app").exists());
        drop(extracted);
        fs::remove_dir_all(&moved).unwrap();
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn two_member_replacement_between_writes_stays_in_owned_root() {
        let dir = TestDir::new();
        let archive = tar_file(
            dir.path(),
            "two.tar.gz",
            &[
                TarFixture {
                    path: "one",
                    kind: b'0',
                    body: b"first",
                    link: None,
                },
                TarFixture {
                    path: "two",
                    kind: b'0',
                    body: b"second",
                    link: None,
                },
            ],
        );
        let plan = make_plan(
            archive,
            ArchiveFormat::TarGz,
            vec![
                member("one", "one", b"first"),
                member("two", "two", b"second"),
            ],
            limits(),
        );
        let moved = dir.path().join("moved-owned-two");
        let extracted = extract_with_hook(
            &plan,
            dir.path(),
            Some(&|seq: usize, root: &Path| {
                if seq == 1 {
                    fs::rename(root, &moved).unwrap();
                    fs::create_dir(root).unwrap();
                    fs::write(root.join("foreign"), b"keep").unwrap();
                }
            }),
        )
        .unwrap();
        let root = extracted.root().to_path_buf();
        // First member was written before the rename; second after. Both
        // must live in the handle-owned (renamed) directory.
        assert_eq!(fs::read(moved.join("one")).unwrap(), b"first");
        assert_eq!(fs::read(moved.join("two")).unwrap(), b"second");
        // Foreign replacement untouched: exactly the one attacker file.
        assert_eq!(fs::read(root.join("foreign")).unwrap(), b"keep");
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        assert_eq!(extracted.members().len(), 2);
        drop(extracted);
        fs::remove_dir_all(&moved).unwrap();
        fs::remove_dir_all(&root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn root_symlink_replacement_is_not_followed_by_materialization() {
        use std::os::unix::fs::symlink;
        let dir = TestDir::new();
        let archive = zip_file(dir.path(), "race.zip", &[("app", b"owned-bytes")]);
        let plan = make_plan(
            archive,
            ArchiveFormat::Zip,
            vec![member("app", "app", b"owned-bytes")],
            limits(),
        );
        let moved = dir.path().join("moved-owned-link");
        let target = dir.path().join("link-target");
        fs::create_dir(&target).unwrap();
        fs::write(target.join("keep"), b"keep").unwrap();
        let extracted = extract_with_hook(
            &plan,
            dir.path(),
            Some(&|seq: usize, root: &Path| {
                assert_eq!(seq, 0);
                fs::rename(root, &moved).unwrap();
                symlink(&target, root).unwrap();
            }),
        )
        .unwrap();
        let root = extracted.root().to_path_buf();
        // Write stayed in the owned directory; the link target is untouched.
        assert_eq!(fs::read(moved.join("app")).unwrap(), b"owned-bytes");
        assert_eq!(fs::read(target.join("keep")).unwrap(), b"keep");
        assert_eq!(fs::read_dir(&target).unwrap().count(), 1);
        drop(extracted);
        fs::remove_dir_all(&moved).unwrap();
        fs::remove_dir_all(&target).unwrap();
        fs::remove_file(&root).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn windows_reparse_replacement_is_not_followed_by_materialization() {
        let dir = TestDir::new();
        let archive = zip_file(dir.path(), "race.zip", &[("app", b"owned-bytes")]);
        let plan = make_plan(
            archive,
            ArchiveFormat::Zip,
            vec![member("app", "app", b"owned-bytes")],
            limits(),
        );
        let moved = dir.path().join("moved-owned-reparse");
        let target = dir.path().join("reparse-target");
        fs::create_dir(&target).unwrap();
        fs::write(target.join("keep"), b"keep").unwrap();
        let extracted = extract_with_hook(
            &plan,
            dir.path(),
            Some(&|seq: usize, root: &Path| {
                assert_eq!(seq, 0);
                fs::rename(root, &moved).unwrap();
                // Best-effort reparse/symlink: may require privileges. If
                // creation is denied, the old pathname simply stays absent;
                // either way materialization must not enter the target.
                let _ = std::os::windows::fs::symlink_dir(&target, root);
                if !root.exists() {
                    fs::create_dir(root).unwrap();
                    fs::write(root.join("foreign"), b"keep").unwrap();
                }
            }),
        )
        .unwrap();
        let root = extracted.root().to_path_buf();
        assert_eq!(fs::read(moved.join("app")).unwrap(), b"owned-bytes");
        assert_eq!(fs::read(target.join("keep")).unwrap(), b"keep");
        assert_eq!(fs::read_dir(&target).unwrap().count(), 1);
        drop(extracted);
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_file(&root);
        let _ = fs::remove_dir_all(&moved);
        let _ = fs::remove_dir(&moved);
        let _ = fs::remove_dir_all(&target);
    }

    #[test]
    fn raced_existing_output_file_in_owned_root_fails_without_clobber() {
        let dir = TestDir::new();
        let archive = zip_file(dir.path(), "race.zip", &[("app", b"new-bytes")]);
        let plan = make_plan(
            archive,
            ArchiveFormat::Zip,
            vec![member("app", "app", b"new-bytes")],
            limits(),
        );
        let error = extract_with_hook(
            &plan,
            dir.path(),
            Some(&|seq: usize, root: &Path| {
                assert_eq!(seq, 0);
                // Concurrent creation inside the owned root (no rename here,
                // so pathname and handle agree on the directory).
                fs::write(root.join("app"), b"existing").unwrap();
            }),
        )
        .unwrap_err();
        assert_eq!(error.kind(), ExtractionErrorKind::Io);
        // No clobber: residue (if any) never contains the new bytes at the
        // raced name; the failure cleans only the owned partial root.
        if let Some(residue) = error.residue_path() {
            if residue.join("app").exists() {
                assert_eq!(fs::read(residue.join("app")).unwrap(), b"existing");
            }
            let _ = fs::remove_dir_all(residue);
        }
    }

    #[cfg(unix)]
    #[test]
    fn raced_output_symlink_in_owned_root_fails_and_target_untouched() {
        use std::os::unix::fs::symlink;
        let dir = TestDir::new();
        let archive = zip_file(dir.path(), "race.zip", &[("app", b"new-bytes")]);
        let plan = make_plan(
            archive,
            ArchiveFormat::Zip,
            vec![member("app", "app", b"new-bytes")],
            limits(),
        );
        let target = dir.path().join("race-target");
        fs::write(&target, b"keep").unwrap();
        let error = extract_with_hook(
            &plan,
            dir.path(),
            Some(&|seq: usize, root: &Path| {
                assert_eq!(seq, 0);
                symlink(&target, root.join("app")).unwrap();
            }),
        )
        .unwrap_err();
        // create_new + no-follow must refuse the symlink; the target is
        // never opened for write.
        assert_eq!(error.kind(), ExtractionErrorKind::Io);
        assert_eq!(fs::read(&target).unwrap(), b"keep");
        if let Some(residue) = error.residue_path() {
            let _ = fs::remove_dir_all(residue);
        }
        fs::remove_file(&target).unwrap();
    }

    // ---- M001d handle-backed source handoff ----
    //
    // These tests prove staged bytes come from the already-open member
    // object, never from re-resolving a recorded pathname or member name.
    // All races are deterministic (synchronous hooks or explicit post-handoff
    // filesystem operations); no test sleeps. The plan is always built from
    // advisory paths while they are still valid — the attacks below happen
    // after the handoff boundary, before staging — except where the attack
    // itself must precede the first write (hook races), in which case bound
    // reads are proven at the archive level.

    /// Replaces a member directory entry while retaining the original object
    /// via its open handle. Portable: member handles carry full share access.
    fn replace_member_entry(path: &Path, foreign: &[u8]) {
        fs::remove_file(path).unwrap();
        fs::write(path, foreign).unwrap();
    }

    /// Builds a core plan from bound advisory paths. Call while advisory
    /// paths are still valid; attacks happen after this returns.
    fn bound_plan(extraction: &BoundExtraction, install_root: &Path) -> InstallPlan {
        let artifacts = ArtifactSet::new(
            extraction
                .members()
                .iter()
                .map(|member| {
                    ArtifactMember::new(
                        MemberId::new(member.output_name()).unwrap(),
                        member.advisory_path(),
                        member.output_name(),
                    )
                    .unwrap()
                })
                .collect(),
        )
        .unwrap();
        InstallPlan::new(
            ProductId::new("example").unwrap(),
            ReleaseId::new("1.0.0").unwrap(),
            install_root,
            artifacts,
        )
        .unwrap()
    }

    /// Stages bound members from their open objects and returns the prepared
    /// transaction with deferred root cleanup.
    fn stage_bound_members(
        extraction: BoundExtraction,
        plan: InstallPlan,
    ) -> (eggup_core::PreparedTransaction, DeferredCleanup) {
        let (members, cleanup) = extraction.into_members();
        let mut sources = BoundSources::new();
        for member in members {
            let id = MemberId::new(member.output_name()).unwrap();
            sources.insert(id, member.into_open_object());
        }
        (plan.prepare_with_bound_sources(sources).unwrap(), cleanup)
    }

    /// Asserts handle-authorized cleanup semantics: the owned root is emptied
    /// via the retained handle and residue evidence names the recorded root.
    fn assert_cleanup_reports_residue(cleanup: DeferredCleanup, residue: &Path, emptied: &Path) {
        let error = cleanup.cleanup().unwrap_err();
        assert_eq!(error.kind(), ExtractionErrorKind::CleanupFailed);
        assert_eq!(error.residue_path(), Some(residue));
        assert_eq!(fs::read_dir(emptied).unwrap().count(), 0);
        fs::remove_dir(emptied).unwrap();
    }

    #[test]
    fn bound_tar_member_entry_replacement_after_handoff_stages_owned_bytes() {
        let dir = TestDir::new();
        let archive = tar_file(
            dir.path(),
            "race.tar.gz",
            &[TarFixture {
                path: "app",
                kind: b'0',
                body: b"owned-bytes",
                link: None,
            }],
        );
        let plan = make_plan(
            archive,
            ArchiveFormat::TarGz,
            vec![member("app", "app", b"owned-bytes")],
            limits(),
        );
        let bound = extract(&plan, dir.path())
            .unwrap()
            .persist()
            .into_bound_sources()
            .unwrap();
        let expected: [u8; 32] = Sha256::digest(b"owned-bytes").into();
        assert_eq!(bound.members()[0].bytes_written(), 11);
        assert_eq!(bound.members()[0].sha256(), expected);
        let install_root = dir.path().join("install");
        fs::create_dir(&install_root).unwrap();
        let root = bound.root().to_path_buf();
        let core_plan = bound_plan(&bound, &install_root);
        // Post-handoff replacement inside the still-owned root: the advisory
        // path now resolves to foreign bytes while the open object still
        // proves the owned bytes.
        let advisory = bound.members()[0].advisory_path().to_path_buf();
        replace_member_entry(&advisory, b"foreign-bytes");
        let (prepared, cleanup) = stage_bound_members(bound, core_plan);
        assert_eq!(
            fs::read(
                prepared
                    .staged_path(&MemberId::new("app").unwrap())
                    .unwrap()
            )
            .unwrap(),
            b"owned-bytes"
        );
        assert_eq!(fs::read(&advisory).unwrap(), b"foreign-bytes");
        assert_cleanup_reports_residue(cleanup, &root, &root);
    }

    #[test]
    fn bound_zip_member_entry_replacement_after_handoff_stages_owned_bytes() {
        let dir = TestDir::new();
        let archive = zip_file(dir.path(), "race.zip", &[("app", b"owned-bytes")]);
        let plan = make_plan(
            archive,
            ArchiveFormat::Zip,
            vec![member("app", "app", b"owned-bytes")],
            limits(),
        );
        let bound = extract(&plan, dir.path())
            .unwrap()
            .persist()
            .into_bound_sources()
            .unwrap();
        let expected: [u8; 32] = Sha256::digest(b"owned-bytes").into();
        assert_eq!(bound.members()[0].bytes_written(), 11);
        assert_eq!(bound.members()[0].sha256(), expected);
        let install_root = dir.path().join("install");
        fs::create_dir(&install_root).unwrap();
        let root = bound.root().to_path_buf();
        let core_plan = bound_plan(&bound, &install_root);
        let advisory = bound.members()[0].advisory_path().to_path_buf();
        replace_member_entry(&advisory, b"foreign-bytes");
        let (prepared, cleanup) = stage_bound_members(bound, core_plan);
        assert_eq!(
            fs::read(
                prepared
                    .staged_path(&MemberId::new("app").unwrap())
                    .unwrap()
            )
            .unwrap(),
            b"owned-bytes"
        );
        assert_eq!(fs::read(&advisory).unwrap(), b"foreign-bytes");
        assert_cleanup_reports_residue(cleanup, &root, &root);
    }

    #[test]
    fn bound_two_member_replacement_after_handoff_stages_owned_bytes() {
        let dir = TestDir::new();
        let archive = tar_file(
            dir.path(),
            "two.tar.gz",
            &[
                TarFixture {
                    path: "one",
                    kind: b'0',
                    body: b"first",
                    link: None,
                },
                TarFixture {
                    path: "two",
                    kind: b'0',
                    body: b"second",
                    link: None,
                },
            ],
        );
        let plan = make_plan(
            archive,
            ArchiveFormat::TarGz,
            vec![
                member("one", "one", b"first"),
                member("two", "two", b"second"),
            ],
            limits(),
        );
        let bound = extract(&plan, dir.path())
            .unwrap()
            .persist()
            .into_bound_sources()
            .unwrap();
        let install_root = dir.path().join("install");
        fs::create_dir(&install_root).unwrap();
        let root = bound.root().to_path_buf();
        let core_plan = bound_plan(&bound, &install_root);
        // Replace both member entries after the handoff; neither staged byte
        // may come from the replacements.
        let first_advisory = bound.members()[0].advisory_path().to_path_buf();
        let second_advisory = bound.members()[1].advisory_path().to_path_buf();
        replace_member_entry(&first_advisory, b"foreign-one");
        replace_member_entry(&second_advisory, b"foreign-two");
        let (prepared, cleanup) = stage_bound_members(bound, core_plan);
        assert_eq!(
            fs::read(
                prepared
                    .staged_path(&MemberId::new("one").unwrap())
                    .unwrap()
            )
            .unwrap(),
            b"first"
        );
        assert_eq!(
            fs::read(
                prepared
                    .staged_path(&MemberId::new("two").unwrap())
                    .unwrap()
            )
            .unwrap(),
            b"second"
        );
        assert_eq!(fs::read(&first_advisory).unwrap(), b"foreign-one");
        assert_eq!(fs::read(&second_advisory).unwrap(), b"foreign-two");
        assert_cleanup_reports_residue(cleanup, &root, &root);
    }

    #[test]
    fn bound_post_handoff_root_replacement_stages_owned_bytes() {
        let dir = TestDir::new();
        let archive = zip_file(dir.path(), "race.zip", &[("app", b"owned-bytes")]);
        let plan = make_plan(
            archive,
            ArchiveFormat::Zip,
            vec![member("app", "app", b"owned-bytes")],
            limits(),
        );
        let bound = extract(&plan, dir.path())
            .unwrap()
            .persist()
            .into_bound_sources()
            .unwrap();
        let install_root = dir.path().join("install");
        fs::create_dir(&install_root).unwrap();
        let root = bound.root().to_path_buf();
        let core_plan = bound_plan(&bound, &install_root);
        // Post-handoff root replacement: the recorded root name now belongs
        // to foreign state while the retained handle still owns the original.
        let moved = dir.path().join("moved-owned-post-handoff");
        fs::rename(&root, &moved).unwrap();
        fs::create_dir(&root).unwrap();
        fs::write(root.join("foreign"), b"keep").unwrap();
        let (prepared, cleanup) = stage_bound_members(bound, core_plan);
        assert_eq!(
            fs::read(
                prepared
                    .staged_path(&MemberId::new("app").unwrap())
                    .unwrap()
            )
            .unwrap(),
            b"owned-bytes"
        );
        assert_eq!(fs::read(root.join("foreign")).unwrap(), b"keep");
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        assert_cleanup_reports_residue(cleanup, &root, &moved);
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn bound_tar_root_rename_before_first_write_reads_owned_bytes() {
        let dir = TestDir::new();
        let archive = tar_file(
            dir.path(),
            "race.tar.gz",
            &[TarFixture {
                path: "app",
                kind: b'0',
                body: b"owned-bytes",
                link: None,
            }],
        );
        let plan = make_plan(
            archive,
            ArchiveFormat::TarGz,
            vec![member("app", "app", b"owned-bytes")],
            limits(),
        );
        let moved = dir.path().join("moved-owned-bound-tar");
        let extracted = extract_with_hook(
            &plan,
            dir.path(),
            Some(&|seq: usize, root: &Path| {
                assert_eq!(seq, 0);
                fs::rename(root, &moved).unwrap();
                fs::create_dir(root).unwrap();
                fs::write(root.join("foreign"), b"keep").unwrap();
            }),
        )
        .unwrap();
        let root = extracted.root().to_path_buf();
        // Bound reads resolve the retained object despite the stale pathname.
        let (members, cleanup) = extracted
            .persist()
            .into_bound_sources()
            .unwrap()
            .into_members();
        assert_eq!(members.len(), 1);
        let mut owned = Vec::new();
        let mut object = members.into_iter().next().unwrap().into_open_object();
        use std::io::Read as _;
        object.read_to_end(&mut owned).unwrap();
        assert_eq!(owned, b"owned-bytes");
        assert_eq!(fs::read(root.join("foreign")).unwrap(), b"keep");
        assert!(!root.join("app").exists());
        let error = cleanup.cleanup().unwrap_err();
        assert_eq!(error.kind(), ExtractionErrorKind::CleanupFailed);
        assert_eq!(fs::read_dir(&moved).unwrap().count(), 0);
        fs::remove_dir(&moved).unwrap();
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn bound_zip_root_rename_before_first_write_reads_owned_bytes() {
        let dir = TestDir::new();
        let archive = zip_file(dir.path(), "race.zip", &[("app", b"owned-bytes")]);
        let plan = make_plan(
            archive,
            ArchiveFormat::Zip,
            vec![member("app", "app", b"owned-bytes")],
            limits(),
        );
        let moved = dir.path().join("moved-owned-bound-zip");
        let extracted = extract_with_hook(
            &plan,
            dir.path(),
            Some(&|seq: usize, root: &Path| {
                assert_eq!(seq, 0);
                fs::rename(root, &moved).unwrap();
                fs::create_dir(root).unwrap();
                fs::write(root.join("foreign"), b"keep").unwrap();
            }),
        )
        .unwrap();
        let root = extracted.root().to_path_buf();
        let (members, cleanup) = extracted
            .persist()
            .into_bound_sources()
            .unwrap()
            .into_members();
        assert_eq!(members.len(), 1);
        let mut owned = Vec::new();
        let mut object = members.into_iter().next().unwrap().into_open_object();
        use std::io::Read as _;
        object.read_to_end(&mut owned).unwrap();
        assert_eq!(owned, b"owned-bytes");
        assert_eq!(fs::read(root.join("foreign")).unwrap(), b"keep");
        assert!(!root.join("app").exists());
        let error = cleanup.cleanup().unwrap_err();
        assert_eq!(error.kind(), ExtractionErrorKind::CleanupFailed);
        assert_eq!(fs::read_dir(&moved).unwrap().count(), 0);
        fs::remove_dir(&moved).unwrap();
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn bound_two_member_replacement_between_writes_reads_owned_bytes() {
        let dir = TestDir::new();
        let archive = tar_file(
            dir.path(),
            "two.tar.gz",
            &[
                TarFixture {
                    path: "one",
                    kind: b'0',
                    body: b"first",
                    link: None,
                },
                TarFixture {
                    path: "two",
                    kind: b'0',
                    body: b"second",
                    link: None,
                },
            ],
        );
        let plan = make_plan(
            archive,
            ArchiveFormat::TarGz,
            vec![
                member("one", "one", b"first"),
                member("two", "two", b"second"),
            ],
            limits(),
        );
        let moved = dir.path().join("moved-owned-bound-two");
        let extracted = extract_with_hook(
            &plan,
            dir.path(),
            Some(&|seq: usize, root: &Path| {
                if seq == 1 {
                    fs::rename(root, &moved).unwrap();
                    fs::create_dir(root).unwrap();
                    fs::write(root.join("foreign"), b"keep").unwrap();
                }
            }),
        )
        .unwrap();
        let root = extracted.root().to_path_buf();
        let (members, cleanup) = extracted
            .persist()
            .into_bound_sources()
            .unwrap()
            .into_members();
        assert_eq!(members.len(), 2);
        use std::io::Read as _;
        let mut bodies = Vec::new();
        for member in members {
            let mut object = member.into_open_object();
            let mut body = Vec::new();
            object.read_to_end(&mut body).unwrap();
            bodies.push(body);
        }
        assert_eq!(bodies, vec![b"first".to_vec(), b"second".to_vec()]);
        assert_eq!(fs::read(root.join("foreign")).unwrap(), b"keep");
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        let error = cleanup.cleanup().unwrap_err();
        assert_eq!(error.kind(), ExtractionErrorKind::CleanupFailed);
        assert_eq!(fs::read_dir(&moved).unwrap().count(), 0);
        fs::remove_dir(&moved).unwrap();
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn bound_nonzero_cursor_stages_full_file_from_byte_zero() {
        let dir = TestDir::new();
        let archive = zip_file(dir.path(), "race.zip", &[("app", b"owned-bytes")]);
        let plan = make_plan(
            archive,
            ArchiveFormat::Zip,
            vec![member("app", "app", b"owned-bytes")],
            limits(),
        );
        let mut bound = extract(&plan, dir.path())
            .unwrap()
            .persist()
            .into_bound_sources()
            .unwrap();
        // Move the shared cursor after handoff: staging must still consume
        // the full file from byte zero, never from the moved cursor.
        let mut prefix = [0u8; 4];
        use std::io::Read as _;
        bound.members_mut()[0]
            .handle_mut()
            .read_exact(&mut prefix)
            .unwrap();
        assert_eq!(&prefix, b"owne");
        let install_root = dir.path().join("install");
        fs::create_dir(&install_root).unwrap();
        let root = bound.root().to_path_buf();
        let core_plan = bound_plan(&bound, &install_root);
        let (prepared, cleanup) = stage_bound_members(bound, core_plan);
        assert_eq!(
            fs::read(
                prepared
                    .staged_path(&MemberId::new("app").unwrap())
                    .unwrap()
            )
            .unwrap(),
            b"owned-bytes"
        );
        assert_cleanup_reports_residue(cleanup, &root, &root);
    }

    #[cfg(unix)]
    #[test]
    fn bound_sources_preserve_owner_private_mode_and_exact_evidence() {
        use std::os::unix::fs::PermissionsExt;
        let dir = TestDir::new();
        let archive = tar_file(
            dir.path(),
            "race.tar.gz",
            &[TarFixture {
                path: "app",
                kind: b'0',
                body: b"owned-bytes",
                link: None,
            }],
        );
        let plan = make_plan(
            archive,
            ArchiveFormat::TarGz,
            vec![member("app", "app", b"owned-bytes")],
            limits(),
        );
        let bound = extract(&plan, dir.path())
            .unwrap()
            .persist()
            .into_bound_sources()
            .unwrap();
        let expected: [u8; 32] = Sha256::digest(b"owned-bytes").into();
        assert_eq!(bound.members()[0].bytes_written(), 11);
        assert_eq!(bound.members()[0].sha256(), expected);
        assert_eq!(bound.members()[0].source_path(), "app");
        assert_eq!(bound.members()[0].output_name(), "app");
        // Handle-relative creation stays owner-private with read authority.
        let mode = fs::metadata(bound.members()[0].advisory_path())
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
        let install_root = dir.path().join("install");
        fs::create_dir(&install_root).unwrap();
        let root = bound.root().to_path_buf();
        let core_plan = bound_plan(&bound, &install_root);
        let (prepared, cleanup) = stage_bound_members(bound, core_plan);
        let staged_mode = fs::metadata(
            prepared
                .staged_path(&MemberId::new("app").unwrap())
                .unwrap(),
        )
        .unwrap()
        .permissions()
        .mode();
        assert_eq!(staged_mode & 0o777, 0o600);
        assert_cleanup_reports_residue(cleanup, &root, &root);
    }
}
