use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::domain::MemberId;
use crate::error::{Error, Result};
use crate::lock::MutationLock;
use crate::stage::PreparedTransaction;

static NEXT_BACKUP_NONCE: AtomicU64 = AtomicU64::new(0);

/// The terminal disposition of a commit attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionDisposition {
    /// Every member was installed and the live set is coherent.
    Committed,
    /// The attempt failed and all old members were restored and verified.
    RolledBack,
    /// The live state or cleanup could not be safely restored automatically.
    RecoveryRequired,
}

/// The cleanup policy represented by a transaction receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostCommitFailurePolicy {
    /// Owned temporary state was cleaned up.
    Cleaned,
    /// Evidence was retained for an operator because cleanup was unsafe or failed.
    RetainForRecovery,
}

/// Evidence describing the terminal result of a transaction attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionReceipt {
    product: crate::domain::ProductId,
    release: crate::domain::ReleaseId,
    disposition: TransactionDisposition,
    rollback_performed: bool,
    rollback_verified: bool,
    cleanup: PostCommitFailurePolicy,
    recovery_path: Option<PathBuf>,
}

impl TransactionReceipt {
    /// Returns the product identity recorded by the transaction.
    pub fn product(&self) -> &crate::domain::ProductId {
        &self.product
    }

    /// Returns the release identity recorded by the transaction.
    pub fn release(&self) -> &crate::domain::ReleaseId {
        &self.release
    }

    /// Returns the terminal disposition.
    pub fn disposition(&self) -> TransactionDisposition {
        self.disposition
    }

    /// Returns whether rollback was attempted.
    pub fn rollback_performed(&self) -> bool {
        self.rollback_performed
    }

    /// Returns whether every restoration check passed.
    pub fn rollback_verified(&self) -> bool {
        self.rollback_verified
    }

    /// Returns the post-commit cleanup result.
    pub fn cleanup(&self) -> PostCommitFailurePolicy {
        self.cleanup
    }

