use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs::{self, File};
use std::path::{Component, Path, PathBuf};

use crate::current_exe::CurrentExecutable;
use crate::error::{Error, Result};
use crate::stage::{PreparedTransaction, Stage};

/// A validated opaque product identifier owned by the consumer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProductId(String);

impl ProductId {
    /// Creates a product identifier after rejecting empty or unsafe control-character input.
    pub fn new(value: impl Into<String>) -> Result<Self> {
        Ok(Self(validate_identifier("product", value.into())?))
    }

    /// Returns the identifier's stable string representation.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProductId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// A validated opaque release identifier owned by the consumer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReleaseId(String);

impl ReleaseId {
    /// Creates a release identifier after rejecting empty or unsafe control-character input.
    pub fn new(value: impl Into<String>) -> Result<Self> {
        Ok(Self(validate_identifier("release", value.into())?))
    }

    /// Returns the identifier's stable string representation.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ReleaseId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// A stable identity for one member of a deployment bundle.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemberId(String);

impl MemberId {
    /// Creates a member identifier after rejecting empty or unsafe control-character input.
    pub fn new(value: impl Into<String>) -> Result<Self> {
        Ok(Self(validate_identifier("member", value.into())?))
    }

    /// Returns the identifier's stable string representation.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MemberId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

fn validate_identifier(kind: &str, value: String) -> Result<String> {
    if value.is_empty() || value.chars().any(char::is_control) {
        return Err(Error::invalid(format!(
            "{kind} identifier is empty or contains control characters"
        )));
    }
    Ok(value)
}

/// The stable member identity used by [`InstallPlan::for_current_executable`].
const CURRENT_EXECUTABLE_MEMBER: &str = "executable";

/// The file kind expected for an artifact member.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FileKind {
    /// A regular file; directories, links, devices, sockets, and FIFOs are rejected.
    Regular,
}

/// The permission intent to apply to a staged member.
///
/// Staged output is always owner-private (`0600`, or `0700` when executable).
/// `Preserve` maps an executable source to `0700` and anything else to
/// `0600`; broad source modes are never inherited.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PermissionsIntent {
    /// Retain executable intent privately where the platform exposes it.
    Preserve,
    /// Mark the staged member executable where the platform exposes executable bits.
    Executable,
}

/// The canonical ownership classification for a live destination.
///
/// `Owned` is never inferred from an [`InstallPlan`]. It is returned only by a
/// consumer-supplied [`OwnershipVerifier`] that proves the destination belongs
/// to the intended deployment. `Foreign` and `Unknown` always fail closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ownership {
    /// No filesystem object exists at the destination.
    Absent,
    /// The destination provably belongs to this deployment and may be replaced.
    Owned,
    /// The destination belongs to another deployment and must not be mutated.
    Foreign,
    /// Ownership cannot be proven; mutation is denied.
    Unknown,
}

/// A declared integrity requirement. Computation is provided by the verification layer.
///
/// Only `Sha256` members are commit-capable. `None` is visible during
/// preparation but can never reach candidate execution or commit. Checksum
/// evidence only; not an authenticity or signature claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum IntegrityRequirement {
    /// No integrity evidence is attached at this stage.
    None,
    /// The staged bytes must later match this SHA-256 digest.
    Sha256([u8; 32]),
}

/// Consumer-supplied proof that a live destination belongs to this deployment.
///
/// Implementations MUST be synchronous, deterministic for a fixed filesystem
/// state, free of application-specific version types, and treat verifier
/// errors or ambiguity as [`Ownership::Unknown`] or a hard failure — never
/// [`Ownership::Owned`].
///
/// The verifier runs again immediately before destructive mutation while the
/// [`crate::MutationLock`] is held. A change between preflight and locked
/// classification fails the transaction.
pub trait OwnershipVerifier {
    /// Classifies one live destination for one artifact member.
    fn verify(&self, member: &MemberId, destination: &Path) -> Ownership;
}

/// Authorization to create a destination that is currently [`Ownership::Absent`].
///
/// This is distinct from authorization to replace an [`Ownership::Owned`]
/// destination. Replacement of `Owned` never requires this flag; creation of
/// `Absent` always does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbsentPolicy {
    /// Installing into an absent destination is permitted.
    AllowCreate,
    /// Only replacement of an already-`Owned` destination is permitted.
    DenyCreate,
}

