use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
};

use crystal::KvStore;
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const FILE_EXTENSION: &str = "bin";

pub struct CrystalSnapshotStore {
    path: PathBuf,
    database: Mutex<KvStore>,
}

impl CrystalSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_CRYSTAL_PATH cannot be empty when SNAPSHOT_STORE=crystal".to_owned(),
            ));
        }

        let database = KvStore::new(path_to_str(&path)?)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            database: Mutex::new(database),
        })
    }

    fn lock_database(&self) -> Result<MutexGuard<'_, KvStore>, StorageError> {
        self.database.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: crystal snapshot store mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn doc_key(doc_id: Uuid) -> String {
        doc_id.to_string()
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        payload: String,
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot = serde_json::from_str::<PersistedSnapshot>(&payload)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot: DocumentSnapshot = snapshot.into();

        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }
}

impl SnapshotStore for CrystalSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let database = self.lock_database()?;
        let payload = database
            .get(&Self::doc_key(*doc_id))
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        payload
            .map(|payload| self.deserialize_snapshot(*doc_id, payload))
            .transpose()
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload = serde_json::to_string(&PersistedSnapshot::from(snapshot))
            .map_err(|error| StorageError::Io(format!("serialize crystal snapshot: {error}")))?;

        let mut database = self.lock_database()?;
        database
            .set(Self::doc_key(doc_id), payload)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut database = self.lock_database()?;
        database
            .remove(&Self::doc_key(*doc_id))
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let database = self.lock_database()?;
        let mut documents = Vec::new();

        for entry in fs::read_dir(&self.path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?
        {
            let entry = entry
                .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some(FILE_EXTENSION) {
                continue;
            }

            let Some(doc_id_key) = path.file_stem().and_then(|value| value.to_str()) else {
                continue;
            };
            let Ok(doc_id) = Uuid::parse_str(doc_id_key) else {
                continue;
            };

            let Some(payload) = database.get(doc_id_key).map_err(|error| {
                StorageError::Io(format!(
                    "{}: failed to read crystal snapshot catalog entry `{doc_id_key}`: {error}",
                    self.path.display()
                ))
            })?
            else {
                continue;
            };

            match self.deserialize_snapshot(doc_id, payload) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt crystal snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        documents.sort_unstable_by_key(|document| document.id);
        Ok(documents)
    }
}

fn path_to_str(path: &Path) -> Result<&str, StorageError> {
    path.to_str().ok_or_else(|| {
        StorageError::Config(format!(
            "SNAPSHOT_CRYSTAL_PATH must be valid UTF-8, received `{}`",
            path.display()
        ))
    })
}
