#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Optional consumer-side adapter from Eggpack ReleaseManifest v1 to Eggup inputs."]

use eggpack_manifest::{ArtifactForm, ReleaseManifest, MAX_DOCUMENT_BYTES};
use eggup_acquisition::{AcquisitionRequest, FetchLimits};
use eggup_archive::{ArchiveFormat, ArchiveLimits, ArchiveMember, ArchivePlan, BoundMember};
use eggup_core::{
    ArtifactMember, ArtifactSet, BoundSources, InstallPlan, IntegrityRequirement, MemberId,
    PermissionsIntent, ProductId, ReleaseId,
};
use std::{
    collections::HashMap,
    fmt, fs,
    path::{Path, PathBuf},
};

/// Typed adapter failure. Details never contain URLs or file contents.
#[derive(Debug)]
#[non_exhaustive]
pub enum AdapterError {
    /// Manifest or Eggup identity is invalid.
    InvalidManifest,
    /// Exact canonical target is absent.
    TargetNotFound,
    /// Request, acquired-file, or permission maps differ from the required set.
    MapMismatch(&'static str),
    /// Caller artifact limit is lower than the manifest exact size.
    CallerLimitTooSmall(String),
    /// Acquired path is relative, missing, linked, non-regular, or the wrong size.
    InvalidAcquiredFile(String),
    /// Archive member bytes cannot be made installable without qualified extraction.
    ArchiveExtractionRequired,
    /// Caller-selected artifact name does not name a supported archive format.
    UnsupportedArchiveFormat,
    /// Construction in a lower Eggup layer failed.
    Eggup(String),
}

impl fmt::Display for AdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidManifest => f.write_str("invalid manifest or identity"),
            Self::TargetNotFound => f.write_str("canonical target not found"),
            Self::MapMismatch(kind) => write!(f, "{kind} map does not exactly match manifest"),
            Self::CallerLimitTooSmall(name) => {
                write!(f, "caller byte limit is below manifest size for {name}")
            }
            Self::InvalidAcquiredFile(name) => write!(f, "acquired file is invalid for {name}"),
            Self::ArchiveExtractionRequired => f.write_str("archive extraction is required"),
            Self::UnsupportedArchiveFormat => {
                f.write_str("artifact name does not name a supported archive format")
            }
            Self::Eggup(detail) => write!(f, "Eggup input construction failed: {detail}"),
        }
    }
}
impl std::error::Error for AdapterError {}

/// Maximum accepted UTF-8 JSON document size, as defined by Eggpack's manifest schema.
pub const MAX_MANIFEST_BYTES: usize = MAX_DOCUMENT_BYTES;

/// One manifest artifact paired with its exact installed member relationship.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactRequirement {
    /// Manifest artifact name used only as an exact map key.
    pub artifact_name: String,
    /// Exact expected byte count.
    pub exact_size: u64,
    /// Manifest SHA-256 bytes.
    pub sha256: [u8; 32],
    /// Eggup member identity.
    pub member_id: MemberId,
    /// Flat relative install destination.
    pub destination: String,
}

/// Preserved archive member evidence; extraction remains a separate boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveMemberRequirement {
    /// Normalized archive source path.
    pub source: String,
    /// Flat installation identity.
    pub install: String,
    /// Exact member size.
    pub exact_size: u64,
    /// Member SHA-256 bytes.
    pub sha256: [u8; 32],
}

/// Manifest projection for one exact canonical target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestProjection {
    /// Direct or bundle files can become one ArtifactSet after acquisition.
    Installable {
        /// Opaque product identity.
        product: ProductId,
        /// Opaque release identity.
        release: ReleaseId,
        /// Exact selected canonical triple.
        target: String,
        /// Ordered direct or bundle relationships.
        artifacts: Vec<ArtifactRequirement>,
    },
    /// Archive facts are retained but require separately qualified extraction.
    Archive {
        /// Opaque product identity.
        product: ProductId,
        /// Opaque release identity.
        release: ReleaseId,
        /// Exact selected canonical triple.
        target: String,
        /// Archive artifact key.
        artifact_name: String,
        /// Exact archive byte count.
        exact_size: u64,
        /// Archive SHA-256 bytes.
        sha256: [u8; 32],
        /// Expected member relationships, not verified extracted files.
        members: Vec<ArchiveMemberRequirement>,
    },
}

