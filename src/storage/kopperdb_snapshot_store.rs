use std::{
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use kopperdb::kopper::{Kopper, KopperError};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

const SNAPSHOT_CATALOG_KEY: &str = "__catalog__";
const SNAPSHOT_TOMBSTONE_VALUE: &str = "__deleted__";
const SEGMENT_SIZE_BYTES: usize = 16 * 1024 * 1024;

pub struct KopperdbSnapshotStore {
    path: PathBuf,
    store: Mutex<Kopper>,
}

impl KopperdbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_KOPPERDB_PATH cannot be empty when SNAPSHOT_STORE=kopperdb".to_owned(),
            ));
        }

        ensure_snapshot_dir(&path)?;
        let store = Kopper::create(path.to_string_lossy().as_ref(), SEGMENT_SIZE_BYTES).map_err(
            |error| Self::map_kopperdb_error(&path, "open kopperdb snapshot store", error),
        )?;

        Ok(Self {
            path,
            store: Mutex::new(store),
        })
    }

    fn lock_store(&self) -> Result<MutexGuard<'_, Kopper>, StorageError> {
        self.store.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: kopperdb store mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn map_kopperdb_error(
        path: &std::path::Path,
        operation: &str,
        error: KopperError,
    ) -> StorageError {
        StorageError::Io(format!(
            "{}: failed to {operation}: {error}",
            path.display()
        ))
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

    fn load_catalog_locked(&self, store: &Kopper) -> Result<Vec<String>, StorageError> {
        match store.read(SNAPSHOT_CATALOG_KEY) {
            Ok(payload) => serde_json::from_str::<Vec<String>>(&payload).map_err(|_| {
                StorageError::Io(format!(
                    "{}: snapshot catalog is corrupt",
                    self.path.display()
                ))
            }),
            Err(KopperError::KeyDoesNotExist(_)) => Ok(Vec::new()),
            Err(error) => Err(Self::map_kopperdb_error(
                &self.path,
                "read kopperdb snapshot catalog",
                error,
            )),
        }
    }

    fn save_catalog_locked(&self, store: &Kopper, catalog: &[String]) -> Result<(), StorageError> {
        let payload = serde_json::to_string(catalog)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        store
            .write(SNAPSHOT_CATALOG_KEY, &payload)
            .map(|_| ())
            .map_err(|error| {
                Self::map_kopperdb_error(&self.path, "write kopperdb snapshot catalog", error)
            })
    }

    fn read_snapshot_payload_locked(
        &self,
        store: &Kopper,
        doc_id: &Uuid,
    ) -> Result<Option<String>, StorageError> {
        match store.read(&doc_id.to_string()) {
            Ok(payload) if payload == SNAPSHOT_TOMBSTONE_VALUE => Ok(None),
            Ok(payload) => Ok(Some(payload)),
            Err(KopperError::KeyDoesNotExist(_)) => Ok(None),
            Err(error) => Err(Self::map_kopperdb_error(
                &self.path,
                "read kopperdb snapshot",
                error,
            )),
        }
    }
}

impl SnapshotStore for KopperdbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let store = self.lock_store()?;
        let Some(payload) = self.read_snapshot_payload_locked(&store, doc_id)? else {
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
                    "failed to serialize kopperdb snapshot `{doc_id}`: {error}"
                ))
            })?;
        let store = self.lock_store()?;
        let mut catalog = self.load_catalog_locked(&store)?;

        store
            .write(&doc_id_key, &payload)
            .map(|_| ())
            .map_err(|error| {
                Self::map_kopperdb_error(&self.path, "write kopperdb snapshot", error)
            })?;

        if !catalog.iter().any(|value| value == &doc_id_key) {
            catalog.push(doc_id_key);
            catalog.sort();
        }

        self.save_catalog_locked(&store, &catalog)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let doc_id_key = doc_id.to_string();
        let store = self.lock_store()?;
        let mut catalog = self.load_catalog_locked(&store)?;

        store
            .write(&doc_id_key, SNAPSHOT_TOMBSTONE_VALUE)
            .map(|_| ())
            .map_err(|error| {
                Self::map_kopperdb_error(&self.path, "delete kopperdb snapshot", error)
            })?;
        catalog.retain(|value| value != &doc_id_key);

        self.save_catalog_locked(&store, &catalog)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let store = self.lock_store()?;
        let catalog = self.load_catalog_locked(&store)?;
        let mut documents = Vec::new();

        for doc_id_key in catalog {
            let Ok(doc_id) = Uuid::parse_str(&doc_id_key) else {
                continue;
            };

            match self.read_snapshot_payload_locked(&store, &doc_id)? {
                Some(payload) => match self.deserialize_snapshot(doc_id, &payload) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping corrupt kopperdb snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                },
                None => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing kopperdb snapshot while building document catalog"
                ),
            }
        }

        Ok(documents)
    }
}
