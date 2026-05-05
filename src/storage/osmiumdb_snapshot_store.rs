use std::{path::PathBuf, sync::Arc};

use osmiumdb::{Engine, EngineConfig, EngineError};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

const SNAPSHOT_CATALOG_KEY: &[u8] = b"__catalog__";
const SNAPSHOT_KEY_PREFIX: &str = "snapshot:";

pub struct OsmiumdbSnapshotStore {
    path: PathBuf,
    database: Arc<Engine>,
}

impl OsmiumdbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_OSMIUMDB_PATH cannot be empty when SNAPSHOT_STORE=osmiumdb".to_owned(),
            ));
        }

        ensure_snapshot_dir(&path)?;
        let database = Engine::open(EngineConfig::new(path.clone()).sync_writes(true))
            .map_err(|error| Self::map_error(&path, "open osmiumdb snapshot store", error))?;

        Ok(Self { path, database })
    }

    fn snapshot_key(doc_id: &Uuid) -> Vec<u8> {
        format!("{SNAPSHOT_KEY_PREFIX}{doc_id}").into_bytes()
    }

    fn map_error(path: &std::path::Path, operation: &str, error: EngineError) -> StorageError {
        StorageError::Io(format!(
            "{}: failed to {operation}: {error}",
            path.display()
        ))
    }

    fn read_catalog(&self) -> Result<Vec<Uuid>, StorageError> {
        let Some(payload) =
            self.read_value(SNAPSHOT_CATALOG_KEY, "read osmiumdb snapshot catalog")?
        else {
            return Ok(Vec::new());
        };

        serde_json::from_slice::<Vec<Uuid>>(&payload).map_err(|_| {
            StorageError::Io(format!(
                "{}: osmiumdb snapshot catalog is corrupt",
                self.path.display()
            ))
        })
    }

    fn serialize_catalog(&self, catalog: &[Uuid]) -> Result<Vec<u8>, StorageError> {
        serde_json::to_vec(catalog).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize osmiumdb snapshot catalog: {error}"
            ))
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

    fn read_value(&self, key: &[u8], operation: &str) -> Result<Option<Vec<u8>>, StorageError> {
        match self.database.get(key) {
            Ok(value) => Ok(value),
            Err(EngineError::KeyNotFound) => Ok(None),
            Err(error) => Err(Self::map_error(&self.path, operation, error)),
        }
    }

    // Flush seals the active buffer and syncs the WAL; checkpoint persists the
    // recovered map snapshot so reopen semantics do not depend on replay alone.
    fn commit(&self) -> Result<(), StorageError> {
        self.database
            .flush()
            .map_err(|error| Self::map_error(&self.path, "flush osmiumdb snapshot store", error))?;
        self.database.checkpoint().map_err(|error| {
            Self::map_error(&self.path, "checkpoint osmiumdb snapshot store", error)
        })
    }
}

impl SnapshotStore for OsmiumdbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let Some(payload) =
            self.read_value(&Self::snapshot_key(doc_id), "read osmiumdb snapshot")?
        else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &payload).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize osmiumdb snapshot `{doc_id}`: {error}"
            ))
        })?;

        let mut catalog = self.read_catalog()?;
        self.database
            .put(&Self::snapshot_key(&doc_id), &payload)
            .map_err(|error| Self::map_error(&self.path, "write osmiumdb snapshot", error))?;

        if !catalog.contains(&doc_id) {
            catalog.push(doc_id);
            catalog.sort_unstable();
        }

        self.database
            .put(SNAPSHOT_CATALOG_KEY, &self.serialize_catalog(&catalog)?)
            .map_err(|error| {
                Self::map_error(&self.path, "write osmiumdb snapshot catalog", error)
            })?;

        self.commit()
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut catalog = self.read_catalog()?;
        catalog.retain(|catalog_doc_id| catalog_doc_id != doc_id);

        self.database
            .delete(&Self::snapshot_key(doc_id))
            .map_err(|error| Self::map_error(&self.path, "delete osmiumdb snapshot", error))?;
        self.database
            .put(SNAPSHOT_CATALOG_KEY, &self.serialize_catalog(&catalog)?)
            .map_err(|error| {
                Self::map_error(&self.path, "write osmiumdb snapshot catalog", error)
            })?;

        self.commit()
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let catalog = self.read_catalog()?;
        let mut documents = Vec::new();

        for doc_id in catalog {
            match self.read_value(&Self::snapshot_key(&doc_id), "read osmiumdb snapshot") {
                Ok(Some(payload)) => match self.deserialize_snapshot(doc_id, &payload) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping corrupt osmiumdb snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                },
                Ok(None) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing osmiumdb snapshot referenced by catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
