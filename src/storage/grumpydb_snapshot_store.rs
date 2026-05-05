use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use grumpydb::{GrumpyDb, GrumpyError, Value};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

pub struct GrumpydbSnapshotStore {
    path: PathBuf,
    db: Mutex<GrumpyDb>,
}

impl GrumpydbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_GRUMPYDB_PATH cannot be empty when SNAPSHOT_STORE=grumpydb".to_owned(),
            ));
        }

        fs::create_dir_all(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        let db = GrumpyDb::open(&path).map_err(|error| Self::map_open_error(&path, error))?;

        Ok(Self {
            path,
            db: Mutex::new(db),
        })
    }

    fn lock_db(&self) -> Result<MutexGuard<'_, GrumpyDb>, StorageError> {
        self.db.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: grumpydb mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn map_open_error(path: &std::path::Path, error: GrumpyError) -> StorageError {
        StorageError::Io(format!("{}: {error}", path.display()))
    }

    fn map_error(&self, error: GrumpyError) -> StorageError {
        StorageError::Io(format!("{}: {error}", self.path.display()))
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        value: Value,
    ) -> Result<DocumentSnapshot, StorageError> {
        let Value::Bytes(payload) = value else {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        };

        let snapshot = serde_json::from_slice::<PersistedSnapshot>(&payload)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot: DocumentSnapshot = snapshot.into();

        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }
}

impl SnapshotStore for GrumpydbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let mut db = self.lock_db()?;
        let Some(value) = db.get(doc_id).map_err(|error| self.map_error(error))? else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, value).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize grumpydb snapshot `{doc_id}`: {error}"
            ))
        })?;
        let mut db = self.lock_db()?;
        let value = Value::Bytes(payload);

        if db
            .get(&doc_id)
            .map_err(|error| self.map_error(error))?
            .is_some()
        {
            db.update(&doc_id, value)
                .map_err(|error| self.map_error(error))?;
        } else {
            db.insert(doc_id, value)
                .map_err(|error| self.map_error(error))?;
        }

        db.flush().map_err(|error| self.map_error(error))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut db = self.lock_db()?;

        match db.delete(doc_id) {
            Ok(()) | Err(GrumpyError::KeyNotFound(_)) => {}
            Err(error) => return Err(self.map_error(error)),
        }

        db.flush().map_err(|error| self.map_error(error))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut db = self.lock_db()?;
        let snapshots = db.scan(..).map_err(|error| self.map_error(error))?;
        let mut documents = Vec::new();

        for (doc_id, value) in snapshots {
            match self.deserialize_snapshot(doc_id, value) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt grumpydb snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
