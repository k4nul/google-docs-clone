use std::{
    collections::BTreeMap,
    error::Error as StdError,
    fmt,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    marker::PhantomData,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};

const STORE_FILE_NAME: &str = "store.json";
const TEMP_FILE_NAME: &str = "store.json.tmp";
const DEFAULT_DATABASE_NAME: &str = "__default__";

pub mod types {
    #[derive(Debug, Clone, Copy)]
    pub struct Bytes;

    #[derive(Debug, Clone, Copy)]
    pub struct Str;
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Serde(serde_json::Error),
    InvalidHex(String),
    Poisoned,
    TooManyDatabases(usize),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Serde(error) => write!(f, "{error}"),
            Self::InvalidHex(value) => write!(f, "invalid hex payload `{value}`"),
            Self::Poisoned => write!(f, "heed state mutex poisoned"),
            Self::TooManyDatabases(limit) => {
                write!(f, "database count exceeds configured max_dbs limit of {limit}")
            }
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Serde(error) => Some(error),
            Self::InvalidHex(_) | Self::Poisoned | Self::TooManyDatabases(_) => None,
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
pub struct EnvOpenOptions {
    map_size: usize,
    max_dbs: usize,
}

impl EnvOpenOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn map_size(&mut self, value: usize) -> &mut Self {
        self.map_size = value;
        self
    }

    pub fn max_dbs(&mut self, value: usize) -> &mut Self {
        self.max_dbs = value;
        self
    }

    pub unsafe fn open(&self, path: impl AsRef<Path>) -> Result<Env, Error> {
        let path = path.as_ref().to_path_buf();
        fs::create_dir_all(&path)?;
        let state = load_store(&path)?;
        if self.max_dbs != 0 && state.databases.len() > self.max_dbs {
            return Err(Error::TooManyDatabases(self.max_dbs));
        }

        Ok(Env {
            inner: Arc::new(EnvInner {
                path,
                max_dbs: self.max_dbs,
                _map_size: self.map_size,
                state: Mutex::new(state),
            }),
        })
    }
}

#[derive(Debug)]
struct EnvInner {
    path: PathBuf,
    max_dbs: usize,
    _map_size: usize,
    state: Mutex<PersistedEnv>,
}

#[derive(Clone, Debug)]
pub struct Env {
    inner: Arc<EnvInner>,
}

impl Env {
    pub fn write_txn(&self) -> Result<RwTxn, Error> {
        let state = self.inner.state.lock().map_err(|_| Error::Poisoned)?.clone();
        Ok(RwTxn {
            env: self.clone(),
            state,
        })
    }

    pub fn read_txn(&self) -> Result<RoTxn, Error> {
        let state = self.inner.state.lock().map_err(|_| Error::Poisoned)?.clone();
        Ok(RoTxn { state })
    }

