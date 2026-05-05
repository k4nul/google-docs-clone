use std::{
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use append_kv::KvStore;
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

const SNAPSHOT_CATALOG_KEY: &str = "__catalog__";
const SNAPSHOT_KEY_PREFIX: &str = "snapshot:";

pub struct AppendKvSnapshotStore {
    path: PathBuf,
    store: Mutex<KvStore>,
}

impl AppendKvSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_APPEND_KV_PATH cannot be empty when SNAPSHOT_STORE=append_kv".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            ensure_snapshot_dir(parent)?;
        }

        let store = KvStore::open(&path).map_err(|error| Self::map_error(&path, error))?;

        Ok(Self {
            path,
            store: Mutex::new(store),
        })
    }

    fn snapshot_key(doc_id: &Uuid) -> String {
        format!("{SNAPSHOT_KEY_PREFIX}{doc_id}")
    }

    fn lock_store(&self) -> Result<MutexGuard<'_, KvStore>, StorageError> {
        self.store.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: append_kv store mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn map_error(path: &std::path::Path, error: anyhow::Error) -> StorageError {
        StorageError::Io(format!("{}: {error}", path.display()))
    }

    fn read_catalog_locked(&self, store: &mut KvStore) -> Result<Vec<Uuid>, StorageError> {
        let Some(payload) = store
            .get(SNAPSHOT_CATALOG_KEY)
            .map_err(|error| Self::map_error(&self.path, error))?
        else {
            return Ok(Vec::new());
        };

        serde_json::from_str::<Vec<Uuid>>(&payload).map_err(|_| {
            StorageError::Io(format!(
                "{}: append_kv snapshot catalog is corrupt",
                self.path.display()
            ))
        })
    }

    fn write_catalog_locked(
        &self,
        store: &mut KvStore,
        catalog: &[Uuid],
    ) -> Result<(), StorageError> {
        let payload = serde_json::to_string(catalog).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize append_kv snapshot catalog: {error}"
            ))
        })?;

        store
            .set(SNAPSHOT_CATALOG_KEY.to_owned(), payload)
            .map_err(|error| Self::map_error(&self.path, error))
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

impl SnapshotStore for AppendKvSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let mut store = self.lock_store()?;
        let Some(payload) = store
            .get(&Self::snapshot_key(doc_id))
            .map_err(|error| Self::map_error(&self.path, error))?
        else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &payload).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload =
            serde_json::to_string(&PersistedSnapshot::from(snapshot)).map_err(|error| {
                StorageError::Io(format!(
                    "failed to serialize append_kv snapshot `{doc_id}`: {error}"
                ))
            })?;
        let mut store = self.lock_store()?;

        store
            .set(Self::snapshot_key(&doc_id), payload)
            .map_err(|error| Self::map_error(&self.path, error))?;

        let mut catalog = self.read_catalog_locked(&mut store)?;
        if !catalog.contains(&doc_id) {
            catalog.push(doc_id);
            catalog.sort_unstable();
            self.write_catalog_locked(&mut store, &catalog)?;
        }

        Ok(())
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut store = self.lock_store()?;

        store
            .remove(Self::snapshot_key(doc_id))
            .map_err(|error| Self::map_error(&self.path, error))?;

        let mut catalog = self.read_catalog_locked(&mut store)?;
        let original_len = catalog.len();
        catalog.retain(|catalog_doc_id| catalog_doc_id != doc_id);
        if catalog.len() != original_len {
            self.write_catalog_locked(&mut store, &catalog)?;
        }

        Ok(())
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut store = self.lock_store()?;
        let catalog = self.read_catalog_locked(&mut store)?;
        let mut documents = Vec::new();

        for doc_id in catalog {
            let Some(payload) = store
                .get(&Self::snapshot_key(&doc_id))
                .map_err(|error| Self::map_error(&self.path, error))?
            else {
                tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing append_kv snapshot referenced by catalog"
                );
                continue;
            };

            match self.deserialize_snapshot(doc_id, &payload) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt append_kv snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
