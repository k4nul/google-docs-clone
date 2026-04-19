use std::{
    fs,
    path::{Path, PathBuf},
};

use microkv::MicroKV;
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

pub struct MicroKvSnapshotStore {
    base_path: PathBuf,
    database_path: PathBuf,
    database_name: String,
    database_root: PathBuf,
}

impl MicroKvSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let base_path = path.into();
        if base_path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_MICROKV_PATH cannot be empty when SNAPSHOT_STORE=microkv".to_owned(),
            ));
        }

        let database_name = base_path
            .file_name()
            .and_then(|value| value.to_str())
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| {
                StorageError::Config(
                    "SNAPSHOT_MICROKV_PATH must end with a non-empty base file name".to_owned(),
                )
            })?
            .to_owned();
        let database_root = base_path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));

        fs::create_dir_all(&database_root)
            .map_err(|error| StorageError::Io(format!("{}: {error}", database_root.display())))?;
        let database_path = Self::database_path(&database_root, &database_name);

        Ok(Self {
            base_path,
            database_path,
            database_name,
            database_root,
        })
    }

    fn database_path(database_root: &Path, database_name: &str) -> PathBuf {
        let mut path = database_root.join(database_name);
        path.set_extension("kv");
        path
    }

    fn open_database(&self) -> Result<MicroKV, StorageError> {
        MicroKV::open_with_base_path(&self.database_name, self.database_root.clone())
            .map(|database| database.set_auto_commit(true))
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.database_path.display())))
    }

    fn deserialize_snapshot(
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

impl SnapshotStore for MicroKvSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let database = self.open_database()?;
        let doc_id_key = doc_id.to_string();
        let snapshot = database
            .get::<PersistedSnapshot>(&doc_id_key)
            .map_err(|_| StorageError::CorruptSnapshot(*doc_id))?;
        let Some(snapshot) = snapshot else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, snapshot).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let database = self.open_database()?;
        let doc_id_key = snapshot.document.id.to_string();
        let persisted_snapshot = PersistedSnapshot::from(snapshot);

        database
            .put(&doc_id_key, &persisted_snapshot)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.database_path.display())))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let database = self.open_database()?;
        let doc_id_key = doc_id.to_string();
        database
            .delete(&doc_id_key)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.database_path.display())))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let database = self.open_database()?;
        let mut documents = Vec::new();

        for doc_id_key in database.keys().map_err(|error| {
            StorageError::Io(format!("{}: {error}", self.database_path.display()))
        })? {
            let Ok(doc_id) = Uuid::parse_str(&doc_id_key) else {
                continue;
            };

            match database.get::<PersistedSnapshot>(&doc_id_key) {
                Ok(Some(snapshot)) => match self.deserialize_snapshot(doc_id, snapshot) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.database_path.display(),
                        base_path = %self.base_path.display(),
                        "skipping corrupt microkv snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                },
                Ok(None) => {}
                Err(_) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.database_path.display(),
                    base_path = %self.base_path.display(),
                    "skipping corrupt microkv snapshot while building document catalog"
                ),
            }
        }

        Ok(documents)
    }
}