impl ManifestProjection {
    fn installable(
        &self,
    ) -> Result<(&ProductId, &ReleaseId, &[ArtifactRequirement]), AdapterError> {
        match self {
            Self::Installable {
                product,
                release,
                artifacts,
                ..
            } => Ok((product, release, artifacts)),
            Self::Archive { .. } => Err(AdapterError::ArchiveExtractionRequired),
        }
    }

    /// Bind every artifact to one exact caller-selected request and tightened limit.
    pub fn bind_requests(
        &self,
        requests: HashMap<String, AcquisitionRequest>,
        baseline: FetchLimits,
    ) -> Result<Vec<PlannedAcquisition>, AdapterError> {
        baseline
            .validate()
            .map_err(|e| AdapterError::Eggup(e.to_string()))?;
        let requirements = match self {
            Self::Installable { artifacts, .. } => artifacts
                .iter()
                .map(|a| {
                    (
                        a.artifact_name.clone(),
                        a.exact_size,
                        a.sha256,
                        a.member_id.clone(),
                    )
                })
                .collect::<Vec<_>>(),
            Self::Archive {
                artifact_name,
                exact_size,
                sha256,
                ..
            } => vec![(
                artifact_name.clone(),
                *exact_size,
                *sha256,
                MemberId::new(artifact_name.clone())
                    .map_err(|e| AdapterError::Eggup(e.to_string()))?,
            )],
        };
        if requests.len() != requirements.len()
            || requirements.iter().any(|(n, ..)| !requests.contains_key(n))
        {
            return Err(AdapterError::MapMismatch("acquisition request"));
        }
        requirements
            .into_iter()
            .map(|(name, size, sha, member)| {
                if baseline.max_artifact_bytes < size {
                    return Err(AdapterError::CallerLimitTooSmall(name.clone()));
                }
                Ok(PlannedAcquisition {
                    artifact_name: name.clone(),
                    request: requests.get(&name).expect("checked map").clone(),
                    limits: FetchLimits {
                        max_artifact_bytes: size,
                        ..baseline
                    },
                    exact_size: size,
                    sha256: sha,
                    member_id: member,
                })
            })
            .collect()
    }

    /// Validate acquired files and build one all-or-nothing direct/bundle ArtifactSet.
    pub fn materialize_artifact_set(
        &self,
        acquired: HashMap<String, PathBuf>,
        permissions: HashMap<MemberId, PermissionsIntent>,
    ) -> Result<ArtifactSet, AdapterError> {
        let (_, _, reqs) = self.installable()?;
        if acquired.len() != reqs.len()
            || reqs
                .iter()
                .any(|r| !acquired.contains_key(&r.artifact_name))
        {
            return Err(AdapterError::MapMismatch("acquired path"));
        }
        if permissions.len() != reqs.len()
            || reqs.iter().any(|r| !permissions.contains_key(&r.member_id))
        {
            return Err(AdapterError::MapMismatch("permissions"));
        }
        let mut members = Vec::with_capacity(reqs.len());
        for r in reqs {
            let path = acquired.get(&r.artifact_name).expect("checked map");
            if !path.is_absolute() {
                return Err(AdapterError::InvalidAcquiredFile(r.artifact_name.clone()));
            }
            let meta = fs::symlink_metadata(path)
                .map_err(|_| AdapterError::InvalidAcquiredFile(r.artifact_name.clone()))?;
            if !meta.file_type().is_file() || meta.len() != r.exact_size {
                return Err(AdapterError::InvalidAcquiredFile(r.artifact_name.clone()));
            }
            members.push(
                ArtifactMember::new(r.member_id.clone(), path, &r.destination)
                    .map_err(|e| AdapterError::Eggup(e.to_string()))?
                    .with_permissions(*permissions.get(&r.member_id).expect("checked map"))
                    .with_integrity(IntegrityRequirement::Sha256(r.sha256)),
            );
        }
        ArtifactSet::new(members).map_err(|e| AdapterError::Eggup(e.to_string()))
    }
}

