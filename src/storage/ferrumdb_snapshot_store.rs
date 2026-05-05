use std::{path::PathBuf, sync::Arc};

use ferrumdb::{Config as FerrumConfig, FerrumDB, FsyncPolicy, StorageEngine};
use tokio::runtime::{Builder, Runtime};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

const SNAPSHOT_CATALOG_KEY: &str = "__catalog__";
const SNAPSHOT_KEY_PREFIX: &str = "snapshot:";

pub struct FerrumdbSnapshotStore {
    path: PathBuf,
    runtime: Runtime,
    engine: Arc<StorageEngine>,
}

impl FerrumdbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_FERRUMDB_PATH cannot be empty when SNAPSHOT_STORE=ferrumdb".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            ensure_snapshot_dir(parent)?;
        }

        let runtime = Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .thread_name("ferrumdb-snapshot-store")
            .build()
            .map_err(|error| {
                StorageError::Io(format!(
                    "{}: failed to start ferrumdb runtime: {error}",
                    path.display()
                ))
            })?;

        let database = runtime
            .block_on(FerrumDB::open(FerrumConfig {
                path: path.clone(),
                encryption_key: None,
                fsync_policy: FsyncPolicy::Always,
            }))
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;
        let engine = database.engine();

        Ok(Self {
            path,
            runtime,
            engine,
        })
    }

    fn snapshot_key(doc_id: &Uuid) -> String {
        format!("{SNAPSHOT_KEY_PREFIX}{doc_id}")
    }

    fn read_catalog(&self) -> Result<Vec<Uuid>, StorageError> {
        let Some(payload) = self.runtime.block_on(self.engine.get(SNAPSHOT_CATALOG_KEY)) else {
            return Ok(Vec::new());
        };

        serde_json::from_value::<Vec<Uuid>>(payload).map_err(|_| {
            StorageError::Io(format!(
                "{}: ferrumdb snapshot catalog is corrupt",
                self.path.display()
            ))
        })
    }

    fn write_catalog(&self, catalog: &[Uuid]) -> Result<(), StorageError> {
        let payload = serde_json::to_value(catalog).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize ferrumdb snapshot catalog: {error}"
            ))
        })?;

        self.runtime
            .block_on(self.engine.set(SNAPSHOT_CATALOG_KEY.to_owned(), payload))
            .map(|_| ())
            .map_err(|error| self.map_error("write ferrumdb snapshot catalog", error))
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        payload: serde_json::Value,
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot = serde_json::from_value::<PersistedSnapshot>(payload)
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

impl SnapshotStore for FerrumdbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let Some(payload) = self
            .runtime
            .block_on(self.engine.get(&Self::snapshot_key(doc_id)))
        else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, payload).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload = serde_json::to_value(PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize ferrumdb snapshot `{doc_id}`: {error}"
            ))
        })?;

        self.runtime
            .block_on(self.engine.set(Self::snapshot_key(&doc_id), payload))
            .map_err(|error| self.map_error("write ferrumdb snapshot", error))?;

        let mut catalog = self.read_catalog()?;
        if !catalog.contains(&doc_id) {
            catalog.push(doc_id);
            catalog.sort_unstable();
            self.write_catalog(&catalog)?;
        }

        Ok(())
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        self.runtime
            .block_on(self.engine.delete(&Self::snapshot_key(doc_id)))
            .map_err(|error| self.map_error("delete ferrumdb snapshot", error))?;

        let mut catalog = self.read_catalog()?;
        let original_len = catalog.len();
        catalog.retain(|catalog_doc_id| catalog_doc_id != doc_id);
        if catalog.len() != original_len {
            self.write_catalog(&catalog)?;
        }

        Ok(())
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let catalog = self.read_catalog()?;
        let mut documents = Vec::new();

        for doc_id in catalog {
            match self
                .runtime
                .block_on(self.engine.get(&Self::snapshot_key(&doc_id)))
            {
                Some(payload) => match self.deserialize_snapshot(doc_id, payload) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping corrupt ferrumdb snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                },
                None => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing ferrumdb snapshot referenced by catalog"
                ),
            }
        }

        Ok(documents)
    }
}
