use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
};

use jasonisnthappy::Database;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const COLLECTION_NAME: &str = "snapshots";
const SNAPSHOT_FIELD: &str = "snapshot";

pub struct JasonisnthappySnapshotStore {
    path: PathBuf,
    db: Mutex<Database>,
}

impl JasonisnthappySnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_JASONISNTHAPPY_PATH cannot be empty when SNAPSHOT_STORE=jasonisnthappy"
                    .to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let db = Database::open(path_to_str(&path)?)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            db: Mutex::new(db),
        })
    }

    fn lock_db(&self) -> Result<MutexGuard<'_, Database>, StorageError> {
        self.db.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: jasonisnthappy mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn map_error(&self, operation: &str, error: impl std::fmt::Display) -> StorageError {
        StorageError::Io(format!(
            "{}: failed to {operation}: {error}",
            self.path.display()
        ))
    }

    fn doc_key(doc_id: Uuid) -> String {
        doc_id.to_string()
    }

    fn serialize_snapshot(snapshot: DocumentSnapshot) -> Result<Value, StorageError> {
        let persisted = PersistedSnapshot::from(snapshot);
        serde_json::to_value(persisted)
            .map(|snapshot| json!({ SNAPSHOT_FIELD: snapshot }))
            .map_err(|error| StorageError::Io(format!("failed to serialize snapshot: {error}")))
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        value: Value,
    ) -> Result<DocumentSnapshot, StorageError> {
        let Some(snapshot_value) = value.get(SNAPSHOT_FIELD).cloned() else {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        };
        let snapshot = serde_json::from_value::<PersistedSnapshot>(snapshot_value)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot: DocumentSnapshot = snapshot.into();

        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }
}

impl SnapshotStore for JasonisnthappySnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let db = self.lock_db()?;
        let collection = db.collection(COLLECTION_NAME);
        let value = match collection.find_by_id(&Self::doc_key(*doc_id)) {
            Ok(value) => value,
            Err(_) => return Ok(None),
        };

        self.deserialize_snapshot(*doc_id, value).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let value = Self::serialize_snapshot(snapshot)?;
        let db = self.lock_db()?;
        db.collection(COLLECTION_NAME)
            .upsert_by_id(&Self::doc_key(doc_id), value)
            .map(|_| ())
            .map_err(|error| self.map_error("write jasonisnthappy snapshot", error))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let db = self.lock_db()?;
        let collection = db.collection(COLLECTION_NAME);
        if collection.find_by_id(&Self::doc_key(*doc_id)).is_err() {
            return Ok(());
        }

        collection
            .delete_by_id(&Self::doc_key(*doc_id))
            .map_err(|error| self.map_error("delete jasonisnthappy snapshot", error))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let db = self.lock_db()?;
        let collection = db.collection(COLLECTION_NAME);
        let values = match collection.find_all() {
            Ok(values) => values,
            Err(_) => return Ok(Vec::new()),
        };

        let mut documents = Vec::new();
        for value in values {
            let Some(doc_id) = value
                .get("_id")
                .and_then(Value::as_str)
                .and_then(|value| Uuid::parse_str(value).ok())
            else {
                continue;
            };

            match self.deserialize_snapshot(doc_id, value) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt jasonisnthappy snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        documents.sort_unstable_by_key(|document| document.id);
        Ok(documents)
    }
}

fn path_to_str(path: &Path) -> Result<&str, StorageError> {
    path.to_str().ok_or_else(|| {
        StorageError::Config(format!(
            "SNAPSHOT_JASONISNTHAPPY_PATH must be valid UTF-8, received `{}`",
            path.display()
        ))
    })
}
