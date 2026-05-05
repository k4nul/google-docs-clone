use std::{
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use mace::{Bucket, Mace, OpCode, Options};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOTS_BUCKET: &str = "snapshots";
const SNAPSHOT_CATALOG_KEY: &[u8] = b"__catalog__";

pub struct MaceSnapshotStore {
    path: PathBuf,
    database: Mace,
    bucket: Mutex<Bucket>,
}

impl MaceSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_MACE_PATH cannot be empty when SNAPSHOT_STORE=mace".to_owned(),
            ));
        }

        let options = Options::new(&path)
            .validate()
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;
        let database = Mace::new(options)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;
        let bucket = match database.get_bucket(SNAPSHOTS_BUCKET) {
            Ok(bucket) => bucket,
            Err(OpCode::NotFound) => database
                .new_bucket(SNAPSHOTS_BUCKET)
                .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?,
            Err(error) => return Err(StorageError::Io(format!("{}: {error}", path.display()))),
        };

        Ok(Self {
            path,
            database,
            bucket: Mutex::new(bucket),
        })
    }

    fn lock_bucket(&self) -> Result<MutexGuard<'_, Bucket>, StorageError> {
        self.bucket.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: mace snapshot store mutex was poisoned",
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

    fn load_catalog_from_view(
        &self,
        view: &mace::TxnView<'_>,
    ) -> Result<Vec<String>, StorageError> {
        let bytes = match view.get(SNAPSHOT_CATALOG_KEY) {
            Ok(value) => value.to_vec(),
            Err(OpCode::NotFound) => return Ok(Vec::new()),
            Err(error) => {
                return Err(StorageError::Io(format!(
                    "{}: {error}",
                    self.path.display()
                )));
            }
        };

        serde_json::from_slice::<Vec<String>>(&bytes).map_err(|_| {
            StorageError::Io(format!(
                "{}: mace snapshot catalog is corrupt",
                self.path.display()
            ))
        })
    }

    fn load_catalog_from_transaction(
        &self,
        transaction: &mace::TxnKV<'_>,
    ) -> Result<Vec<String>, StorageError> {
        let bytes = match transaction.get(SNAPSHOT_CATALOG_KEY) {
            Ok(value) => value.to_vec(),
            Err(OpCode::NotFound) => return Ok(Vec::new()),
            Err(error) => {
                return Err(StorageError::Io(format!(
                    "{}: {error}",
                    self.path.display()
                )));
            }
        };

        serde_json::from_slice::<Vec<String>>(&bytes).map_err(|_| {
            StorageError::Io(format!(
                "{}: mace snapshot catalog is corrupt",
                self.path.display()
            ))
        })
    }

    fn save_catalog(
        &self,
        transaction: &mace::TxnKV<'_>,
        catalog: &[String],
    ) -> Result<(), StorageError> {
        let bytes = serde_json::to_vec(catalog)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        transaction
            .upsert(SNAPSHOT_CATALOG_KEY, bytes.as_slice())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        Ok(())
    }

    fn sync(&self) -> Result<(), StorageError> {
        self.database
            .sync()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }
}

impl SnapshotStore for MaceSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let bucket = self.lock_bucket()?;
        let doc_id_key = doc_id.to_string();
        let view = bucket
            .view()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        let bytes = match view.get(doc_id_key.as_bytes()) {
            Ok(value) => value.to_vec(),
            Err(OpCode::NotFound) => return Ok(None),
            Err(error) => {
                return Err(StorageError::Io(format!(
                    "{}: {error}",
                    self.path.display()
                )));
            }
        };

        self.deserialize_snapshot(*doc_id, &bytes).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let bucket = self.lock_bucket()?;
        let doc_id = snapshot.document.id;
        let doc_id_key = doc_id.to_string();
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize mace snapshot `{doc_id}`: {error}"
            ))
        })?;
        let transaction = bucket
            .begin()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        let mut catalog = self.load_catalog_from_transaction(&transaction)?;

        transaction
            .upsert(doc_id_key.as_bytes(), bytes.as_slice())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        if !catalog.iter().any(|value| value == &doc_id_key) {
            catalog.push(doc_id_key);
            catalog.sort();
        }
        self.save_catalog(&transaction, &catalog)?;
        transaction
            .commit()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        drop(bucket);
        self.sync()
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let bucket = self.lock_bucket()?;
        let doc_id_key = doc_id.to_string();
        let transaction = bucket
            .begin()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        let mut catalog = self.load_catalog_from_transaction(&transaction)?;

        match transaction.del(doc_id_key.as_bytes()) {
            Ok(_) | Err(OpCode::NotFound) => {}
            Err(error) => {
                return Err(StorageError::Io(format!(
                    "{}: {error}",
                    self.path.display()
                )));
            }
        }
        catalog.retain(|value| value != &doc_id_key);
        self.save_catalog(&transaction, &catalog)?;
        transaction
            .commit()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        drop(bucket);
        self.sync()
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let bucket = self.lock_bucket()?;
        let view = bucket
            .view()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        let catalog = self.load_catalog_from_view(&view)?;
        let mut documents = Vec::new();

        for doc_id_key in catalog {
            let Ok(doc_id) = Uuid::parse_str(&doc_id_key) else {
                continue;
            };

            match view.get(doc_id_key.as_bytes()) {
                Ok(value) => match self.deserialize_snapshot(doc_id, value.slice()) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping corrupt mace snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                },
                Err(OpCode::NotFound) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing mace snapshot while building document catalog"
                ),
                Err(error) => {
                    return Err(StorageError::Io(format!(
                        "{}: {error}",
                        self.path.display()
                    )));
                }
            }
        }

        documents.sort_by_key(|document| document.id);
        Ok(documents)
    }
}
