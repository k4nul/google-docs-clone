use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use crepedb::{CrepeDB, backend::RedbDatabase, types::SnapshotId};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOTS_TABLE: &str = "snapshots";
const SNAPSHOT_CATALOG_KEY: &[u8] = b"__catalog__";
const SNAPSHOT_KEY_PREFIX: &str = "snapshot:";

pub struct CrepeDbSnapshotStore {
    path: PathBuf,
    database: Mutex<CrepeDB<RedbDatabase>>,
}

impl CrepeDbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_CREPEDB_PATH cannot be empty when SNAPSHOT_STORE=crepedb".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let backend = RedbDatabase::open_or_create(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;
        let database = CrepeDB::new(backend);
        let store = Self {
            path,
            database: Mutex::new(database),
        };
        store.initialize_schema()?;
        Ok(store)
    }

    fn snapshot_key(doc_id: &Uuid) -> Vec<u8> {
        format!("{SNAPSHOT_KEY_PREFIX}{doc_id}").into_bytes()
    }

    fn lock_database(&self) -> Result<MutexGuard<'_, CrepeDB<RedbDatabase>>, StorageError> {
        self.database.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: crepedb database mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn map_error(&self, operation: &str, error: impl std::fmt::Debug) -> StorageError {
        StorageError::Io(format!(
            "{}: failed to {operation}: {error:?}",
            self.path.display()
        ))
    }

    fn initialize_schema(&self) -> Result<(), StorageError> {
        let database = self.lock_database()?;
        match database.write(None) {
            Ok(txn) => {
                txn.create_basic_table(SNAPSHOTS_TABLE)
                    .map_err(|error| self.map_error("create crepedb snapshots table", error))?;
                {
                    let mut table = txn
                        .open_table(SNAPSHOTS_TABLE)
                        .map_err(|error| self.map_error("open crepedb snapshots table", error))?;
                    table
                        .set(SNAPSHOT_CATALOG_KEY.to_vec(), Vec::new())
                        .map_err(|error| {
                            self.map_error("initialize crepedb snapshot catalog", error)
                        })?;
                }
                txn.commit()
                    .map_err(|error| self.map_error("commit crepedb schema", error))?;
                Self::initialize_snapshot_index(&database)
                    .map_err(|error| self.map_error("initialize crepedb snapshot index", error))?;
            }
            Err(crepedb::Error::OnlySupportOneRoot) => {
                if database
                    .read(Some(SnapshotId::root()))
                    .and_then(|txn| txn.open_table(SNAPSHOTS_TABLE).map(drop))
                    .is_err()
                {
                    let txn = database.write(Some(SnapshotId::root())).map_err(|error| {
                        self.map_error("open crepedb schema transaction", error)
                    })?;
                    txn.create_basic_table(SNAPSHOTS_TABLE)
                        .map_err(|error| self.map_error("create crepedb snapshots table", error))?;
                    {
                        let mut table = txn.open_table(SNAPSHOTS_TABLE).map_err(|error| {
                            self.map_error("open crepedb snapshots table", error)
                        })?;
                        table
                            .set(SNAPSHOT_CATALOG_KEY.to_vec(), Vec::new())
                            .map_err(|error| {
                                self.map_error("initialize crepedb snapshot catalog", error)
                            })?;
                    }
                    txn.commit()
                        .map_err(|error| self.map_error("commit crepedb schema", error))?;
                }
                if database
                    .read(Some(SnapshotId::root()))
                    .and_then(|txn| txn.open_table(SNAPSHOTS_TABLE).map(drop))
                    .is_err()
                {
                    Self::initialize_snapshot_index(&database).map_err(|error| {
                        self.map_error("initialize crepedb snapshot index", error)
                    })?;
                }
            }
            Err(error) => return Err(self.map_error("initialize crepedb snapshot store", error)),
        }

        Ok(())
    }

    fn initialize_snapshot_index(database: &CrepeDB<RedbDatabase>) -> Result<(), crepedb::Error> {
        database.write(Some(SnapshotId::root()))?.commit().map(drop)
    }

    fn read_catalog(&self, database: &CrepeDB<RedbDatabase>) -> Result<Vec<Uuid>, StorageError> {
        let txn = database
            .read(Some(SnapshotId::root()))
            .map_err(|error| self.map_error("open crepedb read transaction", error))?;
        let table = txn
            .open_table(SNAPSHOTS_TABLE)
            .map_err(|error| self.map_error("open crepedb snapshots table", error))?;
        let Some(payload) = table
            .get(SNAPSHOT_CATALOG_KEY.to_vec())
            .map_err(|error| self.map_error("read crepedb snapshot catalog", error))?
        else {
            return Ok(Vec::new());
        };

        if payload.is_empty() {
            return Ok(Vec::new());
        }

        serde_json::from_slice::<Vec<Uuid>>(&payload).map_err(|_| {
            StorageError::Io(format!(
                "{}: crepedb snapshot catalog is corrupt",
                self.path.display()
            ))
        })
    }

    fn write_entries(
        &self,
        database: &CrepeDB<RedbDatabase>,
        entries: Vec<(Vec<u8>, Option<Vec<u8>>)>,
    ) -> Result<(), StorageError> {
        let txn = database
            .write(Some(SnapshotId::root()))
            .map_err(|error| self.map_error("open crepedb write transaction", error))?;
        {
            let mut table = txn
                .open_table(SNAPSHOTS_TABLE)
                .map_err(|error| self.map_error("open crepedb snapshots table", error))?;
            for (key, value) in entries {
                match value {
                    Some(value) => table
                        .set(key, value)
                        .map_err(|error| self.map_error("write crepedb snapshot", error))?,
                    None => table
                        .del(key)
                        .map_err(|error| self.map_error("delete crepedb snapshot", error))?,
                }
            }
        }
        txn.commit()
            .map_err(|error| self.map_error("commit crepedb snapshot transaction", error))
            .map(drop)
    }

    fn load_persisted_snapshot(
        &self,
        database: &CrepeDB<RedbDatabase>,
        expected_doc_id: Uuid,
    ) -> Result<Option<DocumentSnapshot>, StorageError> {
        let txn = database
            .read(Some(SnapshotId::root()))
            .map_err(|error| self.map_error("open crepedb read transaction", error))?;
        let table = txn
            .open_table(SNAPSHOTS_TABLE)
            .map_err(|error| self.map_error("open crepedb snapshots table", error))?;
        let Some(payload) = table
            .get(Self::snapshot_key(&expected_doc_id))
            .map_err(|error| self.map_error("read crepedb snapshot", error))?
        else {
            return Ok(None);
        };

        self.deserialize_snapshot(expected_doc_id, &payload)
            .map(Some)
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

impl SnapshotStore for CrepeDbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let database = self.lock_database()?;
        self.load_persisted_snapshot(&database, *doc_id)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let database = self.lock_database()?;
        let doc_id = snapshot.document.id;
        let payload = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize crepedb snapshot `{doc_id}`: {error}"
            ))
        })?;

        let mut catalog = self.read_catalog(&database)?;
        if !catalog.contains(&doc_id) {
            catalog.push(doc_id);
            catalog.sort_unstable();
        }
        let catalog_payload = serde_json::to_vec(&catalog).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize crepedb snapshot catalog: {error}"
            ))
        })?;

        self.write_entries(
            &database,
            vec![
                (Self::snapshot_key(&doc_id), Some(payload)),
                (SNAPSHOT_CATALOG_KEY.to_vec(), Some(catalog_payload)),
            ],
        )
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let database = self.lock_database()?;
        let mut catalog = self.read_catalog(&database)?;
        let original_len = catalog.len();
        catalog.retain(|catalog_doc_id| catalog_doc_id != doc_id);
        let catalog_payload = serde_json::to_vec(&catalog).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize crepedb snapshot catalog: {error}"
            ))
        })?;

        let mut entries = vec![(Self::snapshot_key(doc_id), None)];
        if catalog.len() != original_len {
            entries.push((SNAPSHOT_CATALOG_KEY.to_vec(), Some(catalog_payload)));
        }

        self.write_entries(&database, entries)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let database = self.lock_database()?;
        let catalog = self.read_catalog(&database)?;
        let mut documents = Vec::new();

        for doc_id in catalog {
            match self.load_persisted_snapshot(&database, doc_id) {
                Ok(Some(snapshot)) => documents.push(snapshot.document),
                Ok(None) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing crepedb snapshot referenced by catalog"
                ),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt crepedb snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
