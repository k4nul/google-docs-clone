use std::{collections::BTreeMap, path::PathBuf, sync::Mutex};

use chrono::Utc;
use feoxdb::FeoxStore;
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOT_KEY_PREFIX: &str = "snapshot:";
const SNAPSHOT_KEY_END: &[u8] = b"snapshot;";
const TOMBSTONE_PAYLOAD: &[u8] = b"__deleted__";
const DEFAULT_FEOXDB_FILE_SIZE: u64 = 16 * 1024 * 1024;
const DEFAULT_FEOXDB_MAX_MEMORY: usize = 64 * 1024 * 1024;
const DEFAULT_FEOXDB_HASH_BITS: u32 = 10;
const MAX_FEOXDB_RANGE_EVENTS: usize = usize::MAX;

pub struct FeoxdbSnapshotStore {
    path: PathBuf,
    store: Mutex<FeoxStore>,
}

impl FeoxdbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_FEOXDB_PATH cannot be empty when SNAPSHOT_STORE=feoxdb".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }
        let store = FeoxStore::builder()
            .device_path(path.to_string_lossy().into_owned())
            .file_size(DEFAULT_FEOXDB_FILE_SIZE)
            .max_memory(DEFAULT_FEOXDB_MAX_MEMORY)
            .hash_bits(DEFAULT_FEOXDB_HASH_BITS)
            .enable_caching(false)
            .build()
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            store: Mutex::new(store),
        })
    }

    fn snapshot_key_prefix(doc_id: &Uuid) -> String {
        format!("{SNAPSHOT_KEY_PREFIX}{doc_id}:")
    }

    fn snapshot_key(doc_id: &Uuid) -> Vec<u8> {
        let timestamp = Utc::now()
            .timestamp_nanos_opt()
            .unwrap_or_else(|| Utc::now().timestamp_micros() * 1_000);
        format!(
            "{}{:020}:{}",
            Self::snapshot_key_prefix(doc_id),
            timestamp.max(0),
            Uuid::new_v4()
        )
        .into_bytes()
    }

    fn snapshot_key_range(doc_id: &Uuid) -> (Vec<u8>, Vec<u8>) {
        let start = Self::snapshot_key_prefix(doc_id).into_bytes();
        let end = format!("{SNAPSHOT_KEY_PREFIX}{doc_id};").into_bytes();
        (start, end)
    }

    fn doc_id_from_snapshot_key(key: &[u8]) -> Option<Uuid> {
        let key = std::str::from_utf8(key).ok()?;
        let remainder = key.strip_prefix(SNAPSHOT_KEY_PREFIX)?;
        let (doc_id, _) = remainder.split_once(':')?;
        Uuid::parse_str(doc_id).ok()
    }

    fn lock_store(&self) -> Result<std::sync::MutexGuard<'_, FeoxStore>, StorageError> {
        self.store.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: feoxdb mutex was poisoned",
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

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        payload: &[u8],
    ) -> Result<Option<DocumentSnapshot>, StorageError> {
        if payload == TOMBSTONE_PAYLOAD {
            return Ok(None);
        }

        let snapshot = serde_json::from_slice::<PersistedSnapshot>(payload)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot: DocumentSnapshot = snapshot.into();

        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(Some(snapshot))
    }

    fn latest_payload_for_doc(
        &self,
        store: &FeoxStore,
        doc_id: &Uuid,
    ) -> Result<Option<Vec<u8>>, StorageError> {
        let (start, end) = Self::snapshot_key_range(doc_id);
        let events = store
            .range_query(&start, &end, MAX_FEOXDB_RANGE_EVENTS)
            .map_err(|error| self.map_error("range feoxdb snapshot events", error))?;

        Ok(events.into_iter().last().map(|(_, payload)| payload))
    }
}

impl SnapshotStore for FeoxdbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let store = self.lock_store()?;
        let Some(payload) = self.latest_payload_for_doc(&store, doc_id)? else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &payload)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize feoxdb snapshot `{doc_id}`: {error}"
            ))
        })?;

        let store = self.lock_store()?;
        store
            .insert(&Self::snapshot_key(&doc_id), &payload)
            .map_err(|error| self.map_error("write feoxdb snapshot", error))?;

        store.flush();
        Ok(())
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let store = self.lock_store()?;
        store
            .insert(&Self::snapshot_key(doc_id), TOMBSTONE_PAYLOAD)
            .map_err(|error| self.map_error("write feoxdb snapshot tombstone", error))?;

        store.flush();
        Ok(())
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let store = self.lock_store()?;
        let events = store
            .range_query(
                SNAPSHOT_KEY_PREFIX.as_bytes(),
                SNAPSHOT_KEY_END,
                MAX_FEOXDB_RANGE_EVENTS,
            )
            .map_err(|error| self.map_error("range feoxdb snapshot catalog", error))?;
        let mut latest_by_doc_id = BTreeMap::<Uuid, Vec<u8>>::new();

        for (key, payload) in events {
            let Some(doc_id) = Self::doc_id_from_snapshot_key(&key) else {
                continue;
            };

            latest_by_doc_id.insert(doc_id, payload);
        }

        let mut documents = Vec::new();

        for (doc_id, payload) in latest_by_doc_id {
            match self.deserialize_snapshot(doc_id, &payload) {
                Ok(Some(snapshot)) => documents.push(snapshot.document),
                Ok(None) => {}
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt feoxdb snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
