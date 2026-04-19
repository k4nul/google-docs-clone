use std::{fs, path::PathBuf};

use persy::{Config, Persy, PersyId, ValueMode};
use tracing::warn;
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOTS_SEGMENT: &str = "snapshots";
const SNAPSHOTS_INDEX: &str = "snapshots_by_doc_id";

pub struct PersySnapshotStore {
    path: PathBuf,
    database: Persy,
}

impl PersySnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_PERSY_PATH cannot be empty when SNAPSHOT_STORE=persy".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let database = Persy::open_or_create_with(&path, Config::new(), |persy| {
            let mut tx = persy.begin()?;
            tx.create_segment(SNAPSHOTS_SEGMENT)?;
            tx.create_index::<String, PersyId>(SNAPSHOTS_INDEX, ValueMode::Replace)?;
            tx.prepare()?.commit()?;
            Ok(())
        })
        .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self { path, database })
    }

    fn lookup_record_id(&self, doc_id: &Uuid) -> Result<Option<PersyId>, StorageError> {
        let mut values = self
            .database
            .get::<String, PersyId>(SNAPSHOTS_INDEX, &doc_id.to_string())
            .map_err(|error| self.map_error(error))?;
        Ok(values.next())
    }

    fn read_snapshot_by_record_id(
        &self,
        expected_doc_id: Uuid,
        record_id: &PersyId,
    ) -> Result<DocumentSnapshot, StorageError> {
        let Some(bytes) = self
            .database
            .read(SNAPSHOTS_SEGMENT, record_id)
            .map_err(|error| self.map_error(error))?
        else {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        };

        let snapshot = serde_json::from_slice::<PersistedSnapshot>(&bytes)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot: DocumentSnapshot = snapshot.into();

        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }

    fn map_error(&self, error: impl std::fmt::Display) -> StorageError {
        StorageError::Io(format!("{}: {error}", self.path.display()))
    }
}

impl SnapshotStore for PersySnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let Some(record_id) = self.lookup_record_id(doc_id)? else {
            return Ok(None);
        };

        self.read_snapshot_by_record_id(*doc_id, &record_id)
            .map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let doc_id_key = doc_id.to_string();
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize persy snapshot `{doc_id}`: {error}"
            ))
        })?;

        let existing_record_id = self.lookup_record_id(&doc_id)?;
        let mut tx = self
            .database
            .begin()
            .map_err(|error| self.map_error(error))?;

        if let Some(record_id) = existing_record_id {
            tx.update(SNAPSHOTS_SEGMENT, &record_id, bytes.as_slice())
                .map_err(|error| self.map_error(error))?;
        } else {
            let record_id = tx
                .insert(SNAPSHOTS_SEGMENT, bytes.as_slice())
                .map_err(|error| self.map_error(error))?;
            tx.put::<String, PersyId>(SNAPSHOTS_INDEX, doc_id_key, record_id)
                .map_err(|error| self.map_error(error))?;
        }

        tx.prepare()
            .map_err(|error| self.map_error(error))?
            .commit()
            .map_err(|error| self.map_error(error))?;
        Ok(())
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let Some(record_id) = self.lookup_record_id(doc_id)? else {
            return Ok(());
        };

        let mut tx = self
            .database
            .begin()
            .map_err(|error| self.map_error(error))?;
        tx.delete(SNAPSHOTS_SEGMENT, &record_id)
            .map_err(|error| self.map_error(error))?;
        tx.remove::<String, PersyId>(SNAPSHOTS_INDEX, doc_id.to_string(), Some(record_id))
            .map_err(|error| self.map_error(error))?;
        tx.prepare()
            .map_err(|error| self.map_error(error))?
            .commit()
            .map_err(|error| self.map_error(error))?;
        Ok(())
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut documents = Vec::new();
        let entries = self
            .database
            .range::<String, PersyId, _>(SNAPSHOTS_INDEX, ..)
            .map_err(|error| self.map_error(error))?;

        for (doc_id_key, mut record_ids) in entries {
            let Ok(doc_id) = Uuid::parse_str(&doc_id_key) else {
                warn!(
                    key = %doc_id_key,
                    path = %self.path.display(),
                    "skipping corrupt persy snapshot index entry while building document catalog"
                );
                continue;
            };

            let Some(record_id) = record_ids.next() else {
                warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping persy snapshot index entry without a record while building document catalog"
                );
                continue;
            };

            match self.read_snapshot_by_record_id(doc_id, &record_id) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt persy snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        documents.sort_by_key(|document| (document.created_at, document.id));
        Ok(documents)
    }
}
