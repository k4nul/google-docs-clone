use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard},
};

use surrealkv::{
    BytewiseComparator,
    bplustree::tree::{BPlusTree, Durability},
};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

pub struct SurrealkvSnapshotStore {
    path: PathBuf,
    tree: Mutex<BPlusTree<File>>,
}

impl SurrealkvSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_SURREALKV_PATH cannot be empty when SNAPSHOT_STORE=surrealkv".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let mut tree = BPlusTree::disk(&path, Arc::new(BytewiseComparator::default()))
            .map_err(|error| Self::map_database_error(&path, error))?;
        tree.set_durability(Durability::Always);

        Ok(Self {
            path,
            tree: Mutex::new(tree),
        })
    }

    fn lock_tree(&self) -> Result<MutexGuard<'_, BPlusTree<File>>, StorageError> {
        self.tree.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: surrealkv tree mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn map_database_error(path: &Path, error: impl std::fmt::Display) -> StorageError {
        StorageError::Io(format!("{}: {error}", path.display()))
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

impl SnapshotStore for SurrealkvSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let tree = self.lock_tree()?;
        let Some(bytes) = tree
            .get(doc_id.as_bytes())
            .map_err(|error| Self::map_database_error(&self.path, error))?
        else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, bytes.as_ref()).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize surrealkv snapshot `{doc_id}`: {error}"
            ))
        })?;
        let mut tree = self.lock_tree()?;
        tree.insert(doc_id.as_bytes(), bytes.as_slice())
            .map_err(|error| Self::map_database_error(&self.path, error))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut tree = self.lock_tree()?;
        tree.delete(doc_id.as_bytes())
            .map(|_| ())
            .map_err(|error| Self::map_database_error(&self.path, error))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let tree = self.lock_tree()?;
        let entries = tree
            .range(..)
            .map_err(|error| Self::map_database_error(&self.path, error))?;
        let mut documents = Vec::new();

        for entry in entries {
            let (key, value) =
                entry.map_err(|error| Self::map_database_error(&self.path, error))?;
            let Ok(doc_id) = Uuid::from_slice(key.as_ref()) else {
                continue;
            };

            match self.deserialize_snapshot(doc_id, value.as_ref()) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt surrealkv snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
