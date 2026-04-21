use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use thetadb::{Error as ThetadbError, ThetaDB};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

pub struct ThetadbSnapshotStore {
    path: PathBuf,
    database: Mutex<ThetaDB>,
}

impl ThetadbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_THETADB_PATH cannot be empty when SNAPSHOT_STORE=thetadb".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let database = ThetaDB::open(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            database: Mutex::new(database),
        })
    }

    fn lock_database(&self) -> Result<MutexGuard<'_, ThetaDB>, StorageError> {
        self.database.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: thetadb database mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn map_error(&self, error: ThetadbError) -> StorageError {
        StorageError::Io(format!("{}: {error}", self.path.display()))
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

impl SnapshotStore for ThetadbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let database = self.lock_database()?;
        let snapshot = database
            .get(doc_id.as_bytes())
            .map_err(|error| self.map_error(error))?;
        let Some(snapshot) = snapshot else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &snapshot).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize thetadb snapshot `{doc_id}`: {error}"
            ))
        })?;
        let database = self.lock_database()?;
        database
            .put(doc_id.as_bytes(), bytes)
            .map_err(|error| self.map_error(error))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let database = self.lock_database()?;
        database
            .delete(doc_id.as_bytes())
            .map_err(|error| self.map_error(error))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let database = self.lock_database()?;
        let mut cursor = database
            .first_cursor()
            .map_err(|error| self.map_error(error))?;
        let mut documents = Vec::new();

        while let Some((key, value)) = cursor.key_value().map_err(|error| self.map_error(error))? {
            let Ok(doc_id) = Uuid::from_slice(&key) else {
                cursor.next().map_err(|error| self.map_error(error))?;
                continue;
            };

            match self.deserialize_snapshot(doc_id, &value) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt thetadb snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }

            cursor.next().map_err(|error| self.map_error(error))?;
        }

        Ok(documents)
    }
}
