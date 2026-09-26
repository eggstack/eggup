//! M001a corrective compatibility matrix for the optional Eggpack adapter.
//!
//! These tests load the checked-in fixtures under `tests/fixtures/` (copied
//! byte-for-byte from Eggpack `678bbf04f5a02827003a1d9ab83ba4f0e6360e41`) and
//! compare adapter output against the `projection-*.json` fixtures with strict
//! test-only structs. The structs below are test evidence only, not a
//! production schema, and are intentionally not exported from the crate.

use eggpack_manifest::ReleaseManifest;
use eggup_acquisition::{AcquisitionRequest, FetchLimits};
use eggup_core::{IntegrityRequirement, MemberId, PermissionsIntent};
use eggup_eggpack::{project, AdapterError, ManifestProjection};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

const LINUX_TARGET: &str = "x86_64-unknown-linux-gnu";
const ALIAS_TARGET: &str = "linux-x64";
const PINNED_EGGPACK_REV: &str = "678bbf04f5a02827003a1d9ab83ba4f0e6360e41";

const DIRECT_MANIFEST: &str = include_str!("fixtures/direct-manifest.json");
const DIRECT_PROJECTION_JSON: &str = include_str!("fixtures/projection-direct.json");
const BUNDLE_MANIFEST: &str = include_str!("fixtures/bundle-manifest.json");
const BUNDLE_PROJECTION_JSON: &str = include_str!("fixtures/projection-bundle.json");
const ARCHIVE_MANIFEST: &str = include_str!("fixtures/archive-manifest.json");
const ARCHIVE_PROJECTION_JSON: &str = include_str!("fixtures/projection-archive.json");
const UNKNOWN_SCHEMA_JSON: &str = include_str!("fixtures/unknown-schema.json");
const WRONG_TARGET_JSON: &str = include_str!("fixtures/wrong-target.json");

