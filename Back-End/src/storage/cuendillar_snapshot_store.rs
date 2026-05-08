use std::{fs, path::PathBuf, sync::Arc};

use cuendillar::{
    Database, DbConfig, EngineError, OwnedEntry,
    config::{version_manager_config::VersionMangerSyncVariant, wal_config::WALSyncVariant},
};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const MAX_SNAPSHOT_PAYLOAD_BYTES: u64 = 16 * 1024 * 1024;
const WAL_FILE_SIZE_BYTES: u64 = MAX_SNAPSHOT_PAYLOAD_BYTES * 16;

pub struct CuendillarSnapshotStore {
    path: PathBuf,
    database: Database,
}

impl CuendillarSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_CUENDILLAR_PATH cannot be empty when SNAPSHOT_STORE=cuendillar"
                    .to_owned(),
            ));
        }

        fs::create_dir_all(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        let database = Database::new(Self::database_config(&path)).map_err(|error| {
            Self::map_engine_error(&path, "open cuendillar snapshot store", error)
        })?;

        Ok(Self { path, database })
    }

    fn database_config(path: &std::path::Path) -> Arc<DbConfig> {
        let sstable_root = path.join("sstable");
        let mut config = DbConfig::get_dynamic_defaults(path, &sstable_root);
        // Cuendillar's dynamic defaults are test-oriented; raise the WAL payload ceiling
        // so full-state Yrs snapshots fit in a single durable write.
        config.wal.wal_max_payload_len_in_bytes = MAX_SNAPSHOT_PAYLOAD_BYTES;
        config.wal.wal_file_size_in_bytes = WAL_FILE_SIZE_BYTES;
        config.wal.wal_sync_variant = WALSyncVariant::Always;
        config.version_manager.version_manager_sync_mode = VersionMangerSyncVariant::Always;
        Arc::new(config)
    }

    fn map_engine_error(
        path: &std::path::Path,
        operation: &str,
        error: EngineError,
    ) -> StorageError {
        StorageError::Io(format!(
            "{}: failed to {operation}: {error:?}",
            path.display()
        ))
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

impl SnapshotStore for CuendillarSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let key = doc_id.to_string();
        let entry = self.database.get(key.as_bytes()).map_err(|error| {
            Self::map_engine_error(&self.path, "read cuendillar snapshot", error)
        })?;

        match entry {
            Some(OwnedEntry::Row { value, .. }) => {
                self.deserialize_snapshot(*doc_id, &value).map(Some)
            }
            Some(OwnedEntry::Tombstone { .. }) | None => Ok(None),
        }
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let key = doc_id.to_string();
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize cuendillar snapshot `{doc_id}`: {error}"
            ))
        })?;

        self.database
            .put(key.as_bytes(), &bytes)
            .map(|_| ())
            .map_err(|error| Self::map_engine_error(&self.path, "write cuendillar snapshot", error))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let key = doc_id.to_string();
        self.database
            .delete(key.as_bytes())
            .map(|_| ())
            .map_err(|error| {
                Self::map_engine_error(&self.path, "delete cuendillar snapshot", error)
            })
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut documents = Vec::new();
        let iterator = self.database.iter(None, None).map_err(|error| {
            Self::map_engine_error(&self.path, "iterate cuendillar snapshot catalog", error)
        })?;

        for entry in iterator {
            let OwnedEntry::Row { key, value, .. } = entry else {
                continue;
            };
            let Ok(doc_id_key) = std::str::from_utf8(&key) else {
                continue;
            };
            let Ok(doc_id) = Uuid::parse_str(doc_id_key) else {
                continue;
            };

            match self.deserialize_snapshot(doc_id, &value) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt cuendillar snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
