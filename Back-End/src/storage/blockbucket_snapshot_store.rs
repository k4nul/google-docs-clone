use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use blockbucket::{Bucket, Trait};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const LIST_PAGE_SIZE: u8 = u8::MAX;

pub struct BlockbucketSnapshotStore {
    path: PathBuf,
    store: Mutex<Bucket>,
}

impl BlockbucketSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_BLOCKBUCKET_PATH cannot be empty when SNAPSHOT_STORE=blockbucket"
                    .to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let store = Bucket::new(path.to_string_lossy().into_owned())
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            store: Mutex::new(store),
        })
    }

    fn lock_store(&self) -> Result<MutexGuard<'_, Bucket>, StorageError> {
        self.store.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: blockbucket snapshot store mutex was poisoned",
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

    fn list_entries(&self, store: &mut Bucket) -> Vec<(Vec<u8>, Vec<u8>)> {
        let mut all_entries = Vec::new();
        let mut skip = 0usize;

        loop {
            let batch = store.list_next(LIST_PAGE_SIZE, skip);
            if batch.is_empty() {
                break;
            }

            skip += batch.len();
            all_entries.extend(batch);
        }

        all_entries
    }
}

impl SnapshotStore for BlockbucketSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let mut store = self.lock_store()?;
        let (stored_key, payload) = store.get(doc_id.to_string().into_bytes());
        if stored_key.is_empty() {
            return Ok(None);
        }

        self.deserialize_snapshot(*doc_id, &payload).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let mut store = self.lock_store()?;
        let doc_id = snapshot.document.id;
        let payload = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize blockbucket snapshot `{doc_id}`: {error}"
            ))
        })?;

        store
            .set(doc_id.to_string().into_bytes(), payload)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut store = self.lock_store()?;
        store
            .delete(doc_id.to_string().into_bytes())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut store = self.lock_store()?;
        let mut documents = Vec::new();

        for (key, payload) in self.list_entries(&mut store) {
            let Ok(key) = String::from_utf8(key) else {
                continue;
            };
            let Ok(doc_id) = Uuid::parse_str(&key) else {
                continue;
            };

            match self.deserialize_snapshot(doc_id, &payload) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt blockbucket snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
