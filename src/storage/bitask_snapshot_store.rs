use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use bitask::db::{Bitask as BitaskDb, Error as BitaskError};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOT_CATALOG_KEY: &[u8] = b"__catalog__";

pub struct BitaskSnapshotStore {
    path: PathBuf,
    database: Mutex<BitaskDb>,
}

impl BitaskSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_BITASK_PATH cannot be empty when SNAPSHOT_STORE=bitask".to_owned(),
            ));
        }

        fs::create_dir_all(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        let database = BitaskDb::open(&path)
            .map_err(|error| Self::map_bitask_error(&path, "open bitask snapshot store", error))?;

        Ok(Self {
            path,
            database: Mutex::new(database),
        })
    }

    fn lock_database(&self) -> Result<MutexGuard<'_, BitaskDb>, StorageError> {
        self.database.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: bitask database mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn map_bitask_error(
        path: &std::path::Path,
        operation: &str,
        error: BitaskError,
    ) -> StorageError {
        StorageError::Io(format!(
            "{}: failed to {operation}: {error}",
            path.display()
        ))
    }

    fn load_catalog(&self, database: &mut BitaskDb) -> Result<Vec<String>, StorageError> {
        match database.ask(SNAPSHOT_CATALOG_KEY) {
            Ok(bytes) => serde_json::from_slice::<Vec<String>>(&bytes).map_err(|_| {
                StorageError::Io(format!(
                    "{}: snapshot catalog is corrupt",
                    self.path.display()
                ))
            }),
            Err(BitaskError::KeyNotFound) => Ok(Vec::new()),
            Err(error) => Err(Self::map_bitask_error(
                &self.path,
                "read bitask snapshot catalog",
                error,
            )),
        }
    }

    fn save_catalog(
        &self,
        database: &mut BitaskDb,
        catalog: &[String],
    ) -> Result<(), StorageError> {
        let bytes = serde_json::to_vec(catalog)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        database
            .put(SNAPSHOT_CATALOG_KEY.to_vec(), bytes)
            .map_err(|error| {
                Self::map_bitask_error(&self.path, "write bitask snapshot catalog", error)
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
}

impl SnapshotStore for BitaskSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let mut database = self.lock_database()?;
        match database.ask(doc_id.to_string().as_bytes()) {
            Ok(bytes) => self.deserialize_snapshot(*doc_id, &bytes).map(Some),
            Err(BitaskError::KeyNotFound) => Ok(None),
            Err(error) => Err(Self::map_bitask_error(
                &self.path,
                "read bitask snapshot",
                error,
            )),
        }
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let key = doc_id.to_string();
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize bitask snapshot `{doc_id}`: {error}"
            ))
        })?;
        let mut database = self.lock_database()?;
        let mut catalog = self.load_catalog(&mut database)?;

        database
            .put(key.as_bytes().to_vec(), bytes)
            .map_err(|error| Self::map_bitask_error(&self.path, "write bitask snapshot", error))?;
        if !catalog.iter().any(|value| value == &key) {
            catalog.push(key);
            catalog.sort();
        }

        self.save_catalog(&mut database, &catalog)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let key = doc_id.to_string();
        let mut database = self.lock_database()?;
        let mut catalog = self.load_catalog(&mut database)?;

        database
            .remove(key.as_bytes().to_vec())
            .map_err(|error| Self::map_bitask_error(&self.path, "delete bitask snapshot", error))?;
        catalog.retain(|value| value != &key);

        self.save_catalog(&mut database, &catalog)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut database = self.lock_database()?;
        let catalog = self.load_catalog(&mut database)?;
        let mut documents = Vec::new();

        for doc_id_key in catalog {
            let Ok(doc_id) = Uuid::parse_str(&doc_id_key) else {
                continue;
            };

            match database.ask(doc_id_key.as_bytes()) {
                Ok(bytes) => match self.deserialize_snapshot(doc_id, &bytes) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping corrupt bitask snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                },
                Err(BitaskError::KeyNotFound) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing bitask snapshot while building document catalog"
                ),
                Err(error) => {
                    return Err(Self::map_bitask_error(
                        &self.path,
                        "read bitask snapshot",
                        error,
                    ));
                }
            }
        }

        Ok(documents)
    }
}