// ---------------------------------------------------------------------------
// Test-only strict projection structs (evidence only, never exported).
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ReleaseIdentity {
    product_id: String,
    release_id: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct InstallableUnit {
    name: String,
    exact_size: u64,
    sha256: String,
    member_id: String,
    relative_destination: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct DirectProjection {
    selected_target: String,
    release_identity: ReleaseIdentity,
    acquisition_units: Vec<InstallableUnit>,
    transaction_group: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct BundleProjection {
    selected_target: String,
    release_identity: ReleaseIdentity,
    acquisition_units: Vec<InstallableUnit>,
    transaction_group: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ArchiveAcquisitionUnit {
    name: String,
    exact_size: u64,
    sha256: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ArchiveProjectedMember {
    source: String,
    install: String,
    size: u64,
    sha256: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ArchiveProjection {
    selected_target: String,
    acquisition_units: Vec<ArchiveAcquisitionUnit>,
    members: Vec<ArchiveProjectedMember>,
    extraction_required: bool,
    transaction_group: String,
}

// ---------------------------------------------------------------------------
// Helpers.
// ---------------------------------------------------------------------------

fn parse_manifest(json: &str) -> ReleaseManifest {
    ReleaseManifest::from_json(json).expect("fixture must be valid ReleaseManifest v1")
}

fn parse_direct_projection() -> DirectProjection {
    serde_json::from_str(DIRECT_PROJECTION_JSON).expect("projection-direct must parse strictly")
}

fn parse_bundle_projection() -> BundleProjection {
    serde_json::from_str(BUNDLE_PROJECTION_JSON).expect("projection-bundle must parse strictly")
}

fn parse_archive_projection() -> ArchiveProjection {
    serde_json::from_str(ARCHIVE_PROJECTION_JSON).expect("projection-archive must parse strictly")
}

fn decode_sha256(hex: &str) -> [u8; 32] {
    assert_eq!(hex.len(), 64, "sha256 hex must be 64 chars");
    let mut out = [0u8; 32];
    for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
        out[i] = u8::from_str_radix(std::str::from_utf8(chunk).unwrap(), 16).unwrap();
    }
    out
}

fn exact_requests(names: &[String], url_prefix: &str) -> HashMap<String, AcquisitionRequest> {
    names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            (
                name.clone(),
                AcquisitionRequest::new(format!("{url_prefix}/{i}/{name}")).unwrap(),
            )
        })
        .collect()
}

fn installable_parts(
    projection: &ManifestProjection,
) -> (
    &eggup_core::ProductId,
    &eggup_core::ReleaseId,
    &[eggup_eggpack::ArtifactRequirement],
) {
    match projection {
        ManifestProjection::Installable {
            product,
            release,
            artifacts,
            ..
        } => (product, release, artifacts),
        ManifestProjection::Archive { .. } => panic!("expected installable projection"),
    }
}

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(label: &str) -> Self {
        let id = TEMP_COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "eggup-eggpack-{}-{}-{}",
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

fn write_sized_file(dir: &Path, name: &str, size: u64) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, vec![b'x'; size as usize]).unwrap();
    path
}

fn acquired_map(names_sizes: &[(&str, u64)], dir: &Path) -> HashMap<String, PathBuf> {
    names_sizes
        .iter()
        .map(|(name, size)| (name.to_string(), write_sized_file(dir, name, *size)))
        .collect()
}

fn permissions_map(
    projection: &ManifestProjection,
    intent: PermissionsIntent,
) -> HashMap<MemberId, PermissionsIntent> {
    let (_, _, reqs) = installable_parts(projection);
    reqs.iter().map(|r| (r.member_id.clone(), intent)).collect()
}

fn check_direct_against_fixture(
    projection: &ManifestProjection,
    expected: &DirectProjection,
    manifest: &ReleaseManifest,
) -> Result<(), String> {
    let (product, release, artifacts) = installable_parts(projection);
    if expected.selected_target != LINUX_TARGET {
        return Err(format!(
            "selected_target {} != {LINUX_TARGET}",
            expected.selected_target
        ));
    }
    if product.as_str() != expected.release_identity.product_id
        || product.as_str() != manifest.product_id
    {
        return Err("product identity mismatch".to_string());
    }
    if release.as_str() != expected.release_identity.release_id
        || release.as_str() != manifest.release_id
    {
        return Err("release identity mismatch".to_string());
    }
    if expected.acquisition_units.len() != 1 || artifacts.len() != 1 {
        return Err("direct projection must have exactly one unit".to_string());
    }
    let unit = &expected.acquisition_units[0];
    let req = &artifacts[0];
    if req.artifact_name != unit.name
        || req.exact_size != unit.exact_size
        || req.sha256 != decode_sha256(&unit.sha256)
        || req.member_id.as_str() != unit.member_id
        || req.destination != unit.relative_destination
    {
        return Err("direct adapter requirement does not match projection fixture".to_string());
    }
    if expected.transaction_group != "one-artifact-set" {
        return Err("direct transaction_group mismatch".to_string());
    }
    Ok(())
}

fn check_bundle_against_fixture(
    projection: &ManifestProjection,
    expected: &BundleProjection,
    manifest: &ReleaseManifest,
) -> Result<(), String> {
    let (product, release, artifacts) = installable_parts(projection);
    if expected.selected_target != LINUX_TARGET {
        return Err("bundle selected_target mismatch".to_string());
    }
    if product.as_str() != expected.release_identity.product_id
        || product.as_str() != manifest.product_id
    {
        return Err("bundle product identity mismatch".to_string());
    }
    if release.as_str() != expected.release_identity.release_id
        || release.as_str() != manifest.release_id
    {
        return Err("bundle release identity mismatch".to_string());
    }
    if artifacts.len() != expected.acquisition_units.len() {
        return Err(format!(
            "bundle count {} != fixture {}",
            artifacts.len(),
            expected.acquisition_units.len()
        ));
    }
    // Exact pairing in both directions preserves artifact/install relationships.
    for req in artifacts {
        let matched = expected.acquisition_units.iter().any(|u| {
            u.name == req.artifact_name
                && u.exact_size == req.exact_size
                && decode_sha256(&u.sha256) == req.sha256
                && u.member_id == req.member_id.as_str()
                && u.relative_destination == req.destination
        });
        if !matched {
            return Err(format!(
                "no fixture unit matches adapter requirement {}",
                req.artifact_name
            ));
        }
    }
    for unit in &expected.acquisition_units {
        let matched = artifacts.iter().any(|r| {
            r.artifact_name == unit.name
                && r.exact_size == unit.exact_size
                && r.sha256 == decode_sha256(&unit.sha256)
                && r.member_id.as_str() == unit.member_id
                && r.destination == unit.relative_destination
        });
        if !matched {
            return Err(format!(
                "fixture unit {} has no paired adapter requirement",
                unit.name
            ));
        }
    }
    // Deterministic order: adapter order must equal fixture order exactly.
    for (req, unit) in artifacts.iter().zip(expected.acquisition_units.iter()) {
        if req.artifact_name != unit.name {
            return Err("bundle order is not deterministic".to_string());
        }
    }
    if expected.transaction_group != "one-artifact-set" {
        return Err("bundle transaction_group mismatch".to_string());
    }
    Ok(())
}

fn check_archive_against_fixture(
    projection: &ManifestProjection,
    expected: &ArchiveProjection,
    manifest: &ReleaseManifest,
) -> Result<(), String> {
    let (artifact_name, exact_size, sha256, members) = match projection {
        ManifestProjection::Archive {
            artifact_name,
            exact_size,
            sha256,
            members,
            ..
        } => (artifact_name, *exact_size, *sha256, members),
        _ => return Err("expected archive projection".to_string()),
    };
    let target = manifest
        .target(LINUX_TARGET)
        .map_err(|e| format!("manifest target: {e}"))?;
    let (m_artifact, m_members) = match &target.form {
        eggpack_manifest::ArtifactForm::Archive { artifact, members } => (artifact, members),
        _ => return Err("manifest form is not archive".to_string()),
    };
    if expected.selected_target != LINUX_TARGET {
        return Err("archive selected_target mismatch".to_string());
    }
    if expected.acquisition_units.len() != 1 {
        return Err("archive must have exactly one acquisition unit".to_string());
    }
    let unit = &expected.acquisition_units[0];
    if unit.name != *artifact_name
        || unit.name != m_artifact.name
        || unit.exact_size != exact_size
        || unit.exact_size != m_artifact.size
        || decode_sha256(&unit.sha256) != sha256
        || unit.sha256 != m_artifact.sha256
    {
        return Err("archive artifact facts do not match".to_string());
    }
    if members.len() != expected.members.len() || members.len() != m_members.len() {
        return Err("archive member count mismatch".to_string());
    }
    for m in members {
        let matched = expected.members.iter().any(|p| {
            p.source == m.source
                && p.install == m.install
                && p.size == m.exact_size
                && decode_sha256(&p.sha256) == m.sha256
        });
        if !matched {
            return Err(format!("archive member {} unmatched in fixture", m.source));
        }
        let in_manifest = m_members.iter().any(|mm| {
            mm.source == m.source
                && mm.install == m.install
                && mm.bytes.size == m.exact_size
                && mm.bytes.sha256_bytes().unwrap() == m.sha256
        });
        if !in_manifest {
            return Err(format!("archive member {} unmatched in manifest", m.source));
        }
    }
    if !expected.extraction_required {
        return Err("archive fixture must declare extraction_required=true".to_string());
    }
    if !expected.transaction_group.contains("consumer-owned") {
        return Err("archive transaction_group must reflect consumer-owned extraction".to_string());
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Positive matrix.
// ---------------------------------------------------------------------------

#[test]
fn direct_positive_matches_projection_fixture() {
    let manifest = parse_manifest(DIRECT_MANIFEST);
    let projection = project(&manifest, LINUX_TARGET).unwrap();
    let expected = parse_direct_projection();
    check_direct_against_fixture(&projection, &expected, &manifest).unwrap();

    let (_, _, artifacts) = installable_parts(&projection);
    assert_eq!(artifacts.len(), 1);
    let req = &artifacts[0];
    let unit = &expected.acquisition_units[0];
    assert_eq!(req.artifact_name, unit.name);
    assert_eq!(req.exact_size, unit.exact_size);
    assert_eq!(req.sha256, decode_sha256(&unit.sha256));
    assert_eq!(req.member_id.as_str(), unit.member_id);
    assert_eq!(req.destination, unit.relative_destination);
}

#[test]
fn direct_request_binding_preserves_url_and_tightens_limits() {
    let manifest = parse_manifest(DIRECT_MANIFEST);
    let projection = project(&manifest, LINUX_TARGET).unwrap();
    let (_, _, artifacts) = installable_parts(&projection);
    let name = artifacts[0].artifact_name.clone();
    let exact = artifacts[0].exact_size;

    let url = "https://user:secret@example.test/path?token=kept-exact";
    let request = AcquisitionRequest::new(url).unwrap();
    let baseline = FetchLimits {
        max_metadata_bytes: 64 * 1024,
        max_artifact_bytes: exact + 1024,
        connect_timeout: Duration::from_secs(7),
        total_timeout: Duration::from_secs(77),
    };
    let bound = projection
        .bind_requests(HashMap::from([(name.clone(), request)]), baseline)
        .unwrap();
    assert_eq!(bound.len(), 1);
    // Exact caller URL retained byte-for-byte.
    assert_eq!(bound[0].request.url(), url);
    // Byte cap tightened to the exact manifest size.
    assert_eq!(bound[0].limits.max_artifact_bytes, exact);
    // Caller timeouts and metadata bound preserved, never widened.
    assert_eq!(bound[0].limits.connect_timeout, Duration::from_secs(7));
    assert_eq!(bound[0].limits.total_timeout, Duration::from_secs(77));
    assert_eq!(bound[0].limits.max_metadata_bytes, 64 * 1024);
    assert_eq!(bound[0].exact_size, exact);
}

#[test]
fn direct_materialization_yields_single_member_with_manifest_digest() {
    let manifest = parse_manifest(DIRECT_MANIFEST);
    let projection = project(&manifest, LINUX_TARGET).unwrap();
    let (_, _, artifacts) = installable_parts(&projection);
    let req = &artifacts[0];

    let tmp = TempDir::new("direct-positive");
    let path = write_sized_file(tmp.path(), "artifact", req.exact_size);
    let acquired = HashMap::from([(req.artifact_name.clone(), path)]);
    let permissions = HashMap::from([(req.member_id.clone(), PermissionsIntent::Executable)]);
    let set = projection
        .materialize_artifact_set(acquired, permissions)
        .unwrap();
    assert_eq!(set.len(), 1);
    let member = set.iter().next().unwrap();
    assert_eq!(member.id(), &req.member_id);
    assert_eq!(member.integrity(), IntegrityRequirement::Sha256(req.sha256));
    assert_eq!(
        member.integrity(),
        IntegrityRequirement::Sha256(decode_sha256(
            &parse_direct_projection().acquisition_units[0].sha256
        ))
    );
    assert_eq!(member.permissions(), PermissionsIntent::Executable);
}

#[test]
fn bundle_positive_matches_corrected_three_member_fixture() {
    let manifest = parse_manifest(BUNDLE_MANIFEST);
    let projection = project(&manifest, LINUX_TARGET).unwrap();
    let expected = parse_bundle_projection();
    assert_eq!(expected.acquisition_units.len(), 3);
    check_bundle_against_fixture(&projection, &expected, &manifest).unwrap();

    let (_, _, artifacts) = installable_parts(&projection);
    assert_eq!(artifacts.len(), 3, "corrected CodeGG bundle cardinality");
    // Member identities and destinations remain distinct.
    let mut ids: Vec<_> = artifacts.iter().map(|a| a.member_id.as_str()).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), 3);
    // No entry from another product can appear.
    for req in artifacts {
        assert!(
            !req.artifact_name.starts_with("eggsact"),
            "bundle must not contain eggsact entries"
        );
    }
    // Deterministic: projecting twice yields identical order.
    let again = project(&manifest, LINUX_TARGET).unwrap();
    assert_eq!(projection, again);
}

#[test]
fn bundle_exact_maps_produce_three_member_set() {
    let manifest = parse_manifest(BUNDLE_MANIFEST);
    let projection = project(&manifest, LINUX_TARGET).unwrap();
    let (_, _, artifacts) = installable_parts(&projection);
    let names: Vec<String> = artifacts.iter().map(|a| a.artifact_name.clone()).collect();

    let requests = exact_requests(&names, "https://example.test/codegg");
    let bound = projection
        .bind_requests(requests, FetchLimits::default())
        .unwrap();
    assert_eq!(bound.len(), 3);
    for b in &bound {
        assert_eq!(b.limits.max_artifact_bytes, b.exact_size);
    }

    let tmp = TempDir::new("bundle-positive");
    let sizes: Vec<(&str, u64)> = artifacts
        .iter()
        .map(|a| (a.artifact_name.as_str(), a.exact_size))
        .collect();
    let acquired = acquired_map(&sizes, tmp.path());
    let permissions = permissions_map(&projection, PermissionsIntent::Preserve);
    let set = projection
        .materialize_artifact_set(acquired, permissions)
        .unwrap();
    assert_eq!(set.len(), 3);
    for member in set.iter() {
        let req = artifacts
            .iter()
            .find(|r| &r.member_id == member.id())
            .expect("member must pair with a requirement");
        assert_eq!(member.integrity(), IntegrityRequirement::Sha256(req.sha256));
    }
}

#[test]
fn archive_positive_preserves_facts_and_blocks_materialization() {
    let manifest = parse_manifest(ARCHIVE_MANIFEST);
    let projection = project(&manifest, LINUX_TARGET).unwrap();
    let expected = parse_archive_projection();
    assert!(expected.extraction_required);
    check_archive_against_fixture(&projection, &expected, &manifest).unwrap();

    let (artifact_name, exact_size, _, _) = match &projection {
        ManifestProjection::Archive {
            artifact_name,
            exact_size,
            ..
        } => (artifact_name.clone(), *exact_size, (), ()),
        _ => panic!("expected archive projection"),
    };
    assert_eq!(artifact_name, expected.acquisition_units[0].name);
    assert_eq!(exact_size, expected.acquisition_units[0].exact_size);

    // Archive request binding accepts exactly one archive request and tightens it.
    let request = AcquisitionRequest::new("https://example.test/egress/archive").unwrap();
    let bound = projection
        .bind_requests(
            HashMap::from([(artifact_name, request)]),
            FetchLimits::default(),
        )
        .unwrap();
    assert_eq!(bound.len(), 1);
    assert_eq!(bound[0].limits.max_artifact_bytes, exact_size);

    // Materialization through the generic entry point stays blocked.
    let tmp = TempDir::new("archive-blocked");
    let path = write_sized_file(tmp.path(), "archive", exact_size);
    let acquired = HashMap::from([(bound[0].artifact_name.clone(), path)]);
    let permissions = HashMap::from([(bound[0].member_id.clone(), PermissionsIntent::Preserve)]);
    assert!(matches!(
        projection.materialize_artifact_set(acquired, permissions),
        Err(AdapterError::ArchiveExtractionRequired)
    ));
}

// ---------------------------------------------------------------------------
// Negative matrix (plan section 7, cases 1-20).
// ---------------------------------------------------------------------------

#[test]
fn negative_01_alias_target_is_rejected() {
    let manifest = parse_manifest(DIRECT_MANIFEST);
    assert!(matches!(
        project(&manifest, ALIAS_TARGET),
        Err(AdapterError::TargetNotFound)
    ));
}

#[test]
fn negative_02_unknown_canonical_target_is_rejected() {
    let manifest = parse_manifest(DIRECT_MANIFEST);
    // Unknown canonical triple taken from the checked-in wrong-target fixture.
    let value: serde_json::Value = serde_json::from_str(WRONG_TARGET_JSON).unwrap();
    let target = value["selected_target"].as_str().unwrap();
    assert!(matches!(
        project(&manifest, target),
        Err(AdapterError::TargetNotFound)
    ));
    // Bundle manifest covers only the Linux target; macOS is unknown there.
    let bundle = parse_manifest(BUNDLE_MANIFEST);
    assert!(matches!(
        project(&bundle, "aarch64-apple-darwin"),
        Err(AdapterError::TargetNotFound)
    ));
}

#[test]
fn negative_03_unknown_schema_cannot_project() {
    // Strict parsing rejects the unknown schema version.
    assert!(ReleaseManifest::from_json(UNKNOWN_SCHEMA_JSON).is_err());
    // Even structurally parsed, projection must fail via validation.
    let raw: ReleaseManifest = serde_json::from_str(UNKNOWN_SCHEMA_JSON).unwrap();
    assert!(matches!(
        project(&raw, LINUX_TARGET),
        Err(AdapterError::InvalidManifest)
    ));
}

#[test]
fn negative_04_missing_acquisition_request_fails() {
    let manifest = parse_manifest(BUNDLE_MANIFEST);
    let projection = project(&manifest, LINUX_TARGET).unwrap();
    let (_, _, artifacts) = installable_parts(&projection);
    let mut requests = exact_requests(
        &artifacts
            .iter()
            .map(|a| a.artifact_name.clone())
            .collect::<Vec<_>>(),
        "https://example.test/codegg",
    );
    let drop_key = requests.keys().next().cloned().unwrap();
    requests.remove(&drop_key);
    assert!(matches!(
        projection.bind_requests(requests, FetchLimits::default()),
        Err(AdapterError::MapMismatch(_))
    ));
}

#[test]
fn negative_05_extra_acquisition_request_fails() {
    let manifest = parse_manifest(DIRECT_MANIFEST);
    let projection = project(&manifest, LINUX_TARGET).unwrap();
    let (_, _, artifacts) = installable_parts(&projection);
    let name = artifacts[0].artifact_name.clone();
    let mut requests = HashMap::from([(
        name,
        AcquisitionRequest::new("https://example.test/eggsact").unwrap(),
    )]);
    requests.insert(
        "extra-artifact".to_string(),
        AcquisitionRequest::new("https://example.test/extra").unwrap(),
    );
    let err = projection
        .bind_requests(requests, FetchLimits::default())
        .unwrap_err();
    assert!(matches!(err, AdapterError::MapMismatch(_)));
    // Diagnostics must not leak credential-bearing URL text.
    assert!(!format!("{err}").contains("example.test"));
}

#[test]
fn negative_06_caller_limit_below_exact_size_fails() {
    let manifest = parse_manifest(DIRECT_MANIFEST);
    let projection = project(&manifest, LINUX_TARGET).unwrap();
    let (_, _, artifacts) = installable_parts(&projection);
    let name = artifacts[0].artifact_name.clone();
    let exact = artifacts[0].exact_size;
    let too_small = FetchLimits {
        max_artifact_bytes: exact - 1,
        ..FetchLimits::default()
    };
    assert!(matches!(
        projection.bind_requests(
            HashMap::from([(
                name,
                AcquisitionRequest::new("https://example.test/eggsact").unwrap()
            )]),
            too_small
        ),
        Err(AdapterError::CallerLimitTooSmall(_))
    ));
}

#[test]
fn negative_07_timeouts_and_metadata_bounds_are_never_widened() {
    let manifest = parse_manifest(DIRECT_MANIFEST);
    let projection = project(&manifest, LINUX_TARGET).unwrap();
    let (_, _, artifacts) = installable_parts(&projection);
    let name = artifacts[0].artifact_name.clone();
    let exact = artifacts[0].exact_size;
    let baseline = FetchLimits {
        max_metadata_bytes: 4096,
        max_artifact_bytes: 1024 * 1024,
        connect_timeout: Duration::from_secs(3),
        total_timeout: Duration::from_secs(30),
    };
    let bound = projection
        .bind_requests(
            HashMap::from([(
                name,
                AcquisitionRequest::new("https://example.test/eggsact").unwrap(),
            )]),
            baseline,
        )
        .unwrap();
    assert_eq!(bound[0].limits.max_metadata_bytes, 4096);
    assert_eq!(bound[0].limits.connect_timeout, Duration::from_secs(3));
    assert_eq!(bound[0].limits.total_timeout, Duration::from_secs(30));
    // Unbounded caller cap is tightened to the exact size, never left open.
    assert_eq!(bound[0].limits.max_artifact_bytes, exact);
}

#[test]
fn negative_08_missing_acquired_bundle_artifact_fails() {
    let manifest = parse_manifest(BUNDLE_MANIFEST);
    let projection = project(&manifest, LINUX_TARGET).unwrap();
    let (_, _, artifacts) = installable_parts(&projection);
    let tmp = TempDir::new("neg08");
    let mut acquired = HashMap::new();
    for req in &artifacts[..2] {
        acquired.insert(
            req.artifact_name.clone(),
            write_sized_file(tmp.path(), &req.artifact_name, req.exact_size),
        );
    }
    let permissions = permissions_map(&projection, PermissionsIntent::Preserve);
    assert!(matches!(
        projection.materialize_artifact_set(acquired, permissions),
        Err(AdapterError::MapMismatch(_))
    ));
}

#[test]
fn negative_09_extra_acquired_artifact_fails() {
    let manifest = parse_manifest(DIRECT_MANIFEST);
    let projection = project(&manifest, LINUX_TARGET).unwrap();
    let (_, _, artifacts) = installable_parts(&projection);
    let req = &artifacts[0];
    let tmp = TempDir::new("neg09");
    let mut acquired = HashMap::from([(
        req.artifact_name.clone(),
        write_sized_file(tmp.path(), "artifact", req.exact_size),
    )]);
    acquired.insert(
        "extra".to_string(),
        write_sized_file(tmp.path(), "extra", 3),
    );
    let permissions = HashMap::from([(req.member_id.clone(), PermissionsIntent::Preserve)]);
    assert!(matches!(
        projection.materialize_artifact_set(acquired, permissions),
        Err(AdapterError::MapMismatch(_))
    ));
}

#[test]
fn negative_10_missing_permission_entry_fails() {
    let manifest = parse_manifest(DIRECT_MANIFEST);
    let projection = project(&manifest, LINUX_TARGET).unwrap();
    let (_, _, artifacts) = installable_parts(&projection);
    let req = &artifacts[0];
    let tmp = TempDir::new("neg10");
    let acquired = HashMap::from([(
        req.artifact_name.clone(),
        write_sized_file(tmp.path(), "artifact", req.exact_size),
    )]);
    assert!(matches!(
        projection.materialize_artifact_set(acquired, HashMap::new()),
        Err(AdapterError::MapMismatch(_))
    ));
}

#[test]
fn negative_11_extra_permission_entry_fails() {
    let manifest = parse_manifest(DIRECT_MANIFEST);
    let projection = project(&manifest, LINUX_TARGET).unwrap();
    let (_, _, artifacts) = installable_parts(&projection);
    let req = &artifacts[0];
    let tmp = TempDir::new("neg11");
    let acquired = HashMap::from([(
        req.artifact_name.clone(),
        write_sized_file(tmp.path(), "artifact", req.exact_size),
    )]);
    let mut permissions = HashMap::from([(req.member_id.clone(), PermissionsIntent::Preserve)]);
    permissions.insert(
        MemberId::new("extra-member").unwrap(),
        PermissionsIntent::Preserve,
    );
    assert!(matches!(
        projection.materialize_artifact_set(acquired, permissions),
        Err(AdapterError::MapMismatch(_))
    ));
}

#[test]
fn negative_12_relative_acquired_path_is_rejected() {
    let manifest = parse_manifest(DIRECT_MANIFEST);
    let projection = project(&manifest, LINUX_TARGET).unwrap();
    let (_, _, artifacts) = installable_parts(&projection);
    let req = &artifacts[0];
    let acquired = HashMap::from([(
        req.artifact_name.clone(),
        PathBuf::from("relative/artifact"),
    )]);
    let permissions = HashMap::from([(req.member_id.clone(), PermissionsIntent::Preserve)]);
    assert!(matches!(
        projection.materialize_artifact_set(acquired, permissions),
        Err(AdapterError::InvalidAcquiredFile(_))
    ));
}

#[test]
fn negative_13_symlink_acquired_path_is_rejected() {
    let manifest = parse_manifest(DIRECT_MANIFEST);
    let projection = project(&manifest, LINUX_TARGET).unwrap();
    let (_, _, artifacts) = installable_parts(&projection);
    let req = &artifacts[0];
    let tmp = TempDir::new("neg13");
    let target = write_sized_file(tmp.path(), "real", req.exact_size);
    let link = tmp.path().join("link");
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&target, &link).unwrap();
    }
    #[cfg(windows)]
    {
        if std::os::windows::fs::symlink_file(&target, &link).is_err() {
            eprintln!("SKIP: symlink creation is privilege-constrained on this Windows host");
            return;
        }
    }
    let acquired = HashMap::from([(req.artifact_name.clone(), link)]);
    let permissions = HashMap::from([(req.member_id.clone(), PermissionsIntent::Preserve)]);
    assert!(matches!(
        projection.materialize_artifact_set(acquired, permissions),
        Err(AdapterError::InvalidAcquiredFile(_))
    ));
}

