use std::{
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
};

use serde_json::Value;
use uuid::Uuid;
use yedb::{Database, Error as YedbError, ErrorKind as YedbErrorKind};

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOT_NAMESPACE: &str = "snapshots";

pub struct YedbSnapshotStore {
    path: PathBuf,
    database: Mutex<Database>,
}

impl YedbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_YEDB_PATH cannot be empty when SNAPSHOT_STORE=yedb".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let mut database = Database::new();
        let path_string = path.to_string_lossy().into_owned();
        database
            .set_db_path(&path_string)
            .map_err(|error| Self::map_database_error(&path, error))?;
        database
            .open()
            .map_err(|error| Self::map_database_error(&path, error))?;

        Ok(Self {
            path,
            database: Mutex::new(database),
        })
    }

    fn key(doc_id: &Uuid) -> String {
        format!("{SNAPSHOT_NAMESPACE}/{doc_id}")
    }

    fn lock_database(&self) -> Result<MutexGuard<'_, Database>, StorageError> {
        self.database.lock().map_err(|error| {
            StorageError::Io(format!(
                "{}: database mutex poisoned: {error}",
                self.path.display()
            ))
        })
    }

    fn map_database_error(path: &Path, error: YedbError) -> StorageError {
        match error.kind() {
            YedbErrorKind::IOError | YedbErrorKind::TimeoutError | YedbErrorKind::Busy => {
                StorageError::Io(format!("{}: {error}", path.display()))
            }
            _ => StorageError::Config(format!("{}: {error}", path.display())),
        }
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        value: Value,
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot: PersistedSnapshot = serde_json::from_value(value)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot: DocumentSnapshot = snapshot.into();
        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }
}

impl SnapshotStore for YedbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let mut database = self.lock_database()?;
        let key = Self::key(doc_id);
        let value = match database.key_get(&key) {
            Ok(value) => value,
            Err(error) if error.kind() == YedbErrorKind::KeyNotFound => return Ok(None),
            Err(error) if error.kind() == YedbErrorKind::DataError => {
                return Err(StorageError::CorruptSnapshot(*doc_id));
            }
            Err(error) => return Err(Self::map_database_error(&self.path, error)),
        };

        self.deserialize_snapshot(*doc_id, value).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let key = Self::key(&doc_id);
        let persisted_snapshot = PersistedSnapshot::from(snapshot);
        let value = serde_json::to_value(persisted_snapshot)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        let mut database = self.lock_database()?;
        database
            .key_set(&key, value)
            .map_err(|error| Self::map_database_error(&self.path, error))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut database = self.lock_database()?;
        let key = Self::key(doc_id);
        match database.key_delete(&key) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == YedbErrorKind::KeyNotFound => Ok(()),
            Err(error) => Err(Self::map_database_error(&self.path, error)),
        }
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut database = self.lock_database()?;
        let keys = database
            .key_list_all(SNAPSHOT_NAMESPACE)
            .map_err(|error| Self::map_database_error(&self.path, error))?;
        let mut documents = Vec::new();

        for key in keys {
            let Some(doc_id_raw) = key.rsplit('/').next() else {
                continue;
            };
            let Ok(doc_id) = Uuid::parse_str(doc_id_raw) else {
                continue;
            };

            match database.key_get(&key) {
                Ok(value) => match self.deserialize_snapshot(doc_id, value) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping corrupt yedb snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                },
                Err(error) if error.kind() == YedbErrorKind::DataError => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt yedb snapshot while building document catalog"
                ),
                Err(error) if error.kind() == YedbErrorKind::KeyNotFound => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing yedb snapshot while building document catalog"
                ),
                Err(error) => return Err(Self::map_database_error(&self.path, error)),
            }
        }

        Ok(documents)
    }
}
