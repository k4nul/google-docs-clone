mod abyssiniandb_snapshot_store;
mod aeternusdb_snapshot_store;
mod bitask_snapshot_store;
mod btree_store_snapshot_store;
mod canopydb_snapshot_store;
mod caves_snapshot_store;
mod ckydb_snapshot_store;
mod dbless_snapshot_store;
mod dblite_snapshot_store;
mod docdb_snapshot_store;
mod file_snapshot_store;
mod fjall_snapshot_store;
mod flash_kv_snapshot_store;
mod heed_snapshot_store;
mod hightower_kv_snapshot_store;
mod hmdb_snapshot_store;
mod jammdb_snapshot_store;
mod jsondb_snapshot_store;
mod managed_snapshot_store;
mod microkv_snapshot_store;
mod native_db_snapshot_store;
mod nikidb_snapshot_store;
mod nodb_snapshot_store;
mod parity_db_snapshot_store;
mod persistent_kv_snapshot_store;
mod persy_snapshot_store;
mod pickledb_snapshot_store;
mod readb_snapshot_store;
mod redb_snapshot_store;
mod rskey_snapshot_store;
mod rustbreak_snapshot_store;
mod rustlite_snapshot_store;
mod s3_snapshot_store;
mod saberdb_snapshot_store;
mod sanakirja_snapshot_store;
mod scdb_snapshot_store;
mod shorterdb_snapshot_store;
mod siamesedb_snapshot_store;
mod simple_db_snapshot_store;
mod sled_snapshot_store;
mod snaildb_snapshot_store;
mod sqlite_snapshot_store;
mod structsy_snapshot_store;
mod surrealkv_snapshot_store;
mod thunderdb_snapshot_store;
mod tinykv_snapshot_store;
mod yedb_snapshot_store;

use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::{config::Config, models::document::Document};

pub use abyssiniandb_snapshot_store::AbyssiniandbSnapshotStore;
pub use aeternusdb_snapshot_store::AeternusdbSnapshotStore;
pub use bitask_snapshot_store::BitaskSnapshotStore;
pub use btree_store_snapshot_store::BtreeStoreSnapshotStore;
pub use canopydb_snapshot_store::CanopydbSnapshotStore;
pub use caves_snapshot_store::CavesSnapshotStore;
pub use ckydb_snapshot_store::CkydbSnapshotStore;
pub use dbless_snapshot_store::DblessSnapshotStore;
pub use dblite_snapshot_store::DbliteSnapshotStore;
pub use docdb_snapshot_store::DocDbSnapshotStore;
pub use file_snapshot_store::FileSnapshotStore;
pub use fjall_snapshot_store::FjallSnapshotStore;
pub use flash_kv_snapshot_store::FlashKvSnapshotStore;
pub use heed_snapshot_store::HeedSnapshotStore;
pub use hightower_kv_snapshot_store::HightowerKvSnapshotStore;
pub use hmdb_snapshot_store::HmdbSnapshotStore;
pub use jammdb_snapshot_store::JammdbSnapshotStore;
pub use jsondb_snapshot_store::JsondbSnapshotStore;
pub use managed_snapshot_store::ManagedSnapshotStore;
pub use microkv_snapshot_store::MicroKvSnapshotStore;
pub use native_db_snapshot_store::NativeDbSnapshotStore;
pub use nikidb_snapshot_store::NikidbSnapshotStore;
pub use nodb_snapshot_store::NodbSnapshotStore;
pub use parity_db_snapshot_store::ParityDbSnapshotStore;
pub use persistent_kv_snapshot_store::PersistentKvSnapshotStore;
pub use persy_snapshot_store::PersySnapshotStore;
pub use pickledb_snapshot_store::PickleDbSnapshotStore;
pub use readb_snapshot_store::ReadbSnapshotStore;
pub use redb_snapshot_store::RedbSnapshotStore;
pub use rskey_snapshot_store::RskeySnapshotStore;
pub use rustbreak_snapshot_store::RustbreakSnapshotStore;
pub use rustlite_snapshot_store::RustliteSnapshotStore;
pub use s3_snapshot_store::S3SnapshotStore;
pub use saberdb_snapshot_store::SaberdbSnapshotStore;
pub use sanakirja_snapshot_store::SanakirjaSnapshotStore;
pub use scdb_snapshot_store::ScdbSnapshotStore;
pub use shorterdb_snapshot_store::ShorterDbSnapshotStore;
pub use siamesedb_snapshot_store::SiamesedbSnapshotStore;
pub use simple_db_snapshot_store::SimpleDbSnapshotStore;
pub use sled_snapshot_store::SledSnapshotStore;
pub use snaildb_snapshot_store::SnaildbSnapshotStore;
pub use sqlite_snapshot_store::SqliteSnapshotStore;
pub use structsy_snapshot_store::StructsySnapshotStore;
pub use surrealkv_snapshot_store::SurrealkvSnapshotStore;
pub use thunderdb_snapshot_store::ThunderdbSnapshotStore;
pub use tinykv_snapshot_store::TinykvSnapshotStore;
pub use yedb_snapshot_store::YedbSnapshotStore;

