use std::{
    collections::BTreeMap,
    error::Error as StdError,
    fmt,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    time::Duration,
};

use serde::{Deserialize, Serialize};

const STORE_FILE_NAME: &str = "store.json";
const TEMP_FILE_NAME: &str = "store.json.tmp";

#[derive(Debug)]
pub enum TransientError {
    Io(io::Error),
    Serde(serde_json::Error),
}

impl fmt::Display for TransientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Serde(error) => write!(f, "{error}"),
        }
    }
}

impl StdError for TransientError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Serde(error) => Some(error),
        }
    }
}

impl From<io::Error> for TransientError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for TransientError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serde(value)
    }
}

#[derive(Debug, Clone)]
pub struct DB {
    pub path: PathBuf,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct PersistedStore {
    entries: BTreeMap<String, String>,
}

impl DB {
    pub fn new(path: &Path) -> Result<DB, TransientError> {
        fs::create_dir_all(path)?;
        Ok(Self {
            path: path.to_path_buf(),
        })
    }

    pub fn set(&self, key: &str, val: &str, _ttl: Option<Duration>) -> Result<(), TransientError> {
        let mut store = self.load_store()?;
        store.entries.insert(key.to_owned(), val.to_owned());
        self.persist_store(&store)
    }

    pub fn get(&self, key: &str) -> Result<Option<String>, TransientError> {
        Ok(self.load_store()?.entries.get(key).cloned())
    }

    pub fn remove(&self, key: &str) -> Result<(), TransientError> {
        let mut store = self.load_store()?;
        store.entries.remove(key);
        self.persist_store(&store)
    }

    fn load_store(&self) -> Result<PersistedStore, TransientError> {
        let store_path = self.store_path();
        match fs::read(&store_path) {
            Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(PersistedStore::default()),
            Err(error) => Err(TransientError::Io(error)),
        }
    }

    fn persist_store(&self, store: &PersistedStore) -> Result<(), TransientError> {
        fs::create_dir_all(&self.path)?;

        let temp_path = self.path.join(TEMP_FILE_NAME);
        let final_path = self.store_path();
        let bytes = serde_json::to_vec(store)?;
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&temp_path)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        drop(file);

        fs::rename(&temp_path, &final_path)?;
        sync_dir(&self.path)?;
        Ok(())
    }

    fn store_path(&self) -> PathBuf {
        self.path.join(STORE_FILE_NAME)
    }
}

fn sync_dir(path: &Path) -> Result<(), TransientError> {
    let file = File::open(path)?;
    file.sync_all()?;
    Ok(())
}
