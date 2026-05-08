use std::{fs, path::PathBuf};

use sanakirja::{
    Commit, Env, RootDb, RootPageMut,
    btree::{self, create_db_},
};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const ROOT_DB_INDEX: usize = 0;
const INITIAL_MAP_SIZE_BYTES: u64 = 1 << 20;
const ROOT_PAGE_COUNT: usize = 1;

type SanakirjaPage = sanakirja::btree::page_unsized::Page<[u8; 16], [u8]>;

pub struct SanakirjaSnapshotStore {
    path: PathBuf,
    env: Env,
}

impl SanakirjaSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_SANAKIRJA_PATH cannot be empty when SNAPSHOT_STORE=sanakirja".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let env = Env::new(&path, INITIAL_MAP_SIZE_BYTES, ROOT_PAGE_COUNT)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self { path, env })
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

impl SnapshotStore for SanakirjaSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let txn = Env::txn_begin(&self.env)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        let Some(db) = txn.root_db::<[u8; 16], [u8], SanakirjaPage>(ROOT_DB_INDEX) else {
            return Ok(None);
        };
        let Some((_, bytes)) = btree::get(&txn, &db, doc_id.as_bytes(), None)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?
        else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, bytes).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize sanakirja snapshot `{doc_id}`: {error}"
            ))
        })?;
        let mut txn = Env::mut_txn_begin(&self.env)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        let mut db = match txn.root_db::<[u8; 16], [u8], SanakirjaPage>(ROOT_DB_INDEX) {
            Some(db) => db,
            None => unsafe { create_db_::<_, [u8; 16], [u8], SanakirjaPage>(&mut txn) }
                .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?,
        };
        btree::put(&mut txn, &mut db, doc_id.as_bytes(), bytes.as_slice())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        txn.set_root(ROOT_DB_INDEX, db.db.into());
        txn.commit()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut txn = Env::mut_txn_begin(&self.env)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        let Some(mut db) = txn.root_db::<[u8; 16], [u8], SanakirjaPage>(ROOT_DB_INDEX) else {
            return Ok(());
        };
        btree::del(&mut txn, &mut db, doc_id.as_bytes(), None)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        txn.set_root(ROOT_DB_INDEX, db.db.into());
        txn.commit()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let txn = Env::txn_begin(&self.env)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        let Some(db) = txn.root_db::<[u8; 16], [u8], SanakirjaPage>(ROOT_DB_INDEX) else {
            return Ok(Vec::new());
        };
        let iter = btree::iter(&txn, &db, None)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        let mut documents = Vec::new();

        for entry in iter {
            let (key, bytes) = entry
                .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
            let doc_id = Uuid::from_bytes(*key);

            match self.deserialize_snapshot(doc_id, bytes) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt sanakirja snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
