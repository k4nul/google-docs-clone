use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use mhdb::Db;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const CATALOG_KEY: &str = "__catalog__";
const CHUNK_SIZE: usize = 384;

#[derive(Debug, Serialize, Deserialize)]
struct BlobMeta {
    chunk_count: usize,
}

pub struct MhdbSnapshotStore {
    path: PathBuf,
    database: Mutex<Db>,
}

impl MhdbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_MHDB_PATH cannot be empty when SNAPSHOT_STORE=mhdb".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let database = Db::open(path.to_string_lossy().as_ref())
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            database: Mutex::new(database),
        })
    }

    fn lock_database(&self) -> Result<MutexGuard<'_, Db>, StorageError> {
        self.database.lock().map_err(|_| {
            StorageError::Io(format!("{}: mhdb mutex was poisoned", self.path.display()))
        })
    }

    fn map_error(&self, operation: &str, error: impl std::fmt::Debug) -> StorageError {
        StorageError::Io(format!(
            "{}: failed to {operation}: {error:?}",
            self.path.display()
        ))
    }

    fn meta_key(name: &str) -> String {
        format!("{name}:meta")
    }

    fn chunk_key(name: &str, index: usize) -> String {
        format!("{name}:chunk:{index:08}")
    }

    fn load_blob(&self, database: &mut Db, name: &str) -> Result<Option<Vec<u8>>, StorageError> {
        let meta_bytes = database
            .fetch::<Vec<u8>>(Self::meta_key(name))
            .map_err(|error| self.map_error("read mhdb blob metadata", error))?;
        let Some(meta_bytes) = meta_bytes else {
            return Ok(None);
        };

        let meta = serde_json::from_slice::<BlobMeta>(&meta_bytes)
            .map_err(|error| self.map_error("decode mhdb blob metadata", error))?;
        let mut payload = Vec::with_capacity(meta.chunk_count * CHUNK_SIZE);

        for index in 0..meta.chunk_count {
            let chunk = database
                .fetch::<Vec<u8>>(Self::chunk_key(name, index))
                .map_err(|error| self.map_error("read mhdb blob chunk", error))?
                .ok_or_else(|| {
                    StorageError::Io(format!(
                        "{}: mhdb blob `{name}` is missing chunk {index}",
                        self.path.display()
                    ))
                })?;
            payload.extend(chunk);
        }

        Ok(Some(payload))
    }

    fn save_blob(&self, database: &mut Db, name: &str, payload: &[u8]) -> Result<(), StorageError> {
        self.delete_blob(database, name)?;

        let meta = BlobMeta {
            chunk_count: payload.chunks(CHUNK_SIZE).count(),
        };
        let meta_bytes = serde_json::to_vec(&meta)
            .map_err(|error| self.map_error("encode mhdb blob metadata", error))?;

        database
            .store(Self::meta_key(name), meta_bytes)
            .map_err(|error| self.map_error("write mhdb blob metadata", error))?;

        for (index, chunk) in payload.chunks(CHUNK_SIZE).enumerate() {
            database
                .store(Self::chunk_key(name, index), chunk.to_vec())
                .map_err(|error| self.map_error("write mhdb blob chunk", error))?;
        }

        Ok(())
    }

    fn delete_blob(&self, database: &mut Db, name: &str) -> Result<(), StorageError> {
        let meta_bytes = database
            .fetch::<Vec<u8>>(Self::meta_key(name))
            .map_err(|error| self.map_error("read mhdb blob metadata before delete", error))?;

        if let Some(meta_bytes) = meta_bytes {
            let meta = serde_json::from_slice::<BlobMeta>(&meta_bytes).map_err(|error| {
                self.map_error("decode mhdb blob metadata before delete", error)
            })?;
            for index in 0..meta.chunk_count {
                database
                    .delete(Self::chunk_key(name, index))
                    .map_err(|error| self.map_error("delete mhdb blob chunk", error))?;
            }
        }

        database
            .delete(Self::meta_key(name))
            .map_err(|error| self.map_error("delete mhdb blob metadata", error))?;
        Ok(())
    }

    fn snapshot_key(doc_id: &Uuid) -> String {
        format!("snapshot:{doc_id}")
    }

    fn load_catalog(&self, database: &mut Db) -> Result<Vec<Uuid>, StorageError> {
        let Some(payload) = self.load_blob(database, CATALOG_KEY)? else {
            return Ok(Vec::new());
        };

        serde_json::from_slice::<Vec<Uuid>>(&payload)
            .map_err(|error| self.map_error("decode mhdb catalog", error))
    }

    fn save_catalog(&self, database: &mut Db, documents: &[Uuid]) -> Result<(), StorageError> {
        let payload = serde_json::to_vec(documents)
            .map_err(|error| self.map_error("encode mhdb catalog", error))?;
        self.save_blob(database, CATALOG_KEY, &payload)
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

impl SnapshotStore for MhdbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let mut database = self.lock_database()?;
        let Some(payload) = self.load_blob(&mut database, &Self::snapshot_key(doc_id))? else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &payload).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize mhdb snapshot `{doc_id}`: {error}"
            ))
        })?;

        let mut database = self.lock_database()?;
        self.save_blob(&mut database, &Self::snapshot_key(&doc_id), &payload)?;

        let mut catalog = self.load_catalog(&mut database)?;
        catalog.push(doc_id);
        catalog.sort_unstable();
        catalog.dedup();
        self.save_catalog(&mut database, &catalog)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut database = self.lock_database()?;
        self.delete_blob(&mut database, &Self::snapshot_key(doc_id))?;

        let mut catalog = self.load_catalog(&mut database)?;
        catalog.retain(|catalog_doc_id| catalog_doc_id != doc_id);
        self.save_catalog(&mut database, &catalog)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut database = self.lock_database()?;
        let catalog = self.load_catalog(&mut database)?;
        let mut documents = Vec::new();

        for doc_id in catalog {
            let Some(payload) = self.load_blob(&mut database, &Self::snapshot_key(&doc_id))? else {
                continue;
            };

            match self.deserialize_snapshot(doc_id, &payload) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt mhdb snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        documents.sort_unstable_by_key(|document| document.id);
        Ok(documents)
    }
}
