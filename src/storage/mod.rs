mod abyssiniandb_snapshot_store;
mod aeternusdb_snapshot_store;
mod agdb_snapshot_store;
mod amandine_snapshot_store;
mod apex_store_snapshot_store;
mod append_kv_snapshot_store;
mod armdb_snapshot_store;
mod bitask_snapshot_store;
mod bitcask_engine_snapshot_store;
mod bitkv_rs_snapshot_store;
mod blazeup_snapshot_store;
mod blockbucket_snapshot_store;
mod btree_store_snapshot_store;
mod candystore_snapshot_store;
mod canopydb_snapshot_store;
mod caves_snapshot_store;
mod celerix_store_snapshot_store;
mod ckydb_snapshot_store;
mod crepedb_snapshot_store;
mod crystal_snapshot_store;
mod cuendillar_snapshot_store;
mod datastack_snapshot_store;
mod db_rs_snapshot_store;
mod dbless_snapshot_store;
mod dblite_snapshot_store;
mod dharmadb_snapshot_store;
mod docdb_snapshot_store;
mod eight_snapshot_store;
mod epoch_db_snapshot_store;
mod etchdb_snapshot_store;
mod feoxdb_snapshot_store;
mod ferrumdb_snapshot_store;
mod file_snapshot_store;
mod fjall_snapshot_store;
mod flash_kv_snapshot_store;
mod ghaladb_snapshot_store;
mod graus_db_snapshot_store;
mod grebedb_snapshot_store;
mod grumpydb_snapshot_store;
mod heed_snapshot_store;
mod highlandcows_isam_snapshot_store;
mod hightower_kv_snapshot_store;
mod hmdb_snapshot_store;
mod icefalldb_snapshot_store;
mod infusedb_snapshot_store;
mod jammdb_snapshot_store;
mod janql_snapshot_store;
mod jasondb_snapshot_store;
mod jasonisnthappy_snapshot_store;
mod jfs_snapshot_store;
mod joydb_snapshot_store;
mod json_store_snapshot_store;
mod jsondb_snapshot_store;
mod kafi_snapshot_store;
mod koit_snapshot_store;
mod kopperdb_snapshot_store;
mod kstone_snapshot_store;
mod kv_snapshot_store;
mod ledger_kv_snapshot_store;
mod lite_db_snapshot_store;
mod log_kv_snapshot_store;
mod loro_kv_snapshot_store;
mod lsm_engine_snapshot_store;
mod lsm_storage_engine_snapshot_store;
mod lsm_tree_snapshot_store;
mod lsmdb_snapshot_store;
mod luckdb_snapshot_store;
mod mace_snapshot_store;
mod managed_snapshot_store;
mod mhdb_snapshot_store;
mod microkv_snapshot_store;
mod mindb_snapshot_store;
mod mmdb_snapshot_store;
mod nanodb_snapshot_store;
mod native_db_snapshot_store;
mod nebari_snapshot_store;
mod nikidb_snapshot_store;
mod nodb_snapshot_store;
mod okofdb_snapshot_store;
mod parity_db_snapshot_store;
mod persistent_kv_snapshot_store;
mod persy_snapshot_store;
mod pickledb_snapshot_store;
mod raindb_snapshot_store;
mod rcask_snapshot_store;
mod readb_snapshot_store;
mod redb_snapshot_store;
mod roughdb_snapshot_store;
mod rskey_snapshot_store;
mod rubin_snapshot_store;
mod rumdb_snapshot_store;
mod rustbreak_snapshot_store;
mod rustcask_snapshot_store;
mod rustlite_snapshot_store;
mod rusty_leveldb_snapshot_store;
mod s3_snapshot_store;
mod saberdb_snapshot_store;
mod sanakirja_snapshot_store;
mod scdb_snapshot_store;
mod shorterdb_snapshot_store;
mod siamesedb_snapshot_store;
mod simple_db_snapshot_store;
mod skv_snapshot_store;
mod sled_snapshot_store;
mod smolldb_snapshot_store;
mod snaildb_snapshot_store;
mod sqlite_snapshot_store;
mod structsy_snapshot_store;
mod surrealkv_snapshot_store;
mod thetadb_snapshot_store;
mod thunderdb_snapshot_store;
mod tinkv_snapshot_store;
mod tinybase_snapshot_store;
mod tinydb_snapshot_store;
mod tinykv_snapshot_store;
mod vsdb_snapshot_store;
mod yakv_snapshot_store;
mod yakvdb_snapshot_store;
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
pub use agdb_snapshot_store::AgdbSnapshotStore;
pub use amandine_snapshot_store::AmandineSnapshotStore;
pub use apex_store_snapshot_store::ApexStoreSnapshotStore;
pub use append_kv_snapshot_store::AppendKvSnapshotStore;
pub use armdb_snapshot_store::ArmdbSnapshotStore;
pub use bitask_snapshot_store::BitaskSnapshotStore;
pub use bitcask_engine_snapshot_store::BitcaskEngineSnapshotStore;
pub use bitkv_rs_snapshot_store::BitkvRsSnapshotStore;
pub use blazeup_snapshot_store::BlazeupSnapshotStore;
pub use blockbucket_snapshot_store::BlockbucketSnapshotStore;
pub use btree_store_snapshot_store::BtreeStoreSnapshotStore;
pub use candystore_snapshot_store::CandystoreSnapshotStore;
pub use canopydb_snapshot_store::CanopydbSnapshotStore;
pub use caves_snapshot_store::CavesSnapshotStore;
pub use celerix_store_snapshot_store::CelerixStoreSnapshotStore;
pub use ckydb_snapshot_store::CkydbSnapshotStore;
pub use crepedb_snapshot_store::CrepeDbSnapshotStore;
pub use crystal_snapshot_store::CrystalSnapshotStore;
pub use cuendillar_snapshot_store::CuendillarSnapshotStore;
pub use datastack_snapshot_store::DatastackSnapshotStore;
pub use db_rs_snapshot_store::DbRsSnapshotStore;
pub use dbless_snapshot_store::DblessSnapshotStore;
pub use dblite_snapshot_store::DbliteSnapshotStore;
pub use dharmadb_snapshot_store::DharmadbSnapshotStore;
pub use docdb_snapshot_store::DocDbSnapshotStore;
pub use eight_snapshot_store::EightSnapshotStore;
pub use epoch_db_snapshot_store::EpochDbSnapshotStore;
pub use etchdb_snapshot_store::EtchdbSnapshotStore;
pub use feoxdb_snapshot_store::FeoxdbSnapshotStore;
pub use ferrumdb_snapshot_store::FerrumdbSnapshotStore;
pub use file_snapshot_store::FileSnapshotStore;
pub use fjall_snapshot_store::FjallSnapshotStore;
pub use flash_kv_snapshot_store::FlashKvSnapshotStore;
pub use ghaladb_snapshot_store::GhaladbSnapshotStore;
pub use graus_db_snapshot_store::GrausDbSnapshotStore;
pub use grebedb_snapshot_store::GrebedbSnapshotStore;
pub use grumpydb_snapshot_store::GrumpydbSnapshotStore;
pub use heed_snapshot_store::HeedSnapshotStore;
pub use highlandcows_isam_snapshot_store::HighlandcowsIsamSnapshotStore;
pub use hightower_kv_snapshot_store::HightowerKvSnapshotStore;
pub use hmdb_snapshot_store::HmdbSnapshotStore;
pub use icefalldb_snapshot_store::IcefalldbSnapshotStore;
pub use infusedb_snapshot_store::InfusedbSnapshotStore;
pub use jammdb_snapshot_store::JammdbSnapshotStore;
pub use janql_snapshot_store::JanqlSnapshotStore;
pub use jasondb_snapshot_store::JasondbSnapshotStore;
pub use jasonisnthappy_snapshot_store::JasonisnthappySnapshotStore;
pub use jfs_snapshot_store::JfsSnapshotStore;
pub use joydb_snapshot_store::JoydbSnapshotStore;
pub use json_store_snapshot_store::JsonStoreSnapshotStore;
pub use jsondb_snapshot_store::JsondbSnapshotStore;
pub use kafi_snapshot_store::KafiSnapshotStore;
pub use koit_snapshot_store::KoitSnapshotStore;
pub use kopperdb_snapshot_store::KopperdbSnapshotStore;
pub use kstone_snapshot_store::KstoneSnapshotStore;
pub use kv_snapshot_store::KvSnapshotStore;
pub use ledger_kv_snapshot_store::LedgerKvSnapshotStore;
pub use lite_db_snapshot_store::LiteDbSnapshotStore;
pub use log_kv_snapshot_store::LogKvSnapshotStore;
pub use loro_kv_snapshot_store::LoroKvSnapshotStore;
pub use lsm_engine_snapshot_store::LsmEngineSnapshotStore;
pub use lsm_storage_engine_snapshot_store::LsmStorageEngineSnapshotStore;
pub use lsm_tree_snapshot_store::LsmTreeSnapshotStore;
pub use lsmdb_snapshot_store::LsmdbSnapshotStore;
pub use luckdb_snapshot_store::LuckdbSnapshotStore;
pub use mace_snapshot_store::MaceSnapshotStore;
pub use managed_snapshot_store::ManagedSnapshotStore;
pub use mhdb_snapshot_store::MhdbSnapshotStore;
pub use microkv_snapshot_store::MicroKvSnapshotStore;
pub use mindb_snapshot_store::MindbSnapshotStore;
pub use mmdb_snapshot_store::MmdbSnapshotStore;
pub use nanodb_snapshot_store::NanodbSnapshotStore;
pub use native_db_snapshot_store::NativeDbSnapshotStore;
pub use nebari_snapshot_store::NebariSnapshotStore;
pub use nikidb_snapshot_store::NikidbSnapshotStore;
pub use nodb_snapshot_store::NodbSnapshotStore;
pub use okofdb_snapshot_store::OkofdbSnapshotStore;
pub use parity_db_snapshot_store::ParityDbSnapshotStore;
pub use persistent_kv_snapshot_store::PersistentKvSnapshotStore;
pub use persy_snapshot_store::PersySnapshotStore;
pub use pickledb_snapshot_store::PickleDbSnapshotStore;
pub use raindb_snapshot_store::RaindbSnapshotStore;
pub use rcask_snapshot_store::RcaskSnapshotStore;
pub use readb_snapshot_store::ReadbSnapshotStore;
pub use redb_snapshot_store::RedbSnapshotStore;
pub use roughdb_snapshot_store::RoughdbSnapshotStore;
pub use rskey_snapshot_store::RskeySnapshotStore;
pub use rubin_snapshot_store::RubinSnapshotStore;
pub use rumdb_snapshot_store::RumDbSnapshotStore;
pub use rustbreak_snapshot_store::RustbreakSnapshotStore;
pub use rustcask_snapshot_store::RustcaskSnapshotStore;
pub use rustlite_snapshot_store::RustliteSnapshotStore;
pub use rusty_leveldb_snapshot_store::RustyLeveldbSnapshotStore;
pub use s3_snapshot_store::S3SnapshotStore;
pub use saberdb_snapshot_store::SaberdbSnapshotStore;
pub use sanakirja_snapshot_store::SanakirjaSnapshotStore;
pub use scdb_snapshot_store::ScdbSnapshotStore;
pub use shorterdb_snapshot_store::ShorterDbSnapshotStore;
pub use siamesedb_snapshot_store::SiamesedbSnapshotStore;
pub use simple_db_snapshot_store::SimpleDbSnapshotStore;
pub use skv_snapshot_store::SkvSnapshotStore;
pub use sled_snapshot_store::SledSnapshotStore;
pub use smolldb_snapshot_store::SmolldbSnapshotStore;
pub use snaildb_snapshot_store::SnaildbSnapshotStore;
pub use sqlite_snapshot_store::SqliteSnapshotStore;
pub use structsy_snapshot_store::StructsySnapshotStore;
pub use surrealkv_snapshot_store::SurrealkvSnapshotStore;
pub use thetadb_snapshot_store::ThetadbSnapshotStore;
pub use thunderdb_snapshot_store::ThunderdbSnapshotStore;
pub use tinkv_snapshot_store::TinkvSnapshotStore;
pub use tinybase_snapshot_store::TinybaseSnapshotStore;
pub use tinydb_snapshot_store::TinydbSnapshotStore;
pub use tinykv_snapshot_store::TinykvSnapshotStore;
pub use vsdb_snapshot_store::VsdbSnapshotStore;
pub use yakv_snapshot_store::YakvSnapshotStore;
pub use yakvdb_snapshot_store::YakvdbSnapshotStore;
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
        "agdb" => Ok(Arc::new(AgdbSnapshotStore::new(
            &config.snapshot_agdb_path,
        )?)),
        "amandine" => Ok(Arc::new(AmandineSnapshotStore::new(
            &config.snapshot_amandine_path,
        )?)),
        "apex_store" => Ok(Arc::new(ApexStoreSnapshotStore::new(
            &config.snapshot_apex_store_path,
        )?)),
        "armdb" => Ok(Arc::new(ArmdbSnapshotStore::new(
            &config.snapshot_armdb_path,
        )?)),
        "flash_kv" => Ok(Arc::new(FlashKvSnapshotStore::new(
            &config.snapshot_flash_kv_path,
        )?)),
        "ghaladb" => Ok(Arc::new(GhaladbSnapshotStore::new(
            &config.snapshot_ghaladb_path,
        )?)),
        "blockbucket" => Ok(Arc::new(BlockbucketSnapshotStore::new(
            &config.snapshot_blockbucket_path,
        )?)),
        "grebedb" => Ok(Arc::new(GrebedbSnapshotStore::new(
            &config.snapshot_grebedb_path,
        )?)),
        "grumpydb" => Ok(Arc::new(GrumpydbSnapshotStore::new(
            &config.snapshot_grumpydb_path,
        )?)),
        "graus_db" => Ok(Arc::new(GrausDbSnapshotStore::new(
            &config.snapshot_graus_db_path,
        )?)),
        "highlandcows_isam" => Ok(Arc::new(HighlandcowsIsamSnapshotStore::new(
            &config.snapshot_highlandcows_isam_path,
        )?)),
        "simple_db" => Ok(Arc::new(SimpleDbSnapshotStore::new(
            &config.snapshot_simple_db_path,
        )?)),
        "docdb" => Ok(Arc::new(DocDbSnapshotStore::new(
            &config.snapshot_docdb_path,
        )?)),
        "eight" => Ok(Arc::new(EightSnapshotStore::new(
            &config.snapshot_eight_path,
        )?)),
        "epoch_db" => Ok(Arc::new(EpochDbSnapshotStore::new(
            &config.snapshot_epoch_db_path,
        )?)),
        "etchdb" => Ok(Arc::new(EtchdbSnapshotStore::new(
            &config.snapshot_etchdb_path,
        )?)),
        "ferrumdb" => Ok(Arc::new(FerrumdbSnapshotStore::new(
            &config.snapshot_ferrumdb_path,
        )?)),
        "rumdb" => Ok(Arc::new(RumDbSnapshotStore::new(
            &config.snapshot_rumdb_path,
        )?)),
        "rubin" => Ok(Arc::new(RubinSnapshotStore::new(
            &config.snapshot_rubin_path,
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
        "icefalldb" => Ok(Arc::new(IcefalldbSnapshotStore::new(
            &config.snapshot_icefalldb_path,
        )?)),
        "bitask" => Ok(Arc::new(BitaskSnapshotStore::new(
            &config.snapshot_bitask_path,
        )?)),
        "bitkv_rs" => Ok(Arc::new(BitkvRsSnapshotStore::new(
            &config.snapshot_bitkv_rs_path,
        )?)),
        "bitcask_engine" => Ok(Arc::new(BitcaskEngineSnapshotStore::new(
            &config.snapshot_bitcask_engine_path,
        )?)),
        "blazeup" => Ok(Arc::new(BlazeupSnapshotStore::new(
            &config.snapshot_blazeup_path,
        )?)),
        "candystore" => Ok(Arc::new(CandystoreSnapshotStore::new(
            &config.snapshot_candystore_path,
        )?)),
        "celerix_store" => Ok(Arc::new(CelerixStoreSnapshotStore::new(
            &config.snapshot_celerix_store_path,
        )?)),
        "cuendillar" => Ok(Arc::new(CuendillarSnapshotStore::new(
            &config.snapshot_cuendillar_path,
        )?)),
        "datastack" => Ok(Arc::new(DatastackSnapshotStore::new(
            &config.snapshot_datastack_path,
        )?)),
        "jammdb" => Ok(Arc::new(JammdbSnapshotStore::new(
            &config.snapshot_jammdb_path,
        )?)),
        "mace" => Ok(Arc::new(MaceSnapshotStore::new(
            &config.snapshot_mace_path,
        )?)),
        "janql" => Ok(Arc::new(JanqlSnapshotStore::new(
            &config.snapshot_janql_path,
        )?)),
        "jasondb" => Ok(Arc::new(JasondbSnapshotStore::new(
            &config.snapshot_jasondb_path,
        )?)),
        "jasonisnthappy" => Ok(Arc::new(JasonisnthappySnapshotStore::new(
            &config.snapshot_jasonisnthappy_path,
        )?)),
        "jfs" => Ok(Arc::new(JfsSnapshotStore::new(&config.snapshot_jfs_path)?)),
        "json_store" => Ok(Arc::new(JsonStoreSnapshotStore::new(
            &config.snapshot_json_store_path,
        )?)),
        "feoxdb" => Ok(Arc::new(FeoxdbSnapshotStore::new(
            &config.snapshot_feoxdb_path,
        )?)),
        "jsondb" => Ok(Arc::new(JsondbSnapshotStore::new(
            &config.snapshot_jsondb_path,
        )?)),
        "kopperdb" => Ok(Arc::new(KopperdbSnapshotStore::new(
            &config.snapshot_kopperdb_path,
        )?)),
        "kv" => Ok(Arc::new(KvSnapshotStore::new(&config.snapshot_kv_path)?)),
        "koit" => Ok(Arc::new(KoitSnapshotStore::new(
            &config.snapshot_koit_path,
        )?)),
        "lite_db" => Ok(Arc::new(LiteDbSnapshotStore::new(
            &config.snapshot_lite_db_path,
        )?)),
        "log_kv" => Ok(Arc::new(LogKvSnapshotStore::new(
            &config.snapshot_log_kv_path,
        )?)),
        "append_kv" => Ok(Arc::new(AppendKvSnapshotStore::new(
            &config.snapshot_append_kv_path,
        )?)),
        "mhdb" => Ok(Arc::new(MhdbSnapshotStore::new(
            &config.snapshot_mhdb_path,
        )?)),
        "loro_kv" => Ok(Arc::new(LoroKvSnapshotStore::new(
            &config.snapshot_loro_kv_path,
        )?)),
        "luckdb" => Ok(Arc::new(LuckdbSnapshotStore::new(
            &config.snapshot_luckdb_path,
        )?)),
        "lsm_engine" => Ok(Arc::new(LsmEngineSnapshotStore::new(
            &config.snapshot_lsm_engine_path,
        )?)),
        "lsm_storage_engine" => Ok(Arc::new(LsmStorageEngineSnapshotStore::new(
            &config.snapshot_lsm_storage_engine_path,
        )?)),
        "lsmdb" => Ok(Arc::new(LsmdbSnapshotStore::new(
            &config.snapshot_lsmdb_path,
        )?)),
        "lsm_tree" => Ok(Arc::new(LsmTreeSnapshotStore::new(
            &config.snapshot_lsm_tree_path,
        )?)),
        "mindb" => Ok(Arc::new(MindbSnapshotStore::new(
            &config.snapshot_mindb_path,
        )?)),
        "mmdb" => Ok(Arc::new(MmdbSnapshotStore::new(
            &config.snapshot_mmdb_path,
        )?)),
        "nanodb" => Ok(Arc::new(NanodbSnapshotStore::new(
            &config.snapshot_nanodb_path,
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
        "nebari" => Ok(Arc::new(NebariSnapshotStore::new(
            &config.snapshot_nebari_path,
        )?)),
        "nikidb" => Ok(Arc::new(NikidbSnapshotStore::new(
            &config.snapshot_nikidb_path,
        )?)),
        "nodb" => Ok(Arc::new(NodbSnapshotStore::new(
            &config.snapshot_nodb_path,
        )?)),
        "okofdb" => Ok(Arc::new(OkofdbSnapshotStore::new(
            &config.snapshot_okofdb_path,
        )?)),
        "parity_db" => Ok(Arc::new(ParityDbSnapshotStore::new(
            &config.snapshot_parity_db_path,
        )?)),
        "pickledb" => Ok(Arc::new(PickleDbSnapshotStore::new(
            &config.snapshot_pickledb_path,
        )?)),
        "rcask" => Ok(Arc::new(RcaskSnapshotStore::new(
            &config.snapshot_rcask_path,
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
        "crepedb" => Ok(Arc::new(CrepeDbSnapshotStore::new(
            &config.snapshot_crepedb_path,
        )?)),
        "crystal" => Ok(Arc::new(CrystalSnapshotStore::new(
            &config.snapshot_crystal_path,
        )?)),
        "scdb" => Ok(Arc::new(ScdbSnapshotStore::new(
            &config.snapshot_scdb_path,
        )?)),
        "skv" => Ok(Arc::new(SkvSnapshotStore::new(&config.snapshot_skv_path)?)),
        "surrealkv" => Ok(Arc::new(SurrealkvSnapshotStore::new(
            &config.snapshot_surrealkv_path,
        )?)),
        "sled" => Ok(Arc::new(SledSnapshotStore::new(
            &config.snapshot_sled_path,
        )?)),
        "rustbreak" => Ok(Arc::new(RustbreakSnapshotStore::new(
            &config.snapshot_rustbreak_path,
        )?)),
        "rustcask" => Ok(Arc::new(RustcaskSnapshotStore::new(
            &config.snapshot_rustcask_path,
        )?)),
        "rusty_leveldb" => Ok(Arc::new(RustyLeveldbSnapshotStore::new(
            &config.snapshot_rusty_leveldb_path,
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
        "thetadb" => Ok(Arc::new(ThetadbSnapshotStore::new(
            &config.snapshot_thetadb_path,
        )?)),
        "tinybase" => Ok(Arc::new(TinybaseSnapshotStore::new(
            &config.snapshot_tinybase_path,
        )?)),
        "tinydb" => Ok(Arc::new(TinydbSnapshotStore::new(
            &config.snapshot_tinydb_path,
        )?)),
        "dblite" => Ok(Arc::new(DbliteSnapshotStore::new(
            &config.snapshot_dblite_path,
        )?)),
        "dbless" => Ok(Arc::new(DblessSnapshotStore::new(
            &config.snapshot_dbless_path,
        )?)),
        "db_rs" => Ok(Arc::new(DbRsSnapshotStore::new(
            &config.snapshot_db_rs_path,
        )?)),
        "dharmadb" => Ok(Arc::new(DharmadbSnapshotStore::new(
            &config.snapshot_dharmadb_path,
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
        "vsdb" => Ok(Arc::new(VsdbSnapshotStore::new(
            &config.snapshot_vsdb_path,
        )?)),
        "yakv" => Ok(Arc::new(YakvSnapshotStore::new(
            &config.snapshot_yakv_path,
        )?)),
        "yakvdb" => Ok(Arc::new(YakvdbSnapshotStore::new(
            &config.snapshot_yakvdb_path,
        )?)),
        "saberdb" => Ok(Arc::new(SaberdbSnapshotStore::new(
            &config.snapshot_saberdb_path,
        )?)),
        "smolldb" => Ok(Arc::new(SmolldbSnapshotStore::new(
            &config.snapshot_smolldb_path,
        )?)),
        "kstone" => Ok(Arc::new(KstoneSnapshotStore::new(
            &config.snapshot_kstone_path,
        )?)),
        "roughdb" => Ok(Arc::new(RoughdbSnapshotStore::new(
            &config.snapshot_roughdb_path,
        )?)),
        "raindb" => Ok(Arc::new(RaindbSnapshotStore::new(
            &config.snapshot_raindb_path,
        )?)),
        "infusedb" => Ok(Arc::new(InfusedbSnapshotStore::new(
            &config.snapshot_infusedb_path,
        )?)),
        "kafi" => Ok(Arc::new(KafiSnapshotStore::new(
            &config.snapshot_kafi_path,
        )?)),
        "tinkv" => Ok(Arc::new(TinkvSnapshotStore::new(
            &config.snapshot_tinkv_path,
        )?)),
        "ledger_kv" => Ok(Arc::new(LedgerKvSnapshotStore::new(
            &config.snapshot_ledger_kv_path,
        )?)),
        "joydb" => Ok(Arc::new(JoydbSnapshotStore::new(
            &config.snapshot_joydb_path,
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
            "SNAPSHOT_STORE must be `memory`, `file`, `agdb`, `amandine`, `apex_store`, `armdb`, `flash_kv`, `ghaladb`, `blockbucket`, `grebedb`, `grumpydb`, `graus_db`, `highlandcows_isam`, `simple_db`, `docdb`, `eight`, `epoch_db`, `etchdb`, `ferrumdb`, `rumdb`, `rubin`, `shorterdb`, `sqlite`, `heed`, `hightower_kv`, `hmdb`, `icefalldb`, `bitask`, `bitkv_rs`, `bitcask_engine`, `blazeup`, `candystore`, `celerix_store`, `cuendillar`, `datastack`, `jammdb`, `mace`, `janql`, `jasondb`, `jasonisnthappy`, `jfs`, `json_store`, `feoxdb`, `jsondb`, `joydb`, `kopperdb`, `kstone`, `roughdb`, `raindb`, `infusedb`, `kafi`, `tinkv`, `ledger_kv`, `kv`, `koit`, `lite_db`, `log_kv`, `mhdb`, `loro_kv`, `luckdb`, `lsm_engine`, `lsm_storage_engine`, `lsmdb`, `lsm_tree`, `mindb`, `mmdb`, `nanodb`, `fjall`, `persy`, `persistent_kv`, `native_db`, `nebari`, `nikidb`, `nodb`, `okofdb`, `parity_db`, `pickledb`, `rcask`, `microkv`, `redb`, `rskey`, `readb`, `rustlite`, `rustcask`, `rusty_leveldb`, `canopydb`, `caves`, `ckydb`, `crepedb`, `crystal`, `scdb`, `skv`, `surrealkv`, `sled`, `rustbreak`, `yedb`, `btree_store`, `siamesedb`, `structsy`, `abyssiniandb`, `aeternusdb`, `thunderdb`, `thetadb`, `tinybase`, `tinydb`, `dblite`, `dbless`, `db_rs`, `dharmadb`, `sanakirja`, `snaildb`, `tinykv`, `vsdb`, `yakv`, `saberdb`, `smolldb`, `s3`, or `managed`, received `{other}`"
        ))),
    }
}

pub(crate) fn ensure_snapshot_dir(path: &Path) -> Result<(), StorageError> {
    fs::create_dir_all(path)
        .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))
}
