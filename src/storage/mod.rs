use std::sync::Arc;

use dashmap::DashMap;
use thiserror::Error;
use uuid::Uuid;

use crate::models::document::Document;

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

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("snapshot `{0}` is temporarily busy")]
    Busy(Uuid),
    #[error("snapshot `{0}` was corrupt")]
    CorruptSnapshot(Uuid),
}

pub trait SnapshotStore: Send + Sync {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError>;
    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError>;
    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError>;
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
}

pub fn in_memory_snapshot_store() -> Arc<dyn SnapshotStore> {
    Arc::new(InMemorySnapshotStore::new())
}