/// Exact request and manifest evidence to pass to an acquisition transport.
#[derive(Debug, Clone)]
pub struct PlannedAcquisition {
    /// Exact manifest artifact key.
    pub artifact_name: String,
    /// Exact unmodified caller request.
    pub request: AcquisitionRequest,
    /// Caller limits with artifact byte cap tightened to manifest size.
    pub limits: FetchLimits,
    /// Expected byte count.
    pub exact_size: u64,
    /// Expected SHA-256 bytes.
    pub sha256: [u8; 32],
    /// Related Eggup member identity.
    pub member_id: MemberId,
}

/// Validate and project one exact canonical target without performing I/O.
pub fn project(
    manifest: &ReleaseManifest,
    canonical_target: &str,
) -> Result<ManifestProjection, AdapterError> {
    manifest
        .validate()
        .map_err(|_| AdapterError::InvalidManifest)?;
    let target = manifest
        .target(canonical_target)
        .map_err(|_| AdapterError::TargetNotFound)?;
    let product = ProductId::new(manifest.product_id.clone())
        .map_err(|e| AdapterError::Eggup(e.to_string()))?;
    let release = ReleaseId::new(manifest.release_id.clone())
        .map_err(|e| AdapterError::Eggup(e.to_string()))?;
    let common = (product, release, target.target.clone());
    match &target.form {
        ArtifactForm::Direct { artifact, install } => Ok(ManifestProjection::Installable {
            product: common.0,
            release: common.1,
            target: common.2,
            artifacts: vec![ArtifactRequirement {
                artifact_name: artifact.name.clone(),
                exact_size: artifact.size,
                sha256: artifact
                    .sha256_bytes()
                    .map_err(|_| AdapterError::InvalidManifest)?,
                member_id: MemberId::new(install.clone())
                    .map_err(|e| AdapterError::Eggup(e.to_string()))?,
                destination: install.clone(),
            }],
        }),
        ArtifactForm::Bundle { entries } => Ok(ManifestProjection::Installable {
            product: common.0,
            release: common.1,
            target: common.2,
            artifacts: entries
                .iter()
                .map(|e| {
                    Ok(ArtifactRequirement {
                        artifact_name: e.artifact.name.clone(),
                        exact_size: e.artifact.size,
                        sha256: e
                            .artifact
                            .sha256_bytes()
                            .map_err(|_| AdapterError::InvalidManifest)?,
                        member_id: MemberId::new(e.install.clone())
                            .map_err(|x| AdapterError::Eggup(x.to_string()))?,
                        destination: e.install.clone(),
                    })
                })
                .collect::<Result<_, AdapterError>>()?,
        }),
        ArtifactForm::Archive { artifact, members } => Ok(ManifestProjection::Archive {
            product: common.0,
            release: common.1,
            target: common.2,
            artifact_name: artifact.name.clone(),
            exact_size: artifact.size,
            sha256: artifact
                .sha256_bytes()
                .map_err(|_| AdapterError::InvalidManifest)?,
            members: members
                .iter()
                .map(|m| {
                    Ok(ArchiveMemberRequirement {
                        source: m.source.clone(),
                        install: m.install.clone(),
                        exact_size: m.bytes.size,
                        sha256: m
                            .bytes
                            .sha256_bytes()
                            .map_err(|_| AdapterError::InvalidManifest)?,
                    })
                })
                .collect::<Result<_, AdapterError>>()?,
        }),
    }
}

