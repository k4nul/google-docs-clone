use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard, OnceLock},
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use vsdb::Mapx;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

static VSDB_ACCESS_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

const VSDB_METADATA_FILE: &str = "store.meta.json";
#[derive(Debug, Serialize, Deserialize)]
struct VsdbMetadata {
    instance_id: u64,
}

pub struct VsdbSnapshotStore {
    root: PathBuf,
    metadata_path: PathBuf,
}

impl VsdbSnapshotStore {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let root = root.into();
        if root.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_VSDB_PATH cannot be empty when SNAPSHOT_STORE=vsdb".to_owned(),
            ));
        }

        fs::create_dir_all(&root)
            .map_err(|error| StorageError::Io(format!("{}: {error}", root.display())))?;
        let metadata_path = root.join(VSDB_METADATA_FILE);

        let store = Self {
            root,
            metadata_path,
        };
        store.ensure_store_initialized()?;

        Ok(store)
    }

    fn access_lock() -> &'static Mutex<()> {
        VSDB_ACCESS_LOCK.get_or_init(|| Mutex::new(()))
    }

    fn lock_access(&self) -> Result<MutexGuard<'static, ()>, StorageError> {
        Self::access_lock().lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: vsdb access lock was poisoned",
                self.root.display()
            ))
        })
    }

    fn map_vsdb_error(&self, context: &str, error: impl std::fmt::Display) -> StorageError {
        StorageError::Io(format!("{context} ({}): {error}", self.root.display()))
    }

    fn flush(&self) -> Result<(), StorageError> {
        vsdb::vsdb_flush();
        Ok(())
    }

    fn load_metadata(&self) -> Result<VsdbMetadata, StorageError> {
        let bytes = fs::read(&self.metadata_path).map_err(|error| {
            StorageError::Io(format!("{}: {error}", self.metadata_path.display()))
        })?;
        serde_json::from_slice(&bytes).map_err(|error| {
            StorageError::Io(format!(
                "failed to decode vsdb metadata `{}`: {error}",
                self.metadata_path.display()
            ))
        })
    }

    fn write_metadata(&self, metadata: &VsdbMetadata) -> Result<(), StorageError> {
        let bytes = serde_json::to_vec(metadata).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize vsdb metadata `{}`: {error}",
                self.metadata_path.display()
            ))
        })?;
        fs::write(&self.metadata_path, bytes)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.metadata_path.display())))
    }

    fn open_map(&self) -> Result<Mapx<String, Vec<u8>>, StorageError> {
        let metadata = self.load_metadata()?;
        Mapx::from_meta(metadata.instance_id)
            .map_err(|error| self.map_vsdb_error("failed to open vsdb snapshot map", error))
    }

    fn ensure_store_initialized(&self) -> Result<(), StorageError> {
        let _guard = self.lock_access()?;

        if self.metadata_path.exists() {
            self.open_map().map(|_| ())
        } else {
            let map = Mapx::<String, Vec<u8>>::new();
            let metadata = VsdbMetadata {
                instance_id: map.save_meta().map_err(|error| {
                    self.map_vsdb_error("failed to persist vsdb map metadata", error)
                })?,
            };
            self.write_metadata(&metadata)?;
            self.flush()
        }
    }

    fn with_map<T>(
        &self,
        op: impl FnOnce(&mut Mapx<String, Vec<u8>>) -> Result<T, StorageError>,
    ) -> Result<T, StorageError> {
        let _guard = self.lock_access()?;
        let mut map = self.open_map()?;
        op(&mut map)
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

impl SnapshotStore for VsdbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        self.with_map(|map| {
            let Some(bytes) = map.get(&doc_id.to_string()) else {
                return Ok(None);
            };

            self.deserialize_snapshot(*doc_id, &bytes).map(Some)
        })
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        self.with_map(|map| {
            let doc_id = snapshot.document.id;
            let payload =
                serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
                    StorageError::Io(format!(
                        "failed to serialize vsdb snapshot `{doc_id}`: {error}"
                    ))
                })?;

            map.insert(&doc_id.to_string(), &payload);
            self.flush()
        })
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        self.with_map(|map| {
            map.remove(&doc_id.to_string());
            self.flush()
        })
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        self.with_map(|map| {
            let mut documents = Vec::new();

            for (key, payload) in map.iter() {
                let Ok(doc_id) = Uuid::parse_str(&key) else {
                    continue;
                };

                match self.deserialize_snapshot(doc_id, &payload) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.root.display(),
                        "skipping corrupt vsdb snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                }
            }

            Ok(documents)
        })
    }
}
