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

/// A bounded, read-only observation of one exact lock record.
///
/// This is the only evidence Core can prove about a record: the path, its exact
/// bounded bytes, and whatever the known record format yields. There is no
/// process liveness, age, or executable fact here, and none is inferred from
/// the pathname.
///
/// Field values are exposed through accessors rather than public fields so the
/// record format can stay an implementation detail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockObservation {
    path: PathBuf,
    record: String,
    pid: Option<u32>,
    nonce: Option<u64>,
    product: Option<String>,
    release: Option<String>,
}

impl LockObservation {
    /// Returns the observed lock record path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the exact bounded record identity that was observed.
    ///
    /// Recovery is authorized against this value, not against the pathname: if
    /// the object at the path changes, the authorization no longer applies.
    pub fn record(&self) -> &str {
        &self.record
    }

    /// Returns the recorded process id, when the known format supplies one.
    ///
    /// This is a fact about the record, not about the current process table.
    /// Core never uses it as proof of staleness.
    pub fn pid(&self) -> Option<u32> {
        self.pid
    }

    /// Returns the recorded per-process nonce, when the known format supplies one.
    pub fn nonce(&self) -> Option<u64> {
        self.nonce
    }

    /// Returns the recorded product identity, when the known format supplies one.
    pub fn product(&self) -> Option<&str> {
        self.product.as_deref()
    }

    /// Returns the recorded release identity, when the known format supplies one.
    pub fn release(&self) -> Option<&str> {
        self.release.as_deref()
    }

    /// Returns whether the record matched the known bounded format.
    ///
    /// A record from an older or unknown format is still observable and still
    /// byte-identifiable, but it carries no parsed evidence, so a caller that
    /// needs fields cannot prove staleness from it.
    pub fn format_known(&self) -> bool {
        self.pid.is_some() && self.nonce.is_some()
    }

    fn parse(path: PathBuf, record: String) -> Self {
        let mut parsed = Self {
            path,
            record,
            pid: None,
            nonce: None,
            product: None,
            release: None,
        };
        parsed.parse_fields();
        parsed
    }

    /// Conservative parsing: every field must be present and well-formed for
    /// any of them to be reported. Partial parses are treated as unknown.
    fn parse_fields(&mut self) {
        let Some(body) = self.record.strip_suffix('\n') else {
            return;
        };
        let mut pid = None;
        let mut nonce = None;
        let mut product = None;
        let mut release = None;
        for field in body.split(' ') {
            let Some((key, value)) = field.split_once('=') else {
                return;
            };
            if value.is_empty() {
                return;
            }
            match key {
                "pid" => match value.parse::<u32>() {
                    Ok(value) => pid = Some(value),
                    Err(_) => return,
                },
                "nonce" => match value.parse::<u64>() {
                    Ok(value) => nonce = Some(value),
                    Err(_) => return,
                },
                "product" => product = Some(value.to_string()),
                "release" => release = Some(value.to_string()),
                _ => return,
            }
        }
        self.pid = pid;
        self.nonce = nonce;
        self.product = product;
        self.release = release;
    }
}

/// The caller's verdict about one observed lock record.
///
/// Core does not decide staleness. Only [`ProvenStale`](Self::ProvenStale)
/// authorizes recovery, and only for the exact observation that was classified.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum StaleLockDecision {
    /// The recorded owner is believed to still be working. Retain the record.
    Active,
    /// The caller has proven from its own deployment evidence that the exact
    /// observed owner is gone. Authorizes claiming that exact record.
    ProvenStale,
    /// Staleness could not be established. Retain the record and fail closed.
    Unknown,
}

/// Consumer-supplied proof that one observed lock record is stale.
///
/// Implementations receive only what Core can prove about the record, plus their
/// own runtime facts. They must not treat PID liveness, record age, executable
/// name, or service state as universal proof, and they must return
/// [`StaleLockDecision::Unknown`] whenever their evidence is incomplete.
///
/// The verifier cannot delete anything: it returns a verdict, and Core performs
/// the claim.
pub trait StaleLockVerifier {
    /// Classifies one observed record.
    fn classify(&self, observed: &LockObservation) -> StaleLockDecision;
}

