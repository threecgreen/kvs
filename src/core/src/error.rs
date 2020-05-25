use std::error::Error as StdError;
use std::fmt::{self, Display};
use std::io;

/// Possible errors that occur when interacting with [KvStore](crate::KvStore).
#[derive(Debug)]
pub enum Error {
    /// Tried to remove an entry for a key that doesn't exist
    KeyNotFound { key: String },
    /// Error related to file IO
    Io { cause: io::Error },
    /// Error (de)serializing the data in the store
    Serialization { cause: Box<bincode::ErrorKind> },
}

/// Alias for a `kvs` operation that may fail.
pub type Result<T> = std::result::Result<T, Error>;

impl From<Box<bincode::ErrorKind>> for Error {
    fn from(bincode_error: Box<bincode::ErrorKind>) -> Self {
        Self::Serialization {
            cause: bincode_error,
        }
    }
}

impl From<io::Error> for Error {
    fn from(io_error: io::Error) -> Self {
        Self::Io { cause: io_error }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KeyNotFound { key } => write!(f, "Key not found: {}", key),
            Self::Io { cause } => write!(f, "Io: {}", cause),
            Self::Serialization { cause } => write!(f, "Serialization: {}", cause),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match self {
            Self::Io { .. } => "IO error occurred",
            Self::Serialization { .. } => "Serialization error occurred",
            _ => "Key not found",
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match self {
            Self::Io { cause } => Some(cause),
            Self::Serialization { cause } => Some(cause),
            _ => None,
        }
    }
}
