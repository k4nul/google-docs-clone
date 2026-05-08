use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use db_rs::{Config as DbRsConfig, Db, LookupTable};
use db_rs_derive::Schema;
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

#[derive(Schema)]
struct SnapshotCatalog {
    snapshots: LookupTable<String, PersistedSnapshot>,
}

pub struct DbRsSnapshotStore {
    path: PathBuf,
    database: Mutex<SnapshotCatalog>,
}

impl DbRsSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_DB_RS_PATH cannot be empty when SNAPSHOT_STORE=db_rs".to_owned(),
            ));
        }

        fs::create_dir_all(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        let database = SnapshotCatalog::init(DbRsConfig::in_folder(&path))
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            database: Mutex::new(database),
        })
    }

    fn lock_database(&self) -> Result<MutexGuard<'_, SnapshotCatalog>, StorageError> {
        self.database.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: db_rs database mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        snapshot: &PersistedSnapshot,
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot: DocumentSnapshot = snapshot.clone().into();
        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }
}

impl SnapshotStore for DbRsSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let database = self.lock_database()?;
        let Some(snapshot) = database.snapshots.get().get(&doc_id.to_string()) else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, snapshot).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let mut database = self.lock_database()?;
        let doc_id = snapshot.document.id;
        database
            .snapshots
            .insert(doc_id.to_string(), PersistedSnapshot::from(snapshot))
            .map(|_| ())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut database = self.lock_database()?;
        database
            .snapshots
            .remove(&doc_id.to_string())
            .map(|_| ())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let database = self.lock_database()?;
        let mut documents = Vec::new();

        for (doc_id_key, snapshot) in database.snapshots.get() {
            let Ok(doc_id) = Uuid::parse_str(doc_id_key) else {
                continue;
            };

            match self.deserialize_snapshot(doc_id, snapshot) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt db_rs snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