/// An exclusive ownership record for one installation domain.
///
/// The default acquisition path is fail-closed: ambiguous, malformed, oversized,
/// or otherwise unreadable records never trigger automatic removal, and
/// [`acquire`](Self::acquire) never recovers a stale record at all. PID liveness
/// alone cannot prove process identity across this public contract, so Core
/// never treats it as proof.
///
/// Recovery is available, but only when a caller explicitly supplies
/// [`StaleLockVerifier`] evidence through
/// [`acquire_with_recovery`](Self::acquire_with_recovery), and only for the
/// exact record that evidence classified.
#[derive(Debug)]
pub struct MutationLock {
    path: PathBuf,
    token: String,
    cleanup: bool,
    _file: File,
}

impl MutationLock {
    /// Acquires the installation-root lock using create-new filesystem semantics.
    ///
    /// This is the default and never recovers: an existing record always
    /// returns [`Error::UpdateInProgress`] and is left untouched.
    pub fn acquire(root: &Path, product: &ProductId, release: &ReleaseId) -> Result<Self> {
        match Self::create_new(root, product, release)? {
            Some(lock) => Ok(lock),
            None => Err(Error::UpdateInProgress {
                lock: lock_path(root),
            }),
        }
    }

    /// Acquires the lock, recovering a stale record the caller has proven stale.
    ///
    /// The sequence is deliberately conservative:
    ///
    /// 1. create-new;
    /// 2. on an existing record, observe it within bounds;
    /// 3. ask the caller's verifier for a verdict on that exact observation;
    /// 4. only on [`StaleLockDecision::ProvenStale`], re-read the record and
    ///    rename the pathname into a unique Eggup-owned claim path in the same
    ///    directory;
    /// 5. re-read the claimed object and require it to still equal the authorized
    ///    observation;
    /// 6. create-new the real lock, then delete the claimed stale record only
    ///    after ownership is established.
    ///
    /// A record that is malformed, oversized, a symlink, non-regular, or
    /// unreadable is never claimed. `Active` and `Unknown` verdicts retain the
    /// record. If the record changes before the claim, no stale deletion happens
    /// and contention is returned. If another writer creates a new lock after the
    /// claim, that writer wins and its record is never removed.
    pub fn acquire_with_recovery(
        root: &Path,
        product: &ProductId,
        release: &ReleaseId,
        verifier: &dyn StaleLockVerifier,
    ) -> Result<Self> {
        if let Some(lock) = Self::create_new(root, product, release)? {
            return Ok(lock);
        }
        let observed = match Self::observe(root)? {
            Some(observation) => observation,
            None => {
                // The record disappeared between create-new and observation.
                // Retrying is safe: create-new still decides the winner.
                return match Self::create_new(root, product, release)? {
                    Some(lock) => Ok(lock),
                    None => Err(Error::UpdateInProgress {
                        lock: lock_path(root),
                    }),
                };
            }
        };
        if verifier.classify(&observed) != StaleLockDecision::ProvenStale {
            return Err(Error::UpdateInProgress {
                lock: observed.path().to_path_buf(),
            });
        }
        let claim = Self::claim(root, &observed)?;
        fire_post_claim_hook();
        // Ownership of the installation domain is only established by a fresh
        // create-new. If another writer won the race, its record is left alone.
        match Self::create_new(root, product, release) {
            Ok(Some(lock)) => {
                // A claim that cannot be removed means recovery did not complete
                // cleanly. Reporting it with its real path is the only truthful
                // outcome; returning `lock` here would claim a success that left
                // undisplaced evidence behind. Dropping `lock` on this path
                // releases the record again, so the domain is left as found.
                remove_claim(&claim)?;
                Ok(lock)
            }
            Ok(None) => {
                // Another writer holds the domain now. Our displaced record is
                // ours to clean up; if it cannot be cleaned, the caller is told
                // exactly where the evidence is.
                remove_claim(&claim)?;
                Err(Error::UpdateInProgress {
                    lock: lock_path(root),
                })
            }
            Err(error) => {
                // We displaced the stale record but could not take the domain.
                // Retaining the evidence and reporting it is the only truthful
                // outcome; silently dropping the record would let the caller
                // believe recovery succeeded.
                match remove_claim(&claim) {
                    Ok(()) => Err(error),
                    Err(retained) => Err(retained),
                }
            }
        }
    }

