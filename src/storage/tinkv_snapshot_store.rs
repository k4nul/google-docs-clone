use std::{
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use tinkv::{OpenOptions, Store, TinkvError};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

const CATALOG_KEY: &[u8] = b"__catalog__";
const MAX_SNAPSHOT_VALUE_SIZE: u64 = 64 * 1024 * 1024;

pub struct TinkvSnapshotStore {
    path: PathBuf,
    store: Mutex<Store>,
}

impl TinkvSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_TINKV_PATH cannot be empty when SNAPSHOT_STORE=tinkv".to_owned(),
            ));
        }

        ensure_snapshot_dir(&path)?;
        let mut options = OpenOptions::new();
        let store = options
            .max_value_size(MAX_SNAPSHOT_VALUE_SIZE)
            .sync(true)
            .open(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            store: Mutex::new(store),
        })
    }

    fn lock_store(&self) -> Result<MutexGuard<'_, Store>, StorageError> {
        self.store.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: tinkv store mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn snapshot_key(doc_id: &Uuid) -> String {
        format!("snapshot:{doc_id}")
    }

    fn load_catalog(&self, store: &mut Store) -> Result<Vec<Uuid>, StorageError> {
        let Some(payload) = store
            .get(CATALOG_KEY)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?
        else {
            return Ok(Vec::new());
        };

        serde_json::from_slice::<Vec<Uuid>>(&payload).map_err(|error| {
            StorageError::Io(format!(
                "{}: failed to parse tinkv snapshot catalog: {error}",
                self.path.display()
            ))
        })
    }

    fn save_catalog(&self, store: &mut Store, mut catalog: Vec<Uuid>) -> Result<(), StorageError> {
        catalog.sort_unstable();
        catalog.dedup();
        let payload = serde_json::to_vec(&catalog).map_err(|error| {
            StorageError::Io(format!(
                "{}: failed to serialize tinkv snapshot catalog: {error}",
                self.path.display()
            ))
        })?;
        store
            .set(CATALOG_KEY, &payload)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn sync(&self, store: &mut Store) -> Result<(), StorageError> {
        store
            .sync()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
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

impl SnapshotStore for TinkvSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let mut store = self.lock_store()?;
        let key = Self::snapshot_key(doc_id);
        let Some(payload) = store
            .get(key.as_bytes())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?
        else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &payload).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let mut store = self.lock_store()?;
        let doc_id = snapshot.document.id;
        let payload = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize tinkv snapshot `{doc_id}`: {error}"
            ))
        })?;

        store
            .set(Self::snapshot_key(&doc_id).as_bytes(), &payload)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        let mut catalog = self.load_catalog(&mut store)?;
        catalog.push(doc_id);
        self.save_catalog(&mut store, catalog)?;
        self.sync(&mut store)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut store = self.lock_store()?;
        match store.remove(Self::snapshot_key(doc_id).as_bytes()) {
            Ok(()) | Err(TinkvError::KeyNotFound(_)) => {}
            Err(error) => {
                return Err(StorageError::Io(format!(
                    "{}: {error}",
                    self.path.display()
                )));
            }
        }

        let mut catalog = self.load_catalog(&mut store)?;
        catalog.retain(|candidate| candidate != doc_id);
        self.save_catalog(&mut store, catalog)?;
        self.sync(&mut store)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut store = self.lock_store()?;
        let catalog = self.load_catalog(&mut store)?;
        let mut documents = Vec::new();

        for doc_id in catalog {
            let key = Self::snapshot_key(&doc_id);
            let Some(payload) = store
                .get(key.as_bytes())
                .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?
            else {
                tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing tinkv snapshot referenced by catalog"
                );
                continue;
            };

            match self.deserialize_snapshot(doc_id, &payload) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt tinkv snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        documents.sort_unstable_by_key(|document| document.id);
        Ok(documents)
    }
}
