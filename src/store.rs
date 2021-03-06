use crate::{KvsError, KvsResult};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{create_dir_all, read_dir, remove_file, File, OpenOptions};
use std::io::{BufWriter, Seek, SeekFrom};
use std::path::PathBuf;

/// Key-value store where both key and value are `String`s. Uses a
/// write-ahead log (WAL) to safely persist data to the filesystem. This also
/// allows the database to hold more data than can be stored in memory.
///
/// Implements periodic compaction to eliminate duplicate entries and prevent
/// the write-ahead log from continuously growing. The compaction happens
/// automatically once the number of opportunities has reached
/// `COMPACTION_LIMIT`, however it can also be triggered manually by calling
/// `KvStore::compact()`.
#[derive(Debug)]
pub struct KvStore {
    path: PathBuf,
    log_file: File,
    /// Store position and file instead of deserialized values to save memory
    index: HashMap<String, LogPtr>,
    /// Number of opportunities for compaction, i.e. places where there are
    /// log entries that could be eliminated
    compactions: u16,
    /// max id of current log files
    monotonic: u64,
}

/// Arbitrary limit before compacting. Could be made configurable or experiment
/// to find good number
static COMPACTION_LIMIT: u16 = 50;

impl KvStore {
    /// Open the database at `path`. To create a new database `path` should be
    /// an empty directory.
    pub fn open(path: impl Into<PathBuf>) -> KvsResult<KvStore> {
        let path = path.into();
        create_dir_all(&path)?;
        let log_file_nums = KvStore::sorted_file_nums(&path)?;

        // Build index
        let mut index = HashMap::new();
        let mut compactions = 0u16;
        let monotonic = if log_file_nums.is_empty() {
            1
        } else {
            // `fold` files together
            for file_num in &log_file_nums {
                let mut log_file = KvStore::open_file(&path.join(format!("{}.log", file_num)))?;
                loop {
                    let pos = KvStore::current_pos(&mut log_file)?;
                    if let Ok(op) = bincode::deserialize_from(&log_file) {
                        match op {
                            Op::Set { key, .. } => {
                                if index
                                    .insert(
                                        key,
                                        LogPtr {
                                            file_num: file_num.to_owned(),
                                            pos,
                                        },
                                    )
                                    .is_some()
                                {
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
            }
            log_file_nums.last().unwrap().to_owned()
        };
        Ok(KvStore {
            log_file: KvStore::open_file(&path.join(format!("{}.log", monotonic)))?,
            path,
            index,
            compactions,
            monotonic,
        })
    }

    /// Set the value of `key` to `value`. Overwrites any existing entry for
    /// `key`.
    pub fn set(&mut self, key: String, value: String) -> KvsResult<()> {
        // Log
        let op = Op::Set {
            key: key.clone(),
            value,
        };
        let pos = self.log_file.seek(SeekFrom::End(0))?;
        let writer = BufWriter::new(&self.log_file);
        bincode::serialize_into(writer, &op)?;
        // Set
        if self
            .index
            .insert(
                key,
                LogPtr {
                    file_num: self.monotonic,
                    pos,
                },
            )
            .is_some()
        {
            // Compaction
            self.compactions += 1;
            self.compact_maybe()?;
        }
        Ok(())
    }

    /// Get the value associated with `key`. Returns `Some(value)` if the entry
    // exists, otherwise `None`
    pub fn get(&mut self, key: String) -> KvsResult<Option<String>> {
        match self.index.get(&key) {
            Some(log_ptr) => KvStore::value_at_pos(&self.log_file, log_ptr.pos).map(Some),
            None => Ok(None),
        }
    }

    /// Remove the entry for `key`. Returns `Err(KvsError::KeyNotFound)` if
    /// there is no entry for `key`.
    pub fn remove(&mut self, key: String) -> KvsResult<()> {
        // Error checking
        if !self.index.contains_key(&key) {
            return Err(KvsError::KeyNotFound { key });
        }
        // Log
        let op = Op::Rm { key: key.clone() };
        let writer = BufWriter::new(&self.log_file);
        bincode::serialize_into(writer, &op)?;
        // Remove
        self.index.remove(&key);
        // Compaction
        self.compactions += 1;
        self.compact_maybe()?;
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
    /// removals and sets that are overwritten later.
    pub fn compact(&mut self) -> KvsResult<()> {
        let mut new_log =
            KvStore::open_file(&self.path.join(format!("{}.log", self.monotonic + 1)))?;
        for (key, log_ptr) in &mut self.index {
            // Even if we error out writing these, the data will not be
            // corrupted
            let value = if log_ptr.file_num == self.monotonic {
                KvStore::value_at_pos(&self.log_file, log_ptr.pos)?
            } else {
                let log_file =
                    KvStore::open_file(&self.path.join(format!("{}.log", log_ptr.file_num)))?;
                KvStore::value_at_pos(&log_file, log_ptr.pos)?
            };
            let pos = new_log.seek(SeekFrom::End(0))?;
            let writer = BufWriter::new(&new_log);
            bincode::serialize_into(
                writer,
                &Op::Set {
                    key: key.clone(),
                    value,
                },
            )?;
            log_ptr.file_num = self.monotonic + 1;
            log_ptr.pos = pos;
        }
        remove_file(self.path.join(format!("{}.log", self.monotonic)))?;
        self.log_file = new_log;
        self.compactions = 0;
        self.monotonic += 1;
        Ok(())
    }

    fn sorted_file_nums(path: &PathBuf) -> KvsResult<Vec<u64>> {
        let mut log_files: Vec<u64> = read_dir(path)?
            .filter_map(|fp| {
                if let Ok(fp) = fp {
                    let file_name = fp.file_name().into_string();
                    match (fp.path().is_dir(), file_name) {
                        (true, _) => None,
                        (false, Ok(n)) if n.ends_with(".log") => KvStore::parse_file_num(&n),
                        _ => None,
                    }
                } else {
                    None
                }
            })
            .collect();
        log_files.sort();
        Ok(log_files)
    }

    fn parse_file_num(file_name: &str) -> Option<u64> {
        file_name
            .chars()
            .take_while(|c| c.is_digit(10))
            .fold("".to_owned(), |acc, c| format!("{}{}", acc, c))
            .parse::<u64>()
            .ok()
    }

    fn open_file(path: &PathBuf) -> Result<File, std::io::Error> {
        OpenOptions::new()
            .create(true)
            .read(true)
            // Always append the log
            .append(true)
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

#[derive(Debug)]
struct LogPtr {
    pub file_num: u64,
    pub pos: u64,
}

#[derive(Deserialize, Serialize, Debug)]
enum Op {
    Set { key: String, value: String },
    Rm { key: String },
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_good_file_num() {
        assert_eq!(Some(100_102), KvStore::parse_file_num("100102.log"));
        assert_eq!(Some(0), KvStore::parse_file_num("0.log"));
    }

    #[test]
    fn parse_bad_file_num() {
        assert_eq!(None, KvStore::parse_file_num("kvs.log"));
    }
}
