use std::{
    fs,
    path::{Path, PathBuf},
};

use siamesedb::{DbXxx, DbXxxBase};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOTS_MAP: &str = "snapshots";
const SNAPSHOT_CATALOG_KEY: &str = "__catalog__";

pub struct SiamesedbSnapshotStore {
    path: PathBuf,
}

impl SiamesedbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_SIAMESDB_PATH cannot be empty when SNAPSHOT_STORE=siamesedb".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        siamesedb::open_file(&path).map_err(|error| Self::map_database_error(&path, error))?;

        Ok(Self { path })
    }

    fn map_database_error(path: &Path, error: std::io::Error) -> StorageError {
        StorageError::Io(format!("{}: {error}", path.display()))
    }

    fn deserialize_snapshot(
        &self,
        expected_doc_id: Uuid,
        bytes: &[u8],
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot = serde_json::from_slice::<PersistedSnapshot>(bytes)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot: DocumentSnapshot = snapshot.into();

        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }

    fn load_catalog<T>(&self, snapshots: &mut T) -> Result<Vec<String>, StorageError>
    where
        T: DbXxx<siamesedb::DbString>,
    {
        let Some(bytes) = snapshots
            .get(SNAPSHOT_CATALOG_KEY)
            .map_err(|error| Self::map_database_error(&self.path, error))?
        else {
            return Ok(Vec::new());
        };

        serde_json::from_slice::<Vec<String>>(&bytes).map_err(|_| {
            StorageError::Io(format!(
                "{}: snapshot catalog is corrupt",
                self.path.display()
            ))
        })
    }

    fn save_catalog<T>(&self, snapshots: &mut T, catalog: &[String]) -> Result<(), StorageError>
    where
        T: DbXxx<siamesedb::DbString>,
    {
        let bytes = serde_json::to_vec(catalog)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        snapshots
            .put(SNAPSHOT_CATALOG_KEY, &bytes)
            .map_err(|error| Self::map_database_error(&self.path, error))
    }
}

impl SnapshotStore for SiamesedbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let database = siamesedb::open_file(&self.path)
            .map_err(|error| Self::map_database_error(&self.path, error))?;
        let mut snapshots = database
            .db_map_string(SNAPSHOTS_MAP)
            .map_err(|error| Self::map_database_error(&self.path, error))?;
        let doc_id_key = doc_id.to_string();
        let Some(bytes) = snapshots
            .get(&doc_id_key)
            .map_err(|error| Self::map_database_error(&self.path, error))?
        else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &bytes).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let database = siamesedb::open_file(&self.path)
            .map_err(|error| Self::map_database_error(&self.path, error))?;
        let doc_id = snapshot.document.id;
        let doc_id_key = doc_id.to_string();
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize siamesedb snapshot `{doc_id}`: {error}"
            ))
        })?;
        let mut snapshots = database
            .db_map_string(SNAPSHOTS_MAP)
            .map_err(|error| Self::map_database_error(&self.path, error))?;
        let mut catalog = self.load_catalog(&mut snapshots)?;

        snapshots
            .put(&doc_id_key, &bytes)
            .map_err(|error| Self::map_database_error(&self.path, error))?;
        if !catalog.iter().any(|value| value == &doc_id_key) {
            catalog.push(doc_id_key.clone());
            catalog.sort();
        }
        self.save_catalog(&mut snapshots, &catalog)?;
        snapshots
            .sync_data()
            .map_err(|error| Self::map_database_error(&self.path, error))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let database = siamesedb::open_file(&self.path)
            .map_err(|error| Self::map_database_error(&self.path, error))?;
        let mut snapshots = database
            .db_map_string(SNAPSHOTS_MAP)
            .map_err(|error| Self::map_database_error(&self.path, error))?;
        let doc_id_key = doc_id.to_string();
        let mut catalog = self.load_catalog(&mut snapshots)?;

        snapshots
            .delete(&doc_id_key)
            .map_err(|error| Self::map_database_error(&self.path, error))?;
        catalog.retain(|value| value != &doc_id_key);
        self.save_catalog(&mut snapshots, &catalog)?;
        snapshots
            .sync_data()
            .map_err(|error| Self::map_database_error(&self.path, error))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let database = siamesedb::open_file(&self.path)
            .map_err(|error| Self::map_database_error(&self.path, error))?;
        let snapshots = database
            .db_map_string(SNAPSHOTS_MAP)
            .map_err(|error| Self::map_database_error(&self.path, error))?;
        let mut snapshots = snapshots;
        let mut documents = Vec::new();
        let catalog = self.load_catalog(&mut snapshots)?;

        for doc_id_key in catalog {
            let Ok(doc_id) = Uuid::parse_str(&doc_id_key) else {
                continue;
            };

            match snapshots.get(&doc_id_key) {
                Ok(Some(bytes)) => match self.deserialize_snapshot(doc_id, &bytes) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping corrupt siamesedb snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                },
                Ok(None) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing siamesedb snapshot while building document catalog"
                ),
                Err(error) => return Err(Self::map_database_error(&self.path, error)),
            }
        }

        Ok(documents)
    }
}
