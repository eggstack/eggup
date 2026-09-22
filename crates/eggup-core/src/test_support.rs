use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FailurePoint {
    Prepare,
    Commit,
}

#[derive(Debug, Default)]
pub(crate) struct FailureInjector {
    fail_at: Option<FailurePoint>,
}

impl FailureInjector {
    pub(crate) fn check(&mut self, point: FailurePoint) -> io::Result<()> {
        if self.fail_at == Some(point) {
            return Err(io::Error::other(format!("injected failure at {point:?}")));
        }
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct InstallationRoot {
    path: PathBuf,
}

impl InstallationRoot {
    pub(crate) fn new() -> io::Result<Self> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_nanos();
        let sequence = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "eggup-test-{timestamp}-{sequence}-{}",
            std::process::id()
        ));
        fs::create_dir(&path)?;
        Ok(Self { path })
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn write_file(&self, relative: &str, bytes: &[u8]) -> io::Result<PathBuf> {
        let path = self.safe_path(relative)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, bytes)?;
        Ok(path)
    }

    pub(crate) fn read_file(&self, relative: &str) -> io::Result<Vec<u8>> {
        fs::read(self.safe_path(relative)?)
    }

    fn safe_path(&self, relative: &str) -> io::Result<PathBuf> {
        let path = Path::new(relative);
        if path.is_absolute()
            || path.components().any(|component| {
                matches!(
                    component,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
        {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "fixture path escapes its private root",
            ));
        }
        Ok(self.path.join(path))
    }
}

impl Drop for InstallationRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
