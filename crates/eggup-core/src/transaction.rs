use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::domain::{AbsentPolicy, CommitOwnership, MemberId, Ownership};
use crate::error::{Error, Result};
use crate::integrity::hash_file;
use crate::lock::MutationLock;
use crate::stage::PreparedTransaction;

static NEXT_BACKUP_NONCE: AtomicU64 = AtomicU64::new(0);

/// The terminal disposition of a commit attempt.
///
/// A rolled-back or recovery-required result is never equivalent to success;
/// see `examples/receipts.rs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionDisposition {
    /// Every member was installed and the live set is coherent.
    Committed,
    /// The attempt failed and all old members were restored and verified.
    RolledBack,
    /// The live state or cleanup could not be safely restored automatically.
    RecoveryRequired,
}

/// How transaction-owned temporary evidence was handled.
///
/// This replaces the former `PostCommitFailurePolicy::{Cleaned,
/// RetainForRecovery}` name, which conflated cleanup disposition with the
/// ADR-0002 post-commit `KeepInstalled | RollBack` policy. The ADR-0002 policy
/// is reserved for a future post-commit failure boundary and is not
/// implemented here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanupDisposition {
    /// Owned temporary state was cleaned up.
    Cleaned,
    /// Evidence was retained for an operator because cleanup was unsafe or failed.
    RetainedForRecovery,
}

/// The transaction phase that produced a failure.
///
/// Phases are ordered by first occurrence in a commit attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FailurePhase {
    /// Mutation-lock acquisition or inspection.
    Lock,
    /// Destination ownership classification or parent containment.
    Ownership,
    /// Re-reading and re-hashing staged bytes under lock.
    StageRevalidation,
    /// Moving live destinations into the backup set.
    Backup,
    /// Renaming staged members into live destinations.
    Commit,
    /// Restoring backups after a failure.
    Rollback,
    /// Removing the backup set after a successful commit.
    Finalize,
}

impl FailurePhase {
    /// Returns the stable phase name used in receipts and diagnostics.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Lock => "lock",
            Self::Ownership => "ownership",
            Self::StageRevalidation => "stage-revalidation",
            Self::Backup => "backup",
            Self::Commit => "commit",
            Self::Rollback => "rollback",
            Self::Finalize => "finalize",
        }
    }
}

/// A stable machine-readable failure category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FailureCategory {
    /// Another transaction or ambiguous lock owns the domain.
    LockContention,
    /// A destination is foreign, unknown, or violates containment/parent rules.
    OwnershipConflict,
    /// Staged bytes changed, are unreadable, or lack required integrity evidence.
    Verification,
    /// A live filesystem mutation failed.
    Filesystem,
    /// Caller input or transaction configuration is invalid.
    InvalidInput,
    /// The failure was injected by the test fault harness.
    Injected,
}

impl FailureCategory {
    /// Returns the stable category name.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LockContention => "lock-contention",
            Self::OwnershipConflict => "ownership-conflict",
            Self::Verification => "verification",
            Self::Filesystem => "filesystem",
            Self::InvalidInput => "invalid-input",
            Self::Injected => "injected",
        }
    }
}

/// A bounded structured report describing one transaction failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureReport {
    phase: FailurePhase,
    member: Option<MemberId>,
    category: FailureCategory,
    detail: String,
}

impl FailureReport {
    pub(crate) fn new(
        phase: FailurePhase,
        member: Option<MemberId>,
        category: FailureCategory,
        detail: impl Into<String>,
    ) -> Self {
        let mut detail = detail.into();
        // Bound human-readable detail; receipts must never embed unbounded output.
        if detail.len() > 512 {
            detail.truncate(512);
        }
        Self {
            phase,
            member,
            category,
            detail,
        }
    }

    /// Returns the phase that produced the failure.
    pub fn phase(&self) -> FailurePhase {
        self.phase
    }

    /// Returns the affected member, when the failure is member-specific.
    pub fn member(&self) -> Option<&MemberId> {
        self.member.as_ref()
    }

