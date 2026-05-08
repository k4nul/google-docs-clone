use std::{
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use bitcask_engine_rs::{
    bitcask::{BitCask, KVStorage},
    error::BitCaskError,
};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

const SNAPSHOT_CATALOG_KEY: &[u8] = b"__catalog__";
const SNAPSHOT_KEY_PREFIX: &str = "snapshot:";

pub struct BitcaskEngineSnapshotStore {
    path: PathBuf,
    database: Mutex<BitCask>,
}

impl BitcaskEngineSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_BITCASK_ENGINE_PATH cannot be empty when SNAPSHOT_STORE=bitcask_engine"
                    .to_owned(),
            ));
        }

        ensure_snapshot_dir(&path)?;
        let database = BitCask::new(&path).map_err(|error| {
            Self::map_bitcask_error(&path, "open bitcask-engine snapshot store", error)
        })?;

        Ok(Self {
            path,
            database: Mutex::new(database),
        })
    }

    fn snapshot_key(doc_id: &Uuid) -> Vec<u8> {
        format!("{SNAPSHOT_KEY_PREFIX}{doc_id}").into_bytes()
    }

    fn lock_database(&self) -> Result<MutexGuard<'_, BitCask>, StorageError> {
        self.database.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: bitcask-engine database mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn map_bitcask_error(
        path: &std::path::Path,
        action: &str,
        error: BitCaskError,
    ) -> StorageError {
        StorageError::Io(format!("{}: {action} failed: {error}", path.display()))
    }

    fn read_catalog(&self, database: &BitCask) -> Result<Vec<Uuid>, StorageError> {
        let Some(payload) = database.get(&SNAPSHOT_CATALOG_KEY.to_vec()) else {
            return Ok(Vec::new());
        };

        serde_json::from_slice::<Vec<Uuid>>(&payload).map_err(|_| {
            StorageError::Io(format!(
                "{}: bitcask-engine snapshot catalog is corrupt",
                self.path.display()
            ))
        })
    }

    fn write_catalog(&self, database: &mut BitCask, catalog: &[Uuid]) -> Result<(), StorageError> {
        let payload = serde_json::to_vec(catalog).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize bitcask-engine snapshot catalog: {error}"
            ))
        })?;

        database
            .put(&SNAPSHOT_CATALOG_KEY.to_vec(), &payload)
            .map_err(|error| {
                Self::map_bitcask_error(&self.path, "write bitcask-engine snapshot catalog", error)
            })
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

impl SnapshotStore for BitcaskEngineSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let database = self.lock_database()?;
        let Some(payload) = database.get(&Self::snapshot_key(doc_id)) else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &payload).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize bitcask-engine snapshot `{doc_id}`: {error}"
            ))
        })?;
        let mut database = self.lock_database()?;
        let mut catalog = self.read_catalog(&database)?;

        database
            .put(&Self::snapshot_key(&doc_id), &payload)
            .map_err(|error| {
                Self::map_bitcask_error(&self.path, "write bitcask-engine snapshot", error)
            })?;

        if !catalog.contains(&doc_id) {
            catalog.push(doc_id);
            catalog.sort_unstable();
            self.write_catalog(&mut database, &catalog)?;
        }

        Ok(())
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut database = self.lock_database()?;
        let mut catalog = self.read_catalog(&database)?;
        let original_len = catalog.len();

        database
            .delete(&Self::snapshot_key(doc_id))
            .map_err(|error| {
                Self::map_bitcask_error(&self.path, "delete bitcask-engine snapshot", error)
            })?;
        catalog.retain(|catalog_doc_id| catalog_doc_id != doc_id);

        if catalog.len() != original_len {
            self.write_catalog(&mut database, &catalog)?;
        }

        Ok(())
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let database = self.lock_database()?;
        let catalog = self.read_catalog(&database)?;
        let mut documents = Vec::new();

        for doc_id in catalog {
            let Some(payload) = database.get(&Self::snapshot_key(&doc_id)) else {
                tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing bitcask-engine snapshot referenced by catalog"
                );
                continue;
            };

            match self.deserialize_snapshot(doc_id, &payload) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(_)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt bitcask-engine snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
