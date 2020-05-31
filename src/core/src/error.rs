use std::error::Error as StdError;
use std::fmt::{self, Display};
use std::io;
use std::sync;

/// Possible errors that occur when interacting with [KvStore](crate::KvStore).
#[derive(Debug)]
pub enum Error {
    /// Tried to remove an entry for a key that doesn't exist
    KeyNotFound { key: String },
    /// Error related to file IO
    Io { cause: io::Error },
    /// Error (de)serializing the data in the store
    Serialization { cause: Box<bincode::ErrorKind> },
    /// Server error
    Server { msg: String },
    /// Synchronization
    Synchronization { msg: String }
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
            Self::Server { msg } => write!(f, "Server: {}", msg),
            Self::Synchronization { msg } => write!(f, "Synchronization: {}", msg),
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

mod slog {
    use super::Error;
    use slog::{Key, Record, Result, Serializer, Value};

    impl Value for Error {
        fn serialize(&self, _rec: &Record, key: Key, serializer: &mut dyn Serializer) -> Result {
            serializer.emit_str(key, &format!("{:?}", self))
        }
    }
}

#[cfg(feature = "sled_engine")]
pub mod sled {
    use super::Error;

    impl From<sled::Error> for Error {
        fn from(sled_error: sled::Error) -> Self {
            match sled_error {
                sled::Error::Io(e) => Error::Io { cause: e },
                e => Error::Server {
                    msg: format!("{}", e),
                },
            }
        }
    }
}
