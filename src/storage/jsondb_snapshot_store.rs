use std::{collections::HashMap, fs, path::PathBuf};

use jsondb::{JsonDb, SchemaV0};
use serde::{Deserialize, Serialize};
use tokio::runtime::{Builder, Runtime};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

#[derive(Debug, Default, Serialize, Deserialize)]
struct JsondbCatalog {
    snapshots: HashMap<String, PersistedSnapshot>,
}

impl SchemaV0 for JsondbCatalog {
    const VERSION_OPTIONAL: bool = true;
}

pub struct JsondbSnapshotStore {
    path: PathBuf,
    _runtime: Runtime,
    database: JsonDb<JsondbCatalog>,
}

impl JsondbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_JSONDB_PATH cannot be empty when SNAPSHOT_STORE=jsondb".to_owned(),
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
            .thread_name("jsondb-snapshot-store")
            .build()
            .map_err(|error| {
                StorageError::Io(format!(
                    "{}: failed to start jsondb runtime: {error}",
                    path.display()
                ))
            })?;
        let database = runtime
            .block_on(JsonDb::load_or(
                path.clone().into(),
                JsondbCatalog::default(),
            ))
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;
        database.blocking_flush();

        Ok(Self {
            path,
            _runtime: runtime,
            database,
        })
    }
}

impl SnapshotStore for JsondbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let catalog = self.database.blocking_read();
        let Some(snapshot) = catalog.snapshots.get(&doc_id.to_string()) else {
            return Ok(None);
        };
        let snapshot: DocumentSnapshot = snapshot.clone().into();

        if snapshot.document.id != *doc_id {
            return Err(StorageError::CorruptSnapshot(*doc_id));
        }

        Ok(Some(snapshot))
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        {
            let mut catalog = self.database.blocking_write();
            catalog
                .snapshots
                .insert(doc_id.to_string(), PersistedSnapshot::from(snapshot));
        }
        self.database.blocking_flush();

        Ok(())
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        {
            let mut catalog = self.database.blocking_write();
            catalog.snapshots.remove(&doc_id.to_string());
        }
        self.database.blocking_flush();

        Ok(())
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let catalog = self.database.blocking_read();
        let mut documents = Vec::new();

        for (doc_id_key, snapshot) in &catalog.snapshots {
            let Ok(doc_id) = Uuid::parse_str(doc_id_key) else {
                continue;
            };

            let snapshot: DocumentSnapshot = snapshot.clone().into();
            if snapshot.document.id != doc_id {
                tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt jsondb snapshot while building document catalog"
                );
                continue;
            }

            documents.push(snapshot.document);
        }

        Ok(documents)
    }
}
