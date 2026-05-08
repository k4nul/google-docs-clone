use std::{collections::HashMap, path::PathBuf};

use bytes::Bytes;
use kstone_core::{DatabaseConfig, Item, Key, LsmEngine, Value, index::TableSchema};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

const SNAPSHOT_CATALOG_KEY: &str = "__catalog__";
const SNAPSHOT_KEY_PREFIX: &str = "snapshot:";
const PAYLOAD_ATTR: &str = "payload";

pub struct KstoneSnapshotStore {
    path: PathBuf,
    engine: LsmEngine,
}

impl KstoneSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_KSTONE_PATH cannot be empty when SNAPSHOT_STORE=kstone".to_owned(),
            ));
        }

        ensure_snapshot_dir(&path)?;

        let engine = if path.join("wal.log").exists() {
            LsmEngine::open(&path)
        } else {
            LsmEngine::create_with_config(
                &path,
                DatabaseConfig::new().with_max_memtable_records(1),
                TableSchema::new(),
            )
        }
        .map_err(|error| Self::map_open_error(&path, error))?;

        Ok(Self { path, engine })
    }

    fn snapshot_key(doc_id: &Uuid) -> String {
        format!("{SNAPSHOT_KEY_PREFIX}{doc_id}")
    }

    fn map_open_error(path: &PathBuf, error: kstone_core::Error) -> StorageError {
        StorageError::Io(format!("{}: {error}", path.display()))
    }

    fn map_error(&self, action: &str, error: kstone_core::Error) -> StorageError {
        StorageError::Io(format!("{}: {action} failed: {error}", self.path.display()))
    }

    fn key(key: &str) -> Key {
        Key::new(Bytes::copy_from_slice(key.as_bytes()))
    }

    fn item(payload: Vec<u8>) -> Item {
        HashMap::from([(PAYLOAD_ATTR.to_owned(), Value::B(Bytes::from(payload)))])
    }

    fn payload_from_item(&self, item: Item, key: &str) -> Result<Vec<u8>, StorageError> {
        match item.get(PAYLOAD_ATTR) {
            Some(Value::B(payload)) => Ok(payload.to_vec()),
            _ => Err(StorageError::Io(format!(
                "{}: kstone snapshot value `{key}` is missing binary payload",
                self.path.display()
            ))),
        }
    }

    fn read_value(&self, key: &str) -> Result<Option<Vec<u8>>, StorageError> {
        let item = self
            .engine
            .get(&Self::key(key))
            .map_err(|error| self.map_error("read kstone snapshot value", error))?;

        item.map(|item| self.payload_from_item(item, key))
            .transpose()
    }

    fn write_value(&self, key: &str, payload: Vec<u8>) -> Result<(), StorageError> {
        self.engine
            .put(Self::key(key), Self::item(payload))
            .map_err(|error| self.map_error("write kstone snapshot value", error))
    }

    fn delete_value(&self, key: &str) -> Result<(), StorageError> {
        self.engine
            .delete(Self::key(key))
            .map_err(|error| self.map_error("delete kstone snapshot value", error))
    }

    fn flush(&self) -> Result<(), StorageError> {
        self.engine
            .flush()
            .map_err(|error| self.map_error("flush kstone snapshot store", error))
    }

    fn read_catalog(&self) -> Result<Vec<Uuid>, StorageError> {
        let Some(payload) = self.read_value(SNAPSHOT_CATALOG_KEY)? else {
            return Ok(Vec::new());
        };

        serde_json::from_slice::<Vec<Uuid>>(&payload).map_err(|_| {
            StorageError::Io(format!(
                "{}: kstone snapshot catalog is corrupt",
                self.path.display()
            ))
        })
    }

    fn write_catalog(&self, catalog: &[Uuid]) -> Result<(), StorageError> {
        let payload = serde_json::to_vec(catalog).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize kstone snapshot catalog: {error}"
            ))
        })?;

        self.write_value(SNAPSHOT_CATALOG_KEY, payload)
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

impl SnapshotStore for KstoneSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let Some(payload) = self.read_value(&Self::snapshot_key(doc_id))? else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &payload).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize kstone snapshot `{doc_id}`: {error}"
            ))
        })?;

        self.write_value(&Self::snapshot_key(&doc_id), payload)?;

        let mut catalog = self.read_catalog()?;
        if !catalog.contains(&doc_id) {
            catalog.push(doc_id);
            catalog.sort_unstable();
            self.write_catalog(&catalog)?;
        }

        self.flush()
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        self.delete_value(&Self::snapshot_key(doc_id))?;

        let mut catalog = self.read_catalog()?;
        let original_len = catalog.len();
        catalog.retain(|catalog_doc_id| catalog_doc_id != doc_id);
        if catalog.len() != original_len {
            self.write_catalog(&catalog)?;
        }

        self.flush()
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let catalog = self.read_catalog()?;
        let mut documents = Vec::new();

        for doc_id in catalog {
            match self.load_snapshot(&doc_id) {
                Ok(Some(snapshot)) => documents.push(snapshot.document),
                Ok(None) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing kstone snapshot referenced by catalog"
                ),
                Err(StorageError::CorruptSnapshot(_)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt kstone snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