    /// Returns retained backup/lock evidence when manual recovery is required.
    pub fn recovery_path(&self) -> Option<&Path> {
        self.recovery_path.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum CommitFault {
    LockCreation,
    Backup(MemberId),
    BeforeFirstCommit,
    Commit(MemberId),
    CommitThenRollback(MemberId, MemberId),
    Finalize,
}

#[derive(Debug)]
struct BackupEntry {
    member: MemberId,
    destination: PathBuf,
    backup: Option<PathBuf>,
}

impl PreparedTransaction {
    /// Commits the prepared artifact set synchronously with lock, backup, and rollback handling.
    pub(crate) fn commit(self) -> Result<TransactionReceipt> {
        self.commit_inner(None)
    }

    #[cfg(test)]
    pub(crate) fn commit_with_fault(self, fault: CommitFault) -> Result<TransactionReceipt> {
        self.commit_inner(Some(fault))
    }

    fn commit_inner(self, fault: Option<CommitFault>) -> Result<TransactionReceipt> {
        if fault == Some(CommitFault::LockCreation) {
            return Err(Error::invalid("injected lock creation failure"));
        }
        let mut lock = MutationLock::acquire(
            self.plan.installation_root(),
            self.plan.product(),
            self.plan.release(),
        )?;
        let backup_root = create_backup_directory(self.plan.installation_root())?;
        let mut entries = Vec::with_capacity(self.plan.artifacts().len());
        if let Err(error) = self.backup_members(&mut entries, &backup_root, fault.clone()) {
            let rollback = restore_entries(&entries, &HashSet::new(), fault.clone());
            return finish_failure(self, lock, backup_root, rollback, error);
        }
        if fault == Some(CommitFault::BeforeFirstCommit) {
            let rollback = restore_entries(&entries, &HashSet::new(), fault.clone());
            return finish_failure(
                self,
                lock,
                backup_root,
                rollback,
                Error::invalid("injected pre-commit failure"),
            );
        }

        let mut committed = HashSet::new();
        let members = self.plan.artifacts().iter().cloned().collect::<Vec<_>>();
        for member in members {
            let commit_should_fail = fault.as_ref().is_some_and(|candidate| match candidate {
                CommitFault::Commit(id) | CommitFault::CommitThenRollback(id, _) => {
                    id == member.id()
                }
                _ => false,
            });
            if commit_should_fail {
                let rollback = restore_entries(&entries, &committed, fault.clone());
                return finish_failure(
                    self,
                    lock,
                    backup_root,
                    rollback,
                    Error::invalid(format!("injected commit failure at {}", member.id())),
                );
            }
            let destination = match self.plan.destination(member.id()) {
                Ok(destination) => destination,
                Err(error) => {
                    let rollback = restore_entries(&entries, &committed, fault.clone());
                    return finish_failure(self, lock, backup_root, rollback, error);
                }
            };
            if let Err(error) =
                ensure_destination_parent(&destination, self.plan.installation_root())
            {
                let rollback = restore_entries(&entries, &committed, fault.clone());
                return finish_failure(self, lock, backup_root, rollback, error);
            }
            let staged = match self.staged_path(member.id()) {
                Ok(staged) => staged,
                Err(error) => {
                    let rollback = restore_entries(&entries, &committed, fault.clone());
                    return finish_failure(self, lock, backup_root, rollback, error);
                }
            };
            if let Err(source) = fs::rename(&staged, &destination) {
                let error = Error::io("replacing destination from prepared stage", source);
                let rollback = restore_entries(&entries, &committed, fault.clone());
                return finish_failure(self, lock, backup_root, rollback, error);
            }
            committed.insert(member.id().clone());
        }

        if fault == Some(CommitFault::Finalize) {
            lock.preserve();
            return Ok(TransactionReceipt {
                product: self.plan.product().clone(),
                release: self.plan.release().clone(),
                disposition: TransactionDisposition::Committed,
                rollback_performed: false,
                rollback_verified: false,
                cleanup: PostCommitFailurePolicy::RetainForRecovery,
                recovery_path: Some(backup_root),
            });
        }
        if let Err(source) = fs::remove_dir_all(&backup_root) {
            lock.preserve();
            return Ok(TransactionReceipt {
                product: self.plan.product().clone(),
                release: self.plan.release().clone(),
                disposition: TransactionDisposition::Committed,
                rollback_performed: false,
                rollback_verified: false,
                cleanup: PostCommitFailurePolicy::RetainForRecovery,
                recovery_path: Some(backup_root.join(format!("cleanup-failed-{source}"))),
            });
        }
        Ok(TransactionReceipt {
            product: self.plan.product().clone(),
            release: self.plan.release().clone(),
            disposition: TransactionDisposition::Committed,
            rollback_performed: false,
            rollback_verified: false,
            cleanup: PostCommitFailurePolicy::Cleaned,
            recovery_path: None,
        })
    }

    fn backup_members(
        &self,
        entries: &mut Vec<BackupEntry>,
        backup_root: &Path,
        fault: Option<CommitFault>,
    ) -> Result<()> {
        let members = self.plan.artifacts().iter().cloned().collect::<Vec<_>>();
        for member in members {
            if fault == Some(CommitFault::Backup(member.id().clone())) {
                return Err(Error::invalid(format!(
                    "injected backup failure at {}",
                    member.id()
                )));
            }
            let destination = self.plan.destination(member.id())?;
            revalidate_destination(&destination, self.plan.installation_root())?;
            let backup = match fs::symlink_metadata(&destination) {
                Ok(_) => {
                    let path = backup_root.join(member.destination());
                    if let Some(parent) = path.parent() {
                        fs::create_dir_all(parent)
                            .map_err(|source| Error::io("creating backup directory", source))?;
                    }
                    fs::rename(&destination, &path)
                        .map_err(|source| Error::io("backing up existing destination", source))?;
                    Some(path)
                }
                Err(source) if source.kind() == std::io::ErrorKind::NotFound => None,
                Err(source) => return Err(Error::io("reading destination before backup", source)),
            };
            entries.push(BackupEntry {
                member: member.id().clone(),
                destination,
                backup,
            });
        }
        Ok(())
    }
}

fn create_backup_directory(root: &Path) -> Result<PathBuf> {
    let nonce = NEXT_BACKUP_NONCE.fetch_add(1, Ordering::Relaxed);
    let path = root.join(format!(".eggup-backup-{}-{nonce}", std::process::id()));
    fs::create_dir(&path).map_err(|source| Error::io("creating backup set", source))?;
    Ok(path)
}

fn revalidate_destination(destination: &Path, root: &Path) -> Result<()> {
    let root = fs::canonicalize(root)
        .map_err(|source| Error::io("canonicalizing install root", source))?;
    let mut ancestor = destination;
    while !ancestor.exists() {
        ancestor = ancestor
            .parent()
            .ok_or_else(|| Error::invalid("destination has no existing ancestor"))?;
    }
    let canonical_ancestor = fs::canonicalize(ancestor)
        .map_err(|source| Error::io("canonicalizing destination ancestor", source))?;
    if !canonical_ancestor.starts_with(&root) {
        return Err(Error::DestinationConflict {
            destination: destination.to_path_buf(),
        });
    }
    match fs::symlink_metadata(destination) {
        Ok(metadata) => {
            if !metadata.is_file() {
                return Err(Error::DestinationConflict {
                    destination: destination.to_path_buf(),
                });
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                if metadata.nlink() != 1 {
                    return Err(Error::DestinationConflict {
                        destination: destination.to_path_buf(),
                    });
                }
            }
        }
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {}
        Err(source) => return Err(Error::io("reading destination during revalidation", source)),
    }
    Ok(())
}

fn ensure_destination_parent(destination: &Path, root: &Path) -> Result<()> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .map_err(|source| Error::io("creating destination parent", source))?;
    }
    revalidate_destination(destination, root)
}

