use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::domain::{ProductId, ReleaseId};
use crate::error::{Error, Result};

static NEXT_LOCK_NONCE: AtomicU64 = AtomicU64::new(0);

/// An exclusive ownership record for one installation domain.
#[derive(Debug)]
pub struct MutationLock {
    path: PathBuf,
    token: String,
    cleanup: bool,
    _file: File,
}

impl MutationLock {
    /// Acquires the installation-root lock using create-new filesystem semantics.
    pub fn acquire(root: &Path, product: &ProductId, release: &ReleaseId) -> Result<Self> {
        let path = root.join(".eggup-mutation.lock");
        let nonce = NEXT_LOCK_NONCE.fetch_add(1, Ordering::Relaxed);
        let token = format!(
            "pid={} nonce={} product={} release={}\n",
            std::process::id(),
            nonce,
            product,
            release
        );
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|source| {
                if source.kind() == std::io::ErrorKind::AlreadyExists {
                    Error::UpdateInProgress { lock: path.clone() }
                } else {
                    Error::io("creating mutation lock", source)
                }
            })?;
        if let Err(source) = file.write_all(token.as_bytes()) {
            let _ = fs::remove_file(&path);
            return Err(Error::io("writing mutation lock", source));
        }
        Ok(Self {
            path,
            token,
            cleanup: true,
            _file: file,
        })
    }

    /// Returns the lock record path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Preserves the lock and record for manual recovery when cleanup is unsafe.
    pub(crate) fn preserve(&mut self) {
        self.cleanup = false;
    }
}

impl Drop for MutationLock {
    fn drop(&mut self) {
        if !self.cleanup {
            return;
        }
        let Ok(metadata) = fs::symlink_metadata(&self.path) else {
            return;
        };
        if !metadata.is_file() {
            return;
        }
        let Ok(contents) = fs::read_to_string(&self.path) else {
            return;
        };
        if contents == self.token {
            let _ = fs::remove_file(&self.path);
        }
    }
}
