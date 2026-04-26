use std::{
    cell::RefCell,
    collections::BTreeMap,
    error::Error as StdError,
    fmt,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};

const STORE_FILE_NAME: &str = "store.json";
const TEMP_FILE_NAME: &str = "store.json.tmp";

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Serde(serde_json::Error),
    InvalidHex(String),
    Poisoned,
    ReadOnlyTree,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Serde(error) => write!(f, "{error}"),
            Self::InvalidHex(value) => write!(f, "invalid hex payload `{value}`"),
            Self::Poisoned => write!(f, "canopydb state mutex poisoned"),
            Self::ReadOnlyTree => write!(f, "tree mutations require a write transaction"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Serde(error) => Some(error),
            Self::InvalidHex(_) | Self::Poisoned | Self::ReadOnlyTree => None,
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

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Bytes(Vec<u8>);

impl Bytes {
    fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub struct Database {
    inner: Arc<DatabaseInner>,
}

#[derive(Debug)]
struct DatabaseInner {
    path: PathBuf,
    state: Mutex<PersistedDatabase>,
}

impl Database {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref().to_path_buf();
        fs::create_dir_all(&path)?;
        let state = load_store(&path)?;

        Ok(Self {
            inner: Arc::new(DatabaseInner {
                path,
                state: Mutex::new(state),
            }),
        })
    }

    pub fn begin_read(&self) -> Result<ReadTransaction, Error> {
        let state = self.inner.state.lock().map_err(|_| Error::Poisoned)?.clone();
        Ok(ReadTransaction { state })
    }

    pub fn begin_write(&self) -> Result<WriteTransaction, Error> {
        let state = self.inner.state.lock().map_err(|_| Error::Poisoned)?.clone();
        Ok(WriteTransaction {
            database: self.clone(),
            state: RefCell::new(state),
        })
    }
}

#[derive(Debug)]
pub struct ReadTransaction {
    state: PersistedDatabase,
}

impl ReadTransaction {
    pub fn get_tree(&self, name: &[u8]) -> Result<Option<Tree<'_>>, Error> {
        if self.state.trees.contains_key(name) {
            Ok(Some(Tree {
                name: name.to_vec(),
                state: TreeState::Read(&self.state),
            }))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug)]
pub struct WriteTransaction {
    database: Database,
    state: RefCell<PersistedDatabase>,
}

impl WriteTransaction {
    pub fn get_tree(&self, name: &[u8]) -> Result<Option<Tree<'_>>, Error> {
        if self.state.borrow().trees.contains_key(name) {
            Ok(Some(Tree {
                name: name.to_vec(),
                state: TreeState::Write(&self.state),
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_or_create_tree(&self, name: &[u8]) -> Result<Tree<'_>, Error> {
        self.state
            .borrow_mut()
            .trees
            .entry(name.to_vec())
            .or_default();
        Ok(Tree {
            name: name.to_vec(),
            state: TreeState::Write(&self.state),
        })
    }

    pub fn commit_with(self, sync: bool) -> Result<u64, Error> {
        let state = self.state.into_inner();
        persist_store(&self.database.inner.path, &state, sync)?;
        let mut shared_state = self
            .database
            .inner
            .state
            .lock()
            .map_err(|_| Error::Poisoned)?;
        *shared_state = state;
        Ok(1)
    }
}

#[derive(Debug)]
pub struct Tree<'a> {
    name: Vec<u8>,
    state: TreeState<'a>,
}

#[derive(Debug)]
enum TreeState<'a> {
    Read(&'a PersistedDatabase),
    Write(&'a RefCell<PersistedDatabase>),
}

impl<'a> Tree<'a> {
    pub fn get(&self, key: &[u8]) -> Result<Option<Bytes>, Error> {
        Ok(match &self.state {
            TreeState::Read(state) => state
                .trees
                .get(&self.name)
                .and_then(|tree| tree.get(key).cloned())
                .map(Bytes::new),
            TreeState::Write(state) => state
                .borrow()
                .trees
                .get(&self.name)
                .and_then(|tree| tree.get(key).cloned())
                .map(Bytes::new),
        })
    }

    pub fn insert(&mut self, key: &[u8], value: &[u8]) -> Result<(), Error> {
        let TreeState::Write(state) = &self.state else {
            return Err(Error::ReadOnlyTree);
        };

        state
            .borrow_mut()
            .trees
            .entry(self.name.clone())
            .or_default()
            .insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    pub fn delete(&mut self, key: &[u8]) -> Result<bool, Error> {
        let TreeState::Write(state) = &self.state else {
            return Err(Error::ReadOnlyTree);
        };

        Ok(state
            .borrow_mut()
            .trees
            .entry(self.name.clone())
            .or_default()
            .remove(key)
            .is_some())
    }

    pub fn iter(&self) -> Result<Iter, Error> {
        let entries = match &self.state {
            TreeState::Read(state) => state
                .trees
                .get(&self.name)
                .map(|tree| {
                    tree.iter()
                        .map(|(key, value)| (Bytes::new(key.clone()), Bytes::new(value.clone())))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            TreeState::Write(state) => state
                .borrow()
                .trees
                .get(&self.name)
                .map(|tree| {
                    tree.iter()
                        .map(|(key, value)| (Bytes::new(key.clone()), Bytes::new(value.clone())))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
        };

        Ok(Iter { entries, index: 0 })
    }
}

pub struct Iter {
    entries: Vec<(Bytes, Bytes)>,
    index: usize,
}

impl Iterator for Iter {
    type Item = Result<(Bytes, Bytes), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.entries.get(self.index).cloned()?;
        self.index += 1;
        Some(Ok(entry))
    }
}

#[derive(Clone, Debug, Default)]
struct PersistedDatabase {
    trees: BTreeMap<Vec<u8>, BTreeMap<Vec<u8>, Vec<u8>>>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct StoredDatabase {
    trees: BTreeMap<String, BTreeMap<String, String>>,
}

fn load_store(path: &Path) -> Result<PersistedDatabase, Error> {
    let store_path = path.join(STORE_FILE_NAME);
    let stored = match fs::read(&store_path) {
        Ok(bytes) => serde_json::from_slice::<StoredDatabase>(&bytes)?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => StoredDatabase::default(),
        Err(error) => return Err(Error::Io(error)),
    };

    let mut trees = BTreeMap::new();
    for (tree_name, entries) in stored.trees {
        let mut decoded_entries = BTreeMap::new();
        for (key, value) in entries {
            decoded_entries.insert(decode_bytes(&key)?, decode_bytes(&value)?);
        }
        trees.insert(decode_bytes(&tree_name)?, decoded_entries);
    }

    Ok(PersistedDatabase { trees })
}

fn persist_store(path: &Path, state: &PersistedDatabase, sync: bool) -> Result<(), Error> {
    fs::create_dir_all(path)?;
    let stored = StoredDatabase {
        trees: state
            .trees
            .iter()
            .map(|(tree_name, entries)| {
                let encoded_entries = entries
                    .iter()
                    .map(|(key, value)| (encode_bytes(key), encode_bytes(value)))
                    .collect::<BTreeMap<_, _>>();
                (encode_bytes(tree_name), encoded_entries)
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
    if sync {
        file.sync_all()?;
    }
    drop(file);

    fs::rename(&temp_path, &final_path)?;
    if sync {
        File::open(path)?.sync_all()?;
    }

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

fn decode_bytes(value: &str) -> Result<Vec<u8>, Error> {
    if value.len() % 2 != 0 {
        return Err(Error::InvalidHex(value.to_owned()));
    }

    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(value.len() / 2);
    let mut index = 0;
    while index < bytes.len() {
        let high = hex_to_nibble(bytes[index]).ok_or_else(|| Error::InvalidHex(value.to_owned()))?;
        let low =
            hex_to_nibble(bytes[index + 1]).ok_or_else(|| Error::InvalidHex(value.to_owned()))?;
        decoded.push((high << 4) | low);
        index += 2;
    }
    Ok(decoded)
}

fn nibble_to_hex(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 => (b'a' + value - 10) as char,
        _ => unreachable!("nibble must be <= 15"),
    }
}

fn hex_to_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}
