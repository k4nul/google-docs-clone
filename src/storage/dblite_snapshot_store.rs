use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use dblite::Database;
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

pub struct DbliteSnapshotStore {
    path: PathBuf,
    database: Mutex<Database>,
}

impl DbliteSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_DBLITE_PATH cannot be empty when SNAPSHOT_STORE=dblite".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let database = Database::open_or_create(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            database: Mutex::new(database),
        })
    }

    fn lock_database(&self) -> Result<MutexGuard<'_, Database>, StorageError> {
        self.database.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: dblite database mutex was poisoned",
                self.path.display()
            ))
        })
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

    fn map_io_error(&self, error: std::io::Error) -> StorageError {
        StorageError::Io(format!("{}: {error}", self.path.display()))
    }
}

impl SnapshotStore for DbliteSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let mut database = self.lock_database()?;
        let Some(bytes) = database
            .get(&doc_id.to_string())
            .map_err(|error| self.map_io_error(error))?
        else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &bytes).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let mut database = self.lock_database()?;
        let doc_id = snapshot.document.id;
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize dblite snapshot `{doc_id}`: {error}"
            ))
        })?;

        database
            .set(&doc_id.to_string(), &bytes)
            .map_err(|error| self.map_io_error(error))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut database = self.lock_database()?;
        let deleted = database
            .delete(&doc_id.to_string())
            .map_err(|error| self.map_io_error(error))?;

        if deleted {
            // dblite appends delete tombstones; compact to prevent a later reused data slot
            // from being shadowed by the older tombstone on the next reopen.
            database
                .compact()
                .map_err(|error| self.map_io_error(error))?;
        }

        Ok(())
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut database = self.lock_database()?;
        let mut documents = Vec::new();
        let keys = database.keys().map_err(|error| self.map_io_error(error))?;

        for key in keys {
            let Ok(doc_id) = Uuid::parse_str(&key) else {
                continue;
            };

            match database.get(&key) {
                Ok(Some(bytes)) => match self.deserialize_snapshot(doc_id, &bytes) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping corrupt dblite snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                },
                Ok(None) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing dblite snapshot while building document catalog"
                ),
                Err(error) => {
                    return Err(self.map_io_error(error));
                }
            }
        }

        Ok(documents)
    }
}
