use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use rskey::Store;
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

pub struct RskeySnapshotStore {
    path: PathBuf,
    store: Mutex<Store<String>>,
}

impl RskeySnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_RSKEY_PATH cannot be empty when SNAPSHOT_STORE=rskey".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let store = Store::open(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            store: Mutex::new(store),
        })
    }

    fn lock_store(&self) -> Result<MutexGuard<'_, Store<String>>, StorageError> {
        self.store.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: rskey store mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn sync_store(&self, store: &Store<String>) -> Result<(), StorageError> {
        store
            .sync()
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

impl SnapshotStore for RskeySnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let store = self.lock_store()?;
        let Some(payload) = store.get(&doc_id.to_string()) else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, payload).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload =
            serde_json::to_string(&PersistedSnapshot::from(snapshot)).map_err(|error| {
                StorageError::Io(format!(
                    "failed to serialize rskey snapshot `{doc_id}`: {error}"
                ))
            })?;
        let mut store = self.lock_store()?;
        store.insert(doc_id.to_string(), payload);
        self.sync_store(&store)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut store = self.lock_store()?;
        store.remove(&doc_id.to_string());
        self.sync_store(&store)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let store = self.lock_store()?;
        let mut documents = Vec::new();

        for (doc_id_key, payload) in store.iter() {
            let Ok(doc_id) = Uuid::parse_str(doc_id_key) else {
                continue;
            };

            match self.deserialize_snapshot(doc_id, payload) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt rskey snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
