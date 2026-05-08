use std::{
    fs,
    panic::{AssertUnwindSafe, catch_unwind},
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use ckydb::{Controller, connect};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOT_CATALOG_KEY: &str = "__catalog__";
const DEFAULT_MAX_FILE_SIZE_KB: f64 = 4096.0;
const DEFAULT_VACUUM_INTERVAL_SECS: f64 = 300.0;

pub struct CkydbSnapshotStore {
    path: PathBuf,
    database: Mutex<Box<dyn Controller + Send>>,
}

impl CkydbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_CKYDB_PATH cannot be empty when SNAPSHOT_STORE=ckydb".to_owned(),
            ));
        }

        fs::create_dir_all(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        let database = connect(
            path.to_string_lossy().as_ref(),
            DEFAULT_MAX_FILE_SIZE_KB,
            DEFAULT_VACUUM_INTERVAL_SECS,
        )
        .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            database: Mutex::new(Box::new(database)),
        })
    }

    fn lock_database(&self) -> Result<MutexGuard<'_, Box<dyn Controller + Send>>, StorageError> {
        self.database.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: ckydb database mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        encoded: &str,
    ) -> Result<DocumentSnapshot, StorageError> {
        let bytes = BASE64_STANDARD
            .decode(encoded)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot = serde_json::from_slice::<PersistedSnapshot>(&bytes)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot: DocumentSnapshot = snapshot.into();

        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }

    fn load_catalog(&self, database: &mut dyn Controller) -> Result<Vec<String>, StorageError> {
        let Some(encoded) = self.read_value(database, SNAPSHOT_CATALOG_KEY)? else {
            return Ok(Vec::new());
        };
        let bytes = BASE64_STANDARD.decode(encoded).map_err(|_| {
            StorageError::Io(format!(
                "{}: snapshot catalog is corrupt",
                self.path.display()
            ))
        })?;

        serde_json::from_slice::<Vec<String>>(&bytes).map_err(|_| {
            StorageError::Io(format!(
                "{}: snapshot catalog is corrupt",
                self.path.display()
            ))
        })
    }

    fn save_catalog(
        &self,
        database: &mut dyn Controller,
        catalog: &[String],
    ) -> Result<(), StorageError> {
        let bytes = serde_json::to_vec(catalog)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        let encoded = BASE64_STANDARD.encode(bytes);

        self.write_value(database, SNAPSHOT_CATALOG_KEY, &encoded)
    }

    fn read_value(
        &self,
        database: &mut dyn Controller,
        key: &str,
    ) -> Result<Option<String>, StorageError> {
        catch_unwind(AssertUnwindSafe(|| database.get(key)))
            .map_err(|_| {
                StorageError::Io(format!(
                    "{}: ckydb read for key `{key}` panicked",
                    self.path.display()
                ))
            })?
            .map(Some)
            .or_else(|_| Ok(None))
    }

    fn write_value(
        &self,
        database: &mut dyn Controller,
        key: &str,
        value: &str,
    ) -> Result<(), StorageError> {
        database.set(key, value).map_err(|error| {
            StorageError::Io(format!(
                "{}: failed to write ckydb key `{key}`: {error}",
                self.path.display()
            ))
        })
    }

    fn delete_value(&self, database: &mut dyn Controller, key: &str) -> Result<(), StorageError> {
        match database.delete(key) {
            Ok(()) | Err(_) => Ok(()),
        }
    }
}

impl SnapshotStore for CkydbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let mut database = self.lock_database()?;
        let Some(encoded) = self.read_value(database.as_mut(), &doc_id.to_string())? else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &encoded).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let mut database = self.lock_database()?;
        let doc_id = snapshot.document.id;
        let doc_id_key = doc_id.to_string();
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize ckydb snapshot `{doc_id}`: {error}"
            ))
        })?;
        let encoded = BASE64_STANDARD.encode(bytes);
        let mut catalog = self.load_catalog(database.as_mut())?;

        self.write_value(database.as_mut(), &doc_id_key, &encoded)?;
        if !catalog.iter().any(|value| value == &doc_id_key) {
            catalog.push(doc_id_key);
            catalog.sort();
        }

        self.save_catalog(database.as_mut(), &catalog)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut database = self.lock_database()?;
        let doc_id_key = doc_id.to_string();
        let mut catalog = self.load_catalog(database.as_mut())?;

        self.delete_value(database.as_mut(), &doc_id_key)?;
        catalog.retain(|value| value != &doc_id_key);

        self.save_catalog(database.as_mut(), &catalog)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut database = self.lock_database()?;
        let mut documents = Vec::new();
        let catalog = self.load_catalog(database.as_mut())?;

        for doc_id_key in catalog {
            let Ok(doc_id) = Uuid::parse_str(&doc_id_key) else {
                continue;
            };

            match self.read_value(database.as_mut(), &doc_id_key)? {
                Some(encoded) => match self.deserialize_snapshot(doc_id, &encoded) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping corrupt ckydb snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                },
                None => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing ckydb snapshot while building document catalog"
                ),
            }
        }

        Ok(documents)
    }
}
