use std::error::Error;
use std::fmt::{self, Display};
use std::io;

/// Possible errors that occur when interacting with [KvStore](crate::KvStore).
#[derive(Debug)]
pub enum KvsError {
    /// Tried to remove an entry for a key that doesn't exist
    KeyNotFound { key: String },
    /// Error related to file IO
    Io { cause: io::Error },
    /// Error (de)serializing the data in the store
    Serialization { cause: Box<bincode::ErrorKind> },
}

/// Alias for a `kvs` operation that may fail.
pub type KvsResult<T> = Result<T, KvsError>;

impl From<Box<bincode::ErrorKind>> for KvsError {
    fn from(bincode_error: Box<bincode::ErrorKind>) -> Self {
        Self::Serialization {
            cause: bincode_error,
        }
    }
}

impl From<io::Error> for KvsError {
    fn from(io_error: io::Error) -> Self {
        Self::Io { cause: io_error }
    }
}

impl Display for KvsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KeyNotFound { key } => write!(f, "Key not found: {}", key),
            Self::Io { cause } => write!(f, "Io: {}", cause),
            Self::Serialization { cause } => write!(f, "Serialization: {}", cause),
        }
    }
}

impl Error for KvsError {
    fn description(&self) -> &str {
        match self {
            Self::Io { .. } => "IO error occurred",
            Self::Serialization { .. } => "Serialization error occurred",
            _ => "Key not found",
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        match self {
            Self::Io { cause } => Some(cause),
            Self::Serialization { cause } => Some(cause),
            _ => None,
        }
    }
}
