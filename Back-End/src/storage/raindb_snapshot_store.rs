use std::path::PathBuf;

use raindb::{Batch, DB, DbOptions, RainDBError, ReadOptions, WriteOptions};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

const SNAPSHOT_CATALOG_KEY: &[u8] = b"__catalog__";
const SNAPSHOT_KEY_PREFIX: &str = "snapshot:";

pub struct RaindbSnapshotStore {
    path: PathBuf,
    db: DB,
}

impl RaindbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_RAINDB_PATH cannot be empty when SNAPSHOT_STORE=raindb".to_owned(),
            ));
        }

        ensure_snapshot_dir(&path)?;

        let db = DB::open(DbOptions {
            db_path: path.to_string_lossy().into_owned(),
            create_if_missing: true,
            ..DbOptions::default()
        })
        .map_err(|error| Self::map_open_error(&path, error))?;

        Ok(Self { path, db })
    }

    fn snapshot_key(doc_id: &Uuid) -> Vec<u8> {
        format!("{SNAPSHOT_KEY_PREFIX}{doc_id}").into_bytes()
    }

    fn write_options() -> WriteOptions {
        WriteOptions { synchronous: true }
    }

    fn map_open_error(path: &PathBuf, error: RainDBError) -> StorageError {
        StorageError::Io(format!("{}: {error}", path.display()))
    }

    fn map_error(&self, action: &str, error: RainDBError) -> StorageError {
        StorageError::Io(format!("{}: {action} failed: {error}", self.path.display()))
    }

    fn read_value(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        match self.db.get(ReadOptions::default(), key) {
            Ok(payload) => Ok(Some(payload)),
            Err(RainDBError::KeyNotFound) => Ok(None),
            Err(error) => Err(self.map_error("read raindb snapshot value", error)),
        }
    }

    fn read_catalog(&self) -> Result<Vec<Uuid>, StorageError> {
        let Some(payload) = self.read_value(SNAPSHOT_CATALOG_KEY)? else {
            return Ok(Vec::new());
        };

        serde_json::from_slice::<Vec<Uuid>>(&payload).map_err(|_| {
            StorageError::Io(format!(
                "{}: raindb snapshot catalog is corrupt",
                self.path.display()
            ))
        })
    }

    fn serialize_catalog(catalog: &[Uuid]) -> Result<Vec<u8>, StorageError> {
        serde_json::to_vec(catalog).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize raindb snapshot catalog: {error}"
            ))
        })
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        payload: &[u8],
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot = serde_json::from_slice::<PersistedSnapshot>(payload)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot: DocumentSnapshot = snapshot.into();

        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }

    fn write_batch(&self, batch: Batch) -> Result<(), StorageError> {
        self.db
            .apply(Self::write_options(), batch)
            .map_err(|error| self.map_error("write raindb snapshot batch", error))
    }
}

impl SnapshotStore for RaindbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let Some(payload) = self.read_value(&Self::snapshot_key(doc_id))? else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &payload).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize raindb snapshot `{doc_id}`: {error}"
            ))
        })?;

        let mut catalog = self.read_catalog()?;
        let mut batch = Batch::new();
        batch.add_put(Self::snapshot_key(&doc_id), payload);
        if !catalog.contains(&doc_id) {
            catalog.push(doc_id);
            catalog.sort_unstable();
            batch.add_put(
                SNAPSHOT_CATALOG_KEY.to_vec(),
                Self::serialize_catalog(&catalog)?,
            );
        }

        self.write_batch(batch)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut catalog = self.read_catalog()?;
        let original_len = catalog.len();
        catalog.retain(|catalog_doc_id| catalog_doc_id != doc_id);

        let mut batch = Batch::new();
        batch.add_delete(Self::snapshot_key(doc_id));
        if catalog.len() != original_len {
            batch.add_put(
                SNAPSHOT_CATALOG_KEY.to_vec(),
                Self::serialize_catalog(&catalog)?,
            );
        }

        self.write_batch(batch)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let catalog = self.read_catalog()?;
        let mut documents = Vec::new();

        for doc_id in catalog {
            match self.load_snapshot(&doc_id) {
                Ok(Some(snapshot)) => documents.push(snapshot.document),
                Ok(None) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing raindb snapshot referenced by catalog"
                ),
                Err(StorageError::CorruptSnapshot(_)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt raindb snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
