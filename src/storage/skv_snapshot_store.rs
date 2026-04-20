use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
};

use serde::{Deserialize, Serialize};
use skv::KeyValueStore;
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOT_CATALOG_KEY: &str = "__catalog__";

#[derive(Debug, Clone, Serialize, Deserialize)]
enum SkvRecord {
    Catalog(Vec<String>),
    Snapshot(PersistedSnapshot),
}

pub struct SkvSnapshotStore {
    path: PathBuf,
    data_path: PathBuf,
    index_path: PathBuf,
    store: Mutex<KeyValueStore<SkvRecord>>,
}

impl SkvSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_SKV_PATH cannot be empty when SNAPSHOT_STORE=skv".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let data_path = PathBuf::from(format!("{}.data", path.display()));
        let index_path = PathBuf::from(format!("{}.index", path.display()));
        let store = Self::open_store(&data_path, &index_path)?;

        Ok(Self {
            path,
            data_path,
            index_path,
            store: Mutex::new(store),
        })
    }

    fn open_store(
        data_path: &Path,
        index_path: &Path,
    ) -> Result<KeyValueStore<SkvRecord>, StorageError> {
        let data_exists = data_path.exists();
        let index_exists = index_path.exists();

        let data_len = data_path
            .metadata()
            .map(|metadata| metadata.len())
            .unwrap_or(0);
        let index_len = index_path
            .metadata()
            .map(|metadata| metadata.len())
            .unwrap_or(0);

        match (data_exists, index_exists) {
            (false, false) => Self::create_store(data_path, index_path),
            (true, true) if data_len == 0 && index_len == 0 => {
                Self::create_store(data_path, index_path)
            }
            (true, true) if index_len > 0 => Self::load_store(data_path, index_path),
            (true, true) => Err(StorageError::Io(format!(
                "{}: skv index is empty while data file is not reopenable",
                index_path.display()
            ))),
            _ => Err(StorageError::Io(format!(
                "{} / {}: skv data and index files must either both exist or both be absent",
                data_path.display(),
                index_path.display()
            ))),
        }
    }

    fn create_store(
        data_path: &Path,
        index_path: &Path,
    ) -> Result<KeyValueStore<SkvRecord>, StorageError> {
        KeyValueStore::new(data_path, index_path).map_err(|error| {
            StorageError::Io(format!(
                "{} / {}: failed to initialize skv snapshot store: {error}",
                data_path.display(),
                index_path.display()
            ))
        })
    }

    fn load_store(
        data_path: &Path,
        index_path: &Path,
    ) -> Result<KeyValueStore<SkvRecord>, StorageError> {
        KeyValueStore::load(data_path, index_path).map_err(|error| {
            StorageError::Io(format!(
                "{} / {}: failed to reopen skv snapshot store: {error}",
                data_path.display(),
                index_path.display()
            ))
        })
    }

    fn lock_store(&self) -> Result<MutexGuard<'_, KeyValueStore<SkvRecord>>, StorageError> {
        self.store.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: skv snapshot store mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn sync_store(&self, store: &KeyValueStore<SkvRecord>) -> Result<(), StorageError> {
        store.sync().map_err(|error| {
            StorageError::Io(format!(
                "{} / {}: failed to sync skv snapshot store: {error}",
                self.data_path.display(),
                self.index_path.display()
            ))
        })
    }

    fn load_catalog(&self, store: &KeyValueStore<SkvRecord>) -> Result<Vec<String>, StorageError> {
        match store.get(SNAPSHOT_CATALOG_KEY).map_err(|error| {
            StorageError::Io(format!(
                "{}: failed to read skv snapshot catalog: {error}",
                self.path.display()
            ))
        })? {
            Some(SkvRecord::Catalog(catalog)) => Ok(catalog),
            Some(SkvRecord::Snapshot(_)) => Err(StorageError::Io(format!(
                "{}: skv snapshot catalog is corrupt",
                self.path.display()
            ))),
            None => Ok(Vec::new()),
        }
    }

    fn save_catalog(
        &self,
        store: &KeyValueStore<SkvRecord>,
        catalog: Vec<String>,
    ) -> Result<(), StorageError> {
        store
            .insert(SNAPSHOT_CATALOG_KEY.to_owned(), SkvRecord::Catalog(catalog))
            .map_err(|error| {
                StorageError::Io(format!(
                    "{}: failed to write skv snapshot catalog: {error}",
                    self.path.display()
                ))
            })?;
        self.sync_store(store)
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        record: SkvRecord,
    ) -> Result<DocumentSnapshot, StorageError> {
        let SkvRecord::Snapshot(snapshot) = record else {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        };

        let snapshot: DocumentSnapshot = snapshot.into();
        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }
}

impl SnapshotStore for SkvSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let store = self.lock_store()?;
        let key = doc_id.to_string();
        let record = store.get(&key).map_err(|error| {
            StorageError::Io(format!(
                "{}: failed to read skv snapshot: {error}",
                self.path.display()
            ))
        })?;
        let Some(record) = record else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, record).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let store = self.lock_store()?;
        let doc_id = snapshot.document.id;
        let key = doc_id.to_string();
        let mut catalog = self.load_catalog(&store)?;

        store
            .insert(
                key.clone(),
                SkvRecord::Snapshot(PersistedSnapshot::from(snapshot)),
            )
            .map_err(|error| {
                StorageError::Io(format!(
                    "{}: failed to write skv snapshot `{doc_id}`: {error}",
                    self.path.display()
                ))
            })?;

        if !catalog.iter().any(|entry| entry == &key) {
            catalog.push(key);
            catalog.sort();
        }

        self.save_catalog(&store, catalog)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let store = self.lock_store()?;
        let key = doc_id.to_string();
        let mut catalog = self.load_catalog(&store)?;

        if catalog.iter().any(|entry| entry == &key) {
            store.delete(&key).map_err(|error| {
                StorageError::Io(format!(
                    "{}: failed to delete skv snapshot `{doc_id}`: {error}",
                    self.path.display()
                ))
            })?;
            catalog.retain(|entry| entry != &key);
            self.save_catalog(&store, catalog)?;
        }

        Ok(())
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let store = self.lock_store()?;
        let catalog = self.load_catalog(&store)?;
        let mut documents = Vec::new();

        for doc_id_key in catalog {
            let Ok(doc_id) = Uuid::parse_str(&doc_id_key) else {
                continue;
            };

            match store.get(&doc_id_key).map_err(|error| {
                StorageError::Io(format!(
                    "{}: failed to read skv snapshot catalog entry: {error}",
                    self.path.display()
                ))
            })? {
                Some(record) => match self.deserialize_snapshot(doc_id, record) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping corrupt skv snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                },
                None => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing skv snapshot while building document catalog"
                ),
            }
        }

        Ok(documents)
    }
}