/// Parse bounded ReleaseManifest JSON and project one exact canonical target.
///
/// Parsing and schema validation are delegated to `eggpack-manifest`. Input
/// contents and parser diagnostics are never included in returned errors.
pub fn project_json(
    input: &[u8],
    canonical_target: &str,
) -> Result<ManifestProjection, AdapterError> {
    if input.len() > MAX_MANIFEST_BYTES {
        return Err(AdapterError::InvalidManifest);
    }
    let json = std::str::from_utf8(input).map_err(|_| AdapterError::InvalidManifest)?;
    let manifest = ReleaseManifest::from_json(json).map_err(|_| AdapterError::InvalidManifest)?;
    project(&manifest, canonical_target)
}

/// Return product and release identities for an installable projection.
pub fn install_ids(
    projection: &ManifestProjection,
) -> Result<(&ProductId, &ReleaseId), AdapterError> {
    let (product, release, _) = projection.installable()?;
    Ok((product, release))
}

// ---------------------------------------------------------------------------
// Archive extraction handoff (M002).
//
// These helpers translate `ManifestProjection::Archive` into typed inputs the
// caller feeds to `eggup-archive::extract` and `eggup-core`'s
// `BoundSources`-based prepare seam. They perform no I/O, no extraction, no
// commit, and never reach outside their arguments; composition stays
// caller-owned so lifetimes and authority stay obvious.
//
// Required caller order:
//   1. validate the acquired archive against `projection` (size + SHA-256);
//   2. call `archive_plan_for` to obtain an `ArchivePlan`;
//   3. extract with `eggup_archive::extract`;
//   4. capture `extracted.root()` before persisting;
//   5. call `core_plan_for_archive` to obtain the core `InstallPlan` while
//      advisory paths are still meaningful;
//   6. call `extracted.persist().into_bound_sources()?.into_members()` to
//      obtain the bound members and deferred cleanup;
//   7. call `bind_archive_members` to obtain a `BoundSources` keyed by member
//      identity from the projection;
//   8. call `InstallPlan::prepare_with_bound_sources`.
// ---------------------------------------------------------------------------

/// Map a caller-selected archive artifact file name to a supported format.
///
/// Only `.tar.gz`, `.tgz`, and `.zip` suffixes (ASCII case-insensitive) are
/// accepted; anything else fails closed. The caller owns artifact selection:
/// this helper never fetches, lists, or ranks releases, it only classifies
/// the name the caller already chose.
pub fn archive_format_for_name(file_name: &str) -> Result<ArchiveFormat, AdapterError> {
    if file_name.is_empty() || !file_name.is_ascii() {
        return Err(AdapterError::UnsupportedArchiveFormat);
    }
    let lower = file_name.to_ascii_lowercase();
    if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") {
        Ok(ArchiveFormat::TarGz)
    } else if lower.ends_with(".zip") {
        Ok(ArchiveFormat::Zip)
    } else {
        Err(AdapterError::UnsupportedArchiveFormat)
    }
}

