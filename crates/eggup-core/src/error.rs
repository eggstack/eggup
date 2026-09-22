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
}

impl Error {
    pub(crate) fn invalid(message: impl Into<String>) -> Self {
        Self::InvalidInput(message.into())
    }

    pub(crate) fn io(operation: &'static str, source: io::Error) -> Self {
        Self::Io { operation, source }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(message) => write!(formatter, "invalid update input: {message}"),
            Self::UnknownMember(member) => write!(formatter, "unknown artifact member: {member}"),
            Self::Io { operation, source } => write!(formatter, "{operation}: {source}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::InvalidInput(_) | Self::UnknownMember(_) => None,
        }
    }
}
