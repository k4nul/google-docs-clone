use std::{
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use apexstore::{LsmConfig, LsmEngine};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

const SNAPSHOT_CATALOG_KEY: &str = "__catalog__";
const SNAPSHOT_KEY_PREFIX: &str = "snapshot:";

pub struct ApexStoreSnapshotStore {
    path: PathBuf,
    engine: Mutex<LsmEngine>,
}

impl ApexStoreSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_APEX_STORE_PATH cannot be empty when SNAPSHOT_STORE=apex_store"
                    .to_owned(),
            ));
        }

        ensure_snapshot_dir(&path)?;
        let config = LsmConfig::builder()
            .dir_path(&path)
            .memtable_max_size(1024 * 1024)
            .block_size(4096)
            .block_cache_size_mb(1)
            .sparse_index_interval(16)
            .bloom_false_positive_rate(0.01)
            .build()
            .map_err(|error| {
                StorageError::Config(format!(
                    "{}: invalid apex_store snapshot config: {error}",
                    path.display()
                ))
            })?;
        let engine = LsmEngine::new(config)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            engine: Mutex::new(engine),
        })
    }

    fn snapshot_key(doc_id: &Uuid) -> String {
        format!("{SNAPSHOT_KEY_PREFIX}{doc_id}")
    }

    fn lock_engine(&self) -> Result<MutexGuard<'_, LsmEngine>, StorageError> {
        self.engine.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: apex_store snapshot engine mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn map_error(&self, operation: &str, error: apexstore::LsmError) -> StorageError {
        StorageError::Io(format!(
            "{}: failed to {operation}: {error}",
            self.path.display()
        ))
    }

    fn read_value(&self, engine: &LsmEngine, key: &str) -> Result<Option<Vec<u8>>, StorageError> {
        engine
            .get(key)
            .map_err(|error| self.map_error("read apex_store snapshot value", error))
    }

    fn write_value(
        &self,
        engine: &LsmEngine,
        key: String,
        value: Vec<u8>,
    ) -> Result<(), StorageError> {
        engine
            .set(key, value)
            .map_err(|error| self.map_error("write apex_store snapshot value", error))
    }

    fn read_catalog(&self, engine: &LsmEngine) -> Result<Vec<Uuid>, StorageError> {
        let Some(payload) = self.read_value(engine, SNAPSHOT_CATALOG_KEY)? else {
            return Ok(Vec::new());
        };

        serde_json::from_slice::<Vec<Uuid>>(&payload).map_err(|_| {
            StorageError::Io(format!(
                "{}: apex_store snapshot catalog is corrupt",
                self.path.display()
            ))
        })
    }

    fn write_catalog(&self, engine: &LsmEngine, catalog: &[Uuid]) -> Result<(), StorageError> {
        let payload = serde_json::to_vec(catalog).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize apex_store snapshot catalog: {error}"
            ))
        })?;

        self.write_value(engine, SNAPSHOT_CATALOG_KEY.to_owned(), payload)
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

impl SnapshotStore for ApexStoreSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let engine = self.lock_engine()?;
        let Some(payload) = self.read_value(&engine, &Self::snapshot_key(doc_id))? else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &payload).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let engine = self.lock_engine()?;
        let doc_id = snapshot.document.id;
        let payload = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize apex_store snapshot `{doc_id}`: {error}"
            ))
        })?;

        self.write_value(&engine, Self::snapshot_key(&doc_id), payload)?;

        let mut catalog = self.read_catalog(&engine)?;
        if !catalog.contains(&doc_id) {
            catalog.push(doc_id);
            catalog.sort_unstable();
            self.write_catalog(&engine, &catalog)?;
        }

        Ok(())
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let engine = self.lock_engine()?;
        engine
            .delete(Self::snapshot_key(doc_id))
            .map_err(|error| self.map_error("delete apex_store snapshot value", error))?;

        let mut catalog = self.read_catalog(&engine)?;
        let original_len = catalog.len();
        catalog.retain(|catalog_doc_id| catalog_doc_id != doc_id);
        if catalog.len() != original_len {
            self.write_catalog(&engine, &catalog)?;
        }

        Ok(())
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let engine = self.lock_engine()?;
        let catalog = self.read_catalog(&engine)?;
        let mut documents = Vec::new();

        for doc_id in catalog {
            match self.read_value(&engine, &Self::snapshot_key(&doc_id)) {
                Ok(Some(payload)) => match self.deserialize_snapshot(doc_id, &payload) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping corrupt apex_store snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                },
                Ok(None) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing apex_store snapshot referenced by catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
