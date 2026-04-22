use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use colon_db::ColonDB;
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

pub struct ColonDbSnapshotStore {
    path: PathBuf,
    database: Mutex<ColonDB>,
}

impl ColonDbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_COLON_DB_PATH cannot be empty when SNAPSHOT_STORE=colon_db".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        if !path.exists() {
            fs::File::create(&path)
                .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;
        }

        let database = ColonDB::find_database(path.to_string_lossy().as_ref());

        Ok(Self {
            path,
            database: Mutex::new(database),
        })
    }

    fn lock_database(&self) -> Result<MutexGuard<'_, ColonDB>, StorageError> {
        self.database.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: colon_db mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn sync_file(&self) -> Result<(), StorageError> {
        fs::OpenOptions::new()
            .read(true)
            .open(&self.path)
            .and_then(|file| file.sync_all())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn persist_database(&self, database: &ColonDB) -> Result<(), StorageError> {
        database
            .save_data_to_file()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        self.sync_file()
    }

    fn decode_snapshot(
        &self,
        expected_doc_id: Uuid,
        row: &[String],
    ) -> Result<DocumentSnapshot, StorageError> {
        let Some(payload) = row.first() else {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        };

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

impl SnapshotStore for ColonDbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let database = self.lock_database()?;
        let Some(row) = database.data.get(&doc_id.to_string()) else {
            return Ok(None);
        };

        self.decode_snapshot(*doc_id, row).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let mut database = self.lock_database()?;
        let doc_id = snapshot.document.id;
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize colon_db snapshot `{doc_id}`: {error}"
            ))
        })?;

        database
            .data
            .insert(doc_id.to_string(), vec![BASE64.encode(bytes)]);
        self.persist_database(&database)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut database = self.lock_database()?;
        database.data.remove(&doc_id.to_string());
        self.persist_database(&database)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let database = self.lock_database()?;
        let mut documents = Vec::new();

        for (doc_id_key, row) in &database.data {
            let Ok(doc_id) = Uuid::parse_str(doc_id_key) else {
                continue;
            };

            match self.decode_snapshot(doc_id, row) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt colon_db snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
