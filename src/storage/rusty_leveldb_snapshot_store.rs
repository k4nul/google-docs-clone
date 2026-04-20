use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use rusty_leveldb::{DB, LdbIterator, Options};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

pub struct RustyLeveldbSnapshotStore {
    path: PathBuf,
    database: Mutex<DB>,
}

impl RustyLeveldbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_RUSTY_LEVELDB_PATH cannot be empty when SNAPSHOT_STORE=rusty_leveldb"
                    .to_owned(),
            ));
        }

        fs::create_dir_all(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        let database = DB::open(&path, Options::default())
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            database: Mutex::new(database),
        })
    }

    fn lock_database(&self) -> Result<MutexGuard<'_, DB>, StorageError> {
        self.database.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: rusty-leveldb database mutex was poisoned",
                self.path.display()
            ))
        })
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

    fn flush(&self, database: &mut DB) -> Result<(), StorageError> {
        database
            .flush()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }
}

impl SnapshotStore for RustyLeveldbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let mut database = self.lock_database()?;
        let Some(bytes) = database.get(doc_id.to_string().as_bytes()) else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, bytes.as_ref()).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let mut database = self.lock_database()?;
        let doc_id = snapshot.document.id;
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize rusty-leveldb snapshot `{doc_id}`: {error}"
            ))
        })?;

        database
            .put(doc_id.to_string().as_bytes(), &bytes)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        self.flush(&mut database)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut database = self.lock_database()?;
        database
            .delete(doc_id.to_string().as_bytes())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        self.flush(&mut database)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut database = self.lock_database()?;
        let mut iterator = database
            .new_iter()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        let mut documents = Vec::new();

        while let Some((key, value)) = iterator.next() {
            let Ok(doc_id_key) = std::str::from_utf8(&key) else {
                continue;
            };
            let Ok(doc_id) = Uuid::parse_str(doc_id_key) else {
                continue;
            };

            match self.deserialize_snapshot(doc_id, &value) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt rusty-leveldb snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
