use std::{
    io::ErrorKind,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use citadel::{Argon2Profile, Database, DatabaseBuilder, SyncMode};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

const SNAPSHOT_TABLE: &[u8] = b"snapshots";
const SNAPSHOT_CATALOG_KEY: &[u8] = b"__catalog__";

pub struct CitadeldbSnapshotStore {
    path: PathBuf,
    database: Mutex<Database>,
}

impl CitadeldbSnapshotStore {
    pub fn new(
        path: impl Into<PathBuf>,
        passphrase: impl AsRef<[u8]>,
    ) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_CITADELDB_PATH cannot be empty when SNAPSHOT_STORE=citadeldb".to_owned(),
            ));
        }

        let passphrase = passphrase.as_ref();
        if passphrase.is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_CITADELDB_PASSPHRASE cannot be empty when SNAPSHOT_STORE=citadeldb"
                    .to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            ensure_snapshot_dir(parent)?;
        }

        let database = Self::open_or_create_database(&path, passphrase)?;
        let store = Self {
            path,
            database: Mutex::new(database),
        };
        store.ensure_catalog()?;

        Ok(store)
    }

    fn open_or_create_database(
        path: &std::path::Path,
        passphrase: &[u8],
    ) -> Result<Database, StorageError> {
        let builder = || {
            DatabaseBuilder::new(path)
                .passphrase(passphrase)
                .argon2_profile(Argon2Profile::Iot)
                .sync_mode(SyncMode::Full)
        };

        if path.exists() {
            builder()
                .open()
                .map_err(|error| Self::map_error(path, error))
        } else {
            match builder().create() {
                Ok(database) => Ok(database),
                Err(citadel::Error::Io(error)) if error.kind() == ErrorKind::AlreadyExists => {
                    builder()
                        .open()
                        .map_err(|error| Self::map_error(path, error))
                }
                Err(error) => Err(Self::map_error(path, error)),
            }
        }
    }

    fn lock_database(&self) -> Result<MutexGuard<'_, Database>, StorageError> {
        self.database.lock().map_err(|_| {
            StorageError::Io(format!(
                "{}: citadeldb database mutex was poisoned",
                self.path.display()
            ))
        })
    }

    fn map_error(path: &std::path::Path, error: citadel::Error) -> StorageError {
        StorageError::Io(format!("{}: {error}", path.display()))
    }

    fn ensure_catalog(&self) -> Result<(), StorageError> {
        let database = self.lock_database()?;
        let mut transaction = database
            .begin_write()
            .map_err(|error| Self::map_error(&self.path, error))?;

        match transaction.create_table(SNAPSHOT_TABLE) {
            Ok(()) => {}
            Err(citadel::Error::TableAlreadyExists(_)) => {}
            Err(error) => return Err(Self::map_error(&self.path, error)),
        }

        if transaction
            .table_get(SNAPSHOT_TABLE, SNAPSHOT_CATALOG_KEY)
            .map_err(|error| Self::map_error(&self.path, error))?
            .is_none()
        {
            transaction
                .table_insert(SNAPSHOT_TABLE, SNAPSHOT_CATALOG_KEY, b"[]")
                .map_err(|error| Self::map_error(&self.path, error))?;
        }

        transaction
            .commit()
            .map_err(|error| Self::map_error(&self.path, error))
    }

    fn snapshot_key(doc_id: &Uuid) -> Vec<u8> {
        doc_id.to_string().into_bytes()
    }

    fn read_catalog_locked(&self, database: &Database) -> Result<Vec<Uuid>, StorageError> {
        let mut transaction = database.begin_read();
        let Some(payload) = transaction
            .table_get(SNAPSHOT_TABLE, SNAPSHOT_CATALOG_KEY)
            .map_err(|error| Self::map_error(&self.path, error))?
        else {
            return Ok(Vec::new());
        };

        serde_json::from_slice::<Vec<Uuid>>(&payload).map_err(|_| {
            StorageError::Io(format!(
                "{}: citadeldb snapshot catalog is corrupt",
                self.path.display()
            ))
        })
    }

    fn write_catalog_locked(
        &self,
        transaction: &mut citadel::txn::write_txn::WriteTxn<'_>,
        catalog: &[Uuid],
    ) -> Result<(), StorageError> {
        let payload = serde_json::to_vec(catalog).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize citadeldb snapshot catalog: {error}"
            ))
        })?;

        transaction
            .table_insert(SNAPSHOT_TABLE, SNAPSHOT_CATALOG_KEY, &payload)
            .map_err(|error| Self::map_error(&self.path, error))?;
        Ok(())
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

impl SnapshotStore for CitadeldbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let database = self.lock_database()?;
        let mut transaction = database.begin_read();
        let Some(payload) = transaction
            .table_get(SNAPSHOT_TABLE, &Self::snapshot_key(doc_id))
            .map_err(|error| Self::map_error(&self.path, error))?
        else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &payload).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize citadeldb snapshot `{doc_id}`: {error}"
            ))
        })?;
        let database = self.lock_database()?;
        let mut catalog = self.read_catalog_locked(&database)?;
        let mut transaction = database
            .begin_write()
            .map_err(|error| Self::map_error(&self.path, error))?;

        transaction
            .table_insert(SNAPSHOT_TABLE, &Self::snapshot_key(&doc_id), &payload)
            .map_err(|error| Self::map_error(&self.path, error))?;

        if !catalog.contains(&doc_id) {
            catalog.push(doc_id);
            catalog.sort_unstable();
            self.write_catalog_locked(&mut transaction, &catalog)?;
        }

        transaction
            .commit()
            .map_err(|error| Self::map_error(&self.path, error))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let database = self.lock_database()?;
        let mut catalog = self.read_catalog_locked(&database)?;
        let mut transaction = database
            .begin_write()
            .map_err(|error| Self::map_error(&self.path, error))?;

        transaction
            .table_delete(SNAPSHOT_TABLE, &Self::snapshot_key(doc_id))
            .map_err(|error| Self::map_error(&self.path, error))?;

        let original_len = catalog.len();
        catalog.retain(|catalog_doc_id| catalog_doc_id != doc_id);
        if catalog.len() != original_len {
            self.write_catalog_locked(&mut transaction, &catalog)?;
        }

        transaction
            .commit()
            .map_err(|error| Self::map_error(&self.path, error))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let database = self.lock_database()?;
        let catalog = self.read_catalog_locked(&database)?;
        let mut transaction = database.begin_read();
        let mut documents = Vec::new();

        for doc_id in catalog {
            let Some(payload) = transaction
                .table_get(SNAPSHOT_TABLE, &Self::snapshot_key(&doc_id))
                .map_err(|error| Self::map_error(&self.path, error))?
            else {
                tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing citadeldb snapshot referenced by catalog"
                );
                continue;
            };

            match self.deserialize_snapshot(doc_id, &payload) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt citadeldb snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
