//! M003a caller-bound destination policy tests.
//!
//! These tests prove that manifest-provided install identity remains producer
//! evidence while the consuming application binds the exact relative
//! deployment destination authorized by its own installation policy. The
//! manifest default helpers stay behavior-compatible; the caller-bound
//! variants accept an exact `MemberId -> destination` map and fail closed on
//! missing, extra, colliding, or invalid destinations without weakening
//! artifact identity, exact size, SHA-256, permission, or acquired-file
//! exactness.

use eggpack_manifest::ReleaseManifest;
use eggup_core::{IntegrityRequirement, MemberId, PermissionsIntent};
use eggup_eggpack::{core_plan_for_archive, project, AdapterError, ManifestProjection};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const LINUX_TARGET: &str = "x86_64-unknown-linux-gnu";

const DIRECT_MANIFEST: &str = include_str!("fixtures/direct-manifest.json");
const BUNDLE_MANIFEST: &str = include_str!("fixtures/bundle-manifest.json");
const ARCHIVE_MANIFEST: &str = include_str!("fixtures/archive-manifest.json");

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(label: &str) -> Self {
        let id = TEMP_COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "eggup-eggpack-m003a-{label}-{}-{id}",
            std::process::id()
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

fn installable_requirements(
    projection: &ManifestProjection,
) -> Vec<eggup_eggpack::ArtifactRequirement> {
    match projection {
        ManifestProjection::Installable { artifacts, .. } => artifacts.clone(),
        ManifestProjection::Archive { .. } => panic!("expected installable projection"),
    }
}

fn write_sized_file(dir: &Path, name: &str, size: u64) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, vec![b'x'; size as usize]).unwrap();
    path
}

fn direct_projection() -> ManifestProjection {
    let manifest = ReleaseManifest::from_json(DIRECT_MANIFEST).unwrap();
    project(&manifest, LINUX_TARGET).unwrap()
}

fn bundle_projection() -> ManifestProjection {
    let manifest = ReleaseManifest::from_json(BUNDLE_MANIFEST).unwrap();
    project(&manifest, LINUX_TARGET).unwrap()
}

fn archive_projection() -> ManifestProjection {
    let manifest = ReleaseManifest::from_json(ARCHIVE_MANIFEST).unwrap();
    project(&manifest, LINUX_TARGET).unwrap()
}

fn archive_members(projection: &ManifestProjection) -> Vec<(MemberId, String, u64, [u8; 32])> {
    match projection {
        ManifestProjection::Archive { members, .. } => members
            .iter()
            .map(|m| {
                (
                    MemberId::new(m.install.clone()).unwrap(),
                    m.install.clone(),
                    m.exact_size,
                    m.sha256,
                )
            })
            .collect(),
        _ => panic!("expected archive projection"),
    }
}

/// Build the advisory extraction tree the archive plan constructor expects:
/// one regular file per manifest install identity. Contents are arbitrary;
/// plan construction only records advisory paths and member evidence.
fn advisory_extraction_root(members: &[(MemberId, String, u64, [u8; 32])], dir: &Path) -> PathBuf {
    let root = dir.join("extraction");
    fs::create_dir_all(&root).unwrap();
    for (_, install, _, _) in members {
        let path = root.join(install);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, b"advisory").unwrap();
    }
    root
}

fn install_root(dir: &Path) -> PathBuf {
    let root = dir.join("install");
    fs::create_dir_all(&root).unwrap();
    root
}

// ---------------------------------------------------------------------------
// Default-destination helper.
// ---------------------------------------------------------------------------

