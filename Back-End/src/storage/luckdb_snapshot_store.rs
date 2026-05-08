use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use luckdb::{Client, Query, config::DatabaseConfig};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const DATABASE_NAME: &str = "backend";
const COLLECTION_NAME: &str = "snapshots";
const DOC_ID_FIELD: &str = "doc_id";
const SNAPSHOT_FIELD: &str = "snapshot";

pub struct LuckdbSnapshotStore {
    path: PathBuf,
    client: Mutex<Client>,
}

impl LuckdbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_LUCKDB_PATH cannot be empty when SNAPSHOT_STORE=luckdb".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let config = DatabaseConfig::with_storage_path(&path);
        let mut client = Client::with_config(config)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;
        client
            .load()
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            client: Mutex::new(client),
        })
    }

    fn lock_client(&self) -> Result<MutexGuard<'_, Client>, StorageError> {
        self.client.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: luckdb mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn map_error(&self, operation: &str, error: impl std::fmt::Display) -> StorageError {
        StorageError::Io(format!(
            "{}: failed to {operation}: {error}",
            self.path.display()
        ))
    }

    fn query_doc_id(doc_id: Uuid) -> Query {
        Query::new().eq(DOC_ID_FIELD, Value::String(doc_id.to_string()))
    }

    fn serialize_snapshot(snapshot: DocumentSnapshot) -> Result<Value, StorageError> {
        let doc_id = snapshot.document.id;
        let persisted = PersistedSnapshot::from(snapshot);
        Ok(json!({
            DOC_ID_FIELD: doc_id.to_string(),
            SNAPSHOT_FIELD: persisted,
        }))
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        value: Value,
    ) -> Result<DocumentSnapshot, StorageError> {
        let Some(snapshot_value) = value.get(SNAPSHOT_FIELD).cloned() else {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        };
        let snapshot = serde_json::from_value::<PersistedSnapshot>(snapshot_value)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot: DocumentSnapshot = snapshot.into();

        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }
}

impl SnapshotStore for LuckdbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let mut client = self.lock_client()?;
        let documents = client
            .db(DATABASE_NAME)
            .collection(COLLECTION_NAME)
            .find(Self::query_doc_id(*doc_id), None)
            .map_err(|error| self.map_error("read luckdb snapshot", error))?;

        let Some((_, document)) = documents.into_iter().next() else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, document).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let value = Self::serialize_snapshot(snapshot)?;
        let mut client = self.lock_client()?;

        {
            let collection = client.db(DATABASE_NAME).collection(COLLECTION_NAME);
            collection
                .delete_many(Self::query_doc_id(doc_id))
                .map_err(|error| self.map_error("replace luckdb snapshot", error))?;
            collection
                .insert(value)
                .map_err(|error| self.map_error("write luckdb snapshot", error))?;
        }

        client
            .save()
            .map_err(|error| self.map_error("flush luckdb snapshot store", error))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut client = self.lock_client()?;

        client
            .db(DATABASE_NAME)
            .collection(COLLECTION_NAME)
            .delete_many(Self::query_doc_id(*doc_id))
            .map_err(|error| self.map_error("delete luckdb snapshot", error))?;

        client
            .save()
            .map_err(|error| self.map_error("flush luckdb snapshot store", error))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut client = self.lock_client()?;
        let values = client
            .db(DATABASE_NAME)
            .collection(COLLECTION_NAME)
            .find(Query::new(), None)
            .map_err(|error| self.map_error("list luckdb snapshots", error))?;

        let mut documents = Vec::new();
        for (_, value) in values {
            let Some(doc_id) = value
                .get(DOC_ID_FIELD)
                .and_then(Value::as_str)
                .and_then(|value| Uuid::parse_str(value).ok())
            else {
                continue;
            };

            match self.deserialize_snapshot(doc_id, value) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt luckdb snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        documents.sort_unstable_by_key(|document| document.id);
        Ok(documents)
    }
}
