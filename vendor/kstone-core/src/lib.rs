use std::{
    collections::{BTreeMap, HashMap},
    fmt,
    fs::{self, File, OpenOptions},
    io::{self, BufRead, BufReader, Write},
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
};

use bytes::Bytes;
use serde::{Deserialize, Serialize};

pub mod index {
    #[derive(Debug, Clone, Default)]
    pub struct TableSchema;

    impl TableSchema {
        pub fn new() -> Self {
            Self
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DatabaseConfig {
    _max_memtable_records: usize,
}

impl DatabaseConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_memtable_records(mut self, value: usize) -> Self {
        self._max_memtable_records = value;
        self
    }
}

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Key(Bytes);

impl Key {
    pub fn new(bytes: Bytes) -> Self {
        Self(bytes)
    }

    fn as_bytes(&self) -> &[u8] {
        self.0.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    B(Bytes),
}

pub type Item = HashMap<String, Value>;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Corrupt(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Corrupt(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum WalOp {
    Put { key: String, item: StoredItem },
    Delete { key: String },
}

type StoredItem = HashMap<String, String>;

struct State {
    data: BTreeMap<Vec<u8>, Item>,
    wal: File,
}

pub struct LsmEngine {
    state: Mutex<State>,
}

impl LsmEngine {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        Self::open_inner(path.as_ref(), false)
    }

    pub fn create_with_config(
        path: impl AsRef<Path>,
        _config: DatabaseConfig,
        _schema: index::TableSchema,
    ) -> Result<Self, Error> {
        Self::open_inner(path.as_ref(), true)
    }

    pub fn get(&self, key: &Key) -> Result<Option<Item>, Error> {
        let state = self.lock_state()?;
        Ok(state.data.get(key.as_bytes()).cloned())
    }

    pub fn put(&self, key: Key, item: Item) -> Result<(), Error> {
        let mut state = self.lock_state()?;
        let encoded_key = encode_bytes(key.as_bytes());
        append_wal(
            &mut state.wal,
            &WalOp::Put {
                key: encoded_key,
                item: encode_item(&item),
            },
        )?;
        state.data.insert(key.as_bytes().to_vec(), item);
        Ok(())
    }

    pub fn delete(&self, key: Key) -> Result<(), Error> {
        let mut state = self.lock_state()?;
        append_wal(
            &mut state.wal,
            &WalOp::Delete {
                key: encode_bytes(key.as_bytes()),
            },
        )?;
        state.data.remove(key.as_bytes());
        Ok(())
    }

    pub fn flush(&self) -> Result<(), Error> {
        let mut state = self.lock_state()?;
        state.wal.flush()?;
        state.wal.sync_data()?;
        Ok(())
    }

    fn open_inner(path: &Path, create: bool) -> Result<Self, Error> {
        fs::create_dir_all(path)?;

        let wal_path = wal_path(path);
        if !create && !wal_path.exists() {
            return Err(Error::Io(io::Error::new(
                io::ErrorKind::NotFound,
                format!("{} does not exist", wal_path.display()),
            )));
        }

        let data = load_wal(&wal_path)?;
        let wal = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(&wal_path)?;

        Ok(Self {
            state: Mutex::new(State { data, wal }),
        })
    }

    fn lock_state(&self) -> Result<MutexGuard<'_, State>, Error> {
        self.state.lock().map_err(|_| {
            Error::Io(io::Error::other(
                "kstone-core in-memory state mutex was poisoned",
            ))
        })
    }
}

fn load_wal(path: &Path) -> Result<BTreeMap<Vec<u8>, Item>, Error> {
    let mut data = BTreeMap::new();
    if !path.exists() {
        return Ok(data);
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    for (line_number, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let op = serde_json::from_str::<WalOp>(&line).map_err(|error| {
            Error::Corrupt(format!(
                "{}:{}: failed to decode kstone wal entry: {error}",
                path.display(),
                line_number + 1
            ))
        })?;

        match op {
            WalOp::Put { key, item } => {
                data.insert(decode_hex(&key)?, decode_item(item)?);
            }
            WalOp::Delete { key } => {
                data.remove(&decode_hex(&key)?);
            }
        }
    }

    Ok(data)
}

fn append_wal(file: &mut File, op: &WalOp) -> Result<(), Error> {
    serde_json::to_writer(&mut *file, op)
        .map_err(|error| Error::Corrupt(format!("failed to encode kstone wal entry: {error}")))?;
    file.write_all(b"\n")?;
    Ok(())
}

fn encode_item(item: &Item) -> StoredItem {
    item.iter()
        .map(|(key, value)| {
            let encoded = match value {
                Value::B(bytes) => encode_bytes(bytes.as_ref()),
            };
            (key.clone(), encoded)
        })
        .collect()
}

fn decode_item(item: StoredItem) -> Result<Item, Error> {
    item.into_iter()
        .map(|(key, value)| Ok((key, Value::B(Bytes::from(decode_hex(&value)?)))))
        .collect()
}

fn wal_path(root: &Path) -> PathBuf {
    root.join("wal.log")
}

fn encode_bytes(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(nibble_to_hex(byte >> 4));
        encoded.push(nibble_to_hex(byte & 0x0f));
    }
    encoded
}

fn decode_hex(value: &str) -> Result<Vec<u8>, Error> {
    let bytes = value.as_bytes();
    if bytes.len() % 2 != 0 {
        return Err(Error::Corrupt(format!(
            "invalid hex payload length {}",
            bytes.len()
        )));
    }

    let mut decoded = Vec::with_capacity(bytes.len() / 2);
    for chunk in bytes.chunks_exact(2) {
        let high = hex_to_nibble(chunk[0])?;
        let low = hex_to_nibble(chunk[1])?;
        decoded.push((high << 4) | low);
    }
    Ok(decoded)
}

fn nibble_to_hex(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 => (b'a' + (value - 10)) as char,
        _ => unreachable!("nibble_to_hex only accepts 0..=15"),
    }
}

fn hex_to_nibble(value: u8) -> Result<u8, Error> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        b'A'..=b'F' => Ok(value - b'A' + 10),
        _ => Err(Error::Corrupt(format!(
            "invalid hex character `{}` in kstone wal payload",
            value as char
        ))),
    }
}
