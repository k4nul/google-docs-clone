use std::{fs, path::PathBuf};

use redb::{Database, ReadableTable, TableDefinition};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOTS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("snapshots");

pub struct RedbSnapshotStore {
    path: PathBuf,
    database: Database,
}

impl RedbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_REDB_PATH cannot be empty when SNAPSHOT_STORE=redb".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let database = Database::create(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;
        let store = Self { path, database };
        store.initialize_schema()?;
        Ok(store)
    }

    fn initialize_schema(&self) -> Result<(), StorageError> {
        let write_txn = self
            .database
            .begin_write()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        write_txn
            .open_table(SNAPSHOTS_TABLE)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        write_txn
            .commit()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        Ok(())
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

impl SnapshotStore for RedbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let doc_id_key = doc_id.to_string();
        let read_txn = self
            .database
            .begin_read()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        let table = read_txn
            .open_table(SNAPSHOTS_TABLE)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        let Some(value) = table
            .get(doc_id_key.as_str())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?
        else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, value.value()).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let doc_id_key = doc_id.to_string();
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize redb snapshot `{doc_id}`: {error}"
            ))
        })?;

        let write_txn = self
            .database
            .begin_write()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        {
            let mut table = write_txn
                .open_table(SNAPSHOTS_TABLE)
                .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
            table
                .insert(doc_id_key.as_str(), bytes.as_slice())
                .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        }
        write_txn
            .commit()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        Ok(())
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let doc_id_key = doc_id.to_string();
        let write_txn = self
            .database
            .begin_write()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        {
            let mut table = write_txn
                .open_table(SNAPSHOTS_TABLE)
                .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
            table
                .remove(doc_id_key.as_str())
                .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        }
        write_txn
            .commit()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        Ok(())
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let read_txn = self
            .database
            .begin_read()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        let table = read_txn
            .open_table(SNAPSHOTS_TABLE)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        let mut documents = Vec::new();

        for entry in table
            .iter()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?
        {
            let (key, value) = entry
                .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
            let Ok(doc_id) = Uuid::parse_str(key.value()) else {
                continue;
            };

            match self.deserialize_snapshot(doc_id, value.value()) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt redb snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