#[test]
fn negative_14_directory_acquired_path_is_rejected() {
    let manifest = parse_manifest(DIRECT_MANIFEST);
    let projection = project(&manifest, LINUX_TARGET).unwrap();
    let (_, _, artifacts) = installable_parts(&projection);
    let req = &artifacts[0];
    let tmp = TempDir::new("neg14");
    let dir = tmp.path().join("subdir");
    fs::create_dir_all(&dir).unwrap();
    let acquired = HashMap::from([(req.artifact_name.clone(), dir)]);
    let permissions = HashMap::from([(req.member_id.clone(), PermissionsIntent::Preserve)]);
    assert!(matches!(
        projection.materialize_artifact_set(acquired, permissions),
        Err(AdapterError::InvalidAcquiredFile(_))
    ));
}

#[test]
fn negative_15_missing_local_file_is_rejected() {
    let manifest = parse_manifest(DIRECT_MANIFEST);
    let projection = project(&manifest, LINUX_TARGET).unwrap();
    let (_, _, artifacts) = installable_parts(&projection);
    let req = &artifacts[0];
    let tmp = TempDir::new("neg15");
    let missing = tmp.path().join("does-not-exist");
    let acquired = HashMap::from([(req.artifact_name.clone(), missing)]);
    let permissions = HashMap::from([(req.member_id.clone(), PermissionsIntent::Preserve)]);
    assert!(matches!(
        projection.materialize_artifact_set(acquired, permissions),
        Err(AdapterError::InvalidAcquiredFile(_))
    ));
}

