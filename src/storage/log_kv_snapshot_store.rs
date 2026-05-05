use std::{
    fs::{File, OpenOptions},
    path::PathBuf,
    sync::Mutex,
};

use log_kv::LogKv;
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

const SNAPSHOT_CATALOG_KEY: &str = "__catalog__";
const SNAPSHOT_KEY_PREFIX: &str = "snapshot:";
const TOMBSTONE: &str = "__deleted__";

pub struct LogKvSnapshotStore {
    path: PathBuf,
    write_lock: Mutex<()>,
}

impl LogKvSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_LOG_KV_PATH cannot be empty when SNAPSHOT_STORE=log_kv".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            ensure_snapshot_dir(parent)?;
        }

        Self::open_database_at(&path)?;

        Ok(Self {
            path,
            write_lock: Mutex::new(()),
        })
    }

    fn snapshot_key(doc_id: &Uuid) -> String {
        format!("{SNAPSHOT_KEY_PREFIX}{doc_id}")
    }

    fn open_database(&self) -> Result<LogKv<String, String, File>, StorageError> {
        Self::open_database_at(&self.path)
    }

    fn open_database_at(path: &PathBuf) -> Result<LogKv<String, String, File>, StorageError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        LogKv::create(file)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))
    }

    fn append_value(&self, key: String, value: String) -> Result<(), StorageError> {
        let _guard = self.write_lock.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: log_kv mutex was poisoned",
                self.path.display()
            ))
        })?;
        let mut database = self.open_database()?;

        database
            .put(key, value)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        self.sync_file()
    }

    fn sync_file(&self) -> Result<(), StorageError> {
        let file = File::open(&self.path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        file.sync_all()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn read_catalog(&self) -> Result<Vec<Uuid>, StorageError> {
        let mut database = self.open_database()?;
        let Some(payload) = database
            .get(SNAPSHOT_CATALOG_KEY.to_owned())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?
        else {
            return Ok(Vec::new());
        };

        serde_json::from_str::<Vec<Uuid>>(&payload).map_err(|_| {
            StorageError::Io(format!(
                "{}: log_kv snapshot catalog is corrupt",
                self.path.display()
            ))
        })
    }

    fn write_catalog(&self, catalog: &[Uuid]) -> Result<(), StorageError> {
        let payload = serde_json::to_string(catalog).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize log_kv snapshot catalog: {error}"
            ))
        })?;

        self.append_value(SNAPSHOT_CATALOG_KEY.to_owned(), payload)
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

impl SnapshotStore for LogKvSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let mut database = self.open_database()?;
        let Some(payload) = database
            .get(Self::snapshot_key(doc_id))
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?
        else {
            return Ok(None);
        };

        if payload == TOMBSTONE {
            return Ok(None);
        }

        self.deserialize_snapshot(*doc_id, &payload).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload =
            serde_json::to_string(&PersistedSnapshot::from(snapshot)).map_err(|error| {
                StorageError::Io(format!(
                    "failed to serialize log_kv snapshot `{doc_id}`: {error}"
                ))
            })?;

        self.append_value(Self::snapshot_key(&doc_id), payload)?;

        let mut catalog = self.read_catalog()?;
        if !catalog.contains(&doc_id) {
            catalog.push(doc_id);
            catalog.sort_unstable();
            self.write_catalog(&catalog)?;
        }

        Ok(())
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        self.append_value(Self::snapshot_key(doc_id), TOMBSTONE.to_owned())?;

        let mut catalog = self.read_catalog()?;
        let original_len = catalog.len();
        catalog.retain(|catalog_doc_id| catalog_doc_id != doc_id);
        if catalog.len() != original_len {
            self.write_catalog(&catalog)?;
        }

        Ok(())
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let catalog = self.read_catalog()?;
        let mut documents = Vec::new();

        for doc_id in catalog {
            match self.load_snapshot(&doc_id)? {
                Some(snapshot) => documents.push(snapshot.document),
                None => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing log_kv snapshot referenced by catalog"
                ),
            }
        }

        Ok(documents)
    }
}
