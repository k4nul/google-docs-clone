use std::{path::PathBuf, sync::Mutex};

use ipjdb::{Collection, Db, Id, Item};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const COLLECTION_NAME: &str = "snapshots";

#[derive(Debug, Clone, Deserialize, Serialize)]
struct IpjdbSnapshotRecord {
    doc_id: Uuid,
    snapshot: PersistedSnapshot,
}

pub struct IpjdbSnapshotStore {
    path: PathBuf,
    collection: Mutex<Collection>,
}

impl IpjdbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_IPJDB_PATH cannot be empty when SNAPSHOT_STORE=ipjdb".to_owned(),
            ));
        }

        let database = Db::open(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;
        let collection = database
            .collection(COLLECTION_NAME)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            collection: Mutex::new(collection),
        })
    }

    fn with_collection<T>(
        &self,
        operation: &str,
        f: impl FnOnce(&Collection) -> Result<T, ipjdb::Error>,
    ) -> Result<T, StorageError> {
        let collection = self.collection.lock().map_err(|_| {
            StorageError::Io(format!("{}: ipjdb mutex was poisoned", self.path.display()))
        })?;

        f(&collection).map_err(|error| {
            StorageError::Io(format!(
                "{}: failed to {operation}: {error}",
                self.path.display()
            ))
        })
    }

    fn deserialize_snapshot(
        expected_doc_id: Uuid,
        record: IpjdbSnapshotRecord,
    ) -> Result<DocumentSnapshot, StorageError> {
        if record.doc_id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        let snapshot: DocumentSnapshot = record.snapshot.into();
        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }

    fn item_ids_for_doc(&self, doc_id: Uuid) -> Result<Vec<Id>, StorageError> {
        let items = self.with_collection("scan ipjdb snapshot catalog", |collection| {
            collection.get_all::<IpjdbSnapshotRecord>()
        })?;

        Ok(items
            .into_iter()
            .filter_map(|item| (item.data.doc_id == doc_id).then_some(item.id))
            .collect())
    }

    fn delete_items(&self, item_ids: Vec<Id>) -> Result<(), StorageError> {
        self.with_collection("delete ipjdb snapshots", |collection| {
            for item_id in item_ids {
                collection.delete_one(&item_id)?;
            }
            Ok(())
        })
    }
}

impl SnapshotStore for IpjdbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let items = self.with_collection("load ipjdb snapshot", |collection| {
            collection.find_many::<IpjdbSnapshotRecord, _>(|item| item.data.doc_id == *doc_id)
        })?;

        let Some(item) = items.into_iter().next() else {
            return Ok(None);
        };

        Self::deserialize_snapshot(*doc_id, item.data).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let record = IpjdbSnapshotRecord {
            doc_id,
            snapshot: PersistedSnapshot::from(snapshot),
        };

        self.delete_items(self.item_ids_for_doc(doc_id)?)?;
        self.with_collection("write ipjdb snapshot", |collection| {
            collection.insert_one(&record).map(|_| ())
        })
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        self.delete_items(self.item_ids_for_doc(*doc_id)?)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let items = self.with_collection("list ipjdb snapshots", |collection| {
            collection.get_all::<IpjdbSnapshotRecord>()
        })?;

        let mut documents = Vec::new();
        for Item { data, .. } in items {
            match Self::deserialize_snapshot(data.doc_id, data) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt ipjdb snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        documents.sort_unstable_by_key(|document| document.id);
        Ok(documents)
    }
}
