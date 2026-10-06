use std::fmt;
use std::io;

/// The result type used by eggup-core.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors produced while validating a local update plan or preparing private stage state.
#[derive(Debug)]
pub enum Error {
    /// A caller supplied a value that violates a core invariant.
    InvalidInput(String),
    /// The requested member was not present in the artifact set.
    UnknownMember(String),
    /// An owned filesystem operation failed.
    Io {
        /// The operation being performed.
        operation: &'static str,
        /// The underlying operating-system error.
        source: io::Error,
    },
    /// Another process or an ambiguous lock record owns the installation domain.
    UpdateInProgress {
        /// The lock path that prevented acquisition.
        lock: std::path::PathBuf,
    },
    /// A live destination failed the immediate pre-mutation ownership check.
    DestinationConflict {
        /// The destination path that failed revalidation.
        destination: std::path::PathBuf,
    },
    /// Declared integrity or candidate identity evidence did not pass.
    VerificationFailed(String),
    /// Candidate execution could not produce an accepted bounded result.
    CandidateExecution(String),
    /// A failure deliberately produced by the built-in test fault harness.
    ///
    /// This variant exists so fault injection is identified structurally,
    /// never by matching caller-supplied message text. It is compiled only for
    /// this crate's own tests, so the published error surface stays exactly
    /// the seven variants `0.1.2` shipped; production paths cannot construct
    /// it because the packaged enum does not contain it.
    #[cfg(test)]
    Injected(String),
}

/// The result type used by the recovery-aware entry points.
///
/// These APIs can fail either with an ordinary [`Error`] or by retaining real
/// evidence that an operator must inspect. Keeping those apart means a
/// downstream `match` on [`Error`] stays exhaustive, while a recovery failure
/// still carries its evidence path in a typed field rather than in text.
#[non_exhaustive]
#[derive(Debug)]
pub enum RecoveryError {
    /// An ordinary core failure with no retained evidence.
    Core(Error),
    /// A partial recovery left Eggup-owned evidence that could not be resolved
    /// automatically.
    ///
    /// `evidence` always names real retained bytes on disk. This is never
    /// reported as a successful acquisition or a completed transaction.
    RecoveryRequired {
        /// Real, retained Eggup-owned evidence an operator must inspect.
        evidence: std::path::PathBuf,
        /// Bounded human-readable detail.
        detail: String,
    },
}

/// The result type returned by the recovery-aware entry points.
pub type RecoveryResult<T> = std::result::Result<T, RecoveryError>;

impl RecoveryError {
    /// Returns the real retained evidence path when this is a recovery failure.
    pub fn evidence(&self) -> Option<&std::path::Path> {
        match self {
            Self::Core(_) => None,
            Self::RecoveryRequired { evidence, .. } => Some(evidence),
        }
    }

    /// Returns bounded human-readable detail for either variant.
    pub fn detail(&self) -> &str {
        match self {
            Self::Core(error) => match error {
                Error::InvalidInput(m)
                | Error::UnknownMember(m)
                | Error::VerificationFailed(m)
                | Error::CandidateExecution(m) => m,
                _ => "",
            },
            Self::RecoveryRequired { detail, .. } => detail,
        }
    }

    /// Converts back to an ordinary [`Error`] for internal callers that cannot
    /// represent retained evidence.
    ///
    /// Only used by paths that never enable stale-lock recovery, so
    /// [`RecoveryError::RecoveryRequired`] cannot be constructed there. The
    /// fallback arm still names the retained evidence instead of discarding it,
    /// because silently dropping a path an operator needs is worse than an
    /// approximate variant.
    pub(crate) fn into_core(self) -> Error {
        match self {
            Self::Core(error) => error,
            Self::RecoveryRequired { evidence, detail } => Error::io(
                "internal recovery error outside a recovery entry point",
                io::Error::other(format!("{detail} (retained at {})", evidence.display())),
            ),
        }
    }
}

impl From<Error> for RecoveryError {
    fn from(error: Error) -> Self {
        Self::Core(error)
    }
}

impl Error {
    pub(crate) fn invalid(message: impl Into<String>) -> Self {
        Self::InvalidInput(message.into())
    }

    /// Builds a fault-harness failure. Only the built-in fault injection
    /// points may call this, and only in this crate's own test builds.
    #[cfg(test)]
    pub(crate) fn injected(message: impl Into<String>) -> Self {
        Self::Injected(message.into())
    }

    pub(crate) fn io(operation: &'static str, source: io::Error) -> Self {
        Self::Io { operation, source }
    }
}

impl fmt::Display for RecoveryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Core(error) => error.fmt(formatter),
            Self::RecoveryRequired { evidence, detail } => write!(
                formatter,
                "recovery evidence retained at {}: {detail}",
                evidence.display()
            ),
        }
    }
}

impl std::error::Error for RecoveryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Core(error) => Some(error),
            Self::RecoveryRequired { .. } => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(message) => write!(formatter, "invalid update input: {message}"),
            Self::UnknownMember(member) => write!(formatter, "unknown artifact member: {member}"),
            Self::Io { operation, source } => write!(formatter, "{operation}: {source}"),
            Self::UpdateInProgress { lock } => {
                write!(
                    formatter,
                    "installation update already in progress: {}",
                    lock.display()
                )
            }
            Self::DestinationConflict { destination } => write!(
                formatter,
                "destination failed ownership revalidation: {}",
                destination.display()
            ),
            Self::VerificationFailed(message) => {
                write!(formatter, "verification failed: {message}")
            }
            Self::CandidateExecution(message) => {
                write!(formatter, "candidate execution failed: {message}")
            }
            #[cfg(test)]
            Self::Injected(message) => {
                write!(formatter, "injected failure: {message}")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            #[cfg(test)]
            Self::Injected(_) => None,
            Self::InvalidInput(_)
            | Self::UnknownMember(_)
            | Self::UpdateInProgress { .. }
            | Self::DestinationConflict { .. }
            | Self::VerificationFailed(_)
            | Self::CandidateExecution(_) => None,
        }
    }
}