    /// Returns the stable failure category.
    pub fn category(&self) -> FailureCategory {
        self.category
    }

    /// Returns bounded human-readable detail.
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

pub(crate) fn report_for_error(
    phase: FailurePhase,
    member: Option<MemberId>,
    error: &Error,
) -> FailureReport {
    let (category, detail) = match error {
        Error::UpdateInProgress { lock } => (
            FailureCategory::LockContention,
            format!("lock contention at {}", lock.display()),
        ),
        Error::DestinationConflict { destination } => (
            FailureCategory::OwnershipConflict,
            format!("destination conflict at {}", destination.display()),
        ),
        Error::VerificationFailed(m) => (FailureCategory::Verification, m.clone()),
        Error::CandidateExecution(m) => (FailureCategory::Verification, m.clone()),
        Error::InvalidInput(m) => {
            let category = if m.contains("injected") {
                FailureCategory::Injected
            } else {
                FailureCategory::InvalidInput
            };
            (category, m.clone())
        }
        Error::UnknownMember(m) => (FailureCategory::InvalidInput, format!("unknown member {m}")),
        Error::Io { operation, source } => (
            FailureCategory::Filesystem,
            format!("{operation}: {source}"),
        ),
    };
    FailureReport::new(phase, member, category, detail)
}

/// Evidence describing the terminal result of a transaction attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionReceipt {
    product: crate::domain::ProductId,
    release: crate::domain::ReleaseId,
    disposition: TransactionDisposition,
    rollback_performed: bool,
    rollback_verified: bool,
    cleanup: CleanupDisposition,
    recovery_path: Option<PathBuf>,
    failure: Option<FailureReport>,
    rollback_failure: Option<FailureReport>,
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

    /// Returns how transaction-owned temporary evidence was handled.
    pub fn cleanup(&self) -> CleanupDisposition {
        self.cleanup
    }

    /// Returns retained backup/lock evidence when manual recovery is required.
    ///
    /// When present, the path always refers to real retained evidence. A
    /// cleanup failure after an otherwise successful commit retains the real
    /// backup root and reports the cleanup problem in [`Self::failure`];
    /// no synthetic non-existent path is ever returned.
    pub fn recovery_path(&self) -> Option<&Path> {
        self.recovery_path.as_deref()
    }

    /// Returns the structured report for the triggering failure, if any.
    ///
    /// A rolled-back transaction remains a first-class terminal result, but
    /// the caller can answer "what failed?" from this report without scraping
    /// logs. Successful commits with cleanup trouble also report the finalize
    /// problem here.
    pub fn failure(&self) -> Option<&FailureReport> {
        self.failure.as_ref()
    }

