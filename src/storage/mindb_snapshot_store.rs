use std::{path::PathBuf, sync::Mutex};

use mindb::{
    db::{Database, DatabaseOptions},
    recovery::{RecoveryManager, RecoveryOptions},
    storage::CompressionCodec,
};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

const SNAPSHOT_CATALOG_KEY: &[u8] = b"__catalog__";
const SNAPSHOT_KEY_PREFIX: &str = "snapshot:";

pub struct MindbSnapshotStore {
    path: PathBuf,
    db: Mutex<Database>,
}

impl MindbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_MINDB_PATH cannot be empty when SNAPSHOT_STORE=mindb".to_owned(),
            ));
        }

        ensure_snapshot_dir(&path)?;
        let db = Database::open(DatabaseOptions {
            data_dir: path.clone(),
            wal_direct_io: false,
            compression: CompressionCodec::None,
            wal_max_batch_ops: 1,
            ..DatabaseOptions::new(&path)
        })
        .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            db: Mutex::new(db),
        })
    }

    fn snapshot_key(doc_id: &Uuid) -> Vec<u8> {
        format!("{SNAPSHOT_KEY_PREFIX}{doc_id}").into_bytes()
    }

    fn lock_db(&self) -> Result<std::sync::MutexGuard<'_, Database>, StorageError> {
        self.db.lock().map_err(|_| {
            StorageError::Io(format!("{}: mindb mutex was poisoned", self.path.display()))
        })
    }

    fn map_error(&self, operation: &str, error: impl std::fmt::Display) -> StorageError {
        StorageError::Io(format!(
            "{}: failed to {operation}: {error}",
            self.path.display()
        ))
    }

    fn read_catalog(&self, db: &Database) -> Result<Vec<Uuid>, StorageError> {
        let Some(payload) =
            self.read_value(db, SNAPSHOT_CATALOG_KEY, "read mindb snapshot catalog")?
        else {
            return Ok(Vec::new());
        };

        serde_json::from_slice::<Vec<Uuid>>(&payload).map_err(|_| {
            StorageError::Io(format!(
                "{}: mindb snapshot catalog is corrupt",
                self.path.display()
            ))
        })
    }

    fn serialize_catalog(&self, catalog: &[Uuid]) -> Result<Vec<u8>, StorageError> {
        serde_json::to_vec(catalog).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize mindb snapshot catalog: {error}"
            ))
        })
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

    fn sync(&self, db: &Database, operation: &str) -> Result<(), StorageError> {
        db.sync()
            .map(|_| ())
            .map_err(|error| self.map_error(operation, error))
    }

    fn read_value(
        &self,
        db: &Database,
        key: &[u8],
        operation: &str,
    ) -> Result<Option<Vec<u8>>, StorageError> {
        if let Some(payload) = db
            .get(key)
            .map_err(|error| self.map_error(operation, error))?
        {
            return Ok(Some(payload));
        }

        self.recover_value_from_wal(key)
    }

    fn recover_value_from_wal(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        let wal_path = self.path.join("wal.log");
        let manifest_path = self.path.join("manifest.json");
        if !wal_path.exists() || !manifest_path.exists() {
            return Ok(None);
        }

        let outcome = RecoveryManager::new(RecoveryOptions::new(wal_path, manifest_path))
            .recover()
            .map_err(|error| self.map_error("recover mindb WAL", error))?;
        let mut latest: Option<mindb::write::memtable::MemTableEntry> = None;

        for memtable in outcome.memtables {
            let Some(entry) = memtable.get(key) else {
                continue;
            };

            let should_replace = match latest.as_ref() {
                Some(current) => entry.sequence >= current.sequence,
                None => true,
            };

            if should_replace {
                latest = Some(entry);
            }
        }

        Ok(latest.and_then(|entry| {
            if entry.tombstone {
                None
            } else {
                Some(entry.value.as_ref().to_vec())
            }
        }))
    }
}

impl SnapshotStore for MindbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let db = self.lock_db()?;
        let Some(payload) =
            self.read_value(&db, &Self::snapshot_key(doc_id), "read mindb snapshot")?
        else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &payload).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize mindb snapshot `{doc_id}`: {error}"
            ))
        })?;

        let db = self.lock_db()?;
        let mut catalog = self.read_catalog(&db)?;

        db.put(Self::snapshot_key(&doc_id), payload)
            .map_err(|error| self.map_error("write mindb snapshot", error))?;

        if !catalog.contains(&doc_id) {
            catalog.push(doc_id);
            catalog.sort_unstable();
            db.put(
                SNAPSHOT_CATALOG_KEY.to_vec(),
                self.serialize_catalog(&catalog)?,
            )
            .map_err(|error| self.map_error("write mindb snapshot catalog", error))?;
        }

        self.sync(&db, "sync mindb snapshot store")
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let db = self.lock_db()?;
        let mut catalog = self.read_catalog(&db)?;
        let original_len = catalog.len();
        catalog.retain(|catalog_doc_id| catalog_doc_id != doc_id);

        db.delete(Self::snapshot_key(doc_id))
            .map_err(|error| self.map_error("delete mindb snapshot", error))?;

        if catalog.len() != original_len {
            db.put(
                SNAPSHOT_CATALOG_KEY.to_vec(),
                self.serialize_catalog(&catalog)?,
            )
            .map_err(|error| self.map_error("write mindb snapshot catalog", error))?;
        }

        self.sync(&db, "sync mindb snapshot store")
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let db = self.lock_db()?;
        let catalog = self.read_catalog(&db)?;
        let mut documents = Vec::new();

        for doc_id in catalog {
            match self.read_value(&db, &Self::snapshot_key(&doc_id), "read mindb snapshot") {
                Ok(Some(payload)) => match self.deserialize_snapshot(doc_id, &payload) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping corrupt mindb snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                },
                Ok(None) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing mindb snapshot referenced by catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
