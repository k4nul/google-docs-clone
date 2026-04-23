use std::{
    fs::File,
    io,
    path::{Path, PathBuf},
};

use saturn::{Key, SaturnDB, Value};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

const SNAPSHOT_CATALOG_KEY: &[u8] = b"__catalog__";
const SNAPSHOT_KEY_PREFIX: &[u8] = b"snapshot:";

pub struct SaturnSnapshotStore {
    path: PathBuf,
    database: SaturnDB,
}

impl SaturnSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_SATURN_PATH cannot be empty when SNAPSHOT_STORE=saturn".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            ensure_snapshot_dir(parent)?;
        }

        let path_string = path
            .to_str()
            .ok_or_else(|| {
                StorageError::Config(
                    "SNAPSHOT_SATURN_PATH must be valid unicode when SNAPSHOT_STORE=saturn"
                        .to_owned(),
                )
            })?
            .to_owned();

        let database = SaturnDB::new(&path_string)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;
        database
            .recover(&path_string)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        let store = Self { path, database };
        store.sync_file()?;

        Ok(store)
    }

    fn snapshot_key(doc_id: &Uuid) -> Key {
        let mut key = Vec::with_capacity(SNAPSHOT_KEY_PREFIX.len() + 36);
        key.extend_from_slice(SNAPSHOT_KEY_PREFIX);
        key.extend_from_slice(doc_id.to_string().as_bytes());
        key
    }

    fn read_catalog(&self) -> Result<Vec<Uuid>, StorageError> {
        let catalog = self
            .database
            .get(&SNAPSHOT_CATALOG_KEY.to_vec())
            .map_err(|error| {
                StorageError::Io(format!(
                    "{}: failed to read saturn snapshot catalog: {error}",
                    self.path.display()
                ))
            })?;

        match catalog {
            Some(catalog) => serde_json::from_slice(&catalog).map_err(|error| {
                StorageError::Io(format!(
                    "{}: failed to decode saturn snapshot catalog: {error}",
                    self.path.display()
                ))
            }),
            None => Ok(Vec::new()),
        }
    }

    fn write_catalog(&self, catalog: &[Uuid]) -> Result<(), StorageError> {
        let payload = serde_json::to_vec(catalog).map_err(|error| {
            StorageError::Io(format!(
                "{}: failed to encode saturn snapshot catalog: {error}",
                self.path.display()
            ))
        })?;

        self.database
            .put(SNAPSHOT_CATALOG_KEY.to_vec(), payload)
            .map_err(|error| {
                StorageError::Io(format!(
                    "{}: failed to write saturn snapshot catalog: {error}",
                    self.path.display()
                ))
            })?;
        self.sync_file()
    }

    fn sync_file(&self) -> Result<(), StorageError> {
        File::open(&self.path)
            .and_then(|file| file.sync_all())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        if let Some(parent) = self
            .path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            sync_parent_dir(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        Ok(())
    }
}

impl SnapshotStore for SaturnSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let snapshot = self
            .database
            .get(&Self::snapshot_key(doc_id))
            .map_err(|_| StorageError::CorruptSnapshot(*doc_id))?;

        let Some(snapshot) = snapshot else {
            return Ok(None);
        };

        let snapshot: PersistedSnapshot = serde_json::from_slice(&snapshot)
            .map_err(|_| StorageError::CorruptSnapshot(*doc_id))?;
        let snapshot: DocumentSnapshot = snapshot.into();
        if snapshot.document.id != *doc_id {
            return Err(StorageError::CorruptSnapshot(*doc_id));
        }

        Ok(Some(snapshot))
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let persisted = PersistedSnapshot::from(snapshot);
        let payload: Value = serde_json::to_vec(&persisted).map_err(|error| {
            StorageError::Io(format!(
                "{}: failed to encode saturn snapshot `{doc_id}`: {error}",
                self.path.display()
            ))
        })?;

        self.database
            .put(Self::snapshot_key(&doc_id), payload)
            .map_err(|error| {
                StorageError::Io(format!(
                    "{}: failed to write saturn snapshot `{doc_id}`: {error}",
                    self.path.display()
                ))
            })?;
        self.sync_file()?;

        let mut catalog = self.read_catalog()?;
        if !catalog.contains(&doc_id) {
            catalog.push(doc_id);
            catalog.sort_unstable();
            self.write_catalog(&catalog)?;
        }

        Ok(())
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        self.database
            .delete(Self::snapshot_key(doc_id))
            .map_err(|error| {
                StorageError::Io(format!(
                    "{}: failed to delete saturn snapshot `{doc_id}`: {error}",
                    self.path.display()
                ))
            })?;
        self.sync_file()?;

        let mut catalog = self.read_catalog()?;
        let original_len = catalog.len();
        catalog.retain(|catalog_doc_id| catalog_doc_id != doc_id);
        if catalog.len() != original_len {
            self.write_catalog(&catalog)?;
        }

        Ok(())
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut documents = Vec::new();

        for doc_id in self.read_catalog()? {
            match self.load_snapshot(&doc_id)? {
                Some(snapshot) => documents.push(snapshot.document),
                None => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing saturn snapshot referenced by catalog"
                ),
            }
        }

        Ok(documents)
    }
}

fn sync_parent_dir(path: &Path) -> io::Result<()> {
    File::open(path).and_then(|file| file.sync_all())
}
