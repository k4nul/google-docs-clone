use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use rustcask::Rustcask;
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOT_CATALOG_KEY: &[u8] = b"__catalog__";

pub struct RustcaskSnapshotStore {
    path: PathBuf,
    store: Mutex<Rustcask>,
}

impl RustcaskSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_RUSTCASK_PATH cannot be empty when SNAPSHOT_STORE=rustcask".to_owned(),
            ));
        }

        fs::create_dir_all(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;
        let store = Rustcask::builder()
            .set_sync_mode(true)
            .open(&path)
            .map_err(|error| Self::map_open_error(&path, error))?;

        Ok(Self {
            path,
            store: Mutex::new(store),
        })
    }

    fn lock_store(&self) -> Result<MutexGuard<'_, Rustcask>, StorageError> {
        self.store.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: rustcask store mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn map_open_error(path: &std::path::Path, error: rustcask::error::OpenError) -> StorageError {
        StorageError::Io(format!(
            "{}: failed to open rustcask snapshot store: {error}",
            path.display()
        ))
    }

    fn map_set_error(
        path: &std::path::Path,
        error: rustcask::error::SetError,
        operation: &str,
    ) -> StorageError {
        StorageError::Io(format!(
            "{}: failed to {operation}: {error}",
            path.display()
        ))
    }

    fn map_get_error(
        path: &std::path::Path,
        error: rustcask::error::GetError<'_>,
        operation: &str,
    ) -> StorageError {
        StorageError::Io(format!(
            "{}: failed to {operation}: {error}",
            path.display()
        ))
    }

    fn map_remove_error(
        path: &std::path::Path,
        error: rustcask::error::RemoveError,
        operation: &str,
    ) -> StorageError {
        StorageError::Io(format!(
            "{}: failed to {operation}: {error}",
            path.display()
        ))
    }

    fn load_catalog(&self, store: &mut Rustcask) -> Result<Vec<String>, StorageError> {
        let Some(bytes) = store
            .get(&SNAPSHOT_CATALOG_KEY.to_vec())
            .map_err(|error| Self::map_get_error(&self.path, error, "read rustcask catalog"))?
        else {
            return Ok(Vec::new());
        };

        serde_json::from_slice::<Vec<String>>(&bytes).map_err(|_| {
            StorageError::Io(format!(
                "{}: snapshot catalog is corrupt",
                self.path.display()
            ))
        })
    }

    fn save_catalog(&self, store: &mut Rustcask, catalog: &[String]) -> Result<(), StorageError> {
        let bytes = serde_json::to_vec(catalog)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        store
            .set(SNAPSHOT_CATALOG_KEY.to_vec(), bytes)
            .map_err(|error| Self::map_set_error(&self.path, error, "write rustcask catalog"))
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        bytes: &[u8],
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot = serde_json::from_slice::<PersistedSnapshot>(bytes)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot: DocumentSnapshot = snapshot.into();

        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }
}

impl SnapshotStore for RustcaskSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let mut store = self.lock_store()?;
        let key = doc_id.to_string().into_bytes();
        let snapshot = store
            .get(&key)
            .map_err(|error| Self::map_get_error(&self.path, error, "read rustcask snapshot"))?;
        let Some(snapshot) = snapshot else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &snapshot).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let mut store = self.lock_store()?;
        let doc_id = snapshot.document.id;
        let key = doc_id.to_string();
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize rustcask snapshot `{doc_id}`: {error}"
            ))
        })?;
        let mut catalog = self.load_catalog(&mut store)?;

        store
            .set(key.as_bytes().to_vec(), bytes)
            .map_err(|error| Self::map_set_error(&self.path, error, "write rustcask snapshot"))?;

        if !catalog.iter().any(|entry| entry == &key) {
            catalog.push(key);
            catalog.sort();
        }

        self.save_catalog(&mut store, &catalog)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut store = self.lock_store()?;
        let key = doc_id.to_string();
        let mut catalog = self.load_catalog(&mut store)?;

        store.remove(key.as_bytes().to_vec()).map_err(|error| {
            Self::map_remove_error(&self.path, error, "delete rustcask snapshot")
        })?;
        catalog.retain(|entry| entry != &key);

        self.save_catalog(&mut store, &catalog)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut store = self.lock_store()?;
        let catalog = self.load_catalog(&mut store)?;
        let mut documents = Vec::new();

        for doc_id_key in catalog {
            let Ok(doc_id) = Uuid::parse_str(&doc_id_key) else {
                continue;
            };

            match store
                .get(&doc_id_key.clone().into_bytes())
                .map_err(|error| Self::map_get_error(&self.path, error, "read rustcask snapshot"))?
            {
                Some(bytes) => match self.deserialize_snapshot(doc_id, &bytes) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping corrupt rustcask snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                },
                None => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing rustcask snapshot while building document catalog"
                ),
            }
        }

        Ok(documents)
    }
}
