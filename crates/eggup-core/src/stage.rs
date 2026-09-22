use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::domain::{ArtifactMember, InstallPlan, PermissionsIntent};
use crate::error::{Error, Result};

static NEXT_STAGE_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub(crate) struct Stage {
    path: PathBuf,
    parent: PathBuf,
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
        let (path, parent) = create_stage_directory(plan.installation_root())?;
        let stage = Self { path, parent };
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
                #[cfg(unix)]
                {
                    // Stage subdirectories stay owner-private; intermediate
                    // parents created here are transaction-owned.
                    use std::os::unix::fs::PermissionsExt;
                    let _ = fs::set_permissions(parent, fs::Permissions::from_mode(0o700));
                }
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
        // Only remove what we own: the path must still be a real directory
        // under the recorded parent with the expected transaction prefix, and
        // must not have become a symlink.
        let file_name = self.path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !file_name.starts_with(".eggup-stage-") {
            return;
        }
        let Ok(meta) = fs::symlink_metadata(&self.path) else {
            return;
        };
        if meta.file_type().is_symlink() || !meta.is_dir() {
            return;
        }
        if self.path.parent() != Some(self.parent.as_path()) {
            return;
        }
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn create_stage_directory(installation_root: &Path) -> Result<(PathBuf, PathBuf)> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let parent = installation_root
        .parent()
        .ok_or_else(|| Error::invalid("installation root has no stage parent"))?
        .to_path_buf();
    let root_name = installation_root
        .file_name()
        .ok_or_else(|| Error::invalid("installation root has no name"))?
        .to_string_lossy();
    for _ in 0..32 {
        let sequence = NEXT_STAGE_ID.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = parent.join(format!(
            ".eggup-stage-{root_name}-{}-{sequence}-{nanos}",
            std::process::id()
        ));
        match fs::create_dir(&path) {
            Ok(()) => {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o700));
                }
                return Ok((path, parent));
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(source) => return Err(Error::io("creating private stage", source)),
        }
    }
    Err(Error::invalid("could not create a unique stage directory"))
}

fn apply_permissions(member: &ArtifactMember, destination: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        // Staged files are always owner-private. Explicit executable intent
        // forces 0700. Preserve maps the source executable bit to a private
        // equivalent: executable sources become 0700, others 0600. Broad
        // source modes (group/other read/write) are never inherited.
        let mode = match member.permissions() {
            PermissionsIntent::Executable => 0o700,
            PermissionsIntent::Preserve => {
                let staged_mode = fs::metadata(destination)
                    .map_err(|source| Error::io("reading staged permissions", source))?
                    .permissions()
                    .mode();
                if staged_mode & 0o111 != 0 {
                    0o700
                } else {
                    0o600
                }
            }
        };
        fs::set_permissions(destination, fs::Permissions::from_mode(mode))
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
