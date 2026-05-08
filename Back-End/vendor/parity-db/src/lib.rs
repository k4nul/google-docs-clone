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
    InvalidColumn(u8),
    Poisoned,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Serde(error) => write!(f, "{error}"),
            Self::InvalidColumn(column) => write!(f, "invalid column `{column}`"),
            Self::Poisoned => write!(f, "database mutex poisoned"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Serde(error) => Some(error),
            Self::InvalidColumn(_) | Self::Poisoned => None,
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
pub struct ColumnOptions {
    pub btree_index: bool,
}

#[derive(Debug, Clone)]
pub struct Options {
    path: PathBuf,
    pub columns: Vec<ColumnOptions>,
}

impl Options {
    pub fn with_columns(path: impl AsRef<Path>, columns: usize) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            columns: vec![ColumnOptions::default(); columns],
        }
    }
}

#[derive(Debug)]
pub struct Db {
    path: PathBuf,
    state: Mutex<PersistedStore>,
}

impl Db {
    pub fn open_or_create(options: &Options) -> Result<Self, Error> {
        fs::create_dir_all(&options.path)?;
        let mut state = load_store(&options.path)?;
        ensure_column_count(&mut state, options.columns.len());
        persist_store(&options.path, &state)?;

        Ok(Self {
            path: options.path.clone(),
            state: Mutex::new(state),
        })
    }

    pub fn get(&self, column: u8, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>, Error> {
        let state = self.state.lock().map_err(|_| Error::Poisoned)?;
        let column = state
            .columns
            .get(column as usize)
            .ok_or(Error::InvalidColumn(column))?;
        Ok(column
            .entries
            .get(&encode_bytes(key.as_ref()))
            .map(|value| decode_bytes(value))
            .transpose()?)
    }

    pub fn commit<I>(&self, operations: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = (u8, Vec<u8>, Option<Vec<u8>>)>,
    {
        let mut state = self.state.lock().map_err(|_| Error::Poisoned)?;
        let mut changed = false;

        for (column, key, value) in operations {
            let column = state
                .columns
                .get_mut(column as usize)
                .ok_or(Error::InvalidColumn(column))?;
            let key = encode_bytes(&key);
            match value {
                Some(value) => {
                    column.entries.insert(key, encode_bytes(&value));
                }
                None => {
                    column.entries.remove(&key);
                }
            }
            changed = true;
        }

        if changed {
            persist_store(&self.path, &state)?;
        }

        Ok(())
    }

    pub fn iter(&self, column: u8) -> Result<Iter, Error> {
        let state = self.state.lock().map_err(|_| Error::Poisoned)?;
        let column = state
            .columns
            .get(column as usize)
            .ok_or(Error::InvalidColumn(column))?;
        let entries = column
            .entries
            .iter()
            .map(|(key, value)| Ok((decode_bytes(key)?, decode_bytes(value)?)))
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(Iter { entries, index: 0 })
    }
}

pub struct Iter {
    entries: Vec<(Vec<u8>, Vec<u8>)>,
    index: usize,
}

impl Iter {
    pub fn next(&mut self) -> Result<Option<(Vec<u8>, Vec<u8>)>, Error> {
        if self.index >= self.entries.len() {
            return Ok(None);
        }

        let entry = self.entries[self.index].clone();
        self.index += 1;
        Ok(Some(entry))
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct PersistedStore {
    columns: Vec<PersistedColumn>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct PersistedColumn {
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

fn ensure_column_count(state: &mut PersistedStore, count: usize) {
    if state.columns.len() < count {
        state
            .columns
            .resize_with(count, PersistedColumn::default);
    }
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
    let mut chars = encoded.as_bytes().chunks_exact(2);
    for pair in &mut chars {
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
            "hex payload contains a non-hex character",
        ))),
    }
}