/// Commit-time ownership contract: who proves ownership and whether absent
/// destinations may be created.
#[derive(Clone, Copy)]
pub struct CommitOwnership<'a> {
    /// Consumer verifier consulted under lock before any live mutation.
    pub verifier: &'a dyn OwnershipVerifier,
    /// Whether `Absent` destinations may be created.
    pub absent: AbsentPolicy,
}

impl<'a> CommitOwnership<'a> {
    /// Creates an ownership contract from a verifier and an absent policy.
    pub fn new(verifier: &'a dyn OwnershipVerifier, absent: AbsentPolicy) -> Self {
        Self { verifier, absent }
    }
}

impl std::fmt::Debug for CommitOwnership<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommitOwnership")
            .field("absent", &self.absent)
            .finish_non_exhaustive()
    }
}

/// A verifier that permits creation only: `Absent` when missing, `Foreign`
/// when anything exists. Useful for first-install flows that must never
/// replace an existing file.
#[derive(Debug, Clone, Copy, Default)]
pub struct AbsentOnlyVerifier;

impl OwnershipVerifier for AbsentOnlyVerifier {
    fn verify(&self, _member: &MemberId, destination: &Path) -> Ownership {
        match std::fs::symlink_metadata(destination) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ownership::Absent,
            Ok(_) => Ownership::Foreign,
            Err(_) => Ownership::Unknown,
        }
    }
}

/// A verifier that treats an existing regular file as owned.
///
/// This helper exists only for deterministic tests and examples that do not
/// model real deployment identity. Production consumers SHOULD use
/// [`ExactDigestVerifier`] or a deployment-specific identity check instead of
/// this permissive mapping.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExistingAsOwnedVerifier;

impl OwnershipVerifier for ExistingAsOwnedVerifier {
    fn verify(&self, _member: &MemberId, destination: &Path) -> Ownership {
        match std::fs::symlink_metadata(destination) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ownership::Absent,
            Ok(m) if m.is_file() => Ownership::Owned,
            Ok(_) => Ownership::Foreign,
            Err(_) => Ownership::Unknown,
        }
    }
}

/// A verifier that proves ownership by exact prior SHA-256 content.
///
/// Returns `Absent` when nothing exists, `Owned` when the live file hashes to
/// the expected digest for that member, `Foreign` when content differs, and
/// `Unknown` when the destination cannot be read unambiguously.
#[derive(Debug, Clone)]
pub struct ExactDigestVerifier {
    expected: std::collections::HashMap<MemberId, [u8; 32]>,
}

impl ExactDigestVerifier {
    /// Creates a verifier from `(member, expected_live_digest)` pairs.
    pub fn new(expected: Vec<(MemberId, [u8; 32])>) -> Self {
        Self {
            expected: expected.into_iter().collect(),
        }
    }
}

impl OwnershipVerifier for ExactDigestVerifier {
    fn verify(&self, member: &MemberId, destination: &Path) -> Ownership {
        let expected = match self.expected.get(member) {
            Some(d) => d,
            None => return Ownership::Unknown,
        };
        match std::fs::symlink_metadata(destination) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ownership::Absent,
            Err(_) => Ownership::Unknown,
            Ok(m) => {
                if !m.is_file() {
                    return Ownership::Foreign;
                }
                #[cfg(unix)]
                {
                    use std::os::unix::fs::MetadataExt;
                    if m.nlink() != 1 {
                        return Ownership::Foreign;
                    }
                }
                match crate::integrity::hash_file(destination) {
                    Ok(d) if &d == expected => Ownership::Owned,
                    Ok(_) => Ownership::Foreign,
                    Err(_) => Ownership::Unknown,
                }
            }
        }
    }
}

/// One explicitly acquired local artifact and its intended destination.
///
/// The member declares where bytes come from and where they should be
/// installed. It never authorizes replacement: destination ownership is proven
/// separately at commit time by an [`OwnershipVerifier`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactMember {
    id: MemberId,
    source: PathBuf,
    destination: PathBuf,
    file_kind: FileKind,
    permissions: PermissionsIntent,
    integrity: IntegrityRequirement,
}

