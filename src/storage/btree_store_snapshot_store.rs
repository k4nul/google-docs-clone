use std::{fs, path::PathBuf};

use btree_store::{BTree, Error as BtreeStoreError};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOTS_BUCKET: &str = "snapshots";

pub struct BtreeStoreSnapshotStore {
    path: PathBuf,
    database: BTree,
}

impl BtreeStoreSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_BTREE_STORE_PATH cannot be empty when SNAPSHOT_STORE=btree_store"
                    .to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let database = BTree::open(&path)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self { path, database })
    }

    fn key(doc_id: &Uuid) -> &[u8] {
        doc_id.as_bytes()
    }

    fn map_error(&self, error: BtreeStoreError) -> StorageError {
        StorageError::Io(format!("{}: {error}", self.path.display()))
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

impl SnapshotStore for BtreeStoreSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let maybe_bytes =
            match self
                .database
                .view(SNAPSHOTS_BUCKET, |txn| match txn.get(Self::key(doc_id)) {
                    Ok(bytes) => Ok(Some(bytes)),
                    Err(BtreeStoreError::NotFound) => Ok(None),
                    Err(error) => Err(error),
                }) {
                Ok(maybe_bytes) => maybe_bytes,
                Err(BtreeStoreError::NotFound) => return Ok(None),
                Err(error) => return Err(self.map_error(error)),
            };

        maybe_bytes
            .map(|bytes| self.deserialize_snapshot(*doc_id, &bytes))
            .transpose()
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize btree_store snapshot `{doc_id}`: {error}"
            ))
        })?;

        self.database
            .exec(SNAPSHOTS_BUCKET, |txn| {
                txn.put(Self::key(&doc_id), bytes.as_slice())?;
                Ok(())
            })
            .map_err(|error| self.map_error(error))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let bucket_exists = match self.database.view(SNAPSHOTS_BUCKET, |_| Ok(())) {
            Ok(()) => true,
            Err(BtreeStoreError::NotFound) => false,
            Err(error) => return Err(self.map_error(error)),
        };

        if !bucket_exists {
            return Ok(());
        }

        self.database
            .exec(SNAPSHOTS_BUCKET, |txn| match txn.del(Self::key(doc_id)) {
                Ok(()) | Err(BtreeStoreError::NotFound) => Ok(()),
                Err(error) => Err(error),
            })
            .map_err(|error| self.map_error(error))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let entries = match self.database.view(SNAPSHOTS_BUCKET, |txn| {
            let mut iter = txn.iter();
            let mut key_buf = Vec::new();
            let mut value_buf = Vec::new();
            let mut entries = Vec::new();

            while iter.next_ref(&mut key_buf, &mut value_buf) {
                entries.push((key_buf.clone(), value_buf.clone()));
            }

            Ok(entries)
        }) {
            Ok(entries) => entries,
            Err(BtreeStoreError::NotFound) => return Ok(Vec::new()),
            Err(error) => return Err(self.map_error(error)),
        };

        let mut documents = Vec::new();
        for (key, value) in entries {
            let Ok(doc_id) = Uuid::from_slice(&key) else {
                continue;
            };

            match self.deserialize_snapshot(doc_id, &value) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt btree_store snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
