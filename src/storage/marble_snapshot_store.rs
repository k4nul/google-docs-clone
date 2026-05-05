use std::{
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use marble::{Config as MarbleConfig, Marble};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

const CATALOG_OBJECT_ID: u64 = 1;
const FIRST_SNAPSHOT_OBJECT_ID: u64 = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MarbleCatalog {
    next_object_id: u64,
    documents: Vec<MarbleCatalogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MarbleCatalogEntry {
    doc_id: Uuid,
    object_id: u64,
}

impl Default for MarbleCatalog {
    fn default() -> Self {
        Self {
            next_object_id: FIRST_SNAPSHOT_OBJECT_ID,
            documents: Vec::new(),
        }
    }
}

pub struct MarbleSnapshotStore {
    path: PathBuf,
    heap: Mutex<Marble>,
}

impl MarbleSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_MARBLE_PATH cannot be empty when SNAPSHOT_STORE=marble".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            ensure_snapshot_dir(parent)?;
        }

        let heap = MarbleConfig {
            path: path.clone(),
            fsync_each_batch: true,
            ..MarbleConfig::default()
        }
        .open()
        .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            heap: Mutex::new(heap),
        })
    }

    fn lock_heap(&self) -> Result<MutexGuard<'_, Marble>, StorageError> {
        self.heap.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: marble mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn load_catalog(&self, heap: &Marble) -> Result<MarbleCatalog, StorageError> {
        let Some(bytes) = heap
            .read(CATALOG_OBJECT_ID)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?
        else {
            return Ok(MarbleCatalog::default());
        };

        serde_json::from_slice(bytes.as_ref()).map_err(|error| {
            StorageError::Io(format!(
                "{}: marble snapshot catalog is corrupt: {error}",
                self.path.display()
            ))
        })
    }

    fn serialize_catalog(&self, catalog: &MarbleCatalog) -> Result<Vec<u8>, StorageError> {
        serde_json::to_vec(catalog).map_err(|error| {
            StorageError::Io(format!(
                "{}: failed to serialize marble snapshot catalog: {error}",
                self.path.display()
            ))
        })
    }

    fn object_id_for(catalog: &MarbleCatalog, doc_id: &Uuid) -> Option<u64> {
        catalog
            .documents
            .iter()
            .find(|entry| entry.doc_id == *doc_id)
            .map(|entry| entry.object_id)
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        bytes: &[u8],
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot: PersistedSnapshot = serde_json::from_slice(bytes)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot: DocumentSnapshot = snapshot.into();

        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }
}

impl SnapshotStore for MarbleSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let heap = self.lock_heap()?;
        let catalog = self.load_catalog(&heap)?;
        let Some(object_id) = Self::object_id_for(&catalog, doc_id) else {
            return Ok(None);
        };

        let Some(bytes) = heap
            .read(object_id)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?
        else {
            tracing::warn!(
                doc_id = %doc_id,
                path = %self.path.display(),
                object_id,
                "marble snapshot catalog referenced a missing object"
            );
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, bytes.as_ref()).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let snapshot_bytes =
            serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
                StorageError::Io(format!(
                    "{}: failed to serialize marble snapshot `{doc_id}`: {error}",
                    self.path.display()
                ))
            })?;

        let heap = self.lock_heap()?;
        let mut catalog = self.load_catalog(&heap)?;
        let object_id = match Self::object_id_for(&catalog, &doc_id) {
            Some(object_id) => object_id,
            None => {
                let object_id = catalog.next_object_id;
                catalog.next_object_id =
                    catalog.next_object_id.checked_add(1).ok_or_else(|| {
                        StorageError::Io(format!(
                            "{}: marble snapshot object id overflow",
                            self.path.display()
                        ))
                    })?;
                catalog
                    .documents
                    .push(MarbleCatalogEntry { doc_id, object_id });
                catalog.documents.sort_by_key(|entry| entry.doc_id);
                object_id
            }
        };
        let catalog_bytes = self.serialize_catalog(&catalog)?;

        heap.write_batch([
            (object_id, Some(snapshot_bytes)),
            (CATALOG_OBJECT_ID, Some(catalog_bytes)),
        ])
        .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let heap = self.lock_heap()?;
        let mut catalog = self.load_catalog(&heap)?;
        let Some(object_id) = Self::object_id_for(&catalog, doc_id) else {
            return Ok(());
        };

        catalog.documents.retain(|entry| entry.doc_id != *doc_id);
        let catalog_bytes = self.serialize_catalog(&catalog)?;

        heap.write_batch([
            (object_id, Option::<Vec<u8>>::None),
            (CATALOG_OBJECT_ID, Some(catalog_bytes)),
        ])
        .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let heap = self.lock_heap()?;
        let catalog = self.load_catalog(&heap)?;
        let mut documents = Vec::new();

        for entry in catalog.documents {
            let Some(bytes) = heap
                .read(entry.object_id)
                .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?
            else {
                tracing::warn!(
                    doc_id = %entry.doc_id,
                    path = %self.path.display(),
                    object_id = entry.object_id,
                    "skipping missing marble snapshot while building document catalog"
                );
                continue;
            };

            match self.deserialize_snapshot(entry.doc_id, bytes.as_ref()) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    object_id = entry.object_id,
                    "skipping corrupt marble snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