    /// Returns the rollback-phase failure when restoration itself failed.
    ///
    /// When present, [`Self::failure`] still carries the original triggering
    /// cause; this field carries the distinct recovery failure.
    pub fn rollback_failure(&self) -> Option<&FailureReport> {
        self.rollback_failure.as_ref()
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
    pub(crate) fn commit_inner(
        self,
        ownership: CommitOwnership<'_>,
        verified_digests: &HashMap<MemberId, [u8; 32]>,
        fault: Option<CommitFault>,
    ) -> Result<TransactionReceipt> {
        if fault == Some(CommitFault::LockCreation) {
            return Err(Error::invalid("injected lock creation failure"));
        }
        // Preflight ownership classification before taking the lock so a
        // change between preflight and locked revalidation is detectable.
        let preflight = classify_all(&self, &ownership)?;
        let mut lock = MutationLock::acquire(
            self.plan.installation_root(),
            self.plan.product(),
            self.plan.release(),
        )?;
        // Locked ownership revalidation: must match preflight and authorize
        // the intended mutation.
        if let Err(error) = revalidate_ownership_locked(&self, &ownership, &preflight) {
            let report = report_for_error(FailurePhase::Ownership, None, &error);
            let verified = true;
            return finish_failure_no_mutation(self, lock, verified, report);
        }
        // Verification-to-commit continuity: re-read and re-hash every staged
        // member under lock before backing up any live destination.
        if let Err((member, error)) = revalidate_staged_under_lock(&self, verified_digests) {
            let report = report_for_error(FailurePhase::StageRevalidation, member, &error);
            return finish_failure_no_mutation(self, lock, true, report);
        }
        let backup_root = match create_backup_directory(self.plan.installation_root()) {
            Ok(p) => p,
            Err(error) => {
                let report = report_for_error(FailurePhase::Backup, None, &error);
                return finish_failure_no_mutation(self, lock, true, report);
            }
        };
        let mut entries = Vec::with_capacity(self.plan.artifacts().len());
        if let Err((member, error)) = self.backup_members(&mut entries, &backup_root, fault.clone())
        {
            let phase = FailurePhase::Backup;
            let report = report_for_error(phase, member, &error);
            let (verified, rollback_failure) =
                restore_entries(&entries, &HashSet::new(), fault.clone());
            return finish_failure(self, lock, backup_root, verified, rollback_failure, report);
        }
        if fault == Some(CommitFault::BeforeFirstCommit) {
            let error = Error::invalid("injected pre-commit failure");
            let report = report_for_error(FailurePhase::Commit, None, &error);
            let (verified, rollback_failure) =
                restore_entries(&entries, &HashSet::new(), fault.clone());
            return finish_failure(self, lock, backup_root, verified, rollback_failure, report);
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
                let error = Error::invalid(format!("injected commit failure at {}", member.id()));
                let report =
                    report_for_error(FailurePhase::Commit, Some(member.id().clone()), &error);
                let (verified, rollback_failure) =
                    restore_entries(&entries, &committed, fault.clone());
                return finish_failure(self, lock, backup_root, verified, rollback_failure, report);
            }
            let destination = match self.plan.destination(member.id()) {
                Ok(destination) => destination,
                Err(error) => {
                    let report =
                        report_for_error(FailurePhase::Commit, Some(member.id().clone()), &error);
                    let (verified, rollback_failure) =
                        restore_entries(&entries, &committed, fault.clone());
                    return finish_failure(
                        self,
                        lock,
                        backup_root,
                        verified,
                        rollback_failure,
                        report,
                    );
                }
            };
            if let Err(error) = require_ready_parent(&destination, self.plan.installation_root()) {
                let report =
                    report_for_error(FailurePhase::Commit, Some(member.id().clone()), &error);
                let (verified, rollback_failure) =
                    restore_entries(&entries, &committed, fault.clone());
                return finish_failure(self, lock, backup_root, verified, rollback_failure, report);
            }
            let staged = match self.staged_path(member.id()) {
                Ok(staged) => staged,
                Err(error) => {
                    let report =
                        report_for_error(FailurePhase::Commit, Some(member.id().clone()), &error);
                    let (verified, rollback_failure) =
                        restore_entries(&entries, &committed, fault.clone());
                    return finish_failure(
                        self,
                        lock,
                        backup_root,
                        verified,
                        rollback_failure,
                        report,
                    );
                }
            };
            if let Err(source) = fs::rename(&staged, &destination) {
                let error = Error::io("replacing destination from prepared stage", source);
                let report =
                    report_for_error(FailurePhase::Commit, Some(member.id().clone()), &error);
                let (verified, rollback_failure) =
                    restore_entries(&entries, &committed, fault.clone());
                return finish_failure(self, lock, backup_root, verified, rollback_failure, report);
            }
            committed.insert(member.id().clone());
        }

        if fault == Some(CommitFault::Finalize) {
            lock.preserve();
            let report = FailureReport::new(
                FailurePhase::Finalize,
                None,
                FailureCategory::Injected,
                "injected finalize failure",
            );
            return Ok(TransactionReceipt {
                product: self.plan.product().clone(),
                release: self.plan.release().clone(),
                disposition: TransactionDisposition::Committed,
                rollback_performed: false,
                rollback_verified: false,
                cleanup: CleanupDisposition::RetainedForRecovery,
                recovery_path: Some(backup_root),
                failure: Some(report),
                rollback_failure: None,
            });
        }
        if let Err(source) = fs::remove_dir_all(&backup_root) {
            lock.preserve();
            let error = Error::io("removing backup set after commit", source);
            let report = report_for_error(FailurePhase::Finalize, None, &error);
            return Ok(TransactionReceipt {
                product: self.plan.product().clone(),
                release: self.plan.release().clone(),
                disposition: TransactionDisposition::Committed,
                rollback_performed: false,
                rollback_verified: false,
                cleanup: CleanupDisposition::RetainedForRecovery,
                recovery_path: Some(backup_root),
                failure: Some(report),
                rollback_failure: None,
            });
        }
        Ok(TransactionReceipt {
            product: self.plan.product().clone(),
            release: self.plan.release().clone(),
            disposition: TransactionDisposition::Committed,
            rollback_performed: false,
            rollback_verified: false,
            cleanup: CleanupDisposition::Cleaned,
            recovery_path: None,
            failure: None,
            rollback_failure: None,
        })
    }

