use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use epoch_db::DB;
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOT_CATALOG_KEY: &str = "__catalog__";

pub struct EpochDbSnapshotStore {
    path: PathBuf,
    store: Mutex<DB>,
}

impl EpochDbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_EPOCH_DB_PATH cannot be empty when SNAPSHOT_STORE=epoch_db".to_owned(),
            ));
        }

        fs::create_dir_all(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;
        let store = DB::new(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            store: Mutex::new(store),
        })
    }

    fn lock_store(&self) -> Result<MutexGuard<'_, DB>, StorageError> {
        self.store.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: epoch-db snapshot store mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn map_error(&self, operation: &str, error: impl std::fmt::Display) -> StorageError {
        StorageError::Io(format!(
            "{}: failed to {operation}: {error}",
            self.path.display()
        ))
    }

    fn load_catalog(&self, store: &DB) -> Result<Vec<String>, StorageError> {
        let Some(catalog) = store
            .get(SNAPSHOT_CATALOG_KEY)
            .map_err(|error| self.map_error("read epoch-db catalog", error))?
        else {
            return Ok(Vec::new());
        };

        serde_json::from_str(&catalog).map_err(|_| {
            StorageError::Io(format!(
                "{}: epoch-db snapshot catalog is corrupt",
                self.path.display()
            ))
        })
    }

    fn save_catalog(&self, store: &DB, catalog: &[String]) -> Result<(), StorageError> {
        let payload = serde_json::to_string(catalog)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        store
            .set(SNAPSHOT_CATALOG_KEY, &payload, None)
            .map_err(|error| self.map_error("write epoch-db catalog", error))
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        payload: &str,
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot = serde_json::from_str::<PersistedSnapshot>(payload)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot: DocumentSnapshot = snapshot.into();

        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }
}

impl SnapshotStore for EpochDbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let store = self.lock_store()?;
        let key = doc_id.to_string();
        let snapshot = store
            .get(&key)
            .map_err(|error| self.map_error("read epoch-db snapshot", error))?;
        let Some(snapshot) = snapshot else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &snapshot).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let store = self.lock_store()?;
        let doc_id = snapshot.document.id;
        let key = doc_id.to_string();
        let payload =
            serde_json::to_string(&PersistedSnapshot::from(snapshot)).map_err(|error| {
                StorageError::Io(format!(
                    "failed to serialize epoch-db snapshot `{doc_id}`: {error}"
                ))
            })?;
        let mut catalog = self.load_catalog(&store)?;

        store
            .set(&key, &payload, None)
            .map_err(|error| self.map_error("write epoch-db snapshot", error))?;

        if !catalog.iter().any(|entry| entry == &key) {
            catalog.push(key);
            catalog.sort();
        }

        self.save_catalog(&store, &catalog)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let store = self.lock_store()?;
        let key = doc_id.to_string();
        let mut catalog = self.load_catalog(&store)?;

        store
            .remove(&key)
            .map_err(|error| self.map_error("delete epoch-db snapshot", error))?;
        catalog.retain(|entry| entry != &key);

        self.save_catalog(&store, &catalog)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let store = self.lock_store()?;
        let catalog = self.load_catalog(&store)?;
        let mut documents = Vec::new();

        for doc_id_key in catalog {
            let Ok(doc_id) = Uuid::parse_str(&doc_id_key) else {
                continue;
            };

            match store
                .get(&doc_id_key)
                .map_err(|error| self.map_error("read epoch-db snapshot", error))?
            {
                Some(snapshot) => match self.deserialize_snapshot(doc_id, &snapshot) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping corrupt epoch-db snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                },
                None => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing epoch-db snapshot while building document catalog"
                ),
            }
        }

        Ok(documents)
    }
}
