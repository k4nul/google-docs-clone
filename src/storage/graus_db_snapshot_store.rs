use std::{
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use graus_db::{GrausDb, GrausError};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOT_CATALOG_KEY: &[u8] = b"__catalog__";

pub struct GrausDbSnapshotStore {
    path: PathBuf,
    db: Mutex<Option<GrausDb>>,
}

impl GrausDbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_GRAUS_DB_PATH cannot be empty when SNAPSHOT_STORE=graus_db".to_owned(),
            ));
        }

        let db = GrausDb::open(&path).map_err(|error| Self::map_open_error(&path, error))?;

        Ok(Self {
            path,
            db: Mutex::new(Some(db)),
        })
    }

    fn lock_db(&self) -> Result<MutexGuard<'_, Option<GrausDb>>, StorageError> {
        self.db.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: graus_db mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn map_open_error(path: &std::path::Path, error: GrausError) -> StorageError {
        StorageError::Io(format!("{}: {error}", path.display()))
    }

    fn map_error(&self, error: GrausError) -> StorageError {
        StorageError::Io(format!("{}: {error}", self.path.display()))
    }

    fn with_db<T>(
        &self,
        action: impl FnOnce(&GrausDb) -> Result<T, StorageError>,
    ) -> Result<T, StorageError> {
        let db = self.lock_db()?;
        let db = db.as_ref().ok_or_else(|| {
            StorageError::Io(format!(
                "{}: graus_db handle is unavailable",
                self.path.display()
            ))
        })?;

        action(db)
    }

    fn write_then_reopen(
        &self,
        action: impl FnOnce(&GrausDb) -> Result<(), StorageError>,
    ) -> Result<(), StorageError> {
        let mut db = self.lock_db()?;
        let active = db.take().ok_or_else(|| {
            StorageError::Io(format!(
                "{}: graus_db handle is unavailable",
                self.path.display()
            ))
        })?;

        action(&active)?;
        drop(active);

        let reopened =
            GrausDb::open(&self.path).map_err(|error| Self::map_open_error(&self.path, error))?;
        *db = Some(reopened);

        Ok(())
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        payload: &[u8],
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot = serde_json::from_slice::<PersistedSnapshot>(payload)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot: DocumentSnapshot = snapshot.into();

        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }

    fn load_catalog_from(&self, db: &GrausDb) -> Result<Vec<String>, StorageError> {
        let Some(payload) = db
            .get(SNAPSHOT_CATALOG_KEY)
            .map_err(|error| self.map_error(error))?
        else {
            return Ok(Vec::new());
        };

        serde_json::from_slice::<Vec<String>>(&payload).map_err(|_| {
            StorageError::Io(format!(
                "{}: snapshot catalog is corrupt",
                self.path.display()
            ))
        })
    }

    fn save_catalog_to(&self, db: &GrausDb, catalog: &[String]) -> Result<(), StorageError> {
        let payload = serde_json::to_vec(catalog).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize graus_db snapshot catalog: {error}"
            ))
        })?;

        db.set(SNAPSHOT_CATALOG_KEY.to_vec(), &payload)
            .map_err(|error| self.map_error(error))
    }
}

impl SnapshotStore for GrausDbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let doc_id_key = doc_id.to_string();

        self.with_db(|db| {
            let Some(payload) = db
                .get(doc_id_key.as_bytes())
                .map_err(|error| self.map_error(error))?
            else {
                return Ok(None);
            };

            self.deserialize_snapshot(*doc_id, &payload).map(Some)
        })
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let doc_id_key = doc_id.to_string();
        let payload = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize graus_db snapshot `{doc_id}`: {error}"
            ))
        })?;

        self.write_then_reopen(|db| {
            let mut catalog = self.load_catalog_from(db)?;

            db.set(doc_id_key.as_bytes().to_vec(), &payload)
                .map_err(|error| self.map_error(error))?;

            if !catalog.iter().any(|value| value == &doc_id_key) {
                catalog.push(doc_id_key);
                catalog.sort();
            }

            self.save_catalog_to(db, &catalog)
        })
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let doc_id_key = doc_id.to_string();

        self.write_then_reopen(|db| {
            match db.remove(doc_id_key.as_bytes()) {
                Ok(()) | Err(GrausError::KeyNotFound) => {}
                Err(error) => return Err(self.map_error(error)),
            }

            let mut catalog = self.load_catalog_from(db)?;
            catalog.retain(|value| value != &doc_id_key);

            self.save_catalog_to(db, &catalog)
        })
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        self.with_db(|db| {
            let catalog = self.load_catalog_from(db)?;
            let mut documents = Vec::new();

            for doc_id_key in catalog {
                let Ok(doc_id) = Uuid::parse_str(&doc_id_key) else {
                    continue;
                };

                match db.get(doc_id_key.as_bytes()) {
                    Ok(Some(payload)) => match self.deserialize_snapshot(doc_id, &payload) {
                        Ok(snapshot) => documents.push(snapshot.document),
                        Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                            doc_id = %doc_id,
                            path = %self.path.display(),
                            "skipping corrupt graus_db snapshot while building document catalog"
                        ),
                        Err(error) => return Err(error),
                    },
                    Ok(None) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping missing graus_db snapshot while building document catalog"
                    ),
                    Err(error) => return Err(self.map_error(error)),
                }
            }

            Ok(documents)
        })
    }
}