impl ArtifactMember {
    /// Creates a regular-file member from an acquired source and relative destination.
    pub fn new(
        id: MemberId,
        source: impl Into<PathBuf>,
        destination: impl Into<PathBuf>,
    ) -> Result<Self> {
        let destination = normalize_relative_path(&destination.into())?;
        Ok(Self {
            id,
            source: source.into(),
            destination,
            file_kind: FileKind::Regular,
            permissions: PermissionsIntent::Preserve,
            integrity: IntegrityRequirement::None,
        })
    }

    /// Sets the intended staged-file permissions.
    pub fn with_permissions(mut self, permissions: PermissionsIntent) -> Self {
        self.permissions = permissions;
        self
    }

    /// Attaches a declared integrity requirement without computing it.
    ///
    /// Only `Sha256` members can be committed. `None` is permitted during
    /// preparation so callers can observe the missing-evidence failure mode,
    /// but verification and commit reject it.
    pub fn with_integrity(mut self, integrity: IntegrityRequirement) -> Self {
        self.integrity = integrity;
        self
    }

    /// Returns the stable member identity.
    pub fn id(&self) -> &MemberId {
        &self.id
    }

    /// Returns the acquired local source path.
    pub fn source(&self) -> &Path {
        &self.source
    }

    /// Returns the normalized destination path relative to the installation root.
    pub fn destination(&self) -> &Path {
        &self.destination
    }

    /// Returns the expected file kind.
    pub fn file_kind(&self) -> FileKind {
        self.file_kind
    }

    /// Returns the permission intent.
    pub fn permissions(&self) -> PermissionsIntent {
        self.permissions
    }

    /// Returns the declared integrity requirement.
    pub fn integrity(&self) -> IntegrityRequirement {
        self.integrity
    }
}

/// A non-empty set of artifact members with unique member identities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactSet(Vec<ArtifactMember>);

impl ArtifactSet {
    /// Creates a set and rejects an empty input or duplicate member identity.
    pub fn new(members: Vec<ArtifactMember>) -> Result<Self> {
        if members.is_empty() {
            return Err(Error::invalid(
                "artifact set must contain at least one member",
            ));
        }
        let mut ids = HashSet::new();
        for member in &members {
            if !ids.insert(member.id.clone()) {
                return Err(Error::invalid(format!(
                    "duplicate artifact member: {}",
                    member.id
                )));
            }
        }
        Ok(Self(members))
    }

    /// Creates a one-member set.
    pub fn single(member: ArtifactMember) -> Result<Self> {
        Self::new(vec![member])
    }

    /// Iterates over members in caller-provided order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &ArtifactMember> {
        self.0.iter()
    }

    /// Returns the number of members in the set.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns whether the set contains no members. Always false for a valid set.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(crate) fn as_slice(&self) -> &[ArtifactMember] {
        &self.0
    }
}

/// Already-open member objects staged without pathname re-resolution.
///
/// A bound source proves which bytes to stage by object identity: the moved
/// open [`File`] is the same object whose bytes were acquired and verified,
/// so no directory-entry replacement after the handoff boundary can redirect
/// staged bytes. Handles are single-owner and move-only. [`File`] cursors are
/// shared across clones, so this type is deliberately not [`Clone`]: move the
/// handle into [`InstallPlan::prepare_with_bound_sources`], which rewinds it
/// to byte zero before staging. Do not [`try_clone`](File::try_clone) a bound
/// handle and assume an independent offset.
///
/// The recorded [`ArtifactMember::source`] path of a bound member is advisory
/// diagnostics only once a handle is supplied for its identity.
#[derive(Debug, Default)]
pub struct BoundSources {
    handles: HashMap<MemberId, File>,
}

impl BoundSources {
    /// Creates an empty bound-source map, equivalent to no bound members.
    pub fn new() -> Self {
        Self {
            handles: HashMap::new(),
        }
    }

    /// Registers the already-open object for `member`, returning any
    /// previously registered handle for the same identity.
    pub fn insert(&mut self, member: MemberId, handle: File) -> Option<File> {
        self.handles.insert(member, handle)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.handles.is_empty()
    }

    pub(crate) fn take(&mut self, member: &MemberId) -> Option<File> {
        self.handles.remove(member)
    }
}

