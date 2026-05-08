use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use shorterdb::ShorterDB;
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOT_CATALOG_KEY: &[u8] = b"__catalog__";

pub struct ShorterDbSnapshotStore {
    path: PathBuf,
    database: Mutex<ShorterDB>,
}

impl ShorterDbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_SHORTERDB_PATH cannot be empty when SNAPSHOT_STORE=shorterdb".to_owned(),
            ));
        }

        fs::create_dir_all(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        let database = ShorterDB::new(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            database: Mutex::new(database),
        })
    }

    fn lock_database(&self) -> Result<MutexGuard<'_, ShorterDB>, StorageError> {
        self.database.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: shorterdb database mutex was poisoned",
                self.path.display()
            ))
        })
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

    fn load_catalog(&self, database: &ShorterDB) -> Result<Vec<String>, StorageError> {
        let Some(bytes) = database
            .get(SNAPSHOT_CATALOG_KEY)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?
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

    fn save_catalog(
        &self,
        database: &mut ShorterDB,
        catalog: &[String],
    ) -> Result<(), StorageError> {
        let bytes = serde_json::to_vec(catalog)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        database
            .set(SNAPSHOT_CATALOG_KEY, &bytes)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }
}

impl SnapshotStore for ShorterDbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let database = self.lock_database()?;
        let doc_id_key = doc_id.to_string();
        let Some(bytes) = database
            .get(doc_id_key.as_bytes())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?
        else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &bytes).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let mut database = self.lock_database()?;
        let doc_id = snapshot.document.id;
        let doc_id_key = doc_id.to_string();
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize shorterdb snapshot `{doc_id}`: {error}"
            ))
        })?;
        let mut catalog = self.load_catalog(&database)?;

        database
            .set(doc_id_key.as_bytes(), &bytes)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        if !catalog.iter().any(|value| value == &doc_id_key) {
            catalog.push(doc_id_key);
            catalog.sort();
        }

        // ShorterDB syncs every WAL write, so save/delete remain durable without
        // closing and reopening the engine on each mutation.
        self.save_catalog(&mut database, &catalog)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut database = self.lock_database()?;
        let doc_id_key = doc_id.to_string();
        let mut catalog = self.load_catalog(&database)?;

        let _ = database
            .delete(doc_id_key.as_bytes())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        catalog.retain(|value| value != &doc_id_key);

        self.save_catalog(&mut database, &catalog)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let database = self.lock_database()?;
        let mut documents = Vec::new();
        let catalog = self.load_catalog(&database)?;

        for doc_id_key in catalog {
            let Ok(doc_id) = Uuid::parse_str(&doc_id_key) else {
                continue;
            };

            match database.get(doc_id_key.as_bytes()) {
                Ok(Some(bytes)) => match self.deserialize_snapshot(doc_id, &bytes) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping corrupt shorterdb snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                },
                Ok(None) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing shorterdb snapshot while building document catalog"
                ),
                Err(error) => {
                    return Err(StorageError::Io(format!(
                        "{}: {error}",
                        self.path.display()
                    )));
                }
            }
        }

        Ok(documents)
    }
}
