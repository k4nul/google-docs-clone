use std::{path::PathBuf, sync::Mutex};

use blazeup::kv::{self, Record, Types};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

const BUCKET: &str = "snapshots";
const CATALOG_KEY: &str = "__catalog__";
const SNAPSHOT_KEY_PREFIX: &str = "snapshot:";

static BLAZEUP_LOCK: Mutex<()> = Mutex::new(());

pub struct BlazeupSnapshotStore {
    path: PathBuf,
}

impl BlazeupSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_BLAZEUP_PATH cannot be empty when SNAPSHOT_STORE=blazeup".to_owned(),
            ));
        }

        ensure_snapshot_dir(&path)?;
        let store = Self { path };
        store.with_database(|| Ok(()))?;
        Ok(store)
    }

    fn with_database<T>(
        &self,
        operation: impl FnOnce() -> Result<T, StorageError>,
    ) -> Result<T, StorageError> {
        let _guard = BLAZEUP_LOCK.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: blazeup global mutex was poisoned",
                self.path.display()
            ))
        })?;

        kv::init(Some(&self.path))
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        operation()
    }

    fn snapshot_key(doc_id: &Uuid) -> String {
        format!("{SNAPSHOT_KEY_PREFIX}{doc_id}")
    }

    fn record(name: &str, payload: String) -> Record {
        Record {
            name: name.to_owned(),
            values: vec![Types::String(payload)],
        }
    }

    fn record_payload(&self, key: &str, record: Record) -> Result<String, StorageError> {
        match record.values.as_slice() {
            [Types::String(payload)] => Ok(payload.clone()),
            _ => Err(StorageError::Io(format!(
                "{}: blazeup record `{key}` has unexpected value shape",
                self.path.display()
            ))),
        }
    }

    fn read_value(&self, key: &str) -> Result<Option<String>, StorageError> {
        self.with_database(|| {
            let Some(record) = kv::get(BUCKET, key) else {
                return Ok(None);
            };

            self.record_payload(key, record.record).map(Some)
        })
    }

    fn write_value(&self, key: &str, name: &str, payload: String) -> Result<(), StorageError> {
        self.with_database(|| {
            kv::set(BUCKET, key, Self::record(name, payload))
                .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
        })
    }

    fn remove_value(&self, key: &str) -> Result<(), StorageError> {
        self.with_database(|| {
            kv::remove(BUCKET, key)
                .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
        })
    }

    fn read_catalog(&self) -> Result<Vec<Uuid>, StorageError> {
        let Some(payload) = self.read_value(CATALOG_KEY)? else {
            return Ok(Vec::new());
        };

        serde_json::from_str::<Vec<Uuid>>(&payload).map_err(|error| {
            StorageError::Io(format!(
                "{}: failed to parse blazeup snapshot catalog: {error}",
                self.path.display()
            ))
        })
    }

    fn write_catalog(&self, mut catalog: Vec<Uuid>) -> Result<(), StorageError> {
        catalog.sort_unstable();
        catalog.dedup();
        let payload = serde_json::to_string(&catalog).map_err(|error| {
            StorageError::Io(format!(
                "{}: failed to serialize blazeup snapshot catalog: {error}",
                self.path.display()
            ))
        })?;

        self.write_value(CATALOG_KEY, "catalog", payload)
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        payload: &str,
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot = serde_json::from_str::<PersistedSnapshot>(payload)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot: DocumentSnapshot = snapshot.into();

        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }
}

impl SnapshotStore for BlazeupSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let Some(payload) = self.read_value(&Self::snapshot_key(doc_id))? else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &payload).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload =
            serde_json::to_string(&PersistedSnapshot::from(snapshot)).map_err(|error| {
                StorageError::Io(format!(
                    "failed to serialize blazeup snapshot `{doc_id}`: {error}"
                ))
            })?;

        self.write_value(&Self::snapshot_key(&doc_id), "snapshot", payload)?;

        let mut catalog = self.read_catalog()?;
        catalog.push(doc_id);
        self.write_catalog(catalog)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        self.remove_value(&Self::snapshot_key(doc_id))?;
        let mut catalog = self.read_catalog()?;
        catalog.retain(|candidate| candidate != doc_id);
        self.write_catalog(catalog)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let catalog = self.read_catalog()?;
        let mut documents = Vec::new();

        for doc_id in catalog {
            match self.load_snapshot(&doc_id)? {
                Some(snapshot) => documents.push(snapshot.document),
                None => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing blazeup snapshot referenced by catalog"
                ),
            }
        }

        documents.sort_unstable_by_key(|document| document.id);
        Ok(documents)
    }
}
