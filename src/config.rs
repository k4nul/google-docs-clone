use std::env;

use axum::http::{HeaderValue, Uri};

use crate::errors::{AppError, AppResult};

pub const DEFAULT_HOST: &str = "127.0.0.1";
pub const DEFAULT_PORT: u16 = 4000;
pub const DEFAULT_FRONTEND_ORIGIN: &str = "http://localhost:3000";
pub const DEFAULT_RUST_LOG: &str = "backend=debug,tower_http=info";
pub const DEFAULT_API_TOKEN: &str = "dev-admin-token";
pub const DEFAULT_SNAPSHOT_STORE: &str = "memory";
pub const DEFAULT_SNAPSHOT_DIR: &str = "./data/snapshots";
pub const DEFAULT_SNAPSHOT_AGDB_PATH: &str = "./data/snapshots.agdb";
pub const DEFAULT_SNAPSHOT_AMANDINE_PATH: &str = "./data/snapshots.amandine";
pub const DEFAULT_SNAPSHOT_APPEND_LOG_PATH: &str = "./data/snapshots.append_log";
pub const DEFAULT_SNAPSHOT_APEX_STORE_PATH: &str = "./data/snapshots.apex_store";
pub const DEFAULT_SNAPSHOT_ARMDB_PATH: &str = "./data/snapshots.armdb";
pub const DEFAULT_SNAPSHOT_ASSYSTEM_PATH: &str = "./data/snapshots.assystem";
pub const DEFAULT_SNAPSHOT_COLON_DB_PATH: &str = "./data/snapshots.colon_db";
pub const DEFAULT_SNAPSHOT_FLASH_KV_PATH: &str = "./data/snapshots.flash_kv";
pub const DEFAULT_SNAPSHOT_GHALADB_PATH: &str = "./data/snapshots.ghaladb";
pub const DEFAULT_SNAPSHOT_BLOCKBUCKET_PATH: &str = "./data/snapshots.blockbucket";
pub const DEFAULT_SNAPSHOT_GREBEDB_PATH: &str = "./data/snapshots.grebedb";
pub const DEFAULT_SNAPSHOT_GRUMPYDB_PATH: &str = "./data/snapshots.grumpydb";
pub const DEFAULT_SNAPSHOT_GRAUS_DB_PATH: &str = "./data/snapshots.graus_db";
pub const DEFAULT_SNAPSHOT_HIGHLANDCOWS_ISAM_PATH: &str = "./data/snapshots.highlandcows_isam";
pub const DEFAULT_SNAPSHOT_SIMPLE_DB_PATH: &str = "./data/snapshots.simple_db";
pub const DEFAULT_SNAPSHOT_DOCDB_PATH: &str = "./data/snapshots.docdb.json";
pub const DEFAULT_SNAPSHOT_EMDB_PATH: &str = "./data/snapshots.emdb";
pub const DEFAULT_SNAPSHOT_OSMIUMDB_PATH: &str = "./data/snapshots.osmiumdb";
pub const DEFAULT_SNAPSHOT_EIGHT_PATH: &str = "./data/snapshots.eight";
pub const DEFAULT_SNAPSHOT_EPOCH_DB_PATH: &str = "./data/snapshots.epoch_db";
pub const DEFAULT_SNAPSHOT_ETCHDB_PATH: &str = "./data/snapshots.etchdb";
pub const DEFAULT_SNAPSHOT_FASTKV_PATH: &str = "./data/snapshots.fastkv";
pub const DEFAULT_SNAPSHOT_FERRUMDB_PATH: &str = "./data/snapshots.ferrumdb";
pub const DEFAULT_SNAPSHOT_RUMDB_PATH: &str = "./data/snapshots.rumdb";
pub const DEFAULT_SNAPSHOT_RUBIN_PATH: &str = "./data/snapshots.rubin.json";
pub const DEFAULT_SNAPSHOT_SHORTERDB_PATH: &str = "./data/snapshots.shorterdb";
pub const DEFAULT_SNAPSHOT_SQLITE_PATH: &str = "./data/snapshots.sqlite3";
pub const DEFAULT_SNAPSHOT_HEED_PATH: &str = "./data/snapshots.heed";
pub const DEFAULT_SNAPSHOT_HIGHTOWER_KV_PATH: &str = "./data/snapshots.hightower_kv";
pub const DEFAULT_SNAPSHOT_HMDB_PATH: &str = "./data/snapshots.hmdb";
pub const DEFAULT_SNAPSHOT_HURRAHDB_PATH: &str = "./data/snapshots.hurrahdb";
pub const DEFAULT_SNAPSHOT_FS_DB_PATH: &str = "./data/snapshots.fs_db";
pub const DEFAULT_SNAPSHOT_SQJSON_PATH: &str = "./data/snapshots.sqjson";
pub const DEFAULT_SNAPSHOT_ICEFALLDB_PATH: &str = "./data/snapshots.icefalldb";
pub const DEFAULT_SNAPSHOT_BITASK_PATH: &str = "./data/snapshots.bitask";
pub const DEFAULT_SNAPSHOT_BITKV_RS_PATH: &str = "./data/snapshots.bitkv_rs";
pub const DEFAULT_SNAPSHOT_BITCASK_ENGINE_PATH: &str = "./data/snapshots.bitcask_engine";
pub const DEFAULT_SNAPSHOT_BLAZEUP_PATH: &str = "./data/snapshots.blazeup";
pub const DEFAULT_SNAPSHOT_CANDYSTORE_PATH: &str = "./data/snapshots.candystore";
pub const DEFAULT_SNAPSHOT_CELERIX_STORE_PATH: &str = "./data/snapshots.celerix_store";
pub const DEFAULT_SNAPSHOT_CITADELDB_PATH: &str = "./data/snapshots.citadeldb";
pub const DEFAULT_SNAPSHOT_CITADELDB_PASSPHRASE: &str = "dev-citadel-snapshot-passphrase";
pub const DEFAULT_SNAPSHOT_CUENDILLAR_PATH: &str = "./data/snapshots.cuendillar";
pub const DEFAULT_SNAPSHOT_DATA_PILE_PATH: &str = "./data/snapshots.data_pile";
pub const DEFAULT_SNAPSHOT_DATASTACK_PATH: &str = "./data/snapshots.datastack";
pub const DEFAULT_SNAPSHOT_JAMMDB_PATH: &str = "./data/snapshots.jammdb";
pub const DEFAULT_SNAPSHOT_MACE_PATH: &str = "./data/snapshots.mace";
pub const DEFAULT_SNAPSHOT_JANQL_PATH: &str = "./data/snapshots.janql";
pub const DEFAULT_SNAPSHOT_JASONDB_PATH: &str = "./data/snapshots.jasondb";
pub const DEFAULT_SNAPSHOT_JASONISNTHAPPY_PATH: &str = "./data/snapshots.jasonisnthappy";
pub const DEFAULT_SNAPSHOT_JFS_PATH: &str = "./data/snapshots.jfs.json";
pub const DEFAULT_SNAPSHOT_JSON_STORE_PATH: &str = "./data/snapshots.json_store.jsonl";
pub const DEFAULT_SNAPSHOT_JSON_DB_RS_PATH: &str = "./data/snapshots.json_db_rs.json";
pub const DEFAULT_SNAPSHOT_CDB64_PATH: &str = "./data/snapshots.cdb64";
pub const DEFAULT_SNAPSHOT_JSON_MUTEX_DB_PATH: &str = "./data/snapshots.json_mutex_db.json";
pub const DEFAULT_SNAPSHOT_TOILETDB_PATH: &str = "./data/snapshots.toiletdb.json";
pub const DEFAULT_SNAPSHOT_FEOXDB_PATH: &str = "./data/snapshots.feoxdb";
pub const DEFAULT_SNAPSHOT_JSONDB_PATH: &str = "./data/snapshots.jsondb.json";
pub const DEFAULT_SNAPSHOT_KOPPERDB_PATH: &str = "./data/snapshots.kopperdb";
pub const DEFAULT_SNAPSHOT_KV_PATH: &str = "./data/snapshots.kv";
pub const DEFAULT_SNAPSHOT_KOIT_PATH: &str = "./data/snapshots.koit.json";
pub const DEFAULT_SNAPSHOT_LITE_DB_PATH: &str = "./data/snapshots.lite_db";
pub const DEFAULT_SNAPSHOT_LMDB_RS_CORE_PATH: &str = "./data/snapshots.lmdb_rs_core";
pub const DEFAULT_SNAPSHOT_LOG_KV_PATH: &str = "./data/snapshots.log_kv";
pub const DEFAULT_SNAPSHOT_APPEND_KV_PATH: &str = "./data/snapshots.append_kv";
pub const DEFAULT_SNAPSHOT_MHDB_PATH: &str = "./data/snapshots.mhdb";
pub const DEFAULT_SNAPSHOT_MARBLE_PATH: &str = "./data/snapshots.marble";
pub const DEFAULT_SNAPSHOT_LORO_KV_PATH: &str = "./data/snapshots.loro_kv";
pub const DEFAULT_SNAPSHOT_LUCKDB_PATH: &str = "./data/snapshots.luckdb.json";
pub const DEFAULT_SNAPSHOT_IPJDB_PATH: &str = "./data/snapshots.ipjdb";
pub const DEFAULT_SNAPSHOT_KAGI_PATH: &str = "./data/snapshots.kagi";
pub const DEFAULT_SNAPSHOT_DEEB_PATH: &str = "./data/snapshots.deeb.json";
pub const DEFAULT_SNAPSHOT_LSM_ENGINE_PATH: &str = "./data/snapshots.lsm_engine";
pub const DEFAULT_SNAPSHOT_LSM_STORAGE_ENGINE_PATH: &str = "./data/snapshots.lsm_storage_engine";
pub const DEFAULT_SNAPSHOT_LSMDB_PATH: &str = "./data/snapshots.lsmdb";
pub const DEFAULT_SNAPSHOT_LSM_TREE_PATH: &str = "./data/snapshots.lsm_tree";
pub const DEFAULT_SNAPSHOT_MINDB_PATH: &str = "./data/snapshots.mindb";
pub const DEFAULT_SNAPSHOT_MMDB_PATH: &str = "./data/snapshots.mmdb";
pub const DEFAULT_SNAPSHOT_MU_DB_PATH: &str = "./data/snapshots.mu_db";
pub const DEFAULT_SNAPSHOT_NANODB_PATH: &str = "./data/snapshots.nanodb.json";
pub const DEFAULT_SNAPSHOT_FJALL_PATH: &str = "./data/snapshots.fjall";
pub const DEFAULT_SNAPSHOT_PERSY_PATH: &str = "./data/snapshots.persy";
pub const DEFAULT_SNAPSHOT_PERSISTENT_KV_PATH: &str = "./data/snapshots.persistent_kv";
pub const DEFAULT_SNAPSHOT_NATIVE_DB_PATH: &str = "./data/snapshots.native_db";
pub const DEFAULT_SNAPSHOT_NEBARI_PATH: &str = "./data/snapshots.nebari";
pub const DEFAULT_SNAPSHOT_NIKIDB_PATH: &str = "./data/snapshots.nikidb";
pub const DEFAULT_SNAPSHOT_NODB_PATH: &str = "./data/snapshots.nodb";
pub const DEFAULT_SNAPSHOT_OKOFDB_PATH: &str = "./data/snapshots.okofdb";
pub const DEFAULT_SNAPSHOT_PARITY_DB_PATH: &str = "./data/snapshots.parity_db";
pub const DEFAULT_SNAPSHOT_PICKLEDB_PATH: &str = "./data/snapshots.pickledb";
pub const DEFAULT_SNAPSHOT_RCASK_PATH: &str = "./data/snapshots.rcask";
pub const DEFAULT_SNAPSHOT_MICROKV_PATH: &str = "./data/snapshots_microkv";
pub const DEFAULT_SNAPSHOT_REDB_PATH: &str = "./data/snapshots.redb";
pub const DEFAULT_SNAPSHOT_RSKEY_PATH: &str = "./data/snapshots.rskey";
pub const DEFAULT_SNAPSHOT_READB_PATH: &str = "./data/snapshots.readb";
pub const DEFAULT_SNAPSHOT_RUSTLITE_PATH: &str = "./data/snapshots.rustlite";
pub const DEFAULT_SNAPSHOT_RUSTCASK_PATH: &str = "./data/snapshots.rustcask";
pub const DEFAULT_SNAPSHOT_RUSTY_LEVELDB_PATH: &str = "./data/snapshots.rusty_leveldb";
pub const DEFAULT_SNAPSHOT_CANOPYDB_PATH: &str = "./data/snapshots.canopydb";
pub const DEFAULT_SNAPSHOT_CAVES_PATH: &str = "./data/snapshots.caves";
pub const DEFAULT_SNAPSHOT_CKYDB_PATH: &str = "./data/snapshots.ckydb";
pub const DEFAULT_SNAPSHOT_CREPEDB_PATH: &str = "./data/snapshots.crepedb";
pub const DEFAULT_SNAPSHOT_CRYSTAL_PATH: &str = "./data/snapshots.crystal";
pub const DEFAULT_SNAPSHOT_SCDB_PATH: &str = "./data/snapshots.scdb";
pub const DEFAULT_SNAPSHOT_SKV_PATH: &str = "./data/snapshots.skv";
pub const DEFAULT_SNAPSHOT_SURREALKV_PATH: &str = "./data/snapshots.surrealkv";
pub const DEFAULT_SNAPSHOT_SLED_PATH: &str = "./data/snapshots.sled";
pub const DEFAULT_SNAPSHOT_RUSTBREAK_PATH: &str = "./data/snapshots.rustbreak";
pub const DEFAULT_SNAPSHOT_YEDB_PATH: &str = "./data/snapshots.yedb";
pub const DEFAULT_SNAPSHOT_BTREE_STORE_PATH: &str = "./data/snapshots.btree_store";
pub const DEFAULT_SNAPSHOT_CACACHE_PATH: &str = "./data/snapshots.cacache";
pub const DEFAULT_SNAPSHOT_SIAMESDB_PATH: &str = "./data/snapshots.siamesedb";
pub const DEFAULT_SNAPSHOT_STRUCTSY_PATH: &str = "./data/snapshots.structsy";
pub const DEFAULT_SNAPSHOT_ABYSSINIANDB_PATH: &str = "./data/snapshots.abyssiniandb";
pub const DEFAULT_SNAPSHOT_AETERNUSDB_PATH: &str = "./data/snapshots.aeternusdb";
pub const DEFAULT_SNAPSHOT_THUNDERDB_PATH: &str = "./data/snapshots.thunderdb";
pub const DEFAULT_SNAPSHOT_THETADB_PATH: &str = "./data/snapshots.thetadb";
pub const DEFAULT_SNAPSHOT_TINYBASE_PATH: &str = "./data/snapshots.tinybase";
pub const DEFAULT_SNAPSHOT_TINYDB_PATH: &str = "./data/snapshots.tinydb";
pub const DEFAULT_SNAPSHOT_DBLITE_PATH: &str = "./data/snapshots.dblite";
pub const DEFAULT_SNAPSHOT_DBLESS_PATH: &str = "./data/snapshots.dbless";
pub const DEFAULT_SNAPSHOT_DB_RS_PATH: &str = "./data/snapshots.db_rs";
pub const DEFAULT_SNAPSHOT_DHARMADB_PATH: &str = "./data/snapshots.dharmadb";
pub const DEFAULT_SNAPSHOT_DIR_CACHE_PATH: &str = "./data/snapshots.dir_cache";
pub const DEFAULT_SNAPSHOT_SANAKIRJA_PATH: &str = "./data/snapshots.sanakirja";
pub const DEFAULT_SNAPSHOT_SATURN_PATH: &str = "./data/snapshots.saturn";
pub const DEFAULT_SNAPSHOT_SNAILDB_PATH: &str = "./data/snapshots.snaildb";
pub const DEFAULT_SNAPSHOT_TINYKV_PATH: &str = "./data/snapshots.tinykv.json";
pub const DEFAULT_SNAPSHOT_VSDB_PATH: &str = "./data/snapshots.vsdb";
pub const DEFAULT_SNAPSHOT_YAKV_PATH: &str = "./data/snapshots.yakv";
pub const DEFAULT_SNAPSHOT_YAKVDB_PATH: &str = "./data/snapshots.yakvdb";
pub const DEFAULT_SNAPSHOT_SABERDB_PATH: &str = "./data/snapshots.saberdb.json";
pub const DEFAULT_SNAPSHOT_SMOLLDB_PATH: &str = "./data/snapshots.smolldb";
pub const DEFAULT_SNAPSHOT_KSTONE_PATH: &str = "./data/snapshots.kstone";
pub const DEFAULT_SNAPSHOT_ROUGHDB_PATH: &str = "./data/snapshots.roughdb";
pub const DEFAULT_SNAPSHOT_RAINDB_PATH: &str = "./data/snapshots.raindb";
pub const DEFAULT_SNAPSHOT_INFUSEDB_PATH: &str = "./data/snapshots.infusedb";
pub const DEFAULT_SNAPSHOT_KAFI_PATH: &str = "./data/snapshots.kafi";
pub const DEFAULT_SNAPSHOT_TINKV_PATH: &str = "./data/snapshots.tinkv";
pub const DEFAULT_SNAPSHOT_LEDGER_KV_PATH: &str = "./data/snapshots.ledger_kv";
pub const DEFAULT_SNAPSHOT_JOYDB_PATH: &str = "./data/snapshots.joydb.json";
pub const DEFAULT_SNAPSHOT_PNG_DB_PATH: &str = "./data/snapshots.png";
pub const DEFAULT_SNAPSHOT_S3_REGION: &str = "us-east-1";
pub const DEFAULT_SNAPSHOT_S3_PREFIX: &str = "snapshots/";
pub const DEFAULT_SNAPSHOT_S3_TIMEOUT_SECS: u64 = 5;
pub const DEFAULT_SNAPSHOT_S3_PATH_STYLE: bool = true;
pub const DEFAULT_SNAPSHOT_MANAGED_TIMEOUT_SECS: u64 = 5;
pub const DEFAULT_ROOM_LOCATOR: &str = "local";
pub const DEFAULT_ROOM_COORDINATOR: &str = "noop";
pub const DEFAULT_ROOM_COORDINATOR_STATE_DIR: &str = "./data/room-coordinator";
pub const DEFAULT_ROOM_COORDINATOR_SQLITE_PATH: &str = "./data/room-coordinator.sqlite3";
pub const DEFAULT_ROOM_COORDINATOR_HEARTBEAT_INTERVAL_SECS: u64 = 10;
pub const DEFAULT_ROOM_COORDINATOR_LEASE_TTL_SECS: u64 = 30;
pub const DEFAULT_ROOM_COORDINATION_MANAGED_TIMEOUT_SECS: u64 = 5;
pub const DEFAULT_NODE_ID: &str = "local-node";

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub frontend_origin: String,
    pub rust_log: String,
    pub api_token: String,
    pub snapshot_store: String,
    pub snapshot_dir: String,
    pub snapshot_agdb_path: String,
    pub snapshot_amandine_path: String,
    pub snapshot_append_log_path: String,
    pub snapshot_apex_store_path: String,
    pub snapshot_armdb_path: String,
    pub snapshot_assystem_path: String,
    pub snapshot_colon_db_path: String,
    pub snapshot_flash_kv_path: String,
    pub snapshot_ghaladb_path: String,
    pub snapshot_blockbucket_path: String,
    pub snapshot_grebedb_path: String,
    pub snapshot_grumpydb_path: String,
    pub snapshot_graus_db_path: String,
    pub snapshot_highlandcows_isam_path: String,
    pub snapshot_simple_db_path: String,
    pub snapshot_docdb_path: String,
    pub snapshot_emdb_path: String,
    pub snapshot_osmiumdb_path: String,
    pub snapshot_eight_path: String,
    pub snapshot_epoch_db_path: String,
    pub snapshot_etchdb_path: String,
    pub snapshot_fastkv_path: String,
    pub snapshot_ferrumdb_path: String,
    pub snapshot_rumdb_path: String,
    pub snapshot_rubin_path: String,
    pub snapshot_shorterdb_path: String,
    pub snapshot_sqlite_path: String,
    pub snapshot_heed_path: String,
    pub snapshot_hightower_kv_path: String,
    pub snapshot_hmdb_path: String,
    pub snapshot_hurrahdb_path: String,
    pub snapshot_fs_db_path: String,
    pub snapshot_sqjson_path: String,
    pub snapshot_icefalldb_path: String,
    pub snapshot_bitask_path: String,
    pub snapshot_bitkv_rs_path: String,
    pub snapshot_bitcask_engine_path: String,
    pub snapshot_blazeup_path: String,
    pub snapshot_candystore_path: String,
    pub snapshot_celerix_store_path: String,
    pub snapshot_citadeldb_path: String,
    pub snapshot_citadeldb_passphrase: String,
    pub snapshot_cuendillar_path: String,
    pub snapshot_data_pile_path: String,
    pub snapshot_datastack_path: String,
    pub snapshot_jammdb_path: String,
    pub snapshot_mace_path: String,
    pub snapshot_janql_path: String,
    pub snapshot_jasondb_path: String,
    pub snapshot_jasonisnthappy_path: String,
    pub snapshot_jfs_path: String,
    pub snapshot_json_store_path: String,
    pub snapshot_json_db_rs_path: String,
    pub snapshot_cdb64_path: String,
    pub snapshot_json_mutex_db_path: String,
    pub snapshot_toiletdb_path: String,
    pub snapshot_feoxdb_path: String,
    pub snapshot_jsondb_path: String,
    pub snapshot_kopperdb_path: String,
    pub snapshot_kv_path: String,
    pub snapshot_koit_path: String,
    pub snapshot_lite_db_path: String,
    pub snapshot_lmdb_rs_core_path: String,
    pub snapshot_log_kv_path: String,
    pub snapshot_append_kv_path: String,
    pub snapshot_mhdb_path: String,
    pub snapshot_marble_path: String,
    pub snapshot_loro_kv_path: String,
    pub snapshot_luckdb_path: String,
    pub snapshot_ipjdb_path: String,
    pub snapshot_kagi_path: String,
    pub snapshot_deeb_path: String,
    pub snapshot_lsm_engine_path: String,
    pub snapshot_lsm_storage_engine_path: String,
    pub snapshot_lsmdb_path: String,
    pub snapshot_lsm_tree_path: String,
    pub snapshot_mindb_path: String,
    pub snapshot_mmdb_path: String,
    pub snapshot_mu_db_path: String,
    pub snapshot_nanodb_path: String,
    pub snapshot_fjall_path: String,
    pub snapshot_persy_path: String,
    pub snapshot_persistent_kv_path: String,
    pub snapshot_native_db_path: String,
    pub snapshot_nebari_path: String,
    pub snapshot_nikidb_path: String,
    pub snapshot_nodb_path: String,
    pub snapshot_okofdb_path: String,
    pub snapshot_parity_db_path: String,
    pub snapshot_pickledb_path: String,
    pub snapshot_rcask_path: String,
    pub snapshot_microkv_path: String,
    pub snapshot_redb_path: String,
    pub snapshot_rskey_path: String,
    pub snapshot_readb_path: String,
    pub snapshot_rustlite_path: String,
    pub snapshot_rustcask_path: String,
    pub snapshot_rusty_leveldb_path: String,
    pub snapshot_canopydb_path: String,
    pub snapshot_caves_path: String,
    pub snapshot_ckydb_path: String,
    pub snapshot_crepedb_path: String,
    pub snapshot_crystal_path: String,
    pub snapshot_scdb_path: String,
    pub snapshot_skv_path: String,
    pub snapshot_surrealkv_path: String,
    pub snapshot_sled_path: String,
    pub snapshot_rustbreak_path: String,
    pub snapshot_yedb_path: String,
    pub snapshot_btree_store_path: String,
    pub snapshot_cacache_path: String,
    pub snapshot_siamesedb_path: String,
    pub snapshot_structsy_path: String,
    pub snapshot_abyssiniandb_path: String,
    pub snapshot_aeternusdb_path: String,
    pub snapshot_thunderdb_path: String,
    pub snapshot_thetadb_path: String,
    pub snapshot_tinybase_path: String,
    pub snapshot_tinydb_path: String,
    pub snapshot_dblite_path: String,
    pub snapshot_dbless_path: String,
    pub snapshot_db_rs_path: String,
    pub snapshot_dharmadb_path: String,
    pub snapshot_dir_cache_path: String,
    pub snapshot_sanakirja_path: String,
    pub snapshot_saturn_path: String,
    pub snapshot_snaildb_path: String,
    pub snapshot_tinykv_path: String,
    pub snapshot_vsdb_path: String,
    pub snapshot_yakv_path: String,
    pub snapshot_yakvdb_path: String,
    pub snapshot_saberdb_path: String,
    pub snapshot_smolldb_path: String,
    pub snapshot_kstone_path: String,
    pub snapshot_roughdb_path: String,
    pub snapshot_raindb_path: String,
    pub snapshot_infusedb_path: String,
    pub snapshot_kafi_path: String,
    pub snapshot_tinkv_path: String,
    pub snapshot_ledger_kv_path: String,
    pub snapshot_joydb_path: String,
    pub snapshot_png_db_path: String,
    pub snapshot_s3_endpoint: Option<String>,
    pub snapshot_s3_region: String,
    pub snapshot_s3_bucket: Option<String>,
    pub snapshot_s3_prefix: String,
    pub snapshot_s3_access_key_id: Option<String>,
    pub snapshot_s3_secret_access_key: Option<String>,
    pub snapshot_s3_session_token: Option<String>,
    pub snapshot_s3_timeout_secs: u64,
    pub snapshot_s3_path_style: bool,
    pub snapshot_managed_base_url: Option<String>,
    pub snapshot_managed_auth_token: Option<String>,
    pub snapshot_managed_timeout_secs: u64,
    pub room_locator: String,
    pub room_coordinator: String,
    pub room_coordinator_state_dir: String,
    pub room_coordinator_sqlite_path: String,
    pub room_coordinator_heartbeat_interval_secs: u64,
    pub room_coordinator_lease_ttl_secs: u64,
    pub room_coordination_managed_base_url: Option<String>,
    pub room_coordination_managed_auth_token: Option<String>,
    pub room_coordination_managed_timeout_secs: u64,
    pub node_id: String,
    pub node_base_url: Option<String>,
    pub room_owner_hints_path: Option<String>,
}

