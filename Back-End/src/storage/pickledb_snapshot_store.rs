use std::{fs, path::PathBuf};

use pickledb::{PickleDb, PickleDbDumpPolicy};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

pub struct PickleDbSnapshotStore {
    path: PathBuf,
}

impl PickleDbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_PICKLEDB_PATH cannot be empty when SNAPSHOT_STORE=pickledb".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        Ok(Self { path })
    }

    fn open_database(&self) -> Result<PickleDb, StorageError> {
        if self.path.exists() {
            PickleDb::load_json(&self.path, PickleDbDumpPolicy::AutoDump)
                .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
        } else {
            Ok(PickleDb::new_json(&self.path, PickleDbDumpPolicy::AutoDump))
        }
    }
}

impl SnapshotStore for PickleDbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let database = self.open_database()?;
        let doc_id_key = doc_id.to_string();
        let Some(snapshot) = database.get::<PersistedSnapshot>(&doc_id_key) else {
            return Ok(None);
        };
        let snapshot: DocumentSnapshot = snapshot.into();

        if snapshot.document.id != *doc_id {
            return Err(StorageError::CorruptSnapshot(*doc_id));
        }

        Ok(Some(snapshot))
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let mut database = self.open_database()?;
        let doc_id_key = snapshot.document.id.to_string();
        let persisted_snapshot = PersistedSnapshot::from(snapshot);

        database
            .set(&doc_id_key, &persisted_snapshot)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut database = self.open_database()?;
        let doc_id_key = doc_id.to_string();
        database
            .rem(&doc_id_key)
            .map(|_| ())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let database = self.open_database()?;
        let mut documents = Vec::new();

        for doc_id_key in database.get_all() {
            let Ok(doc_id) = Uuid::parse_str(&doc_id_key) else {
                continue;
            };

            match database.get::<PersistedSnapshot>(&doc_id_key) {
                Some(snapshot) => {
                    let snapshot: DocumentSnapshot = snapshot.into();
                    if snapshot.document.id != doc_id {
                        tracing::warn!(
                            doc_id = %doc_id,
                            path = %self.path.display(),
                            "skipping corrupt pickledb snapshot while building document catalog"
                        );
                        continue;
                    }
                    documents.push(snapshot.document);
                }
                None => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt pickledb snapshot while building document catalog"
                ),
            }
        }

        Ok(documents)
    }
}
