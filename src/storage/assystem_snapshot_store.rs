use std::{
    fs::{self, File, OpenOptions},
    panic::{AssertUnwindSafe, catch_unwind},
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use assystem::ASS;
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

pub struct AssystemSnapshotStore {
    path: PathBuf,
    database: Mutex<ASS<File>>,
}

impl AssystemSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_ASSYSTEM_PATH cannot be empty when SNAPSHOT_STORE=assystem".to_owned(),
            ));
        }

        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
            }
        }

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;
        let database = catch_unwind(AssertUnwindSafe(|| ASS::open(file)))
            .map_err(|_| StorageError::Io(format!("{}: assystem open panicked", path.display())))?
            .map_err(|error| StorageError::Io(format!("{}: {error:?}", path.display())))?;

        Ok(Self {
            path,
            database: Mutex::new(database),
        })
    }

    fn lock_database(&self) -> Result<MutexGuard<'_, ASS<File>>, StorageError> {
        self.database.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: assystem snapshot store mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn with_database<T>(
        &self,
        action: impl FnOnce(&mut ASS<File>) -> Result<T, StorageError>,
    ) -> Result<T, StorageError> {
        let mut database = self.lock_database()?;
        catch_unwind(AssertUnwindSafe(|| action(&mut database))).map_err(|_| {
            StorageError::Io(format!(
                "{}: assystem operation panicked",
                self.path.display()
            ))
        })?
    }

    fn sync_file(&self) -> Result<(), StorageError> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.path)
            .and_then(|file| file.sync_all())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn deserialize_snapshot(
        expected_doc_id: Uuid,
        payload: &[u8],
    ) -> Result<DocumentSnapshot, StorageError> {
        let snapshot = serde_json::from_slice::<PersistedSnapshot>(payload)
            .map_err(|_| StorageError::CorruptSnapshot(expected_doc_id))?;
        let snapshot: DocumentSnapshot = snapshot.into();

        if snapshot.document.id != expected_doc_id {
            return Err(StorageError::CorruptSnapshot(expected_doc_id));
        }

        Ok(snapshot)
    }
}

impl SnapshotStore for AssystemSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        self.with_database(|database| {
            database
                .get(doc_id.to_string().as_bytes())
                .map(|payload| Self::deserialize_snapshot(*doc_id, &payload))
                .transpose()
        })
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize assystem snapshot `{doc_id}`: {error}"
            ))
        })?;

        self.with_database(|database| {
            database.set(doc_id.to_string().as_bytes(), &payload);
            Ok(())
        })?;
        self.sync_file()
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        self.with_database(|database| {
            database.remove(doc_id.to_string().as_bytes());
            Ok(())
        })?;
        self.sync_file()
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut documents = self.with_database(|database| {
            let mut documents = Vec::new();

            for (key, payload) in database.list() {
                let Ok(doc_id_key) = String::from_utf8(key) else {
                    continue;
                };
                let Ok(doc_id) = Uuid::parse_str(&doc_id_key) else {
                    continue;
                };

                match Self::deserialize_snapshot(doc_id, &payload) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping corrupt assystem snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                }
            }

            Ok(documents)
        })?;

        documents.sort_unstable_by_key(|document| document.id);
        Ok(documents)
    }
}
