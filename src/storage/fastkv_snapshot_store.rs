use std::{
    fs::{self, File},
    panic::{AssertUnwindSafe, catch_unwind},
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
};

use fastkv::Store;
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

const SNAPSHOT_CATALOG_KEY: &[u8] = b"__catalog__";
const SNAPSHOT_KEY_PREFIX: &str = "snapshot:";
const SHARD_COUNT: usize = 16;

pub struct FastKvSnapshotStore {
    path: PathBuf,
    store: Mutex<Store>,
}

impl FastKvSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = normalize_path(path.into())?;
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            ensure_snapshot_dir(parent)?;
        }

        let store = Store::new(SHARD_COUNT);
        if path.exists() {
            let path_str = path_to_str(&path)?;
            catch_unwind(AssertUnwindSafe(|| store.load_binary(path_str))).map_err(|_| {
                StorageError::Io(format!(
                    "{}: fastkv panicked while loading snapshot store",
                    path.display()
                ))
            })?;
        }

        Ok(Self {
            path,
            store: Mutex::new(store),
        })
    }

    fn snapshot_key(doc_id: &Uuid) -> Vec<u8> {
        format!("{SNAPSHOT_KEY_PREFIX}{doc_id}").into_bytes()
    }

    fn lock_store(&self) -> Result<MutexGuard<'_, Store>, StorageError> {
        self.store.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: fastkv mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn read_catalog_from(store: &Store, path: &Path) -> Result<Vec<Uuid>, StorageError> {
        let Some(payload) = store.get(SNAPSHOT_CATALOG_KEY) else {
            return Ok(Vec::new());
        };

        serde_json::from_slice::<Vec<Uuid>>(&payload).map_err(|_| {
            StorageError::Io(format!(
                "{}: fastkv snapshot catalog is corrupt",
                path.display()
            ))
        })
    }

    fn write_catalog_to(store: &Store, catalog: &[Uuid]) -> Result<(), StorageError> {
        let payload = serde_json::to_vec(catalog).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize fastkv snapshot catalog: {error}"
            ))
        })?;
        store.set(SNAPSHOT_CATALOG_KEY.to_vec(), payload, None);
        Ok(())
    }

    fn persist_locked(&self, store: &Store) -> Result<(), StorageError> {
        let temp_path = temp_path(&self.path);
        if temp_path.exists() {
            fs::remove_file(&temp_path)
                .map_err(|error| StorageError::Io(format!("{}: {error}", temp_path.display())))?;
        }

        let temp_path_str = path_to_str(&temp_path)?;
        catch_unwind(AssertUnwindSafe(|| store.save_binary(temp_path_str))).map_err(|_| {
            StorageError::Io(format!(
                "{}: fastkv panicked while persisting snapshot store",
                self.path.display()
            ))
        })?;

        sync_file(&temp_path)?;
        fs::rename(&temp_path, &self.path).map_err(|error| {
            StorageError::Io(format!(
                "{} -> {}: {error}",
                temp_path.display(),
                self.path.display()
            ))
        })?;
        sync_file(&self.path)?;
        sync_parent_dir(&self.path)
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        payload: &[u8],
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot = serde_json::from_slice::<PersistedSnapshot>(payload)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot: DocumentSnapshot = snapshot.into();

        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }
}

impl SnapshotStore for FastKvSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let store = self.lock_store()?;
        let Some(payload) = store.get(&Self::snapshot_key(doc_id)) else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &payload).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize fastkv snapshot `{doc_id}`: {error}"
            ))
        })?;

        let store = self.lock_store()?;
        store.set(Self::snapshot_key(&doc_id), payload, None);

        let mut catalog = Self::read_catalog_from(&store, &self.path)?;
        if !catalog.contains(&doc_id) {
            catalog.push(doc_id);
            catalog.sort_unstable();
            Self::write_catalog_to(&store, &catalog)?;
        }

        self.persist_locked(&store)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let store = self.lock_store()?;
        store.del(&Self::snapshot_key(doc_id));

        let mut catalog = Self::read_catalog_from(&store, &self.path)?;
        let original_len = catalog.len();
        catalog.retain(|catalog_doc_id| catalog_doc_id != doc_id);
        if catalog.len() != original_len {
            Self::write_catalog_to(&store, &catalog)?;
        }

        self.persist_locked(&store)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let store = self.lock_store()?;
        let catalog = Self::read_catalog_from(&store, &self.path)?;
        let mut documents = Vec::new();

        for doc_id in catalog {
            let Some(payload) = store.get(&Self::snapshot_key(&doc_id)) else {
                tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing fastkv snapshot referenced by catalog"
                );
                continue;
            };

            documents.push(self.deserialize_snapshot(doc_id, &payload)?.document);
        }

        Ok(documents)
    }
}

fn normalize_path(path: PathBuf) -> Result<PathBuf, StorageError> {
    if path.as_os_str().is_empty() {
        return Err(StorageError::Config(
            "SNAPSHOT_FASTKV_PATH cannot be empty when SNAPSHOT_STORE=fastkv".to_owned(),
        ));
    }

    if path
        .parent()
        .is_some_and(|parent| !parent.as_os_str().is_empty())
    {
        Ok(path)
    } else {
        Ok(PathBuf::from(".").join(path))
    }
}

fn temp_path(path: &Path) -> PathBuf {
    let mut temp_path = path.to_path_buf();
    let extension = path
        .extension()
        .map(|extension| format!("{}.tmp", extension.to_string_lossy()))
        .unwrap_or_else(|| "tmp".to_owned());
    temp_path.set_extension(extension);
    temp_path
}

fn path_to_str(path: &Path) -> Result<&str, StorageError> {
    path.to_str().ok_or_else(|| {
        StorageError::Config(
            "SNAPSHOT_FASTKV_PATH must be valid unicode when SNAPSHOT_STORE=fastkv".to_owned(),
        )
    })
}

fn sync_file(path: &Path) -> Result<(), StorageError> {
    File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))
}

fn sync_parent_dir(path: &Path) -> Result<(), StorageError> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));

    File::open(parent)
        .and_then(|file| file.sync_all())
        .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))
}
