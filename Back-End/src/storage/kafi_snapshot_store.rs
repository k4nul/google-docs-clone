use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use kafi::Store;
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const CATALOG_KEY: &str = "__catalog__";

pub struct KafiSnapshotStore {
    path: PathBuf,
    store: Mutex<Store<String, String>>,
}

impl KafiSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_KAFI_PATH cannot be empty when SNAPSHOT_STORE=kafi".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let mut store = Store::open(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;
        if !store.exists(CATALOG_KEY) {
            store.insert(CATALOG_KEY.to_owned(), "[]".to_owned());
            store
                .flush()
                .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;
        }

        Ok(Self {
            path,
            store: Mutex::new(store),
        })
    }

    fn lock_store(&self) -> Result<MutexGuard<'_, Store<String, String>>, StorageError> {
        self.store.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: kafi store mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn snapshot_key(doc_id: &Uuid) -> String {
        format!("snapshot:{doc_id}")
    }

    fn load_catalog(&self, store: &mut Store<String, String>) -> Result<Vec<Uuid>, StorageError> {
        let Some(payload) = store.get(CATALOG_KEY) else {
            return Ok(Vec::new());
        };

        serde_json::from_str::<Vec<Uuid>>(payload).map_err(|error| {
            StorageError::Io(format!(
                "{}: failed to parse kafi snapshot catalog: {error}",
                self.path.display()
            ))
        })
    }

    fn save_catalog(
        &self,
        store: &mut Store<String, String>,
        mut catalog: Vec<Uuid>,
    ) -> Result<(), StorageError> {
        catalog.sort_unstable();
        catalog.dedup();
        let payload = serde_json::to_string(&catalog).map_err(|error| {
            StorageError::Io(format!(
                "{}: failed to serialize kafi snapshot catalog: {error}",
                self.path.display()
            ))
        })?;
        store.insert(CATALOG_KEY.to_owned(), payload);
        Ok(())
    }

    fn flush(&self, store: &mut Store<String, String>) -> Result<(), StorageError> {
        store
            .flush()
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

impl SnapshotStore for KafiSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let mut store = self.lock_store()?;
        let key = Self::snapshot_key(doc_id);
        let Some(payload) = store.get(&key) else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, payload).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let mut store = self.lock_store()?;
        let doc_id = snapshot.document.id;
        let payload =
            serde_json::to_string(&PersistedSnapshot::from(snapshot)).map_err(|error| {
                StorageError::Io(format!(
                    "failed to serialize kafi snapshot `{doc_id}`: {error}"
                ))
            })?;

        store.insert(Self::snapshot_key(&doc_id), payload);
        let mut catalog = self.load_catalog(&mut store)?;
        catalog.push(doc_id);
        self.save_catalog(&mut store, catalog)?;
        self.flush(&mut store)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut store = self.lock_store()?;
        store.remove(&Self::snapshot_key(doc_id));
        let mut catalog = self.load_catalog(&mut store)?;
        catalog.retain(|candidate| candidate != doc_id);
        self.save_catalog(&mut store, catalog)?;
        self.flush(&mut store)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut store = self.lock_store()?;
        let catalog = self.load_catalog(&mut store)?;
        let mut documents = Vec::new();

        for doc_id in catalog {
            let key = Self::snapshot_key(&doc_id);
            let Some(payload) = store.get(&key) else {
                continue;
            };

            match self.deserialize_snapshot(doc_id, payload) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt kafi snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        documents.sort_unstable_by_key(|document| document.id);
        Ok(documents)
    }
}
