#[cfg(feature = "full-snapshot-stores")]
mod abyssiniandb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod aeternusdb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod agdb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod amandine_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod apex_store_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod append_kv_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod append_log_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod armdb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod assystem_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod bitask_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod bitcask_engine_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod bitkv_rs_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod blazeup_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod blockbucket_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod btree_store_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod cacache_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod candystore_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod canopydb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod caves_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod cdb64_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod celerix_store_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod citadeldb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod ckydb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod colon_db_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod crepedb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod crystal_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod cuendillar_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod data_pile_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod datastack_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod db_rs_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod dbless_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod dblite_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod deeb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod dharmadb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod dir_cache_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod docdb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod eight_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod emdb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod epoch_db_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod etchdb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod fastkv_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod feoxdb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod ferrumdb_snapshot_store;
mod file_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod fjall_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod flash_kv_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod fs_db_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod ghaladb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod graus_db_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod grebedb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod grumpydb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod heed_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod highlandcows_isam_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod hightower_kv_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod hmdb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod hurrahdb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod icefalldb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod infusedb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod ipjdb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod jammdb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod janql_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod jasondb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod jasonisnthappy_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod jfs_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod joydb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod json_db_rs_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod json_mutex_db_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod json_store_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod jsondb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod kafi_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod kagi_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod koit_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod kopperdb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod kstone_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod kv_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod ledger_kv_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod lite_db_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod lmdb_rs_core_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod log_kv_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod loro_kv_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod lsm_engine_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod lsm_storage_engine_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod lsm_tree_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod lsmdb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod luckdb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod mace_snapshot_store;
mod managed_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod marble_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod mhdb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod microkv_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod mindb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod mmdb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod mu_db_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod nanodb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod native_db_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod nebari_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod nikidb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod nodb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod okofdb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod osmiumdb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod parity_db_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod persistent_kv_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod persy_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod pickledb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod png_db_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod raindb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod rcask_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod readb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod redb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod roughdb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod rskey_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod rubin_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod rumdb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod rustbreak_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod rustcask_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod rustlite_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod rusty_leveldb_snapshot_store;
mod s3_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod saberdb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod sanakirja_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod saturn_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod scdb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod shorterdb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod siamesedb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod simple_db_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod skv_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod sled_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod smolldb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod snaildb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod sqjson_snapshot_store;
mod sqlite_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod structsy_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod surrealkv_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod thetadb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod thunderdb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod tinkv_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod tinybase_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod tinydb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod tinykv_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod toiletdb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod vsdb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod yakv_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
mod yakvdb_snapshot_store;
#[cfg(feature = "full-snapshot-stores")]
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

