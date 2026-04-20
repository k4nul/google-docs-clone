use std::path::PathBuf;

use candystore::{CandyStore, Config as CandyConfig, Error as CandyError};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

const SNAPSHOT_CATALOG_KEY: &[u8] = b"__catalog__";

pub struct CandystoreSnapshotStore {
    path: PathBuf,
    store: CandyStore,
}

impl CandystoreSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_CANDYSTORE_PATH cannot be empty when SNAPSHOT_STORE=candystore"
                    .to_owned(),
            ));
        }

        ensure_snapshot_dir(&path)?;
        let store = CandyStore::open(&path, CandyConfig::default()).map_err(|error| {
            Self::map_candystore_error(&path, "open candystore snapshot store", error)
        })?;

        Ok(Self { path, store })
    }

    fn map_candystore_error(
        path: &std::path::Path,
        operation: &str,
        error: CandyError,
    ) -> StorageError {
        StorageError::Io(format!(
            "{}: failed to {operation}: {error}",
            path.display()
        ))
    }

    fn load_catalog(&self) -> Result<Vec<String>, StorageError> {
        let Some(bytes) = self.store.get(SNAPSHOT_CATALOG_KEY).map_err(|error| {
            Self::map_candystore_error(&self.path, "read candystore catalog", error)
        })?
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

    fn save_catalog(&self, catalog: &[String]) -> Result<(), StorageError> {
        let bytes = serde_json::to_vec(catalog)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        self.store
            .set(SNAPSHOT_CATALOG_KEY, &bytes)
            .map_err(|error| {
                Self::map_candystore_error(&self.path, "write candystore catalog", error)
            })?;
        self.sync("persist candystore catalog")
    }

    fn sync(&self, operation: &str) -> Result<(), StorageError> {
        self.store
            .flush()
            .map_err(|error| Self::map_candystore_error(&self.path, operation, error))?;
        self.store
            .checkpoint()
            .map_err(|error| Self::map_candystore_error(&self.path, operation, error))
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

impl SnapshotStore for CandystoreSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let Some(bytes) = self
            .store
            .get_big(doc_id.to_string().as_bytes())
            .map_err(|error| {
                Self::map_candystore_error(&self.path, "read candystore snapshot", error)
            })?
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
                "failed to serialize candystore snapshot `{doc_id}`: {error}"
            ))
        })?;
        let mut catalog = self.load_catalog()?;

        self.store
            .set_big(key.as_bytes(), &bytes)
            .map_err(|error| {
                Self::map_candystore_error(&self.path, "write candystore snapshot", error)
            })?;
        if !catalog.iter().any(|value| value == &key) {
            catalog.push(key);
            catalog.sort();
        }

        self.save_catalog(&catalog)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let key = doc_id.to_string();
        let mut catalog = self.load_catalog()?;

        self.store.remove_big(key.as_bytes()).map_err(|error| {
            Self::map_candystore_error(&self.path, "delete candystore snapshot", error)
        })?;
        catalog.retain(|value| value != &key);

        self.save_catalog(&catalog)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let catalog = self.load_catalog()?;
        let mut documents = Vec::new();

        for doc_id_key in catalog {
            let Ok(doc_id) = Uuid::parse_str(&doc_id_key) else {
                continue;
            };

            match self.store.get_big(doc_id_key.as_bytes()).map_err(|error| {
                Self::map_candystore_error(&self.path, "read candystore snapshot", error)
            })? {
                Some(bytes) => match self.deserialize_snapshot(doc_id, &bytes) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping corrupt candystore snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                },
                None => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing candystore snapshot while building document catalog"
                ),
            }
        }

        Ok(documents)
    }
}
