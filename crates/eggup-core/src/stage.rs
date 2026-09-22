use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::domain::{ArtifactMember, InstallPlan, PermissionsIntent};
use crate::error::{Error, Result};

static NEXT_STAGE_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub(crate) struct Stage {
    path: PathBuf,
}

impl Stage {
    pub(crate) fn prepare(plan: InstallPlan) -> Result<PreparedTransaction> {
        Self::prepare_inner(plan, None)
    }

    #[cfg(test)]
    pub(crate) fn prepare_with_failure(
        plan: InstallPlan,
        point: crate::test_support::FailurePoint,
    ) -> Result<PreparedTransaction> {
        let failure = match point {
            crate::test_support::FailurePoint::StageCreate => Some(FailureAt::Create),
            crate::test_support::FailurePoint::StageCopy => Some(FailureAt::Copy),
            _ => None,
        };
        Self::prepare_inner(plan, failure)
    }

    fn prepare_inner(plan: InstallPlan, failure: Option<FailureAt>) -> Result<PreparedTransaction> {
        check_failure(failure, FailureAt::Create)?;
        let path = create_stage_directory()?;
        let stage = Self { path };
        if let Err(error) = stage.copy_members(&plan, failure) {
            drop(stage);
            return Err(error);
        }
        Ok(PreparedTransaction { plan, stage })
    }

    fn copy_members(&self, plan: &InstallPlan, failure: Option<FailureAt>) -> Result<()> {
        for member in plan.artifacts().iter() {
            check_failure(failure, FailureAt::Copy)?;
            let destination = self.member_path(member);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)
                    .map_err(|source| Error::io("creating private stage directory", source))?;
            }
            fs::copy(member.source(), &destination)
                .map_err(|source| Error::io("copying artifact into private stage", source))?;
            apply_permissions(member, &destination)?;
        }
        Ok(())
    }

    fn member_path(&self, member: &ArtifactMember) -> PathBuf {
        self.path.join(member.destination())
    }
}

/// A validated, privately staged transaction that has not performed live mutation.
#[derive(Debug)]
pub struct PreparedTransaction {
    pub(crate) plan: InstallPlan,
    pub(crate) stage: Stage,
}

impl PreparedTransaction {
    /// Returns the product identity associated with this prepared transaction.
    pub fn product(&self) -> &crate::domain::ProductId {
        self.plan.product()
    }

    /// Returns the release identity associated with this prepared transaction.
    pub fn release(&self) -> &crate::domain::ReleaseId {
        self.plan.release()
    }

    /// Returns the validated artifact set.
    pub fn artifacts(&self) -> &crate::domain::ArtifactSet {
        self.plan.artifacts()
    }

    /// Returns the exact live destination for a prepared member.
    pub fn destination(&self, member: &crate::domain::MemberId) -> Result<PathBuf> {
        self.plan.destination(member)
    }

    /// Returns the private staged path for a prepared member.
    pub fn staged_path(&self, member: &crate::domain::MemberId) -> Result<PathBuf> {
        let artifact = self
            .plan
            .artifacts()
            .iter()
            .find(|candidate| candidate.id() == member)
            .ok_or_else(|| Error::UnknownMember(member.to_string()))?;
        Ok(self.stage.member_path(artifact))
    }

    /// Returns the private staging directory for diagnostics and later verification.
    pub fn stage_root(&self) -> &Path {
        &self.stage.path
    }
}

impl Drop for Stage {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn create_stage_directory() -> Result<PathBuf> {
    let sequence = NEXT_STAGE_ID.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("eggup-stage-{}-{sequence}", std::process::id()));
    fs::create_dir(&path).map_err(|source| Error::io("creating private stage", source))?;
    Ok(path)
}

fn apply_permissions(member: &ArtifactMember, destination: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = fs::metadata(destination)
            .map_err(|source| Error::io("reading staged permissions", source))?
            .permissions();
        let mode = permissions.mode();
        let mode = match member.permissions() {
            PermissionsIntent::Preserve => mode,
            PermissionsIntent::Executable => mode | 0o111,
        };
        permissions.set_mode(mode);
        fs::set_permissions(destination, permissions)
            .map_err(|source| Error::io("setting staged permissions", source))?;
    }
    #[cfg(not(unix))]
    let _ = (member, destination);
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FailureAt {
    Create,
    Copy,
}

fn check_failure(configured: Option<FailureAt>, point: FailureAt) -> Result<()> {
    if configured == Some(point) {
        return Err(Error::invalid(format!(
            "injected preparation failure at {point:?}"
        )));
    }
    Ok(())
}