impl Config {
    pub fn from_env() -> AppResult<Self> {
        let _ = dotenvy::dotenv();

        let host = env_string("HOST", DEFAULT_HOST)?;
        let port = env_u16("PORT", DEFAULT_PORT)?;
        let frontend_origin = env_string("FRONTEND_ORIGIN", DEFAULT_FRONTEND_ORIGIN)?;
        let rust_log = env_string("RUST_LOG", DEFAULT_RUST_LOG)?;
        let api_token = env_string("API_TOKEN", DEFAULT_API_TOKEN)?;
        let snapshot_store = env_string("SNAPSHOT_STORE", DEFAULT_SNAPSHOT_STORE)?;
        let snapshot_dir = env_string("SNAPSHOT_DIR", DEFAULT_SNAPSHOT_DIR)?;
        let snapshot_agdb_path = env_string("SNAPSHOT_AGDB_PATH", DEFAULT_SNAPSHOT_AGDB_PATH)?;
        let snapshot_amandine_path =
            env_string("SNAPSHOT_AMANDINE_PATH", DEFAULT_SNAPSHOT_AMANDINE_PATH)?;
        let snapshot_append_log_path =
            env_string("SNAPSHOT_APPEND_LOG_PATH", DEFAULT_SNAPSHOT_APPEND_LOG_PATH)?;
        let snapshot_apex_store_path =
            env_string("SNAPSHOT_APEX_STORE_PATH", DEFAULT_SNAPSHOT_APEX_STORE_PATH)?;
        let snapshot_armdb_path = env_string("SNAPSHOT_ARMDB_PATH", DEFAULT_SNAPSHOT_ARMDB_PATH)?;
        let snapshot_assystem_path =
            env_string("SNAPSHOT_ASSYSTEM_PATH", DEFAULT_SNAPSHOT_ASSYSTEM_PATH)?;
        let snapshot_colon_db_path =
            env_string("SNAPSHOT_COLON_DB_PATH", DEFAULT_SNAPSHOT_COLON_DB_PATH)?;
        let snapshot_flash_kv_path =
            env_string("SNAPSHOT_FLASH_KV_PATH", DEFAULT_SNAPSHOT_FLASH_KV_PATH)?;
        let snapshot_ghaladb_path =
            env_string("SNAPSHOT_GHALADB_PATH", DEFAULT_SNAPSHOT_GHALADB_PATH)?;
        let snapshot_blockbucket_path = env_string(
            "SNAPSHOT_BLOCKBUCKET_PATH",
            DEFAULT_SNAPSHOT_BLOCKBUCKET_PATH,
        )?;
        let snapshot_grebedb_path =
            env_string("SNAPSHOT_GREBEDB_PATH", DEFAULT_SNAPSHOT_GREBEDB_PATH)?;
        let snapshot_grumpydb_path =
            env_string("SNAPSHOT_GRUMPYDB_PATH", DEFAULT_SNAPSHOT_GRUMPYDB_PATH)?;
        let snapshot_graus_db_path =
            env_string("SNAPSHOT_GRAUS_DB_PATH", DEFAULT_SNAPSHOT_GRAUS_DB_PATH)?;
        let snapshot_highlandcows_isam_path = env_string(
            "SNAPSHOT_HIGHLANDCOWS_ISAM_PATH",
            DEFAULT_SNAPSHOT_HIGHLANDCOWS_ISAM_PATH,
        )?;
        let snapshot_simple_db_path =
            env_string("SNAPSHOT_SIMPLE_DB_PATH", DEFAULT_SNAPSHOT_SIMPLE_DB_PATH)?;
        let snapshot_docdb_path = env_string("SNAPSHOT_DOCDB_PATH", DEFAULT_SNAPSHOT_DOCDB_PATH)?;
        let snapshot_emdb_path = env_string("SNAPSHOT_EMDB_PATH", DEFAULT_SNAPSHOT_EMDB_PATH)?;
        let snapshot_osmiumdb_path =
            env_string("SNAPSHOT_OSMIUMDB_PATH", DEFAULT_SNAPSHOT_OSMIUMDB_PATH)?;
        let snapshot_eight_path = env_string("SNAPSHOT_EIGHT_PATH", DEFAULT_SNAPSHOT_EIGHT_PATH)?;
        let snapshot_epoch_db_path =
            env_string("SNAPSHOT_EPOCH_DB_PATH", DEFAULT_SNAPSHOT_EPOCH_DB_PATH)?;
        let snapshot_etchdb_path =
            env_string("SNAPSHOT_ETCHDB_PATH", DEFAULT_SNAPSHOT_ETCHDB_PATH)?;
        let snapshot_fastkv_path =
            env_string("SNAPSHOT_FASTKV_PATH", DEFAULT_SNAPSHOT_FASTKV_PATH)?;
        let snapshot_ferrumdb_path =
            env_string("SNAPSHOT_FERRUMDB_PATH", DEFAULT_SNAPSHOT_FERRUMDB_PATH)?;
        let snapshot_rumdb_path = env_string("SNAPSHOT_RUMDB_PATH", DEFAULT_SNAPSHOT_RUMDB_PATH)?;
        let snapshot_rubin_path = env_string("SNAPSHOT_RUBIN_PATH", DEFAULT_SNAPSHOT_RUBIN_PATH)?;
        let snapshot_shorterdb_path =
            env_string("SNAPSHOT_SHORTERDB_PATH", DEFAULT_SNAPSHOT_SHORTERDB_PATH)?;
        let snapshot_sqlite_path =
            env_string("SNAPSHOT_SQLITE_PATH", DEFAULT_SNAPSHOT_SQLITE_PATH)?;
        let snapshot_heed_path = env_string("SNAPSHOT_HEED_PATH", DEFAULT_SNAPSHOT_HEED_PATH)?;
        let snapshot_hightower_kv_path = env_string(
            "SNAPSHOT_HIGHTOWER_KV_PATH",
            DEFAULT_SNAPSHOT_HIGHTOWER_KV_PATH,
        )?;
        let snapshot_hmdb_path = env_string("SNAPSHOT_HMDB_PATH", DEFAULT_SNAPSHOT_HMDB_PATH)?;
        let snapshot_hurrahdb_path =
            env_string("SNAPSHOT_HURRAHDB_PATH", DEFAULT_SNAPSHOT_HURRAHDB_PATH)?;
        let snapshot_fs_db_path = env_string("SNAPSHOT_FS_DB_PATH", DEFAULT_SNAPSHOT_FS_DB_PATH)?;
        let snapshot_sqjson_path =
            env_string("SNAPSHOT_SQJSON_PATH", DEFAULT_SNAPSHOT_SQJSON_PATH)?;
        let snapshot_icefalldb_path =
            env_string("SNAPSHOT_ICEFALLDB_PATH", DEFAULT_SNAPSHOT_ICEFALLDB_PATH)?;
        let snapshot_bitask_path =
            env_string("SNAPSHOT_BITASK_PATH", DEFAULT_SNAPSHOT_BITASK_PATH)?;
        let snapshot_bitkv_rs_path =
            env_string("SNAPSHOT_BITKV_RS_PATH", DEFAULT_SNAPSHOT_BITKV_RS_PATH)?;
        let snapshot_bitcask_engine_path = env_string(
            "SNAPSHOT_BITCASK_ENGINE_PATH",
            DEFAULT_SNAPSHOT_BITCASK_ENGINE_PATH,
        )?;
        let snapshot_blazeup_path =
            env_string("SNAPSHOT_BLAZEUP_PATH", DEFAULT_SNAPSHOT_BLAZEUP_PATH)?;
        let snapshot_candystore_path =
            env_string("SNAPSHOT_CANDYSTORE_PATH", DEFAULT_SNAPSHOT_CANDYSTORE_PATH)?;
        let snapshot_celerix_store_path = env_string(
            "SNAPSHOT_CELERIX_STORE_PATH",
            DEFAULT_SNAPSHOT_CELERIX_STORE_PATH,
        )?;
        let snapshot_citadeldb_path =
            env_string("SNAPSHOT_CITADELDB_PATH", DEFAULT_SNAPSHOT_CITADELDB_PATH)?;
        let snapshot_citadeldb_passphrase = env_string(
            "SNAPSHOT_CITADELDB_PASSPHRASE",
            DEFAULT_SNAPSHOT_CITADELDB_PASSPHRASE,
        )?;
        let snapshot_cuendillar_path =
            env_string("SNAPSHOT_CUENDILLAR_PATH", DEFAULT_SNAPSHOT_CUENDILLAR_PATH)?;
        let snapshot_data_pile_path =
            env_string("SNAPSHOT_DATA_PILE_PATH", DEFAULT_SNAPSHOT_DATA_PILE_PATH)?;
        let snapshot_datastack_path =
            env_string("SNAPSHOT_DATASTACK_PATH", DEFAULT_SNAPSHOT_DATASTACK_PATH)?;
        let snapshot_jammdb_path =
            env_string("SNAPSHOT_JAMMDB_PATH", DEFAULT_SNAPSHOT_JAMMDB_PATH)?;
        let snapshot_mace_path = env_string("SNAPSHOT_MACE_PATH", DEFAULT_SNAPSHOT_MACE_PATH)?;
        let snapshot_janql_path = env_string("SNAPSHOT_JANQL_PATH", DEFAULT_SNAPSHOT_JANQL_PATH)?;
        let snapshot_jasondb_path =
            env_string("SNAPSHOT_JASONDB_PATH", DEFAULT_SNAPSHOT_JASONDB_PATH)?;
        let snapshot_jasonisnthappy_path = env_string(
            "SNAPSHOT_JASONISNTHAPPY_PATH",
            DEFAULT_SNAPSHOT_JASONISNTHAPPY_PATH,
        )?;
        let snapshot_jfs_path = env_string("SNAPSHOT_JFS_PATH", DEFAULT_SNAPSHOT_JFS_PATH)?;
        let snapshot_json_store_path =
            env_string("SNAPSHOT_JSON_STORE_PATH", DEFAULT_SNAPSHOT_JSON_STORE_PATH)?;
        let snapshot_json_db_rs_path =
            env_string("SNAPSHOT_JSON_DB_RS_PATH", DEFAULT_SNAPSHOT_JSON_DB_RS_PATH)?;
        let snapshot_cdb64_path = env_string("SNAPSHOT_CDB64_PATH", DEFAULT_SNAPSHOT_CDB64_PATH)?;
        let snapshot_json_mutex_db_path = env_string(
            "SNAPSHOT_JSON_MUTEX_DB_PATH",
            DEFAULT_SNAPSHOT_JSON_MUTEX_DB_PATH,
        )?;
        let snapshot_toiletdb_path =
            env_string("SNAPSHOT_TOILETDB_PATH", DEFAULT_SNAPSHOT_TOILETDB_PATH)?;
        let snapshot_feoxdb_path =
            env_string("SNAPSHOT_FEOXDB_PATH", DEFAULT_SNAPSHOT_FEOXDB_PATH)?;
        let snapshot_jsondb_path =
            env_string("SNAPSHOT_JSONDB_PATH", DEFAULT_SNAPSHOT_JSONDB_PATH)?;
        let snapshot_kopperdb_path =
            env_string("SNAPSHOT_KOPPERDB_PATH", DEFAULT_SNAPSHOT_KOPPERDB_PATH)?;
        let snapshot_kv_path = env_string("SNAPSHOT_KV_PATH", DEFAULT_SNAPSHOT_KV_PATH)?;
        let snapshot_koit_path = env_string("SNAPSHOT_KOIT_PATH", DEFAULT_SNAPSHOT_KOIT_PATH)?;
        let snapshot_lite_db_path =
            env_string("SNAPSHOT_LITE_DB_PATH", DEFAULT_SNAPSHOT_LITE_DB_PATH)?;
        let snapshot_lmdb_rs_core_path = env_string(
            "SNAPSHOT_LMDB_RS_CORE_PATH",
            DEFAULT_SNAPSHOT_LMDB_RS_CORE_PATH,
        )?;
        let snapshot_log_kv_path =
            env_string("SNAPSHOT_LOG_KV_PATH", DEFAULT_SNAPSHOT_LOG_KV_PATH)?;
        let snapshot_append_kv_path =
            env_string("SNAPSHOT_APPEND_KV_PATH", DEFAULT_SNAPSHOT_APPEND_KV_PATH)?;
        let snapshot_mhdb_path = env_string("SNAPSHOT_MHDB_PATH", DEFAULT_SNAPSHOT_MHDB_PATH)?;
        let snapshot_marble_path =
            env_string("SNAPSHOT_MARBLE_PATH", DEFAULT_SNAPSHOT_MARBLE_PATH)?;
        let snapshot_loro_kv_path =
            env_string("SNAPSHOT_LORO_KV_PATH", DEFAULT_SNAPSHOT_LORO_KV_PATH)?;
        let snapshot_luckdb_path =
            env_string("SNAPSHOT_LUCKDB_PATH", DEFAULT_SNAPSHOT_LUCKDB_PATH)?;
        let snapshot_ipjdb_path = env_string("SNAPSHOT_IPJDB_PATH", DEFAULT_SNAPSHOT_IPJDB_PATH)?;
        let snapshot_kagi_path = env_string("SNAPSHOT_KAGI_PATH", DEFAULT_SNAPSHOT_KAGI_PATH)?;
        let snapshot_deeb_path = env_string("SNAPSHOT_DEEB_PATH", DEFAULT_SNAPSHOT_DEEB_PATH)?;
        let snapshot_lsm_engine_path =
            env_string("SNAPSHOT_LSM_ENGINE_PATH", DEFAULT_SNAPSHOT_LSM_ENGINE_PATH)?;
        let snapshot_lsm_storage_engine_path = env_string(
            "SNAPSHOT_LSM_STORAGE_ENGINE_PATH",
            DEFAULT_SNAPSHOT_LSM_STORAGE_ENGINE_PATH,
        )?;
        let snapshot_lsmdb_path = env_string("SNAPSHOT_LSMDB_PATH", DEFAULT_SNAPSHOT_LSMDB_PATH)?;
        let snapshot_lsm_tree_path =
            env_string("SNAPSHOT_LSM_TREE_PATH", DEFAULT_SNAPSHOT_LSM_TREE_PATH)?;
        let snapshot_mindb_path = env_string("SNAPSHOT_MINDB_PATH", DEFAULT_SNAPSHOT_MINDB_PATH)?;
        let snapshot_mmdb_path = env_string("SNAPSHOT_MMDB_PATH", DEFAULT_SNAPSHOT_MMDB_PATH)?;
        let snapshot_mu_db_path = env_string("SNAPSHOT_MU_DB_PATH", DEFAULT_SNAPSHOT_MU_DB_PATH)?;
        let snapshot_nanodb_path =
            env_string("SNAPSHOT_NANODB_PATH", DEFAULT_SNAPSHOT_NANODB_PATH)?;
        let snapshot_fjall_path = env_string("SNAPSHOT_FJALL_PATH", DEFAULT_SNAPSHOT_FJALL_PATH)?;
        let snapshot_persy_path = env_string("SNAPSHOT_PERSY_PATH", DEFAULT_SNAPSHOT_PERSY_PATH)?;
        let snapshot_persistent_kv_path = env_string(
            "SNAPSHOT_PERSISTENT_KV_PATH",
            DEFAULT_SNAPSHOT_PERSISTENT_KV_PATH,
        )?;
        let snapshot_native_db_path =
            env_string("SNAPSHOT_NATIVE_DB_PATH", DEFAULT_SNAPSHOT_NATIVE_DB_PATH)?;
        let snapshot_nebari_path =
            env_string("SNAPSHOT_NEBARI_PATH", DEFAULT_SNAPSHOT_NEBARI_PATH)?;
        let snapshot_nikidb_path =
            env_string("SNAPSHOT_NIKIDB_PATH", DEFAULT_SNAPSHOT_NIKIDB_PATH)?;
        let snapshot_nodb_path = env_string("SNAPSHOT_NODB_PATH", DEFAULT_SNAPSHOT_NODB_PATH)?;
        let snapshot_okofdb_path =
            env_string("SNAPSHOT_OKOFDB_PATH", DEFAULT_SNAPSHOT_OKOFDB_PATH)?;
        let snapshot_parity_db_path =
            env_string("SNAPSHOT_PARITY_DB_PATH", DEFAULT_SNAPSHOT_PARITY_DB_PATH)?;
        let snapshot_pickledb_path =
            env_string("SNAPSHOT_PICKLEDB_PATH", DEFAULT_SNAPSHOT_PICKLEDB_PATH)?;
        let snapshot_rcask_path = env_string("SNAPSHOT_RCASK_PATH", DEFAULT_SNAPSHOT_RCASK_PATH)?;
        let snapshot_microkv_path =
            env_string("SNAPSHOT_MICROKV_PATH", DEFAULT_SNAPSHOT_MICROKV_PATH)?;
        let snapshot_redb_path = env_string("SNAPSHOT_REDB_PATH", DEFAULT_SNAPSHOT_REDB_PATH)?;
        let snapshot_rskey_path = env_string("SNAPSHOT_RSKEY_PATH", DEFAULT_SNAPSHOT_RSKEY_PATH)?;
        let snapshot_readb_path = env_string("SNAPSHOT_READB_PATH", DEFAULT_SNAPSHOT_READB_PATH)?;
        let snapshot_rustlite_path =
            env_string("SNAPSHOT_RUSTLITE_PATH", DEFAULT_SNAPSHOT_RUSTLITE_PATH)?;
        let snapshot_rustcask_path =
            env_string("SNAPSHOT_RUSTCASK_PATH", DEFAULT_SNAPSHOT_RUSTCASK_PATH)?;
        let snapshot_rusty_leveldb_path = env_string(
            "SNAPSHOT_RUSTY_LEVELDB_PATH",
            DEFAULT_SNAPSHOT_RUSTY_LEVELDB_PATH,
        )?;
        let snapshot_canopydb_path =
            env_string("SNAPSHOT_CANOPYDB_PATH", DEFAULT_SNAPSHOT_CANOPYDB_PATH)?;
        let snapshot_caves_path = env_string("SNAPSHOT_CAVES_PATH", DEFAULT_SNAPSHOT_CAVES_PATH)?;
        let snapshot_ckydb_path = env_string("SNAPSHOT_CKYDB_PATH", DEFAULT_SNAPSHOT_CKYDB_PATH)?;
        let snapshot_crepedb_path =
            env_string("SNAPSHOT_CREPEDB_PATH", DEFAULT_SNAPSHOT_CREPEDB_PATH)?;
        let snapshot_crystal_path =
            env_string("SNAPSHOT_CRYSTAL_PATH", DEFAULT_SNAPSHOT_CRYSTAL_PATH)?;
        let snapshot_scdb_path = env_string("SNAPSHOT_SCDB_PATH", DEFAULT_SNAPSHOT_SCDB_PATH)?;
        let snapshot_skv_path = env_string("SNAPSHOT_SKV_PATH", DEFAULT_SNAPSHOT_SKV_PATH)?;
        let snapshot_surrealkv_path =
            env_string("SNAPSHOT_SURREALKV_PATH", DEFAULT_SNAPSHOT_SURREALKV_PATH)?;
        let snapshot_sled_path = env_string("SNAPSHOT_SLED_PATH", DEFAULT_SNAPSHOT_SLED_PATH)?;
        let snapshot_rustbreak_path =
            env_string("SNAPSHOT_RUSTBREAK_PATH", DEFAULT_SNAPSHOT_RUSTBREAK_PATH)?;
        let snapshot_yedb_path = env_string("SNAPSHOT_YEDB_PATH", DEFAULT_SNAPSHOT_YEDB_PATH)?;
        let snapshot_btree_store_path = env_string(
            "SNAPSHOT_BTREE_STORE_PATH",
            DEFAULT_SNAPSHOT_BTREE_STORE_PATH,
        )?;
        let snapshot_cacache_path =
            env_string("SNAPSHOT_CACACHE_PATH", DEFAULT_SNAPSHOT_CACACHE_PATH)?;
        let snapshot_siamesedb_path =
            env_string("SNAPSHOT_SIAMESDB_PATH", DEFAULT_SNAPSHOT_SIAMESDB_PATH)?;
        let snapshot_structsy_path =
            env_string("SNAPSHOT_STRUCTSY_PATH", DEFAULT_SNAPSHOT_STRUCTSY_PATH)?;
        let snapshot_abyssiniandb_path = env_string(
            "SNAPSHOT_ABYSSINIANDB_PATH",
            DEFAULT_SNAPSHOT_ABYSSINIANDB_PATH,
        )?;
        let snapshot_aeternusdb_path =
            env_string("SNAPSHOT_AETERNUSDB_PATH", DEFAULT_SNAPSHOT_AETERNUSDB_PATH)?;
        let snapshot_thunderdb_path =
            env_string("SNAPSHOT_THUNDERDB_PATH", DEFAULT_SNAPSHOT_THUNDERDB_PATH)?;
        let snapshot_thetadb_path =
            env_string("SNAPSHOT_THETADB_PATH", DEFAULT_SNAPSHOT_THETADB_PATH)?;
        let snapshot_tinybase_path =
            env_string("SNAPSHOT_TINYBASE_PATH", DEFAULT_SNAPSHOT_TINYBASE_PATH)?;
        let snapshot_tinydb_path =
            env_string("SNAPSHOT_TINYDB_PATH", DEFAULT_SNAPSHOT_TINYDB_PATH)?;
        let snapshot_dblite_path =
            env_string("SNAPSHOT_DBLITE_PATH", DEFAULT_SNAPSHOT_DBLITE_PATH)?;
        let snapshot_dbless_path =
            env_string("SNAPSHOT_DBLESS_PATH", DEFAULT_SNAPSHOT_DBLESS_PATH)?;
        let snapshot_db_rs_path = env_string("SNAPSHOT_DB_RS_PATH", DEFAULT_SNAPSHOT_DB_RS_PATH)?;
        let snapshot_dharmadb_path =
            env_string("SNAPSHOT_DHARMADB_PATH", DEFAULT_SNAPSHOT_DHARMADB_PATH)?;
        let snapshot_dir_cache_path =
            env_string("SNAPSHOT_DIR_CACHE_PATH", DEFAULT_SNAPSHOT_DIR_CACHE_PATH)?;
        let snapshot_sanakirja_path =
            env_string("SNAPSHOT_SANAKIRJA_PATH", DEFAULT_SNAPSHOT_SANAKIRJA_PATH)?;
        let snapshot_saturn_path =
            env_string("SNAPSHOT_SATURN_PATH", DEFAULT_SNAPSHOT_SATURN_PATH)?;
        let snapshot_snaildb_path =
            env_string("SNAPSHOT_SNAILDB_PATH", DEFAULT_SNAPSHOT_SNAILDB_PATH)?;
        let snapshot_tinykv_path =
            env_string("SNAPSHOT_TINYKV_PATH", DEFAULT_SNAPSHOT_TINYKV_PATH)?;
        let snapshot_vsdb_path = env_string("SNAPSHOT_VSDB_PATH", DEFAULT_SNAPSHOT_VSDB_PATH)?;
        let snapshot_yakv_path = env_string("SNAPSHOT_YAKV_PATH", DEFAULT_SNAPSHOT_YAKV_PATH)?;
        let snapshot_yakvdb_path =
            env_string("SNAPSHOT_YAKVDB_PATH", DEFAULT_SNAPSHOT_YAKVDB_PATH)?;
        let snapshot_saberdb_path =
            env_string("SNAPSHOT_SABERDB_PATH", DEFAULT_SNAPSHOT_SABERDB_PATH)?;
        let snapshot_smolldb_path =
            env_string("SNAPSHOT_SMOLLDB_PATH", DEFAULT_SNAPSHOT_SMOLLDB_PATH)?;
        let snapshot_kstone_path =
            env_string("SNAPSHOT_KSTONE_PATH", DEFAULT_SNAPSHOT_KSTONE_PATH)?;
        let snapshot_roughdb_path =
            env_string("SNAPSHOT_ROUGHDB_PATH", DEFAULT_SNAPSHOT_ROUGHDB_PATH)?;
        let snapshot_raindb_path =
            env_string("SNAPSHOT_RAINDB_PATH", DEFAULT_SNAPSHOT_RAINDB_PATH)?;
        let snapshot_infusedb_path =
            env_string("SNAPSHOT_INFUSEDB_PATH", DEFAULT_SNAPSHOT_INFUSEDB_PATH)?;
        let snapshot_kafi_path = env_string("SNAPSHOT_KAFI_PATH", DEFAULT_SNAPSHOT_KAFI_PATH)?;
        let snapshot_tinkv_path = env_string("SNAPSHOT_TINKV_PATH", DEFAULT_SNAPSHOT_TINKV_PATH)?;
        let snapshot_ledger_kv_path =
            env_string("SNAPSHOT_LEDGER_KV_PATH", DEFAULT_SNAPSHOT_LEDGER_KV_PATH)?;
        let snapshot_joydb_path = env_string("SNAPSHOT_JOYDB_PATH", DEFAULT_SNAPSHOT_JOYDB_PATH)?;
        let snapshot_png_db_path =
            env_string("SNAPSHOT_PNG_DB_PATH", DEFAULT_SNAPSHOT_PNG_DB_PATH)?;
        let snapshot_s3_endpoint = env_optional_http_base_url("SNAPSHOT_S3_ENDPOINT")?;
        let snapshot_s3_region = env_string("SNAPSHOT_S3_REGION", DEFAULT_SNAPSHOT_S3_REGION)?;
        let snapshot_s3_bucket = env_optional_string("SNAPSHOT_S3_BUCKET")?;
        let snapshot_s3_prefix = env_string("SNAPSHOT_S3_PREFIX", DEFAULT_SNAPSHOT_S3_PREFIX)?;
        let snapshot_s3_access_key_id = env_optional_string("SNAPSHOT_S3_ACCESS_KEY_ID")?;
        let snapshot_s3_secret_access_key = env_optional_string("SNAPSHOT_S3_SECRET_ACCESS_KEY")?;
        let snapshot_s3_session_token = env_optional_string("SNAPSHOT_S3_SESSION_TOKEN")?;
        let snapshot_s3_timeout_secs =
            env_u64("SNAPSHOT_S3_TIMEOUT_SECS", DEFAULT_SNAPSHOT_S3_TIMEOUT_SECS)?;
        let snapshot_s3_path_style =
            env_bool("SNAPSHOT_S3_PATH_STYLE", DEFAULT_SNAPSHOT_S3_PATH_STYLE)?;
        let snapshot_managed_base_url = env_optional_http_base_url("SNAPSHOT_MANAGED_BASE_URL")?;
        let snapshot_managed_auth_token = env_optional_string("SNAPSHOT_MANAGED_AUTH_TOKEN")?;
        let snapshot_managed_timeout_secs = env_u64(
            "SNAPSHOT_MANAGED_TIMEOUT_SECS",
            DEFAULT_SNAPSHOT_MANAGED_TIMEOUT_SECS,
        )?;
        let room_locator = env_string("ROOM_LOCATOR", DEFAULT_ROOM_LOCATOR)?;
        let room_coordinator = env_string("ROOM_COORDINATOR", DEFAULT_ROOM_COORDINATOR)?;
        let room_coordinator_state_dir = env_string(
            "ROOM_COORDINATOR_STATE_DIR",
            DEFAULT_ROOM_COORDINATOR_STATE_DIR,
        )?;
        let room_coordinator_sqlite_path = env_string(
            "ROOM_COORDINATOR_SQLITE_PATH",
            DEFAULT_ROOM_COORDINATOR_SQLITE_PATH,
        )?;
        let room_coordinator_heartbeat_interval_secs = env_u64(
            "ROOM_COORDINATOR_HEARTBEAT_INTERVAL_SECS",
            DEFAULT_ROOM_COORDINATOR_HEARTBEAT_INTERVAL_SECS,
        )?;
        let room_coordinator_lease_ttl_secs = env_u64(
            "ROOM_COORDINATOR_LEASE_TTL_SECS",
            DEFAULT_ROOM_COORDINATOR_LEASE_TTL_SECS,
        )?;
        let room_coordination_managed_base_url =
            env_optional_http_base_url("ROOM_COORDINATION_MANAGED_BASE_URL")?;
        let room_coordination_managed_auth_token =
            env_optional_string("ROOM_COORDINATION_MANAGED_AUTH_TOKEN")?;
        let room_coordination_managed_timeout_secs = env_u64(
            "ROOM_COORDINATION_MANAGED_TIMEOUT_SECS",
            DEFAULT_ROOM_COORDINATION_MANAGED_TIMEOUT_SECS,
        )?;
        let node_id = env_string("NODE_ID", DEFAULT_NODE_ID)?;
        let node_base_url = env_optional_origin("NODE_BASE_URL")?;
        let room_owner_hints_path = env_optional_string("ROOM_OWNER_HINTS_PATH")?;

        Ok(Self {
            host,
            port,
            frontend_origin,
            rust_log,
            api_token,
            snapshot_store,
            snapshot_dir,
            snapshot_agdb_path,
            snapshot_amandine_path,
            snapshot_append_log_path,
            snapshot_apex_store_path,
            snapshot_armdb_path,
            snapshot_assystem_path,
            snapshot_colon_db_path,
            snapshot_flash_kv_path,
            snapshot_ghaladb_path,
            snapshot_blockbucket_path,
            snapshot_grebedb_path,
            snapshot_grumpydb_path,
            snapshot_graus_db_path,
            snapshot_highlandcows_isam_path,
            snapshot_simple_db_path,
            snapshot_docdb_path,
            snapshot_emdb_path,
            snapshot_osmiumdb_path,
            snapshot_eight_path,
            snapshot_epoch_db_path,
            snapshot_etchdb_path,
            snapshot_fastkv_path,
            snapshot_ferrumdb_path,
            snapshot_rumdb_path,
            snapshot_rubin_path,
            snapshot_shorterdb_path,
            snapshot_sqlite_path,
            snapshot_heed_path,
            snapshot_hightower_kv_path,
            snapshot_hmdb_path,
            snapshot_hurrahdb_path,
            snapshot_fs_db_path,
            snapshot_sqjson_path,
            snapshot_icefalldb_path,
            snapshot_bitask_path,
            snapshot_bitkv_rs_path,
            snapshot_bitcask_engine_path,
            snapshot_blazeup_path,
            snapshot_candystore_path,
            snapshot_celerix_store_path,
            snapshot_citadeldb_path,
            snapshot_citadeldb_passphrase,
            snapshot_cuendillar_path,
            snapshot_data_pile_path,
            snapshot_datastack_path,
            snapshot_jammdb_path,
            snapshot_mace_path,
            snapshot_janql_path,
            snapshot_jasondb_path,
            snapshot_jasonisnthappy_path,
            snapshot_jfs_path,
            snapshot_json_store_path,
            snapshot_json_db_rs_path,
            snapshot_cdb64_path,
            snapshot_json_mutex_db_path,
            snapshot_toiletdb_path,
            snapshot_feoxdb_path,
            snapshot_jsondb_path,
            snapshot_kopperdb_path,
            snapshot_kv_path,
            snapshot_koit_path,
            snapshot_lite_db_path,
            snapshot_lmdb_rs_core_path,
            snapshot_log_kv_path,
            snapshot_append_kv_path,
            snapshot_mhdb_path,
            snapshot_marble_path,
            snapshot_loro_kv_path,
            snapshot_luckdb_path,
            snapshot_ipjdb_path,
            snapshot_kagi_path,
            snapshot_deeb_path,
            snapshot_lsm_engine_path,
            snapshot_lsm_storage_engine_path,
            snapshot_lsmdb_path,
            snapshot_lsm_tree_path,
            snapshot_mindb_path,
            snapshot_mmdb_path,
            snapshot_mu_db_path,
            snapshot_nanodb_path,
            snapshot_fjall_path,
            snapshot_persy_path,
            snapshot_persistent_kv_path,
            snapshot_native_db_path,
            snapshot_nebari_path,
            snapshot_nikidb_path,
            snapshot_nodb_path,
            snapshot_okofdb_path,
            snapshot_parity_db_path,
            snapshot_pickledb_path,
            snapshot_rcask_path,
            snapshot_microkv_path,
            snapshot_redb_path,
            snapshot_rskey_path,
            snapshot_readb_path,
            snapshot_rustlite_path,
            snapshot_rustcask_path,
            snapshot_rusty_leveldb_path,
            snapshot_canopydb_path,
            snapshot_caves_path,
            snapshot_ckydb_path,
            snapshot_crepedb_path,
            snapshot_crystal_path,
            snapshot_scdb_path,
            snapshot_skv_path,
            snapshot_surrealkv_path,
            snapshot_sled_path,
            snapshot_rustbreak_path,
            snapshot_yedb_path,
            snapshot_btree_store_path,
            snapshot_cacache_path,
            snapshot_siamesedb_path,
            snapshot_structsy_path,
            snapshot_abyssiniandb_path,
            snapshot_aeternusdb_path,
            snapshot_thunderdb_path,
            snapshot_thetadb_path,
            snapshot_tinybase_path,
            snapshot_tinydb_path,
            snapshot_dblite_path,
            snapshot_dbless_path,
            snapshot_db_rs_path,
            snapshot_dharmadb_path,
            snapshot_dir_cache_path,
            snapshot_sanakirja_path,
            snapshot_saturn_path,
            snapshot_snaildb_path,
            snapshot_tinykv_path,
            snapshot_vsdb_path,
            snapshot_yakv_path,
            snapshot_yakvdb_path,
            snapshot_saberdb_path,
            snapshot_smolldb_path,
            snapshot_kstone_path,
            snapshot_roughdb_path,
            snapshot_raindb_path,
            snapshot_infusedb_path,
            snapshot_kafi_path,
            snapshot_tinkv_path,
            snapshot_ledger_kv_path,
            snapshot_joydb_path,
            snapshot_png_db_path,
            snapshot_s3_endpoint,
            snapshot_s3_region,
            snapshot_s3_bucket,
            snapshot_s3_prefix,
            snapshot_s3_access_key_id,
            snapshot_s3_secret_access_key,
            snapshot_s3_session_token,
            snapshot_s3_timeout_secs,
            snapshot_s3_path_style,
            snapshot_managed_base_url,
            snapshot_managed_auth_token,
            snapshot_managed_timeout_secs,
            room_locator,
            room_coordinator,
            room_coordinator_state_dir,
            room_coordinator_sqlite_path,
            room_coordinator_heartbeat_interval_secs,
            room_coordinator_lease_ttl_secs,
            room_coordination_managed_base_url,
            room_coordination_managed_auth_token,
            room_coordination_managed_timeout_secs,
            node_id,
            node_base_url,
            room_owner_hints_path,
        })
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn frontend_origin_header(&self) -> AppResult<HeaderValue> {
        HeaderValue::from_str(&self.frontend_origin).map_err(|_| {
            AppError::Config(format!(
                "FRONTEND_ORIGIN must be a valid header-safe origin, received `{}`",
                self.frontend_origin
            ))
        })
    }
}

fn env_string(key: &str, default: &str) -> AppResult<String> {
    match env::var(key) {
        Ok(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                Err(AppError::Config(format!("{key} cannot be empty")))
            } else {
                Ok(trimmed.to_owned())
            }
        }
        Err(env::VarError::NotPresent) => Ok(default.to_owned()),
        Err(env::VarError::NotUnicode(_)) => {
            Err(AppError::Config(format!("{key} must be valid unicode")))
        }
    }
}