#[test]
fn default_destinations_match_manifest_install_identity() {
    let direct = direct_projection();
    let defaults = direct.default_destinations().unwrap();
    let reqs = installable_requirements(&direct);
    assert_eq!(defaults.len(), reqs.len());
    for req in &reqs {
        assert_eq!(defaults.get(&req.member_id).unwrap(), &req.destination);
    }

    let bundle = bundle_projection();
    let defaults = bundle.default_destinations().unwrap();
    let reqs = installable_requirements(&bundle);
    assert_eq!(defaults.len(), 3);
    for req in &reqs {
        assert_eq!(defaults.get(&req.member_id).unwrap(), &req.destination);
    }

    let archive = archive_projection();
    let defaults = archive.default_destinations().unwrap();
    let members = archive_members(&archive);
    assert_eq!(defaults.len(), members.len());
    for (id, install, _, _) in &members {
        assert_eq!(defaults.get(id).unwrap(), install);
    }
}

// ---------------------------------------------------------------------------
// Direct/bundle caller-bound materialization.
// ---------------------------------------------------------------------------

#[test]
fn direct_wrapper_matches_prior_manifest_default_behavior() {
    let projection = direct_projection();
    let reqs = installable_requirements(&projection);
    let req = &reqs[0];

    let tmp = TempDir::new("direct-default");
    let path = write_sized_file(tmp.path(), "artifact", req.exact_size);
    let acquired = HashMap::from([(req.artifact_name.clone(), path)]);
    let permissions = HashMap::from([(req.member_id.clone(), PermissionsIntent::Executable)]);

    let via_wrapper = projection
        .materialize_artifact_set(acquired.clone(), permissions.clone())
        .unwrap();
    let via_explicit_defaults = projection
        .materialize_artifact_set_with_destinations(
            acquired,
            projection.default_destinations().unwrap(),
            permissions,
        )
        .unwrap();
    assert_eq!(via_wrapper, via_explicit_defaults);
    let member = via_wrapper.iter().next().unwrap();
    assert_eq!(member.id(), &req.member_id);
    assert_eq!(member.destination().as_os_str(), req.destination.as_str());
    assert_eq!(member.integrity(), IntegrityRequirement::Sha256(req.sha256));
}

#[test]
fn direct_caller_destination_differs_from_manifest_install() {
    let projection = direct_projection();
    let reqs = installable_requirements(&projection);
    let req = &reqs[0];
    assert_eq!(req.destination, "eggsact");

    let tmp = TempDir::new("direct-custom");
    let path = write_sized_file(tmp.path(), "artifact", req.exact_size);
    let acquired = HashMap::from([(req.artifact_name.clone(), path)]);
    let destinations = HashMap::from([(req.member_id.clone(), "renamed-executable".to_string())]);
    let permissions = HashMap::from([(req.member_id.clone(), PermissionsIntent::Executable)]);
    let set = projection
        .materialize_artifact_set_with_destinations(acquired, destinations, permissions)
        .unwrap();
    assert_eq!(set.len(), 1);
    let member = set.iter().next().unwrap();
    // Member identity is unchanged while the destination is caller-bound.
    assert_eq!(member.id(), &req.member_id);
    assert_eq!(member.destination().as_os_str(), "renamed-executable");
    assert_eq!(member.integrity(), IntegrityRequirement::Sha256(req.sha256));
    assert_eq!(member.permissions(), PermissionsIntent::Executable);
}

#[test]
fn bundle_caller_destinations_bind_every_member_independently() {
    let projection = bundle_projection();
    let reqs = installable_requirements(&projection);
    assert_eq!(reqs.len(), 3);

    let tmp = TempDir::new("bundle-custom");
    let mut acquired = HashMap::new();
    let mut destinations = HashMap::new();
    let mut permissions = HashMap::new();
    for (index, req) in reqs.iter().enumerate() {
        let file = write_sized_file(tmp.path(), &format!("artifact-{index}"), req.exact_size);
        acquired.insert(req.artifact_name.clone(), file);
        destinations.insert(req.member_id.clone(), format!("deployed-{index}"));
        permissions.insert(req.member_id.clone(), PermissionsIntent::Preserve);
    }
    let set = projection
        .materialize_artifact_set_with_destinations(acquired, destinations, permissions)
        .unwrap();
    assert_eq!(set.len(), 3);
    for (index, req) in reqs.iter().enumerate() {
        let member = set.iter().find(|m| m.id() == &req.member_id).unwrap();
        assert_eq!(
            member.destination().as_os_str(),
            format!("deployed-{index}").as_str()
        );
        assert_eq!(member.integrity(), IntegrityRequirement::Sha256(req.sha256));
    }
}

