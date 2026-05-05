use std::{
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use ghaladb::{DatabaseOptions, GhalaDb};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

const SNAPSHOT_CATALOG_KEY: &str = "__catalog__";
const SNAPSHOT_KEY_PREFIX: &str = "snapshot:";

type SnapshotDb = GhalaDb<String, String>;

pub struct GhaladbSnapshotStore {
    path: PathBuf,
    database: Mutex<SnapshotDb>,
}

impl GhaladbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_GHALADB_PATH cannot be empty when SNAPSHOT_STORE=ghaladb".to_owned(),
            ));
        }

        ensure_snapshot_dir(&path)?;

        let options = DatabaseOptions::builder()
            .sync(true)
            .compact(false)
            .compress(false)
            .vlog_mem_buf_enabled(false)
            .build();
        let database = SnapshotDb::new(&path, Some(options))
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            database: Mutex::new(database),
        })
    }

    fn snapshot_key(doc_id: &Uuid) -> String {
        format!("{SNAPSHOT_KEY_PREFIX}{doc_id}")
    }

    fn lock_database(&self) -> Result<MutexGuard<'_, SnapshotDb>, StorageError> {
        self.database.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: ghaladb database mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn map_error(&self, action: &str, error: ghaladb::GhalaDbError) -> StorageError {
        StorageError::Io(format!("{}: {action} failed: {error}", self.path.display()))
    }

    fn read_value(
        &self,
        database: &mut SnapshotDb,
        key: &str,
    ) -> Result<Option<String>, StorageError> {
        database
            .get(key)
            .map_err(|error| self.map_error("read ghaladb snapshot value", error))
    }

    fn write_value(
        &self,
        database: &mut SnapshotDb,
        key: &str,
        value: &str,
    ) -> Result<(), StorageError> {
        database
            .put(key, &value.to_owned())
            .map_err(|error| self.map_error("write ghaladb snapshot value", error))
    }

    fn sync(&self, database: &mut SnapshotDb) -> Result<(), StorageError> {
        database
            .sync()
            .map_err(|error| self.map_error("sync ghaladb snapshot store", error))
    }

    fn read_catalog(&self, database: &mut SnapshotDb) -> Result<Vec<Uuid>, StorageError> {
        let Some(payload) = self.read_value(database, SNAPSHOT_CATALOG_KEY)? else {
            return Ok(Vec::new());
        };

        serde_json::from_str::<Vec<Uuid>>(&payload).map_err(|_| {
            StorageError::Io(format!(
                "{}: ghaladb snapshot catalog is corrupt",
                self.path.display()
            ))
        })
    }

    fn write_catalog(
        &self,
        database: &mut SnapshotDb,
        catalog: &[Uuid],
    ) -> Result<(), StorageError> {
        let payload = serde_json::to_string(catalog).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize ghaladb snapshot catalog: {error}"
            ))
        })?;

        self.write_value(database, SNAPSHOT_CATALOG_KEY, &payload)
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        payload: &str,
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot = serde_json::from_str::<PersistedSnapshot>(payload)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot: DocumentSnapshot = snapshot.into();

        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }
}

impl SnapshotStore for GhaladbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let mut database = self.lock_database()?;
        let Some(payload) = self.read_value(&mut database, &Self::snapshot_key(doc_id))? else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &payload).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let mut database = self.lock_database()?;
        let doc_id = snapshot.document.id;
        let payload =
            serde_json::to_string(&PersistedSnapshot::from(snapshot)).map_err(|error| {
                StorageError::Io(format!(
                    "failed to serialize ghaladb snapshot `{doc_id}`: {error}"
                ))
            })?;

        self.write_value(&mut database, &Self::snapshot_key(&doc_id), &payload)?;

        let mut catalog = self.read_catalog(&mut database)?;
        if !catalog.contains(&doc_id) {
            catalog.push(doc_id);
            catalog.sort_unstable();
            self.write_catalog(&mut database, &catalog)?;
        }

        self.sync(&mut database)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut database = self.lock_database()?;
        database
            .delete(&Self::snapshot_key(doc_id))
            .map_err(|error| self.map_error("delete ghaladb snapshot value", error))?;

        let mut catalog = self.read_catalog(&mut database)?;
        let original_len = catalog.len();
        catalog.retain(|catalog_doc_id| catalog_doc_id != doc_id);
        if catalog.len() != original_len {
            self.write_catalog(&mut database, &catalog)?;
        }

        self.sync(&mut database)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut database = self.lock_database()?;
        let catalog = self.read_catalog(&mut database)?;
        let mut documents = Vec::new();

        for doc_id in catalog {
            match self.read_value(&mut database, &Self::snapshot_key(&doc_id)) {
                Ok(Some(payload)) => match self.deserialize_snapshot(doc_id, &payload) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping corrupt ghaladb snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                },
                Ok(None) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing ghaladb snapshot referenced by catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
