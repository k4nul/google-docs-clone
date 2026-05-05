use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use infusedb::{DataType, InfuseDB};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOT_COLLECTION: &str = "snapshots";
const SNAPSHOT_CATALOG_KEY: &str = "__catalog__";
const SNAPSHOT_KEY_PREFIX: &str = "snapshot:";

pub struct InfusedbSnapshotStore {
    path: PathBuf,
    database: Mutex<InfuseDB>,
}

impl InfusedbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_INFUSEDB_PATH cannot be empty when SNAPSHOT_STORE=infusedb".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let mut database = if path.exists() {
            InfuseDB::load(&Self::path_string(&path))
                .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?
        } else {
            let mut database = InfuseDB::new();
            database.path = Self::path_string(&path);
            database
        };

        Self::ensure_collection(&mut database, &path)?;
        database
            .dump()
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            database: Mutex::new(database),
        })
    }

    fn path_string(path: &Path) -> String {
        path.to_string_lossy().into_owned()
    }

    fn snapshot_key(doc_id: &Uuid) -> String {
        format!("{SNAPSHOT_KEY_PREFIX}{doc_id}")
    }

    fn ensure_collection(database: &mut InfuseDB, path: &Path) -> Result<(), StorageError> {
        if database.get_collection(SNAPSHOT_COLLECTION).is_none() {
            database
                .create_collection(SNAPSHOT_COLLECTION)
                .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;
        }

        Ok(())
    }

    fn lock_database(&self) -> Result<MutexGuard<'_, InfuseDB>, StorageError> {
        self.database.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: infusedb snapshot store lock poisoned",
                self.path.display()
            ))
        })
    }

    fn read_text(database: &mut InfuseDB, key: &str) -> Option<String> {
        let value = database.get_collection(SNAPSHOT_COLLECTION)?.get(key)?;
        match value {
            DataType::Text(text) => Some(text.clone()),
            _ => None,
        }
    }

    fn write_text(database: &mut InfuseDB, key: &str, value: String) -> Result<(), StorageError> {
        let collection = database
            .get_collection(SNAPSHOT_COLLECTION)
            .ok_or_else(|| {
                StorageError::Io("infusedb snapshots collection is missing".to_owned())
            })?;
        collection.add(key, DataType::from(value));
        Ok(())
    }

    fn remove_key(database: &mut InfuseDB, key: &str) -> Result<(), StorageError> {
        let collection = database
            .get_collection(SNAPSHOT_COLLECTION)
            .ok_or_else(|| {
                StorageError::Io("infusedb snapshots collection is missing".to_owned())
            })?;
        collection.rm(key);
        Ok(())
    }

    fn flush(&self, database: &InfuseDB) -> Result<(), StorageError> {
        database
            .dump()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn decode_payload(&self, key: &str, payload: &str) -> Result<Vec<u8>, StorageError> {
        STANDARD.decode(payload).map_err(|error| {
            StorageError::Io(format!(
                "{}: invalid infusedb base64 payload `{key}`: {error}",
                self.path.display()
            ))
        })
    }

    fn read_catalog(&self, database: &mut InfuseDB) -> Result<Vec<Uuid>, StorageError> {
        let Some(payload) = Self::read_text(database, SNAPSHOT_CATALOG_KEY) else {
            return Ok(Vec::new());
        };
        let payload = self.decode_payload(SNAPSHOT_CATALOG_KEY, &payload)?;

        serde_json::from_slice::<Vec<Uuid>>(&payload).map_err(|_| {
            StorageError::Io(format!(
                "{}: infusedb snapshot catalog is corrupt",
                self.path.display()
            ))
        })
    }

    fn write_catalog(&self, database: &mut InfuseDB, catalog: &[Uuid]) -> Result<(), StorageError> {
        let payload = serde_json::to_vec(catalog).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize infusedb snapshot catalog: {error}"
            ))
        })?;
        Self::write_text(database, SNAPSHOT_CATALOG_KEY, STANDARD.encode(payload))
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

impl SnapshotStore for InfusedbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let mut database = self.lock_database()?;
        let key = Self::snapshot_key(doc_id);
        let Some(payload) = Self::read_text(&mut database, &key) else {
            return Ok(None);
        };
        let payload = self.decode_payload(&key, &payload)?;

        self.deserialize_snapshot(*doc_id, &payload).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize infusedb snapshot `{doc_id}`: {error}"
            ))
        })?;

        let mut database = self.lock_database()?;
        let mut catalog = self.read_catalog(&mut database)?;
        Self::write_text(
            &mut database,
            &Self::snapshot_key(&doc_id),
            STANDARD.encode(payload),
        )?;

        if !catalog.contains(&doc_id) {
            catalog.push(doc_id);
            catalog.sort_unstable();
            self.write_catalog(&mut database, &catalog)?;
        }

        self.flush(&database)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut database = self.lock_database()?;
        let mut catalog = self.read_catalog(&mut database)?;

        Self::remove_key(&mut database, &Self::snapshot_key(doc_id))?;
        let original_len = catalog.len();
        catalog.retain(|catalog_doc_id| catalog_doc_id != doc_id);
        if catalog.len() != original_len {
            self.write_catalog(&mut database, &catalog)?;
        }

        self.flush(&database)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut database = self.lock_database()?;
        let catalog = self.read_catalog(&mut database)?;
        let mut documents = Vec::new();

        for doc_id in catalog {
            let key = Self::snapshot_key(&doc_id);
            let Some(payload) = Self::read_text(&mut database, &key) else {
                tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing infusedb snapshot referenced by catalog"
                );
                continue;
            };
            let payload = self.decode_payload(&key, &payload)?;

            match self.deserialize_snapshot(doc_id, &payload) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(_)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt infusedb snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
