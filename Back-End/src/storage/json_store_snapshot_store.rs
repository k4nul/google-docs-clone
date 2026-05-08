use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use json_store::JSStore;
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

pub struct JsonStoreSnapshotStore {
    path: PathBuf,
    store: Mutex<JSStore<String, PersistedSnapshot>>,
}

impl JsonStoreSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_JSON_STORE_PATH cannot be empty when SNAPSHOT_STORE=json_store"
                    .to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let store = JSStore::new(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            store: Mutex::new(store),
        })
    }

    fn lock_store(
        &self,
    ) -> Result<MutexGuard<'_, JSStore<String, PersistedSnapshot>>, StorageError> {
        self.store.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: json_store mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn map_error(&self, error: std::io::Error) -> StorageError {
        StorageError::Io(format!("{}: {error}", self.path.display()))
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        snapshot: PersistedSnapshot,
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot: DocumentSnapshot = snapshot.into();
        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }
}

impl SnapshotStore for JsonStoreSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let store = self.lock_store()?;
        let snapshot = store
            .get(doc_id.to_string())
            .map_err(|error| self.map_error(error))?;
        let Some(snapshot) = snapshot else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, snapshot).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let mut store = self.lock_store()?;
        store
            .insert(doc_id.to_string(), PersistedSnapshot::from(snapshot))
            .map_err(|error| self.map_error(error))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut store = self.lock_store()?;
        store
            .remove(doc_id.to_string())
            .map_err(|error| self.map_error(error))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let store = self.lock_store()?;
        let keys = store.keys().cloned().collect::<Vec<_>>();
        let mut documents = Vec::new();

        for doc_id_key in keys {
            let Ok(doc_id) = Uuid::parse_str(&doc_id_key) else {
                continue;
            };

            let snapshot = store
                .get(doc_id_key.clone())
                .map_err(|error| self.map_error(error))?;
            let Some(snapshot) = snapshot else {
                tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing json_store snapshot while building document catalog"
                );
                continue;
            };

            match self.deserialize_snapshot(doc_id, snapshot) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt json_store snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