    /// Observes the lock record without acquiring or removing it.
    ///
    /// Returns `Ok(None)` when no record exists, and a bounded observation for a
    /// regular, non-symlink, UTF-8, in-bounds record. Records that are oversized,
    /// non-UTF-8, symlinked, non-regular, or unreadable yield `Err`, because
    /// Core cannot prove what they are and must therefore never displace them.
    pub fn observe(root: &Path) -> Result<Option<LockObservation>> {
        match Self::inspect(root)? {
            LockStatus::Available => Ok(None),
            LockStatus::Held {
                lock,
                contents: Some(contents),
            } => Ok(Some(LockObservation::parse(lock, contents))),
            LockStatus::Held { lock, .. } | LockStatus::Malformed { lock } => {
                Err(Error::UpdateInProgress { lock })
            }
        }
    }

    /// Inspects the lock record without acquiring or removing it.
    ///
    /// Returns [`LockStatus::Available`] when no record exists,
    /// [`LockStatus::Held`] when a bounded readable record exists, and
    /// [`LockStatus::Malformed`] when the record is missing, unreadable,
    /// oversized, or non-UTF-8. This function never deletes anything.
    pub fn inspect(root: &Path) -> Result<LockStatus> {
        let path = lock_path(root);
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

    /// Returns a freshly created lock, or `None` when a record already exists.
    fn create_new(root: &Path, product: &ProductId, release: &ReleaseId) -> Result<Option<Self>> {
        let path = lock_path(root);
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
        let mut file = match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => file,
            Err(source) if source.kind() == std::io::ErrorKind::AlreadyExists => return Ok(None),
            Err(source) => return Err(Error::io("creating mutation lock", source)),
        };
        if let Err(source) = file.write_all(token.as_bytes()) {
            let _ = fs::remove_file(&path);
            return Err(Error::io("writing mutation lock", source));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
        }
        Ok(Some(Self {
            path,
            token,
            cleanup: true,
            _file: file,
        }))
    }

