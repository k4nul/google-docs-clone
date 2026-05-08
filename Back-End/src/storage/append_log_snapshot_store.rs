use std::{
    collections::BTreeMap,
    fs::File,
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
};

use append_log::{Log, Options};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum AppendLogSnapshotRecord {
    Save {
        doc_id: Uuid,
        snapshot: PersistedSnapshot,
    },
    Delete {
        doc_id: Uuid,
    },
}

pub struct AppendLogSnapshotStore {
    path: PathBuf,
    log: Mutex<Log>,
    snapshots: Mutex<BTreeMap<Uuid, DocumentSnapshot>>,
}

impl AppendLogSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_APPEND_LOG_PATH cannot be empty when SNAPSHOT_STORE=append_log"
                    .to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            ensure_snapshot_dir(parent)?;
        }

        let mut log = Self::open_log_at(&path)?;
        let snapshots = Self::replay_records(&path, &mut log)?;

        Ok(Self {
            path,
            log: Mutex::new(log),
            snapshots: Mutex::new(snapshots),
        })
    }

    fn open_log_at(path: &Path) -> Result<Log, StorageError> {
        Log::open_default(path).map_err(|error| {
            StorageError::Io(format!(
                "{}: failed to open append_log snapshot log: {error}",
                path.display()
            ))
        })
    }

    fn replay_records(
        path: &Path,
        log: &mut Log,
    ) -> Result<BTreeMap<Uuid, DocumentSnapshot>, StorageError> {
        let mut snapshots = BTreeMap::new();
        let options = Options::default();
        let block_size = options.block_size as u64;
        let mut offset = 0;
        let last_data_offset = log.last_data_off();

        while !log.is_empty() && offset <= last_data_offset {
            let chunk = log.read(offset).map_err(|error| {
                StorageError::Io(format!(
                    "{}: failed to read append_log snapshot record: {error}",
                    path.display()
                ))
            })?;
            let record = serde_json::from_slice::<AppendLogSnapshotRecord>(&chunk.data).map_err(
                |error| {
                    StorageError::Io(format!(
                        "{}: failed to parse append_log snapshot record: {error}",
                        path.display()
                    ))
                },
            )?;

            match record {
                AppendLogSnapshotRecord::Save { doc_id, snapshot } => {
                    let snapshot: DocumentSnapshot = snapshot.into();
                    if snapshot.document.id != doc_id {
                        return Err(StorageError::CorruptSnapshot(doc_id));
                    }
                    snapshots.insert(doc_id, snapshot);
                }
                AppendLogSnapshotRecord::Delete { doc_id } => {
                    snapshots.remove(&doc_id);
                }
            }

            offset = align_to_block(chunk.next + 16, block_size);
        }

        Ok(snapshots)
    }

    fn lock_log(&self) -> Result<MutexGuard<'_, Log>, StorageError> {
        self.log.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: append_log snapshot log mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn lock_snapshots(
        &self,
    ) -> Result<MutexGuard<'_, BTreeMap<Uuid, DocumentSnapshot>>, StorageError> {
        self.snapshots.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: append_log snapshot map mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn append_record(&self, record: &AppendLogSnapshotRecord) -> Result<(), StorageError> {
        let payload = serde_json::to_vec(record).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize append_log snapshot record: {error}"
            ))
        })?;

        let mut log = self.lock_log()?;
        log.append(&payload);
        log.flush()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        self.sync_file()
    }

    fn sync_file(&self) -> Result<(), StorageError> {
        File::open(&self.path)
            .and_then(|file| file.sync_all())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }
}

fn align_to_block(offset: u64, block_size: u64) -> u64 {
    let remainder = offset % block_size;
    if remainder == 0 {
        offset
    } else {
        offset + (block_size - remainder)
    }
}

impl SnapshotStore for AppendLogSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        Ok(self.lock_snapshots()?.get(doc_id).cloned())
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        self.append_record(&AppendLogSnapshotRecord::Save {
            doc_id,
            snapshot: PersistedSnapshot::from(snapshot.clone()),
        })?;
        self.lock_snapshots()?.insert(doc_id, snapshot);
        Ok(())
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        self.append_record(&AppendLogSnapshotRecord::Delete { doc_id: *doc_id })?;
        self.lock_snapshots()?.remove(doc_id);
        Ok(())
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        Ok(self
            .lock_snapshots()?
            .values()
            .map(|snapshot| snapshot.document.clone())
            .collect())
    }
}
