use std::{
    hash::{Hash, Hasher},
    path::PathBuf,
    sync::Mutex,
};

use serde::{Deserialize, Serialize};
use tinydb::{Database, error::DatabaseError};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

pub struct TinydbSnapshotStore {
    path: PathBuf,
    db: Mutex<Database<TinydbSnapshotRecord>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TinydbSnapshotRecord {
    doc_id: Uuid,
    payload: Vec<u8>,
}

impl PartialEq for TinydbSnapshotRecord {
    fn eq(&self, other: &Self) -> bool {
        self.doc_id == other.doc_id
    }
}

impl Eq for TinydbSnapshotRecord {}

impl Hash for TinydbSnapshotRecord {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.doc_id.hash(state);
    }
}

impl TinydbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_TINYDB_PATH cannot be empty when SNAPSHOT_STORE=tinydb".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let db = match Database::auto_from(path.clone(), false) {
            Ok(db) => db,
            Err(error) => return Err(Self::map_tinydb_error(&path, error)),
        };

        Ok(Self {
            path,
            db: Mutex::new(db),
        })
    }

    fn find_record(
        db: &Database<TinydbSnapshotRecord>,
        doc_id: Uuid,
    ) -> Option<TinydbSnapshotRecord> {
        db.query_item(|record| &record.doc_id, doc_id).ok().cloned()
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

    fn with_db<T>(
        &self,
        operation: &'static str,
        action: impl FnOnce(&mut Database<TinydbSnapshotRecord>) -> Result<T, StorageError>,
    ) -> Result<T, StorageError> {
        let mut db = self.db.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: tinydb mutex was poisoned during {operation}",
                self.path.display()
            ))
        })?;

        action(&mut db)
    }

    fn dump_db(&self, db: &Database<TinydbSnapshotRecord>) -> Result<(), StorageError> {
        db.dump_db()
            .map_err(|error| Self::map_tinydb_error(&self.path, error))
    }

    fn map_tinydb_error(path: &std::path::Path, error: DatabaseError) -> StorageError {
        StorageError::Io(format!("{}: {error:?}", path.display()))
    }
}

impl SnapshotStore for TinydbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        self.with_db("load", |db| {
            let Some(record) = Self::find_record(db, *doc_id) else {
                return Ok(None);
            };

            self.deserialize_snapshot(*doc_id, &record.payload)
                .map(Some)
        })
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize tinydb snapshot `{doc_id}`: {error}"
            ))
        })?;

        self.with_db("save", |db| {
            if let Some(existing) = Self::find_record(db, doc_id) {
                db.remove_item(&existing)
                    .map_err(|error| Self::map_tinydb_error(&self.path, error))?;
            }

            db.add_item(TinydbSnapshotRecord { doc_id, payload })
                .map_err(|error| Self::map_tinydb_error(&self.path, error))?;
            self.dump_db(db)
        })
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        self.with_db("delete", |db| {
            if let Some(existing) = Self::find_record(db, *doc_id) {
                db.remove_item(&existing)
                    .map_err(|error| Self::map_tinydb_error(&self.path, error))?;
                self.dump_db(db)?;
            }

            Ok(())
        })
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        self.with_db("list", |db| {
            let mut records = db.items.iter().cloned().collect::<Vec<_>>();
            records.sort_by_key(|record| record.doc_id);

            let mut documents = Vec::new();
            for record in records {
                match self.deserialize_snapshot(record.doc_id, &record.payload) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping corrupt tinydb snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                }
            }

            Ok(documents)
        })
    }
}