#[test]
fn destination_change_preserves_size_gate_and_digest_evidence() {
    let projection = direct_projection();
    let reqs = installable_requirements(&projection);
    let req = &reqs[0];

    // A wrong-size acquired file still fails even with a valid custom
    // destination: the exact-size gate is independent of destination policy.
    let tmp = TempDir::new("size-gate");
    let wrong = write_sized_file(tmp.path(), "wrong", req.exact_size + 1);
    let acquired = HashMap::from([(req.artifact_name.clone(), wrong)]);
    let destinations = HashMap::from([(req.member_id.clone(), "custom-name".to_string())]);
    let permissions = HashMap::from([(req.member_id.clone(), PermissionsIntent::Executable)]);
    assert!(matches!(
        projection.materialize_artifact_set_with_destinations(acquired, destinations, permissions),
        Err(AdapterError::InvalidAcquiredFile(_))
    ));
}

#[test]
fn missing_destination_key_fails_closed() {
    let projection = bundle_projection();
    let reqs = installable_requirements(&projection);

    let tmp = TempDir::new("missing-dest");
    let mut acquired = HashMap::new();
    for req in &reqs {
        acquired.insert(
            req.artifact_name.clone(),
            write_sized_file(tmp.path(), &req.artifact_name, req.exact_size),
        );
    }
    let mut destinations = projection.default_destinations().unwrap();
    let drop_key = destinations.keys().next().cloned().unwrap();
    destinations.remove(&drop_key);
    let permissions: HashMap<MemberId, PermissionsIntent> = reqs
        .iter()
        .map(|r| (r.member_id.clone(), PermissionsIntent::Preserve))
        .collect();
    assert!(matches!(
        projection.materialize_artifact_set_with_destinations(acquired, destinations, permissions),
        Err(AdapterError::MapMismatch(_))
    ));
}

#[test]
fn extra_destination_key_fails_closed() {
    let projection = direct_projection();
    let reqs = installable_requirements(&projection);
    let req = &reqs[0];

    let tmp = TempDir::new("extra-dest");
    let acquired = HashMap::from([(
        req.artifact_name.clone(),
        write_sized_file(tmp.path(), "artifact", req.exact_size),
    )]);
    let mut destinations = projection.default_destinations().unwrap();
    destinations.insert(
        MemberId::new("extra-member").unwrap(),
        "extra-destination".to_string(),
    );
    let permissions = HashMap::from([(req.member_id.clone(), PermissionsIntent::Executable)]);
    let err = projection
        .materialize_artifact_set_with_destinations(acquired, destinations, permissions)
        .unwrap_err();
    assert!(matches!(err, AdapterError::MapMismatch(_)));
    assert!(!format!("{err}").contains("extra-destination"));
}