    /// Moves the authorized record aside into a unique Eggup-owned claim path.
    ///
    /// The claim lives in the same directory as the lock so the move is a
    /// same-filesystem rename. The claimed object is re-read and must still equal
    /// the authorized observation; if it does not, the displacement is undone
    /// where that is safe, and otherwise reported with its real retained path.
    fn claim(root: &Path, observed: &LockObservation) -> Result<PathBuf> {
        let path = observed.path().to_path_buf();
        let claim = claim_path(root);
        // Re-read immediately before displacing: the verdict was given for a
        // specific observation, not for a pathname.
        match read_record(&path) {
            Ok(current) if current == observed.record() => {}
            Ok(_) => return Err(Error::UpdateInProgress { lock: path }),
            Err(error) => return Err(error),
        }
        fs::rename(&path, &claim)
            .map_err(|source| Error::io("claiming the authorized stale lock record", source))?;
        match read_record(&claim) {
            Ok(claimed) if claimed == observed.record() => Ok(claim),
            // The displaced object is not the authorized one, so the claim is not
            // authorized either: undo it if that is safe, otherwise retain it.
            Ok(claimed) => Err(restore_claim(&claim, &path, &claimed)),
            // The record is displaced but unverified. Reporting only the
            // underlying error would leave real Eggup-owned bytes behind with
            // no indication of where they are.
            Err(error) => Err(Error::RecoveryRequired {
                evidence: claim,
                detail: format!("the claimed stale lock record could not be verified: {error}"),
            }),
        }
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

/// Test-only seam fired in the window between the claim and the create-new retry.
///
/// A competing writer that wins that window is the case the whole claim
/// protocol exists to survive, but the window is microseconds wide. Without a
/// deterministic seam the only way to exercise the branch is a real
/// multi-process race, which would prove nothing about ordering. It is
/// `cfg(test)` so no production binary carries it.
#[cfg(test)]
pub(crate) static POST_CLAIM_HOOK: std::sync::Mutex<Option<Box<dyn Fn() + Send>>> =
    std::sync::Mutex::new(None);

#[cfg(test)]
fn fire_post_claim_hook() {
    // `take` so the hook fires exactly once even if recovery retries.
    let hook = POST_CLAIM_HOOK.lock().ok().and_then(|mut slot| slot.take());
    if let Some(hook) = hook {
        hook();
    }
}

/// The call site stays ungated so production builds do not carry a branch on a
/// test-only symbol.
#[cfg(not(test))]
fn fire_post_claim_hook() {}

fn lock_path(root: &Path) -> PathBuf {
    root.join(".eggup-mutation.lock")
}

fn claim_path(root: &Path) -> PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nonce = NEXT_LOCK_NONCE.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    root.join(format!(
        ".eggup-stale-claim-{}-{nonce}-{nanos}",
        std::process::id()
    ))
}

/// Reads a lock record only when it is a safe, bounded, UTF-8 regular file.
fn read_record(path: &Path) -> Result<String> {
    let metadata = match fs::symlink_metadata(path) {
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
            return Err(Error::UpdateInProgress {
                lock: path.to_path_buf(),
            })
        }
        Err(source) => return Err(Error::io("reading lock record", source)),
        Ok(metadata) => metadata,
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(Error::UpdateInProgress {
            lock: path.to_path_buf(),
        });
    }
    if metadata.len() > MAX_LOCK_BYTES {
        return Err(Error::UpdateInProgress {
            lock: path.to_path_buf(),
        });
    }
    match fs::read_to_string(path) {
        Ok(contents) if contents.len() as u64 <= MAX_LOCK_BYTES => Ok(contents),
        _ => Err(Error::UpdateInProgress {
            lock: path.to_path_buf(),
        }),
    }
}

/// Puts a displaced record back only if the lock pathname is still free.
///
/// The restore deliberately does **not** `rename` the claim back. POSIX `rename`
/// atomically *replaces* an existing destination, so a writer that took the lock
/// between a freeness check and a rename would have its record unlinked
/// underneath it while it still believes it holds the domain. `create_new` can
/// never replace anything, so re-materializing the record through it makes the
/// restore safe on every platform.
///
/// On success the lock path holds the displaced record again and the caller
/// reports contention; the displaced writer owns the domain, which is the
/// correct outcome for a record Core was never authorized to displace.
pub(crate) fn restore_claim(claim: &Path, lock: &Path, record: &str) -> Error {
    match OpenOptions::new().write(true).create_new(true).open(lock) {
        Ok(mut file) => {
            if let Err(source) = file.write_all(record.as_bytes()) {
                // The lock path now holds a partial record. That is fail-closed:
                // `acquire` still refuses it, and the full bytes remain at the
                // claim, which is what the error names.
                return Error::RecoveryRequired {
                    evidence: claim.to_path_buf(),
                    detail: format!(
                        "a displaced lock record was re-created at {} but could not be written: {source}",
                        lock.display()
                    ),
                };
            }
            // The claim is now a duplicate of a record that owns the lock path,
            // so removing it cannot lose the only copy.
            let _ = fs::remove_file(claim);
            Error::UpdateInProgress {
                lock: lock.to_path_buf(),
            }
        }
        Err(_) => Error::RecoveryRequired {
            evidence: claim.to_path_buf(),
            detail: "the displaced lock record was not the authorized one and the lock path was re-taken; retained evidence requires inspection"
                .to_string(),
        },
    }
}

/// Deletes a claim this transaction created and has already proven it owns.
fn remove_claim(claim: &Path) -> Result<()> {
    match fs::remove_file(claim) {
        Ok(()) => Ok(()),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(Error::RecoveryRequired {
            evidence: claim.to_path_buf(),
            detail: format!("a claimed stale lock record could not be removed: {source}"),
        }),
    }
}
