use crate::store::LOG_EXT;
use crate::{Error, KvsEngine, Result};

use std::fs::read_dir;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct SledEngine {
    db: sled::Db,
}

impl KvsEngine for SledEngine {
    fn set(&self, key: String, value: String) -> Result<()> {
        self.db.insert(key, value.into_bytes())?;
        Ok(())
    }

    fn get(&self, key: String) -> Result<Option<String>> {
        match self.db.get(key.into_bytes()) {
            Ok(Some(v)) => {
                let s = String::from_utf8(v.to_vec()).map_err(|e| Error::Server {
                    msg: format!("{}", e),
                })?;
                Ok(Some(s))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(Error::from(e)),
        }
    }

    fn remove(&self, key: String) -> Result<()> {
        match self.db.remove(&key)? {
            Some(_) => Ok(()),
            None => Err(Error::KeyNotFound { key }),
        }
    }
}

impl SledEngine {
    pub fn open(path: impl Into<PathBuf>) -> Result<SledEngine> {
        let path = path.into();
        // Check if dir contains kvs log files
        let contains_kvs_files = read_dir(&path)?.any(|dir_entry| {
            if let Ok(dir_entry) = dir_entry {
                !dir_entry.path().is_dir() && dir_entry.path().ends_with(&format!(".{}", LOG_EXT))
            } else {
                false
            }
        });
        if contains_kvs_files {
            return Err(Error::Io {
                cause: std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "path contains data for a different database engine",
                ),
            });
        }
        Ok(Self {
            db: sled::open(path)?,
        })
    }
}
