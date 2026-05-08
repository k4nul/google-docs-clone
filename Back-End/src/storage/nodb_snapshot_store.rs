use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use nodb::{DumpPolicy, NoDb, SerializationMethod};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

pub struct NodbSnapshotStore {
    path: PathBuf,
    database: Mutex<NoDb>,
}

impl NodbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_NODB_PATH cannot be empty when SNAPSHOT_STORE=nodb".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let database = if path.exists() {
            NoDb::load(&path, DumpPolicy::Auto, SerializationMethod::Json)
                .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?
        } else {
            NoDb::new(&path, DumpPolicy::Auto, SerializationMethod::Json)
        };

        Ok(Self {
            path,
            database: Mutex::new(database),
        })
    }

    fn lock_database(&self) -> Result<MutexGuard<'_, NoDb>, StorageError> {
        self.database.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: nodb database mutex was poisoned",
                self.path.display()
            ))
        })
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

impl SnapshotStore for NodbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let database = self.lock_database()?;
        let doc_id_key = doc_id.to_string();
        let Some(snapshot) = database.get::<_, PersistedSnapshot>(&doc_id_key) else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, snapshot).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let mut database = self.lock_database()?;
        let doc_id_key = snapshot.document.id.to_string();
        let persisted_snapshot = PersistedSnapshot::from(snapshot);

        database
            .set(&doc_id_key, persisted_snapshot)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut database = self.lock_database()?;
        let doc_id_key = doc_id.to_string();

        database
            .rem(&doc_id_key)
            .map(|_| ())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let database = self.lock_database()?;
        let mut documents = Vec::new();

        for doc_id_key in database.get_all() {
            let Ok(doc_id) = Uuid::parse_str(&doc_id_key) else {
                continue;
            };

            match database.get::<_, PersistedSnapshot>(&doc_id_key) {
                Some(snapshot) => match self.deserialize_snapshot(doc_id, snapshot) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping corrupt nodb snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                },
                None => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt nodb snapshot while building document catalog"
                ),
            }
        }

        Ok(documents)
    }
}
