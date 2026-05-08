use std::{
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use aeternusdb::{Db, DbError as AeternusdbError};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOT_CATALOG_KEY: &[u8] = b"__catalog__";

pub struct AeternusdbSnapshotStore {
    path: PathBuf,
    database: Mutex<Db>,
}

impl AeternusdbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_AETERNUSDB_PATH cannot be empty when SNAPSHOT_STORE=aeternusdb"
                    .to_owned(),
            ));
        }

        let database = Db::open(&path, Default::default())
            .map_err(|error| Self::map_open_error(&path, error))?;

        Ok(Self {
            path,
            database: Mutex::new(database),
        })
    }

    fn map_open_error(path: &std::path::Path, error: AeternusdbError) -> StorageError {
        StorageError::Io(format!("{}: {error}", path.display()))
    }

    fn lock_database(&self) -> Result<MutexGuard<'_, Db>, StorageError> {
        self.database.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: aeternusdb database mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn map_error(&self, operation: &str, error: AeternusdbError) -> StorageError {
        StorageError::Io(format!(
            "{}: failed to {operation}: {error}",
            self.path.display()
        ))
    }

    fn load_catalog(&self, database: &Db) -> Result<Vec<String>, StorageError> {
        let Some(bytes) = database
            .get(SNAPSHOT_CATALOG_KEY)
            .map_err(|error| self.map_error("read aeternusdb catalog", error))?
        else {
            return Ok(Vec::new());
        };

        serde_json::from_slice::<Vec<String>>(&bytes).map_err(|_| {
            StorageError::Io(format!(
                "{}: snapshot catalog is corrupt",
                self.path.display()
            ))
        })
    }

    fn save_catalog(&self, database: &Db, catalog: &[String]) -> Result<(), StorageError> {
        let bytes = serde_json::to_vec(catalog)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        database
            .put(SNAPSHOT_CATALOG_KEY, &bytes)
            .map_err(|error| self.map_error("write aeternusdb catalog", error))
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

impl SnapshotStore for AeternusdbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let database = self.lock_database()?;
        let Some(bytes) = database
            .get(doc_id.to_string().as_bytes())
            .map_err(|error| self.map_error("read aeternusdb snapshot", error))?
        else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &bytes).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let key = doc_id.to_string();
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize aeternusdb snapshot `{doc_id}`: {error}"
            ))
        })?;
        let database = self.lock_database()?;
        let mut catalog = self.load_catalog(&database)?;

        database
            .put(key.as_bytes(), &bytes)
            .map_err(|error| self.map_error("write aeternusdb snapshot", error))?;
        if !catalog.iter().any(|value| value == &key) {
            catalog.push(key);
            catalog.sort();
        }

        self.save_catalog(&database, &catalog)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let key = doc_id.to_string();
        let database = self.lock_database()?;
        let mut catalog = self.load_catalog(&database)?;

        database
            .delete(key.as_bytes())
            .map_err(|error| self.map_error("delete aeternusdb snapshot", error))?;
        catalog.retain(|value| value != &key);

        self.save_catalog(&database, &catalog)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let database = self.lock_database()?;
        let mut documents = Vec::new();
        let catalog = self.load_catalog(&database)?;

        for doc_id_key in catalog {
            let Ok(doc_id) = Uuid::parse_str(&doc_id_key) else {
                continue;
            };

            match database
                .get(doc_id_key.as_bytes())
                .map_err(|error| self.map_error("read aeternusdb snapshot", error))?
            {
                Some(bytes) => match self.deserialize_snapshot(doc_id, &bytes) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping corrupt aeternusdb snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                },
                None => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing aeternusdb snapshot while building document catalog"
                ),
            }
        }

        Ok(documents)
    }
}
