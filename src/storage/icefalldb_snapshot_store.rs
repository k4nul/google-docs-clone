use std::{
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use icefalldb::RSDB;
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

const SNAPSHOT_CATALOG_KEY: &[u8] = b"__catalog__";
const SNAPSHOT_TOMBSTONE_VALUE: &[u8] = b"__deleted__";

pub struct IcefalldbSnapshotStore {
    path: PathBuf,
    store: Mutex<RSDB<'static>>,
}

impl IcefalldbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_ICEFALLDB_PATH cannot be empty when SNAPSHOT_STORE=icefalldb".to_owned(),
            ));
        }

        ensure_snapshot_dir(&path)?;
        let leaked_path = Box::leak(path.to_string_lossy().into_owned().into_boxed_str());
        let store = RSDB::new(leaked_path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            store: Mutex::new(store),
        })
    }

    fn lock_store(&self) -> Result<MutexGuard<'_, RSDB<'static>>, StorageError> {
        self.store.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: icefalldb snapshot store mutex was poisoned",
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

    fn set_bytes(
        &self,
        store: &mut RSDB<'static>,
        key: &[u8],
        value: &[u8],
        operation: &str,
    ) -> Result<(), StorageError> {
        // SAFETY: `RSDB::set` immediately copies the provided key/value bytes into its log
        // and in-memory map. It does not retain the input references after the call returns.
        let key: &'static [u8] = unsafe { std::mem::transmute(key) };
        let value: &'static [u8] = unsafe { std::mem::transmute(value) };

        store
            .set(key, value)
            .map_err(|error| self.map_error(operation, error))
    }

    fn load_catalog(&self, store: &RSDB<'static>) -> Result<Vec<String>, StorageError> {
        let Some(payload) = store
            .get(SNAPSHOT_CATALOG_KEY)
            .map_err(|error| self.map_error("read icefalldb catalog", error))?
        else {
            return Ok(Vec::new());
        };

        serde_json::from_slice(payload).map_err(|_| {
            StorageError::Io(format!(
                "{}: icefalldb snapshot catalog is corrupt",
                self.path.display()
            ))
        })
    }

    fn save_catalog(
        &self,
        store: &mut RSDB<'static>,
        catalog: &[String],
    ) -> Result<(), StorageError> {
        let payload = serde_json::to_vec(catalog)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        self.set_bytes(
            store,
            SNAPSHOT_CATALOG_KEY,
            &payload,
            "write icefalldb catalog",
        )
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

    fn read_snapshot_payload(
        &self,
        store: &RSDB<'static>,
        doc_id: &Uuid,
    ) -> Result<Option<Vec<u8>>, StorageError> {
        let Some(payload) = store
            .get(doc_id.to_string().as_bytes())
            .map_err(|error| self.map_error("read icefalldb snapshot", error))?
        else {
            return Ok(None);
        };

        if payload.as_slice() == SNAPSHOT_TOMBSTONE_VALUE {
            return Ok(None);
        }

        Ok(Some(payload.clone()))
    }
}

impl SnapshotStore for IcefalldbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let store = self.lock_store()?;
        let Some(payload) = self.read_snapshot_payload(&store, doc_id)? else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &payload).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let mut store = self.lock_store()?;
        let doc_id = snapshot.document.id;
        let doc_id_key = doc_id.to_string();
        let payload = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize icefalldb snapshot `{doc_id}`: {error}"
            ))
        })?;
        let mut catalog = self.load_catalog(&store)?;

        self.set_bytes(
            &mut store,
            doc_id_key.as_bytes(),
            &payload,
            "write icefalldb snapshot",
        )?;

        if !catalog.iter().any(|entry| entry == &doc_id_key) {
            catalog.push(doc_id_key);
            catalog.sort();
        }

        self.save_catalog(&mut store, &catalog)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut store = self.lock_store()?;
        let doc_id_key = doc_id.to_string();
        let mut catalog = self.load_catalog(&store)?;

        self.set_bytes(
            &mut store,
            doc_id_key.as_bytes(),
            SNAPSHOT_TOMBSTONE_VALUE,
            "delete icefalldb snapshot",
        )?;
        catalog.retain(|entry| entry != &doc_id_key);

        self.save_catalog(&mut store, &catalog)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let store = self.lock_store()?;
        let catalog = self.load_catalog(&store)?;
        let mut documents = Vec::new();

        for doc_id_key in catalog {
            let Ok(doc_id) = Uuid::parse_str(&doc_id_key) else {
                continue;
            };

            match self.read_snapshot_payload(&store, &doc_id)? {
                Some(payload) => match self.deserialize_snapshot(doc_id, &payload) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping corrupt icefalldb snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                },
                None => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing icefalldb snapshot while building document catalog"
                ),
            }
        }

        Ok(documents)
    }
}
