use std::{fs, path::PathBuf};

use fjall::{Database, Keyspace, KeyspaceCreateOptions, PersistMode};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOTS_KEYSPACE: &str = "snapshots";

pub struct FjallSnapshotStore {
    path: PathBuf,
    database: Database,
    snapshots: Keyspace,
}

impl FjallSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_FJALL_PATH cannot be empty when SNAPSHOT_STORE=fjall".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let database = Database::builder(&path)
            .open()
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;
        let snapshots = database
            .keyspace(SNAPSHOTS_KEYSPACE, KeyspaceCreateOptions::default)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            database,
            snapshots,
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

    fn persist(&self) -> Result<(), StorageError> {
        self.database
            .persist(PersistMode::SyncAll)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }
}

impl SnapshotStore for FjallSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let doc_id_key = doc_id.to_string();
        let Some(value) = self
            .snapshots
            .get(doc_id_key.as_bytes())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?
        else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, value.as_ref()).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let doc_id_key = doc_id.to_string();
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize fjall snapshot `{doc_id}`: {error}"
            ))
        })?;

        self.snapshots
            .insert(doc_id_key.as_bytes(), bytes)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        self.persist()
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let doc_id_key = doc_id.to_string();
        self.snapshots
            .remove(doc_id_key.as_bytes())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        self.persist()
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut documents = Vec::new();

        for entry in self.snapshots.iter() {
            let (key, value) = entry
                .into_inner()
                .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
            let Ok(doc_id_key) = std::str::from_utf8(key.as_ref()) else {
                continue;
            };
            let Ok(doc_id) = Uuid::parse_str(doc_id_key) else {
                continue;
            };

            match self.deserialize_snapshot(doc_id, value.as_ref()) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt fjall snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
