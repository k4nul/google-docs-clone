use std::{
    collections::BTreeMap,
    fs::{self, File},
    panic::{self, AssertUnwindSafe},
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
};

use json_db_rs::controller::{Connection, connect};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum JsonDbRsSnapshotRecord {
    Save {
        doc_id: Uuid,
        snapshot: PersistedSnapshot,
    },
    Delete {
        doc_id: Uuid,
    },
}

pub struct JsonDbRsSnapshotStore {
    path: PathBuf,
    connection: Mutex<Connection<JsonDbRsSnapshotRecord>>,
}

impl JsonDbRsSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_JSON_DB_RS_PATH cannot be empty when SNAPSHOT_STORE=json_db_rs"
                    .to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        if !path.exists() {
            fs::write(&path, b"[]")
                .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;
            Self::sync_path(&path)?;
        }

        let path_text = path.to_string_lossy().into_owned();
        let mut connection = panic::catch_unwind(AssertUnwindSafe(|| {
            connect::<JsonDbRsSnapshotRecord>(&path_text, Vec::new())
        }))
        .map_err(|_| {
            StorageError::Io(format!(
                "{}: json_db_rs failed to open snapshot store",
                path.display()
            ))
        })?;

        Self::read_records_from_connection(&path, &mut connection)?;

        Ok(Self {
            path,
            connection: Mutex::new(connection),
        })
    }

    fn lock_connection(
        &self,
    ) -> Result<MutexGuard<'_, Connection<JsonDbRsSnapshotRecord>>, StorageError> {
        self.connection.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: json_db_rs snapshot mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn read_records(
        &self,
        connection: &mut Connection<JsonDbRsSnapshotRecord>,
    ) -> Result<Vec<JsonDbRsSnapshotRecord>, StorageError> {
        Self::read_records_from_connection(&self.path, connection)
    }

    fn read_records_from_connection(
        path: &Path,
        connection: &mut Connection<JsonDbRsSnapshotRecord>,
    ) -> Result<Vec<JsonDbRsSnapshotRecord>, StorageError> {
        panic::catch_unwind(AssertUnwindSafe(|| connection.read_data().clone())).map_err(|_| {
            StorageError::Io(format!(
                "{}: json_db_rs failed to read snapshot records",
                path.display()
            ))
        })
    }

    fn replay_records(
        records: Vec<JsonDbRsSnapshotRecord>,
    ) -> Result<BTreeMap<Uuid, DocumentSnapshot>, StorageError> {
        let mut snapshots = BTreeMap::new();

        for record in records {
            match record {
                JsonDbRsSnapshotRecord::Save { doc_id, snapshot } => {
                    let snapshot: DocumentSnapshot = snapshot.into();
                    if snapshot.document.id != doc_id {
                        return Err(StorageError::CorruptSnapshot(doc_id));
                    }
                    snapshots.insert(doc_id, snapshot);
                }
                JsonDbRsSnapshotRecord::Delete { doc_id } => {
                    snapshots.remove(&doc_id);
                }
            }
        }

        Ok(snapshots)
    }

    fn append_record(
        &self,
        connection: &mut Connection<JsonDbRsSnapshotRecord>,
        record: JsonDbRsSnapshotRecord,
    ) -> Result<(), StorageError> {
        self.read_records(connection)?;
        connection.append(record);
        panic::catch_unwind(AssertUnwindSafe(|| connection.sync())).map_err(|_| {
            StorageError::Io(format!(
                "{}: json_db_rs failed to sync snapshot records",
                self.path.display()
            ))
        })?;
        Self::sync_path(&self.path)
    }

    fn sync_path(path: &Path) -> Result<(), StorageError> {
        File::open(path)
            .and_then(|file| file.sync_all())
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            File::open(parent)
                .and_then(|file| file.sync_all())
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        Ok(())
    }
}

impl SnapshotStore for JsonDbRsSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let mut connection = self.lock_connection()?;
        let snapshots = Self::replay_records(self.read_records(&mut connection)?)?;
        Ok(snapshots.get(doc_id).cloned())
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let record = JsonDbRsSnapshotRecord::Save {
            doc_id,
            snapshot: PersistedSnapshot::from(snapshot),
        };

        let mut connection = self.lock_connection()?;
        self.append_record(&mut connection, record)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut connection = self.lock_connection()?;
        self.append_record(
            &mut connection,
            JsonDbRsSnapshotRecord::Delete { doc_id: *doc_id },
        )
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut connection = self.lock_connection()?;
        let snapshots = Self::replay_records(self.read_records(&mut connection)?)?;

        Ok(snapshots
            .values()
            .map(|snapshot| snapshot.document.clone())
            .collect())
    }
}
