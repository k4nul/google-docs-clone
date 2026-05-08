use std::{fs, path::PathBuf};

use hightower_kv::{KvEngine, SingleNodeEngine, StoreConfig};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOT_PREFIX: &str = "snapshot:";

pub struct HightowerKvSnapshotStore {
    path: PathBuf,
    engine: SingleNodeEngine,
}

impl HightowerKvSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_HIGHTOWER_KV_PATH cannot be empty when SNAPSHOT_STORE=hightower_kv"
                    .to_owned(),
            ));
        }

        fs::create_dir_all(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        let config = StoreConfig {
            data_dir: path.to_string_lossy().into_owned(),
            worker_threads: 0,
            ..StoreConfig::default()
        };
        let engine = SingleNodeEngine::with_config(config)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self { path, engine })
    }

    fn snapshot_key(doc_id: &Uuid) -> String {
        format!("{SNAPSHOT_PREFIX}{doc_id}")
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

    fn flush(&self) -> Result<(), StorageError> {
        self.engine
            .flush()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }
}

impl SnapshotStore for HightowerKvSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let Some(bytes) = self
            .engine
            .get(Self::snapshot_key(doc_id).as_bytes())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?
        else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &bytes).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize hightower_kv snapshot `{doc_id}`: {error}"
            ))
        })?;

        self.engine
            .put(Self::snapshot_key(&doc_id).into_bytes(), bytes)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        self.flush()
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        self.engine
            .delete(Self::snapshot_key(doc_id).into_bytes())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        self.flush()
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let entries = self
            .engine
            .get_prefix(SNAPSHOT_PREFIX.as_bytes())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        let mut documents = Vec::new();

        for (key, value) in entries {
            let Ok(key) = String::from_utf8(key) else {
                continue;
            };
            let Some(doc_id_key) = key.strip_prefix(SNAPSHOT_PREFIX) else {
                continue;
            };
            let Ok(doc_id) = Uuid::parse_str(doc_id_key) else {
                continue;
            };

            match self.deserialize_snapshot(doc_id, &value) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt hightower_kv snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
