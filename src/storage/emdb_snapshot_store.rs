use std::{fs, path::PathBuf};

use emdb::{Emdb, FlushPolicy, Transaction};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOT_CATALOG_KEY: &[u8] = b"__catalog__";

pub struct EmdbSnapshotStore {
    path: PathBuf,
    database: Emdb,
}

impl EmdbSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_EMDB_PATH cannot be empty when SNAPSHOT_STORE=emdb".to_owned(),
            ));
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| StorageError::Io(format!("{}: {error}", parent.display())))?;
        }

        // Pin emdb to the v0.7 engine so the adapter always uses the newer
        // transaction + replay path instead of the legacy default backend.
        let database = Emdb::builder()
            .path(path.clone())
            .prefer_v4(true)
            .flush_policy(FlushPolicy::Manual)
            .build()
            .map_err(|error| Self::map_error(&path, "open emdb snapshot store", error))?;

        Ok(Self { path, database })
    }

    fn map_error(path: &std::path::Path, operation: &str, error: emdb::Error) -> StorageError {
        StorageError::Io(format!(
            "{}: failed to {operation}: {error}",
            path.display()
        ))
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

    fn load_catalog(&self) -> Result<Vec<String>, StorageError> {
        let Some(bytes) = self
            .database
            .get(SNAPSHOT_CATALOG_KEY)
            .map_err(|error| Self::map_error(&self.path, "read emdb snapshot catalog", error))?
        else {
            return Ok(Vec::new());
        };

        Self::deserialize_catalog(&self.path, &bytes)
    }

    fn load_catalog_in_tx_for_emdb(&self, tx: &Transaction<'_>) -> emdb::Result<Vec<String>> {
        let Some(bytes) = tx.get(SNAPSHOT_CATALOG_KEY)? else {
            return Ok(Vec::new());
        };

        serde_json::from_slice::<Vec<String>>(&bytes).map_err(|error| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("emdb snapshot catalog is corrupt: {error}"),
            )
            .into()
        })
    }

    fn deserialize_catalog(
        path: &std::path::Path,
        bytes: &[u8],
    ) -> Result<Vec<String>, StorageError> {
        serde_json::from_slice::<Vec<String>>(bytes).map_err(|_| {
            StorageError::Io(format!(
                "{}: emdb snapshot catalog is corrupt",
                path.display()
            ))
        })
    }

    fn serialize_catalog_for_emdb(catalog: &[String]) -> emdb::Result<Vec<u8>> {
        serde_json::to_vec(catalog).map_err(|error| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("failed to serialize emdb snapshot catalog: {error}"),
            )
            .into()
        })
    }

    fn flush(&self) -> Result<(), StorageError> {
        self.database
            .flush()
            .map_err(|error| Self::map_error(&self.path, "flush emdb snapshot store", error))
    }
}

impl SnapshotStore for EmdbSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        let Some(bytes) = self
            .database
            .get(doc_id.to_string().as_bytes())
            .map_err(|error| Self::map_error(&self.path, "read emdb snapshot", error))?
        else {
            return Ok(None);
        };

        self.deserialize_snapshot(*doc_id, &bytes).map(Some)
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let doc_id_key = doc_id.to_string();
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize emdb snapshot `{doc_id}`: {error}"
            ))
        })?;

        self.database
            .transaction(|tx| {
                let mut catalog = self.load_catalog_in_tx_for_emdb(tx)?;
                tx.insert(doc_id_key.as_bytes(), bytes.clone())?;

                if !catalog.iter().any(|entry| entry == &doc_id_key) {
                    catalog.push(doc_id_key.clone());
                    catalog.sort();
                }

                tx.insert(
                    SNAPSHOT_CATALOG_KEY,
                    Self::serialize_catalog_for_emdb(&catalog)?,
                )?;

                Ok(())
            })
            .map_err(|error| {
                Self::map_error(&self.path, "commit emdb snapshot transaction", error)
            })?;

        self.flush()
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let doc_id_key = doc_id.to_string();

        self.database
            .transaction(|tx| {
                let mut catalog = self.load_catalog_in_tx_for_emdb(tx)?;
                let _ = tx.remove(doc_id_key.as_bytes())?;

                catalog.retain(|entry| entry != &doc_id_key);
                tx.insert(
                    SNAPSHOT_CATALOG_KEY,
                    Self::serialize_catalog_for_emdb(&catalog)?,
                )?;

                Ok(())
            })
            .map_err(|error| {
                Self::map_error(&self.path, "commit emdb snapshot transaction", error)
            })?;

        self.flush()
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
                .get(doc_id_key.as_bytes())
                .map_err(|error| Self::map_error(&self.path, "read emdb snapshot", error))?
            {
                Some(bytes) => match self.deserialize_snapshot(doc_id, &bytes) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping corrupt emdb snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                },
                None => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing emdb snapshot while building document catalog"
                ),
            }
        }

        Ok(documents)
    }
}
