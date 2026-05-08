use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqjson::{DbError, YourDb};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOT_KEY_PREFIX: &str = "snapshot:";
const SNAPSHOT_META_SUFFIX: &str = ":meta";
const CHUNK_SIZE: usize = 2048;

#[derive(Debug, Serialize, Deserialize)]
struct BlobMeta {
    version: Uuid,
    chunk_count: usize,
}

pub struct SqjsonSnapshotStore {
    path: PathBuf,
    database: Mutex<YourDb>,
}

impl SqjsonSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = normalize_path(path.into())?;
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let path_str = path_to_str(&path)?;
        let database =
            YourDb::open(path_str).map_err(|error| map_sqjson_error(&path, "open", error))?;

        Ok(Self {
            path,
            database: Mutex::new(database),
        })
    }

    fn lock_database(&self) -> Result<MutexGuard<'_, YourDb>, StorageError> {
        self.database.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: sqjson mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn meta_key(name: &str) -> String {
        format!("{name}{SNAPSHOT_META_SUFFIX}")
    }

    fn chunk_key(name: &str, version: &Uuid, index: usize) -> String {
        format!("{name}:chunk:{version}:{index:08}")
    }

    fn snapshot_key(doc_id: &Uuid) -> String {
        format!("{SNAPSHOT_KEY_PREFIX}{doc_id}")
    }

    fn doc_id_from_meta_key(key: &str) -> Option<Uuid> {
        let doc_id = key
            .strip_prefix(SNAPSHOT_KEY_PREFIX)?
            .strip_suffix(SNAPSHOT_META_SUFFIX)?;
        Uuid::parse_str(doc_id).ok()
    }

    fn get_meta(&self, database: &YourDb, name: &str) -> Result<Option<BlobMeta>, StorageError> {
        let Some(value) = database
            .get(&Self::meta_key(name))
            .map_err(|error| map_sqjson_error(&self.path, "read blob metadata", error))?
        else {
            return Ok(None);
        };

        serde_json::from_value(value).map(Some).map_err(|error| {
            StorageError::Io(format!(
                "{}: sqjson blob `{name}` metadata is corrupt: {error}",
                self.path.display()
            ))
        })
    }

    fn load_blob(&self, database: &YourDb, name: &str) -> Result<Option<Vec<u8>>, StorageError> {
        let Some(meta) = self.get_meta(database, name)? else {
            return Ok(None);
        };

        let mut payload = Vec::with_capacity(meta.chunk_count * CHUNK_SIZE);
        for index in 0..meta.chunk_count {
            let key = Self::chunk_key(name, &meta.version, index);
            let Some(value) = database
                .get(&key)
                .map_err(|error| map_sqjson_error(&self.path, "read blob chunk", error))?
            else {
                return Err(StorageError::Io(format!(
                    "{}: sqjson blob `{name}` is missing chunk {index}",
                    self.path.display()
                )));
            };

            let Some(chunk) = value.as_str() else {
                return Err(StorageError::Io(format!(
                    "{}: sqjson blob `{name}` chunk {index} is not a string",
                    self.path.display()
                )));
            };
            let chunk = STANDARD.decode(chunk).map_err(|error| {
                StorageError::Io(format!(
                    "{}: sqjson blob `{name}` chunk {index} is not valid base64: {error}",
                    self.path.display()
                ))
            })?;
            payload.extend(chunk);
        }

        Ok(Some(payload))
    }

    fn save_blob(
        &self,
        database: &mut YourDb,
        name: &str,
        payload: &[u8],
    ) -> Result<(), StorageError> {
        let previous_meta = self.get_meta(database, name)?;
        let version = Uuid::new_v4();
        let chunk_count = payload.chunks(CHUNK_SIZE).count();

        for (index, chunk) in payload.chunks(CHUNK_SIZE).enumerate() {
            database
                .put(
                    &Self::chunk_key(name, &version, index),
                    &json!(STANDARD.encode(chunk)),
                )
                .map_err(|error| map_sqjson_error(&self.path, "write blob chunk", error))?;
        }

        let meta = serde_json::to_value(BlobMeta {
            version,
            chunk_count,
        })
        .map_err(|error| {
            StorageError::Io(format!(
                "{}: failed to encode sqjson blob metadata: {error}",
                self.path.display()
            ))
        })?;
        database
            .put(&Self::meta_key(name), &meta)
            .map_err(|error| map_sqjson_error(&self.path, "write blob metadata", error))?;

        if let Some(previous_meta) = previous_meta {
            for index in 0..previous_meta.chunk_count {
                delete_if_present(
                    database,
                    &Self::chunk_key(name, &previous_meta.version, index),
                )
                .map_err(|error| map_sqjson_error(&self.path, "delete stale blob chunk", error))?;
            }
        }

        self.flush_and_sync(database)
    }

    fn delete_blob(&self, database: &mut YourDb, name: &str) -> Result<(), StorageError> {
        let Some(meta) = self.get_meta(database, name)? else {
            return Ok(());
        };

        delete_if_present(database, &Self::meta_key(name))
            .map_err(|error| map_sqjson_error(&self.path, "delete blob metadata", error))?;

        for index in 0..meta.chunk_count {
            delete_if_present(database, &Self::chunk_key(name, &meta.version, index))
                .map_err(|error| map_sqjson_error(&self.path, "delete blob chunk", error))?;
        }

        self.flush_and_sync(database)
    }

    fn flush_and_sync(&self, database: &mut YourDb) -> Result<(), StorageError> {
        database
            .flush()
            .map_err(|error| map_sqjson_error(&self.path, "flush", error))?;
        sync_file(&self.path)?;
        sync_parent_dir(&self.path)
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

impl SnapshotStore for SqjsonSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let database = self.lock_database()?;
        let Some(payload) = self.load_blob(&database, &Self::snapshot_key(doc_id))? else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &payload).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize sqjson snapshot `{doc_id}`: {error}"
            ))
        })?;

        let mut database = self.lock_database()?;
        self.save_blob(&mut database, &Self::snapshot_key(&doc_id), &payload)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut database = self.lock_database()?;
        self.delete_blob(&mut database, &Self::snapshot_key(doc_id))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let database = self.lock_database()?;
        let mut keys = database.list_keys();
        keys.sort();
        let mut documents = Vec::new();

        for key in keys {
            let Some(doc_id) = Self::doc_id_from_meta_key(&key) else {
                continue;
            };
            let Some(payload) = self.load_blob(&database, &Self::snapshot_key(&doc_id))? else {
                continue;
            };

            match self.deserialize_snapshot(doc_id, &payload) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt sqjson snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        documents.sort_unstable_by_key(|document| document.id);
        Ok(documents)
    }
}

