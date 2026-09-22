use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::domain::{ProductId, ReleaseId};
use crate::error::{Error, Result};

static NEXT_LOCK_NONCE: AtomicU64 = AtomicU64::new(0);

const MAX_LOCK_BYTES: u64 = 4096;

/// The observable state of an installation-domain lock record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LockStatus {
    /// No lock record exists.
    Available,
    /// A lock record exists; its bounded contents are returned for diagnosis.
    /// Eggup never auto-removes this record.
    Held {
        /// The lock path that is held.
        lock: PathBuf,
        /// Bounded lock-record contents, when readable.
        contents: Option<String>,
    },
    /// A lock record exists but cannot be parsed as UTF-8 or is oversized.
    /// Still never auto-removed.
    Malformed {
        /// The lock path that is malformed.
        lock: PathBuf,
    },
}

/// An exclusive ownership record for one installation domain.
///
/// The lock is fail-closed: ambiguous, malformed, oversized, or otherwise
/// unreadable records never trigger automatic removal. There is deliberately
/// no automatic stale-lock recovery based on PID liveness alone, because PID
/// reuse cannot prove process identity across the public contract. Operators
/// establish ownership out-of-band (for example via [`MutationLock::inspect`]
/// plus deployment-specific process evidence) and remove a stale record
/// manually.
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
        if token.len() as u64 > MAX_LOCK_BYTES {
            return Err(Error::invalid("lock record exceeds bound"));
        }
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
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
        }
        Ok(Self {
            path,
            token,
            cleanup: true,
            _file: file,
        })
    }

    /// Inspects the lock record without acquiring or removing it.
    ///
    /// Returns [`LockStatus::Available`] when no record exists,
    /// [`LockStatus::Held`] when a bounded readable record exists, and
    /// [`LockStatus::Malformed`] when the record is missing, unreadable,
    /// oversized, or non-UTF-8. This function never deletes anything.
    pub fn inspect(root: &Path) -> Result<LockStatus> {
        let path = root.join(".eggup-mutation.lock");
        let meta = match fs::symlink_metadata(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(LockStatus::Available);
            }
            Err(source) => return Err(Error::io("reading mutation lock", source)),
            Ok(m) => m,
        };
        if !meta.is_file() {
            return Ok(LockStatus::Malformed { lock: path });
        }
        #[cfg(unix)]
        {
            if meta.file_type().is_symlink() {
                return Ok(LockStatus::Malformed { lock: path });
            }
        }
        if meta.len() > MAX_LOCK_BYTES {
            return Ok(LockStatus::Malformed { lock: path });
        }
        match fs::read_to_string(&path) {
            Ok(contents) if contents.len() as u64 <= MAX_LOCK_BYTES => Ok(LockStatus::Held {
                lock: path,
                contents: Some(contents),
            }),
            _ => Ok(LockStatus::Malformed { lock: path }),
        }
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
