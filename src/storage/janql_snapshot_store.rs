use std::{
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use janql::Database;
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

const SNAPSHOT_CATALOG_KEY: &str = "__catalog__";

pub struct JanqlSnapshotStore {
    path: PathBuf,
    store: Mutex<Database>,
}

impl JanqlSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_JANQL_PATH cannot be empty when SNAPSHOT_STORE=janql".to_owned(),
            ));
        }

        ensure_snapshot_dir(&path)?;
        let store = Database::load(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            store: Mutex::new(store),
        })
    }

    fn lock_store(&self) -> Result<MutexGuard<'_, Database>, StorageError> {
        self.store.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: janql snapshot store mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn run_mutation<T>(
        &self,
        operation: &str,
        f: impl FnOnce(&mut Database) -> T,
    ) -> Result<T, StorageError> {
        let mut store = self.lock_store()?;
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(&mut store))).map_err(|_| {
            StorageError::Io(format!(
                "{}: janql snapshot store panicked while trying to {operation}",
                self.path.display()
            ))
        })
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

    fn load_catalog(&self, store: &mut Database) -> Result<Vec<String>, StorageError> {
        let Some(payload) = store.get(SNAPSHOT_CATALOG_KEY) else {
            return Ok(Vec::new());
        };

        serde_json::from_str(&payload).map_err(|_| {
            StorageError::Io(format!(
                "{}: janql snapshot catalog is corrupt",
                self.path.display()
            ))
        })
    }

    fn save_catalog(&self, store: &mut Database, catalog: &[String]) -> Result<(), StorageError> {
        let payload = serde_json::to_string(catalog)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        store.set(SNAPSHOT_CATALOG_KEY.to_owned(), payload);
        Ok(())
    }
}

impl SnapshotStore for JanqlSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let mut store = self.lock_store()?;
        let Some(payload) = store.get(&doc_id.to_string()) else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &payload).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let doc_id_key = doc_id.to_string();
        let payload =
            serde_json::to_string(&PersistedSnapshot::from(snapshot)).map_err(|error| {
                StorageError::Io(format!(
                    "failed to serialize janql snapshot `{doc_id}`: {error}"
                ))
            })?;

        self.run_mutation("save snapshot", |store| {
            let mut catalog = self.load_catalog(store)?;
            store.set(doc_id_key.clone(), payload);

            if !catalog.iter().any(|entry| entry == &doc_id_key) {
                catalog.push(doc_id_key);
                catalog.sort();
            }

            self.save_catalog(store, &catalog)
        })?
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let doc_id_key = doc_id.to_string();

        self.run_mutation("delete snapshot", |store| {
            let mut catalog = self.load_catalog(store)?;
            store.del(&doc_id_key);
            catalog.retain(|entry| entry != &doc_id_key);
            self.save_catalog(store, &catalog)
        })?
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut store = self.lock_store()?;
        let catalog = self.load_catalog(&mut store)?;
        let mut documents = Vec::new();

        for key in catalog {
            let Ok(doc_id) = Uuid::parse_str(&key) else {
                continue;
            };

            let Some(payload) = store.get(&key) else {
                tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing janql snapshot while building document catalog"
                );
                continue;
            };

            match self.deserialize_snapshot(doc_id, &payload) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt janql snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