fn env_u16(key: &str, default: u16) -> AppResult<u16> {
    match env::var(key) {
        Ok(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Err(AppError::Config(format!("{key} cannot be empty")));
            }

            trimmed.parse::<u16>().map_err(|_| {
                AppError::Config(format!(
                    "{key} must be an unsigned 16-bit integer, received `{trimmed}`"
                ))
            })
        }
        Err(env::VarError::NotPresent) => Ok(default),
        Err(env::VarError::NotUnicode(_)) => {
            Err(AppError::Config(format!("{key} must be valid unicode")))
        }
    }
}

fn env_u64(key: &str, default: u64) -> AppResult<u64> {
    match env::var(key) {
        Ok(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Err(AppError::Config(format!("{key} cannot be empty")));
            }

            trimmed.parse::<u64>().map_err(|_| {
                AppError::Config(format!(
                    "{key} must be an unsigned 64-bit integer, received `{trimmed}`"
                ))
            })
        }
        Err(env::VarError::NotPresent) => Ok(default),
        Err(env::VarError::NotUnicode(_)) => {
            Err(AppError::Config(format!("{key} must be valid unicode")))
        }
    }
}

fn env_bool(key: &str, default: bool) -> AppResult<bool> {
    match env::var(key) {
        Ok(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Err(AppError::Config(format!("{key} cannot be empty")));
            }

            match trimmed.to_ascii_lowercase().as_str() {
                "true" | "1" | "yes" | "on" => Ok(true),
                "false" | "0" | "no" | "off" => Ok(false),
                _ => Err(AppError::Config(format!(
                    "{key} must be a boolean (`true`/`false`), received `{trimmed}`"
                ))),
            }
        }
        Err(env::VarError::NotPresent) => Ok(default),
        Err(env::VarError::NotUnicode(_)) => {
            Err(AppError::Config(format!("{key} must be valid unicode")))
        }
    }
}

