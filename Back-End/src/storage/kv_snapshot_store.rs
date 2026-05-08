use std::{fs, path::PathBuf};

use kv::{Config as KvConfig, Json, Store};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOT_BUCKET: &str = "snapshots";

pub struct KvSnapshotStore {
    path: PathBuf,
    store: Store,
}

impl KvSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_KV_PATH cannot be empty when SNAPSHOT_STORE=kv".to_owned(),
            ));
        }

        fs::create_dir_all(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        let store = Store::new(KvConfig::new(&path).flush_every_ms(0))
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self { path, store })
    }

    fn bucket(&self) -> Result<kv::Bucket<'_, String, Json<PersistedSnapshot>>, StorageError> {
        self.store
            .bucket(Some(SNAPSHOT_BUCKET))
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        snapshot: PersistedSnapshot,
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot: DocumentSnapshot = snapshot.into();
        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }
}

impl SnapshotStore for KvSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let bucket = self.bucket()?;
        let snapshot = bucket
            .get(&doc_id.to_string())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        let Some(snapshot) = snapshot else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, snapshot.0).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let bucket = self.bucket()?;
        bucket
            .set(
                &doc_id.to_string(),
                &Json(PersistedSnapshot::from(snapshot)),
            )
            .map(|_| ())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let bucket = self.bucket()?;
        bucket
            .remove(&doc_id.to_string())
            .map(|_| ())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let bucket = self.bucket()?;
        let mut documents = Vec::new();

        for item in bucket.iter() {
            let item = item
                .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
            let doc_id_key: String = item
                .key()
                .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
            let Ok(doc_id) = Uuid::parse_str(&doc_id_key) else {
                continue;
            };
            let snapshot: Json<PersistedSnapshot> = item
                .value()
                .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

            match self.deserialize_snapshot(doc_id, snapshot.0) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt kv snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
