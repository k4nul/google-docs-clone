use std::{fs, path::PathBuf};

use okofdb::okof;
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

pub struct OkofdbSnapshotStore {
    root: PathBuf,
}

impl OkofdbSnapshotStore {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let root = root.into();
        if root.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_OKOFDB_PATH cannot be empty when SNAPSHOT_STORE=okofdb".to_owned(),
            ));
        }

        fs::create_dir_all(&root)
            .map_err(|error| StorageError::Io(format!("{}: {error}", root.display())))?;

        Ok(Self { root })
    }

    fn snapshot_key(doc_id: &Uuid) -> String {
        format!("doc_{}", doc_id.simple())
    }

    fn parse_doc_id(key: &str) -> Option<Uuid> {
        let hex = key.strip_prefix("doc_")?;
        Uuid::parse_str(hex).ok()
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

    fn map_okof_error(&self, error: okof::Error) -> StorageError {
        StorageError::Io(format!("{}: {error:?}", self.root.display()))
    }
}

impl SnapshotStore for OkofdbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let key = Self::snapshot_key(doc_id);
        match okof::read(&self.root, &key) {
            Ok(payload) => self.deserialize_snapshot(*doc_id, &payload).map(Some),
            Err(okof::Error::NotFound) => Ok(None),
            Err(error) => Err(self.map_okof_error(error)),
        }
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let key = Self::snapshot_key(&doc_id);
        let payload = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize okofdb snapshot `{doc_id}`: {error}"
            ))
        })?;

        okof::write(&self.root, &key, &payload).map_err(|error| self.map_okof_error(error))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let key = Self::snapshot_key(doc_id);
        match okof::delete(&self.root, &key) {
            Ok(()) | Err(okof::Error::NotFound) => Ok(()),
            Err(error) => Err(self.map_okof_error(error)),
        }
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut entries = fs::read_dir(&self.root)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.root.display())))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.root.display())))?;
        entries.sort_by_key(|entry| entry.file_name());

        let mut documents = Vec::new();
        for entry in entries {
            let Some(key) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            let Some(doc_id) = Self::parse_doc_id(&key) else {
                continue;
            };

            let payload =
                okof::read(&self.root, &key).map_err(|error| self.map_okof_error(error))?;
            match self.deserialize_snapshot(doc_id, &payload) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.root.display(),
                    "skipping corrupt okofdb snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
