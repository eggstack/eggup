use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

/// The exact executable that is currently running, bound once to a canonical target.
///
/// Binding resolves symlinks, so an invocation through a link updates the real
/// image and never overwrites the link object. It also proves that the target is
/// a single-linked regular file beneath a real directory: a destination whose
/// exact identity cannot be proven is refused rather than replaced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentExecutable {
    target: PathBuf,
    installation_root: PathBuf,
    destination_name: PathBuf,
    #[cfg(unix)]
    device: u64,
    #[cfg(unix)]
    inode: u64,
}

impl CurrentExecutable {
    /// Resolves and binds the running executable from the process environment.
    ///
    /// `std::env::current_exe` is only a starting point: on some platforms it
    /// reports the invocation path, which may traverse symlinks. The result is
    /// always canonicalized and re-proven before it is used for mutation.
    pub fn resolve() -> Result<Self> {
        let reported = std::env::current_exe()
            .map_err(|source| Error::io("resolving current executable", source))?;
        Self::bind(reported)
    }

    /// Binds an already-known executable path, applying the same proof rules as
    /// [`resolve`](Self::resolve).
    ///
    /// This is the seam a child-process fixture uses to address a specific
    /// image; production callers normally use [`resolve`](Self::resolve).
    pub fn bind(executable: impl Into<PathBuf>) -> Result<Self> {
        let reported = executable.into();
        if !reported.is_absolute() {
            return Err(Error::invalid("current executable path must be absolute"));
        }
        // Canonicalization is what makes an invocation symlink safe: the link
        // object is never the destination, only its real target is.
        let target = fs::canonicalize(&reported)
            .map_err(|source| Error::io("canonicalizing current executable", source))?;
        if target.as_os_str().is_empty() {
            return Err(Error::invalid(
                "current executable resolved to an empty path",
            ));
        }
        let metadata = fs::symlink_metadata(&target)
            .map_err(|source| Error::io("reading current executable metadata", source))?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(Error::DestinationConflict {
                destination: target,
            });
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            // A hard-linked image has other names whose contents would silently
            // change with it, so exact identity is not provable and the update
            // is refused.
            if metadata.nlink() != 1 {
                return Err(Error::DestinationConflict {
                    destination: target,
                });
            }
        }
        let installation_root = target
            .parent()
            .ok_or_else(|| Error::invalid("current executable has no parent directory"))?
            .to_path_buf();
        let root_metadata = fs::symlink_metadata(&installation_root)
            .map_err(|source| Error::io("reading executable directory metadata", source))?;
        if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
            return Err(Error::DestinationConflict {
                destination: target,
            });
        }
        let destination_name = PathBuf::from(
            target
                .file_name()
                .ok_or_else(|| Error::invalid("current executable has no file name"))?,
        );
        #[cfg(unix)]
        let (device, inode) = {
            use std::os::unix::fs::MetadataExt;
            (metadata.dev(), metadata.ino())
        };
        Ok(Self {
            target,
            installation_root,
            destination_name,
            #[cfg(unix)]
            device,
            #[cfg(unix)]
            inode,
        })
    }

    /// Returns the canonical live path that will be replaced.
    pub fn target(&self) -> &Path {
        &self.target
    }

    /// Returns the executable's own directory, which is the installation root
    /// used by [`crate::InstallPlan::for_current_executable`].
    ///
    /// The whole self-update transaction stays inside this directory, so no
    /// write authority above it is ever required.
    pub fn installation_root(&self) -> &Path {
        &self.installation_root
    }

    /// Returns the file name used as the single destination member, relative to
    /// [`installation_root`](Self::installation_root).
    pub fn destination_name(&self) -> &Path {
        &self.destination_name
    }

    /// Returns the absolute destination path of the running image.
    pub fn destination(&self) -> PathBuf {
        self.installation_root.join(&self.destination_name)
    }

    /// Re-proves that this exact object is still the bound current executable.
    ///
    /// Called immediately before live mutation under the mutation lock. A
    /// replaced, swapped, or re-linked image fails closed instead of being
    /// overwritten.
    pub(crate) fn revalidate(&self) -> Result<()> {
        let metadata = match fs::symlink_metadata(&self.target) {
            Ok(metadata) => metadata,
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
                return Err(Error::DestinationConflict {
                    destination: self.target.clone(),
                })
            }
            Err(source) => {
                return Err(Error::io(
                    "reading current executable before mutation",
                    source,
                ))
            }
        };
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(Error::DestinationConflict {
                destination: self.target.clone(),
            });
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if metadata.nlink() != 1
                || metadata.dev() != self.device
                || metadata.ino() != self.inode
            {
                return Err(Error::DestinationConflict {
                    destination: self.target.clone(),
                });
            }
        }
        Ok(())
    }
}
