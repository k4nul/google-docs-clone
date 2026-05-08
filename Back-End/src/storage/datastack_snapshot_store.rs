use std::{
    fs,
    future::Future,
    path::{Path, PathBuf},
};

use datastack::DataStack;
use serde_json::{Value, json};
use tokio::runtime::{Builder, Runtime};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const COLLECTION_NAME: &str = "snapshots";
const SNAPSHOT_FIELD: &str = "snapshot";

pub struct DatastackSnapshotStore {
    path: PathBuf,
    runtime: Runtime,
    database: DataStack,
}

impl DatastackSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_DATASTACK_PATH cannot be empty when SNAPSHOT_STORE=datastack".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let runtime = Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .thread_name("datastack-snapshot-store")
            .build()
            .map_err(|error| {
                StorageError::Io(format!(
                    "{}: failed to start datastack runtime: {error}",
                    path.display()
                ))
            })?;
        let database = Self::run_with_runtime(&runtime, DataStack::new(path_to_str(&path)?))
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            runtime,
            database,
        })
    }

    fn run_with_runtime<T>(runtime: &Runtime, future: impl Future<Output = T>) -> T {
        if tokio::runtime::Handle::try_current().is_ok() {
            tokio::task::block_in_place(|| runtime.block_on(future))
        } else {
            runtime.block_on(future)
        }
    }

    fn run<T>(&self, future: impl Future<Output = T>) -> T {
        Self::run_with_runtime(&self.runtime, future)
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

    fn map_error(&self, operation: &str, error: impl std::fmt::Display) -> StorageError {
        StorageError::Io(format!(
            "{}: failed to {operation}: {error}",
            self.path.display()
        ))
    }
}

impl SnapshotStore for DatastackSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let value = self
            .run(self.database.get(COLLECTION_NAME, &Self::doc_key(*doc_id)))
            .map_err(|error| self.map_error("read datastack snapshot", error))?;

        value
            .map(|value| self.deserialize_snapshot(*doc_id, value))
            .transpose()
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let value = Self::serialize_snapshot(snapshot)?;
        self.run(
            self.database
                .add(COLLECTION_NAME, &Self::doc_key(doc_id), &value),
        )
        .map_err(|error| self.map_error("write datastack snapshot", error))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        self.run(
            self.database
                .delete(COLLECTION_NAME, &Self::doc_key(*doc_id)),
        )
        .map_err(|error| self.map_error("delete datastack snapshot", error))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let values = self
            .run(self.database.scan(COLLECTION_NAME, u32::MAX, "", "a"))
            .map_err(|error| self.map_error("scan datastack snapshot catalog", error))?;

        let mut documents = Vec::new();
        let Value::Object(values) = values else {
            return Ok(documents);
        };

        for (doc_id_key, value) in values {
            let Ok(doc_id) = Uuid::parse_str(&doc_id_key) else {
                continue;
            };

            match self.deserialize_snapshot(doc_id, value) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt datastack snapshot while building document catalog"
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
            "SNAPSHOT_DATASTACK_PATH must be valid UTF-8, received `{}`",
            path.display()
        ))
    })
}
