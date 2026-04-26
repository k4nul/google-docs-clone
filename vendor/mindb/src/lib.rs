use std::{
    collections::BTreeMap,
    error::Error as StdError,
    fmt,
    fs::{self, File, OpenOptions},
    io::{self, BufRead, BufReader, Write},
    path::{Path, PathBuf},
    sync::Mutex,
};

use serde::{Deserialize, Serialize};

const STORE_FILE_NAME: &str = "store.json";
const WAL_FILE_NAME: &str = "wal.log";
const MANIFEST_FILE_NAME: &str = "manifest.json";
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

pub mod storage {
    #[derive(Debug, Clone, Copy, Default)]
    pub enum CompressionCodec {
        #[default]
        None,
    }
}

pub mod write {
    pub mod memtable {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct MemTableEntry {
            pub sequence: u64,
            pub tombstone: bool,
            pub value: Box<[u8]>,
        }
    }
}

pub mod db {
    use super::*;
    use crate::storage::CompressionCodec;

    #[derive(Debug, Clone)]
    pub struct DatabaseOptions {
        pub data_dir: PathBuf,
        pub wal_direct_io: bool,
        pub compression: CompressionCodec,
        pub wal_max_batch_ops: usize,
    }

    impl DatabaseOptions {
        pub fn new(path: impl AsRef<Path>) -> Self {
            Self {
                data_dir: path.as_ref().to_path_buf(),
                wal_direct_io: false,
                compression: CompressionCodec::None,
                wal_max_batch_ops: 1,
            }
        }
    }

    #[derive(Debug)]
    pub struct Database {
        path: PathBuf,
        state: Mutex<StoreState>,
    }

    impl Database {
        pub fn open(options: DatabaseOptions) -> Result<Self, Error> {
            fs::create_dir_all(&options.data_dir)?;
            let mut store = load_store(&options.data_dir)?;
            let wal = load_wal_records(&options.data_dir)?;
            let next_sequence = apply_wal(&mut store, &wal)?;
            persist_store(&options.data_dir, &store)?;
            ensure_manifest(&options.data_dir)?;

            Ok(Self {
                path: options.data_dir,
                state: Mutex::new(StoreState {
                    entries: store.entries,
                    next_sequence,
                }),
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

        pub fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), Error> {
            let mut state = self.state.lock().map_err(|_| Error::Poisoned)?;
            let sequence = state.next_sequence;
            state.next_sequence += 1;
            state.entries.insert(encode_bytes(&key), encode_bytes(&value));
            append_wal_record(
                &self.path,
                &WalRecord {
                    key: encode_bytes(&key),
                    value: Some(encode_bytes(&value)),
                    sequence,
                    tombstone: false,
                },
            )
        }

        pub fn delete(&self, key: Vec<u8>) -> Result<(), Error> {
            let mut state = self.state.lock().map_err(|_| Error::Poisoned)?;
            let sequence = state.next_sequence;
            state.next_sequence += 1;
            let encoded_key = encode_bytes(&key);
            state.entries.remove(&encoded_key);
            append_wal_record(
                &self.path,
                &WalRecord {
                    key: encoded_key,
                    value: None,
                    sequence,
                    tombstone: true,
                },
            )
        }

        pub fn sync(&self) -> Result<(), Error> {
            let state = self.state.lock().map_err(|_| Error::Poisoned)?;
            persist_store(
                &self.path,
                &PersistedStore {
                    entries: state.entries.clone(),
                },
            )
        }
    }

    #[derive(Debug)]
    struct StoreState {
        entries: BTreeMap<String, String>,
        next_sequence: u64,
    }
}

pub mod recovery {
    use super::*;
    use crate::write::memtable::MemTableEntry;

    #[derive(Debug, Clone)]
    pub struct RecoveryOptions {
        wal_path: PathBuf,
        manifest_path: PathBuf,
    }

    impl RecoveryOptions {
        pub fn new(wal_path: impl Into<PathBuf>, manifest_path: impl Into<PathBuf>) -> Self {
            Self {
                wal_path: wal_path.into(),
                manifest_path: manifest_path.into(),
            }
        }
    }

    pub struct RecoveryManager {
        options: RecoveryOptions,
    }

    impl RecoveryManager {
        pub fn new(options: RecoveryOptions) -> Self {
            Self { options }
        }

        pub fn recover(&self) -> Result<RecoveryOutcome, Error> {
            if !self.options.manifest_path.exists() {
                return Ok(RecoveryOutcome {
                    memtables: Vec::new(),
                });
            }

            let records = load_wal_records_from_path(&self.options.wal_path)?;
            let mut entries = BTreeMap::new();
            for record in records {
                let value = match record.value {
                    Some(value) => decode_bytes(&value)?,
                    None => Vec::new(),
                };
                entries.insert(
                    record.key,
                    MemTableEntry {
                        sequence: record.sequence,
                        tombstone: record.tombstone,
                        value: value.into_boxed_slice(),
                    },
                );
            }

            Ok(RecoveryOutcome {
                memtables: vec![MemTable { entries }],
            })
        }
    }

    pub struct RecoveryOutcome {
        pub memtables: Vec<MemTable>,
    }

    pub struct MemTable {
        entries: BTreeMap<String, MemTableEntry>,
    }

    impl MemTable {
        pub fn get(&self, key: &[u8]) -> Option<MemTableEntry> {
            self.entries.get(&encode_bytes(key)).cloned()
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct PersistedStore {
    entries: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WalRecord {
    key: String,
    value: Option<String>,
    sequence: u64,
    tombstone: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Manifest {
    version: u8,
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

fn load_wal_records(path: &Path) -> Result<Vec<WalRecord>, Error> {
    load_wal_records_from_path(&path.join(WAL_FILE_NAME))
}

fn load_wal_records_from_path(path: &Path) -> Result<Vec<WalRecord>, Error> {
    match File::open(path) {
        Ok(file) => {
            let mut records = Vec::new();
            for line in BufReader::new(file).lines() {
                let line = line?;
                if line.is_empty() {
                    continue;
                }
                records.push(serde_json::from_str(&line)?);
            }
            Ok(records)
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(Error::Io(error)),
    }
}

fn append_wal_record(path: &Path, record: &WalRecord) -> Result<(), Error> {
    ensure_manifest(path)?;
    let wal_path = path.join(WAL_FILE_NAME);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&wal_path)?;
    serde_json::to_writer(&mut file, record)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    Ok(())
}

fn apply_wal(store: &mut PersistedStore, records: &[WalRecord]) -> Result<u64, Error> {
    let mut next_sequence = 0;
    for record in records {
        next_sequence = next_sequence.max(record.sequence.saturating_add(1));
        if record.tombstone {
            store.entries.remove(&record.key);
        } else if let Some(value) = &record.value {
            store.entries.insert(record.key.clone(), value.clone());
        } else {
            return Err(Error::Io(io::Error::new(
                io::ErrorKind::InvalidData,
                "non-tombstone wal record is missing a value",
            )));
        }
    }
    Ok(next_sequence)
}

fn ensure_manifest(path: &Path) -> Result<(), Error> {
    fs::create_dir_all(path)?;
    let manifest_path = path.join(MANIFEST_FILE_NAME);
    if manifest_path.exists() {
        return Ok(());
    }

    let temp_path = path.join("manifest.json.tmp");
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&temp_path)?;
    serde_json::to_writer(&mut file, &Manifest { version: 1 })?;
    file.sync_all()?;
    drop(file);
    fs::rename(&temp_path, &manifest_path)?;
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
