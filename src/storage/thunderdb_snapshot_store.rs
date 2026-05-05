use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use thunderdb::{Database, Error as ThunderdbError};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOTS_BUCKET: &[u8] = b"snapshots";

pub struct ThunderdbSnapshotStore {
    path: PathBuf,
    database: Mutex<Database>,
}

impl ThunderdbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_THUNDERDB_PATH cannot be empty when SNAPSHOT_STORE=thunderdb".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let mut database = Database::open(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;
        let mut transaction = database.write_tx();
        transaction
            .create_bucket_if_not_exists(SNAPSHOTS_BUCKET)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;
        transaction
            .commit()
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            database: Mutex::new(database),
        })
    }

    fn lock_database(&self) -> Result<MutexGuard<'_, Database>, StorageError> {
        self.database.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: thunderdb database mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn map_error(&self, error: ThunderdbError) -> StorageError {
        StorageError::Io(format!("{}: {error}", self.path.display()))
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

impl SnapshotStore for ThunderdbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let database = self.lock_database()?;
        let transaction = database.read_tx();
        let bucket = match transaction.bucket(SNAPSHOTS_BUCKET) {
            Ok(bucket) => bucket,
            Err(ThunderdbError::BucketNotFound { .. }) => return Ok(None),
            Err(error) => return Err(self.map_error(error)),
        };
        let Some(bytes) = bucket.get(doc_id.as_bytes()) else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, bytes).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize thunderdb snapshot `{doc_id}`: {error}"
            ))
        })?;
        let mut database = self.lock_database()?;
        let mut transaction = database.write_tx();
        transaction
            .create_bucket_if_not_exists(SNAPSHOTS_BUCKET)
            .map_err(|error| self.map_error(error))?;
        transaction
            .bucket_put(SNAPSHOTS_BUCKET, doc_id.as_bytes(), &bytes)
            .map_err(|error| self.map_error(error))?;
        transaction.commit().map_err(|error| self.map_error(error))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut database = self.lock_database()?;
        let mut transaction = database.write_tx();
        transaction
            .create_bucket_if_not_exists(SNAPSHOTS_BUCKET)
            .map_err(|error| self.map_error(error))?;
        transaction
            .bucket_delete(SNAPSHOTS_BUCKET, doc_id.as_bytes())
            .map_err(|error| self.map_error(error))?;
        transaction.commit().map_err(|error| self.map_error(error))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let database = self.lock_database()?;
        let transaction = database.read_tx();
        let bucket = match transaction.bucket(SNAPSHOTS_BUCKET) {
            Ok(bucket) => bucket,
            Err(ThunderdbError::BucketNotFound { .. }) => return Ok(Vec::new()),
            Err(error) => return Err(self.map_error(error)),
        };
        let mut documents = Vec::new();

        for (key, value) in bucket.iter() {
            let Ok(doc_id) = Uuid::from_slice(key) else {
                continue;
            };

            match self.deserialize_snapshot(doc_id, value) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt thunderdb snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
