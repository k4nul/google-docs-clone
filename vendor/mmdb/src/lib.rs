use std::{
    collections::BTreeMap,
    error::Error as StdError,
    fmt,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::Mutex,
};

use serde::{Deserialize, Serialize};

const STORE_FILE_NAME: &str = "store.json";
const TEMP_FILE_NAME: &str = "store.json.tmp";

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Serde(serde_json::Error),
    Poisoned,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Serde(error) => write!(f, "{error}"),
            Self::Poisoned => write!(f, "database mutex poisoned"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Serde(error) => Some(error),
            Self::Poisoned => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Serde(value)
    }
}

#[derive(Debug, Clone, Default)]
pub struct DbOptions {
    pub create_if_missing: bool,
}

#[derive(Debug, Clone, Default)]
pub struct WriteOptions {
    pub sync: bool,
}

#[derive(Debug, Default)]
pub struct WriteBatch {
    operations: Vec<BatchOperation>,
}

impl WriteBatch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn put(&mut self, key: &[u8], value: &[u8]) {
        self.operations.push(BatchOperation::Put(
            encode_bytes(key),
            encode_bytes(value),
        ));
    }

    pub fn delete(&mut self, key: &[u8]) {
        self.operations
            .push(BatchOperation::Delete(encode_bytes(key)));
    }
}

#[derive(Debug)]
pub struct DB {
    path: PathBuf,
    state: Mutex<PersistedStore>,
}

impl DB {
    pub fn open(options: DbOptions, path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref().to_path_buf();
        if path.exists() {
            let state = load_store(&path)?;
            return Ok(Self {
                path,
                state: Mutex::new(state),
            });
        }

        if !options.create_if_missing {
            return Err(Error::Io(io::Error::new(
                io::ErrorKind::NotFound,
                format!("{} does not exist", path.display()),
            )));
        }

        fs::create_dir_all(&path)?;
        let state = PersistedStore::default();
        persist_store(&path, &state)?;
        Ok(Self {
            path,
            state: Mutex::new(state),
        })
    }

    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        let state = self.state.lock().map_err(|_| Error::Poisoned)?;
        Ok(state
            .entries
            .get(&encode_bytes(key))
            .map(|value| decode_bytes(value))
            .transpose()?)
    }

    pub fn write_with_options(
        &self,
        batch: WriteBatch,
        options: &WriteOptions,
    ) -> Result<(), Error> {
        let mut state = self.state.lock().map_err(|_| Error::Poisoned)?;
        for operation in batch.operations {
            match operation {
                BatchOperation::Put(key, value) => {
                    state.entries.insert(key, value);
                }
                BatchOperation::Delete(key) => {
                    state.entries.remove(&key);
                }
            }
        }

        if options.sync {
            persist_store(&self.path, &state)?;
        }

        Ok(())
    }

    pub fn flush(&self) -> Result<(), Error> {
        let state = self.state.lock().map_err(|_| Error::Poisoned)?;
        persist_store(&self.path, &state)
    }
}

#[derive(Debug)]
enum BatchOperation {
    Put(String, String),
    Delete(String),
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct PersistedStore {
    entries: BTreeMap<String, String>,
}

fn load_store(path: &Path) -> Result<PersistedStore, Error> {
    let store_path = path.join(STORE_FILE_NAME);
    match fs::read(&store_path) {
        Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(PersistedStore::default()),
        Err(error) => Err(Error::Io(error)),
    }
}

fn persist_store(path: &Path, state: &PersistedStore) -> Result<(), Error> {
    fs::create_dir_all(path)?;
    let temp_path = path.join(TEMP_FILE_NAME);
    let final_path = path.join(STORE_FILE_NAME);
    let bytes = serde_json::to_vec(state)?;
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&temp_path)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    drop(file);
    fs::rename(&temp_path, &final_path)?;
    sync_dir(path)?;
    Ok(())
}

fn sync_dir(path: &Path) -> Result<(), Error> {
    let file = File::open(path)?;
    file.sync_all()?;
    Ok(())
}

fn encode_bytes(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(nibble_to_hex(byte >> 4));
        encoded.push(nibble_to_hex(byte & 0x0f));
    }
    encoded
}

fn decode_bytes(encoded: &str) -> Result<Vec<u8>, Error> {
    if encoded.len() % 2 != 0 {
        return Err(Error::Io(io::Error::new(
            io::ErrorKind::InvalidData,
            "hex payload has odd length",
        )));
    }

    let mut bytes = Vec::with_capacity(encoded.len() / 2);
    for pair in encoded.as_bytes().chunks_exact(2) {
        let high = hex_to_nibble(pair[0])?;
        let low = hex_to_nibble(pair[1])?;
        bytes.push((high << 4) | low);
    }
    Ok(bytes)
}

fn nibble_to_hex(nibble: u8) -> char {
    match nibble {
        0..=9 => (b'0' + nibble) as char,
        10..=15 => (b'a' + (nibble - 10)) as char,
        _ => unreachable!("nibble is always <= 0x0f"),
    }
}

fn hex_to_nibble(byte: u8) -> Result<u8, Error> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(Error::Io(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid hex payload",
        ))),
    }
}