/// Validate caller-owned acquired archive continuity against the manifest.
///
/// Checks that `acquired_archive_path` is absolute, names a regular file
/// (never a symlink, directory, or missing path), is exactly the manifest
/// archive byte count, and hashes to the manifest archive SHA-256. This is
/// the same continuity gate `materialize_artifact_set` applies to
/// direct/bundle files, moved ahead of extraction so no unverified byte is
/// ever extracted. Performs bounded streaming reads only; no network, no
/// alternate-source retry, no provenance claim.
pub fn validate_acquired_archive(
    projection: &ManifestProjection,
    acquired_archive_path: &Path,
) -> Result<(), AdapterError> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    let (artifact_name, exact_size, sha256) = match projection {
        ManifestProjection::Archive {
            artifact_name,
            exact_size,
            sha256,
            ..
        } => (artifact_name, *exact_size, *sha256),
        _ => return Err(AdapterError::ArchiveExtractionRequired),
    };
    if !acquired_archive_path.is_absolute() {
        return Err(AdapterError::InvalidAcquiredFile(artifact_name.clone()));
    }
    let meta = fs::symlink_metadata(acquired_archive_path)
        .map_err(|_| AdapterError::InvalidAcquiredFile(artifact_name.clone()))?;
    if !meta.file_type().is_file() || meta.len() != exact_size {
        return Err(AdapterError::InvalidAcquiredFile(artifact_name.clone()));
    }
    let mut file = fs::File::open(acquired_archive_path)
        .map_err(|_| AdapterError::InvalidAcquiredFile(artifact_name.clone()))?;
    let mut hasher = Sha256::new();
    let mut chunk = [0u8; 16 * 1024];
    loop {
        let read = file
            .read(&mut chunk)
            .map_err(|_| AdapterError::InvalidAcquiredFile(artifact_name.clone()))?;
        if read == 0 {
            break;
        }
        hasher.update(&chunk[..read]);
    }
    let digest: [u8; 32] = hasher.finalize().into();
    if digest != sha256 {
        return Err(AdapterError::InvalidAcquiredFile(artifact_name.clone()));
    }
    Ok(())
}

/// Translate `ManifestProjection::Archive` into a validated `ArchivePlan`.
///
/// Each declared member maps `source -> eggup_archive::ArchiveMember::source_path`
/// and `install -> eggup_archive::ArchiveMember::output_name`. Caller-supplied
/// `expected_size` and `expected_sha256` come from the projection so the
/// extractor can verify bytes against the manifest during extraction.
///
/// The caller must already have validated the acquired archive against the
/// manifest's exact size and SHA-256; this helper does not re-validate the
/// archive bytes and does not perform any I/O.
pub fn archive_plan_for(
    projection: &ManifestProjection,
    archive_format: ArchiveFormat,
    acquired_archive_path: &Path,
    limits: ArchiveLimits,
) -> Result<ArchivePlan, AdapterError> {
    let members = match projection {
        ManifestProjection::Archive { members, .. } => members,
        _ => return Err(AdapterError::ArchiveExtractionRequired),
    };
    let mut declared = Vec::with_capacity(members.len());
    for m in members {
        let member = ArchiveMember::new(
            m.source.clone(),
            m.install.clone(),
            Some(m.exact_size),
            Some(m.sha256),
        )
        .map_err(|e| AdapterError::Eggup(e.to_string()))?;
        declared.push(member);
    }
    ArchivePlan::new(acquired_archive_path, archive_format, declared, limits)
        .map_err(|e| AdapterError::Eggup(e.to_string()))
}

/// Build the core `InstallPlan` for an archive projection, recording each
/// member's advisory path as `extraction_root.join(member.install)`.
///
/// The caller is responsible for capturing `extraction_root` (typically
/// `ExtractedArchive::root()`) before any handle-bound handoff, and for
/// keeping the path reachable until staging no longer needs it.
/// Advisory paths are never opened during staging when a bound handle is
/// supplied; the path is recorded solely to satisfy
/// `ArtifactMember::new`'s validation and to expose diagnostics.
///
/// The returned plan's `BoundSources` are still empty at this point; the
/// caller transfers bound members separately via `bind_archive_members`.
pub fn core_plan_for_archive(
    projection: &ManifestProjection,
    extraction_root: &Path,
    destination_root: &Path,
    permissions: HashMap<MemberId, PermissionsIntent>,
) -> Result<InstallPlan, AdapterError> {
    let (product, release, members) = match projection {
        ManifestProjection::Archive {
            product,
            release,
            members,
            ..
        } => (product, release, members),
        _ => return Err(AdapterError::ArchiveExtractionRequired),
    };
    if permissions.len() != members.len() {
        return Err(AdapterError::MapMismatch("permissions"));
    }
    let mut artifacts = Vec::with_capacity(members.len());
    for m in members {
        let id =
            MemberId::new(m.install.clone()).map_err(|e| AdapterError::Eggup(e.to_string()))?;
        let advisory = extraction_root.join(&m.install);
        let permissions_intent = permissions
            .get(&id)
            .copied()
            .ok_or(AdapterError::MapMismatch("permissions"))?;
        let member = ArtifactMember::new(id.clone(), &advisory, &m.install)
            .map_err(|e| AdapterError::Eggup(e.to_string()))?
            .with_permissions(permissions_intent)
            .with_integrity(IntegrityRequirement::Sha256(m.sha256));
        artifacts.push(member);
    }
    let set = ArtifactSet::new(artifacts).map_err(|e| AdapterError::Eggup(e.to_string()))?;
    InstallPlan::new(product.clone(), release.clone(), destination_root, set)
        .map_err(|e| AdapterError::Eggup(e.to_string()))
}

