use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use dbless::{Database, TableReadInterface, TableWriteInterface};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

pub struct DblessSnapshotStore {
    path: PathBuf,
    database: Mutex<Database>,
}

impl DblessSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_DBLESS_PATH cannot be empty when SNAPSHOT_STORE=dbless".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let database = Database::open(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            database: Mutex::new(database),
        })
    }

    fn lock_database(&self) -> Result<MutexGuard<'_, Database>, StorageError> {
        self.database.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: dbless database mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn load_persisted_snapshot(
        &self,
        database: &Database,
        expected_doc_id: Uuid,
    ) -> Result<Option<DocumentSnapshot>, StorageError> {
        let Some(snapshot) = database
            .get::<PersistedSnapshot>(&expected_doc_id.to_string())
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?
        else {
            return Ok(None);
        };
        let snapshot: DocumentSnapshot = snapshot.into();

        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(Some(snapshot))
    }
}

impl SnapshotStore for DblessSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let database = self.lock_database()?;
        self.load_persisted_snapshot(&database, *doc_id)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let mut database = self.lock_database()?;
        let doc_id = snapshot.document.id;
        let persisted = PersistedSnapshot::from(snapshot);

        database
            .set(&doc_id.to_string(), &persisted)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut database = self.lock_database()?;
        let doc_id_key = doc_id.to_string();
        let exists = database
            .contains_key(&doc_id_key)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        if !exists {
            return Ok(());
        }

        database
            .delete(&doc_id_key)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let database = self.lock_database()?;
        let keys = database
            .keys()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        let mut documents = Vec::new();

        for key in keys {
            let Ok(doc_id) = Uuid::parse_str(&key) else {
                continue;
            };

            match self.load_persisted_snapshot(&database, doc_id) {
                Ok(Some(snapshot)) => documents.push(snapshot.document),
                Ok(None) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing dbless snapshot while building document catalog"
                ),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt dbless snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
