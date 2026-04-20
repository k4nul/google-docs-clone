use std::{
    fs,
    path::{Path, PathBuf},
};

use highlandcows_isam::{Isam, IsamError, Transaction};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOT_CATALOG_KEY: &str = "__catalog__";

#[derive(Debug, Clone, Serialize, Deserialize)]
enum HighlandcowsIsamRecord {
    Snapshot(PersistedSnapshot),
    Catalog(Vec<String>),
}

type HighlandcowsIsamDatabase = Isam<String, HighlandcowsIsamRecord>;

pub struct HighlandcowsIsamSnapshotStore {
    path: PathBuf,
    database: HighlandcowsIsamDatabase,
}

impl HighlandcowsIsamSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_HIGHLANDCOWS_ISAM_PATH cannot be empty when SNAPSHOT_STORE=highlandcows_isam"
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

        let database = if Self::data_path(&path).exists() || Self::index_path(&path).exists() {
            HighlandcowsIsamDatabase::open(&path)
        } else {
            HighlandcowsIsamDatabase::create(&path)
        }
        .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self { path, database })
    }

    fn data_path(path: &Path) -> PathBuf {
        path.with_extension("idb")
    }

    fn index_path(path: &Path) -> PathBuf {
        path.with_extension("idx")
    }

    fn map_database_error(&self, error: IsamError) -> StorageError {
        StorageError::Io(format!("{}: {error}", self.path.display()))
    }

    fn upsert_record(
        &self,
        txn: &mut Transaction<'_, String, HighlandcowsIsamRecord>,
        key: String,
        record: &HighlandcowsIsamRecord,
    ) -> Result<(), IsamError> {
        match self.database.get(txn, &key)? {
            Some(_) => self.database.update(txn, key, record),
            None => self.database.insert(txn, key, record),
        }
    }

    fn load_catalog(&self) -> Result<Vec<String>, StorageError> {
        let catalog_key = SNAPSHOT_CATALOG_KEY.to_owned();
        match self
            .database
            .read(|txn| self.database.get(txn, &catalog_key))
        {
            Ok(Some(HighlandcowsIsamRecord::Catalog(catalog))) => Ok(catalog),
            Ok(None) => Ok(Vec::new()),
            Ok(Some(HighlandcowsIsamRecord::Snapshot(_))) | Err(IsamError::Bincode(_)) => {
                Err(StorageError::Io(format!(
                    "{}: snapshot catalog is corrupt",
                    self.path.display()
                )))
            }
            Err(error) => Err(self.map_database_error(error)),
        }
    }
}

impl SnapshotStore for HighlandcowsIsamSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let doc_id_key = doc_id.to_string();
        let record = match self
            .database
            .read(|txn| self.database.get(txn, &doc_id_key))
        {
            Ok(record) => record,
            Err(IsamError::Bincode(_)) => return Err(StorageError::CorruptSnapshot(*doc_id)),
            Err(error) => return Err(self.map_database_error(error)),
        };

        let Some(record) = record else {
            return Ok(None);
        };

        let HighlandcowsIsamRecord::Snapshot(snapshot) = record else {
            return Err(StorageError::CorruptSnapshot(*doc_id));
        };

        let snapshot: DocumentSnapshot = snapshot.into();
        if snapshot.document.id != *doc_id {
            return Err(StorageError::CorruptSnapshot(*doc_id));
        }

        Ok(Some(snapshot))
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let doc_id_key = doc_id.to_string();
        let snapshot_record = HighlandcowsIsamRecord::Snapshot(PersistedSnapshot::from(snapshot));
        let catalog_key = SNAPSHOT_CATALOG_KEY.to_owned();

        self.database
            .write(|txn| {
                let mut catalog = match self.database.get(txn, &catalog_key)? {
                    Some(HighlandcowsIsamRecord::Catalog(catalog)) => catalog,
                    None => Vec::new(),
                    Some(HighlandcowsIsamRecord::Snapshot(_)) => {
                        return Err(IsamError::CorruptIndex(
                            "snapshot catalog key stored snapshot payload".to_owned(),
                        ));
                    }
                };

                self.upsert_record(txn, doc_id_key.clone(), &snapshot_record)?;

                if !catalog.iter().any(|entry| entry == &doc_id_key) {
                    catalog.push(doc_id_key);
                    catalog.sort();
                }

                self.upsert_record(
                    txn,
                    SNAPSHOT_CATALOG_KEY.to_owned(),
                    &HighlandcowsIsamRecord::Catalog(catalog),
                )
            })
            .map_err(|error| self.map_database_error(error))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let doc_id_key = doc_id.to_string();
        let catalog_key = SNAPSHOT_CATALOG_KEY.to_owned();

        self.database
            .write(|txn| {
                let mut catalog = match self.database.get(txn, &catalog_key)? {
                    Some(HighlandcowsIsamRecord::Catalog(catalog)) => catalog,
                    None => Vec::new(),
                    Some(HighlandcowsIsamRecord::Snapshot(_)) => {
                        return Err(IsamError::CorruptIndex(
                            "snapshot catalog key stored snapshot payload".to_owned(),
                        ));
                    }
                };

                if self.database.get(txn, &doc_id_key)?.is_some() {
                    self.database.delete(txn, &doc_id_key)?;
                }

                catalog.retain(|entry| entry != &doc_id_key);

                self.upsert_record(
                    txn,
                    SNAPSHOT_CATALOG_KEY.to_owned(),
                    &HighlandcowsIsamRecord::Catalog(catalog),
                )
            })
            .map_err(|error| self.map_database_error(error))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let catalog = self.load_catalog()?;
        let mut documents = Vec::new();

        for doc_id_key in catalog {
            let Ok(doc_id) = Uuid::parse_str(&doc_id_key) else {
                continue;
            };

            match self
                .database
                .read(|txn| self.database.get(txn, &doc_id_key))
            {
                Ok(Some(HighlandcowsIsamRecord::Snapshot(snapshot))) => {
                    let snapshot: DocumentSnapshot = snapshot.into();
                    if snapshot.document.id != doc_id {
                        tracing::warn!(
                            doc_id = %doc_id,
                            path = %self.path.display(),
                            "skipping corrupt highlandcows-isam snapshot while building document catalog"
                        );
                        continue;
                    }

                    documents.push(snapshot.document);
                }
                Ok(Some(HighlandcowsIsamRecord::Catalog(_))) | Err(IsamError::Bincode(_)) => {
                    tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping corrupt highlandcows-isam snapshot while building document catalog"
                    );
                }
                Ok(None) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing highlandcows-isam snapshot while building document catalog"
                ),
                Err(error) => return Err(self.map_database_error(error)),
            }
        }

        Ok(documents)
    }
}