    fn backup_members(
        &self,
        entries: &mut Vec<BackupEntry>,
        backup_root: &Path,
        fault: Option<CommitFault>,
    ) -> std::result::Result<(), (Option<MemberId>, Error)> {
        let members = self.plan.artifacts().iter().cloned().collect::<Vec<_>>();
        for member in members {
            if fault == Some(CommitFault::Backup(member.id().clone())) {
                return Err((
                    Some(member.id().clone()),
                    Error::invalid(format!("injected backup failure at {}", member.id())),
                ));
            }
            let destination = self
                .plan
                .destination(member.id())
                .map_err(|e| (Some(member.id().clone()), e))?;
            revalidate_destination(&destination, self.plan.installation_root())
                .map_err(|e| (Some(member.id().clone()), e))?;
            let backup = match fs::symlink_metadata(&destination) {
                Ok(_) => {
                    let path = backup_root.join(member.destination());
                    if let Some(parent) = path.parent() {
                        fs::create_dir_all(parent).map_err(|source| {
                            (
                                Some(member.id().clone()),
                                Error::io("creating backup directory", source),
                            )
                        })?;
                    }
                    fs::rename(&destination, &path).map_err(|source| {
                        (
                            Some(member.id().clone()),
                            Error::io("backing up existing destination", source),
                        )
                    })?;
                    Some(path)
                }
                Err(source) if source.kind() == std::io::ErrorKind::NotFound => None,
                Err(source) => {
                    return Err((
                        Some(member.id().clone()),
                        Error::io("reading destination before backup", source),
                    ));
                }
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

fn classify_all(
    txn: &PreparedTransaction,
    ownership: &CommitOwnership<'_>,
) -> Result<HashMap<MemberId, Ownership>> {
    let mut out = HashMap::new();
    for member in txn.artifacts().iter() {
        let destination = txn.destination(member.id())?;
        out.insert(
            member.id().clone(),
            ownership.verifier.verify(member.id(), &destination),
        );
    }
    Ok(out)
}

fn revalidate_ownership_locked(
    txn: &PreparedTransaction,
    ownership: &CommitOwnership<'_>,
    preflight: &HashMap<MemberId, Ownership>,
) -> Result<()> {
    for member in txn.artifacts().iter() {
        let destination = txn.destination(member.id())?;
        let locked = ownership.verifier.verify(member.id(), &destination);
        let first = preflight
            .get(member.id())
            .copied()
            .unwrap_or(Ownership::Unknown);
        // Any change between preflight and locked classification fails: the
        // live deployment moved under us.
        if locked != first {
            return Err(Error::DestinationConflict { destination });
        }
        match locked {
            Ownership::Owned => {}
            Ownership::Absent if ownership.absent == AbsentPolicy::AllowCreate => {}
            Ownership::Absent | Ownership::Foreign | Ownership::Unknown => {
                return Err(Error::DestinationConflict { destination });
            }
        }
        // Parent containment is re-proven under lock; missing parents fail
        // without creating anything.
        require_ready_parent(&destination, txn.plan.installation_root())?;
        revalidate_destination(&destination, txn.plan.installation_root())?;
    }
    Ok(())
}

fn revalidate_staged_under_lock(
    txn: &PreparedTransaction,
    verified_digests: &HashMap<MemberId, [u8; 32]>,
) -> std::result::Result<(), (Option<MemberId>, Error)> {
    for member in txn.artifacts().iter() {
        let expected = verified_digests.get(member.id()).ok_or_else(|| {
            (
                Some(member.id().clone()),
                Error::VerificationFailed(format!(
                    "member {} has no verified digest; commit requires verified integrity",
                    member.id()
                )),
            )
        })?;
        let staged = txn
            .staged_path(member.id())
            .map_err(|e| (Some(member.id().clone()), e))?;
        let metadata = fs::symlink_metadata(&staged).map_err(|source| {
            (
                Some(member.id().clone()),
                Error::io("reading staged candidate", source),
            )
        })?;
        if !metadata.is_file() {
            return Err((
                Some(member.id().clone()),
                Error::VerificationFailed(format!(
                    "staged member {} is not a regular file",
                    member.id()
                )),
            ));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if metadata.nlink() != 1 {
                return Err((
                    Some(member.id().clone()),
                    Error::VerificationFailed(format!(
                        "staged member {} is hard-linked",
                        member.id()
                    )),
                ));
            }
        }
        let actual = hash_file(&staged).map_err(|e| (Some(member.id().clone()), e))?;
        if &actual != expected {
            return Err((
                Some(member.id().clone()),
                Error::VerificationFailed(format!(
                    "staged member {} changed after verification",
                    member.id()
                )),
            ));
        }
    }
    Ok(())
}

fn create_backup_directory(root: &Path) -> Result<PathBuf> {
    use std::time::{SystemTime, UNIX_EPOCH};
    for _ in 0..32 {
        let nonce = NEXT_BACKUP_NONCE.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = root.join(format!(
            ".eggup-backup-{}-{nonce}-{nanos}",
            std::process::id()
        ));
        match fs::create_dir(&path) {
            Ok(()) => {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o700));
                }
                return Ok(path);
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(source) => return Err(Error::io("creating backup set", source)),
        }
    }
    Err(Error::invalid("could not create a unique backup directory"))
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

/// Requires that a live destination's parent already exists, is a real
/// directory beneath the installation root, and contains no symlink ambiguity.
///
/// Unlike the pre-corrective engine, this never calls `create_dir_all` on an
/// unchecked live path. Missing parents are an actionable error; symlink or
/// otherwise ambiguous parents fail closed.
fn require_ready_parent(destination: &Path, root: &Path) -> Result<()> {
    let parent = destination
        .parent()
        .ok_or_else(|| Error::invalid("destination has no parent"))?;
    let parent_meta = fs::symlink_metadata(parent).map_err(|source| {
        if source.kind() == std::io::ErrorKind::NotFound {
            Error::invalid(format!(
                "destination parent does not exist: {}",
                parent.display()
            ))
        } else {
            Error::io("reading destination parent", source)
        }
    })?;
    if parent_meta.file_type().is_symlink() {
        return Err(Error::DestinationConflict {
            destination: destination.to_path_buf(),
        });
    }
    if !parent_meta.is_dir() {
        return Err(Error::DestinationConflict {
            destination: destination.to_path_buf(),
        });
    }
    let canonical_root = fs::canonicalize(root)
        .map_err(|source| Error::io("canonicalizing install root", source))?;
    let canonical_parent = fs::canonicalize(parent)
        .map_err(|source| Error::io("canonicalizing destination parent", source))?;
    if !canonical_parent.starts_with(&canonical_root) {
        return Err(Error::DestinationConflict {
            destination: destination.to_path_buf(),
        });
    }
    // Walk ancestors of the parent to reject any symlink component, even when
    // the canonical target remains beneath the root.
    let mut cursor = Some(parent);
    while let Some(p) = cursor {
        if p == root {
            break;
        }
        let meta = fs::symlink_metadata(p)
            .map_err(|source| Error::io("reading destination ancestor", source))?;
        if meta.file_type().is_symlink() {
            return Err(Error::DestinationConflict {
                destination: destination.to_path_buf(),
            });
        }
        cursor = p.parent();
        if let Some(next) = cursor {
            if next.as_os_str().is_empty() {
                break;
            }
        } else {
            break;
        }
    }
    Ok(())
}

fn restore_entries(
    entries: &[BackupEntry],
    committed: &HashSet<MemberId>,
    fault: Option<CommitFault>,
) -> (bool, Option<FailureReport>) {
    let mut verified = true;
    let mut rollback_failure: Option<FailureReport> = None;
    for entry in entries.iter().rev() {
        let rollback_target = match fault.as_ref() {
            Some(CommitFault::CommitThenRollback(_, id)) => Some(id),
            _ => None,
        };
        if rollback_target == Some(&entry.member) {
            verified = false;
            rollback_failure = Some(FailureReport::new(
                FailurePhase::Rollback,
                Some(entry.member.clone()),
                FailureCategory::Injected,
                format!("injected rollback failure at {}", entry.member),
            ));
            continue;
        }
        if entry.backup.is_some() || committed.contains(&entry.member) {
            if let Ok(metadata) = fs::symlink_metadata(&entry.destination) {
                if !metadata.is_file() || fs::remove_file(&entry.destination).is_err() {
                    verified = false;
                    if rollback_failure.is_none() {
                        rollback_failure = Some(FailureReport::new(
                            FailurePhase::Rollback,
                            Some(entry.member.clone()),
                            FailureCategory::Filesystem,
                            format!("could not remove partial member {}", entry.member),
                        ));
                    }
                    continue;
                }
            }
        }
        if let Some(backup) = &entry.backup {
            if fs::rename(backup, &entry.destination).is_err() {
                verified = false;
                if rollback_failure.is_none() {
                    rollback_failure = Some(FailureReport::new(
                        FailurePhase::Rollback,
                        Some(entry.member.clone()),
                        FailureCategory::Filesystem,
                        format!("could not restore backup for {}", entry.member),
                    ));
                }
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
    (verified, rollback_failure)
}

fn finish_failure_no_mutation(
    transaction: PreparedTransaction,
    lock: MutationLock,
    nothing_mutated: bool,
    report: FailureReport,
) -> Result<TransactionReceipt> {
    debug_assert!(nothing_mutated);
    drop(lock);
    Ok(TransactionReceipt {
        product: transaction.plan.product().clone(),
        release: transaction.plan.release().clone(),
        disposition: TransactionDisposition::RolledBack,
        rollback_performed: false,
        rollback_verified: true,
        cleanup: CleanupDisposition::Cleaned,
        recovery_path: None,
        failure: Some(report),
        rollback_failure: None,
    })
}

fn finish_failure(
    transaction: PreparedTransaction,
    mut lock: MutationLock,
    backup_root: PathBuf,
    rollback_verified: bool,
    rollback_failure: Option<FailureReport>,
    report: FailureReport,
) -> Result<TransactionReceipt> {
    if rollback_verified && rollback_failure.is_none() && fs::remove_dir_all(&backup_root).is_ok() {
        return Ok(TransactionReceipt {
            product: transaction.plan.product().clone(),
            release: transaction.plan.release().clone(),
            disposition: TransactionDisposition::RolledBack,
            rollback_performed: true,
            rollback_verified: true,
            cleanup: CleanupDisposition::Cleaned,
            recovery_path: None,
            failure: Some(report),
            rollback_failure: None,
        });
    }
    lock.preserve();
    Ok(TransactionReceipt {
        product: transaction.plan.product().clone(),
        release: transaction.plan.release().clone(),
        disposition: TransactionDisposition::RecoveryRequired,
        rollback_performed: true,
        rollback_verified: false,
        cleanup: CleanupDisposition::RetainedForRecovery,
        recovery_path: Some(backup_root),
        failure: Some(report),
        rollback_failure,
    })
}