#[cfg(feature = "full-snapshot-stores")]
pub use abyssiniandb_snapshot_store::AbyssiniandbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use aeternusdb_snapshot_store::AeternusdbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use agdb_snapshot_store::AgdbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use amandine_snapshot_store::AmandineSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use apex_store_snapshot_store::ApexStoreSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use append_kv_snapshot_store::AppendKvSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use append_log_snapshot_store::AppendLogSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use armdb_snapshot_store::ArmdbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use assystem_snapshot_store::AssystemSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use bitask_snapshot_store::BitaskSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use bitcask_engine_snapshot_store::BitcaskEngineSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use bitkv_rs_snapshot_store::BitkvRsSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use blazeup_snapshot_store::BlazeupSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use blockbucket_snapshot_store::BlockbucketSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use btree_store_snapshot_store::BtreeStoreSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use cacache_snapshot_store::CacacheSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use candystore_snapshot_store::CandystoreSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use canopydb_snapshot_store::CanopydbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use caves_snapshot_store::CavesSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use cdb64_snapshot_store::Cdb64SnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use celerix_store_snapshot_store::CelerixStoreSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use citadeldb_snapshot_store::CitadeldbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use ckydb_snapshot_store::CkydbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use colon_db_snapshot_store::ColonDbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use crepedb_snapshot_store::CrepeDbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use crystal_snapshot_store::CrystalSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use cuendillar_snapshot_store::CuendillarSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use data_pile_snapshot_store::DataPileSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use datastack_snapshot_store::DatastackSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use db_rs_snapshot_store::DbRsSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use dbless_snapshot_store::DblessSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use dblite_snapshot_store::DbliteSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use deeb_snapshot_store::DeebSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use dharmadb_snapshot_store::DharmadbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use dir_cache_snapshot_store::DirCacheSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use docdb_snapshot_store::DocDbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use eight_snapshot_store::EightSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use emdb_snapshot_store::EmdbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use epoch_db_snapshot_store::EpochDbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use etchdb_snapshot_store::EtchdbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use fastkv_snapshot_store::FastKvSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use feoxdb_snapshot_store::FeoxdbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use ferrumdb_snapshot_store::FerrumdbSnapshotStore;
pub use file_snapshot_store::FileSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use fjall_snapshot_store::FjallSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use flash_kv_snapshot_store::FlashKvSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use fs_db_snapshot_store::FsDbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use ghaladb_snapshot_store::GhaladbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use graus_db_snapshot_store::GrausDbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use grebedb_snapshot_store::GrebedbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use grumpydb_snapshot_store::GrumpydbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use heed_snapshot_store::HeedSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use highlandcows_isam_snapshot_store::HighlandcowsIsamSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use hightower_kv_snapshot_store::HightowerKvSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use hmdb_snapshot_store::HmdbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use hurrahdb_snapshot_store::HurrahdbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use icefalldb_snapshot_store::IcefalldbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use infusedb_snapshot_store::InfusedbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use ipjdb_snapshot_store::IpjdbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use jammdb_snapshot_store::JammdbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use janql_snapshot_store::JanqlSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use jasondb_snapshot_store::JasondbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use jasonisnthappy_snapshot_store::JasonisnthappySnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use jfs_snapshot_store::JfsSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use joydb_snapshot_store::JoydbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use json_db_rs_snapshot_store::JsonDbRsSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use json_mutex_db_snapshot_store::JsonMutexDbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use json_store_snapshot_store::JsonStoreSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use jsondb_snapshot_store::JsondbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use kafi_snapshot_store::KafiSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use kagi_snapshot_store::KagiSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use koit_snapshot_store::KoitSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use kopperdb_snapshot_store::KopperdbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use kstone_snapshot_store::KstoneSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use kv_snapshot_store::KvSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use ledger_kv_snapshot_store::LedgerKvSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use lite_db_snapshot_store::LiteDbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use lmdb_rs_core_snapshot_store::LmdbRsCoreSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use log_kv_snapshot_store::LogKvSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use loro_kv_snapshot_store::LoroKvSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use lsm_engine_snapshot_store::LsmEngineSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use lsm_storage_engine_snapshot_store::LsmStorageEngineSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use lsm_tree_snapshot_store::LsmTreeSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use lsmdb_snapshot_store::LsmdbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use luckdb_snapshot_store::LuckdbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use mace_snapshot_store::MaceSnapshotStore;
pub use managed_snapshot_store::ManagedSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use marble_snapshot_store::MarbleSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use mhdb_snapshot_store::MhdbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use microkv_snapshot_store::MicroKvSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use mindb_snapshot_store::MindbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use mmdb_snapshot_store::MmdbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use mu_db_snapshot_store::MuDbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use nanodb_snapshot_store::NanodbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use native_db_snapshot_store::NativeDbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use nebari_snapshot_store::NebariSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use nikidb_snapshot_store::NikidbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use nodb_snapshot_store::NodbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use okofdb_snapshot_store::OkofdbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use osmiumdb_snapshot_store::OsmiumdbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use parity_db_snapshot_store::ParityDbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use persistent_kv_snapshot_store::PersistentKvSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use persy_snapshot_store::PersySnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use pickledb_snapshot_store::PickleDbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use png_db_snapshot_store::PngDbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use raindb_snapshot_store::RaindbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use rcask_snapshot_store::RcaskSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use readb_snapshot_store::ReadbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use redb_snapshot_store::RedbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use roughdb_snapshot_store::RoughdbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use rskey_snapshot_store::RskeySnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use rubin_snapshot_store::RubinSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use rumdb_snapshot_store::RumDbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use rustbreak_snapshot_store::RustbreakSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use rustcask_snapshot_store::RustcaskSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use rustlite_snapshot_store::RustliteSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use rusty_leveldb_snapshot_store::RustyLeveldbSnapshotStore;
pub use s3_snapshot_store::S3SnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use saberdb_snapshot_store::SaberdbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use sanakirja_snapshot_store::SanakirjaSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use saturn_snapshot_store::SaturnSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use scdb_snapshot_store::ScdbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use shorterdb_snapshot_store::ShorterDbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use siamesedb_snapshot_store::SiamesedbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use simple_db_snapshot_store::SimpleDbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use skv_snapshot_store::SkvSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use sled_snapshot_store::SledSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use smolldb_snapshot_store::SmolldbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use snaildb_snapshot_store::SnaildbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use sqjson_snapshot_store::SqjsonSnapshotStore;
pub use sqlite_snapshot_store::SqliteSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use structsy_snapshot_store::StructsySnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use surrealkv_snapshot_store::SurrealkvSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use thetadb_snapshot_store::ThetadbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use thunderdb_snapshot_store::ThunderdbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use tinkv_snapshot_store::TinkvSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use tinybase_snapshot_store::TinybaseSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use tinydb_snapshot_store::TinydbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use tinykv_snapshot_store::TinykvSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use toiletdb_snapshot_store::ToiletdbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use vsdb_snapshot_store::VsdbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use yakv_snapshot_store::YakvSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
pub use yakvdb_snapshot_store::YakvdbSnapshotStore;
#[cfg(feature = "full-snapshot-stores")]
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

