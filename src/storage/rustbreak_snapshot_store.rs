use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use rustbreak::{PathDatabase, deser::Bincode, error::RustbreakError};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

type SnapshotCatalog = HashMap<String, PersistedSnapshot>;

pub struct RustbreakSnapshotStore {
    path: PathBuf,
    database: PathDatabase<SnapshotCatalog, Bincode>,
}

impl RustbreakSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_RUSTBREAK_PATH cannot be empty when SNAPSHOT_STORE=rustbreak".to_owned(),
            ));
        }

        let parent_dir = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        fs::create_dir_all(&parent_dir)
            .map_err(|error| StorageError::Io(format!("{}: {error}", parent_dir.display())))?;

        let database = PathDatabase::load_from_path_or_default(path.clone())
            .map_err(|error| Self::map_database_error(&path, error))?;

        Ok(Self { path, database })
    }

    fn map_database_error(path: &Path, error: RustbreakError) -> StorageError {
        StorageError::Io(format!("{}: {error}", path.display()))
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

impl SnapshotStore for RustbreakSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let key = doc_id.to_string();
        let snapshot = self
            .database
            .read(|catalog| catalog.get(&key).cloned())
            .map_err(|error| Self::map_database_error(&self.path, error))?;
        let Some(snapshot) = snapshot else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, snapshot).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id.to_string();
        let persisted_snapshot = PersistedSnapshot::from(snapshot);

        self.database
            .write(|catalog| {
                catalog.insert(doc_id, persisted_snapshot);
            })
            .map_err(|error| Self::map_database_error(&self.path, error))?;
        self.database
            .save()
            .map_err(|error| Self::map_database_error(&self.path, error))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let key = doc_id.to_string();
        self.database
            .write(|catalog| {
                catalog.remove(&key);
            })
            .map_err(|error| Self::map_database_error(&self.path, error))?;
        self.database
            .save()
            .map_err(|error| Self::map_database_error(&self.path, error))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut documents = Vec::new();
        let catalog = self
            .database
            .read(|catalog| catalog.values().cloned().collect::<Vec<_>>())
            .map_err(|error| Self::map_database_error(&self.path, error))?;

        for snapshot in catalog {
            let doc_id = snapshot.document.id;

            match self.deserialize_snapshot(doc_id, snapshot) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt rustbreak snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
