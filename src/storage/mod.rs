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
    #[error("document `{0}` still has active collaboration sessions")]
    DocumentBusy(Uuid),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::document::Document;

    fn make_snapshot(title: &str) -> DocumentSnapshot {
        let doc = Document::new(Uuid::new_v4(), Some(title.to_owned()));
        DocumentSnapshot::new(doc, vec![1, 2, 3])
    }

    #[test]
    fn in_memory_store_returns_none_for_missing_snapshot() {
        let store = InMemorySnapshotStore::new();
        let result = store
            .load_snapshot(&Uuid::new_v4())
            .expect("lookup should not error");
        assert!(result.is_none());
    }

    #[test]
    fn in_memory_store_saves_and_loads_snapshot() {
        let store = InMemorySnapshotStore::new();
        let snapshot = make_snapshot("Hello");
        let doc_id = snapshot.document.id;

        store.save_snapshot(snapshot).expect("save should succeed");

        let loaded = store
            .load_snapshot(&doc_id)
            .expect("load should not error")
            .expect("snapshot should exist after save");

        assert_eq!(loaded.document.id, doc_id);
        assert_eq!(loaded.update, vec![1, 2, 3]);
    }

    #[test]
    fn in_memory_store_replaces_existing_snapshot_on_second_save() {
        let store = InMemorySnapshotStore::new();
        let doc = Document::new(Uuid::new_v4(), Some("Original".to_owned()));
        let doc_id = doc.id;

        store
            .save_snapshot(DocumentSnapshot::new(doc.clone(), vec![1]))
            .expect("first save should succeed");
        store
            .save_snapshot(DocumentSnapshot::new(doc, vec![2]))
            .expect("second save should succeed");

        let loaded = store
            .load_snapshot(&doc_id)
            .expect("load should not error")
            .expect("snapshot should exist");
        assert_eq!(loaded.update, vec![2]);
    }

    #[test]
    fn in_memory_store_delete_removes_snapshot() {
        let store = InMemorySnapshotStore::new();
        let snapshot = make_snapshot("Deletable");
        let doc_id = snapshot.document.id;

        store.save_snapshot(snapshot).expect("save should succeed");
        store
            .delete_snapshot(&doc_id)
            .expect("delete should succeed");

        let result = store
            .load_snapshot(&doc_id)
            .expect("lookup after delete should not error");
        assert!(result.is_none());
    }

    #[test]
    fn in_memory_store_delete_is_idempotent_for_missing_snapshot() {
        let store = InMemorySnapshotStore::new();
        store
            .delete_snapshot(&Uuid::new_v4())
            .expect("deleting a non-existent snapshot should not error");
    }

    #[test]
    fn in_memory_store_lists_all_saved_documents() {
        let store = InMemorySnapshotStore::new();
        let snapshot_a = make_snapshot("Alpha");
        let snapshot_b = make_snapshot("Beta");
        let id_a = snapshot_a.document.id;
        let id_b = snapshot_b.document.id;

        store.save_snapshot(snapshot_a).expect("save A should succeed");
        store.save_snapshot(snapshot_b).expect("save B should succeed");

        let mut listed = store.list_documents().expect("list should succeed");
        listed.sort_by_key(|d| d.id);

        let mut expected_ids = vec![id_a, id_b];
        expected_ids.sort();

        assert_eq!(
            listed.iter().map(|d| d.id).collect::<Vec<_>>(),
            expected_ids
        );
    }

    #[test]
    fn in_memory_store_returns_empty_list_when_no_snapshots() {
        let store = InMemorySnapshotStore::new();
        let documents = store.list_documents().expect("list should succeed");
        assert!(documents.is_empty());
    }

    #[test]
    fn in_memory_store_excludes_deleted_document_from_list() {
        let store = InMemorySnapshotStore::new();
        let snapshot = make_snapshot("Temporary");
        let doc_id = snapshot.document.id;

        store.save_snapshot(snapshot).expect("save should succeed");
        store
            .delete_snapshot(&doc_id)
            .expect("delete should succeed");

        let documents = store.list_documents().expect("list should succeed");
        assert!(documents.is_empty());
    }
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
