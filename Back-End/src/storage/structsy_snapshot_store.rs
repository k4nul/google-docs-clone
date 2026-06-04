use std::{fs, path::PathBuf};

use chrono::{TimeZone, Utc};
use structsy::{Ref, Structsy, StructsyTx, derive::Persistent};
use tracing::warn;
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, SnapshotStore, StorageError},
};

#[derive(Persistent, Clone)]
struct StructsySnapshotRecord {
    doc_id: String,
    title: String,
    created_at_seconds: i64,
    created_at_subsec_nanos: u32,
    updated_at_seconds: i64,
    updated_at_subsec_nanos: u32,
    access_token: String,
    hide_preview: bool,
    update: Vec<u8>,
}

pub struct StructsySnapshotStore {
    path: PathBuf,
    database: Structsy,
}

impl StructsySnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_STRUCTSY_PATH cannot be empty when SNAPSHOT_STORE=structsy".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let database = Structsy::open(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;
        database
            .define::<StructsySnapshotRecord>()
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self { path, database })
    }

    fn find_snapshot_record(
        &self,
        doc_id: &Uuid,
    ) -> Result<Option<(Ref<StructsySnapshotRecord>, StructsySnapshotRecord)>, StorageError> {
        let scan = self
            .database
            .scan::<StructsySnapshotRecord>()
            .map_err(|error| self.map_error(error))?;

        for (record_ref, record) in scan {
            if record.doc_id == doc_id.to_string() {
                return Ok(Some((record_ref, record)));
            }
        }

        Ok(None)
    }

    fn map_error(&self, error: impl std::fmt::Display) -> StorageError {
        StorageError::Io(format!("{}: {error}", self.path.display()))
    }

    fn record_from_snapshot(snapshot: DocumentSnapshot) -> StructsySnapshotRecord {
        let document = snapshot.document;
        let access_token = document.access_token().to_owned();
        StructsySnapshotRecord {
            doc_id: document.id.to_string(),
            title: document.title,
            created_at_seconds: document.created_at.timestamp(),
            created_at_subsec_nanos: document.created_at.timestamp_subsec_nanos(),
            updated_at_seconds: document.updated_at.timestamp(),
            updated_at_subsec_nanos: document.updated_at.timestamp_subsec_nanos(),
            access_token,
            hide_preview: document.hide_preview,
            update: snapshot.update,
        }
    }

    fn snapshot_from_record(
        &self,
        expected_doc_id: Uuid,
        record: StructsySnapshotRecord,
    ) -> Result<DocumentSnapshot, StorageError> {
        let record_doc_id = Uuid::parse_str(&record.doc_id)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        if record_doc_id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        let created_at = Utc
            .timestamp_opt(record.created_at_seconds, record.created_at_subsec_nanos)
            .single()
            .ok_or(StorageError::CorruptSnapshot(expected_doc_id))?;
        let updated_at = Utc
            .timestamp_opt(record.updated_at_seconds, record.updated_at_subsec_nanos)
            .single()
            .ok_or(StorageError::CorruptSnapshot(expected_doc_id))?;

        Ok(DocumentSnapshot::new(
            Document::from_parts(
                record_doc_id,
                record.title,
                created_at,
                updated_at,
                record.access_token,
                record.hide_preview,
            ),
            record.update,
        ))
    }
}

impl SnapshotStore for StructsySnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let Some((_, record)) = self.find_snapshot_record(doc_id)? else {
            return Ok(None);
        };

        self.snapshot_from_record(*doc_id, record).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let record = Self::record_from_snapshot(snapshot);
        let existing_record = self.find_snapshot_record(&doc_id)?;
        let mut transaction = self
            .database
            .begin()
            .map_err(|error| self.map_error(error))?;

        if let Some((record_ref, _)) = existing_record {
            transaction
                .update(&record_ref, &record)
                .map_err(|error| self.map_error(error))?;
        } else {
            transaction
                .insert(&record)
                .map_err(|error| self.map_error(error))?;
        }

        transaction
            .commit()
            .map_err(|error| self.map_error(error))?;
        Ok(())
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let Some((record_ref, _)) = self.find_snapshot_record(doc_id)? else {
            return Ok(());
        };
        let mut transaction = self
            .database
            .begin()
            .map_err(|error| self.map_error(error))?;
        transaction
            .delete(&record_ref)
            .map_err(|error| self.map_error(error))?;
        transaction
            .commit()
            .map_err(|error| self.map_error(error))?;
        Ok(())
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let scan = self
            .database
            .scan::<StructsySnapshotRecord>()
            .map_err(|error| self.map_error(error))?;
        let mut documents = Vec::new();

        for (_, record) in scan {
            let Ok(doc_id) = Uuid::parse_str(&record.doc_id) else {
                warn!(
                    doc_id = %record.doc_id,
                    path = %self.path.display(),
                    "skipping corrupt structsy snapshot identifier while building document catalog"
                );
                continue;
            };

            match self.snapshot_from_record(doc_id, record) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt structsy snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        documents.sort_by_key(|document| (document.created_at, document.id));
        Ok(documents)
    }
}
