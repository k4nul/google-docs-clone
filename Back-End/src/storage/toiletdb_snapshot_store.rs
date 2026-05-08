use std::{
    fs,
    panic::{AssertUnwindSafe, catch_unwind},
    path::{Path, PathBuf},
    sync::Mutex,
};

use serde_json::{Map, Value};
use toiletdb::Toiletdb;
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

pub struct ToiletdbSnapshotStore {
    path: PathBuf,
    db: Mutex<Toiletdb>,
}

impl ToiletdbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = normalize_path(path.into())?;
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let db = catch_unwind(AssertUnwindSafe(|| Toiletdb::new(&path)))
            .map_err(|_| StorageError::Io(format!("{}: invalid toiletdb JSON", path.display())))?
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            db: Mutex::new(db),
        })
    }

    fn lock_db(&self) -> Result<std::sync::MutexGuard<'_, Toiletdb>, StorageError> {
        self.db.lock().map_err(|_| {
            StorageError::Io(format!("{}: toiletdb mutex poisoned", self.path.display()))
        })
    }

    fn read_root(&self) -> Result<Map<String, Value>, StorageError> {
        if !self.path.exists() {
            return Ok(Map::new());
        }

        let mut db = self.lock_db()?;
        let payload = catch_unwind(AssertUnwindSafe(|| db.read()))
            .map_err(|_| {
                StorageError::Io(format!("{}: toiletdb read panicked", self.path.display()))
            })?
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        if payload.trim().is_empty() {
            return Ok(Map::new());
        }

        match serde_json::from_str::<Value>(&payload)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?
        {
            Value::Object(map) => Ok(map),
            _ => Err(StorageError::Io(format!(
                "{}: toiletdb root value is not an object",
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

    fn sync_file(&self) -> Result<(), StorageError> {
        fs::OpenOptions::new()
            .read(true)
            .open(&self.path)
            .and_then(|file| file.sync_all())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }
}

impl SnapshotStore for ToiletdbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let root = self.read_root()?;
        let Some(value) = root.get(&doc_id.to_string()).cloned() else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, value).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload = PersistedSnapshot::from(snapshot);
        let mut db = self.lock_db()?;

        catch_unwind(AssertUnwindSafe(|| db.write(doc_id.to_string(), payload)))
            .map_err(|_| {
                StorageError::Io(format!("{}: toiletdb write panicked", self.path.display()))
            })?
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        drop(db);

        self.sync_file()
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut db = self.lock_db()?;

        catch_unwind(AssertUnwindSafe(|| db.delete(doc_id.to_string())))
            .map_err(|_| {
                StorageError::Io(format!("{}: toiletdb delete panicked", self.path.display()))
            })?
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        drop(db);

        self.sync_file()
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let root = self.read_root()?;
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
                    "skipping corrupt toiletdb snapshot while building document catalog"
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
            "SNAPSHOT_TOILETDB_PATH cannot be empty when SNAPSHOT_STORE=toiletdb".to_owned(),
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
