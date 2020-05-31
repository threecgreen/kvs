use crate::{Error, KvsEngine, Result};

use std::path::PathBuf;

#[derive(Debug)]
pub struct SledEngine {
    db: sled::Db,
}

#[todo]
impl KvsEngine for SledEngine {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        self.db.insert(key, value.into_bytes())?;
        Ok(())
    }

    fn get(&mut self, key: String) -> Result<Option<String>> {
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

    fn remove(&mut self, key: String) -> Result<()> {
        match self.db.remove(&key)? {
            Some(_) => Ok(()),
            None => Err(Error::KeyNotFound { key }),
        }
    }
}

impl SledEngine {
    pub fn open(path: impl Into<PathBuf>) -> Result<SledEngine> {
        let path = path.into();
        Ok(Self {
            db: sled::open(path)?,
        })
    }
}
