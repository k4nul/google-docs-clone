use std::{
    collections::BTreeMap,
    fs::File,
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
};

use data_pile::Database;
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
enum DataPileSnapshotRecord {
    Save {
        doc_id: Uuid,
        snapshot: PersistedSnapshot,
    },
    Delete {
        doc_id: Uuid,
    },
}

pub struct DataPileSnapshotStore {
    path: PathBuf,
    database: Database,
    snapshots: Mutex<BTreeMap<Uuid, DocumentSnapshot>>,
}

impl DataPileSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_DATA_PILE_PATH cannot be empty when SNAPSHOT_STORE=data_pile".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            ensure_snapshot_dir(parent)?;
        }

        let database = Database::file(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;
        let snapshots = Mutex::new(Self::replay_records(&path, &database)?);

        Ok(Self {
            path,
            database,
            snapshots,
        })
    }

    fn replay_records(
        path: &Path,
        database: &Database,
    ) -> Result<BTreeMap<Uuid, DocumentSnapshot>, StorageError> {
        let Some(records) = database.iter_from_seqno(0) else {
            return Ok(BTreeMap::new());
        };

        let mut snapshots = BTreeMap::new();
        for record in records {
            let record = serde_json::from_slice::<DataPileSnapshotRecord>(record.as_ref())
                .map_err(|error| {
                    StorageError::Io(format!(
                        "{}: failed to parse data_pile snapshot record: {error}",
                        path.display()
                    ))
                })?;

            match record {
                DataPileSnapshotRecord::Save { doc_id, snapshot } => {
                    let snapshot: DocumentSnapshot = snapshot.into();
                    if snapshot.document.id != doc_id {
                        return Err(StorageError::CorruptSnapshot(doc_id));
                    }
                    snapshots.insert(doc_id, snapshot);
                }
                DataPileSnapshotRecord::Delete { doc_id } => {
                    snapshots.remove(&doc_id);
                }
            }
        }

        Ok(snapshots)
    }

    fn lock_snapshots(
        &self,
    ) -> Result<MutexGuard<'_, BTreeMap<Uuid, DocumentSnapshot>>, StorageError> {
        self.snapshots.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: data_pile snapshot mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn append_record(&self, record: &DataPileSnapshotRecord) -> Result<(), StorageError> {
        let payload = serde_json::to_vec(record).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize data_pile snapshot record: {error}"
            ))
        })?;

        self.database
            .put(&payload)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        self.sync_files()
    }

    fn sync_files(&self) -> Result<(), StorageError> {
        for file_name in ["data", "seqno"] {
            let file_path = self.path.join(file_name);
            if file_path.exists() {
                File::open(&file_path)
                    .and_then(|file| file.sync_all())
                    .map_err(|error| {
                        StorageError::Io(format!("{}: {error}", file_path.display()))
                    })?;
            }
        }

        File::open(&self.path)
            .and_then(|file| file.sync_all())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }
}

impl SnapshotStore for DataPileSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        Ok(self.lock_snapshots()?.get(doc_id).cloned())
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let persisted = PersistedSnapshot::from(snapshot.clone());
        self.append_record(&DataPileSnapshotRecord::Save {
            doc_id,
            snapshot: persisted,
        })?;
        self.lock_snapshots()?.insert(doc_id, snapshot);
        Ok(())
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        self.append_record(&DataPileSnapshotRecord::Delete { doc_id: *doc_id })?;
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
