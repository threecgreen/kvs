use crate::{KvsError, KvsResult};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{create_dir_all, remove_file, File, OpenOptions};
use std::io::{BufWriter, Seek, SeekFrom};
use std::path::PathBuf;

/// Key-value store where both key and value are `String`s
pub struct KvStore {
    path: PathBuf,
    log_file: File,
    /// Store position and file instead of deserialized values to save memory
    // TODO: Could be an optimization to store values for smaller data and
    // position for larger.
    index: HashMap<String, u64>,
    /// Number of opportunities for compaction, i.e. places where there are
    /// log entries that could be eliminated
    compactions: u16,
    /// max id of current log files
    monotonic: u64,
}

/// Arbitrary limit before compacting. Could be made configurable or experiment
/// to find good number
static COMPACTION_LIMIT: u16 = 5;

impl KvStore {
    pub fn open(path: impl Into<PathBuf>) -> KvsResult<KvStore> {
        let path = path.into();
        create_dir_all(&path)?;
        // `fold` files together
        let mut log_file = KvStore::open_file(&path.join("kvs1.log"))?;

        // Build index
        let mut index = HashMap::new();
        let mut compactions = 0u16;
        loop {
            let last_pos = KvStore::current_pos(&mut log_file)?;
            if let Ok(op) = bincode::deserialize_from(&log_file) {
                match op {
                    Op::Set { key, .. } => {
                        if let Some(_) = index.insert(key, last_pos) {
                            // `key` previously existed in `index`. This is an
                            // opportunity for compaction
                            compactions += 1;
                        }
                    }
                    Op::Rm { key } => {
                        index.remove(&key);
                        compactions += 1;
                    }
                };
            } else {
                break;
            }
        }
        // TODO: compact here?
        Ok(KvStore {
            path,
            log_file,
            index,
            compactions,
            monotonic: 1,
        })
    }

    pub fn set(&mut self, key: String, value: String) -> KvsResult<()> {
        // Compaction
        if self.index.contains_key(&key) {
            self.compactions += 1;
            self.compact_maybe()?;
        }
        // Log
        let op = Op::Set {
            key: key.clone(),
            value,
        };
        let pos = self.log_file.seek(SeekFrom::End(0))?;
        let writer = BufWriter::new(&self.log_file);
        bincode::serialize_into(writer, &op)?;
        // Set
        self.index.insert(key, pos);
        Ok(())
    }

    pub fn get(&mut self, key: String) -> KvsResult<Option<String>> {
        match self.index.get(&key) {
            Some(pos) => KvStore::value_at_pos(&self.log_file, *pos).map(Some),
            None => Ok(None),
        }
    }

    pub fn remove(&mut self, key: String) -> KvsResult<()> {
        // Error checking
        if !self.index.contains_key(&key) {
            return Err(KvsError::KeyNotFound { key });
        }
        // Compaction
        self.compactions += 1;
        self.compact_maybe()?;
        // Log
        let op = Op::Rm { key: key.clone() };
        let writer = BufWriter::new(&self.log_file);
        bincode::serialize_into(writer, &op)?;
        // Remove
        self.index.remove(&key);
        Ok(())
    }

    fn compact_maybe(&mut self) -> KvsResult<()> {
        if self.compactions >= COMPACTION_LIMIT {
            self.compact()
        } else {
            Ok(())
        }
    }

    /// Forces compaction. Rewrites log, eliminating unnecessary logs, i.e.
    /// `Op::Rm`s and `Op::Set`s that are set again later.
    pub fn compact(&mut self) -> KvsResult<()> {
        let new_log = File::open(format!("kvs{}.log", self.monotonic + 1))?;
        for (key, pos) in &self.index {
            // Even if we error out writing these, the data will not be
            // corrupted
            let value = KvStore::value_at_pos(&self.log_file, *pos)?;
            let writer = BufWriter::new(&new_log);
            bincode::serialize_into(
                writer,
                &Op::Set {
                    key: key.clone(),
                    value,
                },
            )?;
        }
        remove_file(self.path.join(format!("kvs{}.log", self.monotonic)))?;
        self.log_file = new_log;
        self.monotonic += 1;
        Ok(())
    }

    fn open_file(path: &PathBuf) -> Result<File, std::io::Error> {
        OpenOptions::new()
            .create(true)
            .read(true)
            // Always append the log
            .append(true)
            // TODO: use `monotonic`
            .open(path.join(path))
    }

    fn current_pos<S: Seek>(reader: &mut S) -> KvsResult<u64> {
        Ok(reader.seek(SeekFrom::Current(0))?)
    }

    fn value_at_pos<S: Seek + std::io::Read>(mut reader: S, pos: u64) -> KvsResult<String> {
        reader.seek(SeekFrom::Start(pos))?;
        match bincode::deserialize_from(reader)? {
            Op::Set { value, .. } => Ok(value),
            // TODO: create error enum for this. If this happens the
            // index is somewhat corrupted and should maybe be rebuilt.
            Op::Rm { key } => Err(KvsError::KeyNotFound { key }),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
enum Op {
    Set { key: String, value: String },
    Rm { key: String },
}
