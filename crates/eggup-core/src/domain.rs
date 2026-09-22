use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};

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

/// The file kind expected for an artifact member.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    /// A regular file; directories, links, devices, sockets, and FIFOs are rejected.
    Regular,
}

/// The permission intent to apply to a staged member.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionsIntent {
    /// Retain the source file's permissions where the platform exposes them.
    Preserve,
    /// Mark the staged member executable where the platform exposes executable bits.
    Executable,
}

/// The ownership policy for a destination, without selecting a consumer policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ownership {
    /// The caller owns and explicitly describes this destination.
    Managed,
}

/// A declared integrity requirement. Computation is provided by the verification milestone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrityRequirement {
    /// No integrity evidence is attached at this stage.
    None,
    /// The staged bytes must later match this SHA-256 digest.
    Sha256([u8; 32]),
}

/// An authenticity requirement descriptor reserved for a later policy layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthenticityRequirement {
    /// No authenticity claim is made by core.
    None,
    /// The caller requires an authenticity validator before commit.
    Required,
}

/// One explicitly acquired local artifact and its intended destination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactMember {
    id: MemberId,
    source: PathBuf,
    destination: PathBuf,
    file_kind: FileKind,
    permissions: PermissionsIntent,
    ownership: Ownership,
    integrity: IntegrityRequirement,
    authenticity: AuthenticityRequirement,
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
            ownership: Ownership::Managed,
            integrity: IntegrityRequirement::None,
            authenticity: AuthenticityRequirement::None,
        })
    }

    /// Sets the intended staged-file permissions.
    pub fn with_permissions(mut self, permissions: PermissionsIntent) -> Self {
        self.permissions = permissions;
        self
    }

    /// Sets the destination ownership policy.
    pub fn with_ownership(mut self, ownership: Ownership) -> Self {
        self.ownership = ownership;
        self
    }

    /// Attaches a declared integrity requirement without computing it.
    pub fn with_integrity(mut self, integrity: IntegrityRequirement) -> Self {
        self.integrity = integrity;
        self
    }

    /// Attaches an authenticity requirement without claiming that it passed.
    pub fn with_authenticity(mut self, authenticity: AuthenticityRequirement) -> Self {
        self.authenticity = authenticity;
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

    /// Returns the ownership policy.
    pub fn ownership(&self) -> Ownership {
        self.ownership
    }

    /// Returns the declared integrity requirement.
    pub fn integrity(&self) -> IntegrityRequirement {
        self.integrity
    }

    /// Returns the declared authenticity requirement.
    pub fn authenticity(&self) -> AuthenticityRequirement {
        self.authenticity
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

/// An explicit installation root and a coherent artifact set ready for private preparation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallPlan {
    product: ProductId,
    release: ReleaseId,
    installation_root: PathBuf,
    artifacts: ArtifactSet,
}

impl InstallPlan {
    /// Validates an installation root, destination containment, sources, and member uniqueness.
    pub fn new(
        product: ProductId,
        release: ReleaseId,
        installation_root: impl Into<PathBuf>,
        artifacts: ArtifactSet,
    ) -> Result<Self> {
        let installation_root = installation_root.into();
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
        })
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
