use std::{
    collections::BTreeMap,
    fs::{self, File},
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
};

use cdb64::{Cdb, CdbHash, CdbWriter};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

pub struct Cdb64SnapshotStore {
    path: PathBuf,
    lock: Mutex<()>,
}

impl Cdb64SnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = normalize_path(path.into())?;
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let store = Self {
            path,
            lock: Mutex::new(()),
        };

        if store.path.exists() {
            store.load_all()?;
        }

        Ok(store)
    }

    fn lock_store(&self) -> Result<MutexGuard<'_, ()>, StorageError> {
        self.lock.lock().map_err(|_| {
            StorageError::Io(format!("{}: cdb64 mutex was poisoned", self.path.display()))
        })
    }

    fn load_all(&self) -> Result<BTreeMap<Uuid, PersistedSnapshot>, StorageError> {
        if !self.path.exists() {
            return Ok(BTreeMap::new());
        }

        let cdb = Cdb::<File, CdbHash>::open(&self.path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        let mut snapshots = BTreeMap::new();

        for entry in cdb.iter() {
            let (key, value) = entry
                .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
            let Some(doc_id) = decode_doc_id(&key) else {
                continue;
            };
            let snapshot = serde_json::from_slice::<PersistedSnapshot>(&value)
                .map_err(|_| StorageError::CorruptSnapshot(doc_id))?;
            snapshots.insert(doc_id, snapshot);
        }

        Ok(snapshots)
    }

    fn persist_all(
        &self,
        snapshots: &BTreeMap<Uuid, PersistedSnapshot>,
    ) -> Result<(), StorageError> {
        let temp_path = temp_path(&self.path);
        if temp_path.exists() {
            fs::remove_file(&temp_path)
                .map_err(|error| StorageError::Io(format!("{}: {error}", temp_path.display())))?;
        }

        let persist_result = (|| -> Result<(), StorageError> {
            let mut writer = CdbWriter::<File, CdbHash>::create(&temp_path)
                .map_err(|error| StorageError::Io(format!("{}: {error}", temp_path.display())))?;

            for (doc_id, snapshot) in snapshots {
                let payload = serde_json::to_vec(snapshot).map_err(|error| {
                    StorageError::Io(format!(
                        "{}: failed to serialize cdb64 snapshot `{doc_id}`: {error}",
                        self.path.display()
                    ))
                })?;
                writer
                    .put(doc_id.to_string().as_bytes(), &payload)
                    .map_err(|error| {
                        StorageError::Io(format!(
                            "{}: failed to stage cdb64 snapshot `{doc_id}`: {error}",
                            temp_path.display()
                        ))
                    })?;
            }

            writer
                .finalize()
                .map_err(|error| StorageError::Io(format!("{}: {error}", temp_path.display())))?;
            let file = writer
                .into_inner()
                .map_err(|error| StorageError::Io(format!("{}: {error}", temp_path.display())))?;
            file.sync_all()
                .map_err(|error| StorageError::Io(format!("{}: {error}", temp_path.display())))?;

            Ok(())
        })();

        if let Err(error) = persist_result {
            let _ = fs::remove_file(&temp_path);
            return Err(error);
        }

        fs::rename(&temp_path, &self.path).map_err(|error| {
            StorageError::Io(format!(
                "{} -> {}: {error}",
                temp_path.display(),
                self.path.display()
            ))
        })?;
        sync_file(&self.path)?;
        sync_parent_dir(&self.path)
    }

    fn decode_snapshot(
        &self,
        expected_doc_id: Uuid,
        snapshot: PersistedSnapshot,
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot: DocumentSnapshot = snapshot.into();
        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }
}

impl SnapshotStore for Cdb64SnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let _guard = self.lock_store()?;
        let snapshots = self.load_all()?;
        let Some(snapshot) = snapshots.get(doc_id).cloned() else {
            return Ok(None);
        };

        self.decode_snapshot(*doc_id, snapshot).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let persisted = PersistedSnapshot::from(snapshot);

        let _guard = self.lock_store()?;
        let mut snapshots = self.load_all()?;
        snapshots.insert(doc_id, persisted);
        self.persist_all(&snapshots)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let _guard = self.lock_store()?;
        let mut snapshots = self.load_all()?;
        snapshots.remove(doc_id);
        self.persist_all(&snapshots)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let _guard = self.lock_store()?;
        let snapshots = self.load_all()?;
        let mut documents = Vec::new();

        for (doc_id, snapshot) in snapshots {
            match self.decode_snapshot(doc_id, snapshot) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt cdb64 snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}

fn normalize_path(path: PathBuf) -> Result<PathBuf, StorageError> {
    if path.as_os_str().is_empty() {
        return Err(StorageError::Config(
            "SNAPSHOT_CDB64_PATH cannot be empty when SNAPSHOT_STORE=cdb64".to_owned(),
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

fn decode_doc_id(bytes: &[u8]) -> Option<Uuid> {
    let key = std::str::from_utf8(bytes).ok()?;
    Uuid::parse_str(key).ok()
}

fn temp_path(path: &Path) -> PathBuf {
    let mut temp_path = path.to_path_buf();
    let extension = path
        .extension()
        .map(|extension| format!("{}.tmp", extension.to_string_lossy()))
        .unwrap_or_else(|| "tmp".to_owned());
    temp_path.set_extension(extension);
    temp_path
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
