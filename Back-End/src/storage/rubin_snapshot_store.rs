use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use rubin::store::mem::MemStore;
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

pub struct RubinSnapshotStore {
    path: PathBuf,
    store: Mutex<MemStore>,
}

impl RubinSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_RUBIN_PATH cannot be empty when SNAPSHOT_STORE=rubin".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let store = if path.exists() {
            let raw = fs::read_to_string(&path)
                .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;
            if raw.trim().is_empty() {
                MemStore::new()
            } else {
                serde_json::from_str::<MemStore>(&raw).map_err(|_| {
                    StorageError::Io(format!("{}: corrupt rubin store", path.display()))
                })?
            }
        } else {
            MemStore::new()
        };

        Ok(Self {
            path,
            store: Mutex::new(store),
        })
    }

    fn lock_store(&self) -> Result<MutexGuard<'_, MemStore>, StorageError> {
        self.store.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: rubin snapshot store mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn persist(&self, store: &MemStore) -> Result<(), StorageError> {
        store
            .dump_store(&self.path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
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

impl SnapshotStore for RubinSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let store = self.lock_store()?;
        let payload = store
            .get_string(&doc_id.to_string())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        if payload.is_empty() {
            return Ok(None);
        }

        self.deserialize_snapshot(*doc_id, &payload).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload =
            serde_json::to_string(&PersistedSnapshot::from(snapshot)).map_err(|error| {
                StorageError::Io(format!(
                    "failed to serialize rubin snapshot `{doc_id}`: {error}"
                ))
            })?;

        let mut store = self.lock_store()?;
        store
            .insert_string(&doc_id.to_string(), &payload)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        self.persist(&store)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut store = self.lock_store()?;
        store
            .remove_string(&doc_id.to_string())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        self.persist(&store)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let store = self.lock_store()?;
        let mut documents = Vec::new();

        for (doc_id_key, payload) in store.get_string_store_ref() {
            let Ok(doc_id) = Uuid::parse_str(doc_id_key) else {
                continue;
            };

            match self.deserialize_snapshot(doc_id, payload) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt rubin snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        documents.sort_unstable_by_key(|document| document.id);
        Ok(documents)
    }
}
