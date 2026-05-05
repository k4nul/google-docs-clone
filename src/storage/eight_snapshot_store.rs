use std::{fs, future::Future, path::PathBuf};

use eight::embedded::storage::{Storage as EightStorage, filesystem};
use tokio::runtime::{Builder, Runtime};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

pub struct EightSnapshotStore {
    root: PathBuf,
    runtime: Runtime,
    storage: filesystem::Storage,
}

impl EightSnapshotStore {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let root = root.into();
        if root.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_EIGHT_PATH cannot be empty when SNAPSHOT_STORE=eight".to_owned(),
            ));
        }

        fs::create_dir_all(&root)
            .map_err(|error| StorageError::Io(format!("{}: {error}", root.display())))?;

        let runtime = Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .thread_name("eight-snapshot-store")
            .build()
            .map_err(|error| {
                StorageError::Io(format!(
                    "{}: failed to start eight runtime: {error}",
                    root.display()
                ))
            })?;
        let storage = filesystem::Storage::from_path(&root);

        Ok(Self {
            root,
            runtime,
            storage,
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

    fn snapshot_key(doc_id: &Uuid) -> String {
        format!("doc_{}", doc_id.simple())
    }

    fn parse_doc_id(key: &str) -> Option<Uuid> {
        let hex = key.strip_prefix("doc_")?;
        Uuid::parse_str(hex).ok()
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        raw_snapshot: &str,
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot = serde_json::from_str::<PersistedSnapshot>(raw_snapshot)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot: DocumentSnapshot = snapshot.into();

        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }

    fn map_io_error(&self, error: impl std::fmt::Display) -> StorageError {
        StorageError::Io(format!("{}: {error}", self.root.display()))
    }
}

impl SnapshotStore for EightSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let key = Self::snapshot_key(doc_id);
        let exists = self
            .run(self.storage.exists(key.clone()))
            .map_err(|error| self.map_io_error(error))?;
        if !exists {
            return Ok(None);
        }

        let raw_snapshot = self
            .run(self.storage.get(key))
            .map_err(|error| self.map_io_error(error))?;

        self.deserialize_snapshot(*doc_id, &raw_snapshot).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let key = Self::snapshot_key(&doc_id);
        let raw_snapshot =
            serde_json::to_string(&PersistedSnapshot::from(snapshot)).map_err(|error| {
                StorageError::Io(format!(
                    "failed to serialize eight snapshot `{doc_id}`: {error}"
                ))
            })?;

        self.run(self.storage.set(key, raw_snapshot))
            .map_err(|error| self.map_io_error(error))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let key = Self::snapshot_key(doc_id);
        let exists = self
            .run(self.storage.exists(key.clone()))
            .map_err(|error| self.map_io_error(error))?;
        if !exists {
            return Ok(());
        }

        self.run(self.storage.delete(key))
            .map_err(|error| self.map_io_error(error))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut documents = Vec::new();
        let mut keys = self
            .run(self.storage.search(String::new()))
            .map_err(|error| self.map_io_error(error))?;
        keys.sort();

        for key in keys {
            let Some(doc_id) = Self::parse_doc_id(&key) else {
                continue;
            };

            let raw_snapshot = self
                .run(self.storage.get(key.clone()))
                .map_err(|error| self.map_io_error(error))?;

            match self.deserialize_snapshot(doc_id, &raw_snapshot) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.root.display(),
                    "skipping corrupt eight snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
