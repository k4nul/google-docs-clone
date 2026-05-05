use std::{
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
};

use dir_cache::{
    DirCache,
    opts::{CacheOpenOptions, DirCacheOpts, DirOpenOpt},
};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const CATALOG_KEY: &str = "__catalog__";

pub struct DirCacheSnapshotStore {
    path: PathBuf,
    cache: Mutex<DirCache>,
}

impl DirCacheSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_DIR_CACHE_PATH cannot be empty when SNAPSHOT_STORE=dir_cache".to_owned(),
            ));
        }

        let cache = DirCacheOpts::default()
            .open(
                &path,
                CacheOpenOptions::new(DirOpenOpt::CreateIfMissing, false),
            )
            .map_err(|error| map_dir_cache_error(&path, error))?;

        Ok(Self {
            path,
            cache: Mutex::new(cache),
        })
    }

    fn lock_cache(&self) -> Result<MutexGuard<'_, DirCache>, StorageError> {
        self.cache.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: dir-cache mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn snapshot_key(doc_id: &Uuid) -> PathBuf {
        PathBuf::from(format!("snapshot-{doc_id}.json"))
    }

    fn load_catalog(&self, cache: &mut DirCache) -> Result<Vec<Uuid>, StorageError> {
        let Some(bytes) = cache
            .get(Path::new(CATALOG_KEY))
            .map_err(|error| map_dir_cache_error(&self.path, error))?
        else {
            return Ok(Vec::new());
        };

        serde_json::from_slice(bytes.as_ref())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn save_catalog(&self, cache: &mut DirCache, catalog: &[Uuid]) -> Result<(), StorageError> {
        let bytes = serde_json::to_vec(catalog)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        cache
            .insert(Path::new(CATALOG_KEY), bytes)
            .map_err(|error| map_dir_cache_error(&self.path, error))?;
        cache
            .sync()
            .map_err(|error| map_dir_cache_error(&self.path, error))
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        bytes: &[u8],
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot: PersistedSnapshot = serde_json::from_slice(bytes)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot: DocumentSnapshot = snapshot.into();
        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }
}

impl SnapshotStore for DirCacheSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let mut cache = self.lock_cache()?;
        let Some(bytes) = cache
            .get(&Self::snapshot_key(doc_id))
            .map_err(|error| map_dir_cache_error(&self.path, error))?
        else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, bytes.as_ref()).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot))
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        let mut cache = self.lock_cache()?;

        cache
            .insert(&Self::snapshot_key(&doc_id), bytes)
            .map_err(|error| map_dir_cache_error(&self.path, error))?;

        let mut catalog = self.load_catalog(&mut cache)?;
        if !catalog.contains(&doc_id) {
            catalog.push(doc_id);
            catalog.sort_unstable();
            self.save_catalog(&mut cache, &catalog)?;
        } else {
            cache
                .sync()
                .map_err(|error| map_dir_cache_error(&self.path, error))?;
        }

        Ok(())
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut cache = self.lock_cache()?;
        cache
            .remove(&Self::snapshot_key(doc_id))
            .map_err(|error| map_dir_cache_error(&self.path, error))?;

        let mut catalog = self.load_catalog(&mut cache)?;
        catalog.retain(|catalog_doc_id| catalog_doc_id != doc_id);
        self.save_catalog(&mut cache, &catalog)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut cache = self.lock_cache()?;
        let catalog = self.load_catalog(&mut cache)?;
        let mut documents = Vec::new();

        for doc_id in catalog {
            let Some(bytes) = cache
                .get(&Self::snapshot_key(&doc_id))
                .map_err(|error| map_dir_cache_error(&self.path, error))?
            else {
                tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing dir-cache snapshot while building document catalog"
                );
                continue;
            };

            match self.deserialize_snapshot(doc_id, bytes.as_ref()) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt dir-cache snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}

fn map_dir_cache_error(path: &Path, error: dir_cache::error::Error) -> StorageError {
    StorageError::Io(format!("{}: {error}", path.display()))
}
