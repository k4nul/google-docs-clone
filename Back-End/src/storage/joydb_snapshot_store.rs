use std::{fs, path::PathBuf};

use joydb::{Joydb, Model, adapters::JsonAdapter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
struct JoydbSnapshotRecord {
    id: String,
    snapshot: PersistedSnapshot,
}

joydb::state! {
    JoydbSnapshotState,
    models: [JoydbSnapshotRecord],
}

type SnapshotDatabase = Joydb<JoydbSnapshotState, JsonAdapter>;

pub struct JoydbSnapshotStore {
    path: PathBuf,
    database: SnapshotDatabase,
}

impl JoydbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_JOYDB_PATH cannot be empty when SNAPSHOT_STORE=joydb".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let database = SnapshotDatabase::open(&path).map_err(|error| {
            StorageError::Io(format!(
                "{}: failed to open joydb store: {error}",
                path.display()
            ))
        })?;

        Ok(Self { path, database })
    }

    fn snapshot_id(doc_id: &Uuid) -> String {
        doc_id.to_string()
    }

    fn decode_record(
        expected_doc_id: Uuid,
        record: JoydbSnapshotRecord,
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot: DocumentSnapshot = record.snapshot.into();

        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }

    fn flush(&self) -> Result<(), StorageError> {
        self.database.flush().map_err(|error| {
            StorageError::Io(format!(
                "{}: failed to flush joydb store: {error}",
                self.path.display()
            ))
        })
    }
}

impl SnapshotStore for JoydbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let id = Self::snapshot_id(doc_id);
        let Some(record) = self
            .database
            .get::<JoydbSnapshotRecord>(&id)
            .map_err(|error| {
                StorageError::Io(format!(
                    "{}: failed to load joydb snapshot `{doc_id}`: {error}",
                    self.path.display()
                ))
            })?
        else {
            return Ok(None);
        };

        Self::decode_record(*doc_id, record).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let record = JoydbSnapshotRecord {
            id: Self::snapshot_id(&doc_id),
            snapshot: PersistedSnapshot::from(snapshot),
        };

        self.database.upsert(&record).map_err(|error| {
            StorageError::Io(format!(
                "{}: failed to save joydb snapshot `{doc_id}`: {error}",
                self.path.display()
            ))
        })?;
        self.flush()
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let id = Self::snapshot_id(doc_id);
        self.database
            .delete::<JoydbSnapshotRecord>(&id)
            .map_err(|error| {
                StorageError::Io(format!(
                    "{}: failed to delete joydb snapshot `{doc_id}`: {error}",
                    self.path.display()
                ))
            })?;
        self.flush()
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let records = self
            .database
            .get_all::<JoydbSnapshotRecord>()
            .map_err(|error| {
                StorageError::Io(format!(
                    "{}: failed to list joydb snapshots: {error}",
                    self.path.display()
                ))
            })?;
        let mut documents = Vec::new();

        for record in records {
            let Ok(doc_id) = Uuid::parse_str(&record.id) else {
                tracing::warn!(
                    record_id = %record.id,
                    path = %self.path.display(),
                    "skipping joydb snapshot with invalid document id"
                );
                continue;
            };

            match Self::decode_record(doc_id, record) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt joydb snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
