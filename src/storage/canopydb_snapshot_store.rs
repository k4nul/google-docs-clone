use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use canopydb::{Database, Error as CanopydbError};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOTS_TREE: &[u8] = b"snapshots";

pub struct CanopydbSnapshotStore {
    path: PathBuf,
    database: Mutex<Database>,
}

impl CanopydbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_CANOPYDB_PATH cannot be empty when SNAPSHOT_STORE=canopydb".to_owned(),
            ));
        }

        fs::create_dir_all(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        let database =
            Database::new(&path).map_err(|error| Self::map_database_error(&path, error))?;

        Ok(Self {
            path,
            database: Mutex::new(database),
        })
    }

    fn lock_database(&self) -> Result<MutexGuard<'_, Database>, StorageError> {
        self.database.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: canopydb database mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn map_database_error(path: &PathBuf, error: CanopydbError) -> StorageError {
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

impl SnapshotStore for CanopydbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let database = self.lock_database()?;
        let transaction = database
            .begin_read()
            .map_err(|error| Self::map_database_error(&self.path, error))?;
        let Some(tree) = transaction
            .get_tree(SNAPSHOTS_TREE)
            .map_err(|error| Self::map_database_error(&self.path, error))?
        else {
            return Ok(None);
        };
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
                "failed to serialize canopydb snapshot `{doc_id}`: {error}"
            ))
        })?;
        let database = self.lock_database()?;
        let transaction = database
            .begin_write()
            .map_err(|error| Self::map_database_error(&self.path, error))?;
        {
            let mut tree = transaction
                .get_or_create_tree(SNAPSHOTS_TREE)
                .map_err(|error| Self::map_database_error(&self.path, error))?;
            tree.insert(doc_id.as_bytes(), &bytes)
                .map_err(|error| Self::map_database_error(&self.path, error))?;
        }
        transaction
            .commit_with(true)
            .map(|_| ())
            .map_err(|error| Self::map_database_error(&self.path, error))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let database = self.lock_database()?;
        let transaction = database
            .begin_write()
            .map_err(|error| Self::map_database_error(&self.path, error))?;
        {
            let mut tree = transaction
                .get_or_create_tree(SNAPSHOTS_TREE)
                .map_err(|error| Self::map_database_error(&self.path, error))?;
            tree.delete(doc_id.as_bytes())
                .map_err(|error| Self::map_database_error(&self.path, error))?;
        }
        transaction
            .commit_with(true)
            .map(|_| ())
            .map_err(|error| Self::map_database_error(&self.path, error))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let database = self.lock_database()?;
        let transaction = database
            .begin_read()
            .map_err(|error| Self::map_database_error(&self.path, error))?;
        let Some(tree) = transaction
            .get_tree(SNAPSHOTS_TREE)
            .map_err(|error| Self::map_database_error(&self.path, error))?
        else {
            return Ok(Vec::new());
        };
        let entries = tree
            .iter()
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
                    "skipping corrupt canopydb snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
