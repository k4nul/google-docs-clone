use std::{path::PathBuf, sync::Mutex};

use mmdb::{DB, DbOptions, WriteBatch, WriteOptions};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

const SNAPSHOT_CATALOG_KEY: &[u8] = b"__catalog__";
const SNAPSHOT_KEY_PREFIX: &str = "snapshot:";

pub struct MmdbSnapshotStore {
    path: PathBuf,
    db: Mutex<DB>,
    write_options: WriteOptions,
}

impl MmdbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_MMDB_PATH cannot be empty when SNAPSHOT_STORE=mmdb".to_owned(),
            ));
        }

        ensure_snapshot_dir(&path)?;
        let db = DB::open(
            DbOptions {
                create_if_missing: true,
                ..Default::default()
            },
            &path,
        )
        .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            db: Mutex::new(db),
            write_options: WriteOptions {
                sync: true,
                ..Default::default()
            },
        })
    }

    fn snapshot_key(doc_id: &Uuid) -> Vec<u8> {
        format!("{SNAPSHOT_KEY_PREFIX}{doc_id}").into_bytes()
    }

    fn lock_db(&self) -> Result<std::sync::MutexGuard<'_, DB>, StorageError> {
        self.db.lock().map_err(|_| {
            StorageError::Io(format!("{}: mmdb mutex was poisoned", self.path.display()))
        })
    }

    fn map_error(&self, operation: &str, error: impl std::fmt::Display) -> StorageError {
        StorageError::Io(format!(
            "{}: failed to {operation}: {error}",
            self.path.display()
        ))
    }

    fn read_catalog(&self, db: &DB) -> Result<Vec<Uuid>, StorageError> {
        let Some(payload) = db
            .get(SNAPSHOT_CATALOG_KEY)
            .map_err(|error| self.map_error("read mmdb snapshot catalog", error))?
        else {
            return Ok(Vec::new());
        };

        serde_json::from_slice::<Vec<Uuid>>(&payload).map_err(|_| {
            StorageError::Io(format!(
                "{}: mmdb snapshot catalog is corrupt",
                self.path.display()
            ))
        })
    }

    fn serialize_catalog(&self, catalog: &[Uuid]) -> Result<Vec<u8>, StorageError> {
        serde_json::to_vec(catalog).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize mmdb snapshot catalog: {error}"
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
}

impl SnapshotStore for MmdbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let db = self.lock_db()?;
        let Some(payload) = db
            .get(&Self::snapshot_key(doc_id))
            .map_err(|error| self.map_error("read mmdb snapshot", error))?
        else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &payload).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize mmdb snapshot `{doc_id}`: {error}"
            ))
        })?;

        let db = self.lock_db()?;
        let mut catalog = self.read_catalog(&db)?;
        let mut batch = WriteBatch::new();
        batch.put(&Self::snapshot_key(&doc_id), &payload);

        if !catalog.contains(&doc_id) {
            catalog.push(doc_id);
            catalog.sort_unstable();
            batch.put(SNAPSHOT_CATALOG_KEY, &self.serialize_catalog(&catalog)?);
        }

        db.write_with_options(batch, &self.write_options)
            .map_err(|error| self.map_error("write mmdb snapshot", error))?;
        db.flush()
            .map_err(|error| self.map_error("flush mmdb snapshot store", error))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let db = self.lock_db()?;
        let mut catalog = self.read_catalog(&db)?;
        let original_len = catalog.len();
        catalog.retain(|catalog_doc_id| catalog_doc_id != doc_id);

        let mut batch = WriteBatch::new();
        batch.delete(&Self::snapshot_key(doc_id));
        if catalog.len() != original_len {
            batch.put(SNAPSHOT_CATALOG_KEY, &self.serialize_catalog(&catalog)?);
        }

        db.write_with_options(batch, &self.write_options)
            .map_err(|error| self.map_error("delete mmdb snapshot", error))?;
        db.flush()
            .map_err(|error| self.map_error("flush mmdb snapshot store", error))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let db = self.lock_db()?;
        let catalog = self.read_catalog(&db)?;
        let mut documents = Vec::new();

        for doc_id in catalog {
            match db.get(&Self::snapshot_key(&doc_id)) {
                Ok(Some(payload)) => match self.deserialize_snapshot(doc_id, &payload) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping corrupt mmdb snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                },
                Ok(None) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing mmdb snapshot referenced by catalog"
                ),
                Err(error) => return Err(self.map_error("read mmdb snapshot", error)),
            }
        }

        Ok(documents)
    }
}