/// Where transaction-owned staging state is created for a plan.
///
/// The distinction is an authority boundary, not a convenience: staging
/// placement decides which directory a caller must be able to write in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum StagePlacement {
    /// A private, unpredictable directory beside the installation root.
    ///
    /// Requires write authority in the installation root's **parent**.
    SiblingOfInstallationRoot,
    /// A private, unpredictable directory inside the installation root itself.
    ///
    /// Requires write authority in the installation root only. This is what
    /// [`InstallPlan::for_current_executable`] uses so that replacing the
    /// running executable never needs authority above its own directory.
    InsideInstallationRoot,
}

/// An explicit installation root and a coherent artifact set ready for private preparation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallPlan {
    product: ProductId,
    release: ReleaseId,
    installation_root: PathBuf,
    artifacts: ArtifactSet,
    placement: StagePlacement,
    current_executable: Option<CurrentExecutable>,
}

impl InstallPlan {
    /// Validates an installation root, destination containment, sources, and member uniqueness.
    pub fn new(
        product: ProductId,
        release: ReleaseId,
        installation_root: impl Into<PathBuf>,
        artifacts: ArtifactSet,
    ) -> Result<Self> {
        Self::build(
            product,
            release,
            installation_root.into(),
            artifacts,
            StagePlacement::SiblingOfInstallationRoot,
            None,
        )
    }

    fn build(
        product: ProductId,
        release: ReleaseId,
        installation_root: PathBuf,
        artifacts: ArtifactSet,
        placement: StagePlacement,
        current_executable: Option<CurrentExecutable>,
    ) -> Result<Self> {
        validate_installation_root(&installation_root)?;
        let canonical_root = fs::canonicalize(&installation_root)
            .map_err(|source| Error::io("canonicalizing installation root", source))?;
        let mut destinations = HashSet::new();
        for member in artifacts.as_slice() {
            if !destinations.insert(member.destination.clone()) {
                return Err(Error::invalid(format!(
                    "duplicate destination: {}",
                    member.destination.display()
                )));
            }
            validate_source(member, &canonical_root, &installation_root)?;
        }
        Ok(Self {
            product,
            release,
            installation_root,
            artifacts,
            placement,
            current_executable,
        })
    }

    /// Builds a one-member plan that replaces the executable currently running.
    ///
    /// This is an ordinary [`InstallPlan`], not a second transaction model: it
    /// advances through exactly the same prepared/verified/validated/receipt
    /// path as any other one-member plan, and its terminal contract is the same
    /// [`crate::TransactionReceipt`]. The only differences are authority and
    /// identity:
    ///
    /// - The installation root is the executable's own directory, and staging is
    ///   [`StagePlacement::InsideInstallationRoot`], so the transaction needs
    ///   write authority in that directory and nowhere above it.
    /// - The single destination is the canonical live image, so an invocation
    ///   symlink is followed to its real target instead of being overwritten.
    /// - The bound executable identity is re-proved immediately before mutation.
    ///
    /// The candidate must be a local path outside the executable's directory,
    /// exactly as for any other member. Nothing here selects a release, infers a
    /// version, or reads product identity from the executable: `product`,
    /// `release`, and `integrity` are all caller-supplied.
    pub fn for_current_executable(
        product: ProductId,
        release: ReleaseId,
        candidate: impl Into<PathBuf>,
        current: &CurrentExecutable,
        integrity: IntegrityRequirement,
    ) -> Result<Self> {
        let member = ArtifactMember::new(
            MemberId::new(CURRENT_EXECUTABLE_MEMBER)?,
            candidate,
            current.destination_name(),
        )?
        .with_permissions(PermissionsIntent::Executable)
        .with_integrity(integrity);
        let artifacts = ArtifactSet::single(member)?;
        Self::build(
            product,
            release,
            current.installation_root().to_path_buf(),
            artifacts,
            StagePlacement::InsideInstallationRoot,
            Some(current.clone()),
        )
    }

    /// Returns where this plan creates its private staging state.
    pub fn stage_placement(&self) -> StagePlacement {
        self.placement
    }

    /// Returns the bound current executable when this plan replaces one.
    pub fn current_executable(&self) -> Option<&CurrentExecutable> {
        self.current_executable.as_ref()
    }

    pub(crate) fn revalidate_current_executable(&self) -> Result<()> {
        match &self.current_executable {
            Some(current) => current.revalidate(),
            None => Ok(()),
        }
    }