fn delete_if_present(database: &mut YourDb, key: &str) -> Result<(), DbError> {
    if database.get(key)?.is_some() {
        database.delete(key)?;
    }
    Ok(())
}

fn normalize_path(path: PathBuf) -> Result<PathBuf, StorageError> {
    if path.as_os_str().is_empty() {
        return Err(StorageError::Config(
            "SNAPSHOT_SQJSON_PATH cannot be empty when SNAPSHOT_STORE=sqjson".to_owned(),
        ));
    }

    if path
        .parent()
        .is_some_and(|parent| !parent.as_os_str().is_empty())
    {
        Ok(path)
    } else {
        Ok(PathBuf::from(".").join(path))
    }
}

fn path_to_str(path: &Path) -> Result<&str, StorageError> {
    path.to_str().ok_or_else(|| {
        StorageError::Config(
            "SNAPSHOT_SQJSON_PATH must be valid unicode when SNAPSHOT_STORE=sqjson".to_owned(),
        )
    })
}

fn map_sqjson_error(path: &Path, operation: &str, error: DbError) -> StorageError {
    StorageError::Io(format!(
        "{}: failed to {operation} sqjson snapshot store: {error}",
        path.display()
    ))
}

fn sync_file(path: &Path) -> Result<(), StorageError> {
    File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))
}

fn sync_parent_dir(path: &Path) -> Result<(), StorageError> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));

    File::open(parent)
        .and_then(|file| file.sync_all())
        .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))
}
