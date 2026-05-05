use std::{fs, path::PathBuf};

use persistent_kv::{Config as PersistentKvConfig, PersistentKeyValueStore};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

pub struct PersistentKvSnapshotStore {
    path: PathBuf,
    store: PersistentKeyValueStore<String, Vec<u8>>,
}

impl PersistentKvSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_PERSISTENT_KV_PATH cannot be empty when SNAPSHOT_STORE=persistent_kv"
                    .to_owned(),
            ));
        }

        fs::create_dir_all(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        let store = PersistentKeyValueStore::new(&path, PersistentKvConfig::default())
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self { path, store })
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        raw_snapshot: &[u8],
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot = serde_json::from_slice::<PersistedSnapshot>(raw_snapshot)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot: DocumentSnapshot = snapshot.into();

        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }
}

impl SnapshotStore for PersistentKvSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let doc_id_key = doc_id.to_string();
        let Some(raw_snapshot) = self.store.get(doc_id_key.as_str()) else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &raw_snapshot).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let doc_id_key = doc_id.to_string();
        let raw_snapshot =
            serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
                StorageError::Io(format!(
                    "failed to serialize persistent_kv snapshot `{doc_id}`: {error}"
                ))
            })?;

        self.store
            .set(doc_id_key, raw_snapshot)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let doc_id_key = doc_id.to_string();

        self.store
            .unset(doc_id_key.as_str())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut documents = Vec::new();

        for (doc_id_key, raw_snapshot) in self.store.iter() {
            let Ok(doc_id) = Uuid::parse_str(&doc_id_key) else {
                continue;
            };

            match self.deserialize_snapshot(doc_id, &raw_snapshot) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt persistent_kv snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