    pub fn create_database<K, V>(
        &self,
        wtxn: &mut RwTxn,
        name: Option<&str>,
    ) -> Result<Database<K, V>, Error> {
        let name = normalize_database_name(name);
        if !wtxn.state.databases.contains_key(&name) {
            if self.inner.max_dbs != 0 && wtxn.state.databases.len() >= self.inner.max_dbs {
                return Err(Error::TooManyDatabases(self.inner.max_dbs));
            }
            wtxn.state.databases.insert(name.clone(), BTreeMap::new());
        }

        Ok(Database {
            name,
            _marker: PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct RwTxn {
    env: Env,
    state: PersistedEnv,
}

impl RwTxn {
    pub fn commit(self) -> Result<(), Error> {
        persist_store(&self.env.inner.path, &self.state)?;
        let mut shared_state = self
            .env
            .inner
            .state
            .lock()
            .map_err(|_| Error::Poisoned)?;
        *shared_state = self.state;
        Ok(())
    }
}

#[derive(Debug)]
pub struct RoTxn {
    state: PersistedEnv,
}

#[derive(Debug, Clone)]
pub struct Database<K, V> {
    name: String,
    _marker: PhantomData<(K, V)>,
}

impl<K, V> Database<K, V> {
    pub fn get<'txn>(&self, txn: &'txn RoTxn, key: &str) -> Result<Option<&'txn [u8]>, Error> {
        Ok(txn
            .state
            .databases
            .get(&self.name)
            .and_then(|database| database.get(key).map(Vec::as_slice)))
    }

    pub fn put(&self, txn: &mut RwTxn, key: &str, value: &[u8]) -> Result<(), Error> {
        txn.state
            .databases
            .entry(self.name.clone())
            .or_default()
            .insert(key.to_owned(), value.to_vec());
        Ok(())
    }

    pub fn delete(&self, txn: &mut RwTxn, key: &str) -> Result<bool, Error> {
        Ok(txn
            .state
            .databases
            .entry(self.name.clone())
            .or_default()
            .remove(key)
            .is_some())
    }

    pub fn iter<'txn>(&self, txn: &'txn RoTxn) -> Result<Iter<'txn>, Error> {
        let entries = txn
            .state
            .databases
            .get(&self.name)
            .map(|database| {
                database
                    .iter()
                    .map(|(key, value)| (key.as_str(), value.as_slice()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        Ok(Iter { entries, index: 0 })
    }
}

pub struct Iter<'txn> {
    entries: Vec<(&'txn str, &'txn [u8])>,
    index: usize,
}

impl<'txn> Iterator for Iter<'txn> {
    type Item = Result<(&'txn str, &'txn [u8]), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.entries.get(self.index).copied()?;
        self.index += 1;
        Some(Ok(entry))
    }
}

#[derive(Debug, Clone, Default)]
struct PersistedEnv {
    databases: BTreeMap<String, BTreeMap<String, Vec<u8>>>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct StoredEnv {
    databases: BTreeMap<String, BTreeMap<String, String>>,
}

fn normalize_database_name(name: Option<&str>) -> String {
    name.unwrap_or(DEFAULT_DATABASE_NAME).to_owned()
}

fn load_store(path: &Path) -> Result<PersistedEnv, Error> {
    let store_path = path.join(STORE_FILE_NAME);
    let stored = match fs::read(&store_path) {
        Ok(bytes) => serde_json::from_slice::<StoredEnv>(&bytes)?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => StoredEnv::default(),
        Err(error) => return Err(Error::Io(error)),
    };

    let mut databases = BTreeMap::new();
    for (database_name, entries) in stored.databases {
        let mut decoded_entries = BTreeMap::new();
        for (key, value) in entries {
            decoded_entries.insert(key, decode_bytes(&value)?);
        }
        databases.insert(database_name, decoded_entries);
    }

    Ok(PersistedEnv { databases })
}

fn persist_store(path: &Path, state: &PersistedEnv) -> Result<(), Error> {
    fs::create_dir_all(path)?;
    let stored = StoredEnv {
        databases: state
            .databases
            .iter()
            .map(|(database_name, entries)| {
                let encoded_entries = entries
                    .iter()
                    .map(|(key, value)| (key.clone(), encode_bytes(value)))
                    .collect::<BTreeMap<_, _>>();
                (database_name.clone(), encoded_entries)
            })
            .collect(),
    };
    let bytes = serde_json::to_vec(&stored)?;
    let temp_path = path.join(TEMP_FILE_NAME);
    let final_path = path.join(STORE_FILE_NAME);

    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&temp_path)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    drop(file);

    fs::rename(&temp_path, &final_path)?;
    File::open(path)?.sync_all()?;
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
        return Err(Error::InvalidHex(encoded.to_owned()));
    }

    let mut bytes = Vec::with_capacity(encoded.len() / 2);
    let chars = encoded.as_bytes();
    let mut index = 0;
    while index < chars.len() {
        let high = hex_to_nibble(chars[index])?;
        let low = hex_to_nibble(chars[index + 1])?;
        bytes.push((high << 4) | low);
        index += 2;
    }
    Ok(bytes)
}

fn nibble_to_hex(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 => (b'a' + (value - 10)) as char,
        _ => unreachable!("nibble must fit in a hex digit"),
    }
}

fn hex_to_nibble(value: u8) -> Result<u8, Error> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        b'A'..=b'F' => Ok(value - b'A' + 10),
        _ => Err(Error::InvalidHex((value as char).to_string())),
    }
}