#[test]
fn colliding_destinations_fail_closed() {
    let projection = bundle_projection();
    let reqs = installable_requirements(&projection);

    let tmp = TempDir::new("colliding-dest");
    let mut acquired = HashMap::new();
    for req in &reqs {
        acquired.insert(
            req.artifact_name.clone(),
            write_sized_file(tmp.path(), &req.artifact_name, req.exact_size),
        );
    }
    // Two distinct members bound to one destination, plus a normalized
    // collision ("./shared" normalizes onto "shared").
    let destinations: HashMap<MemberId, String> = reqs
        .iter()
        .map(|r| (r.member_id.clone(), "shared".to_string()))
        .collect();
    let permissions: HashMap<MemberId, PermissionsIntent> = reqs
        .iter()
        .map(|r| (r.member_id.clone(), PermissionsIntent::Preserve))
        .collect();
    assert!(matches!(
        projection.materialize_artifact_set_with_destinations(
            acquired.clone(),
            destinations,
            permissions.clone()
        ),
        Err(AdapterError::MapMismatch(_))
    ));

    let mut normalized_collision = projection.default_destinations().unwrap();
    let keys: Vec<MemberId> = normalized_collision.keys().cloned().collect();
    normalized_collision.insert(keys[0].clone(), "shared".to_string());
    normalized_collision.insert(keys[1].clone(), "./shared".to_string());
    assert!(matches!(
        projection.materialize_artifact_set_with_destinations(
            acquired,
            normalized_collision,
            permissions
        ),
        Err(AdapterError::MapMismatch(_))
    ));
}

#[test]
fn invalid_destinations_are_rejected_by_core_validation() {
    let projection = direct_projection();
    let reqs = installable_requirements(&projection);
    let req = &reqs[0];

    for bad in [
        "/absolute/destination",
        "../escape-root",
        "",
        "nested/../escape-root",
    ] {
        let tmp = TempDir::new("invalid-dest");
        let acquired = HashMap::from([(
            req.artifact_name.clone(),
            write_sized_file(tmp.path(), "artifact", req.exact_size),
        )]);
        let destinations = HashMap::from([(req.member_id.clone(), bad.to_string())]);
        let permissions = HashMap::from([(req.member_id.clone(), PermissionsIntent::Executable)]);
        let err = projection
            .materialize_artifact_set_with_destinations(acquired, destinations, permissions)
            .unwrap_err();
        assert!(
            matches!(err, AdapterError::Eggup(_)),
            "invalid destination {bad:?} must fail through Eggup validation"
        );
    }
}

#[test]
fn permission_and_acquired_maps_stay_exact_and_independent() {
    let projection = direct_projection();
    let reqs = installable_requirements(&projection);
    let req = &reqs[0];
    let destinations = HashMap::from([(req.member_id.clone(), "custom-name".to_string())]);

    // Complete destinations do not excuse a missing permission entry.
    let tmp = TempDir::new("perm-indep");
    let acquired = HashMap::from([(
        req.artifact_name.clone(),
        write_sized_file(tmp.path(), "artifact", req.exact_size),
    )]);
    assert!(matches!(
        projection.materialize_artifact_set_with_destinations(
            acquired.clone(),
            destinations.clone(),
            HashMap::new()
        ),
        Err(AdapterError::MapMismatch(_))
    ));

    // Complete destinations do not excuse a missing acquired file.
    let permissions = HashMap::from([(req.member_id.clone(), PermissionsIntent::Executable)]);
    assert!(matches!(
        projection.materialize_artifact_set_with_destinations(
            HashMap::new(),
            destinations.clone(),
            permissions.clone()
        ),
        Err(AdapterError::MapMismatch(_))
    ));

    // Acquired files stay keyed by manifest artifact name, not destination.
    let miskeyed = HashMap::from([(
        "custom-name".to_string(),
        acquired.values().next().cloned().unwrap(),
    )]);
    assert!(matches!(
        projection.materialize_artifact_set_with_destinations(miskeyed, destinations, permissions),
        Err(AdapterError::MapMismatch(_))
    ));
}

#[test]
fn archive_projection_rejects_direct_caller_bound_materialization() {
    let projection = archive_projection();
    let tmp = TempDir::new("archive-direct");
    let path = write_sized_file(tmp.path(), "archive", 3);
    let members = archive_members(&projection);
    let (id, install, _, _) = &members[0];
    let acquired = HashMap::from([(install.clone(), path)]);
    let destinations = HashMap::from([(id.clone(), "custom".to_string())]);
    let permissions = HashMap::from([(id.clone(), PermissionsIntent::Preserve)]);
    assert!(matches!(
        projection.materialize_artifact_set_with_destinations(acquired, destinations, permissions),
        Err(AdapterError::ArchiveExtractionRequired)
    ));
    assert!(direct_projection().default_destinations().is_ok());
    assert!(archive_projection().default_destinations().is_ok());
}

