use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use jasondb::{Database, error::JasonError};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

pub struct JasondbSnapshotStore {
    path: PathBuf,
    lock: Mutex<()>,
}

impl JasondbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_JASONDB_PATH cannot be empty when SNAPSHOT_STORE=jasondb".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let mut db = Self::open_db(&path)?;
        db.compact()
            .map_err(|error| Self::map_error(&path, "compact jasondb snapshot store", error))?;

        Ok(Self {
            path,
            lock: Mutex::new(()),
        })
    }

    fn lock_store(&self) -> Result<MutexGuard<'_, ()>, StorageError> {
        self.lock.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: jasondb snapshot store mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn open_db(path: &std::path::Path) -> Result<Database<String>, StorageError> {
        Database::new(path)
            .map_err(|error| Self::map_error(path, "open jasondb snapshot store", error))
    }

    fn map_error(path: &std::path::Path, operation: &str, error: JasonError) -> StorageError {
        StorageError::Io(format!(
            "{}: failed to {operation}: {error}",
            path.display()
        ))
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        payload: String,
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot = serde_json::from_str::<PersistedSnapshot>(&payload).map_err(|_| {
            tracing::warn!(
                doc_id = %expected_doc_id,
                path = %self.path.display(),
                "jasondb snapshot payload was not valid persisted snapshot JSON"
            );
            StorageError::CorruptSnapshot(expected_doc_id)
        })?;
        let snapshot: DocumentSnapshot = snapshot.into();
        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }
}

impl SnapshotStore for JasondbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let _guard = self.lock_store()?;
        let mut db = Self::open_db(&self.path)?;

        match db.get(doc_id.to_string()) {
            Ok(payload) => self.deserialize_snapshot(*doc_id, payload).map(Some),
            Err(JasonError::InvalidKey) => Ok(None),
            Err(error) => Err(Self::map_error(&self.path, "read jasondb snapshot", error)),
        }
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload = serde_json::to_string(&PersistedSnapshot::from(snapshot))
            .map_err(|error| StorageError::Io(format!("serialize jasondb snapshot: {error}")))?;

        let _guard = self.lock_store()?;
        let mut db = Self::open_db(&self.path)?;
        db.set(doc_id.to_string(), &payload)
            .map_err(|error| Self::map_error(&self.path, "write jasondb snapshot", error))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let _guard = self.lock_store()?;
        let mut db = Self::open_db(&self.path)?;

        match db.delete(doc_id.to_string()) {
            Ok(()) | Err(JasonError::InvalidKey) => Ok(()),
            Err(error) => Err(Self::map_error(
                &self.path,
                "delete jasondb snapshot",
                error,
            )),
        }
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let _guard = self.lock_store()?;
        let mut db = Self::open_db(&self.path)?;
        let mut documents = Vec::new();

        for entry in db.iter() {
            let (doc_id_key, payload) = entry
                .map_err(|error| Self::map_error(&self.path, "iterate jasondb snapshots", error))?;
            let Ok(doc_id) = Uuid::parse_str(&doc_id_key) else {
                continue;
            };

            match self.deserialize_snapshot(doc_id, payload) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt jasondb snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        documents.sort_unstable_by_key(|document| document.id);
        Ok(documents)
    }
}
