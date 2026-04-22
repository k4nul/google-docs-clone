use std::{
    fs::{self, File},
    io::Write,
    ops::Bound,
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
};

use bytes::Bytes;
use loro_kv_store::{MemKvStore, mem_store::MemKvConfig};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

pub struct LoroKvSnapshotStore {
    path: PathBuf,
    store: Mutex<MemKvStore>,
}

impl LoroKvSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_LORO_KV_PATH cannot be empty when SNAPSHOT_STORE=loro_kv".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let mut store = MemKvConfig::new().build();
        match fs::read(&path) {
            Ok(bytes) if !bytes.is_empty() => store
                .import_all(Bytes::from(bytes))
                .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?,
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(StorageError::Io(format!("{}: {error}", path.display())));
            }
        }

        Ok(Self {
            path,
            store: Mutex::new(store),
        })
    }

    fn lock_store(&self) -> Result<MutexGuard<'_, MemKvStore>, StorageError> {
        self.store.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: loro_kv mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn key(doc_id: &Uuid) -> String {
        doc_id.to_string()
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        bytes: Bytes,
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot = serde_json::from_slice::<PersistedSnapshot>(&bytes)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot: DocumentSnapshot = snapshot.into();

        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }

    fn persist_store(&self, store: &mut MemKvStore) -> Result<(), StorageError> {
        let data = store.export_all();
        atomic_write(&self.path, data.as_ref())
    }
}

impl SnapshotStore for LoroKvSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let store = self.lock_store()?;
        let Some(bytes) = store.get(Self::key(doc_id).as_bytes()) else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, bytes).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let value = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize loro_kv snapshot `{doc_id}`: {error}"
            ))
        })?;

        let mut store = self.lock_store()?;
        store.set(Self::key(&doc_id).as_bytes(), Bytes::from(value));
        self.persist_store(&mut store)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut store = self.lock_store()?;
        store.remove(Self::key(doc_id).as_bytes());
        self.persist_store(&mut store)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let store = self.lock_store()?;
        let mut documents = Vec::new();

        for (key, value) in store.scan(Bound::Unbounded, Bound::Unbounded) {
            let Ok(doc_id_key) = std::str::from_utf8(&key) else {
                continue;
            };
            let Ok(doc_id) = Uuid::parse_str(doc_id_key) else {
                continue;
            };

            match self.deserialize_snapshot(doc_id, value) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt loro_kv snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        documents.sort_unstable_by_key(|document| document.id);
        Ok(documents)
    }
}

fn atomic_write(path: &Path, data: &[u8]) -> Result<(), StorageError> {
    let tmp_path = path.with_extension("tmp");
    {
        let mut file = File::create(&tmp_path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", tmp_path.display())))?;
        file.write_all(data)
            .map_err(|error| StorageError::Io(format!("{}: {error}", tmp_path.display())))?;
        file.sync_all()
            .map_err(|error| StorageError::Io(format!("{}: {error}", tmp_path.display())))?;
    }

    fs::rename(&tmp_path, path).map_err(|error| {
        StorageError::Io(format!(
            "{} -> {}: {error}",
            tmp_path.display(),
            path.display()
        ))
    })?;

    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        let dir = File::open(parent)
            .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        dir.sync_all()
            .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
    }

    Ok(())
}
