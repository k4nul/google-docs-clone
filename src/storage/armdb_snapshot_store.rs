use std::{
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use armdb::{Config as ArmdbConfig, VarTree};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

pub struct ArmdbSnapshotStore {
    path: PathBuf,
    tree: Mutex<VarTree<[u8; 16]>>,
}

impl ArmdbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_ARMDB_PATH cannot be empty when SNAPSHOT_STORE=armdb".to_owned(),
            ));
        }

        ensure_snapshot_dir(&path)?;

        let mut config = ArmdbConfig::default();
        config.enable_fsync = true;
        config.hints = true;
        config.reversed = false;
        config.shard_count = 2;

        let tree = VarTree::<[u8; 16]>::open(&path, config)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            tree: Mutex::new(tree),
        })
    }

    fn snapshot_key(doc_id: &Uuid) -> [u8; 16] {
        *doc_id.as_bytes()
    }

    fn lock_tree(&self) -> Result<MutexGuard<'_, VarTree<[u8; 16]>>, StorageError> {
        self.tree
            .lock()
            .map_err(|_| StorageError::Io(format!("{}: armdb mutex poisoned", self.path.display())))
    }

    fn map_error(&self, action: &str, error: armdb::DbError) -> StorageError {
        StorageError::Io(format!("{}: {action} failed: {error}", self.path.display()))
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
}

impl SnapshotStore for ArmdbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let tree = self.lock_tree()?;
        let Some(payload) = tree.get(&Self::snapshot_key(doc_id)) else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, payload.as_bytes())
            .map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize armdb snapshot `{doc_id}`: {error}"
            ))
        })?;

        let tree = self.lock_tree()?;
        tree.put(&Self::snapshot_key(&doc_id), &payload)
            .map_err(|error| self.map_error("write armdb snapshot", error))?;
        tree.flush_buffers()
            .map_err(|error| self.map_error("flush armdb snapshot store", error))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let tree = self.lock_tree()?;
        tree.delete(&Self::snapshot_key(doc_id))
            .map_err(|error| self.map_error("delete armdb snapshot", error))?;
        tree.flush_buffers()
            .map_err(|error| self.map_error("flush armdb snapshot store", error))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let tree = self.lock_tree()?;
        let mut documents = Vec::new();

        for (key, payload) in tree.iter() {
            let doc_id = Uuid::from_bytes(key);
            match self.deserialize_snapshot(doc_id, payload.as_bytes()) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt armdb snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