#[derive(Debug, Clone)]
pub struct DocumentSnapshot {
    pub document: Document,
    pub update: Vec<u8>,
}

impl DocumentSnapshot {
    pub fn new(document: Document, update: Vec<u8>) -> Self {
        Self { document, update }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PersistedSnapshot {
    document: PersistedDocument,
    update: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedDocument {
    id: Uuid,
    title: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    access_token: String,
}

impl From<DocumentSnapshot> for PersistedSnapshot {
    fn from(snapshot: DocumentSnapshot) -> Self {
        let document = snapshot.document;
        let access_token = document.access_token().to_owned();

        Self {
            document: PersistedDocument {
                id: document.id,
                title: document.title,
                created_at: document.created_at,
                updated_at: document.updated_at,
                access_token,
            },
            update: snapshot.update,
        }
    }
}

impl From<PersistedSnapshot> for DocumentSnapshot {
    fn from(snapshot: PersistedSnapshot) -> Self {
        Self {
            document: Document::from_parts(
                snapshot.document.id,
                snapshot.document.title,
                snapshot.document.created_at,
                snapshot.document.updated_at,
                snapshot.document.access_token,
            ),
            update: snapshot.update,
        }
    }
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("snapshot `{0}` is temporarily busy")]
    Busy(Uuid),
    #[error("document `{0}` still has active collaboration sessions")]
    DocumentBusy(Uuid),
    #[error("snapshot `{0}` was corrupt")]
    CorruptSnapshot(Uuid),
    #[error("snapshot storage I/O failed: {0}")]
    Io(String),
    #[error("snapshot storage configuration is invalid: {0}")]
    Config(String),
}

pub trait SnapshotStore: Send + Sync {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError>;
    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError>;
    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError>;
    fn list_documents(&self) -> Result<Vec<Document>, StorageError>;
}

#[derive(Default)]
pub struct InMemorySnapshotStore {
    snapshots: DashMap<Uuid, DocumentSnapshot>,
}

impl InMemorySnapshotStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SnapshotStore for InMemorySnapshotStore {
    fn load_snapshot(&self, doc_id: &Uuid) -> Result<Option<DocumentSnapshot>, StorageError> {
        Ok(self
            .snapshots
            .get(doc_id)
            .map(|entry| entry.value().clone()))
    }

    fn save_snapshot(&self, snapshot: DocumentSnapshot) -> Result<(), StorageError> {
        self.snapshots.insert(snapshot.document.id, snapshot);
        Ok(())
    }

    fn delete_snapshot(&self, doc_id: &Uuid) -> Result<(), StorageError> {
        self.snapshots.remove(doc_id);
        Ok(())
    }

    fn list_documents(&self) -> Result<Vec<Document>, StorageError> {
        Ok(self
            .snapshots
            .iter()
            .map(|entry| entry.value().document.clone())
            .collect())
    }
}

pub fn in_memory_snapshot_store() -> Arc<dyn SnapshotStore> {
    Arc::new(InMemorySnapshotStore::new())
}

pub fn file_snapshot_store(
    root: impl Into<PathBuf>,
) -> Result<Arc<dyn SnapshotStore>, StorageError> {
    Ok(Arc::new(FileSnapshotStore::new(root)?))
}

pub fn snapshot_store_from_config(config: &Config) -> Result<Arc<dyn SnapshotStore>, StorageError> {
    match config.snapshot_store.trim().to_ascii_lowercase().as_str() {
        "memory" => Ok(in_memory_snapshot_store()),
        "file" => file_snapshot_store(&config.snapshot_dir),
        "flash_kv" => Ok(Arc::new(FlashKvSnapshotStore::new(
            &config.snapshot_flash_kv_path,
        )?)),
        "simple_db" => Ok(Arc::new(SimpleDbSnapshotStore::new(
            &config.snapshot_simple_db_path,
        )?)),
        "docdb" => Ok(Arc::new(DocDbSnapshotStore::new(
            &config.snapshot_docdb_path,
        )?)),
        "shorterdb" => Ok(Arc::new(ShorterDbSnapshotStore::new(
            &config.snapshot_shorterdb_path,
        )?)),
        "sqlite" => Ok(Arc::new(SqliteSnapshotStore::new(
            &config.snapshot_sqlite_path,
        )?)),
        "heed" => Ok(Arc::new(HeedSnapshotStore::new(
            &config.snapshot_heed_path,
        )?)),
        "hightower_kv" => Ok(Arc::new(HightowerKvSnapshotStore::new(
            &config.snapshot_hightower_kv_path,
        )?)),
        "hmdb" => Ok(Arc::new(HmdbSnapshotStore::new(
            &config.snapshot_hmdb_path,
        )?)),
        "bitask" => Ok(Arc::new(BitaskSnapshotStore::new(
            &config.snapshot_bitask_path,
        )?)),
        "jammdb" => Ok(Arc::new(JammdbSnapshotStore::new(
            &config.snapshot_jammdb_path,
        )?)),
        "jsondb" => Ok(Arc::new(JsondbSnapshotStore::new(
            &config.snapshot_jsondb_path,
        )?)),
        "fjall" => Ok(Arc::new(FjallSnapshotStore::new(
            &config.snapshot_fjall_path,
        )?)),
        "persy" => Ok(Arc::new(PersySnapshotStore::new(
            &config.snapshot_persy_path,
        )?)),
        "persistent_kv" => Ok(Arc::new(PersistentKvSnapshotStore::new(
            &config.snapshot_persistent_kv_path,
        )?)),
        "native_db" => Ok(Arc::new(NativeDbSnapshotStore::new(
            &config.snapshot_native_db_path,
        )?)),
        "nikidb" => Ok(Arc::new(NikidbSnapshotStore::new(
            &config.snapshot_nikidb_path,
        )?)),
        "nodb" => Ok(Arc::new(NodbSnapshotStore::new(
            &config.snapshot_nodb_path,
        )?)),
        "parity_db" => Ok(Arc::new(ParityDbSnapshotStore::new(
            &config.snapshot_parity_db_path,
        )?)),
        "pickledb" => Ok(Arc::new(PickleDbSnapshotStore::new(
            &config.snapshot_pickledb_path,
        )?)),
        "microkv" => Ok(Arc::new(MicroKvSnapshotStore::new(
            &config.snapshot_microkv_path,
        )?)),
        "redb" => Ok(Arc::new(RedbSnapshotStore::new(
            &config.snapshot_redb_path,
        )?)),
        "rskey" => Ok(Arc::new(RskeySnapshotStore::new(
            &config.snapshot_rskey_path,
        )?)),
        "readb" => Ok(Arc::new(ReadbSnapshotStore::new(
            &config.snapshot_readb_path,
        )?)),
        "rustlite" => Ok(Arc::new(RustliteSnapshotStore::new(
            &config.snapshot_rustlite_path,
        )?)),
        "canopydb" => Ok(Arc::new(CanopydbSnapshotStore::new(
            &config.snapshot_canopydb_path,
        )?)),
        "caves" => Ok(Arc::new(CavesSnapshotStore::new(
            &config.snapshot_caves_path,
        )?)),
        "ckydb" => Ok(Arc::new(CkydbSnapshotStore::new(
            &config.snapshot_ckydb_path,
        )?)),
        "scdb" => Ok(Arc::new(ScdbSnapshotStore::new(
            &config.snapshot_scdb_path,
        )?)),
        "surrealkv" => Ok(Arc::new(SurrealkvSnapshotStore::new(
            &config.snapshot_surrealkv_path,
        )?)),
        "sled" => Ok(Arc::new(SledSnapshotStore::new(
            &config.snapshot_sled_path,
        )?)),
        "rustbreak" => Ok(Arc::new(RustbreakSnapshotStore::new(
            &config.snapshot_rustbreak_path,
        )?)),
        "yedb" => Ok(Arc::new(YedbSnapshotStore::new(
            &config.snapshot_yedb_path,
        )?)),
        "btree_store" => Ok(Arc::new(BtreeStoreSnapshotStore::new(
            &config.snapshot_btree_store_path,
        )?)),
        "siamesedb" => Ok(Arc::new(SiamesedbSnapshotStore::new(
            &config.snapshot_siamesedb_path,
        )?)),
        "structsy" => Ok(Arc::new(StructsySnapshotStore::new(
            &config.snapshot_structsy_path,
        )?)),
        "abyssiniandb" => Ok(Arc::new(AbyssiniandbSnapshotStore::new(
            &config.snapshot_abyssiniandb_path,
        )?)),
        "aeternusdb" => Ok(Arc::new(AeternusdbSnapshotStore::new(
            &config.snapshot_aeternusdb_path,
        )?)),
        "thunderdb" => Ok(Arc::new(ThunderdbSnapshotStore::new(
            &config.snapshot_thunderdb_path,
        )?)),
        "dblite" => Ok(Arc::new(DbliteSnapshotStore::new(
            &config.snapshot_dblite_path,
        )?)),
        "dbless" => Ok(Arc::new(DblessSnapshotStore::new(
            &config.snapshot_dbless_path,
        )?)),
        "sanakirja" => Ok(Arc::new(SanakirjaSnapshotStore::new(
            &config.snapshot_sanakirja_path,
        )?)),
        "snaildb" => Ok(Arc::new(SnaildbSnapshotStore::new(
            &config.snapshot_snaildb_path,
        )?)),
        "tinykv" => Ok(Arc::new(TinykvSnapshotStore::new(
            &config.snapshot_tinykv_path,
        )?)),
        "saberdb" => Ok(Arc::new(SaberdbSnapshotStore::new(
            &config.snapshot_saberdb_path,
        )?)),
        "s3" => Ok(Arc::new(S3SnapshotStore::new(
            config.snapshot_s3_endpoint.clone().ok_or_else(|| {
                StorageError::Config(
                    "SNAPSHOT_S3_ENDPOINT is required when SNAPSHOT_STORE=s3".to_owned(),
                )
            })?,
            config.snapshot_s3_region.clone(),
            config.snapshot_s3_bucket.clone().ok_or_else(|| {
                StorageError::Config(
                    "SNAPSHOT_S3_BUCKET is required when SNAPSHOT_STORE=s3".to_owned(),
                )
            })?,
            config.snapshot_s3_prefix.clone(),
            config.snapshot_s3_access_key_id.clone().ok_or_else(|| {
                StorageError::Config(
                    "SNAPSHOT_S3_ACCESS_KEY_ID is required when SNAPSHOT_STORE=s3".to_owned(),
                )
            })?,
            config
                .snapshot_s3_secret_access_key
                .clone()
                .ok_or_else(|| {
                    StorageError::Config(
                        "SNAPSHOT_S3_SECRET_ACCESS_KEY is required when SNAPSHOT_STORE=s3"
                            .to_owned(),
                    )
                })?,
            config.snapshot_s3_session_token.clone(),
            Duration::from_secs(config.snapshot_s3_timeout_secs),
            config.snapshot_s3_path_style,
        )?)),
        "managed" => Ok(Arc::new(ManagedSnapshotStore::new(
            config.snapshot_managed_base_url.clone().ok_or_else(|| {
                StorageError::Config(
                    "SNAPSHOT_MANAGED_BASE_URL is required when SNAPSHOT_STORE=managed".to_owned(),
                )
            })?,
            config.snapshot_managed_auth_token.clone(),
            Duration::from_secs(config.snapshot_managed_timeout_secs),
        )?)),
        other => Err(StorageError::Config(format!(
            "SNAPSHOT_STORE must be `memory`, `file`, `flash_kv`, `simple_db`, `docdb`, `shorterdb`, `sqlite`, `heed`, `hightower_kv`, `hmdb`, `bitask`, `jammdb`, `jsondb`, `fjall`, `persy`, `persistent_kv`, `native_db`, `nikidb`, `nodb`, `parity_db`, `pickledb`, `microkv`, `redb`, `rskey`, `readb`, `rustlite`, `canopydb`, `caves`, `ckydb`, `scdb`, `surrealkv`, `sled`, `rustbreak`, `yedb`, `btree_store`, `siamesedb`, `structsy`, `abyssiniandb`, `aeternusdb`, `thunderdb`, `dblite`, `dbless`, `sanakirja`, `snaildb`, `tinykv`, `saberdb`, `s3`, or `managed`, received `{other}`"
        ))),
    }
}

pub(crate) fn ensure_snapshot_dir(path: &Path) -> Result<(), StorageError> {
    fs::create_dir_all(path)
        .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))
}
