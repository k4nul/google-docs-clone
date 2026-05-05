use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use uuid::Uuid;
use yakvdb::{
    api::{Store, tree::Tree},
    disk::{block::Block, file::File as YakvdbFile},
};

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const PAGE_BYTES: u32 = 4096;
const SNAPSHOT_KEY_PREFIX: &str = "snapshot:";

pub struct YakvdbSnapshotStore {
    path: PathBuf,
    database: Mutex<YakvdbFile<Block>>,
}

impl YakvdbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_YAKVDB_PATH cannot be empty when SNAPSHOT_STORE=yakvdb".to_owned(),
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
            YakvdbFile::<Block>::open(&path)
        } else {
            YakvdbFile::<Block>::make(&path, PAGE_BYTES)
        }
        .map_err(|error| {
            StorageError::Io(format!(
                "{}: failed to open yakvdb snapshot store: {error}",
                path.display()
            ))
        })?;

        Ok(Self {
            path,
            database: Mutex::new(database),
        })
    }

    fn lock_database(&self) -> Result<MutexGuard<'_, YakvdbFile<Block>>, StorageError> {
        self.database.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: yakvdb snapshot database mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn map_error(&self, operation: &str, error: yakvdb::api::error::Error) -> StorageError {
        StorageError::Io(format!(
            "{}: failed to {operation}: {error}",
            self.path.display()
        ))
    }

    fn key_for_doc_id(doc_id: &Uuid) -> Vec<u8> {
        format!("{SNAPSHOT_KEY_PREFIX}{doc_id}").into_bytes()
    }

    fn flush_database(&self, database: &YakvdbFile<Block>) -> Result<(), StorageError> {
        database.mark(1);
        database
            .flush()
            .map_err(|error| self.map_error("flush yakvdb snapshot database", error))
    }

    fn doc_id_from_key(key: &[u8]) -> Option<Uuid> {
        let key = std::str::from_utf8(key).ok()?;
        let doc_id = key.strip_prefix(SNAPSHOT_KEY_PREFIX)?;
        Uuid::parse_str(doc_id).ok()
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        bytes: &[u8],
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot = serde_json::from_slice::<PersistedSnapshot>(bytes)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot: DocumentSnapshot = snapshot.into();

        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }
}

impl SnapshotStore for YakvdbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let database = self.lock_database()?;
        let Some(bytes) = database
            .lookup(&Self::key_for_doc_id(doc_id))
            .map_err(|error| self.map_error("read yakvdb snapshot", error))?
        else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &bytes).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize yakvdb snapshot `{doc_id}`: {error}"
            ))
        })?;
        let database = self.lock_database()?;
        database
            .insert(&Self::key_for_doc_id(&doc_id), &bytes)
            .map_err(|error| self.map_error("write yakvdb snapshot", error))?;
        self.flush_database(&database)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let database = self.lock_database()?;
        database
            .remove(&Self::key_for_doc_id(doc_id))
            .map_err(|error| self.map_error("delete yakvdb snapshot", error))?;
        self.flush_database(&database)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let database = self.lock_database()?;
        let mut documents = Vec::new();
        let mut key = database
            .min()
            .map_err(|error| self.map_error("find first yakvdb snapshot key", error))?;

        while let Some(current_key) = key {
            if let Some(doc_id) = Self::doc_id_from_key(&current_key) {
                let Some(value) = database
                    .lookup(&current_key)
                    .map_err(|error| self.map_error("read yakvdb snapshot catalog value", error))?
                else {
                    key = database.above(&current_key).map_err(|error| {
                        self.map_error("advance yakvdb snapshot catalog", error)
                    })?;
                    continue;
                };

                match self.deserialize_snapshot(doc_id, &value) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping corrupt yakvdb snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                }
            }

            key = database
                .above(&current_key)
                .map_err(|error| self.map_error("advance yakvdb snapshot catalog", error))?;
        }

        Ok(documents)
    }
}
