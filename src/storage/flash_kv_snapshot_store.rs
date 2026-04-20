use std::path::PathBuf;

use flash_kv::{
    db::Engine,
    errors::Errors,
    option::{IndexType, Options},
};
use uuid::Uuid;

use crate::{
    models::document::Document,
    storage::{DocumentSnapshot, PersistedSnapshot, SnapshotStore, StorageError},
};

const SNAPSHOT_CATALOG_KEY: &str = "__catalog__";

pub struct FlashKvSnapshotStore {
    path: PathBuf,
    engine: Engine,
}

impl FlashKvSnapshotStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(StorageError::Config(
                "SNAPSHOT_FLASH_KV_PATH cannot be empty when SNAPSHOT_STORE=flash_kv".to_owned(),
            ));
        }

        let options = Options {
            dir_path: path.clone(),
            sync_writes: true,
            index_type: IndexType::BTree,
            ..Options::default()
        };
        let engine = Engine::open(options)
            .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))?;

        Ok(Self { path, engine })
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
        match self
            .engine
            .get(SNAPSHOT_CATALOG_KEY.as_bytes().to_vec().into())
        {
            Ok(bytes) => serde_json::from_slice::<Vec<String>>(bytes.as_ref()).map_err(|_| {
                StorageError::Io(format!(
                    "{}: snapshot catalog is corrupt",
                    self.path.display()
                ))
            }),
            Err(Errors::KeyNotFound) => Ok(Vec::new()),
            Err(error) => Err(StorageError::Io(format!(
                "{}: {error}",
                self.path.display()
            ))),
        }
    }

    fn save_catalog(&self, catalog: &[String]) -> Result<(), StorageError> {
        let bytes = serde_json::to_vec(catalog)
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        self.engine
            .put(
                SNAPSHOT_CATALOG_KEY.as_bytes().to_vec().into(),
                bytes.into(),
            )
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }
}

impl SnapshotStore for FlashKvSnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        match self.engine.get(doc_id.to_string().into_bytes().into()) {
            Ok(bytes) => self.deserialize_snapshot(*doc_id, bytes.as_ref()).map(Some),
            Err(Errors::KeyNotFound) => Ok(None),
            Err(error) => Err(StorageError::Io(format!(
                "{}: {error}",
                self.path.display()
            ))),
        }
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        let doc_id = snapshot.document.id;
        let doc_id_key = doc_id.to_string();
        let bytes = serde_json::to_vec(&PersistedSnapshot::from(snapshot)).map_err(|error| {
            StorageError::Io(format!(
                "failed to serialize flash_kv snapshot `{doc_id}`: {error}"
            ))
        })?;
        let mut catalog = self.load_catalog()?;

        self.engine
            .put(doc_id_key.as_bytes().to_vec().into(), bytes.into())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;

        if !catalog.iter().any(|value| value == &doc_id_key) {
            catalog.push(doc_id_key);
            catalog.sort();
        }

        self.save_catalog(&catalog)?;
        self.engine
            .sync()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        let doc_id_key = doc_id.to_string();
        let mut catalog = self.load_catalog()?;

        self.engine
            .delete(doc_id_key.as_bytes().to_vec().into())
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))?;
        catalog.retain(|value| value != &doc_id_key);

        self.save_catalog(&catalog)?;
        self.engine
            .sync()
            .map_err(|error| StorageError::Io(format!("{}: {error}", self.path.display())))
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        let mut documents = Vec::new();
        let catalog = self.load_catalog()?;

        for doc_id_key in catalog {
            let Ok(doc_id) = Uuid::parse_str(&doc_id_key) else {
                continue;
            };

            match self.engine.get(doc_id_key.as_bytes().to_vec().into()) {
                Ok(bytes) => match self.deserialize_snapshot(doc_id, bytes.as_ref()) {
                    Ok(snapshot) => documents.push(snapshot.document),
                    Err(StorageError::CorruptSnapshot(doc_id)) => tracing::warn!(
                        doc_id = %doc_id,
                        path = %self.path.display(),
                        "skipping corrupt flash_kv snapshot while building document catalog"
                    ),
                    Err(error) => return Err(error),
                },
                Err(Errors::KeyNotFound) => tracing::warn!(
                    doc_id = %doc_id,
                    path = %self.path.display(),
                    "skipping missing flash_kv snapshot while building document catalog"
                ),
                Err(error) => {
                    return Err(StorageError::Io(format!(
                        "{}: {error}",
                        self.path.display()
                    )));
                }
            }
        }

        Ok(documents)
    }
}
