use std::{
    io,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use scdb::Store;
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOT_CATALOG_KEY: &[u8] = b"__catalog__";

pub struct ScdbSnapshotStore {
    path: PathBuf,
    store: Mutex<Store>,
}

impl ScdbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_SCDB_PATH cannot be empty when SNAPSHOT_STORE=scdb".to_owned(),
            ));
        }

        let store = Store::new(
            path.to_string_lossy().as_ref(),
            None,
            None,
            None,
            Some(0),
            false,
        )
        .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            store: Mutex::new(store),
        })
    }

    fn lock_store(&self) -> Result<MutexGuard<'_, Store>, StorageError> {
        self.store.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: scdb store mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn map_io_error(&self, operation: &str, error: io::Error) -> StorageError {
        StorageError::Io(format!(
            "{}: failed to {operation}: {error}",
            self.path.display()
        ))
    }

    fn load_catalog(&self, store: &mut Store) -> Result<Vec<String>, StorageError> {
        let Some(bytes) = store
            .get(SNAPSHOT_CATALOG_KEY)
            .map_err(|error| self.map_io_error("read scdb catalog", error))?
        else {
            return Ok(Vec::new());
        };

        serde_json::from_slice::<Vec<String>>(&bytes).map_err(|_| {
            StorageError::Io(format!(
                "{}: snapshot catalog is corrupt",
                self.path.display()
            ))
        })
    }

    fn save_catalog(&self, store: &mut Store, catalog: &[String]) -> Result<(), StorageError> {
        let bytes = serde_json::to_vec(catalog)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        store
            .set(SNAPSHOT_CATALOG_KEY, &bytes, None)
            .map_err(|error| self.map_io_error("write scdb catalog", error))
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        bytes: &[u8],
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot = serde_json::from_slice::<PersistedSnapshot>(bytes)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot: DocumentSnapshot = snapshot.into();

        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }
}

impl SnapshotStore for ScdbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let mut store = self.lock_store()?;
        let Some(bytes) = store
            .get(doc_id.to_string().as_bytes())
            .map_err(|error| self.map_io_error("read scdb snapshot", error))?
        else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &bytes).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let key = doc_id.to_string();
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize scdb snapshot `{doc_id}`: {error}"
            ))
        })?;
        let mut store = self.lock_store()?;
        let mut catalog = self.load_catalog(&mut store)?;

        store
            .set(key.as_bytes(), &bytes, None)
            .map_err(|error| self.map_io_error("write scdb snapshot", error))?;
        if !catalog.iter().any(|value| value == &key) {
            catalog.push(key);
            catalog.sort();
        }

        self.save_catalog(&mut store, &catalog)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let key = doc_id.to_string();
        let mut store = self.lock_store()?;
        let mut catalog = self.load_catalog(&mut store)?;

        store
            .delete(key.as_bytes())
            .map_err(|error| self.map_io_error("delete scdb snapshot", error))?;
        catalog.retain(|value| value != &key);

        self.save_catalog(&mut store, &catalog)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut store = self.lock_store()?;
        let mut documents = Vec::new();
        let catalog = self.load_catalog(&mut store)?;

        for doc_id_key in catalog {
            let Ok(doc_id) = Uuid::parse_str(&doc_id_key) else {
                continue;
            };

            match store
                .get(doc_id_key.as_bytes())
                .map_err(|error| self.map_io_error("read scdb snapshot", error))?
            {
                Some(bytes) => match self.deserialize_snapshot(doc_id, &bytes) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping corrupt scdb snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                },
                None => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing scdb snapshot while building document catalog"
                ),
            }
        }

        Ok(documents)
    }
}
