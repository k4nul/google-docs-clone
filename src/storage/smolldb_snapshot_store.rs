use std::{
    fs::{self, File},
    io::ErrorKind,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use smolldb::{DataType, SmollDB};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const CATALOG_KEY: &str = "__catalog__";
const SNAPSHOT_KEY_PREFIX: &str = "snapshot:";

pub struct SmolldbSnapshotStore {
    path: PathBuf,
    db: Mutex<SmollDB>,
}

impl SmolldbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_SMOLLDB_PATH cannot be empty when SNAPSHOT_STORE=smolldb".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        let db = match File::open(&path) {
            Ok(mut file) => SmollDB::load_from_stream(&mut file)
                .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?,
            Err(error) if error.kind() == ErrorKind::NotFound => SmollDB::default(),
            Err(error) => return Err(StorageError::Io(format!("{}: {error}", path.display()))),
        };

        Ok(Self {
            path,
            db: Mutex::new(db),
        })
    }

    fn snapshot_key(doc_id: &Uuid) -> String {
        format!("{SNAPSHOT_KEY_PREFIX}{doc_id}")
    }

    fn lock_db(&self) -> Result<MutexGuard<'_, SmollDB>, StorageError> {
        self.db.lock().map_err(|_| {
            StorageError::Io(format!("{}: smolldb mutex poisoned", self.path.display()))
        })
    }

    fn persist(&self, db: &SmollDB) -> Result<(), StorageError> {
        let temp_path = self.path.with_extension("tmp");
        {
            let mut temp_file = File::create(&temp_path)
                .map_err(|error| StorageError::Io(format!("{}: {error}", temp_path.display())))?;
            db.backup_to_stream(&mut temp_file)
                .map_err(|error| StorageError::Io(format!("{}: {error}", temp_path.display())))?;
            temp_file
                .sync_all()
                .map_err(|error| StorageError::Io(format!("{}: {error}", temp_path.display())))?;
        }

        fs::rename(&temp_path, &self.path).map_err(|error| {
            StorageError::Io(format!(
                "failed to replace smolldb snapshot file {} with {}: {error}",
                self.path.display(),
                temp_path.display()
            ))
        })
    }

    fn bytes_for_key<'a>(
        &'a self,
        db: &'a SmollDB,
        key: &str,
        expected_doc_id: Uuid,
    ) -> Result<Option<&'a Vec<u8>>, StorageError> {
        match db.get(&key) {
            Some(DataType::BYTES(payload)) => Ok(Some(payload)),
            Some(_) => Err(StorageError::CorruptSnapshot(expected_doc_id)),
            None => Ok(None),
        }
    }

    fn read_catalog(&self, db: &SmollDB) -> Result<Vec<Uuid>, StorageError> {
        let Some(payload) = self.bytes_for_key(db, CATALOG_KEY, Uuid::nil())? else {
            return Ok(Vec::new());
        };

        serde_json::from_slice::<Vec<Uuid>>(payload)
            .map_err(|_| StorageError::CorruptSnapshot(Uuid::nil()))
    }

    fn write_catalog(&self, db: &mut SmollDB, catalog: &[Uuid]) -> Result<(), StorageError> {
        let payload = serde_json::to_vec(catalog).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize smolldb snapshot catalog: {error}"
            ))
        })?;
        db.set(CATALOG_KEY, payload);
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

impl SnapshotStore for SmolldbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let db = self.lock_db()?;
        let Some(payload) = self.bytes_for_key(&db, &Self::snapshot_key(doc_id), *doc_id)? else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, payload).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let payload = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize smolldb snapshot `{doc_id}`: {error}"
            ))
        })?;

        let mut db = self.lock_db()?;
        db.set(Self::snapshot_key(&doc_id), payload);

        let mut catalog = self.read_catalog(&db)?;
        if !catalog.contains(&doc_id) {
            catalog.push(doc_id);
            catalog.sort_unstable();
            self.write_catalog(&mut db, &catalog)?;
        }

        self.persist(&db)
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let mut db = self.lock_db()?;
        db.remove(&Self::snapshot_key(doc_id));

        let mut catalog = self.read_catalog(&db)?;
        let original_len = catalog.len();
        catalog.retain(|catalog_doc_id| catalog_doc_id != doc_id);
        if catalog.len() != original_len {
            self.write_catalog(&mut db, &catalog)?;
        }

        self.persist(&db)
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let db = self.lock_db()?;
        let catalog = self.read_catalog(&db)?;
        let mut documents = Vec::new();

        for doc_id in catalog {
            let Some(payload) = self.bytes_for_key(&db, &Self::snapshot_key(&doc_id), doc_id)?
            else {
                tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing smolldb snapshot referenced by catalog"
                );
                continue;
            };

            match self.deserialize_snapshot(doc_id, payload) {
                Ok(snapshot) => documents.push(snapshot.document),
                Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping corrupt smolldb snapshot while building document catalog"
                ),
                Err(error) => return Err(error),
            }
        }

        Ok(documents)
    }
}
