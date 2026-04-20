use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use nikidb::{
    db::{DB, DEFAULT_OPTIONS},
    error::NKError,
};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOT_BUCKET_NAME: &[u8] = b"snapshots";
const SNAPSHOT_CATALOG_KEY: &[u8] = b"__catalog__";
const EMPTY_SNAPSHOT_CATALOG: &[u8] = b"[]";

pub struct NikidbSnapshotStore {
    path: PathBuf,
    database: Mutex<DB>,
}

impl NikidbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_NIKIDB_PATH cannot be empty when SNAPSHOT_STORE=nikidb".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let database = DB::open(path.to_string_lossy().as_ref(), DEFAULT_OPTIONS)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;
        let store = Self {
            path,
            database: Mutex::new(database),
        };
        store.ensure_bucket_exists()?;

        Ok(store)
    }

    fn lock_database(&self) -> Result<MutexGuard<'_, DB>, StorageError> {
        self.database.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: nikidb database mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn ensure_bucket_exists(&self) -> Result<(), StorageError> {
        let database = self.lock_database()?;
        database
            .update(Box::new(|tx| {
                match tx.create_bucket(SNAPSHOT_BUCKET_NAME) {
                    Ok(bucket) => {
                        bucket.put(SNAPSHOT_CATALOG_KEY, EMPTY_SNAPSHOT_CATALOG)?;
                        Ok(())
                    }
                    Err(NKError::ErrBucketExists(_)) => Ok(()),
                    Err(NKError::ErrBucketNotFound) => {
                        let bucket = tx.create_bucket(SNAPSHOT_BUCKET_NAME)?;
                        bucket.put(SNAPSHOT_CATALOG_KEY, EMPTY_SNAPSHOT_CATALOG)?;
                        Ok(())
                    }
                    Err(error) => Err(error),
                }
            }))
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn serialize_catalog(catalog: &[String]) -> Result<Vec<u8>, NKError> {
        serde_json::to_vec(catalog).map_err(|error| {
            NKError::Unexpected(format!("failed to serialize snapshot catalog: {error}"))
        })
    }

    fn deserialize_catalog(bytes: &[u8]) -> Result<Vec<String>, NKError> {
        serde_json::from_slice(bytes)
            .map_err(|_| NKError::Unexpected("snapshot catalog is corrupt".to_owned()))
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

impl SnapshotStore for NikidbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let database = self.lock_database()?;
        let doc_id_key = doc_id.to_string();
        let mut raw_snapshot = None;

        database
            .view(Box::new(|tx| {
                let bucket = match tx.bucket(SNAPSHOT_BUCKET_NAME) {
                    Ok(bucket) => bucket,
                    Err(NKError::ErrBucketNotFound) => return Ok(()),
                    Err(error) => return Err(error),
                };
                raw_snapshot = bucket
                    .get(doc_id_key.as_bytes())
                    .map(|bytes| bytes.to_vec());
                Ok(())
            }))
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        let Some(raw_snapshot) = raw_snapshot else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &raw_snapshot).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let database = self.lock_database()?;
        let doc_id = snapshot.document.id;
        let doc_id_key = doc_id.to_string();
        let raw_snapshot =
            serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
                StorageError::Io(format!(
                    "failed to serialize nikidb snapshot `{doc_id}`: {error}"
                ))
            })?;

        database
            .update(Box::new(|tx| {
                let bucket = match tx.bucket(SNAPSHOT_BUCKET_NAME) {
                    Ok(bucket) => bucket,
                    Err(NKError::ErrBucketNotFound) => tx.create_bucket(SNAPSHOT_BUCKET_NAME)?,
                    Err(error) => return Err(error),
                };
                let mut catalog = match bucket.get(SNAPSHOT_CATALOG_KEY) {
                    Some(bytes) => Self::deserialize_catalog(bytes)?,
                    None => Vec::new(),
                };

                bucket.put(doc_id_key.as_bytes(), &raw_snapshot)?;
                if !catalog.iter().any(|value| value == &doc_id_key) {
                    catalog.push(doc_id_key.clone());
                    catalog.sort();
                }

                let raw_catalog = Self::serialize_catalog(&catalog)?;
                bucket.put(SNAPSHOT_CATALOG_KEY, &raw_catalog)?;
                Ok(())
            }))
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let database = self.lock_database()?;
        let doc_id_key = doc_id.to_string();

        database
            .update(Box::new(|tx| {
                let bucket = match tx.bucket(SNAPSHOT_BUCKET_NAME) {
                    Ok(bucket) => bucket,
                    Err(NKError::ErrBucketNotFound) => return Ok(()),
                    Err(error) => return Err(error),
                };
                let mut catalog = match bucket.get(SNAPSHOT_CATALOG_KEY) {
                    Some(bytes) => Self::deserialize_catalog(bytes)?,
                    None => Vec::new(),
                };

                bucket.delete(doc_id_key.as_bytes())?;
                catalog.retain(|value| value != &doc_id_key);
                let raw_catalog = Self::serialize_catalog(&catalog)?;
                bucket.put(SNAPSHOT_CATALOG_KEY, &raw_catalog)?;
                Ok(())
            }))
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let database = self.lock_database()?;
        let mut listed_snapshots = Vec::new();
        let mut missing_ids = Vec::new();

        database
            .view(Box::new(|tx| {
                let bucket = match tx.bucket(SNAPSHOT_BUCKET_NAME) {
                    Ok(bucket) => bucket,
                    Err(NKError::ErrBucketNotFound) => return Ok(()),
                    Err(error) => return Err(error),
                };
                let catalog = match bucket.get(SNAPSHOT_CATALOG_KEY) {
                    Some(bytes) => Self::deserialize_catalog(bytes)?,
                    None => Vec::new(),
                };

                for doc_id_key in catalog {
                    let Ok(doc_id) = Uuid::parse_str(&doc_id_key) else {
                        continue;
                    };

                    match bucket.get(doc_id_key.as_bytes()) {
                        Some(bytes) => listed_snapshots.push((doc_id, bytes.to_vec())),
                        None => missing_ids.push(doc_id),
                    }
                }

                Ok(())
            }))
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        let mut documents = Vec::new();
        for missing_id in missing_ids {
            tracing::warn!(
                doc_id = %missing_id,
                path = %self.path.display(),
                "skipping missing nikidb snapshot while building document catalog"
            );
        }

        for (doc_id, raw_snapshot) in listed_snapshots {
            match self.deserialize_snapshot(doc_id, &raw_snapshot) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt nikidb snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
