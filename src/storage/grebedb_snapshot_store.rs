use std::{
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use grebedb::{
    CompressionLevel as GrebeCompressionLevel, Database as GrebeDb, Error as GrebeError,
    Options as GrebeOptions,
};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

const SNAPSHOT_CATALOG_KEY: &[u8] = b"__catalog__";

pub struct GrebedbSnapshotStore {
    path: PathBuf,
    database: Mutex<GrebeDb>,
}

impl GrebedbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_GREBEDB_PATH cannot be empty when SNAPSHOT_STORE=grebedb".to_owned(),
            ));
        }

        ensure_snapshot_dir(&path)?;

        let mut options = GrebeOptions::default();
        // Group snapshot payload and catalog updates into one explicit flush boundary.
        options.automatic_flush = false;
        options.compression_level = GrebeCompressionLevel::None;

        let database = GrebeDb::open_path(&path, options)
            .map_err(|error| Self::map_error(&path, "open grebedb snapshot store", error))?;

        Ok(Self {
            path,
            database: Mutex::new(database),
        })
    }

    fn lock_database(&self) -> Result<MutexGuard<'_, GrebeDb>, StorageError> {
        self.database.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: grebedb snapshot store mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn map_error(path: &std::path::Path, operation: &str, error: GrebeError) -> StorageError {
        StorageError::Io(format!(
            "{}: failed to {operation}: {error}",
            path.display()
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

    fn load_catalog(&self, database: &mut GrebeDb) -> Result<Vec<String>, StorageError> {
        let Some(bytes) = database
            .get(SNAPSHOT_CATALOG_KEY)
            .map_err(|error| Self::map_error(&self.path, "read grebedb snapshot catalog", error))?
        else {
            return Ok(Vec::new());
        };

        serde_json::from_slice::<Vec<String>>(&bytes).map_err(|_| {
            StorageError::Io(format!(
                "{}: grebedb snapshot catalog is corrupt",
                self.path.display()
            ))
        })
    }

    fn save_catalog(&self, database: &mut GrebeDb, catalog: &[String]) -> Result<(), StorageError> {
        let bytes = serde_json::to_vec(catalog)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        database
            .put(SNAPSHOT_CATALOG_KEY, bytes)
            .map_err(|error| Self::map_error(&self.path, "write grebedb snapshot catalog", error))
    }

    fn flush(&self, database: &mut GrebeDb, operation: &str) -> Result<(), StorageError> {
        database
            .flush()
            .map_err(|error| Self::map_error(&self.path, operation, error))
    }
}

impl SnapshotStore for GrebedbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let mut database = self.lock_database()?;
        let doc_id_key = doc_id.to_string();
        let Some(bytes) = database
            .get(doc_id_key.as_bytes())
            .map_err(|error| Self::map_error(&self.path, "read grebedb snapshot", error))?
        else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &bytes).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let doc_id_key = doc_id.to_string();
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize grebedb snapshot `{doc_id}`: {error}"
            ))
        })?;
        let mut database = self.lock_database()?;
        let mut catalog = self.load_catalog(&mut database)?;

        database
            .put(doc_id_key.as_bytes(), bytes)
            .map_err(|error| Self::map_error(&self.path, "write grebedb snapshot", error))?;
        if !catalog.iter().any(|entry| entry == &doc_id_key) {
            catalog.push(doc_id_key);
            catalog.sort();
        }

        self.save_catalog(&mut database, &catalog)?;
        self.flush(&mut database, "flush grebedb snapshot store")
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let doc_id_key = doc_id.to_string();
        let mut database = self.lock_database()?;
        let mut catalog = self.load_catalog(&mut database)?;

        database
            .remove(doc_id_key.as_bytes())
            .map_err(|error| Self::map_error(&self.path, "delete grebedb snapshot", error))?;
        catalog.retain(|entry| entry != &doc_id_key);

        self.save_catalog(&mut database, &catalog)?;
        self.flush(&mut database, "flush grebedb snapshot store")
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut database = self.lock_database()?;
        let catalog = self.load_catalog(&mut database)?;
        let mut documents = Vec::new();

        for doc_id_key in catalog {
            let Ok(doc_id) = Uuid::parse_str(&doc_id_key) else {
                continue;
            };

            match database.get(doc_id_key.as_bytes()) {
                Ok(Some(bytes)) => match self.deserialize_snapshot(doc_id, &bytes) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping corrupt grebedb snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                },
                Ok(None) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing grebedb snapshot while building document catalog"
                ),
                Err(error) => {
                    return Err(Self::map_error(&self.path, "read grebedb snapshot", error));
                }
            }
        }

        Ok(documents)
    }
}
