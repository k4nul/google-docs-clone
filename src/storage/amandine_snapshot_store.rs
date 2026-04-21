use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use amandine::{Data, Database, db::TDatabase};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOTS_COLLECTION: &str = "snapshots";

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AmandineSnapshotRecord {
    id: String,
    snapshot: PersistedSnapshot,
}

impl Data for AmandineSnapshotRecord {
    fn uuid(&self) -> String {
        self.id.clone()
    }
}

pub struct AmandineSnapshotStore {
    path: PathBuf,
    db: Mutex<Database>,
}

impl AmandineSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_AMANDINE_PATH cannot be empty when SNAPSHOT_STORE=amandine".to_owned(),
            ));
        }

        fs::create_dir_all(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        let mut db = Database::new();
        db.connect(path.clone())
            .map_err(|error| Self::map_error(&path, "connect amandine snapshot store", error))?;

        let collections = db
            .list_collections()
            .map_err(|error| Self::map_error(&path, "list amandine snapshot collections", error))?;
        if !collections
            .iter()
            .any(|collection| collection == SNAPSHOTS_COLLECTION)
        {
            db.create_collection(SNAPSHOTS_COLLECTION)
                .map_err(|error| {
                    Self::map_error(&path, "create amandine snapshots collection", error)
                })?;
        }

        Ok(Self {
            path,
            db: Mutex::new(db),
        })
    }

    fn lock_db(&self) -> Result<MutexGuard<'_, Database>, StorageError> {
        self.db.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: amandine snapshot store mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn map_error(
        path: &std::path::Path,
        operation: &str,
        error: impl std::fmt::Display,
    ) -> StorageError {
        StorageError::Io(format!(
            "{}: failed to {operation}: {error}",
            path.display()
        ))
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        snapshot: PersistedSnapshot,
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot: DocumentSnapshot = snapshot.into();
        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }
}

impl SnapshotStore for AmandineSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let mut db = self.lock_db()?;
        match db.query_data::<AmandineSnapshotRecord>(SNAPSHOTS_COLLECTION, &doc_id.to_string()) {
            Ok(record) => self
                .deserialize_snapshot(*doc_id, record.snapshot)
                .map(Some),
            Err(error) if error.to_string().contains("Data not found") => Ok(None),
            Err(error) => Err(Self::map_error(&self.path, "read amandine snapshot", error)),
        }
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let record = AmandineSnapshotRecord {
            id: doc_id.to_string(),
            snapshot: PersistedSnapshot::from(snapshot),
        };

        let mut db = self.lock_db()?;
        match db.update_data(SNAPSHOTS_COLLECTION, record.clone()) {
            Ok(()) => Ok(()),
            Err(error) if error.to_string().contains("Data not found") => db
                .insert_data(SNAPSHOTS_COLLECTION, record)
                .map_err(|error| Self::map_error(&self.path, "insert amandine snapshot", error)),
            Err(error) => Err(Self::map_error(
                &self.path,
                "update amandine snapshot",
                error,
            )),
        }
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut db = self.lock_db()?;
        match db.delete_data::<AmandineSnapshotRecord>(SNAPSHOTS_COLLECTION, &doc_id.to_string()) {
            Ok(()) => Ok(()),
            Err(error) if error.to_string().contains("Data not found") => Ok(()),
            Err(error) => Err(Self::map_error(
                &self.path,
                "delete amandine snapshot",
                error,
            )),
        }
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let db = self.lock_db()?;
        let records = db
            .list_data::<AmandineSnapshotRecord>(SNAPSHOTS_COLLECTION)
            .map_err(|error| Self::map_error(&self.path, "list amandine snapshots", error))?;
        let mut documents = Vec::new();

        for record in records {
            let Ok(doc_id) = Uuid::parse_str(&record.id) else {
                continue;
            };

            match self.deserialize_snapshot(doc_id, record.snapshot) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt amandine snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        documents.sort_unstable_by_key(|document| document.id);
        Ok(documents)
    }
}
