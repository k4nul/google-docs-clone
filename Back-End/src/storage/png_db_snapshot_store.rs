use std::{
    collections::BTreeMap,
    fs::{self, File},
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
};

use png_db::{DataRow, PngDatabase, Schema};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

const PNG_WIDTH: u32 = 256;
const PNG_HEIGHT: u32 = 256;

pub struct PngDbSnapshotStore {
    path: PathBuf,
    lock: Mutex<()>,
}

impl PngDbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = normalize_path(path.into())?;
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            ensure_snapshot_dir(parent)?;
        }

        let store = Self {
            path,
            lock: Mutex::new(()),
        };

        if !store.path.exists() {
            store.persist_all(&BTreeMap::new())?;
        } else {
            store.load_all()?;
        }

        Ok(store)
    }

    fn lock_store(&self) -> Result<MutexGuard<'_, ()>, StorageError> {
        self.lock.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: png-db mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn load_database(&self) -> Result<PngDatabase, StorageError> {
        let path = path_to_str(&self.path)?;
        PngDatabase::load_from_png(path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn load_all(&self) -> Result<BTreeMap<Uuid, PersistedSnapshot>, StorageError> {
        let database = self.load_database()?;
        let mut snapshots = BTreeMap::new();

        for row in database.rows {
            let (doc_id, snapshot) = decode_row(row, &self.path)?;
            snapshots.insert(doc_id, snapshot);
        }

        Ok(snapshots)
    }

    fn persist_all(
        &self,
        snapshots: &BTreeMap<Uuid, PersistedSnapshot>,
    ) -> Result<(), StorageError> {
        if snapshots.len() > (PNG_WIDTH * PNG_HEIGHT) as usize {
            return Err(StorageError::Io(format!(
                "{}: png-db snapshot catalog exceeds {PNG_WIDTH}x{PNG_HEIGHT} row capacity",
                self.path.display()
            )));
        }

        let mut database = PngDatabase::new(PNG_WIDTH, PNG_HEIGHT, snapshot_schema());
        for (index, (doc_id, snapshot)) in snapshots.iter().enumerate() {
            let x = (index as u32) % PNG_WIDTH;
            let y = (index as u32) / PNG_WIDTH;
            database
                .insert(x, y, encode_row(*doc_id, snapshot)?)
                .map_err(|error| {
                    StorageError::Io(format!(
                        "{}: failed to stage png-db snapshot row `{doc_id}`: {error}",
                        self.path.display()
                    ))
                })?;
        }

        let temp_path = temp_path(&self.path);
        if temp_path.exists() {
            fs::remove_file(&temp_path)
                .map_err(|error| StorageError::Io(format!("{}: {error}", temp_path.display())))?;
        }

        let temp_path_str = path_to_str(&temp_path)?;
        database.save_to_png(temp_path_str).map_err(|error| {
            StorageError::Io(format!(
                "{}: failed to persist png-db snapshot store: {error}",
                temp_path.display()
            ))
        })?;
        sync_file(&temp_path)?;
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

impl SnapshotStore for PngDbSnapshotStore {
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
            let snapshot = self.decode_snapshot(doc_id, snapshot)?;
            documents.push(snapshot.document);
        }

        Ok(documents)
    }
}

fn snapshot_schema() -> Schema {
    Schema {
        fields: [
            ("doc_id".to_owned(), "string".to_owned()),
            ("snapshot".to_owned(), "object".to_owned()),
        ]
        .into_iter()
        .collect(),
    }
}

fn encode_row(doc_id: Uuid, snapshot: &PersistedSnapshot) -> Result<Value, StorageError> {
    Ok(serde_json::json!({
        "doc_id": doc_id,
        "snapshot": snapshot,
    }))
}

fn decode_row(row: DataRow, path: &Path) -> Result<(Uuid, PersistedSnapshot), StorageError> {
    let doc_id = row
        .data
        .get("doc_id")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            StorageError::Io(format!(
                "{}: png-db snapshot row is missing doc_id",
                path.display()
            ))
        })?;
    let doc_id = Uuid::parse_str(doc_id).map_err(|_| {
        StorageError::Io(format!(
            "{}: png-db snapshot row has invalid doc_id `{doc_id}`",
            path.display()
        ))
    })?;
    let snapshot_value = row.data.get("snapshot").cloned().ok_or_else(|| {
        StorageError::Io(format!(
            "{}: png-db snapshot row `{doc_id}` is missing snapshot payload",
            path.display()
        ))
    })?;
    let snapshot = serde_json::from_value::<PersistedSnapshot>(snapshot_value)
        .map_err(|_| StorageError::CorruptSnapshot(doc_id))?;

    Ok((doc_id, snapshot))
}

fn normalize_path(path: PathBuf) -> Result<PathBuf, StorageError> {
    if path.as_os_str().is_empty() {
        return Err(StorageError::Config(
            "SNAPSHOT_PNG_DB_PATH cannot be empty when SNAPSHOT_STORE=png_db".to_owned(),
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

fn temp_path(path: &Path) -> PathBuf {
    let mut temp_path = path.to_path_buf();
    let extension = path
        .extension()
        .map(|extension| format!("{}.tmp", extension.to_string_lossy()))
        .unwrap_or_else(|| "tmp".to_owned());
    temp_path.set_extension(extension);
    temp_path
}

fn path_to_str(path: &Path) -> Result<&str, StorageError> {
    path.to_str().ok_or_else(|| {
        StorageError::Config(
            "SNAPSHOT_PNG_DB_PATH must be valid unicode when SNAPSHOT_STORE=png_db".to_owned(),
        )
    })
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
