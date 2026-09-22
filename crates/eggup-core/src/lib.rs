#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Policy-neutral local mechanics for verified multi-artifact updates."]
#![doc = ""]
#![doc = "The core crate deliberately does not fetch bytes, manage services, or choose release policy."]
#![doc = "The public API exposes validated domain and preparation layers; it does not yet mutate"]
#![doc = "live destinations. Verification and commit layers are added in later milestones."]

mod domain;
mod error;
mod lock;
mod stage;
mod transaction;

pub use domain::{
    ArtifactMember, ArtifactSet, AuthenticityRequirement, FileKind, InstallPlan,
    IntegrityRequirement, MemberId, Ownership, PermissionsIntent, ProductId, ReleaseId,
};
pub use error::{Error, Result};
pub use lock::MutationLock;
pub use stage::PreparedTransaction;
pub use transaction::{PostCommitFailurePolicy, TransactionDisposition, TransactionReceipt};

#[cfg(test)]
mod test_support;

#[cfg(test)]
mod tests {
    use super::test_support::{FailureInjector, FailurePoint, InstallationRoot};
    use super::transaction::CommitFault;
    use super::{
        ArtifactMember, ArtifactSet, Error, InstallPlan, MemberId, MutationLock, ProductId,
        ReleaseId, TransactionDisposition,
    };
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

    fn prepared_bundle(
        existing: bool,
    ) -> (
        InstallationRoot,
        InstallationRoot,
        super::PreparedTransaction,
    ) {
        let inputs = InstallationRoot::new().expect("inputs");
        let install = InstallationRoot::new().expect("install");
        inputs.write_file("main", b"new-main").expect("write");
        inputs.write_file("helper", b"new-helper").expect("write");
        if existing {
            install
                .write_file("bin/main", b"old-main")
                .expect("old main");
            install
                .write_file("bin/helper", b"old-helper")
                .expect("old helper");
        }
        let members = ArtifactSet::new(vec![
            ArtifactMember::new(
                MemberId::new("main").unwrap(),
                inputs.path().join("main"),
                "bin/main",
            )
            .unwrap(),
            ArtifactMember::new(
                MemberId::new("helper").unwrap(),
                inputs.path().join("helper"),
                "bin/helper",
            )
            .unwrap(),
        ])
        .unwrap();
        let prepared = InstallPlan::new(
            ProductId::new("bundle").unwrap(),
            ReleaseId::new("r1").unwrap(),
            install.path(),
            members,
        )
        .unwrap()
        .prepare()
        .unwrap();
        (inputs, install, prepared)
    }

    #[test]
    fn commits_a_complete_multi_member_generation_and_releases_lock() {
        let (_inputs, install, prepared) = prepared_bundle(true);

        let receipt = prepared.commit().expect("commit");

        assert_eq!(receipt.disposition(), TransactionDisposition::Committed);
        assert_eq!(receipt.cleanup(), super::PostCommitFailurePolicy::Cleaned);
        assert_eq!(
            fs::read(install.path().join("bin/main")).unwrap(),
            b"new-main"
        );
        assert_eq!(
            fs::read(install.path().join("bin/helper")).unwrap(),
            b"new-helper"
        );
        assert!(!install.path().join(".eggup-mutation.lock").exists());
        assert!(install
            .path()
            .read_dir()
            .unwrap()
            .filter_map(std::result::Result::ok)
            .all(|entry| !entry
                .file_name()
                .to_string_lossy()
                .starts_with(".eggup-backup-")));
    }

    #[test]
    fn partial_commit_failure_restores_every_old_member() {
        let (_inputs, install, prepared) = prepared_bundle(true);

        let receipt = prepared
            .commit_with_fault(CommitFault::Commit(MemberId::new("helper").unwrap()))
            .expect("receipt");

        assert_eq!(receipt.disposition(), TransactionDisposition::RolledBack);
        assert!(receipt.rollback_performed());
        assert!(receipt.rollback_verified());
        assert_eq!(
            fs::read(install.path().join("bin/main")).unwrap(),
            b"old-main"
        );
        assert_eq!(
            fs::read(install.path().join("bin/helper")).unwrap(),
            b"old-helper"
        );
    }

    #[test]
    fn rollback_removes_new_members_that_were_absent_before_commit() {
        let inputs = InstallationRoot::new().unwrap();
        let install = InstallationRoot::new().unwrap();
        inputs.write_file("main", b"new-main").unwrap();
        inputs.write_file("helper", b"new-helper").unwrap();
        install.write_file("bin/main", b"old-main").unwrap();
        let prepared = InstallPlan::new(
            ProductId::new("bundle").unwrap(),
            ReleaseId::new("r1").unwrap(),
            install.path(),
            ArtifactSet::new(vec![
                ArtifactMember::new(
                    MemberId::new("main").unwrap(),
                    inputs.path().join("main"),
                    "bin/main",
                )
                .unwrap(),
                ArtifactMember::new(
                    MemberId::new("helper").unwrap(),
                    inputs.path().join("helper"),
                    "bin/helper",
                )
                .unwrap(),
            ])
            .unwrap(),
        )
        .unwrap()
        .prepare()
        .unwrap();

        let receipt = prepared
            .commit_with_fault(CommitFault::Commit(MemberId::new("helper").unwrap()))
            .unwrap();

        assert_eq!(receipt.disposition(), TransactionDisposition::RolledBack);
        assert_eq!(
            fs::read(install.path().join("bin/main")).unwrap(),
            b"old-main"
        );
        assert!(!install.path().join("bin/helper").exists());
    }

