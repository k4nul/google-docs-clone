use std::{
    any::Any,
    collections::HashMap,
    fs, panic,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use tinybase::{Index, Record, Table, TinyBase};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOT_TABLE_NAME: &str = "snapshots";
const SNAPSHOT_DOC_ID_INDEX_NAME: &str = "doc_id";
const SNAPSHOT_CATALOG_INDEX_NAME: &str = "catalog";
const SNAPSHOT_CATALOG_VALUE: &str = "documents";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TinybaseSnapshotRecord {
    doc_id: String,
    catalog_key: String,
    snapshot: PersistedSnapshot,
}

impl TinybaseSnapshotRecord {
    fn from_snapshot(snapshot: DocumentSnapshot) -> Self {
        let doc_id = snapshot.document.id.to_string();

        Self {
            doc_id,
            catalog_key: SNAPSHOT_CATALOG_VALUE.to_owned(),
            snapshot: PersistedSnapshot::from(snapshot),
        }
    }
}

pub struct TinybaseSnapshotStore {
    path: PathBuf,
}

struct TinybaseHandles {
    table: Table<TinybaseSnapshotRecord>,
    doc_id_index: Index<TinybaseSnapshotRecord, String>,
    catalog_index: Index<TinybaseSnapshotRecord, String>,
}

impl TinybaseSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_TINYBASE_PATH cannot be empty when SNAPSHOT_STORE=tinybase".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let store = Self { path };
        let _ = store.open_handles()?;
        Ok(store)
    }

    fn open_database(path: &Path) -> Result<TinyBase, StorageError> {
        let path_string = path.to_string_lossy().into_owned();

        panic::catch_unwind(|| TinyBase::new(Some(path_string.as_str()), false)).map_err(
            |payload| {
                StorageError::Io(format!(
                    "{}: failed to open tinybase snapshot store: {}",
                    path.display(),
                    Self::panic_message(payload)
                ))
            },
        )
    }

    fn panic_message(payload: Box<dyn Any + Send>) -> String {
        if let Some(message) = payload.downcast_ref::<&str>() {
            return (*message).to_owned();
        }

        if let Some(message) = payload.downcast_ref::<String>() {
            return message.clone();
        }

        "unknown panic".to_owned()
    }

    fn map_tinybase_error(
        path: &Path,
        operation: &str,
        error: tinybase::result::TinyBaseError,
    ) -> StorageError {
        StorageError::Io(format!(
            "{}: failed to {operation}: {error}",
            path.display()
        ))
    }

    fn open_handles(&self) -> Result<TinybaseHandles, StorageError> {
        let database = Self::open_database(&self.path)?;
        let table = database
            .open_table::<TinybaseSnapshotRecord>(SNAPSHOT_TABLE_NAME)
            .map_err(|error| {
                Self::map_tinybase_error(&self.path, "open tinybase snapshot table", error)
            })?;
        let doc_id_index = table
            .create_index(SNAPSHOT_DOC_ID_INDEX_NAME, |record| record.doc_id.clone())
            .map_err(|error| {
                Self::map_tinybase_error(&self.path, "create tinybase doc_id index", error)
            })?;
        doc_id_index.sync().map_err(|error| {
            Self::map_tinybase_error(&self.path, "sync tinybase doc_id index", error)
        })?;
        let catalog_index = table
            .create_index(SNAPSHOT_CATALOG_INDEX_NAME, |record| {
                record.catalog_key.clone()
            })
            .map_err(|error| {
                Self::map_tinybase_error(&self.path, "create tinybase catalog index", error)
            })?;
        catalog_index.sync().map_err(|error| {
            Self::map_tinybase_error(&self.path, "sync tinybase catalog index", error)
        })?;

        Ok(TinybaseHandles {
            table,
            doc_id_index,
            catalog_index,
        })
    }

    fn load_records_for_doc_id(
        &self,
        doc_id: &Uuid,
    ) -> Result<Vec<Record<TinybaseSnapshotRecord>>, StorageError> {
        self.open_handles()?
            .doc_id_index
            .select(&doc_id.to_string())
            .map_err(|error| {
                Self::map_tinybase_error(&self.path, "query tinybase snapshot by doc_id", error)
            })
    }

    fn load_catalog_records(&self) -> Result<Vec<Record<TinybaseSnapshotRecord>>, StorageError> {
        self.open_handles()?
            .catalog_index
            .select(&SNAPSHOT_CATALOG_VALUE.to_owned())
            .map_err(|error| {
                Self::map_tinybase_error(&self.path, "query tinybase snapshot catalog", error)
            })
    }

    fn latest_record(
        &self,
        records: Vec<Record<TinybaseSnapshotRecord>>,
    ) -> Option<Record<TinybaseSnapshotRecord>> {
        records.into_iter().max_by_key(|record| record.id)
    }

    fn record_to_snapshot(
        &self,
        expected_doc_id: Uuid,
        record: TinybaseSnapshotRecord,
    ) -> Result<DocumentSnapshot, StorageError> {
        if record.doc_id != expected_doc_id.to_string() {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        let snapshot: DocumentSnapshot = record.snapshot.into();
        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }
}

impl SnapshotStore for TinybaseSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let Some(record) = self.latest_record(self.load_records_for_doc_id(doc_id)?) else {
            return Ok(None);
        };

        self.record_to_snapshot(*doc_id, record.data).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let record = TinybaseSnapshotRecord::from_snapshot(snapshot);
        let handles = self.open_handles()?;
        let inserted_id = handles.table.insert(record).map_err(|error| {
            Self::map_tinybase_error(&self.path, "write tinybase snapshot", error)
        })?;

        for stale_record in handles
            .doc_id_index
            .select(&doc_id.to_string())
            .map_err(|error| {
                Self::map_tinybase_error(&self.path, "query tinybase snapshot by doc_id", error)
            })?
            .into_iter()
            .filter(|record| record.id != inserted_id)
        {
            handles.table.delete(stale_record.id).map_err(|error| {
                Self::map_tinybase_error(&self.path, "delete stale tinybase snapshot", error)
            })?;
        }

        Ok(())
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let handles = self.open_handles()?;
        for record in handles
            .doc_id_index
            .select(&doc_id.to_string())
            .map_err(|error| {
                Self::map_tinybase_error(&self.path, "query tinybase snapshot by doc_id", error)
            })?
        {
            handles.table.delete(record.id).map_err(|error| {
                Self::map_tinybase_error(&self.path, "delete tinybase snapshot", error)
            })?;
        }

        Ok(())
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut latest_records: HashMap<String, Record<TinybaseSnapshotRecord>> = HashMap::new();

        for record in self.load_catalog_records()? {
            let doc_id_key = record.data.doc_id.clone();
            match latest_records.get(&doc_id_key) {
                Some(existing_record) if existing_record.id >= record.id => {}
                _ => {
                    latest_records.insert(doc_id_key, record);
                }
            }
        }

        let mut documents = Vec::new();
        for (doc_id_key, record) in latest_records {
            let Ok(doc_id) = Uuid::parse_str(&doc_id_key) else {
                continue;
            };

            match self.record_to_snapshot(doc_id, record.data) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt tinybase snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        documents.sort_by_key(|document| document.id.to_string());
        Ok(documents)
    }
}
