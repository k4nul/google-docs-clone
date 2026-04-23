use std::{
    any::Any,
    fs,
    panic::{AssertUnwindSafe, catch_unwind},
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
};

use kagi::{Store, open};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

pub struct KagiSnapshotStore {
    path: PathBuf,
    store: Mutex<Store>,
}

impl KagiSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = normalize_path(path.into())?;
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let store = catch_unwind(AssertUnwindSafe(|| open(path.clone()))).map_err(|payload| {
            panic_to_storage_error(&path, "open kagi snapshot store", payload)
        })?;

        Ok(Self {
            path,
            store: Mutex::new(store),
        })
    }

    fn lock_store(&self) -> Result<MutexGuard<'_, Store>, StorageError> {
        self.store.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: kagi snapshot store mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn snapshot_key(doc_id: Uuid) -> String {
        doc_id.to_string()
    }

    fn serialize_snapshot(&self, snapshot: DocumentSnapshot) -> Result<String, StorageError> {
        let doc_id = snapshot.document.id;
        serde_json::to_string(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "{}: failed to serialize kagi snapshot `{doc_id}`: {error}",
                self.path.display()
            ))
        })
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

    fn store_payload<'a>(
        &self,
        expected_doc_id: Uuid,
        payload: &'a str,
    ) -> Result<&'a str, StorageError> {
        if payload.is_empty() {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(payload)
    }

    fn persist(&self, store: &Store, operation: &'static str) -> Result<(), StorageError> {
        catch_unwind(AssertUnwindSafe(|| store.save()))
            .map_err(|payload| panic_to_storage_error(&self.path, operation, payload))
    }
}

impl SnapshotStore for KagiSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let store = self.lock_store()?;
        let Some(payload) = store.map.get(&Self::snapshot_key(*doc_id)) else {
            return Ok(None);
        };
        let payload = self.store_payload(*doc_id, payload)?;

        self.deserialize_snapshot(*doc_id, payload).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload = self.serialize_snapshot(snapshot)?;
        let mut store = self.lock_store()?;
        store.insert(Self::snapshot_key(doc_id), payload);
        self.persist(&store, "persist kagi snapshot")
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut store = self.lock_store()?;
        store.map.remove(&Self::snapshot_key(*doc_id));
        self.persist(&store, "persist kagi snapshot deletion")
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let store = self.lock_store()?;
        let mut documents = Vec::new();

        for (doc_id_key, payload) in &store.map {
            let Ok(doc_id) = Uuid::parse_str(doc_id_key) else {
                continue;
            };

            let payload = match self.store_payload(doc_id, payload) {
                Ok(payload) => payload,
                Err(StorageError::CorruptSnapshot(doc_id)) => {
                    tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping corrupt kagi snapshot while building document catalog"
                    );
                    continue;
                }
                Err(error) => return Err(error),
            };

            match self.deserialize_snapshot(doc_id, payload) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt kagi snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        documents.sort_unstable_by_key(|document| document.id);
        Ok(documents)
    }
}

fn normalize_path(path: PathBuf) -> Result<PathBuf, StorageError> {
    if path.as_os_str().is_empty() {
        return Err(StorageError::Config(
            "SNAPSHOT_KAGI_PATH cannot be empty when SNAPSHOT_STORE=kagi".to_owned(),
        ));
    }

    if has_parent(&path) {
        Ok(path)
    } else {
        Ok(PathBuf::from(".").join(path))
    }
}

fn has_parent(path: &Path) -> bool {
    path.parent()
        .is_some_and(|parent| !parent.as_os_str().is_empty())
}

fn panic_to_storage_error(
    path: &Path,
    operation: &str,
    payload: Box<dyn Any + Send>,
) -> StorageError {
    StorageError::Io(format!(
        "{}: failed to {operation}: {}",
        path.display(),
        panic_message(payload)
    ))
}

fn panic_message(payload: Box<dyn Any + Send>) -> String {
    match payload.downcast::<String>() {
        Ok(message) => *message,
        Err(payload) => match payload.downcast::<&'static str>() {
            Ok(message) => (*message).to_owned(),
            Err(_) => "unknown panic".to_owned(),
        },
    }
}
