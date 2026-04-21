use std::{path::PathBuf, sync::Mutex};

use lsm_storage_engine::Engine;
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

const SNAPSHOT_CATALOG_KEY: &[u8] = b"__catalog__";
const SNAPSHOT_KEY_PREFIX: &str = "snapshot:";
const MAX_MEMTABLE_SIZE: usize = 1024 * 1024;

pub struct LsmStorageEngineSnapshotStore {
    path: PathBuf,
    engine: Mutex<Engine>,
}

impl LsmStorageEngineSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_LSM_STORAGE_ENGINE_PATH cannot be empty when SNAPSHOT_STORE=lsm_storage_engine".to_owned(),
            ));
        }

        ensure_snapshot_dir(&path)?;
        let engine = Engine::open(&path, MAX_MEMTABLE_SIZE)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            engine: Mutex::new(engine),
        })
    }

    fn snapshot_key(doc_id: &Uuid) -> Vec<u8> {
        format!("{SNAPSHOT_KEY_PREFIX}{doc_id}").into_bytes()
    }

    fn lock_engine(&self) -> Result<std::sync::MutexGuard<'_, Engine>, StorageError> {
        self.engine.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: lsm_storage_engine mutex was poisoned",
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

    fn read_catalog(&self, engine: &Engine) -> Result<Vec<Uuid>, StorageError> {
        let Some(payload) = engine
            .get(SNAPSHOT_CATALOG_KEY)
            .map_err(|error| self.map_error("read lsm_storage_engine snapshot catalog", error))?
        else {
            return Ok(Vec::new());
        };

        serde_json::from_slice::<Vec<Uuid>>(&payload).map_err(|_| {
            StorageError::Io(format!(
                "{}: lsm_storage_engine snapshot catalog is corrupt",
                self.path.display()
            ))
        })
    }

    fn write_catalog(&self, engine: &Engine, catalog: &[Uuid]) -> Result<(), StorageError> {
        let payload = serde_json::to_vec(catalog).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize lsm_storage_engine snapshot catalog: {error}"
            ))
        })?;

        engine
            .put(SNAPSHOT_CATALOG_KEY.to_vec(), payload)
            .map_err(|error| self.map_error("write lsm_storage_engine snapshot catalog", error))
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

impl SnapshotStore for LsmStorageEngineSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let engine = self.lock_engine()?;
        let Some(payload) = engine
            .get(&Self::snapshot_key(doc_id))
            .map_err(|error| self.map_error("read lsm_storage_engine snapshot", error))?
        else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &payload).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize lsm_storage_engine snapshot `{doc_id}`: {error}"
            ))
        })?;

        let engine = self.lock_engine()?;
        engine
            .put(Self::snapshot_key(&doc_id), payload)
            .map_err(|error| self.map_error("write lsm_storage_engine snapshot", error))?;

        let mut catalog = self.read_catalog(&engine)?;
        if !catalog.contains(&doc_id) {
            catalog.push(doc_id);
            catalog.sort_unstable();
            self.write_catalog(&engine, &catalog)?;
        }

        engine
            .flush()
            .map_err(|error| self.map_error("flush lsm_storage_engine snapshot store", error))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let engine = self.lock_engine()?;
        engine
            .delete(Self::snapshot_key(doc_id))
            .map_err(|error| self.map_error("delete lsm_storage_engine snapshot", error))?;

        let mut catalog = self.read_catalog(&engine)?;
        let original_len = catalog.len();
        catalog.retain(|catalog_doc_id| catalog_doc_id != doc_id);
        if catalog.len() != original_len {
            self.write_catalog(&engine, &catalog)?;
        }

        engine
            .flush()
            .map_err(|error| self.map_error("flush lsm_storage_engine snapshot store", error))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let engine = self.lock_engine()?;
        let catalog = self.read_catalog(&engine)?;
        let mut documents = Vec::new();

        for doc_id in catalog {
            match engine.get(&Self::snapshot_key(&doc_id)) {
                Ok(Some(payload)) => match self.deserialize_snapshot(doc_id, &payload) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping corrupt lsm_storage_engine snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                },
                Ok(None) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing lsm_storage_engine snapshot referenced by catalog"
                ),
                Err(error) => {
                    return Err(self.map_error("read lsm_storage_engine snapshot", error));
                }
            }
        }

        Ok(documents)
    }
}