fn restore_entries(
    entries: &[BackupEntry],
    committed: &HashSet<MemberId>,
    fault: Option<CommitFault>,
) -> std::result::Result<bool, ()> {
    let mut verified = true;
    for entry in entries.iter().rev() {
        let rollback_target = match fault.as_ref() {
            Some(CommitFault::CommitThenRollback(_, id)) => Some(id),
            _ => None,
        };
        if rollback_target == Some(&entry.member) {
            verified = false;
            continue;
        }
        if entry.backup.is_some() || committed.contains(&entry.member) {
            if let Ok(metadata) = fs::symlink_metadata(&entry.destination) {
                if !metadata.is_file() || fs::remove_file(&entry.destination).is_err() {
                    verified = false;
                    continue;
                }
            }
        }
        if let Some(backup) = &entry.backup {
            if fs::rename(backup, &entry.destination).is_err() {
                verified = false;
            }
        }
    }
    for entry in entries {
        let restored = match &entry.backup {
            Some(_) => fs::symlink_metadata(&entry.destination).is_ok(),
            None => {
                !committed.contains(&entry.member)
                    || fs::symlink_metadata(&entry.destination).is_err()
            }
        };
        if !restored {
            verified = false;
        }
    }
    Ok(verified)
}

fn finish_failure(
    transaction: PreparedTransaction,
    mut lock: MutationLock,
    backup_root: PathBuf,
    rollback: std::result::Result<bool, ()>,
    _error: Error,
) -> Result<TransactionReceipt> {
    let verified = rollback.unwrap_or(false);
    if verified && fs::remove_dir_all(&backup_root).is_ok() {
        return Ok(TransactionReceipt {
            product: transaction.plan.product().clone(),
            release: transaction.plan.release().clone(),
            disposition: TransactionDisposition::RolledBack,
            rollback_performed: true,
            rollback_verified: true,
            cleanup: PostCommitFailurePolicy::Cleaned,
            recovery_path: None,
        });
    }
    lock.preserve();
    Ok(TransactionReceipt {
        product: transaction.plan.product().clone(),
        release: transaction.plan.release().clone(),
        disposition: TransactionDisposition::RecoveryRequired,
        rollback_performed: true,
        rollback_verified: false,
        cleanup: PostCommitFailurePolicy::RetainForRecovery,
        recovery_path: Some(backup_root),
    })
}
