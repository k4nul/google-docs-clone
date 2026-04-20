use std::{fs, path::PathBuf};

use caves::{Cave, FileCave, errors::Error as CaveError};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

pub struct CavesSnapshotStore {
    root: PathBuf,
    cave: FileCave,
}

impl CavesSnapshotStore {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let root = root.into();
        if root.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_CAVES_PATH cannot be empty when SNAPSHOT_STORE=caves".to_owned(),
            ));
        }

        ensure_snapshot_dir(&root)?;
        let cave = FileCave::new(&root)
            .map_err(|error| StorageError::Io(format!("{}: {error}", root.display())))?;

        Ok(Self { root, cave })
    }

    fn snapshot_key(doc_id: &Uuid) -> String {
        doc_id.to_string()
    }

    fn decode_snapshot(
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

    fn map_cave_error(&self, error: CaveError) -> StorageError {
        StorageError::Io(format!("{}: {error}", self.root.display()))
    }

    fn is_not_found(error: &CaveError) -> bool {
        matches!(error, CaveError::NotFound(_))
    }

    fn snapshot_path(&self, key: &str) -> PathBuf {
        self.root.join(key)
    }

    fn load_snapshot_bytes(&self, doc_id: &Uuid) -> Result<Option<Vec<u8>>, StorageError> {
        let key = Self::snapshot_key(doc_id);

        match self.cave.get(&key) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(error) if Self::is_not_found(&error) => Ok(None),
            Err(error) => Err(self.map_cave_error(error)),
        }
    }
}

impl SnapshotStore for CavesSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let Some(bytes) = self.load_snapshot_bytes(doc_id)? else {
            return Ok(None);
        };

        self.decode_snapshot(*doc_id, &bytes).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let key = Self::snapshot_key(&doc_id);
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize caves snapshot `{doc_id}`: {error}"
            ))
        })?;

        self.cave
            .set(&key, &bytes)
            .map(|_| ())
            .map_err(|error| self.map_cave_error(error))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let key = Self::snapshot_key(doc_id);

        match self.cave.delete(&key) {
            Ok(_) => Ok(()),
            Err(error) if Self::is_not_found(&error) => Ok(()),
            Err(error) => Err(self.map_cave_error(error)),
        }
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut documents = Vec::new();

        for entry in fs::read_dir(&self.root)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.root.display())))?
        {
            let entry = entry.map_err(|error| StorageError::Io(error.to_string()))?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
                continue;
            };
            let Ok(doc_id) = Uuid::parse_str(file_name) else {
                continue;
            };

            match self.cave.get(file_name) {
                Ok(bytes) => match self.decode_snapshot(doc_id, &bytes) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.snapshot_path(file_name).display(),
                        "skipping corrupt caves snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                },
                Err(error) if Self::is_not_found(&error) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.snapshot_path(file_name).display(),
                    "skipping missing caves snapshot while building document catalog"
                ),
                Err(error) => return Err(self.map_cave_error(error)),
            }
        }

        Ok(documents)
    }
}