/// Build a `BoundSources` map from the projection's archive member list and
/// the bound members returned by `BoundExtraction::into_members()`.
///
/// Identity is the projection's `install` string converted into a `MemberId`;
/// bound members are paired to the projection's archive members in declaration
/// order. Mismatched counts fail closed.
pub fn bind_archive_members(
    projection: &ManifestProjection,
    bound_members: Vec<BoundMember>,
) -> Result<BoundSources, AdapterError> {
    let members = match projection {
        ManifestProjection::Archive { members, .. } => members,
        _ => return Err(AdapterError::ArchiveExtractionRequired),
    };
    if members.len() != bound_members.len() {
        return Err(AdapterError::MapMismatch("archive bound members"));
    }
    let mut sources = BoundSources::new();
    for (declared, bound) in members.iter().zip(bound_members.into_iter()) {
        let id = MemberId::new(declared.install.clone())
            .map_err(|e| AdapterError::Eggup(e.to_string()))?;
        sources.insert(id, bound.into_open_object());
    }
    Ok(sources)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn direct_json() -> Vec<u8> {
        include_bytes!("../tests/fixtures/direct-manifest.json").to_vec()
    }

    #[test]
    fn bounded_json_projection_accepts_fixture_and_exact_limit() {
        let fixture = direct_json();
        assert!(fixture.len() < MAX_MANIFEST_BYTES);
        assert!(matches!(
            project_json(&fixture, "x86_64-unknown-linux-gnu").unwrap(),
            ManifestProjection::Installable { .. }
        ));

        let mut at_limit = fixture;
        at_limit.resize(MAX_MANIFEST_BYTES, b' ');
        assert!(project_json(&at_limit, "x86_64-unknown-linux-gnu").is_ok());
    }

    #[test]
    fn bounded_json_projection_rejects_invalid_documents_without_leaking_input() {
        let oversized = vec![b' '; MAX_MANIFEST_BYTES + 1];
        let invalid_utf8 = [b'{', 0xff, b'}'];
        let malformed = br#"{"private":"https://user:super-secret@example.invalid/?token=hidden""#;
        let unsupported: Vec<u8> = String::from_utf8(direct_json())
            .unwrap()
            .replace("\"schema_version\":1", "\"schema_version\":999")
            .into_bytes();

        for input in [
            &oversized[..],
            &invalid_utf8[..],
            &malformed[..],
            &unsupported[..],
        ] {
            let error = project_json(input, "x86_64-unknown-linux-gnu").unwrap_err();
            assert!(matches!(error, AdapterError::InvalidManifest));
            let diagnostic = format!("{error} {error:?}");
            assert!(!diagnostic.contains("private"));
            assert!(!diagnostic.contains("super-secret"));
            assert!(!diagnostic.contains("token=hidden"));
        }
    }

    #[test]
    fn json_projection_keeps_exact_target_selection() {
        let error = project_json(&direct_json(), "linux-x64").unwrap_err();
        assert!(matches!(error, AdapterError::TargetNotFound));
    }

    #[test]
    fn corrected_fixtures_project_and_archive_stays_separate() {
        let direct: ReleaseManifest =
            serde_json::from_str(include_str!("../tests/fixtures/direct-manifest.json")).unwrap();
        assert!(matches!(
            project(&direct, "x86_64-unknown-linux-gnu").unwrap(),
            ManifestProjection::Installable { .. }
        ));
        let archive: ReleaseManifest =
            serde_json::from_str(include_str!("../tests/fixtures/archive-manifest.json")).unwrap();
        let p = project(&archive, "x86_64-unknown-linux-gnu").unwrap();
        assert!(matches!(p, ManifestProjection::Archive { .. }));
        assert!(matches!(
            p.bind_requests(HashMap::new(), FetchLimits::default()),
            Err(AdapterError::MapMismatch(_))
        ));
        let b: ReleaseManifest =
            serde_json::from_str(include_str!("../tests/fixtures/bundle-manifest.json")).unwrap();
        assert!(
            matches!(project(&b, "x86_64-unknown-linux-gnu").unwrap(), ManifestProjection::Installable { artifacts, .. } if artifacts.len() == 3)
        );
    }
    #[test]
    fn alias_target_is_not_resolved() {
        let m: ReleaseManifest =
            serde_json::from_str(include_str!("../tests/fixtures/direct-manifest.json")).unwrap();
        assert!(matches!(
            project(&m, "linux-x64"),
            Err(AdapterError::TargetNotFound)
        ));
    }

    #[test]
    fn exact_request_binding_preserves_url_and_tightens_byte_limit() {
        let m: ReleaseManifest =
            serde_json::from_str(include_str!("../tests/fixtures/direct-manifest.json")).unwrap();
        let p = project(&m, "x86_64-unknown-linux-gnu").unwrap();
        let name = match &p {
            ManifestProjection::Installable { artifacts, .. } => artifacts[0].artifact_name.clone(),
            _ => unreachable!(),
        };
        let request =
            AcquisitionRequest::new("https://user:secret@example.test/path?token=kept").unwrap();
        let mut requests = HashMap::new();
        requests.insert(name.clone(), request.clone());
        let bound = p.bind_requests(requests, FetchLimits::default()).unwrap();
        assert_eq!(bound[0].request, request);
        assert_eq!(bound[0].limits.max_artifact_bytes, bound[0].exact_size);
        let too_small = FetchLimits {
            max_artifact_bytes: bound[0].exact_size - 1,
            ..FetchLimits::default()
        };
        assert!(matches!(
            p.bind_requests(HashMap::from([(name, request)]), too_small),
            Err(AdapterError::CallerLimitTooSmall(_))
        ));
    }

    #[test]
    fn exact_size_and_explicit_permissions_gate_artifact_materialization() {
        let m: ReleaseManifest =
            serde_json::from_str(include_str!("../tests/fixtures/direct-manifest.json")).unwrap();
        let p = project(&m, "x86_64-unknown-linux-gnu").unwrap();
        let requirement = match &p {
            ManifestProjection::Installable { artifacts, .. } => &artifacts[0],
            _ => unreachable!(),
        };
        let dir = std::env::temp_dir().join(format!("eggup-eggpack-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("source");
        fs::write(&path, b"abc").unwrap();
        let acquired = HashMap::from([(requirement.artifact_name.clone(), path.clone())]);
        let permissions =
            HashMap::from([(requirement.member_id.clone(), PermissionsIntent::Executable)]);
        assert!(p
            .materialize_artifact_set(acquired.clone(), permissions.clone())
            .is_ok());
        fs::write(&path, b"no").unwrap();
        assert!(matches!(
            p.materialize_artifact_set(acquired, permissions),
            Err(AdapterError::InvalidAcquiredFile(_))
        ));
        let _ = fs::remove_dir_all(dir);
    }
}
