use std::{fs, path::PathBuf};

use jammdb::{DB, Error as JammdbError};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOTS_BUCKET: &str = "snapshots";

pub struct JammdbSnapshotStore {
    path: PathBuf,
    database: DB,
}

impl JammdbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_JAMMDB_PATH cannot be empty when SNAPSHOT_STORE=jammdb".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let database = DB::open(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;
        let store = Self { path, database };
        store.initialize_schema()?;
        Ok(store)
    }

    fn initialize_schema(&self) -> Result<(), StorageError> {
        let tx = self.open_write_tx()?;
        tx.get_or_create_bucket(SNAPSHOTS_BUCKET)
            .map_err(|error| self.map_error(error))?;
        tx.commit().map_err(|error| self.map_error(error))
    }

    fn open_read_tx(&self) -> Result<jammdb::Tx<'_>, StorageError> {
        self.database
            .tx(false)
            .map_err(|error| self.map_error(error))
    }

    fn open_write_tx(&self) -> Result<jammdb::Tx<'_>, StorageError> {
        self.database
            .tx(true)
            .map_err(|error| self.map_error(error))
    }

    fn map_error(&self, error: JammdbError) -> StorageError {
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

impl SnapshotStore for JammdbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let tx = self.open_read_tx()?;
        let bucket = match tx.get_bucket(SNAPSHOTS_BUCKET) {
            Ok(bucket) => bucket,
            Err(JammdbError::BucketMissing) => return Ok(None),
            Err(error) => return Err(self.map_error(error)),
        };
        let doc_id_key = doc_id.to_string();
        let Some(data) = bucket.get(doc_id_key.as_bytes()) else {
            return Ok(None);
        };
        if !data.is_kv() {
            return Err(StorageError::CorruptSnapshot(*doc_id));
        }

        self.deserialize_snapshot(*doc_id, data.kv().value())
            .map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let doc_id_key = doc_id.to_string();
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize jammdb snapshot `{doc_id}`: {error}"
            ))
        })?;

        let tx = self.open_write_tx()?;
        let bucket = tx
            .get_or_create_bucket(SNAPSHOTS_BUCKET)
            .map_err(|error| self.map_error(error))?;
        bucket
            .put(doc_id_key.as_str(), bytes.as_slice())
            .map_err(|error| self.map_error(error))?;
        tx.commit().map_err(|error| self.map_error(error))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let tx = self.open_write_tx()?;
        let bucket = match tx.get_bucket(SNAPSHOTS_BUCKET) {
            Ok(bucket) => bucket,
            Err(JammdbError::BucketMissing) => return Ok(()),
            Err(error) => return Err(self.map_error(error)),
        };
        let doc_id_key = doc_id.to_string();
        match bucket.delete(doc_id_key.as_bytes()) {
            Ok(_) | Err(JammdbError::KeyValueMissing) => {}
            Err(error) => return Err(self.map_error(error)),
        }
        tx.commit().map_err(|error| self.map_error(error))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let tx = self.open_read_tx()?;
        let bucket = match tx.get_bucket(SNAPSHOTS_BUCKET) {
            Ok(bucket) => bucket,
            Err(JammdbError::BucketMissing) => return Ok(Vec::new()),
            Err(error) => return Err(self.map_error(error)),
        };
        let mut documents = Vec::new();

        for data in bucket.cursor() {
            if !data.is_kv() {
                continue;
            }

            let kv = data.kv();
            let Ok(doc_id_key) = std::str::from_utf8(kv.key()) else {
                continue;
            };
            let Ok(doc_id) = Uuid::parse_str(doc_id_key) else {
                continue;
            };

            match self.deserialize_snapshot(doc_id, kv.value()) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt jammdb snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
