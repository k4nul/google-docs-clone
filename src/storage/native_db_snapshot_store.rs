use std::{fs, path::PathBuf, sync::LazyLock};

use native_db::{Builder, Database, Models, ToKey, native_db};
use native_model::{Model, native_model};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

static SNAPSHOT_MODELS: LazyLock<Models> = LazyLock::new(|| {
    let mut models = Models::new();
    models
        .define::<NativeDbSnapshotRecord>()
        .expect("native_db snapshot model should register");
    models
});

#[derive(Debug, Clone, Serialize, Deserialize)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct NativeDbSnapshotRecord {
    #[primary_key]
    doc_id: String,
    payload: Vec<u8>,
}

pub struct NativeDbSnapshotStore {
    path: PathBuf,
    database: Database<'static>,
}

impl NativeDbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_NATIVE_DB_PATH cannot be empty when SNAPSHOT_STORE=native_db".to_owned(),
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
            Builder::new()
                .open(&SNAPSHOT_MODELS, &path)
                .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?
        } else {
            Builder::new()
                .create(&SNAPSHOT_MODELS, &path)
                .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?
        };

        Ok(Self { path, database })
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        payload: &[u8],
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot = serde_json::from_slice::<PersistedSnapshot>(payload)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot: DocumentSnapshot = snapshot.into();

        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }
}

impl SnapshotStore for NativeDbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let doc_id_key = doc_id.to_string();
        let transaction = self
            .database
            .r_transaction()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        let Some(record) = transaction
            .get()
            .primary::<NativeDbSnapshotRecord>(doc_id_key)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?
        else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &record.payload)
            .map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize native_db snapshot `{doc_id}`: {error}"
            ))
        })?;
        let record = NativeDbSnapshotRecord {
            doc_id: doc_id.to_string(),
            payload,
        };

        let transaction = self
            .database
            .rw_transaction()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        transaction
            .upsert(record)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        transaction
            .commit()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        Ok(())
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let doc_id_key = doc_id.to_string();
        let transaction = self
            .database
            .rw_transaction()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        if let Some(record) = transaction
            .get()
            .primary::<NativeDbSnapshotRecord>(doc_id_key)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?
        {
            transaction
                .remove(record)
                .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        }

        transaction
            .commit()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        Ok(())
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let transaction = self
            .database
            .r_transaction()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        let scan = transaction
            .scan()
            .primary::<NativeDbSnapshotRecord>()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        let records = scan
            .all()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        let mut documents = Vec::new();

        for record in records {
            let record = record
                .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
            let Ok(doc_id) = Uuid::parse_str(&record.doc_id) else {
                continue;
            };

            match self.deserialize_snapshot(doc_id, &record.payload) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt native_db snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
