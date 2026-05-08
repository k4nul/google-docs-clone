use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use agdb::{Db, DbElement, DbKeyValue, DbValue, QueryBuilder};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOT_ALIAS_PREFIX: &str = "snapshot:";
const SNAPSHOT_PAYLOAD_KEY: &str = "payload";

pub struct AgdbSnapshotStore {
    path: PathBuf,
    db: Mutex<Db>,
}

impl AgdbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_AGDB_PATH cannot be empty when SNAPSHOT_STORE=agdb".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let db = Db::new(&path.to_string_lossy())
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            db: Mutex::new(db),
        })
    }

    fn lock_db(&self) -> Result<MutexGuard<'_, Db>, StorageError> {
        self.db.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: agdb snapshot store mutex was poisoned",
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

    fn alias(doc_id: &Uuid) -> String {
        format!("{SNAPSHOT_ALIAS_PREFIX}{doc_id}")
    }

    fn payload_from_element(
        &self,
        expected_doc_id: Uuid,
        element: DbElement,
    ) -> Result<DocumentSnapshot, StorageError> {
        let payload = element
            .values
            .iter()
            .find_map(|kv| match (&kv.key, &kv.value) {
                (DbValue::String(key), DbValue::String(value)) if key == SNAPSHOT_PAYLOAD_KEY => {
                    Some(value.as_str())
                }
                _ => None,
            })
            .ok_or(StorageError::CorruptSnapshot(expected_doc_id))?;

        let snapshot = serde_json::from_str::<PersistedSnapshot>(payload)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot: DocumentSnapshot = snapshot.into();

        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }

    fn read_snapshot_locked(
        &self,
        db: &Db,
        doc_id: &Uuid,
    ) -> Result<Option<DocumentSnapshot>, StorageError> {
        let alias = Self::alias(doc_id);
        let result = match db.exec(QueryBuilder::select().ids(alias).query()) {
            Ok(result) => result,
            Err(error) if error.to_string().contains("not found") => return Ok(None),
            Err(error) => return Err(self.map_error("read agdb snapshot", error)),
        };

        let Some(element) = result.elements.into_iter().next() else {
            return Ok(None);
        };

        self.payload_from_element(*doc_id, element).map(Some)
    }
}

impl SnapshotStore for AgdbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let db = self.lock_db()?;
        self.read_snapshot_locked(&db, doc_id)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let alias = Self::alias(&doc_id);
        let payload =
            serde_json::to_string(&PersistedSnapshot::from(snapshot)).map_err(|error| {
                StorageError::Io(format!(
                    "failed to serialize agdb snapshot `{doc_id}`: {error}"
                ))
            })?;
        let mut db = self.lock_db()?;

        db.exec_mut(QueryBuilder::remove().ids(alias.as_str()).query())
            .map_err(|error| self.map_error("remove old agdb snapshot", error))?;
        db.exec_mut(
            QueryBuilder::insert()
                .nodes()
                .aliases(alias.as_str())
                .values([[DbKeyValue {
                    key: SNAPSHOT_PAYLOAD_KEY.into(),
                    value: payload.into(),
                }]])
                .query(),
        )
        .map_err(|error| self.map_error("write agdb snapshot", error))?;

        Ok(())
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let alias = Self::alias(doc_id);
        let mut db = self.lock_db()?;

        db.exec_mut(QueryBuilder::remove().ids(alias.as_str()).query())
            .map_err(|error| self.map_error("delete agdb snapshot", error))?;

        Ok(())
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let db = self.lock_db()?;
        let aliases = db
            .exec(QueryBuilder::select().aliases().query())
            .map_err(|error| self.map_error("read agdb aliases", error))?;
        let mut documents = Vec::new();

        for element in aliases.elements {
            let Some(alias) = alias_from_element(&element) else {
                continue;
            };
            let Some(doc_id_key) = alias.strip_prefix(SNAPSHOT_ALIAS_PREFIX) else {
                continue;
            };
            let Ok(doc_id) = Uuid::parse_str(doc_id_key) else {
                continue;
            };

            match self.read_snapshot_locked(&db, &doc_id)? {
                Some(snapshot) => documents.push(snapshot.document),
                None => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing agdb snapshot while building document catalog"
                ),
            }
        }

        Ok(documents)
    }
}

fn alias_from_element(element: &DbElement) -> Option<&str> {
    element
        .values
        .iter()
        .find_map(|kv| match (&kv.key, &kv.value) {
            (DbValue::String(key), DbValue::String(value)) if key == "alias" => {
                Some(value.as_str())
            }
            _ => None,
        })
}