const SUPPORTED_SNAPSHOT_STORES: &[&str] = &[
    "memory",
    "file",
    #[cfg(feature = "full-snapshot-stores")]
    "agdb",
    #[cfg(feature = "full-snapshot-stores")]
    "amandine",
    #[cfg(feature = "full-snapshot-stores")]
    "append_log",
    #[cfg(feature = "full-snapshot-stores")]
    "apex_store",
    #[cfg(feature = "full-snapshot-stores")]
    "armdb",
    #[cfg(feature = "full-snapshot-stores")]
    "assystem",
    #[cfg(feature = "full-snapshot-stores")]
    "colon_db",
    #[cfg(feature = "full-snapshot-stores")]
    "flash_kv",
    #[cfg(feature = "full-snapshot-stores")]
    "ghaladb",
    #[cfg(feature = "full-snapshot-stores")]
    "blockbucket",
    #[cfg(feature = "full-snapshot-stores")]
    "grebedb",
    #[cfg(feature = "full-snapshot-stores")]
    "grumpydb",
    #[cfg(feature = "full-snapshot-stores")]
    "graus_db",
    #[cfg(feature = "full-snapshot-stores")]
    "highlandcows_isam",
    #[cfg(feature = "full-snapshot-stores")]
    "simple_db",
    #[cfg(feature = "full-snapshot-stores")]
    "docdb",
    #[cfg(feature = "full-snapshot-stores")]
    "emdb",
    #[cfg(feature = "full-snapshot-stores")]
    "osmiumdb",
    #[cfg(feature = "full-snapshot-stores")]
    "eight",
    #[cfg(feature = "full-snapshot-stores")]
    "epoch_db",
    #[cfg(feature = "full-snapshot-stores")]
    "etchdb",
    #[cfg(feature = "full-snapshot-stores")]
    "fastkv",
    #[cfg(feature = "full-snapshot-stores")]
    "ferrumdb",
    #[cfg(feature = "full-snapshot-stores")]
    "rumdb",
    #[cfg(feature = "full-snapshot-stores")]
    "rubin",
    #[cfg(feature = "full-snapshot-stores")]
    "shorterdb",
    "sqlite",
    #[cfg(feature = "full-snapshot-stores")]
    "heed",
    #[cfg(feature = "full-snapshot-stores")]
    "hightower_kv",
    #[cfg(feature = "full-snapshot-stores")]
    "hmdb",
    #[cfg(feature = "full-snapshot-stores")]
    "hurrahdb",
    #[cfg(feature = "full-snapshot-stores")]
    "fs_db",
    #[cfg(feature = "full-snapshot-stores")]
    "sqjson",
    #[cfg(feature = "full-snapshot-stores")]
    "icefalldb",
    #[cfg(feature = "full-snapshot-stores")]
    "bitask",
    #[cfg(feature = "full-snapshot-stores")]
    "bitkv_rs",
    #[cfg(feature = "full-snapshot-stores")]
    "bitcask_engine",
    #[cfg(feature = "full-snapshot-stores")]
    "blazeup",
    #[cfg(feature = "full-snapshot-stores")]
    "candystore",
    #[cfg(feature = "full-snapshot-stores")]
    "celerix_store",
    #[cfg(feature = "full-snapshot-stores")]
    "citadeldb",
    #[cfg(feature = "full-snapshot-stores")]
    "cuendillar",
    #[cfg(feature = "full-snapshot-stores")]
    "data_pile",
    #[cfg(feature = "full-snapshot-stores")]
    "datastack",
    #[cfg(feature = "full-snapshot-stores")]
    "jammdb",
    #[cfg(feature = "full-snapshot-stores")]
    "mace",
    #[cfg(feature = "full-snapshot-stores")]
    "janql",
    #[cfg(feature = "full-snapshot-stores")]
    "jasondb",
    #[cfg(feature = "full-snapshot-stores")]
    "jasonisnthappy",
    #[cfg(feature = "full-snapshot-stores")]
    "jfs",
    #[cfg(feature = "full-snapshot-stores")]
    "json_store",
    #[cfg(feature = "full-snapshot-stores")]
    "json_db_rs",
    #[cfg(feature = "full-snapshot-stores")]
    "cdb64",
    #[cfg(feature = "full-snapshot-stores")]
    "json_mutex_db",
    #[cfg(feature = "full-snapshot-stores")]
    "toiletdb",
    #[cfg(feature = "full-snapshot-stores")]
    "feoxdb",
    #[cfg(feature = "full-snapshot-stores")]
    "jsondb",
    #[cfg(feature = "full-snapshot-stores")]
    "kopperdb",
    #[cfg(feature = "full-snapshot-stores")]
    "kv",
    #[cfg(feature = "full-snapshot-stores")]
    "koit",
    #[cfg(feature = "full-snapshot-stores")]
    "lite_db",
    #[cfg(feature = "full-snapshot-stores")]
    "lmdb_rs_core",
    #[cfg(feature = "full-snapshot-stores")]
    "log_kv",
    #[cfg(feature = "full-snapshot-stores")]
    "append_kv",
    #[cfg(feature = "full-snapshot-stores")]
    "mhdb",
    #[cfg(feature = "full-snapshot-stores")]
    "marble",
    #[cfg(feature = "full-snapshot-stores")]
    "loro_kv",
    #[cfg(feature = "full-snapshot-stores")]
    "luckdb",
    #[cfg(feature = "full-snapshot-stores")]
    "ipjdb",
    #[cfg(feature = "full-snapshot-stores")]
    "kagi",
    #[cfg(feature = "full-snapshot-stores")]
    "deeb",
    #[cfg(feature = "full-snapshot-stores")]
    "lsm_engine",
    #[cfg(feature = "full-snapshot-stores")]
    "lsm_storage_engine",
    #[cfg(feature = "full-snapshot-stores")]
    "lsmdb",
    #[cfg(feature = "full-snapshot-stores")]
    "lsm_tree",
    #[cfg(feature = "full-snapshot-stores")]
    "mindb",
    #[cfg(feature = "full-snapshot-stores")]
    "mmdb",
    #[cfg(feature = "full-snapshot-stores")]
    "mu_db",
    #[cfg(feature = "full-snapshot-stores")]
    "nanodb",
    #[cfg(feature = "full-snapshot-stores")]
    "fjall",
    #[cfg(feature = "full-snapshot-stores")]
    "persy",
    #[cfg(feature = "full-snapshot-stores")]
    "persistent_kv",
    #[cfg(feature = "full-snapshot-stores")]
    "native_db",
    #[cfg(feature = "full-snapshot-stores")]
    "nebari",
    #[cfg(feature = "full-snapshot-stores")]
    "nikidb",
    #[cfg(feature = "full-snapshot-stores")]
    "nodb",
    #[cfg(feature = "full-snapshot-stores")]
    "okofdb",
    #[cfg(feature = "full-snapshot-stores")]
    "parity_db",
    #[cfg(feature = "full-snapshot-stores")]
    "pickledb",
    #[cfg(feature = "full-snapshot-stores")]
    "rcask",
    #[cfg(feature = "full-snapshot-stores")]
    "microkv",
    #[cfg(feature = "full-snapshot-stores")]
    "redb",
    #[cfg(feature = "full-snapshot-stores")]
    "rskey",
    #[cfg(feature = "full-snapshot-stores")]
    "readb",
    #[cfg(feature = "full-snapshot-stores")]
    "rustlite",
    #[cfg(feature = "full-snapshot-stores")]
    "canopydb",
    #[cfg(feature = "full-snapshot-stores")]
    "caves",
    #[cfg(feature = "full-snapshot-stores")]
    "ckydb",
    #[cfg(feature = "full-snapshot-stores")]
    "crepedb",
    #[cfg(feature = "full-snapshot-stores")]
    "crystal",
    #[cfg(feature = "full-snapshot-stores")]
    "scdb",
    #[cfg(feature = "full-snapshot-stores")]
    "skv",
    #[cfg(feature = "full-snapshot-stores")]
    "surrealkv",
    #[cfg(feature = "full-snapshot-stores")]
    "sled",
    #[cfg(feature = "full-snapshot-stores")]
    "rustbreak",
    #[cfg(feature = "full-snapshot-stores")]
    "rustcask",
    #[cfg(feature = "full-snapshot-stores")]
    "rusty_leveldb",
    #[cfg(feature = "full-snapshot-stores")]
    "yedb",
    #[cfg(feature = "full-snapshot-stores")]
    "btree_store",
    #[cfg(feature = "full-snapshot-stores")]
    "cacache",
    #[cfg(feature = "full-snapshot-stores")]
    "siamesedb",
    #[cfg(feature = "full-snapshot-stores")]
    "structsy",
    #[cfg(feature = "full-snapshot-stores")]
    "abyssiniandb",
    #[cfg(feature = "full-snapshot-stores")]
    "aeternusdb",
    #[cfg(feature = "full-snapshot-stores")]
    "thunderdb",
    #[cfg(feature = "full-snapshot-stores")]
    "thetadb",
    #[cfg(feature = "full-snapshot-stores")]
    "tinybase",
    #[cfg(feature = "full-snapshot-stores")]
    "tinydb",
    #[cfg(feature = "full-snapshot-stores")]
    "dblite",
    #[cfg(feature = "full-snapshot-stores")]
    "dbless",
    #[cfg(feature = "full-snapshot-stores")]
    "db_rs",
    #[cfg(feature = "full-snapshot-stores")]
    "dharmadb",
    #[cfg(feature = "full-snapshot-stores")]
    "dir_cache",
    #[cfg(feature = "full-snapshot-stores")]
    "sanakirja",
    #[cfg(feature = "full-snapshot-stores")]
    "saturn",
    #[cfg(feature = "full-snapshot-stores")]
    "snaildb",
    #[cfg(feature = "full-snapshot-stores")]
    "tinykv",
    #[cfg(feature = "full-snapshot-stores")]
    "vsdb",
    #[cfg(feature = "full-snapshot-stores")]
    "yakv",
    #[cfg(feature = "full-snapshot-stores")]
    "yakvdb",
    #[cfg(feature = "full-snapshot-stores")]
    "saberdb",
    #[cfg(feature = "full-snapshot-stores")]
    "smolldb",
    #[cfg(feature = "full-snapshot-stores")]
    "kstone",
    #[cfg(feature = "full-snapshot-stores")]
    "roughdb",
    #[cfg(feature = "full-snapshot-stores")]
    "raindb",
    #[cfg(feature = "full-snapshot-stores")]
    "infusedb",
    #[cfg(feature = "full-snapshot-stores")]
    "kafi",
    #[cfg(feature = "full-snapshot-stores")]
    "tinkv",
    #[cfg(feature = "full-snapshot-stores")]
    "ledger_kv",
    #[cfg(feature = "full-snapshot-stores")]
    "joydb",
    #[cfg(feature = "full-snapshot-stores")]
    "png_db",
    "s3",
    "managed",
];

