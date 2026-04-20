use std::{fs, path::PathBuf};

use uuid::Uuid;
use yakv::storage::{Select, Storage, StorageConfig};

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOT_KEY_PREFIX: &str = "snapshot:";

pub struct YakvSnapshotStore {
    path: PathBuf,
    store: Storage,
}

impl YakvSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_YAKV_PATH cannot be empty when SNAPSHOT_STORE=yakv".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let store = Storage::open(&path, StorageConfig::default()).map_err(|error| {
            StorageError::Io(format!(
                "{}: failed to open yakv snapshot store: {error}",
                path.display()
            ))
        })?;

        Ok(Self { path, store })
    }

    fn map_yakv_error(&self, operation: &str, error: anyhow::Error) -> StorageError {
        StorageError::Io(format!(
            "{}: failed to {operation}: {error}",
            self.path.display()
        ))
    }

    fn key_for_doc_id(doc_id: &Uuid) -> Vec<u8> {
        format!("{SNAPSHOT_KEY_PREFIX}{doc_id}").into_bytes()
    }

    fn doc_id_from_key(key: &[u8]) -> Option<Uuid> {
        let key = std::str::from_utf8(key).ok()?;
        let doc_id = key.strip_prefix(SNAPSHOT_KEY_PREFIX)?;
        Uuid::parse_str(doc_id).ok()
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

impl SnapshotStore for YakvSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let key = Self::key_for_doc_id(doc_id);
        let Some(bytes) = self
            .store
            .get(&key)
            .map_err(|error| self.map_yakv_error("read yakv snapshot", error))?
        else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &bytes).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize yakv snapshot `{doc_id}`: {error}"
            ))
        })?;

        self.store
            .put(Self::key_for_doc_id(&doc_id), bytes)
            .map_err(|error| self.map_yakv_error("write yakv snapshot", error))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        self.store
            .remove(Self::key_for_doc_id(doc_id))
            .map_err(|error| self.map_yakv_error("delete yakv snapshot", error))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut documents = Vec::new();

        for entry in self.store.iter() {
            let (key, bytes) =
                entry.map_err(|error| self.map_yakv_error("scan yakv snapshot catalog", error))?;
            let Some(doc_id) = Self::doc_id_from_key(&key) else {
                continue;
            };

            match self.deserialize_snapshot(doc_id, &bytes) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt yakv snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