#[test]
fn negative_16_wrong_exact_size_is_rejected() {
    let manifest = parse_manifest(DIRECT_MANIFEST);
    let projection = project(&manifest, LINUX_TARGET).unwrap();
    let (_, _, artifacts) = installable_parts(&projection);
    let req = &artifacts[0];
    let tmp = TempDir::new("neg16");
    let path = write_sized_file(tmp.path(), "artifact", req.exact_size + 1);
    let acquired = HashMap::from([(req.artifact_name.clone(), path)]);
    let permissions = HashMap::from([(req.member_id.clone(), PermissionsIntent::Preserve)]);
    assert!(matches!(
        projection.materialize_artifact_set(acquired, permissions),
        Err(AdapterError::InvalidAcquiredFile(_))
    ));
}

#[test]
fn negative_17_archive_materialization_requires_extraction() {
    let manifest = parse_manifest(ARCHIVE_MANIFEST);
    let projection = project(&manifest, LINUX_TARGET).unwrap();
    assert!(matches!(projection, ManifestProjection::Archive { .. }));
    let tmp = TempDir::new("neg17");
    let acquired = HashMap::from([(
        "egress-3.1.0-x86_64-unknown-linux-gnu.tar.gz".to_string(),
        write_sized_file(tmp.path(), "archive", 3),
    )]);
    let permissions = HashMap::from([(
        MemberId::new("egress").unwrap(),
        PermissionsIntent::Preserve,
    )]);
    assert!(matches!(
        projection.materialize_artifact_set(acquired, permissions),
        Err(AdapterError::ArchiveExtractionRequired)
    ));
}

