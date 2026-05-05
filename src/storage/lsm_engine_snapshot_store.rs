use std::{
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
};

use lsm_engine::{LSMBuilder, LSMEngine};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOT_CATALOG_KEY: &str = "__catalog__";
const SNAPSHOT_KEY_PREFIX: &str = "snapshot:";
const SEGMENT_SIZE: usize = 2_048;
const IN_MEMORY_CAPACITY: usize = 256;
const SPARSE_OFFSET: usize = 16;

pub struct LsmEngineSnapshotStore {
    path: PathBuf,
    engine: Mutex<LSMEngine>,
}

impl LsmEngineSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_LSM_ENGINE_PATH cannot be empty when SNAPSHOT_STORE=lsm_engine"
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

        let wal = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;
        let mut engine = Self::build_engine();
        engine.recover_from(wal).map_err(|error| {
            Self::map_path_error(&path, "recover lsm_engine snapshot WAL", error)
        })?;

        Ok(Self {
            path,
            engine: Mutex::new(engine),
        })
    }

    fn build_engine() -> LSMEngine {
        LSMBuilder::new()
            .segment_size(SEGMENT_SIZE)
            .inmemory_capacity(IN_MEMORY_CAPACITY)
            .sparse_offset(SPARSE_OFFSET)
            .build()
    }

    fn snapshot_key(doc_id: &Uuid) -> String {
        format!("{SNAPSHOT_KEY_PREFIX}{doc_id}")
    }

    fn lock_engine(&self) -> Result<MutexGuard<'_, LSMEngine>, StorageError> {
        self.engine.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: lsm_engine mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn map_path_error(path: &Path, operation: &str, error: impl std::fmt::Display) -> StorageError {
        StorageError::Io(format!(
            "{}: failed to {operation}: {error}",
            path.display()
        ))
    }

    fn map_error(&self, operation: &str, error: impl std::fmt::Display) -> StorageError {
        Self::map_path_error(&self.path, operation, error)
    }

    fn read_catalog(&self, engine: &mut LSMEngine) -> Result<Vec<Uuid>, StorageError> {
        let Some(payload) = engine
            .read(SNAPSHOT_CATALOG_KEY)
            .map_err(|error| self.map_error("read lsm_engine snapshot catalog", error))?
        else {
            return Ok(Vec::new());
        };

        serde_json::from_str::<Vec<Uuid>>(&payload).map_err(|_| {
            StorageError::Io(format!(
                "{}: lsm_engine snapshot catalog is corrupt",
                self.path.display()
            ))
        })
    }

    fn write_catalog(&self, engine: &mut LSMEngine, catalog: &[Uuid]) -> Result<(), StorageError> {
        let payload = serde_json::to_string(catalog).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize lsm_engine snapshot catalog: {error}"
            ))
        })?;

        engine
            .write(SNAPSHOT_CATALOG_KEY.to_owned(), payload)
            .map_err(|error| self.map_error("write lsm_engine snapshot catalog", error))
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

impl SnapshotStore for LsmEngineSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let mut engine = self.lock_engine()?;
        let Some(payload) = engine
            .read(&Self::snapshot_key(doc_id))
            .map_err(|error| self.map_error("read lsm_engine snapshot", error))?
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
                    "failed to serialize lsm_engine snapshot `{doc_id}`: {error}"
                ))
            })?;

        let mut engine = self.lock_engine()?;
        engine
            .write(Self::snapshot_key(&doc_id), payload)
            .map_err(|error| self.map_error("write lsm_engine snapshot", error))?;

        let mut catalog = self.read_catalog(&mut engine)?;
        if !catalog.contains(&doc_id) {
            catalog.push(doc_id);
            catalog.sort_unstable();
            self.write_catalog(&mut engine, &catalog)?;
        }

        Ok(())
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut engine = self.lock_engine()?;
        engine
            .delete(&Self::snapshot_key(doc_id))
            .map_err(|error| self.map_error("delete lsm_engine snapshot", error))?;

        let mut catalog = self.read_catalog(&mut engine)?;
        let original_len = catalog.len();
        catalog.retain(|catalog_doc_id| catalog_doc_id != doc_id);
        if catalog.len() != original_len {
            self.write_catalog(&mut engine, &catalog)?;
        }

        Ok(())
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut engine = self.lock_engine()?;
        let catalog = self.read_catalog(&mut engine)?;
        let mut documents = Vec::new();

        for doc_id in catalog {
            match engine.read(&Self::snapshot_key(&doc_id)) {
                Ok(Some(payload)) => match self.deserialize_snapshot(doc_id, &payload) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping corrupt lsm_engine snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                },
                Ok(None) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing lsm_engine snapshot referenced by catalog"
                ),
                Err(error) => return Err(self.map_error("read lsm_engine snapshot", error)),
            }
        }

        Ok(documents)
    }
}
