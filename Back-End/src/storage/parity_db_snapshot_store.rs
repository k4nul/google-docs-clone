use std::{fs, path::PathBuf};

use parity_db::{ColumnOptions, Db, Options};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOT_COLUMN: u8 = 0;

pub struct ParityDbSnapshotStore {
    path: PathBuf,
    database: Db,
}

impl ParityDbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_PARITY_DB_PATH cannot be empty when SNAPSHOT_STORE=parity_db".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let mut options = Options::with_columns(&path, 1);
        options.columns[SNAPSHOT_COLUMN as usize] = ColumnOptions {
            btree_index: true,
            ..Default::default()
        };

        let database = Db::open_or_create(&options)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self { path, database })
    }

    fn snapshot_key(doc_id: &Uuid) -> Vec<u8> {
        doc_id.to_string().into_bytes()
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

impl SnapshotStore for ParityDbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let Some(value) = self
            .database
            .get(SNAPSHOT_COLUMN, Self::snapshot_key(doc_id).as_slice())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?
        else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &value).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize parity_db snapshot `{doc_id}`: {error}"
            ))
        })?;

        self.database
            .commit([(SNAPSHOT_COLUMN, Self::snapshot_key(&doc_id), Some(bytes))])
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        self.database
            .commit([(SNAPSHOT_COLUMN, Self::snapshot_key(doc_id), None)])
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut iter = self
            .database
            .iter(SNAPSHOT_COLUMN)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        let mut documents = Vec::new();

        while let Some((key, value)) = iter
            .next()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?
        {
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
                    "skipping corrupt parity_db snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
