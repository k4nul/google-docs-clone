use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use simple_db::SimpleDB;
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

pub struct SimpleDbSnapshotStore {
    path: PathBuf,
    database: Mutex<SimpleDB>,
}

impl SimpleDbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_SIMPLE_DB_PATH cannot be empty when SNAPSHOT_STORE=simple_db".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let database = SimpleDB::find_database(path.to_string_lossy().as_ref())
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            database: Mutex::new(database),
        })
    }

    fn lock_database(&self) -> Result<MutexGuard<'_, SimpleDB>, StorageError> {
        self.database.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: simple_db mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        payload: &str,
    ) -> Result<DocumentSnapshot, StorageError> {
        let bytes = BASE64
            .decode(payload)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot = serde_json::from_slice::<PersistedSnapshot>(&bytes)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot: DocumentSnapshot = snapshot.into();

        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }
}

impl SnapshotStore for SimpleDbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let database = self.lock_database()?;
        let Some(payload) = database.get_value_from_db(&doc_id.to_string()) else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, payload).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let mut database = self.lock_database()?;
        let doc_id = snapshot.document.id;
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize simple_db snapshot `{doc_id}`: {error}"
            ))
        })?;

        database
            .insert_into_db(doc_id.to_string(), BASE64.encode(bytes))
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut database = self.lock_database()?;
        let doc_id_key = doc_id.to_string();
        if database.get_value_from_db(&doc_id_key).is_none() {
            return Ok(());
        }

        database
            .delete_from_db(&doc_id_key)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let database = self.lock_database()?;
        let mut documents = Vec::new();

        for (doc_id_key, payload) in &database.data {
            let Ok(doc_id) = Uuid::parse_str(doc_id_key) else {
                continue;
            };

            match self.deserialize_snapshot(doc_id, payload) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt simple_db snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
