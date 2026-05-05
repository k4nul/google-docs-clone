use std::{
    fs,
    path::{Path, PathBuf},
};

use json_mutex_db::{DbError, JsonMutexDB};
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

pub struct JsonMutexDbSnapshotStore {
    path: PathBuf,
    db: JsonMutexDB,
}

impl JsonMutexDbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = normalize_path(path.into())?;
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let path_string = path.to_string_lossy().into_owned();
        let db = JsonMutexDB::new(&path_string, false, false, false)
            .map_err(|error| map_json_mutex_db_error(&path, error))?;

        Ok(Self { path, db })
    }

    fn load_root(&self) -> Result<Map<String, Value>, StorageError> {
        match self
            .db
            .get()
            .map_err(|error| map_json_mutex_db_error(&self.path, error))?
        {
            Value::Object(map) => Ok(map),
            _ => Err(StorageError::Io(format!(
                "{}: json_mutex_db root value is not an object",
                self.path.display()
            ))),
        }
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        value: Value,
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot: PersistedSnapshot = serde_json::from_value(value)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot: DocumentSnapshot = snapshot.into();
        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }

    fn save(&self) -> Result<(), StorageError> {
        self.db
            .save_sync()
            .map_err(|error| map_json_mutex_db_error(&self.path, error))
    }
}

impl SnapshotStore for JsonMutexDbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let root = self.load_root()?;
        let Some(value) = root.get(&doc_id.to_string()).cloned() else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, value).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let value = serde_json::to_value(PersistedSnapshot::from(snapshot))
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        self.db
            .update(move |data| {
                if !data.is_object() {
                    *data = Value::Object(Map::new());
                }
                data.as_object_mut()
                    .expect("json_mutex_db snapshot root should be an object")
                    .insert(doc_id.to_string(), value);
            })
            .map_err(|error| map_json_mutex_db_error(&self.path, error))?;
        self.save()
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let doc_id = doc_id.to_string();
        self.db
            .update(move |data| {
                if let Some(map) = data.as_object_mut() {
                    map.remove(&doc_id);
                }
            })
            .map_err(|error| map_json_mutex_db_error(&self.path, error))?;
        self.save()
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let root = self.load_root()?;
        let mut documents = Vec::new();

        for (doc_id_key, value) in root {
            let Ok(doc_id) = Uuid::parse_str(&doc_id_key) else {
                continue;
            };

            match self.deserialize_snapshot(doc_id, value) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt json_mutex_db snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}

fn normalize_path(path: PathBuf) -> Result<PathBuf, StorageError> {
    if path.as_os_str().is_empty() {
        return Err(StorageError::Config(
            "SNAPSHOT_JSON_MUTEX_DB_PATH cannot be empty when SNAPSHOT_STORE=json_mutex_db"
                .to_owned(),
        ));
    }

    if has_parent(&path) {
        Ok(path)
    } else {
        Ok(PathBuf::from(".").join(path))
    }
}

fn has_parent(path: &Path) -> bool {
    path.parent()
        .is_some_and(|parent| !parent.as_os_str().is_empty())
}

fn map_json_mutex_db_error(path: &Path, error: DbError) -> StorageError {
    StorageError::Io(format!("{}: {error:?}", path.display()))
}
