use std::{fs, path::PathBuf};

use heed::{
    Database, Env, EnvOpenOptions,
    types::{Bytes, Str},
};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOTS_DATABASE_NAME: Option<&str> = Some("snapshots");
const DEFAULT_MAP_SIZE_BYTES: usize = 128 * 1024 * 1024;

pub struct HeedSnapshotStore {
    path: PathBuf,
    env: Env,
    database: Database<Str, Bytes>,
}

impl HeedSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_HEED_PATH cannot be empty when SNAPSHOT_STORE=heed".to_owned(),
            ));
        }

        fs::create_dir_all(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        let mut options = EnvOpenOptions::new();
        options.map_size(DEFAULT_MAP_SIZE_BYTES).max_dbs(1);
        let env = unsafe { options.open(&path) }
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        let mut wtxn = env
            .write_txn()
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;
        let database = env
            .create_database::<Str, Bytes>(&mut wtxn, SNAPSHOTS_DATABASE_NAME)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;
        wtxn.commit()
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            env,
            database,
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
}

impl SnapshotStore for HeedSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let doc_id_key = doc_id.to_string();
        let rtxn = self
            .env
            .read_txn()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        let Some(value) = self
            .database
            .get(&rtxn, doc_id_key.as_str())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?
        else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, value).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let doc_id_key = doc_id.to_string();
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize heed snapshot `{doc_id}`: {error}"
            ))
        })?;

        let mut wtxn = self
            .env
            .write_txn()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        self.database
            .put(&mut wtxn, doc_id_key.as_str(), bytes.as_slice())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        wtxn.commit()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        Ok(())
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let doc_id_key = doc_id.to_string();
        let mut wtxn = self
            .env
            .write_txn()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        self.database
            .delete(&mut wtxn, doc_id_key.as_str())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        wtxn.commit()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        Ok(())
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let rtxn = self
            .env
            .read_txn()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        let mut documents = Vec::new();
        let iter = self
            .database
            .iter(&rtxn)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        for entry in iter {
            let (key, value) = entry
                .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
            let Ok(doc_id) = Uuid::parse_str(key) else {
                continue;
            };

            match self.deserialize_snapshot(doc_id, value) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt heed snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