    #[test]
    fn lock_contention_and_malformed_lock_fail_closed() {
        let (_inputs, install, prepared) = prepared_bundle(false);
        let product = ProductId::new("bundle").unwrap();
        let release = ReleaseId::new("r1").unwrap();
        let _lock = MutationLock::acquire(install.path(), &product, &release).unwrap();

        let error = prepared.commit().unwrap_err();
        assert!(matches!(error, Error::UpdateInProgress { .. }));
        drop(_lock);

        fs::write(install.path().join(".eggup-mutation.lock"), b"malformed").unwrap();
        let (_inputs, _install, prepared) = prepared_bundle(false);
        let lock = _install.path().join(".eggup-mutation.lock");
        fs::write(&lock, b"malformed").unwrap();
        let error = prepared.commit().unwrap_err();
        assert!(matches!(error, Error::UpdateInProgress { .. }));
        assert!(lock.exists());
    }

    #[test]
    fn rollback_failure_returns_recovery_required_and_retains_evidence() {
        let (_inputs, install, prepared) = prepared_bundle(true);

        let receipt = prepared
            .commit_with_fault(CommitFault::CommitThenRollback(
                MemberId::new("helper").unwrap(),
                MemberId::new("main").unwrap(),
            ))
            .unwrap();

        assert_eq!(
            receipt.disposition(),
            TransactionDisposition::RecoveryRequired
        );
        assert!(!receipt.rollback_verified());
        assert!(receipt.recovery_path().is_some());
        assert!(install.path().join(".eggup-mutation.lock").exists());
        let recovery = receipt.recovery_path().unwrap();
        assert!(recovery.exists());
        let _ = fs::remove_dir_all(recovery);
        let _ = fs::remove_file(install.path().join(".eggup-mutation.lock"));
    }

    #[test]
    fn precommit_and_backup_failures_restore_old_state() {
        let (_inputs, install, prepared) = prepared_bundle(true);
        let receipt = prepared
            .commit_with_fault(CommitFault::BeforeFirstCommit)
            .unwrap();
        assert_eq!(receipt.disposition(), TransactionDisposition::RolledBack);
        assert_eq!(
            fs::read(install.path().join("bin/main")).unwrap(),
            b"old-main"
        );

        let (_inputs, install, prepared) = prepared_bundle(true);
        let receipt = prepared
            .commit_with_fault(CommitFault::Backup(MemberId::new("helper").unwrap()))
            .unwrap();
        assert_eq!(receipt.disposition(), TransactionDisposition::RolledBack);
        assert_eq!(
            fs::read(install.path().join("bin/helper")).unwrap(),
            b"old-helper"
        );
    }

    #[test]
    fn injected_lock_creation_failure_performs_no_mutation() {
        let (_inputs, install, prepared) = prepared_bundle(true);

        let error = prepared
            .commit_with_fault(CommitFault::LockCreation)
            .unwrap_err();

        assert!(matches!(error, Error::InvalidInput(_)));
        assert_eq!(
            fs::read(install.path().join("bin/main")).unwrap(),
            b"old-main"
        );
        assert!(!install.path().join(".eggup-mutation.lock").exists());
    }

    #[cfg(unix)]
    #[test]
    fn destination_links_fail_immediately_before_backup() {
        use std::fs::hard_link;
        use std::os::unix::fs::symlink;

        let inputs = InstallationRoot::new().unwrap();
        let install = InstallationRoot::new().unwrap();
        inputs.write_file("main", b"new-main").unwrap();
        install.write_file("bin/real", b"old-main").unwrap();
        symlink(install.path().join("real"), install.path().join("bin/main")).unwrap();
        let prepared = InstallPlan::new(
            ProductId::new("bundle").unwrap(),
            ReleaseId::new("r1").unwrap(),
            install.path(),
            ArtifactSet::single(
                ArtifactMember::new(
                    MemberId::new("main").unwrap(),
                    inputs.path().join("main"),
                    "bin/main",
                )
                .unwrap(),
            )
            .unwrap(),
        )
        .unwrap()
        .prepare()
        .unwrap();
        let receipt = prepared.commit().unwrap();
        assert_eq!(receipt.disposition(), TransactionDisposition::RolledBack);
        assert_eq!(
            fs::read(install.path().join("bin/real")).unwrap(),
            b"old-main"
        );

        let (_inputs, install, prepared) = prepared_bundle(true);
        hard_link(
            install.path().join("bin/main"),
            install.path().join("bin/linked"),
        )
        .unwrap();
        let receipt = prepared.commit().unwrap();
        assert_eq!(receipt.disposition(), TransactionDisposition::RolledBack);
        assert_eq!(
            fs::read(install.path().join("bin/main")).unwrap(),
            b"old-main"
        );
    }
}