fn env_optional_string(key: &str) -> AppResult<Option<String>> {
    match env::var(key) {
        Ok(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                Err(AppError::Config(format!("{key} cannot be empty")))
            } else {
                Ok(Some(trimmed.to_owned()))
            }
        }
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => {
            Err(AppError::Config(format!("{key} must be valid unicode")))
        }
    }
}

fn env_optional_origin(key: &str) -> AppResult<Option<String>> {
    match env::var(key) {
        Ok(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                Err(AppError::Config(format!("{key} cannot be empty")))
            } else {
                normalize_origin_url(trimmed, key)
                    .map(Some)
                    .map_err(AppError::Config)
            }
        }
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => {
            Err(AppError::Config(format!("{key} must be valid unicode")))
        }
    }
}

fn env_optional_http_base_url(key: &str) -> AppResult<Option<String>> {
    match env::var(key) {
        Ok(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                Err(AppError::Config(format!("{key} cannot be empty")))
            } else {
                normalize_http_base_url(trimmed, key)
                    .map(Some)
                    .map_err(AppError::Config)
            }
        }
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => {
            Err(AppError::Config(format!("{key} must be valid unicode")))
        }
    }
}

pub fn normalize_origin_url(value: &str, field_name: &str) -> Result<String, String> {
    let invalid_message = || {
        format!(
            "{field_name} must be an origin-only absolute http/https URL without path/query, received `{value}`"
        )
    };

    let uri: Uri = value.parse().map_err(|_| invalid_message())?;
    let Some(scheme) = uri.scheme_str() else {
        return Err(invalid_message());
    };
    let Some(authority) = uri.authority() else {
        return Err(invalid_message());
    };

    if !matches!(scheme, "http" | "https") || uri.query().is_some() {
        return Err(invalid_message());
    }

    if !matches!(uri.path(), "" | "/") {
        return Err(invalid_message());
    }

    Ok(format!("{scheme}://{authority}"))
}

pub fn normalize_http_base_url(value: &str, field_name: &str) -> Result<String, String> {
    let invalid_message = || {
        format!("{field_name} must be an absolute http/https URL without query, received `{value}`")
    };

    let uri: Uri = value.parse().map_err(|_| invalid_message())?;
    let Some(scheme) = uri.scheme_str() else {
        return Err(invalid_message());
    };
    let Some(authority) = uri.authority() else {
        return Err(invalid_message());
    };

    if !matches!(scheme, "http" | "https") || uri.query().is_some() {
        return Err(invalid_message());
    }

    let normalized_path = match uri.path() {
        "" | "/" => String::new(),
        path => path.trim_end_matches('/').to_owned(),
    };

    Ok(format!("{scheme}://{authority}{normalized_path}"))
}