    /// Returns the consumer-owned product identity.
    pub fn product(&self) -> &ProductId {
        &self.product
    }

    /// Returns the consumer-owned release identity.
    pub fn release(&self) -> &ReleaseId {
        &self.release
    }

    /// Returns the exact installation root used for destination resolution.
    pub fn installation_root(&self) -> &Path {
        &self.installation_root
    }

    /// Returns the validated artifact set.
    pub fn artifacts(&self) -> &ArtifactSet {
        &self.artifacts
    }

    /// Resolves a member destination beneath the exact installation root.
    pub fn destination(&self, member: &MemberId) -> Result<PathBuf> {
        let artifact = self
            .artifacts
            .iter()
            .find(|candidate| candidate.id == *member)
            .ok_or_else(|| Error::UnknownMember(member.to_string()))?;
        Ok(self.installation_root.join(&artifact.destination))
    }

    /// Copies every member into private owned stage state without mutating a destination.
    pub fn prepare(self) -> Result<PreparedTransaction> {
        Stage::prepare(self)
    }

    /// Copies every member into private owned stage state, sourcing bound
    /// members from their already-open objects without any pathname lookup.
    ///
    /// Each member identity present in `bound` is staged by reading the moved
    /// open object from byte zero; its recorded source path is treated as
    /// advisory diagnostics only and is never opened, even if it now names
    /// foreign state. Members absent from `bound` stage from their recorded
    /// source paths exactly as in [`prepare`](Self::prepare).
    ///
    /// Every bound identity must name a member of this plan: leftover handles
    /// fail closed and no staged byte is ever obtained by reopening a member
    /// name as fallback. A failed bound stage copy never falls back to the
    /// recorded path.
    pub fn prepare_with_bound_sources(self, bound: BoundSources) -> Result<PreparedTransaction> {
        Stage::prepare_bound(self, bound)
    }

    #[cfg(test)]
    pub(crate) fn prepare_with_failure(
        self,
        point: crate::test_support::FailurePoint,
    ) -> Result<PreparedTransaction> {
        Stage::prepare_with_failure(self, point)
    }
}

fn validate_installation_root(root: &Path) -> Result<()> {
    if !root.is_absolute() {
        return Err(Error::invalid("installation root must be absolute"));
    }
    let metadata = fs::symlink_metadata(root)
        .map_err(|source| Error::io("reading installation root", source))?;
    if !metadata.is_dir() {
        return Err(Error::invalid("installation root must be a directory"));
    }
    Ok(())
}

fn validate_source(
    member: &ArtifactMember,
    canonical_root: &Path,
    installation_root: &Path,
) -> Result<()> {
    if !member.source.is_absolute() {
        return Err(Error::invalid(format!(
            "source for {} must be absolute",
            member.id
        )));
    }
    let metadata = fs::symlink_metadata(&member.source)
        .map_err(|source| Error::io("reading artifact source", source))?;
    if member.file_kind == FileKind::Regular && !metadata.is_file() {
        return Err(Error::invalid(format!(
            "source for {} is not a regular file",
            member.id
        )));
    }
    let canonical_source = fs::canonicalize(&member.source)
        .map_err(|source| Error::io("canonicalizing artifact source", source))?;
    if canonical_source.starts_with(canonical_root) {
        return Err(Error::invalid(format!(
            "source for {} is inside the installation root",
            member.id
        )));
    }
    let destination = installation_root.join(&member.destination);
    if canonical_source == destination {
        return Err(Error::invalid(format!(
            "source for {} aliases its destination",
            member.id
        )));
    }
    Ok(())
}

pub(crate) fn normalize_relative_path(path: &Path) -> Result<PathBuf> {
    if path.as_os_str().is_empty() || path.is_absolute() {
        return Err(Error::invalid(
            "destination must be a non-empty relative path",
        ));
    }
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => {
                if value.to_string_lossy().chars().any(char::is_control) {
                    return Err(Error::invalid("destination contains control characters"));
                }
                normalized.push(value);
            }
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(Error::invalid("destination escapes the installation root"));
            }
        }
    }
    if normalized.as_os_str().is_empty() {
        return Err(Error::invalid("destination must name a file"));
    }
    Ok(normalized)
}
