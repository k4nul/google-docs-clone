use std::{fs, future::Future, path::PathBuf, sync::Mutex};

use nanodb::{error::NanoDBError, nanodb::NanoDB};
use serde_json::Value;
use tokio::runtime::{Builder, Runtime};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

pub struct NanodbSnapshotStore {
    path: PathBuf,
    runtime: Runtime,
    db: Mutex<NanoDB>,
}

impl NanodbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_NANODB_PATH cannot be empty when SNAPSHOT_STORE=nanodb".to_owned(),
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
            .thread_name("nanodb-snapshot-store")
            .build()
            .map_err(|error| {
                StorageError::Io(format!(
                    "{}: failed to start nanodb runtime: {error}",
                    path.display()
                ))
            })?;

        let db = NanoDB::open(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            runtime,
            db: Mutex::new(db),
        })
    }

    fn run<T>(&self, future: impl Future<Output = T>) -> T {
        if tokio::runtime::Handle::try_current().is_ok() {
            tokio::task::block_in_place(|| self.runtime.block_on(future))
        } else {
            self.runtime.block_on(future)
        }
    }

    fn lock_db(&self) -> Result<std::sync::MutexGuard<'_, NanoDB>, StorageError> {
        self.db.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: nanodb mutex was poisoned",
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

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        value: Value,
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot = serde_json::from_value::<PersistedSnapshot>(value)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot: DocumentSnapshot = snapshot.into();

        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }
}

impl SnapshotStore for NanodbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let db = self.lock_db()?;
        let value = self.run(async { db.data().await.get(&doc_id.to_string()) });
        let value = match value {
            Ok(value) => value.inner(),
            Err(NanoDBError::KeyNotFound(_)) => return Ok(None),
            Err(error) => return Err(self.map_error("read nanodb snapshot", error)),
        };

        self.deserialize_snapshot(*doc_id, value).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let value = serde_json::to_value(PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize nanodb snapshot `{doc_id}`: {error}"
            ))
        })?;

        let mut db = self.lock_db()?;
        self.run(async {
            db.insert(&doc_id.to_string(), value)
                .await
                .map_err(|error| self.map_error("write nanodb snapshot", error))?;
            db.write()
                .await
                .map_err(|error| self.map_error("flush nanodb snapshot store", error))
        })
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut db = self.lock_db()?;
        self.run(async {
            match db.remove(&doc_id.to_string()).await {
                Ok(()) | Err(NanoDBError::KeyNotFound(_)) => {}
                Err(error) => return Err(self.map_error("delete nanodb snapshot", error)),
            }

            db.write()
                .await
                .map_err(|error| self.map_error("flush nanodb snapshot store", error))
        })
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let db = self.lock_db()?;
        let value = self.run(async { db.data().await.inner() });
        let Some(entries) = value.as_object() else {
            return Err(StorageError::Io(format!(
                "{}: nanodb root is not a JSON object",
                self.path.display()
            )));
        };

        let mut documents = Vec::new();
        for (doc_id_key, value) in entries {
            let Ok(doc_id) = Uuid::parse_str(doc_id_key) else {
                continue;
            };

            match self.deserialize_snapshot(doc_id, value.clone()) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt nanodb snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        documents.sort_unstable_by_key(|document| document.id);
        Ok(documents)
    }
}
