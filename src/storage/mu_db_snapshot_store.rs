use std::{
    fs::File,
    panic::{AssertUnwindSafe, catch_unwind},
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
};

use mu_db::DataBase;
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

const SNAPSHOT_CATALOG_KEY: &str = "__catalog__";
const SNAPSHOT_KEY_PREFIX: &str = "snapshot:";

pub struct MuDbSnapshotStore {
    path: PathBuf,
    index_path: PathBuf,
    database: Mutex<DataBase>,
}

impl MuDbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_MU_DB_PATH cannot be empty when SNAPSHOT_STORE=mu_db".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            ensure_snapshot_dir(parent)?;
        }

        let database = Self::open_database(&path)?;
        let index_path = Self::index_path(&path)?;

        Ok(Self {
            path,
            index_path,
            database: Mutex::new(database),
        })
    }

    fn snapshot_key(doc_id: &Uuid) -> String {
        format!("{SNAPSHOT_KEY_PREFIX}{doc_id}")
    }

    fn open_database(path: &Path) -> Result<DataBase, StorageError> {
        let normalized_path = if path.parent().is_some() {
            path.to_path_buf()
        } else {
            PathBuf::from(".").join(path)
        };
        let Some(path) = normalized_path.to_str() else {
            return Err(StorageError::Config(
                "SNAPSHOT_MU_DB_PATH must be valid unicode when SNAPSHOT_STORE=mu_db".to_owned(),
            ));
        };

        catch_unwind(AssertUnwindSafe(|| DataBase::new(path))).map_err(|_| {
            StorageError::Io(format!(
                "{}: failed to open mu_db snapshot store",
                normalized_path.display()
            ))
        })
    }

    fn index_path(path: &Path) -> Result<PathBuf, StorageError> {
        let file_name = path
            .file_name()
            .ok_or_else(|| {
                StorageError::Config(
                    "SNAPSHOT_MU_DB_PATH must include a file name when SNAPSHOT_STORE=mu_db"
                        .to_owned(),
                )
            })?
            .to_string_lossy();
        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));

        Ok(parent.join(format!("index_{file_name}")))
    }

    fn lock_database(&self) -> Result<MutexGuard<'_, DataBase>, StorageError> {
        self.database.lock().map_err(|_| {
            StorageError::Io(format!("{}: mu_db mutex was poisoned", self.path.display()))
        })
    }

    fn run_database_operation<T>(
        &self,
        operation: &str,
        f: impl FnOnce(&mut DataBase) -> Result<T, StorageError>,
    ) -> Result<T, StorageError> {
        let mut database = self.lock_database()?;
        catch_unwind(AssertUnwindSafe(|| f(&mut database))).map_err(|_| {
            StorageError::Io(format!(
                "{}: mu_db panicked while {operation}",
                self.path.display()
            ))
        })?
    }

    fn sync_files(&self) -> Result<(), StorageError> {
        Self::sync_path(&self.path)?;
        Self::sync_path(&self.index_path)
    }

    fn sync_path(path: &Path) -> Result<(), StorageError> {
        let file = File::open(path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;
        file.sync_all()
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))
    }

    fn read_string(&self, key: &str) -> Result<Option<String>, StorageError> {
        self.run_database_operation("reading snapshot", |database| Ok(database.get(key)))
    }

    fn write_string(&self, key: &str, value: &str) -> Result<(), StorageError> {
        self.run_database_operation("writing snapshot", |database| {
            database.insert(key, value);
            Ok(())
        })?;
        self.sync_files()
    }

    fn read_catalog(&self) -> Result<Vec<Uuid>, StorageError> {
        let Some(payload) = self.read_string(SNAPSHOT_CATALOG_KEY)? else {
            return Ok(Vec::new());
        };

        serde_json::from_str::<Vec<Uuid>>(&payload).map_err(|_| {
            StorageError::Io(format!(
                "{}: mu_db snapshot catalog is corrupt",
                self.path.display()
            ))
        })
    }

    fn write_catalog(&self, catalog: &[Uuid]) -> Result<(), StorageError> {
        let payload = serde_json::to_string(catalog).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize mu_db snapshot catalog: {error}"
            ))
        })?;

        self.write_string(SNAPSHOT_CATALOG_KEY, &payload)
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        payload: &str,
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot = serde_json::from_str::<PersistedSnapshot>(payload)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot: DocumentSnapshot = snapshot.into();

        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }
}

impl SnapshotStore for MuDbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let Some(payload) = self.read_string(&Self::snapshot_key(doc_id))? else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &payload).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload =
            serde_json::to_string(&PersistedSnapshot::from(snapshot)).map_err(|error| {
                StorageError::Io(format!(
                    "failed to serialize mu_db snapshot `{doc_id}`: {error}"
                ))
            })?;

        self.write_string(&Self::snapshot_key(&doc_id), &payload)?;

        let mut catalog = self.read_catalog()?;
        if !catalog.contains(&doc_id) {
            catalog.push(doc_id);
            catalog.sort_unstable();
            self.write_catalog(&catalog)?;
        }

        Ok(())
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        self.run_database_operation("deleting snapshot", |database| {
            database.remove(&Self::snapshot_key(doc_id));
            Ok(())
        })?;
        self.sync_files()?;

        let mut catalog = self.read_catalog()?;
        let original_len = catalog.len();
        catalog.retain(|catalog_doc_id| catalog_doc_id != doc_id);
        if catalog.len() != original_len {
            self.write_catalog(&catalog)?;
        }

        Ok(())
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let catalog = self.read_catalog()?;
        let mut documents = Vec::new();

        for doc_id in catalog {
            match self.load_snapshot(&doc_id)? {
                Some(snapshot) => documents.push(snapshot.document),
                None => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing mu_db snapshot referenced by catalog"
                ),
            }
        }

        Ok(documents)
    }
}
