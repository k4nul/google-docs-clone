use std::{
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use rcask::RCask;
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

const SNAPSHOT_CATALOG_KEY: &str = "__catalog__";
const SNAPSHOT_TOMBSTONE_VALUE: &str = "__deleted__";
const SNAPSHOT_LOG_PATTERN: &str = "snapshots";
const MAX_WRITES_BEFORE_COMPACTION: u64 = u64::MAX;

pub struct RcaskSnapshotStore {
    path: PathBuf,
    lock: Mutex<()>,
}

impl RcaskSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_RCASK_PATH cannot be empty when SNAPSHOT_STORE=rcask".to_owned(),
            ));
        }

        ensure_snapshot_dir(&path)?;
        let store = Self {
            path,
            lock: Mutex::new(()),
        };
        store.open_store("open rcask snapshot store")?;

        Ok(store)
    }

    fn lock(&self) -> Result<MutexGuard<'_, ()>, StorageError> {
        self.lock.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: rcask snapshot store mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn open_store(&self, operation: &str) -> Result<RCask, StorageError> {
        RCask::init(
            self.path.to_string_lossy().into_owned(),
            SNAPSHOT_LOG_PATTERN.to_owned(),
            MAX_WRITES_BEFORE_COMPACTION,
        )
        .map_err(|error| {
            StorageError::Io(format!(
                "{}: failed to {operation}: {error}",
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

    fn load_catalog_locked(&self) -> Result<Vec<String>, StorageError> {
        let mut store = self.open_store("open rcask snapshot store for catalog read")?;
        match store
            .get(SNAPSHOT_CATALOG_KEY)
            .map_err(|error| self.map_error("read rcask snapshot catalog", error))?
        {
            Some(payload) => serde_json::from_str::<Vec<String>>(&payload).map_err(|_| {
                StorageError::Io(format!(
                    "{}: rcask snapshot catalog is corrupt",
                    self.path.display()
                ))
            }),
            None => Ok(Vec::new()),
        }
    }

    fn save_catalog_locked(
        &self,
        store: &mut RCask,
        catalog: &[String],
    ) -> Result<(), StorageError> {
        let payload = serde_json::to_string(catalog)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        store
            .set(SNAPSHOT_CATALOG_KEY, payload)
            .map_err(|error| self.map_error("write rcask snapshot catalog", error))
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

    fn read_snapshot_payload_locked(&self, doc_id: &Uuid) -> Result<Option<String>, StorageError> {
        let mut store = self.open_store("open rcask snapshot store for snapshot read")?;
        match store
            .get(&doc_id.to_string())
            .map_err(|error| self.map_error("read rcask snapshot", error))?
        {
            Some(payload) if payload == SNAPSHOT_TOMBSTONE_VALUE => Ok(None),
            Some(payload) => Ok(Some(payload)),
            None => Ok(None),
        }
    }
}

impl SnapshotStore for RcaskSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let _guard = self.lock()?;
        let Some(payload) = self.read_snapshot_payload_locked(doc_id)? else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &payload).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let _guard = self.lock()?;
        let doc_id = snapshot.document.id;
        let doc_id_key = doc_id.to_string();
        let payload =
            serde_json::to_string(&PersistedSnapshot::from(snapshot)).map_err(|error| {
                StorageError::Io(format!(
                    "failed to serialize rcask snapshot `{doc_id}`: {error}"
                ))
            })?;
        let mut catalog = self.load_catalog_locked()?;
        let mut store = self.open_store("open rcask snapshot store for snapshot write")?;

        store
            .set(&doc_id_key, payload)
            .map_err(|error| self.map_error("write rcask snapshot", error))?;

        if !catalog.iter().any(|value| value == &doc_id_key) {
            catalog.push(doc_id_key);
            catalog.sort();
        }

        self.save_catalog_locked(&mut store, &catalog)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let _guard = self.lock()?;
        let doc_id_key = doc_id.to_string();
        let mut catalog = self.load_catalog_locked()?;
        let mut store = self.open_store("open rcask snapshot store for snapshot delete")?;

        store
            .set(&doc_id_key, SNAPSHOT_TOMBSTONE_VALUE)
            .map_err(|error| self.map_error("delete rcask snapshot", error))?;
        catalog.retain(|value| value != &doc_id_key);

        self.save_catalog_locked(&mut store, &catalog)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let _guard = self.lock()?;
        let catalog = self.load_catalog_locked()?;
        let mut documents = Vec::new();

        for doc_id_key in catalog {
            let Ok(doc_id) = Uuid::parse_str(&doc_id_key) else {
                continue;
            };

            match self.read_snapshot_payload_locked(&doc_id)? {
                Some(payload) => match self.deserialize_snapshot(doc_id, &payload) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping corrupt rcask snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                },
                None => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing rcask snapshot while building document catalog"
                ),
            }
        }

        Ok(documents)
    }
}