#[test]
fn negative_18_bundle_cardinality_must_be_three() {
    let manifest = parse_manifest(BUNDLE_MANIFEST);
    let projection = project(&manifest, LINUX_TARGET).unwrap();
    let (_, _, artifacts) = installable_parts(&projection);
    assert_eq!(artifacts.len(), 3);
    let expected = parse_bundle_projection();
    assert_eq!(expected.acquisition_units.len(), 3);
    check_bundle_against_fixture(&projection, &expected, &manifest).unwrap();
}

#[test]
fn negative_19_substituted_eggsact_member_fails_comparison() {
    let manifest = parse_manifest(BUNDLE_MANIFEST);
    let projection = project(&manifest, LINUX_TARGET).unwrap();
    let mut mutated = parse_bundle_projection();
    let entry = mutated
        .acquisition_units
        .iter_mut()
        .find(|u| u.name == "codegg-helper-2.4.0-x86_64-unknown-linux-gnu")
        .expect("helper unit exists");
    entry.name = "eggsact-2.4.0-x86_64-unknown-linux-gnu".to_string();
    entry.member_id = "eggsact".to_string();
    entry.relative_destination = "eggsact".to_string();
    assert!(
        check_bundle_against_fixture(&projection, &mutated, &manifest).is_err(),
        "substituting eggsact for the helper must fail comparison"
    );
}