// ---------------------------------------------------------------------------
// Archive caller-bound plan construction.
// ---------------------------------------------------------------------------

#[test]
fn archive_wrapper_matches_prior_manifest_default_behavior() {
    let projection = archive_projection();
    let members = archive_members(&projection);
    let tmp = TempDir::new("archive-default");
    let extraction_root = advisory_extraction_root(&members, tmp.path());
    let destination_root = install_root(tmp.path());
    let permissions: HashMap<MemberId, PermissionsIntent> = members
        .iter()
        .map(|(id, _, _, _)| (id.clone(), PermissionsIntent::Preserve))
        .collect();

    let via_wrapper = core_plan_for_archive(
        &projection,
        &extraction_root,
        &destination_root,
        permissions.clone(),
    )
    .unwrap();
    let via_explicit_defaults = eggup_eggpack::core_plan_for_archive_with_destinations(
        &projection,
        &extraction_root,
        &destination_root,
        projection.default_destinations().unwrap(),
        permissions,
    )
    .unwrap();
    assert_eq!(via_wrapper, via_explicit_defaults);
}

#[test]
fn archive_caller_destinations_preserve_source_and_member_evidence() {
    let projection = archive_projection();
    let members = archive_members(&projection);
    assert_eq!(members.len(), 2);

    let tmp = TempDir::new("archive-custom");
    let extraction_root = advisory_extraction_root(&members, tmp.path());
    let destination_root = install_root(tmp.path());

    let destinations: HashMap<MemberId, String> = members
        .iter()
        .map(|(id, _, _, _)| (id.clone(), format!("custom-{id}")))
        .collect();
    let permissions: HashMap<MemberId, PermissionsIntent> = members
        .iter()
        .map(|(id, _, _, _)| (id.clone(), PermissionsIntent::Executable))
        .collect();
    let plan = eggup_eggpack::core_plan_for_archive_with_destinations(
        &projection,
        &extraction_root,
        &destination_root,
        destinations,
        permissions,
    )
    .unwrap();

    for (id, install, exact_size, sha256) in &members {
        let member = plan
            .artifacts()
            .iter()
            .find(|m| m.id() == id)
            .expect("member must be present");
        // Destination is caller-bound.
        assert_eq!(
            member.destination().as_os_str(),
            format!("custom-{id}").as_str()
        );
        assert_eq!(
            plan.destination(id).unwrap(),
            destination_root.join(format!("custom-{id}"))
        );
        // Advisory source path still derives from the manifest relationship.
        assert_eq!(member.source(), &extraction_root.join(install));
        // Member evidence is unchanged.
        assert_eq!(member.integrity(), IntegrityRequirement::Sha256(*sha256));
        let _ = exact_size;
    }
    assert_eq!(plan.installation_root(), destination_root.as_path());
}

