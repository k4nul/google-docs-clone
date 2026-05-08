use std::{
    collections::BTreeMap,
    env,
    error::Error as StdError,
    fmt,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    process,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

static NEXT_MAP_SEED: AtomicU64 = AtomicU64::new(1);

const DEFAULT_BASE_DIR_NAME: &str = ".vsdb";
const MAPS_DIR_NAME: &str = "maps";

#[derive(Debug)]
pub enum VsdbError {
    Io(io::Error),
    Serde(serde_json::Error),
    MissingMap(u64),
}

impl fmt::Display for VsdbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Serde(error) => write!(f, "{error}"),
            Self::MissingMap(id) => write!(f, "map metadata `{id}` does not exist"),
        }
    }
}

impl StdError for VsdbError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Serde(error) => Some(error),
            Self::MissingMap(_) => None,
        }
    }
}

impl From<io::Error> for VsdbError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for VsdbError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serde(value)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(bound(
    serialize = "K: Ord + Serialize, V: Serialize",
    deserialize = "K: Ord + DeserializeOwned, V: DeserializeOwned"
))]
struct PersistedMap<K, V> {
    entries: BTreeMap<K, V>,
}

pub struct Mapx<K, V> {
    id: Option<u64>,
    path: Option<PathBuf>,
    entries: BTreeMap<K, V>,
}

impl<K, V> Mapx<K, V>
where
    K: Ord + Clone + Serialize + DeserializeOwned,
    V: Clone + Serialize + DeserializeOwned,
{
    pub fn new() -> Self {
        Self {
            id: None,
            path: None,
            entries: BTreeMap::new(),
        }
    }

    pub fn from_meta(id: u64) -> Result<Self, VsdbError> {
        let path = map_file_path(id)?;
        if !path.exists() {
            return Err(VsdbError::MissingMap(id));
        }

        let bytes = fs::read(&path)?;
        let persisted = serde_json::from_slice::<PersistedMap<K, V>>(&bytes)?;

        Ok(Self {
            id: Some(id),
            path: Some(path),
            entries: persisted.entries,
        })
    }

    pub fn save_meta(&self) -> Result<u64, VsdbError> {
        if let Some(id) = self.id {
            persist_map_file(
                self.path
                    .as_deref()
                    .ok_or_else(|| io::Error::other("missing vsdb path for saved map"))?,
                &self.entries,
            )?;
            return Ok(id);
        }

        let (id, path) = allocate_empty_map_file()?;
        persist_map_file(&path, &self.entries)?;
        Ok(id)
    }

    pub fn get(&self, key: &K) -> Option<V> {
        self.entries.get(key).cloned()
    }

    pub fn insert(&mut self, key: &K, value: &V) {
        self.entries.insert(key.clone(), value.clone());
        let _ = self.persist_if_bound();
    }

    pub fn remove(&mut self, key: &K) {
        self.entries.remove(key);
        let _ = self.persist_if_bound();
    }

    pub fn iter(&self) -> std::vec::IntoIter<(K, V)> {
        self.entries
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn persist_if_bound(&self) -> Result<(), VsdbError> {
        if let Some(path) = self.path.as_deref() {
            persist_map_file(path, &self.entries)?;
        }

        Ok(())
    }
}

pub fn vsdb_flush() {}

fn persist_map_file<K, V>(path: &Path, entries: &BTreeMap<K, V>) -> Result<(), VsdbError>
where
    K: Ord + Clone + Serialize + DeserializeOwned,
    V: Clone + Serialize + DeserializeOwned,
{
    let persisted = PersistedMap {
        entries: entries.clone(),
    };
    let bytes = serde_json::to_vec(&persisted)?;
    let mut file = OpenOptions::new().truncate(true).write(true).open(path)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    Ok(())
}

fn allocate_empty_map_file() -> Result<(u64, PathBuf), VsdbError> {
    let maps_dir = maps_dir()?;
    fs::create_dir_all(&maps_dir)?;

    let pid = process::id() as u64;
    for _ in 0..1024 {
        let counter = NEXT_MAP_SEED.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        let id = (nanos ^ pid.rotate_left(17) ^ counter).max(1);
        let path = maps_dir.join(format!("{id}.json"));

        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut file) => {
                file.write_all(br#"{"entries":{}}"#)?;
                file.sync_all()?;
                return Ok((id, path));
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(VsdbError::Io(error)),
        }
    }

    Err(VsdbError::Io(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "failed to allocate a unique vsdb map id",
    )))
}

fn map_file_path(id: u64) -> Result<PathBuf, VsdbError> {
    Ok(maps_dir()?.join(format!("{id}.json")))
}

fn maps_dir() -> Result<PathBuf, VsdbError> {
    Ok(resolve_base_dir()?.join(MAPS_DIR_NAME))
}

fn resolve_base_dir() -> Result<PathBuf, VsdbError> {
    let base_dir = if let Some(path) = env::var_os("VSDB_BASE_DIR") {
        PathBuf::from(path)
    } else if let Some(home) = env::var_os("HOME") {
        PathBuf::from(home).join(DEFAULT_BASE_DIR_NAME)
    } else {
        env::current_dir()?.join(DEFAULT_BASE_DIR_NAME)
    };

    fs::create_dir_all(&base_dir)?;
    Ok(base_dir)
}
