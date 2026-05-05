use std::{fs, path::PathBuf};

use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

hmdb::schema! {
    SnapshotCatalogDb {
        snapshots: <String, PersistedSnapshot>
    }
}

pub struct HmdbSnapshotStore {
    path: PathBuf,
    database: SnapshotCatalogDb,
}

impl HmdbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_HMDB_PATH cannot be empty when SNAPSHOT_STORE=hmdb".to_owned(),
            ));
        }

        fs::create_dir_all(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        let database = SnapshotCatalogDb::init(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error:?}", path.display())))?;

        Ok(Self { path, database })
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

impl SnapshotStore for HmdbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let Some(snapshot) = self
            .database
            .snapshots
            .get(&doc_id.to_string())
            .map_err(|error| StorageError::Io(format!("{}: {error:?}", self.path.display())))?
        else {
            return Ok(None);
        };

        self.decode_snapshot(*doc_id, snapshot).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        self.database
            .snapshots
            .insert(doc_id.to_string(), PersistedSnapshot::from(snapshot))
            .map(|_| ())
            .map_err(|error| StorageError::Io(format!("{}: {error:?}", self.path.display())))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        self.database
            .snapshots
            .delete(doc_id.to_string())
            .map(|_| ())
            .map_err(|error| StorageError::Io(format!("{}: {error:?}", self.path.display())))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let snapshots = self
            .database
            .snapshots
            .get_all()
            .map_err(|error| StorageError::Io(format!("{}: {error:?}", self.path.display())))?;
        let mut documents = Vec::new();

        for (doc_id_key, snapshot) in snapshots {
            let Ok(doc_id) = Uuid::parse_str(&doc_id_key) else {
                continue;
            };

            match self.decode_snapshot(doc_id, snapshot) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt hmdb snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
