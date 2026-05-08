use std::{fs, path::PathBuf};

use bitkv_rs::{
    db::Engine,
    errors::Errors as BitkvError,
    option::{IndexType, Options},
};
use bytes::Bytes;
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

pub struct BitkvRsSnapshotStore {
    path: PathBuf,
    engine: Engine,
}

impl BitkvRsSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_BITKV_RS_PATH cannot be empty when SNAPSHOT_STORE=bitkv_rs".to_owned(),
            ));
        }

        fs::create_dir_all(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        let options = Options {
            dir_path: path.clone(),
            sync_writes: true,
            mmap_at_startup: false,
            index_type: IndexType::BTree,
            ..Options::default()
        };
        let engine = Engine::open(options)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self { path, engine })
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

    fn key(doc_id: &Uuid) -> Bytes {
        Bytes::from(doc_id.to_string())
    }
}

impl Drop for BitkvRsSnapshotStore {
    fn drop(&mut self) {
        if let Err(error) = self.engine.close() {
            tracing::warn!(
                path = %self.path.display(),
                error = %error,
                "failed to close bitkv-rs snapshot store"
            );
        }
    }
}

impl SnapshotStore for BitkvRsSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let payload = match self.engine.get(Self::key(doc_id)) {
            Ok(payload) => payload,
            Err(BitkvError::KeyNotFound) => return Ok(None),
            Err(error) => {
                return Err(StorageError::Io(format!(
                    "{}: failed to load bitkv-rs snapshot `{doc_id}`: {error}",
                    self.path.display()
                )));
            }
        };

        self.deserialize_snapshot(*doc_id, &payload).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize bitkv-rs snapshot `{doc_id}`: {error}"
            ))
        })?;

        self.engine
            .put(Self::key(&doc_id), Bytes::from(payload))
            .map_err(|error| {
                StorageError::Io(format!(
                    "{}: failed to save bitkv-rs snapshot `{doc_id}`: {error}",
                    self.path.display()
                ))
            })?;
        self.engine
            .sync()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        self.engine.delete(Self::key(doc_id)).map_err(|error| {
            StorageError::Io(format!(
                "{}: failed to delete bitkv-rs snapshot `{doc_id}`: {error}",
                self.path.display()
            ))
        })?;
        self.engine
            .sync()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let keys = self.engine.list_keys().map_err(|error| {
            StorageError::Io(format!(
                "{}: failed to list bitkv-rs snapshot keys: {error}",
                self.path.display()
            ))
        })?;
        let mut documents = Vec::new();

        for key in keys {
            let Ok(doc_id_key) = std::str::from_utf8(&key) else {
                continue;
            };
            let Ok(doc_id) = Uuid::parse_str(doc_id_key) else {
                continue;
            };

            let payload = self.engine.get(Bytes::from(key.to_vec())).map_err(|error| {
                StorageError::Io(format!(
                    "{}: failed to load bitkv-rs snapshot `{doc_id}` while listing catalog: {error}",
                    self.path.display()
                ))
            })?;
            match self.deserialize_snapshot(doc_id, &payload) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt bitkv-rs snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
