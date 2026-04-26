use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use log::info;

use crate::errors::{Result, ShortDBErrors};

const STORE_FILE_NAME: &str = "store.bin";
const TEMP_STORE_FILE_NAME: &str = "store.bin.tmp";
const STORE_MAGIC: &[u8; 8] = b"SHRTDB\0\x01";
const MAX_ENTRY_BYTES: usize = 128 * 1024 * 1024;

/// A lightweight embedded key-value store with synchronous file persistence.
///
/// The backend only relies on `new`, `get`, `set`, `delete`, and `close`, so this
/// repository-local implementation keeps that narrow API surface and writes a
/// compact append-free snapshot file on every mutation.
pub struct ShorterDB {
    data_dir: PathBuf,
    entries: BTreeMap<Vec<u8>, Vec<u8>>,
    closed: bool,
}

impl ShorterDB {
    /// Open a database with default settings.
    pub fn new<P: AsRef<Path>>(data_dir: P) -> Result<Self> {
        Self::with_memtable_size(data_dir, 0)
    }

    /// Open a database with a custom memtable size.
    ///
    /// The upstream crate exposes this constructor, so the shim keeps it for
    /// compatibility even though persistence is fully synchronous here.
    pub fn with_memtable_size<P: AsRef<Path>>(data_dir: P, _memtable_size: usize) -> Result<Self> {
        let data_dir = data_dir.as_ref().to_path_buf();
        info!("Opening database at {:?}", data_dir);
        fs::create_dir_all(&data_dir)?;

        let entries = Self::load_entries(&Self::store_path(&data_dir))?;

        Ok(Self {
            data_dir,
            entries,
            closed: false,
        })
    }

    /// Get a value by key.
    pub fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>> {
        Ok(self.entries.get(key.as_ref()).cloned())
    }

    /// Set a key-value pair and persist it immediately.
    pub fn set(&mut self, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> Result<()> {
        self.entries
            .insert(key.as_ref().to_vec(), value.as_ref().to_vec());
        self.persist()
    }

    /// Delete a key and persist the new state immediately.
    pub fn delete(&mut self, key: impl AsRef<[u8]>) -> Result<bool> {
        let existed = self.entries.remove(key.as_ref()).is_some();
        self.persist()?;
        Ok(existed)
    }

    /// Gracefully close the database.
    pub fn close(&mut self) -> Result<()> {
        if self.closed {
            return Ok(());
        }

        info!("Closing database");
        self.closed = true;
        Ok(())
    }

    fn persist(&self) -> Result<()> {
        let store_path = Self::store_path(&self.data_dir);
        let temp_path = Self::temp_store_path(&self.data_dir);
        let file = File::create(&temp_path)?;
        let mut writer = BufWriter::new(file);

        writer.write_all(STORE_MAGIC)?;
        for (key, value) in &self.entries {
            Self::write_len_prefixed(&mut writer, key)?;
            Self::write_len_prefixed(&mut writer, value)?;
        }
        writer.flush()?;
        writer.get_ref().sync_all()?;
        drop(writer);

        fs::rename(&temp_path, &store_path)?;
        Self::sync_directory(&self.data_dir)?;
        Ok(())
    }

    fn load_entries(path: &Path) -> Result<BTreeMap<Vec<u8>, Vec<u8>>> {
        let file = match File::open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(BTreeMap::new());
            }
            Err(error) => return Err(error.into()),
        };
        let mut reader = BufReader::new(file);
        let mut magic = [0u8; STORE_MAGIC.len()];
        reader.read_exact(&mut magic)?;
        if &magic != STORE_MAGIC {
            return Err(ShortDBErrors::SstCorruption(
                "invalid shorterdb store header".to_owned(),
            ));
        }

        let mut entries = BTreeMap::new();
        loop {
            let Some(key_len) = Self::read_u32(&mut reader)? else {
                break;
            };
            let key = Self::read_bytes(&mut reader, key_len as usize)?;
            let value_len = Self::read_required_u32(&mut reader)?;
            let value = Self::read_bytes(&mut reader, value_len as usize)?;
            entries.insert(key, value);
        }

        Ok(entries)
    }

    fn read_u32(reader: &mut BufReader<File>) -> Result<Option<u32>> {
        let mut bytes = [0u8; 4];
        match reader.read_exact(&mut bytes) {
            Ok(()) => Ok(Some(u32::from_le_bytes(bytes))),
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    fn read_required_u32(reader: &mut BufReader<File>) -> Result<u32> {
        Self::read_u32(reader)?.ok_or_else(|| {
            ShortDBErrors::SstCorruption("truncated shorterdb store file".to_owned())
        })
    }

    fn read_bytes(reader: &mut BufReader<File>, len: usize) -> Result<Vec<u8>> {
        if len > MAX_ENTRY_BYTES {
            return Err(ShortDBErrors::SstCorruption(
                "shorterdb entry exceeds maximum size".to_owned(),
            ));
        }

        let mut bytes = vec![0u8; len];
        reader.read_exact(&mut bytes).map_err(|error| match error.kind() {
            io::ErrorKind::UnexpectedEof => {
                ShortDBErrors::SstCorruption("truncated shorterdb store file".to_owned())
            }
            _ => error.into(),
        })?;
        Ok(bytes)
    }

    fn write_len_prefixed(writer: &mut BufWriter<File>, bytes: &[u8]) -> Result<()> {
        writer.write_all(&(bytes.len() as u32).to_le_bytes())?;
        writer.write_all(bytes)?;
        Ok(())
    }

    fn store_path(data_dir: &Path) -> PathBuf {
        data_dir.join(STORE_FILE_NAME)
    }

    fn temp_store_path(data_dir: &Path) -> PathBuf {
        data_dir.join(TEMP_STORE_FILE_NAME)
    }

    fn sync_directory(path: &Path) -> Result<()> {
        File::open(path)?.sync_all()?;
        Ok(())
    }
}

impl Drop for ShorterDB {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