fn invalid_snapshot_store_error(other: &str) -> StorageError {
    let (last, supported) = SUPPORTED_SNAPSHOT_STORES
        .split_last()
        .expect("supported snapshot store list should not be empty");
    let supported = supported
        .iter()
        .map(|store| format!("`{store}`"))
        .collect::<Vec<_>>()
        .join(", ");

    StorageError::Config(format!(
        "SNAPSHOT_STORE must be {supported}, or `{last}`, received `{other}`"
    ))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::document::Document;

    fn make_snapshot(title: &str) -> DocumentSnapshot {
        let doc = Document::new(Uuid::new_v4(), Some(title.to_owned()));
        DocumentSnapshot::new(doc, vec![1, 2, 3])
    }

    #[test]
    fn in_memory_store_returns_none_for_missing_snapshot() {
        let store = InMemorySnapshotStore::new();
        let result = store
            .load_snapshot(&Uuid::new_v4())
            .expect("lookup should not error");
        assert!(result.is_none());
    }

    #[test]
    fn in_memory_store_saves_and_loads_snapshot() {
        let store = InMemorySnapshotStore::new();
        let snapshot = make_snapshot("Hello");
        let doc_id = snapshot.document.id;

        store.save_snapshot(snapshot).expect("save should succeed");

        let loaded = store
            .load_snapshot(&doc_id)
            .expect("load should not error")
            .expect("snapshot should exist after save");

        assert_eq!(loaded.document.id, doc_id);
        assert_eq!(loaded.update, vec![1, 2, 3]);
    }

    #[test]
    fn in_memory_store_replaces_existing_snapshot_on_second_save() {
        let store = InMemorySnapshotStore::new();
        let doc = Document::new(Uuid::new_v4(), Some("Original".to_owned()));
        let doc_id = doc.id;

        store
            .save_snapshot(DocumentSnapshot::new(doc.clone(), vec![1]))
            .expect("first save should succeed");
        store
            .save_snapshot(DocumentSnapshot::new(doc, vec![2]))
            .expect("second save should succeed");

        let loaded = store
            .load_snapshot(&doc_id)
            .expect("load should not error")
            .expect("snapshot should exist");
        assert_eq!(loaded.update, vec![2]);
    }

    #[test]
    fn in_memory_store_delete_removes_snapshot() {
        let store = InMemorySnapshotStore::new();
        let snapshot = make_snapshot("Deletable");
        let doc_id = snapshot.document.id;

        store.save_snapshot(snapshot).expect("save should succeed");
        store
            .delete_snapshot(&doc_id)
            .expect("delete should succeed");

        let result = store
            .load_snapshot(&doc_id)
            .expect("lookup after delete should not error");
        assert!(result.is_none());
    }

    #[test]
    fn in_memory_store_delete_is_idempotent_for_missing_snapshot() {
        let store = InMemorySnapshotStore::new();
        store
            .delete_snapshot(&Uuid::new_v4())
            .expect("deleting a non-existent snapshot should not error");
    }

    #[test]
    fn in_memory_store_lists_all_saved_documents() {
        let store = InMemorySnapshotStore::new();
        let snapshot_a = make_snapshot("Alpha");
        let snapshot_b = make_snapshot("Beta");
        let id_a = snapshot_a.document.id;
        let id_b = snapshot_b.document.id;

        store
            .save_snapshot(snapshot_a)
            .expect("save A should succeed");
        store
            .save_snapshot(snapshot_b)
            .expect("save B should succeed");

        let mut listed = store.list_documents().expect("list should succeed");
        listed.sort_by_key(|d| d.id);

        let mut expected_ids = vec![id_a, id_b];
        expected_ids.sort();

        assert_eq!(
            listed.iter().map(|d| d.id).collect::<Vec<_>>(),
            expected_ids
        );
    }

    #[test]
    fn in_memory_store_returns_empty_list_when_no_snapshots() {
        let store = InMemorySnapshotStore::new();
        let documents = store.list_documents().expect("list should succeed");
        assert!(documents.is_empty());
    }

    #[test]
    fn in_memory_store_excludes_deleted_document_from_list() {
        let store = InMemorySnapshotStore::new();
        let snapshot = make_snapshot("Temporary");
        let doc_id = snapshot.document.id;

        store.save_snapshot(snapshot).expect("save should succeed");
        store
            .delete_snapshot(&doc_id)
            .expect("delete should succeed");

        let documents = store.list_documents().expect("list should succeed");
        assert!(documents.is_empty());
    }
}