#[test]
fn negative_20_crossed_bundle_relationship_fails_comparison() {
    let manifest = parse_manifest(BUNDLE_MANIFEST);
    let projection = project(&manifest, LINUX_TARGET).unwrap();
    let mut mutated = parse_bundle_projection();
    // Cross member identities while keeping artifact names: the pairing breaks.
    let names: Vec<String> = mutated
        .acquisition_units
        .iter()
        .map(|u| u.member_id.clone())
        .collect();
    assert!(names.len() == 3);
    mutated.acquisition_units[0].member_id = names[1].clone();
    mutated.acquisition_units[0].relative_destination = names[1].clone();
    mutated.acquisition_units[1].member_id = names[0].clone();
    mutated.acquisition_units[1].relative_destination = names[0].clone();
    assert!(
        check_bundle_against_fixture(&projection, &mutated, &manifest).is_err(),
        "crossed bundle destination/member relationship must fail comparison"
    );
}

// ---------------------------------------------------------------------------
// Dependency-direction and authority qualification (automated supplement to
// code review and closure-time `cargo tree` evidence).
// ---------------------------------------------------------------------------

#[test]
fn adapter_pins_immutable_eggpack_manifest_and_no_producer_crates() {
    let cargo_toml = include_str!("../Cargo.toml");
    assert!(
        cargo_toml.contains(PINNED_EGGPACK_REV),
        "eggpack-manifest must stay pinned to {PINNED_EGGPACK_REV}"
    );
    for producer in ["eggpack-core", "eggpack-contract", "eggpack-bootstrap"] {
        assert!(
            !cargo_toml.contains(producer),
            "adapter must not depend on {producer}"
        );
    }
}

#[test]
fn adapter_source_claims_no_producer_or_service_authority() {
    let source = include_str!("../src/lib.rs");
    // Authority review concerns production code; the inline `#[cfg(test)]`
    // module legitimately uses `std::process::id` for temp-dir naming.
    let production = source.split("#[cfg(test)]").next().unwrap_or(source);
    // High-signal producer/service authority tokens with no legitimate use in
    // the adapter. Absence here supplements code review; it is not the sole
    // evidence (closure also records `cargo tree` and manual review).
    for token in [
        "api.github.com",
        "reqwest",
        "hyper",
        "tokio",
        "mirror",
        "fallback",
        "latest",
        "base_url",
        "base-origin",
        "install_root",
        "installation_root",
        "ownership",
        "chown",
        "chmod",
        "sudo",
        "elevation",
        "signature",
        "authenticity",
        "systemd",
        "launchd",
        "windows-service",
        "decompress",
        "flate2",
        "std::process",
    ] {
        assert!(
            !production.contains(token),
            "adapter source must not claim authority: {token}"
        );
    }
}