#[test]
fn archive_destination_map_failures_fail_closed() {
    let projection = archive_projection();
    let members = archive_members(&projection);
    let tmp = TempDir::new("archive-neg");
    let extraction_root = advisory_extraction_root(&members, tmp.path());
    let destination_root = install_root(tmp.path());
    let permissions: HashMap<MemberId, PermissionsIntent> = members
        .iter()
        .map(|(id, _, _, _)| (id.clone(), PermissionsIntent::Preserve))
        .collect();

    // Missing key.
    let mut missing = projection.default_destinations().unwrap();
    let drop_key = missing.keys().next().cloned().unwrap();
    missing.remove(&drop_key);
    assert!(matches!(
        eggup_eggpack::core_plan_for_archive_with_destinations(
            &projection,
            &extraction_root,
            &destination_root,
            missing,
            permissions.clone()
        ),
        Err(AdapterError::MapMismatch(_))
    ));

    // Extra key.
    let mut extra = projection.default_destinations().unwrap();
    extra.insert(MemberId::new("extra-member").unwrap(), "extra".to_string());
    assert!(matches!(
        eggup_eggpack::core_plan_for_archive_with_destinations(
            &projection,
            &extraction_root,
            &destination_root,
            extra,
            permissions.clone()
        ),
        Err(AdapterError::MapMismatch(_))
    ));

    // Colliding destinations.
    let colliding: HashMap<MemberId, String> = members
        .iter()
        .map(|(id, _, _, _)| (id.clone(), "shared".to_string()))
        .collect();
    assert!(matches!(
        eggup_eggpack::core_plan_for_archive_with_destinations(
            &projection,
            &extraction_root,
            &destination_root,
            colliding,
            permissions.clone()
        ),
        Err(AdapterError::MapMismatch(_))
    ));

    // Invalid destinations fail through Eggup validation.
    for bad in ["/absolute", "../escape"] {
        let invalid: HashMap<MemberId, String> = members
            .iter()
            .map(|(id, _, _, _)| (id.clone(), bad.to_string()))
            .collect();
        assert!(matches!(
            eggup_eggpack::core_plan_for_archive_with_destinations(
                &projection,
                &extraction_root,
                &destination_root,
                invalid,
                permissions.clone()
            ),
            Err(AdapterError::Eggup(_))
        ));
    }

    // Installable projections stay on the extraction-required path.
    assert!(matches!(
        eggup_eggpack::core_plan_for_archive_with_destinations(
            &direct_projection(),
            &extraction_root,
            &destination_root,
            HashMap::new(),
            HashMap::new()
        ),
        Err(AdapterError::ArchiveExtractionRequired)
    ));
}

// ---------------------------------------------------------------------------
// Diagnostic hygiene.
// ---------------------------------------------------------------------------

#[test]
fn destination_diagnostics_carry_no_credential_or_content_material() {
    let projection = direct_projection();
    let reqs = installable_requirements(&projection);
    let req = &reqs[0];
    let tmp = TempDir::new("diag-hygiene");

    // Missing-key failure with a credential-bearing URL present elsewhere in
    // the caller maps must not echo that URL.
    let secret_url = "https://user:super-secret@example.invalid/?token=hidden-value";
    let _ = secret_url;
    let acquired = HashMap::from([(
        req.artifact_name.clone(),
        write_sized_file(tmp.path(), "artifact", req.exact_size),
    )]);
    let permissions = HashMap::from([(req.member_id.clone(), PermissionsIntent::Executable)]);
    let missing_err = projection
        .materialize_artifact_set_with_destinations(acquired, HashMap::new(), permissions)
        .unwrap_err();
    let diagnostic = format!("{missing_err} {missing_err:?}");
    assert!(!diagnostic.contains("super-secret"));
    assert!(!diagnostic.contains("token=hidden-value"));
    assert!(!diagnostic.contains("example.invalid"));

    // Traversal failure carrying a secret-bearing string still reports only
    // the bounded Eggup validation failure.
    let tmp = TempDir::new("diag-hygiene-2");
    let acquired = HashMap::from([(
        req.artifact_name.clone(),
        write_sized_file(tmp.path(), "artifact", req.exact_size),
    )]);
    let traversal = HashMap::from([(
        req.member_id.clone(),
        "../user:super-secret@example.invalid".to_string(),
    )]);
    let permissions = HashMap::from([(req.member_id.clone(), PermissionsIntent::Executable)]);
    let traversal_err = projection
        .materialize_artifact_set_with_destinations(acquired, traversal, permissions)
        .unwrap_err();
    let diagnostic = format!("{traversal_err} {traversal_err:?}");
    assert!(!diagnostic.contains("super-secret"));
    assert!(!diagnostic.contains("example.invalid"));
}