pub fn snapshot_store_from_config(config: &Config) -> Result<Arc<dyn SnapshotStore>, StorageError> {
    match config.snapshot_store.trim().to_ascii_lowercase().as_str() {
        "memory" => Ok(in_memory_snapshot_store()),
        "file" => file_snapshot_store(&config.snapshot_dir),
        #[cfg(feature = "full-snapshot-stores")]
        "agdb" => Ok(Arc::new(AgdbSnapshotStore::new(
            &config.snapshot_agdb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "amandine" => Ok(Arc::new(AmandineSnapshotStore::new(
            &config.snapshot_amandine_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "append_log" => Ok(Arc::new(AppendLogSnapshotStore::new(
            &config.snapshot_append_log_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "apex_store" => Ok(Arc::new(ApexStoreSnapshotStore::new(
            &config.snapshot_apex_store_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "armdb" => Ok(Arc::new(ArmdbSnapshotStore::new(
            &config.snapshot_armdb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "assystem" => Ok(Arc::new(AssystemSnapshotStore::new(
            &config.snapshot_assystem_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "colon_db" => Ok(Arc::new(ColonDbSnapshotStore::new(
            &config.snapshot_colon_db_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "flash_kv" => Ok(Arc::new(FlashKvSnapshotStore::new(
            &config.snapshot_flash_kv_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "ghaladb" => Ok(Arc::new(GhaladbSnapshotStore::new(
            &config.snapshot_ghaladb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "blockbucket" => Ok(Arc::new(BlockbucketSnapshotStore::new(
            &config.snapshot_blockbucket_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "grebedb" => Ok(Arc::new(GrebedbSnapshotStore::new(
            &config.snapshot_grebedb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "grumpydb" => Ok(Arc::new(GrumpydbSnapshotStore::new(
            &config.snapshot_grumpydb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "graus_db" => Ok(Arc::new(GrausDbSnapshotStore::new(
            &config.snapshot_graus_db_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "highlandcows_isam" => Ok(Arc::new(HighlandcowsIsamSnapshotStore::new(
            &config.snapshot_highlandcows_isam_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "simple_db" => Ok(Arc::new(SimpleDbSnapshotStore::new(
            &config.snapshot_simple_db_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "docdb" => Ok(Arc::new(DocDbSnapshotStore::new(
            &config.snapshot_docdb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "emdb" => Ok(Arc::new(EmdbSnapshotStore::new(
            &config.snapshot_emdb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "osmiumdb" => Ok(Arc::new(OsmiumdbSnapshotStore::new(
            &config.snapshot_osmiumdb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "eight" => Ok(Arc::new(EightSnapshotStore::new(
            &config.snapshot_eight_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "epoch_db" => Ok(Arc::new(EpochDbSnapshotStore::new(
            &config.snapshot_epoch_db_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "etchdb" => Ok(Arc::new(EtchdbSnapshotStore::new(
            &config.snapshot_etchdb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "fastkv" => Ok(Arc::new(FastKvSnapshotStore::new(
            &config.snapshot_fastkv_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "ferrumdb" => Ok(Arc::new(FerrumdbSnapshotStore::new(
            &config.snapshot_ferrumdb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "rumdb" => Ok(Arc::new(RumDbSnapshotStore::new(
            &config.snapshot_rumdb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "rubin" => Ok(Arc::new(RubinSnapshotStore::new(
            &config.snapshot_rubin_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "shorterdb" => Ok(Arc::new(ShorterDbSnapshotStore::new(
            &config.snapshot_shorterdb_path,
        )?)),
        "sqlite" => Ok(Arc::new(SqliteSnapshotStore::new(
            &config.snapshot_sqlite_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "heed" => Ok(Arc::new(HeedSnapshotStore::new(
            &config.snapshot_heed_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "hightower_kv" => Ok(Arc::new(HightowerKvSnapshotStore::new(
            &config.snapshot_hightower_kv_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "hmdb" => Ok(Arc::new(HmdbSnapshotStore::new(
            &config.snapshot_hmdb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "hurrahdb" => Ok(Arc::new(HurrahdbSnapshotStore::new(
            &config.snapshot_hurrahdb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "fs_db" => Ok(Arc::new(FsDbSnapshotStore::new(
            &config.snapshot_fs_db_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "sqjson" => Ok(Arc::new(SqjsonSnapshotStore::new(
            &config.snapshot_sqjson_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "icefalldb" => Ok(Arc::new(IcefalldbSnapshotStore::new(
            &config.snapshot_icefalldb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "bitask" => Ok(Arc::new(BitaskSnapshotStore::new(
            &config.snapshot_bitask_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "bitkv_rs" => Ok(Arc::new(BitkvRsSnapshotStore::new(
            &config.snapshot_bitkv_rs_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "bitcask_engine" => Ok(Arc::new(BitcaskEngineSnapshotStore::new(
            &config.snapshot_bitcask_engine_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "blazeup" => Ok(Arc::new(BlazeupSnapshotStore::new(
            &config.snapshot_blazeup_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "candystore" => Ok(Arc::new(CandystoreSnapshotStore::new(
            &config.snapshot_candystore_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "celerix_store" => Ok(Arc::new(CelerixStoreSnapshotStore::new(
            &config.snapshot_celerix_store_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "citadeldb" => Ok(Arc::new(CitadeldbSnapshotStore::new(
            &config.snapshot_citadeldb_path,
            &config.snapshot_citadeldb_passphrase,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "cuendillar" => Ok(Arc::new(CuendillarSnapshotStore::new(
            &config.snapshot_cuendillar_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "data_pile" => Ok(Arc::new(DataPileSnapshotStore::new(
            &config.snapshot_data_pile_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "datastack" => Ok(Arc::new(DatastackSnapshotStore::new(
            &config.snapshot_datastack_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "jammdb" => Ok(Arc::new(JammdbSnapshotStore::new(
            &config.snapshot_jammdb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "mace" => Ok(Arc::new(MaceSnapshotStore::new(
            &config.snapshot_mace_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "janql" => Ok(Arc::new(JanqlSnapshotStore::new(
            &config.snapshot_janql_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "jasondb" => Ok(Arc::new(JasondbSnapshotStore::new(
            &config.snapshot_jasondb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "jasonisnthappy" => Ok(Arc::new(JasonisnthappySnapshotStore::new(
            &config.snapshot_jasonisnthappy_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "jfs" => Ok(Arc::new(JfsSnapshotStore::new(&config.snapshot_jfs_path)?)),
        #[cfg(feature = "full-snapshot-stores")]
        "json_store" => Ok(Arc::new(JsonStoreSnapshotStore::new(
            &config.snapshot_json_store_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "json_db_rs" => Ok(Arc::new(JsonDbRsSnapshotStore::new(
            &config.snapshot_json_db_rs_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "cdb64" => Ok(Arc::new(Cdb64SnapshotStore::new(
            &config.snapshot_cdb64_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "json_mutex_db" => Ok(Arc::new(JsonMutexDbSnapshotStore::new(
            &config.snapshot_json_mutex_db_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "toiletdb" => Ok(Arc::new(ToiletdbSnapshotStore::new(
            &config.snapshot_toiletdb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "feoxdb" => Ok(Arc::new(FeoxdbSnapshotStore::new(
            &config.snapshot_feoxdb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "jsondb" => Ok(Arc::new(JsondbSnapshotStore::new(
            &config.snapshot_jsondb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "kopperdb" => Ok(Arc::new(KopperdbSnapshotStore::new(
            &config.snapshot_kopperdb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "kv" => Ok(Arc::new(KvSnapshotStore::new(&config.snapshot_kv_path)?)),
        #[cfg(feature = "full-snapshot-stores")]
        "koit" => Ok(Arc::new(KoitSnapshotStore::new(
            &config.snapshot_koit_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "lite_db" => Ok(Arc::new(LiteDbSnapshotStore::new(
            &config.snapshot_lite_db_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "lmdb_rs_core" => Ok(Arc::new(LmdbRsCoreSnapshotStore::new(
            &config.snapshot_lmdb_rs_core_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "log_kv" => Ok(Arc::new(LogKvSnapshotStore::new(
            &config.snapshot_log_kv_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "append_kv" => Ok(Arc::new(AppendKvSnapshotStore::new(
            &config.snapshot_append_kv_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "mhdb" => Ok(Arc::new(MhdbSnapshotStore::new(
            &config.snapshot_mhdb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "marble" => Ok(Arc::new(MarbleSnapshotStore::new(
            &config.snapshot_marble_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "loro_kv" => Ok(Arc::new(LoroKvSnapshotStore::new(
            &config.snapshot_loro_kv_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "luckdb" => Ok(Arc::new(LuckdbSnapshotStore::new(
            &config.snapshot_luckdb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "ipjdb" => Ok(Arc::new(IpjdbSnapshotStore::new(
            &config.snapshot_ipjdb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "kagi" => Ok(Arc::new(KagiSnapshotStore::new(
            &config.snapshot_kagi_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "deeb" => Ok(Arc::new(DeebSnapshotStore::new(
            &config.snapshot_deeb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "lsm_engine" => Ok(Arc::new(LsmEngineSnapshotStore::new(
            &config.snapshot_lsm_engine_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "lsm_storage_engine" => Ok(Arc::new(LsmStorageEngineSnapshotStore::new(
            &config.snapshot_lsm_storage_engine_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "lsmdb" => Ok(Arc::new(LsmdbSnapshotStore::new(
            &config.snapshot_lsmdb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "lsm_tree" => Ok(Arc::new(LsmTreeSnapshotStore::new(
            &config.snapshot_lsm_tree_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "mindb" => Ok(Arc::new(MindbSnapshotStore::new(
            &config.snapshot_mindb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "mmdb" => Ok(Arc::new(MmdbSnapshotStore::new(
            &config.snapshot_mmdb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "mu_db" => Ok(Arc::new(MuDbSnapshotStore::new(
            &config.snapshot_mu_db_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "nanodb" => Ok(Arc::new(NanodbSnapshotStore::new(
            &config.snapshot_nanodb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "fjall" => Ok(Arc::new(FjallSnapshotStore::new(
            &config.snapshot_fjall_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "persy" => Ok(Arc::new(PersySnapshotStore::new(
            &config.snapshot_persy_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "persistent_kv" => Ok(Arc::new(PersistentKvSnapshotStore::new(
            &config.snapshot_persistent_kv_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "native_db" => Ok(Arc::new(NativeDbSnapshotStore::new(
            &config.snapshot_native_db_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "nebari" => Ok(Arc::new(NebariSnapshotStore::new(
            &config.snapshot_nebari_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "nikidb" => Ok(Arc::new(NikidbSnapshotStore::new(
            &config.snapshot_nikidb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "nodb" => Ok(Arc::new(NodbSnapshotStore::new(
            &config.snapshot_nodb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "okofdb" => Ok(Arc::new(OkofdbSnapshotStore::new(
            &config.snapshot_okofdb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "parity_db" => Ok(Arc::new(ParityDbSnapshotStore::new(
            &config.snapshot_parity_db_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "pickledb" => Ok(Arc::new(PickleDbSnapshotStore::new(
            &config.snapshot_pickledb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "rcask" => Ok(Arc::new(RcaskSnapshotStore::new(
            &config.snapshot_rcask_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "microkv" => Ok(Arc::new(MicroKvSnapshotStore::new(
            &config.snapshot_microkv_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "redb" => Ok(Arc::new(RedbSnapshotStore::new(
            &config.snapshot_redb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "rskey" => Ok(Arc::new(RskeySnapshotStore::new(
            &config.snapshot_rskey_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "readb" => Ok(Arc::new(ReadbSnapshotStore::new(
            &config.snapshot_readb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "rustlite" => Ok(Arc::new(RustliteSnapshotStore::new(
            &config.snapshot_rustlite_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "canopydb" => Ok(Arc::new(CanopydbSnapshotStore::new(
            &config.snapshot_canopydb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "caves" => Ok(Arc::new(CavesSnapshotStore::new(
            &config.snapshot_caves_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "ckydb" => Ok(Arc::new(CkydbSnapshotStore::new(
            &config.snapshot_ckydb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "crepedb" => Ok(Arc::new(CrepeDbSnapshotStore::new(
            &config.snapshot_crepedb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "crystal" => Ok(Arc::new(CrystalSnapshotStore::new(
            &config.snapshot_crystal_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "scdb" => Ok(Arc::new(ScdbSnapshotStore::new(
            &config.snapshot_scdb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "skv" => Ok(Arc::new(SkvSnapshotStore::new(&config.snapshot_skv_path)?)),
        #[cfg(feature = "full-snapshot-stores")]
        "surrealkv" => Ok(Arc::new(SurrealkvSnapshotStore::new(
            &config.snapshot_surrealkv_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "sled" => Ok(Arc::new(SledSnapshotStore::new(
            &config.snapshot_sled_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "rustbreak" => Ok(Arc::new(RustbreakSnapshotStore::new(
            &config.snapshot_rustbreak_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "rustcask" => Ok(Arc::new(RustcaskSnapshotStore::new(
            &config.snapshot_rustcask_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "rusty_leveldb" => Ok(Arc::new(RustyLeveldbSnapshotStore::new(
            &config.snapshot_rusty_leveldb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "yedb" => Ok(Arc::new(YedbSnapshotStore::new(
            &config.snapshot_yedb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "btree_store" => Ok(Arc::new(BtreeStoreSnapshotStore::new(
            &config.snapshot_btree_store_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "cacache" => Ok(Arc::new(CacacheSnapshotStore::new(
            &config.snapshot_cacache_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "siamesedb" => Ok(Arc::new(SiamesedbSnapshotStore::new(
            &config.snapshot_siamesedb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "structsy" => Ok(Arc::new(StructsySnapshotStore::new(
            &config.snapshot_structsy_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "abyssiniandb" => Ok(Arc::new(AbyssiniandbSnapshotStore::new(
            &config.snapshot_abyssiniandb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "aeternusdb" => Ok(Arc::new(AeternusdbSnapshotStore::new(
            &config.snapshot_aeternusdb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "thunderdb" => Ok(Arc::new(ThunderdbSnapshotStore::new(
            &config.snapshot_thunderdb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "thetadb" => Ok(Arc::new(ThetadbSnapshotStore::new(
            &config.snapshot_thetadb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "tinybase" => Ok(Arc::new(TinybaseSnapshotStore::new(
            &config.snapshot_tinybase_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "tinydb" => Ok(Arc::new(TinydbSnapshotStore::new(
            &config.snapshot_tinydb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "dblite" => Ok(Arc::new(DbliteSnapshotStore::new(
            &config.snapshot_dblite_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "dbless" => Ok(Arc::new(DblessSnapshotStore::new(
            &config.snapshot_dbless_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "db_rs" => Ok(Arc::new(DbRsSnapshotStore::new(
            &config.snapshot_db_rs_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "dharmadb" => Ok(Arc::new(DharmadbSnapshotStore::new(
            &config.snapshot_dharmadb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "dir_cache" => Ok(Arc::new(DirCacheSnapshotStore::new(
            &config.snapshot_dir_cache_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "sanakirja" => Ok(Arc::new(SanakirjaSnapshotStore::new(
            &config.snapshot_sanakirja_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "saturn" => Ok(Arc::new(SaturnSnapshotStore::new(
            &config.snapshot_saturn_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "snaildb" => Ok(Arc::new(SnaildbSnapshotStore::new(
            &config.snapshot_snaildb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "tinykv" => Ok(Arc::new(TinykvSnapshotStore::new(
            &config.snapshot_tinykv_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "vsdb" => Ok(Arc::new(VsdbSnapshotStore::new(
            &config.snapshot_vsdb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "yakv" => Ok(Arc::new(YakvSnapshotStore::new(
            &config.snapshot_yakv_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "yakvdb" => Ok(Arc::new(YakvdbSnapshotStore::new(
            &config.snapshot_yakvdb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "saberdb" => Ok(Arc::new(SaberdbSnapshotStore::new(
            &config.snapshot_saberdb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "smolldb" => Ok(Arc::new(SmolldbSnapshotStore::new(
            &config.snapshot_smolldb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "kstone" => Ok(Arc::new(KstoneSnapshotStore::new(
            &config.snapshot_kstone_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "roughdb" => Ok(Arc::new(RoughdbSnapshotStore::new(
            &config.snapshot_roughdb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "raindb" => Ok(Arc::new(RaindbSnapshotStore::new(
            &config.snapshot_raindb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "infusedb" => Ok(Arc::new(InfusedbSnapshotStore::new(
            &config.snapshot_infusedb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "kafi" => Ok(Arc::new(KafiSnapshotStore::new(
            &config.snapshot_kafi_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "tinkv" => Ok(Arc::new(TinkvSnapshotStore::new(
            &config.snapshot_tinkv_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "ledger_kv" => Ok(Arc::new(LedgerKvSnapshotStore::new(
            &config.snapshot_ledger_kv_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "joydb" => Ok(Arc::new(JoydbSnapshotStore::new(
            &config.snapshot_joydb_path,
        )?)),
        #[cfg(feature = "full-snapshot-stores")]
        "png_db" => Ok(Arc::new(PngDbSnapshotStore::new(
            &config.snapshot_png_db_path,
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
        other => Err(invalid_snapshot_store_error(other)),
    }
}

pub(crate) fn ensure_snapshot_dir(path: &Path) -> Result<(), StorageError> {
    fs::create_dir_all(path)
        .map_err(|error| StorageError::Io(format!("{}: {error}", path.display())))
}

#[cfg(test)]
mod snapshot_store_config_tests {
    use super::invalid_snapshot_store_error;
    use crate::storage::StorageError;

    const EXPECTED_ALWAYS_ON_STORES: &[&str] = &["file", "managed", "memory", "s3", "sqlite"];

    #[test]
    fn invalid_snapshot_store_error_lists_default_build_backends() {
        let StorageError::Config(message) = invalid_snapshot_store_error("unsupported") else {
            panic!("invalid snapshot store should yield a config error");
        };

        for store in EXPECTED_ALWAYS_ON_STORES {
            assert!(
                message.contains(&format!("`{store}`")),
                "config error should mention `{store}`"
            );
        }

        assert!(
            message.ends_with("received `unsupported`"),
            "config error should include the rejected store value"
        );
    }

    #[cfg(feature = "full-snapshot-stores")]
    #[test]
    fn invalid_snapshot_store_error_lists_extended_backends() {
        let StorageError::Config(message) = invalid_snapshot_store_error("unsupported") else {
            panic!("invalid snapshot store should yield a config error");
        };

        for store in [
            "append_kv",
            "append_log",
            "dir_cache",
            "lmdb_rs_core",
            "marble",
            "mu_db",
            "png_db",
            "toiletdb",
            "yakvdb",
        ] {
            assert!(
                message.contains(&format!("`{store}`")),
                "config error should mention `{store}`"
            );
        }
    }
}
