use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use deeb_core::{
    database::{Database, instance_name::InstanceName, query::Query},
    entity::Entity,
};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const INSTANCE_NAME: &str = "backend";
const ENTITY_NAME: &str = "snapshots";
const DOC_ID_FIELD: &str = "doc_id";
const SNAPSHOT_FIELD: &str = "snapshot";

pub struct DeebSnapshotStore {
    path: PathBuf,
    entity: Entity,
    database: Mutex<Database>,
}

impl DeebSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_DEEB_PATH cannot be empty when SNAPSHOT_STORE=deeb".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let mut entity = Entity::new(ENTITY_NAME);
        let entity = entity.primary_key(DOC_ID_FIELD);
        let mut database = Database::new();
        let instance_name = Self::instance_name();
        let path_string = path.to_string_lossy().into_owned();
        database
            .add_instance(&instance_name, &path_string, vec![entity.clone()])
            .and_then(|database| database.load_instance(&instance_name))
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            entity,
            database: Mutex::new(database),
        })
    }

    fn instance_name() -> InstanceName {
        InstanceName::from(INSTANCE_NAME)
    }

    fn lock_database(&self) -> Result<MutexGuard<'_, Database>, StorageError> {
        self.database.lock().map_err(|_| {
            StorageError::Io(format!("{}: deeb mutex was poisoned", self.path.display()))
        })
    }

    fn map_error(&self, operation: &str, error: impl std::fmt::Display) -> StorageError {
        StorageError::Io(format!(
            "{}: failed to {operation}: {error}",
            self.path.display()
        ))
    }

    fn query_doc_id(doc_id: Uuid) -> Query {
        Query::eq(DOC_ID_FIELD, Value::String(doc_id.to_string()))
    }

    fn commit(&self, database: &Database) -> Result<(), StorageError> {
        database
            .commit(vec![Self::instance_name()])
            .map_err(|error| self.map_error("commit deeb snapshot store", error))
    }

    fn serialize_snapshot(snapshot: DocumentSnapshot) -> Result<Value, StorageError> {
        let doc_id = snapshot.document.id;
        let persisted = PersistedSnapshot::from(snapshot);
        Ok(json!({
            DOC_ID_FIELD: doc_id.to_string(),
            SNAPSHOT_FIELD: persisted,
        }))
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

impl SnapshotStore for DeebSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let database = self.lock_database()?;
        let values = database
            .find_many(&self.entity, Self::query_doc_id(*doc_id), None)
            .map_err(|error| self.map_error("read deeb snapshot", error))?;

        let Some(value) = values.into_iter().next() else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, value).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let value = Self::serialize_snapshot(snapshot)?;
        let mut database = self.lock_database()?;

        database
            .delete_many(&self.entity, Self::query_doc_id(doc_id))
            .map_err(|error| self.map_error("replace deeb snapshot", error))?;
        database
            .insert_one(&self.entity, value)
            .map_err(|error| self.map_error("write deeb snapshot", error))?;
        self.commit(&database)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut database = self.lock_database()?;
        database
            .delete_many(&self.entity, Self::query_doc_id(*doc_id))
            .map_err(|error| self.map_error("delete deeb snapshot", error))?;
        self.commit(&database)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let database = self.lock_database()?;
        let values = database
            .find_many(&self.entity, Query::all(), None)
            .map_err(|error| self.map_error("list deeb snapshots", error))?;

        let mut documents = Vec::new();
        for value in values {
            let Some(doc_id) = value
                .get(DOC_ID_FIELD)
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
                    "skipping corrupt deeb snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        documents.sort_unstable_by_key(|document| document.id);
        Ok(documents)
    }
}
