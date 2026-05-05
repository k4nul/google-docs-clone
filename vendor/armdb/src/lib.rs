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

#[derive(Debug, Clone)]
pub struct Config {
    pub enable_fsync: bool,
    pub hints: bool,
    pub reversed: bool,
    pub shard_count: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enable_fsync: false,
            hints: false,
            reversed: false,
            shard_count: 1,
        }
    }
}

#[derive(Debug)]
pub enum DbError {
    Io(io::Error),
    Serde(serde_json::Error),
    InvalidHex(String),
    Poisoned,
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Serde(error) => write!(f, "{error}"),
            Self::InvalidHex(value) => write!(f, "invalid hex payload `{value}`"),
            Self::Poisoned => write!(f, "armdb state mutex poisoned"),
        }
    }
}

impl StdError for DbError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Serde(error) => Some(error),
            Self::InvalidHex(_) | Self::Poisoned => None,
        }
    }
}

impl From<io::Error> for DbError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for DbError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serde(value)
    }
}

#[derive(Debug, Clone)]
pub struct Value(Vec<u8>);

impl Value {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

pub struct VarTree<K>
where
    K: KeyCodec,
{
    path: PathBuf,
    config: Config,
    state: Mutex<BTreeMap<K, Vec<u8>>>,
}

impl<K> VarTree<K>
where
    K: KeyCodec,
{
    pub fn open(path: impl AsRef<Path>, config: Config) -> Result<Self, DbError> {
        let path = path.as_ref().to_path_buf();
        fs::create_dir_all(&path)?;
        let state = load_store::<K>(&path)?;
        Ok(Self {
            path,
            config,
            state: Mutex::new(state),
        })
    }

    pub fn get(&self, key: &K) -> Option<Value> {
        let state = self.state.lock().ok()?;
        state.get(key).cloned().map(Value)
    }

    pub fn put(&self, key: &K, value: &[u8]) -> Result<(), DbError> {
        let mut state = self.state.lock().map_err(|_| DbError::Poisoned)?;
        state.insert(*key, value.to_vec());
        Ok(())
    }

    pub fn delete(&self, key: &K) -> Result<(), DbError> {
        let mut state = self.state.lock().map_err(|_| DbError::Poisoned)?;
        state.remove(key);
        Ok(())
    }

    pub fn flush_buffers(&self) -> Result<(), DbError> {
        let state = self.state.lock().map_err(|_| DbError::Poisoned)?;
        persist_store(&self.path, &self.config, &state)
    }

    pub fn iter(&self) -> std::vec::IntoIter<(K, Value)> {
        let entries = self
            .state
            .lock()
            .map(|state| {
                state
                    .iter()
                    .map(|(key, value)| (*key, Value(value.clone())))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        entries.into_iter()
    }
}

pub trait KeyCodec: Copy + Ord {
    fn encode(self) -> String;
    fn decode(encoded: &str) -> Result<Self, DbError>;
}

impl KeyCodec for [u8; 16] {
    fn encode(self) -> String {
        encode_bytes(&self)
    }

    fn decode(encoded: &str) -> Result<Self, DbError> {
        let bytes = decode_bytes(encoded)?;
        bytes.try_into().map_err(|value: Vec<u8>| {
            DbError::InvalidHex(format!(
                "expected 16-byte key but found {} bytes",
                value.len()
            ))
        })
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct PersistedStore {
    entries: BTreeMap<String, String>,
}

fn load_store<K>(path: &Path) -> Result<BTreeMap<K, Vec<u8>>, DbError>
where
    K: KeyCodec,
{
    let store_path = path.join(STORE_FILE_NAME);
    let persisted = match fs::read(&store_path) {
        Ok(bytes) => serde_json::from_slice::<PersistedStore>(&bytes)?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => PersistedStore::default(),
        Err(error) => return Err(DbError::Io(error)),
    };

    let mut entries = BTreeMap::new();
    for (key, value) in persisted.entries {
        entries.insert(K::decode(&key)?, decode_bytes(&value)?);
    }
    Ok(entries)
}

fn persist_store<K>(
    path: &Path,
    config: &Config,
    state: &BTreeMap<K, Vec<u8>>,
) -> Result<(), DbError>
where
    K: KeyCodec,
{
    let persisted = PersistedStore {
        entries: state
            .iter()
            .map(|(key, value)| (key.encode(), encode_bytes(value)))
            .collect(),
    };
    let bytes = serde_json::to_vec(&persisted)?;
    let temp_path = path.join(TEMP_FILE_NAME);
    let final_path = path.join(STORE_FILE_NAME);

    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&temp_path)?;
    file.write_all(&bytes)?;
    if config.enable_fsync {
        file.sync_all()?;
    }
    drop(file);

    fs::rename(&temp_path, &final_path)?;
    if config.enable_fsync {
        sync_dir(path)?;
    }
    Ok(())
}

fn sync_dir(path: &Path) -> Result<(), DbError> {
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

fn decode_bytes(encoded: &str) -> Result<Vec<u8>, DbError> {
    if encoded.len() % 2 != 0 {
        return Err(DbError::InvalidHex(encoded.to_owned()));
    }

    let mut bytes = Vec::with_capacity(encoded.len() / 2);
    let chars: Vec<_> = encoded.as_bytes().to_vec();
    for chunk in chars.chunks_exact(2) {
        let high = hex_to_nibble(chunk[0])?;
        let low = hex_to_nibble(chunk[1])?;
        bytes.push((high << 4) | low);
    }
    Ok(bytes)
}

fn nibble_to_hex(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 => (b'a' + (value - 10)) as char,
        _ => unreachable!("nibble must be <= 15"),
    }
}

fn hex_to_nibble(value: u8) -> Result<u8, DbError> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        b'A'..=b'F' => Ok(value - b'A' + 10),
        _ => Err(DbError::InvalidHex((value as char).to_string())),
    }
}
