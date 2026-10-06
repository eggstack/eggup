use std::fs::{self, File};
use std::io::{Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(unix)]
use crate::domain::PermissionsIntent;
use crate::domain::{ArtifactMember, BoundSources, InstallPlan, StagePlacement};
use crate::error::{Error, Result};

static NEXT_STAGE_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub(crate) struct Stage {
    path: PathBuf,
    parent: PathBuf,
}

impl Stage {
    pub(crate) fn prepare(plan: InstallPlan) -> Result<PreparedTransaction> {
        Self::prepare_inner(plan, None, None)
    }

    pub(crate) fn prepare_bound(
        plan: InstallPlan,
        bound: BoundSources,
    ) -> Result<PreparedTransaction> {
        Self::prepare_inner(plan, Some(bound), None)
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
        Self::prepare_inner(plan, None, failure)
    }

    fn prepare_inner(
        plan: InstallPlan,
        mut bound: Option<BoundSources>,
        failure: Option<FailureAt>,
    ) -> Result<PreparedTransaction> {
        check_failure(failure, FailureAt::Create)?;
        let (path, parent) =
            create_stage_directory(plan.installation_root(), plan.stage_placement())?;
        let stage = Self { path, parent };
        if let Err(error) = stage.copy_members(&plan, &mut bound, failure) {
            drop(stage);
            return Err(error);
        }
        if bound.is_some_and(|sources| !sources.is_empty()) {
            drop(stage);
            return Err(Error::invalid("bound source for unknown artifact member"));
        }
        Ok(PreparedTransaction { plan, stage })
    }

    fn copy_members(
        &self,
        plan: &InstallPlan,
        bound: &mut Option<BoundSources>,
        failure: Option<FailureAt>,
    ) -> Result<()> {
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
            let handle = bound.as_mut().and_then(|sources| sources.take(member.id()));
            let source_executable = match handle {
                Some(handle) => {
                    // Executable intent is read from the handle's own metadata,
                    // before the fresh owner-private staged file exists. Reading
                    // it back from the staged copy instead would always read back
                    // the 0600 set below and silently drop the source's
                    // executable bit.
                    let executable = bound_handle_is_executable(&handle)?;
                    stage_bound_source(handle, &destination)?;
                    executable
                }
                None => {
                    fs::copy(member.source(), &destination).map_err(|source| {
                        Error::io("copying artifact into private stage", source)
                    })?;
                    // `fs::copy` carries the source mode onto the staged copy,
                    // so the staged file is a faithful stand-in here.
                    staged_is_executable(&destination)?
                }
            };
            apply_permissions(member, &destination, source_executable)?;
        }
        Ok(())
    }

    fn member_path(&self, member: &ArtifactMember) -> PathBuf {
        self.path.join(member.destination())
    }
}

/// Stages one member from its already-open object without any pathname lookup.
///
/// The handle is rewound to byte zero first, so a non-zero cursor (for
/// example, left at end-of-file by the producer) can never truncate staged
/// bytes. Read, rewind, or copy failure fails closed; the recorded source
/// path is never consulted as fallback.
fn stage_bound_source(mut handle: File, destination: &Path) -> Result<()> {
    handle
        .seek(SeekFrom::Start(0))
        .map_err(|source| Error::io("rewinding bound source for staging", source))?;
    let mut staged = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(destination)
        .map_err(|source| Error::io("creating bound stage destination", source))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        // The staged file is created fresh, so it starts out owner-private by
        // construction. This hardcoded 0600 is hardening, not executable
        // intent: the source's executable bit is captured from the handle
        // before this point and re-applied by `apply_permissions`.
        fs::set_permissions(destination, fs::Permissions::from_mode(0o600))
            .map_err(|source| Error::io("securing bound stage destination", source))?;
    }
    std::io::copy(&mut handle, &mut staged)
        .map_err(|source| Error::io("staging bound source into private stage", source))?;
    Ok(())
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

fn create_stage_directory(
    installation_root: &Path,
    placement: StagePlacement,
) -> Result<(PathBuf, PathBuf)> {
    use std::time::{SystemTime, UNIX_EPOCH};
    // `InsideInstallationRoot` deliberately keeps the stage beside the
    // destinations it will be renamed onto. Same-directory placement is what
    // makes the commit a same-filesystem rename and what keeps the whole
    // transaction inside the caller's own directory: a self-updater needs write
    // authority where its executable lives, never in that directory's parent.
    let (parent, root_name) = match placement {
        StagePlacement::SiblingOfInstallationRoot => {
            let parent = installation_root
                .parent()
                .ok_or_else(|| Error::invalid("installation root has no stage parent"))?
                .to_path_buf();
            let root_name = installation_root
                .file_name()
                .ok_or_else(|| Error::invalid("installation root has no name"))?
                .to_string_lossy()
                .into_owned();
            (parent, root_name)
        }
        StagePlacement::InsideInstallationRoot => {
            (installation_root.to_path_buf(), String::from("self"))
        }
    };
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

/// Reads executable intent from an already-open bound source handle itself.
///
/// `File::metadata` is an `fstat` on the handle, so this is unaffected by the
/// staged file and cannot be masked by the stage's own `0600` hardening.
#[cfg(unix)]
fn bound_handle_is_executable(handle: &File) -> Result<bool> {
    use std::os::unix::fs::PermissionsExt;
    handle
        .metadata()
        .map(|meta| meta.permissions().mode() & 0o111 != 0)
        .map_err(|source| Error::io("reading bound source permissions", source))
}

#[cfg(not(unix))]
fn bound_handle_is_executable(_handle: &File) -> Result<bool> {
    Ok(false)
}

/// Reads executable intent from an already-staged member whose mode was
/// carried over from the source by `fs::copy`.
#[cfg(unix)]
fn staged_is_executable(destination: &Path) -> Result<bool> {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(destination)
        .map(|meta| meta.permissions().mode() & 0o111 != 0)
        .map_err(|source| Error::io("reading staged permissions", source))
}

#[cfg(not(unix))]
fn staged_is_executable(_destination: &Path) -> Result<bool> {
    Ok(false)
}

fn apply_permissions(
    member: &ArtifactMember,
    destination: &Path,
    source_executable: bool,
) -> Result<()> {
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
                if source_executable {
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
    let _ = (member, destination, source_executable);
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FailureAt {
    Create,
    Copy,
}

fn check_failure(configured: Option<FailureAt>, point: FailureAt) -> Result<()> {
    // `configured` is only ever `Some` through the test-only
    // `prepare_with_failure` entry point, so in a packaged build the injected
    // branch does not exist at all rather than existing and being unreachable.
    #[cfg(test)]
    if configured == Some(point) {
        return Err(Error::injected(format!(
            "injected preparation failure at {point:?}"
        )));
    }
    #[cfg(not(test))]
    let _ = (configured, point);
    Ok(())
}
