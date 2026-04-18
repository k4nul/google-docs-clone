mod file_snapshot_store;

use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::{config::Config, models::document::Document};

pub use file_snapshot_store::FileSnapshotStore;

#[derive(Debug, Clone)]
pub struct DocumentSnapshot {
    pub document: Document,
    pub update: Vec<u8>,
}

impl DocumentSnapshot {
    pub fn new(document: Document, update: Vec<u8>) -> Self {
        Self { document, update }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PersistedSnapshot {
    document: PersistedDocument,
    update: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedDocument {
    id: Uuid,
    title: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    access_token: String,
}

impl From<DocumentSnapshot> for PersistedSnapshot {
    fn from(snapshot: DocumentSnapshot) -> Self {
        let document = snapshot.document;
        let access_token = document.access_token().to_owned();

        Self {
            document: PersistedDocument {
                id: document.id,
                title: document.title,
                created_at: document.created_at,
                updated_at: document.updated_at,
                access_token,
            },
            update: snapshot.update,
        }
    }
}

impl From<PersistedSnapshot> for DocumentSnapshot {
    fn from(snapshot: PersistedSnapshot) -> Self {
        Self {
            document: Document::from_parts(
                snapshot.document.id,
                snapshot.document.title,
                snapshot.document.created_at,
                snapshot.document.updated_at,
                snapshot.document.access_token,
            ),
            update: snapshot.update,
        }
    }
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("snapshot `{0}` is temporarily busy")]
    Busy(Uuid),
    #[error("snapshot `{0}` was corrupt")]
    CorruptSnapshot(Uuid),
    #[error("snapshot storage I/O failed: {0}")]
    Io(String),
    #[error("snapshot storage configuration is invalid: {0}")]
    Config(String),
}

pub trait SnapshotStore: Send + Sync {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError>;
    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError>;
    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError>;
    fn list_documents(&self) -> Result<Vec<Document>, StorageError>;
}

#[derive(Default)]
pub struct InMemorySnapshotStore {
    snapshots: DashMap<Uuid, DocumentSnapshot>,
}

impl InMemorySnapshotStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SnapshotStore for InMemorySnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        Ok(self
            .snapshots
            .get(doc_id)
            .map(|entry| entry.value().clone()))
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        self.snapshots.insert(snapshot.document.id, snapshot);
        Ok(())
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        self.snapshots.remove(doc_id);
        Ok(())
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        Ok(self
            .snapshots
            .iter()
            .map(|entry| entry.value().document.clone())
            .collect())
    }
}

pub fn in_memory_snapshot_store() -> Arc<dyn SnapshotStore> {
    Arc::new(InMemorySnapshotStore::new())
}

pub fn file_snapshot_store(
    root: impl Into<PathBuf>,
) -> Result<Arc<dyn SnapshotStore>, StorageError> {
    Ok(Arc::new(FileSnapshotStore::new(root)?))
}

pub fn snapshot_store_from_config(config: &Config) -> Result<Arc<dyn SnapshotStore>, StorageError> {
    match config.snapshot_store.trim().to_ascii_lowercase().as_str() {
        "memory" => Ok(in_memory_snapshot_store()),
        "file" => file_snapshot_store(&config.snapshot_dir),
        other => Err(StorageError::Config(format!(
            "SNAPSHOT_STORE must be `memory` or `file`, received `{other}`"
        ))),
    }
}

pub(crate) fn ensure_snapshot_dir(path: &Path) -> Result<(), StorageError> {
    fs::create_dir_all(path)
        .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))
}
