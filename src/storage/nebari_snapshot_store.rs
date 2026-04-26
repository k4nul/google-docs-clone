use std::{fs, path::PathBuf};

use nebari::{ArcBytes, Config as NebariConfig, Tree, io::fs::StdFile, tree::Unversioned};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

type NebariTree = Tree<Unversioned, StdFile>;

pub struct NebariSnapshotStore {
    path: PathBuf,
    tree: NebariTree,
}

impl NebariSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_NEBARI_PATH cannot be empty when SNAPSHOT_STORE=nebari".to_owned(),
            ));
        }

        fs::create_dir_all(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        let roots = NebariConfig::default_for(&path).open().map_err(|error| {
            StorageError::Io(format!(
                "{}: failed to open nebari snapshot store: {error}",
                path.display()
            ))
        })?;
        let tree = roots
            .tree(Unversioned::tree("snapshots"))
            .map_err(|error| {
                StorageError::Io(format!(
                    "{}: failed to open nebari snapshot tree: {error}",
                    path.display()
                ))
            })?;

        Ok(Self { path, tree })
    }

    fn map_nebari_error(&self, operation: &str, error: nebari::Error) -> StorageError {
        StorageError::Io(format!(
            "{}: failed to {operation}: {error}",
            self.path.display()
        ))
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

impl SnapshotStore for NebariSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let key = doc_id.to_string();
        let Some(bytes) = self
            .tree
            .get(key.as_bytes())
            .map_err(|error| self.map_nebari_error("read nebari snapshot", error))?
        else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, bytes.as_ref()).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let key = doc_id.to_string();
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize nebari snapshot `{doc_id}`: {error}"
            ))
        })?;

        self.tree
            .set(ArcBytes::from(key.into_bytes()), ArcBytes::from(bytes))
            .map_err(|error| self.map_nebari_error("write nebari snapshot", error))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let key = doc_id.to_string();
        self.tree
            .remove(key.as_bytes())
            .map_err(|error| self.map_nebari_error("delete nebari snapshot", error))?;
        Ok(())
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut documents = Vec::new();
        for (key, value) in self
            .tree
            .get_range(&(..))
            .map_err(|error| self.map_nebari_error("scan nebari snapshot catalog", error))?
        {
            let Ok(key) = std::str::from_utf8(key.as_ref()) else {
                continue;
            };
            let Ok(doc_id) = Uuid::parse_str(key) else {
                continue;
            };

            match self.deserialize_snapshot(doc_id, value.as_ref()) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt nebari snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
