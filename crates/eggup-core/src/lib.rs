#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Policy-neutral local mechanics for verified multi-artifact updates."]
#![doc = ""]
#![doc = "The core crate deliberately does not fetch bytes, manage services, or choose release policy."]
#![doc = "The public API exposes validated domain and preparation layers; it does not yet mutate"]
#![doc = "live destinations. Verification and commit layers are added in later milestones."]

mod domain;
mod error;
mod stage;

pub use domain::{
    ArtifactMember, ArtifactSet, AuthenticityRequirement, FileKind, InstallPlan,
    IntegrityRequirement, MemberId, Ownership, PermissionsIntent, ProductId, ReleaseId,
};
pub use error::{Error, Result};
pub use stage::PreparedTransaction;

#[cfg(test)]
mod test_support;

#[cfg(test)]
mod tests {
    use super::test_support::{FailureInjector, FailurePoint, InstallationRoot};
    use super::{ArtifactMember, ArtifactSet, InstallPlan, MemberId, ProductId, ReleaseId};
    use std::fs;

    #[test]
    fn fixture_writes_and_reads_exact_bytes() {
        let fixture = InstallationRoot::new().expect("fixture");
        fixture
            .write_file("bin/eggup", b"exact bytes")
            .expect("write");

        assert_eq!(
            fixture.read_file("bin/eggup").expect("read"),
            b"exact bytes"
        );
        assert!(fixture.path().join("bin/eggup").is_file());
    }

    #[test]
    fn fixture_rejects_escape_paths() {
        let fixture = InstallationRoot::new().expect("fixture");

        let error = fixture.write_file("../outside", b"nope").unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
        assert!(!fixture.path().parent().unwrap().join("outside").exists());
    }

    #[test]
    fn failure_injector_defaults_to_no_failure() {
        let mut injector = FailureInjector::default();

        assert!(injector.check(FailurePoint::Prepare).is_ok());
        assert!(injector.check(FailurePoint::Commit).is_ok());
    }

    #[test]
    fn prepares_a_multi_member_bundle_without_touching_destinations() {
        let inputs = InstallationRoot::new().expect("inputs");
        let install = InstallationRoot::new().expect("install");
        inputs.write_file("egress", b"egress").expect("write");
        inputs.write_file("helper", b"helper").expect("write");
        let product = ProductId::new("egress").expect("product");
        let release = ReleaseId::new("2026.09.22").expect("release");
        let members = ArtifactSet::new(vec![
            ArtifactMember::new(
                MemberId::new("main").expect("member"),
                inputs.path().join("egress"),
                "bin/egress",
            )
            .expect("artifact"),
            ArtifactMember::new(
                MemberId::new("helper").expect("member"),
                inputs.path().join("helper"),
                "bin/helper",
            )
            .expect("artifact"),
        ])
        .expect("members");

        let prepared = InstallPlan::new(product, release, install.path(), members)
            .expect("plan")
            .prepare()
            .expect("prepare");

        assert!(!install.path().join("bin/egress").exists());
        assert_eq!(
            fs::read(
                prepared
                    .staged_path(&MemberId::new("main").unwrap())
                    .unwrap()
            )
            .unwrap(),
            b"egress"
        );
    }

    #[test]
    fn preparation_failure_cleans_private_stage() {
        let inputs = InstallationRoot::new().expect("inputs");
        let install = InstallationRoot::new().expect("install");
        inputs.write_file("eggup", b"bytes").expect("write");
        let plan = InstallPlan::new(
            ProductId::new("eggup").unwrap(),
            ReleaseId::new("r1").unwrap(),
            install.path(),
            ArtifactSet::single(
                ArtifactMember::new(
                    MemberId::new("main").unwrap(),
                    inputs.path().join("eggup"),
                    "eggup",
                )
                .unwrap(),
            )
            .unwrap(),
        )
        .unwrap();

        assert!(plan
            .clone()
            .prepare_with_failure(FailurePoint::StageCreate)
            .is_err());
        let result = plan.prepare_with_failure(FailurePoint::StageCopy);

        assert!(result.is_err());
        assert!(!install.path().join("eggup").exists());
        let stage_prefix = format!(
            ".eggup-stage-{}-",
            install.path().file_name().unwrap().to_string_lossy()
        );
        assert!(install
            .path()
            .parent()
            .unwrap()
            .read_dir()
            .unwrap()
            .filter_map(std::result::Result::ok)
            .all(|entry| !entry
                .file_name()
                .to_string_lossy()
                .starts_with(&stage_prefix)));
    }

    #[test]
    fn rejects_duplicate_normalized_destinations() {
        let inputs = InstallationRoot::new().expect("inputs");
        let install = InstallationRoot::new().expect("install");
        inputs.write_file("one", b"one").expect("write");
        inputs.write_file("two", b"two").expect("write");
        let members = ArtifactSet::new(vec![
            ArtifactMember::new(
                MemberId::new("one").unwrap(),
                inputs.path().join("one"),
                "bin/./app",
            )
            .unwrap(),
            ArtifactMember::new(
                MemberId::new("two").unwrap(),
                inputs.path().join("two"),
                "bin/app",
            )
            .unwrap(),
        ])
        .unwrap();

        let result = InstallPlan::new(
            ProductId::new("eggup").unwrap(),
            ReleaseId::new("r1").unwrap(),
            install.path(),
            members,
        );

        assert!(result.is_err());
    }

    #[test]
    fn rejects_empty_sets_and_escaping_destinations() {
        assert!(ArtifactSet::new(Vec::new()).is_err());
        assert!(ArtifactMember::new(
            MemberId::new("main").unwrap(),
            "/tmp/acquired",
            "../outside",
        )
        .is_err());
        assert!(ArtifactMember::new(
            MemberId::new("main").unwrap(),
            "/tmp/acquired",
            "/absolute/path",
        )
        .is_err());
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_sources_and_preserves_executable_intent_in_stage() {
        use std::os::unix::fs::{symlink, PermissionsExt};

        let inputs = InstallationRoot::new().expect("inputs");
        let install = InstallationRoot::new().expect("install");
        inputs.write_file("real", b"real").expect("write");
        symlink(inputs.path().join("real"), inputs.path().join("link")).expect("symlink");
        let symlink_member = ArtifactMember::new(
            MemberId::new("link").unwrap(),
            inputs.path().join("link"),
            "bin/link",
        )
        .unwrap();
        assert!(InstallPlan::new(
            ProductId::new("eggup").unwrap(),
            ReleaseId::new("r1").unwrap(),
            install.path(),
            ArtifactSet::single(symlink_member).unwrap(),
        )
        .is_err());

        let member = ArtifactMember::new(
            MemberId::new("real").unwrap(),
            inputs.path().join("real"),
            "bin/real",
        )
        .unwrap()
        .with_permissions(super::PermissionsIntent::Executable);
        let prepared = InstallPlan::new(
            ProductId::new("eggup").unwrap(),
            ReleaseId::new("r1").unwrap(),
            install.path(),
            ArtifactSet::single(member).unwrap(),
        )
        .unwrap()
        .prepare()
        .unwrap();
        let mode = fs::metadata(
            prepared
                .staged_path(&MemberId::new("real").unwrap())
                .unwrap(),
        )
        .unwrap()
        .permissions()
        .mode();
        assert_ne!(mode & 0o111, 0);
    }
}
