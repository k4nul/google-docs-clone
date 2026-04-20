use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use rustlite::{Database, Error as RustliteError};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOT_CATALOG_KEY: &[u8] = b"__catalog__";
const SNAPSHOT_KEY_PREFIX: &str = "snapshot:";

pub struct RustliteSnapshotStore {
    path: PathBuf,
    database: Mutex<Database>,
}

impl RustliteSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_RUSTLITE_PATH cannot be empty when SNAPSHOT_STORE=rustlite".to_owned(),
            ));
        }

        fs::create_dir_all(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        let database =
            Database::open(&path).map_err(|error| Self::map_database_error(&path, error))?;

        Ok(Self {
            path,
            database: Mutex::new(database),
        })
    }

    fn lock_database(&self) -> Result<MutexGuard<'_, Database>, StorageError> {
        self.database.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: rustlite database mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn map_database_error(path: &PathBuf, error: RustliteError) -> StorageError {
        StorageError::Io(format!("{}: {error}", path.display()))
    }

    fn snapshot_key(doc_id: &Uuid) -> String {
        format!("{SNAPSHOT_KEY_PREFIX}{doc_id}")
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

    fn load_catalog(&self, database: &Database) -> Result<Vec<String>, StorageError> {
        let Some(bytes) = database
            .get(SNAPSHOT_CATALOG_KEY)
            .map_err(|error| Self::map_database_error(&self.path, error))?
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

    fn save_catalog(&self, database: &Database, catalog: &[String]) -> Result<(), StorageError> {
        let bytes = serde_json::to_vec(catalog)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        database
            .put(SNAPSHOT_CATALOG_KEY, &bytes)
            .map_err(|error| Self::map_database_error(&self.path, error))
    }
}

impl SnapshotStore for RustliteSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let database = self.lock_database()?;
        let key = Self::snapshot_key(doc_id);
        let Some(bytes) = database
            .get(key.as_bytes())
            .map_err(|error| Self::map_database_error(&self.path, error))?
        else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &bytes).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let database = self.lock_database()?;
        let doc_id = snapshot.document.id;
        let doc_id_key = doc_id.to_string();
        let snapshot_key = Self::snapshot_key(&doc_id);
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize rustlite snapshot `{doc_id}`: {error}"
            ))
        })?;
        let mut catalog = self.load_catalog(&database)?;

        database
            .put(snapshot_key.as_bytes(), &bytes)
            .map_err(|error| Self::map_database_error(&self.path, error))?;
        if !catalog.iter().any(|value| value == &doc_id_key) {
            catalog.push(doc_id_key);
            catalog.sort();
        }

        self.save_catalog(&database, &catalog)?;
        database
            .sync()
            .map_err(|error| Self::map_database_error(&self.path, error))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let database = self.lock_database()?;
        let doc_id_key = doc_id.to_string();
        let snapshot_key = Self::snapshot_key(doc_id);
        let mut catalog = self.load_catalog(&database)?;

        database
            .delete(snapshot_key.as_bytes())
            .map_err(|error| Self::map_database_error(&self.path, error))?;
        catalog.retain(|value| value != &doc_id_key);

        self.save_catalog(&database, &catalog)?;
        database
            .sync()
            .map_err(|error| Self::map_database_error(&self.path, error))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let database = self.lock_database()?;
        let mut documents = Vec::new();
        let catalog = self.load_catalog(&database)?;

        for doc_id_key in catalog {
            let Ok(doc_id) = Uuid::parse_str(&doc_id_key) else {
                continue;
            };
            let snapshot_key = Self::snapshot_key(&doc_id);

            match database.get(snapshot_key.as_bytes()) {
                Ok(Some(bytes)) => match self.deserialize_snapshot(doc_id, &bytes) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping corrupt rustlite snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                },
                Ok(None) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing rustlite snapshot while building document catalog"
                ),
                Err(error) => return Err(Self::map_database_error(&self.path, error)),
            }
        }

        Ok(documents)
    }
}
