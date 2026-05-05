use std::{
    collections::BTreeMap,
    error::Error as StdError,
    fmt,
    fs::{self, File, OpenOptions},
    io::{self as std_io, Write},
    marker::PhantomData,
    ops::RangeFull,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};

const STORE_FILE_NAME: &str = "store.json";
const TEMP_FILE_NAME: &str = "store.json.tmp";

pub mod io {
    pub mod fs {
        #[derive(Clone, Copy, Debug, Default)]
        pub struct StdFile;
    }
}

pub mod tree {
    pub trait Root {
        fn name(&self) -> &str;
    }

    #[derive(Clone, Debug)]
    pub struct Unversioned {
        name: String,
    }

    impl Unversioned {
        pub fn tree(name: &str) -> Self {
            Self {
                name: name.to_owned(),
            }
        }
    }

    impl Root for Unversioned {
        fn name(&self) -> &str {
            &self.name
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ArcBytes(Arc<[u8]>);

impl ArcBytes {
    fn new(bytes: Vec<u8>) -> Self {
        Self(Arc::from(bytes))
    }
}

impl AsRef<[u8]> for ArcBytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<Vec<u8>> for ArcBytes {
    fn from(value: Vec<u8>) -> Self {
        Self::new(value)
    }
}

#[derive(Debug)]
pub enum Error {
    Io(std_io::Error),
    Serde(serde_json::Error),
    InvalidHex(String),
    Poisoned,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Serde(error) => write!(f, "{error}"),
            Self::InvalidHex(value) => write!(f, "invalid hex payload `{value}`"),
            Self::Poisoned => write!(f, "nebari state mutex poisoned"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Serde(error) => Some(error),
            Self::InvalidHex(_) | Self::Poisoned => None,
        }
    }
}

impl From<std_io::Error> for Error {
    fn from(value: std_io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Serde(value)
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    path: PathBuf,
}

impl Config {
    pub fn default_for(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn open(&self) -> Result<Roots, Error> {
        fs::create_dir_all(&self.path)?;
        let state = load_store(&self.path)?;
        Ok(Roots {
            inner: Arc::new(RootsInner {
                path: self.path.clone(),
                state: Mutex::new(state),
            }),
        })
    }
}

#[derive(Debug)]
struct RootsInner {
    path: PathBuf,
    state: Mutex<PersistedStore>,
}

#[derive(Clone, Debug)]
pub struct Roots {
    inner: Arc<RootsInner>,
}

impl Roots {
    pub fn tree<R>(&self, root: R) -> Result<Tree<R, io::fs::StdFile>, Error>
    where
        R: tree::Root + Clone,
    {
        {
            let mut state = self.inner.state.lock().map_err(|_| Error::Poisoned)?;
            state.trees.entry(root.name().to_owned()).or_default();
            persist_store(&self.inner.path, &state)?;
        }

        Ok(Tree {
            roots: self.inner.clone(),
            name: root.name().to_owned(),
            _marker: PhantomData,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Tree<R = tree::Unversioned, F = io::fs::StdFile> {
    roots: Arc<RootsInner>,
    name: String,
    _marker: PhantomData<(R, F)>,
}

impl<R, F> Tree<R, F> {
    pub fn get(&self, key: &[u8]) -> Result<Option<ArcBytes>, Error> {
        let state = self.roots.state.lock().map_err(|_| Error::Poisoned)?;
        Ok(state
            .trees
            .get(&self.name)
            .and_then(|tree| tree.get(key).cloned())
            .map(ArcBytes::new))
    }

    pub fn set(&self, key: ArcBytes, value: ArcBytes) -> Result<(), Error> {
        let mut state = self.roots.state.lock().map_err(|_| Error::Poisoned)?;
        state
            .trees
            .entry(self.name.clone())
            .or_default()
            .insert(key.as_ref().to_vec(), value.as_ref().to_vec());
        persist_store(&self.roots.path, &state)?;
        Ok(())
    }

    pub fn remove(&self, key: &[u8]) -> Result<(), Error> {
        let mut state = self.roots.state.lock().map_err(|_| Error::Poisoned)?;
        state
            .trees
            .entry(self.name.clone())
            .or_default()
            .remove(key);
        persist_store(&self.roots.path, &state)?;
        Ok(())
    }

    pub fn get_range(&self, _range: &RangeFull) -> Result<RangeIter, Error> {
        let state = self.roots.state.lock().map_err(|_| Error::Poisoned)?;
        let entries = state
            .trees
            .get(&self.name)
            .map(|tree| {
                tree.iter()
                    .map(|(key, value)| (ArcBytes::new(key.clone()), ArcBytes::new(value.clone())))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        Ok(RangeIter { entries, index: 0 })
    }
}

pub struct RangeIter {
    entries: Vec<(ArcBytes, ArcBytes)>,
    index: usize,
}

impl Iterator for RangeIter {
    type Item = (ArcBytes, ArcBytes);

    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.entries.get(self.index).cloned()?;
        self.index += 1;
        Some(entry)
    }
}

#[derive(Clone, Debug, Default)]
struct PersistedStore {
    trees: BTreeMap<String, BTreeMap<Vec<u8>, Vec<u8>>>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct StoredStore {
    trees: BTreeMap<String, BTreeMap<String, String>>,
}

fn load_store(path: &Path) -> Result<PersistedStore, Error> {
    let store_path = path.join(STORE_FILE_NAME);
    let stored = match fs::read(&store_path) {
        Ok(bytes) => serde_json::from_slice::<StoredStore>(&bytes)?,
        Err(error) if error.kind() == std_io::ErrorKind::NotFound => StoredStore::default(),
        Err(error) => return Err(Error::Io(error)),
    };

    let mut trees = BTreeMap::new();
    for (tree_name, entries) in stored.trees {
        let mut decoded_entries = BTreeMap::new();
        for (key, value) in entries {
            decoded_entries.insert(decode_bytes(&key)?, decode_bytes(&value)?);
        }
        trees.insert(tree_name, decoded_entries);
    }

    Ok(PersistedStore { trees })
}

fn persist_store(path: &Path, state: &PersistedStore) -> Result<(), Error> {
    fs::create_dir_all(path)?;
    let stored = StoredStore {
        trees: state
            .trees
            .iter()
            .map(|(tree_name, entries)| {
                let encoded_entries = entries
                    .iter()
                    .map(|(key, value)| (encode_bytes(key), encode_bytes(value)))
                    .collect::<BTreeMap<_, _>>();
                (tree_name.clone(), encoded_entries)
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
        let high = hex_to_nibble(chars[index]).ok_or_else(|| Error::InvalidHex(encoded.to_owned()))?;
        let low =
            hex_to_nibble(chars[index + 1]).ok_or_else(|| Error::InvalidHex(encoded.to_owned()))?;
        bytes.push((high << 4) | low);
        index += 2;
    }
    Ok(bytes)
}

fn nibble_to_hex(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 => (b'a' + value - 10) as char,
        _ => unreachable!("nibble must fit in a hex digit"),
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
