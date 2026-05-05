use std::{fs::File, path::PathBuf};

use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

pub struct CacacheSnapshotStore {
    path: PathBuf,
}

impl CacacheSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_CACACHE_PATH cannot be empty when SNAPSHOT_STORE=cacache".to_owned(),
            ));
        }

        ensure_snapshot_dir(&path)?;

        Ok(Self { path })
    }

    fn key(doc_id: &Uuid) -> String {
        format!("snapshot:{doc_id}")
    }

    fn doc_id_from_key(key: &str) -> Option<Uuid> {
        key.strip_prefix("snapshot:")
            .and_then(|doc_id| Uuid::parse_str(doc_id).ok())
    }

    fn map_error(&self, error: cacache::Error) -> StorageError {
        StorageError::Io(format!("{}: {error}", self.path.display()))
    }

    fn is_missing_index_error(error: &cacache::Error) -> bool {
        matches!(
            error,
            cacache::Error::IoError(io_error, _) if io_error.kind() == std::io::ErrorKind::NotFound
        )
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        payload: &[u8],
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot = serde_json::from_slice::<PersistedSnapshot>(payload).map_err(|error| {
            StorageError::Io(format!(
                "{}: failed to parse cacache snapshot payload: {error}",
                self.path.display()
            ))
        })?;
        let snapshot: DocumentSnapshot = snapshot.into();
        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }

    fn sync_cache_dir(&self) -> Result<(), StorageError> {
        File::open(&self.path)
            .and_then(|file| file.sync_all())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }
}

impl SnapshotStore for CacacheSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let key = Self::key(doc_id);
        let payload = match cacache::read_sync(&self.path, &key) {
            Ok(payload) => payload,
            Err(cacache::Error::EntryNotFound(_, _)) => return Ok(None),
            Err(error) => return Err(self.map_error(error)),
        };

        self.deserialize_snapshot(*doc_id, &payload).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "{}: failed to serialize cacache snapshot payload: {error}",
                self.path.display()
            ))
        })?;

        cacache::write_sync(&self.path, Self::key(&doc_id), payload)
            .map_err(|error| self.map_error(error))?;
        self.sync_cache_dir()
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        match cacache::remove_sync(&self.path, Self::key(doc_id)) {
            Ok(()) | Err(cacache::Error::EntryNotFound(_, _)) => self.sync_cache_dir(),
            Err(error) => Err(self.map_error(error)),
        }
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut documents = Vec::new();

        for entry in cacache::list_sync(&self.path) {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) if Self::is_missing_index_error(&error) => return Ok(documents),
                Err(error) => return Err(self.map_error(error)),
            };
            let Some(doc_id) = Self::doc_id_from_key(&entry.key) else {
                continue;
            };

            match self.load_snapshot(&doc_id) {
                Ok(Some(snapshot)) => documents.push(snapshot.document),
                Ok(None) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing cacache snapshot while building document catalog"
                ),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt cacache snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        documents.sort_by_key(|document| document.created_at);
        Ok(documents)
    }
}
