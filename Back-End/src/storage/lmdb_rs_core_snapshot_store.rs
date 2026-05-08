use std::{
    fs,
    path::{Path, PathBuf},
};

use lmdb_rs_core::{
    env::Environment,
    error::Error as LmdbError,
    types::{MAIN_DBI, WriteFlags},
};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{
        DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError, ensure_snapshot_dir,
    },
};

const SNAPSHOT_CATALOG_KEY: &[u8] = b"__catalog__";
const SNAPSHOT_KEY_PREFIX: &str = "snapshot:";
const MAP_SIZE_BYTES: usize = 64 * 1024 * 1024;

pub struct LmdbRsCoreSnapshotStore {
    path: PathBuf,
    env: Environment,
}

impl LmdbRsCoreSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = normalize_path(path.into())?;
        ensure_snapshot_dir(&path)?;

        let env = Environment::builder()
            .map_size(MAP_SIZE_BYTES)
            .open(&path)
            .map_err(|error| map_lmdb_error(&path, error))?;

        sync_parent_dir(&path)?;

        Ok(Self { path, env })
    }

    fn snapshot_key(doc_id: &Uuid) -> Vec<u8> {
        format!("{SNAPSHOT_KEY_PREFIX}{doc_id}").into_bytes()
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

    fn read_catalog_from_txn(
        &self,
        txn: &lmdb_rs_core::write::RwTransaction<'_>,
    ) -> Result<Vec<Uuid>, StorageError> {
        match txn.get(MAIN_DBI, SNAPSHOT_CATALOG_KEY) {
            Ok(payload) => serde_json::from_slice::<Vec<Uuid>>(payload).map_err(|_| {
                StorageError::Io(format!(
                    "{}: lmdb-rs-core snapshot catalog is corrupt",
                    self.path.display()
                ))
            }),
            Err(LmdbError::NotFound) => Ok(Vec::new()),
            Err(error) => Err(map_lmdb_error(&self.path, error)),
        }
    }

    fn write_catalog_to_txn(
        &self,
        txn: &mut lmdb_rs_core::write::RwTransaction<'_>,
        catalog: &[Uuid],
    ) -> Result<(), StorageError> {
        let payload = serde_json::to_vec(catalog).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize lmdb-rs-core snapshot catalog: {error}"
            ))
        })?;
        txn.put(
            MAIN_DBI,
            SNAPSHOT_CATALOG_KEY,
            &payload,
            WriteFlags::empty(),
        )
        .map_err(|error| map_lmdb_error(&self.path, error))
    }
}

impl SnapshotStore for LmdbRsCoreSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| map_lmdb_error(&self.path, error))?;

        match txn.get(MAIN_DBI, &Self::snapshot_key(doc_id)) {
            Ok(payload) => self.deserialize_snapshot(*doc_id, payload).map(Some),
            Err(LmdbError::NotFound) => Ok(None),
            Err(error) => Err(map_lmdb_error(&self.path, error)),
        }
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize lmdb-rs-core snapshot `{doc_id}`: {error}"
            ))
        })?;

        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| map_lmdb_error(&self.path, error))?;
        txn.put(
            MAIN_DBI,
            &Self::snapshot_key(&doc_id),
            &payload,
            WriteFlags::empty(),
        )
        .map_err(|error| map_lmdb_error(&self.path, error))?;

        let mut catalog = self.read_catalog_from_txn(&txn)?;
        if !catalog.contains(&doc_id) {
            catalog.push(doc_id);
            catalog.sort_unstable();
            self.write_catalog_to_txn(&mut txn, &catalog)?;
        }

        txn.commit()
            .map_err(|error| map_lmdb_error(&self.path, error))?;
        self.env
            .sync(true)
            .map_err(|error| map_lmdb_error(&self.path, error))?;
        sync_parent_dir(&self.path)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| map_lmdb_error(&self.path, error))?;
        match txn.del(MAIN_DBI, &Self::snapshot_key(doc_id), None) {
            Ok(()) | Err(LmdbError::NotFound) => {}
            Err(error) => return Err(map_lmdb_error(&self.path, error)),
        }

        let mut catalog = self.read_catalog_from_txn(&txn)?;
        let original_len = catalog.len();
        catalog.retain(|catalog_doc_id| catalog_doc_id != doc_id);
        if catalog.len() != original_len {
            self.write_catalog_to_txn(&mut txn, &catalog)?;
        }

        txn.commit()
            .map_err(|error| map_lmdb_error(&self.path, error))?;
        self.env
            .sync(true)
            .map_err(|error| map_lmdb_error(&self.path, error))?;
        sync_parent_dir(&self.path)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| map_lmdb_error(&self.path, error))?;
        let catalog = match txn.get(MAIN_DBI, SNAPSHOT_CATALOG_KEY) {
            Ok(payload) => serde_json::from_slice::<Vec<Uuid>>(payload).map_err(|_| {
                StorageError::Io(format!(
                    "{}: lmdb-rs-core snapshot catalog is corrupt",
                    self.path.display()
                ))
            })?,
            Err(LmdbError::NotFound) => Vec::new(),
            Err(error) => return Err(map_lmdb_error(&self.path, error)),
        };
        let mut documents = Vec::new();

        for doc_id in catalog {
            match txn.get(MAIN_DBI, &Self::snapshot_key(&doc_id)) {
                Ok(payload) => documents.push(self.deserialize_snapshot(doc_id, payload)?.document),
                Err(LmdbError::NotFound) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing lmdb-rs-core snapshot referenced by catalog"
                ),
                Err(error) => return Err(map_lmdb_error(&self.path, error)),
            }
        }

        Ok(documents)
    }
}

fn normalize_path(path: PathBuf) -> Result<PathBuf, StorageError> {
    if path.as_os_str().is_empty() {
        return Err(StorageError::Config(
            "SNAPSHOT_LMDB_RS_CORE_PATH cannot be empty when SNAPSHOT_STORE=lmdb_rs_core"
                .to_owned(),
        ));
    }

    if path
        .parent()
        .is_some_and(|parent| !parent.as_os_str().is_empty())
    {
        Ok(path)
    } else {
        Ok(PathBuf::from(".").join(path))
    }
}

fn sync_parent_dir(path: &Path) -> Result<(), StorageError> {
    let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    else {
        return Ok(());
    };

    fs::OpenOptions::new()
        .read(true)
        .open(parent)
        .and_then(|file| file.sync_all())
        .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))
}

fn map_lmdb_error(path: &Path, error: LmdbError) -> StorageError {
    StorageError::Io(format!("{}: {error}", path.display()))
}
