use std::{
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use lite_db::{
    db::{Adder, Config as LiteDbConfig, Db, ErrDb, Getter, Remover},
    lite::LiteDb,
};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const CATALOG_KEY: &[u8] = b"__catalog__";
const SNAPSHOT_KEY_PREFIX: &str = "snapshot:";

pub struct LiteDbSnapshotStore {
    path: PathBuf,
    db: Mutex<LiteDb>,
}

impl LiteDbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_LITE_DB_PATH cannot be empty when SNAPSHOT_STORE=lite_db".to_owned(),
            ));
        }

        let config = LiteDbConfig {
            path_db: path.clone(),
            sync_writes: true,
            mmap_at_startup: false,
            ..LiteDbConfig::default()
        };
        let db = LiteDb::open(config)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self {
            path,
            db: Mutex::new(db),
        })
    }

    fn snapshot_key(doc_id: &Uuid) -> Vec<u8> {
        format!("{SNAPSHOT_KEY_PREFIX}{doc_id}").into_bytes()
    }

    fn lock_db(&self) -> Result<MutexGuard<'_, LiteDb>, StorageError> {
        self.db.lock().map_err(|_| {
            StorageError::Io(format!("{}: lite_db mutex poisoned", self.path.display()))
        })
    }

    fn map_lite_db_error(&self, error: ErrDb) -> StorageError {
        StorageError::Io(format!("{}: {error}", self.path.display()))
    }

    fn read_catalog(&self, db: &LiteDb) -> Result<Vec<Uuid>, StorageError> {
        match db.get(&CATALOG_KEY.to_vec().into()) {
            Ok(payload) => serde_json::from_slice::<Vec<Uuid>>(&payload)
                .map_err(|_| StorageError::CorruptSnapshot(Uuid::nil())),
            Err(ErrDb::NotFindKey) => Ok(Vec::new()),
            Err(error) => Err(self.map_lite_db_error(error)),
        }
    }

    fn write_catalog(&self, db: &LiteDb, catalog: &[Uuid]) -> Result<(), StorageError> {
        let payload = serde_json::to_vec(catalog).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize lite_db snapshot catalog: {error}"
            ))
        })?;

        db.add(&CATALOG_KEY.to_vec().into(), &payload.into())
            .and_then(|_| db.sync())
            .map_err(|error| self.map_lite_db_error(error))
    }

    fn deserialize_snapshot(
        &self,
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

impl SnapshotStore for LiteDbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let db = self.lock_db()?;
        match db.get(&Self::snapshot_key(doc_id).into()) {
            Ok(payload) => self.deserialize_snapshot(*doc_id, &payload).map(Some),
            Err(ErrDb::NotFindKey) => Ok(None),
            Err(error) => Err(self.map_lite_db_error(error)),
        }
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize lite_db snapshot `{doc_id}`: {error}"
            ))
        })?;

        let db = self.lock_db()?;
        db.add(&Self::snapshot_key(&doc_id).into(), &payload.into())
            .map_err(|error| self.map_lite_db_error(error))?;

        let mut catalog = self.read_catalog(&db)?;
        if !catalog.contains(&doc_id) {
            catalog.push(doc_id);
            catalog.sort_unstable();
            self.write_catalog(&db, &catalog)?;
        } else {
            db.sync().map_err(|error| self.map_lite_db_error(error))?;
        }

        Ok(())
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let db = self.lock_db()?;
        db.remove_fast(&Self::snapshot_key(doc_id).into())
            .map_err(|error| self.map_lite_db_error(error))?;

        let mut catalog = self.read_catalog(&db)?;
        let original_len = catalog.len();
        catalog.retain(|catalog_doc_id| catalog_doc_id != doc_id);
        if catalog.len() != original_len {
            self.write_catalog(&db, &catalog)?;
        } else {
            db.sync().map_err(|error| self.map_lite_db_error(error))?;
        }

        Ok(())
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let db = self.lock_db()?;
        let catalog = self.read_catalog(&db)?;
        let mut documents = Vec::new();

        for doc_id in catalog {
            let payload = match db.get(&Self::snapshot_key(&doc_id).into()) {
                Ok(payload) => payload,
                Err(ErrDb::NotFindKey) => {
                    tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping missing lite_db snapshot referenced by catalog"
                    );
                    continue;
                }
                Err(error) => return Err(self.map_lite_db_error(error)),
            };

            match self.deserialize_snapshot(doc_id, &payload) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt lite_db snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
