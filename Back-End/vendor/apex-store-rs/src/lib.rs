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
pub struct LsmError(String);

impl fmt::Display for LsmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl StdError for LsmError {}

impl From<io::Error> for LsmError {
    fn from(value: io::Error) -> Self {
        Self(value.to_string())
    }
}

impl From<serde_json::Error> for LsmError {
    fn from(value: serde_json::Error) -> Self {
        Self(value.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct LsmConfig {
    dir_path: PathBuf,
}

impl LsmConfig {
    pub fn builder() -> LsmConfigBuilder {
        LsmConfigBuilder::default()
    }
}

#[derive(Debug, Default, Clone)]
pub struct LsmConfigBuilder {
    dir_path: Option<PathBuf>,
}

impl LsmConfigBuilder {
    pub fn dir_path(mut self, path: impl AsRef<Path>) -> Self {
        self.dir_path = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn memtable_max_size(self, _value: usize) -> Self {
        self
    }

    pub fn block_size(self, _value: usize) -> Self {
        self
    }

    pub fn block_cache_size_mb(self, _value: usize) -> Self {
        self
    }

    pub fn sparse_index_interval(self, _value: usize) -> Self {
        self
    }

    pub fn bloom_false_positive_rate(self, _value: f64) -> Self {
        self
    }

    pub fn build(self) -> Result<LsmConfig, LsmError> {
        let Some(dir_path) = self.dir_path else {
            return Err(LsmError("dir_path is required".to_owned()));
        };
        Ok(LsmConfig { dir_path })
    }
}

#[derive(Debug)]
pub struct LsmEngine {
    path: PathBuf,
    state: Mutex<PersistedStore>,
}

impl LsmEngine {
    pub fn new(config: LsmConfig) -> Result<Self, LsmError> {
        let state = load_store(&config.dir_path)?;
        if !config.dir_path.exists() {
            fs::create_dir_all(&config.dir_path)?;
            persist_store(&config.dir_path, &state)?;
        }

        Ok(Self {
            path: config.dir_path,
            state: Mutex::new(state),
        })
    }

    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>, LsmError> {
        let state = self
            .state
            .lock()
            .map_err(|_| LsmError("apex_store mutex was poisoned".to_owned()))?;
        Ok(state
            .entries
            .get(key)
            .map(|value| decode_bytes(value))
            .transpose()?)
    }

    pub fn set(&self, key: String, value: Vec<u8>) -> Result<(), LsmError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| LsmError("apex_store mutex was poisoned".to_owned()))?;
        state.entries.insert(key, encode_bytes(&value));
        persist_store(&self.path, &state)
    }

    pub fn delete(&self, key: String) -> Result<(), LsmError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| LsmError("apex_store mutex was poisoned".to_owned()))?;
        state.entries.remove(&key);
        persist_store(&self.path, &state)
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct PersistedStore {
    entries: BTreeMap<String, String>,
}

fn load_store(path: &Path) -> Result<PersistedStore, LsmError> {
    let store_path = path.join(STORE_FILE_NAME);
    match fs::read(&store_path) {
        Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(PersistedStore::default()),
        Err(error) => Err(error.into()),
    }
}

fn persist_store(path: &Path, state: &PersistedStore) -> Result<(), LsmError> {
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
    let dir = File::open(path)?;
    dir.sync_all()?;
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

fn decode_bytes(encoded: &str) -> Result<Vec<u8>, LsmError> {
    if encoded.len() % 2 != 0 {
        return Err(LsmError("hex payload has odd length".to_owned()));
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

fn hex_to_nibble(byte: u8) -> Result<u8, LsmError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(LsmError("invalid hex payload".to_owned())),
    }
}
