use axum::{
    Json, Router,
    body::Bytes,
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, Method, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use axum_test::{TestServer, WsMessage};
use backend::{
    app::build_app,
    collab::{
        coordinator::{RoomCoordinator, RoomCoordinatorError},
        locator::{ResolvedRoom, RoomLocator, RoomLocatorError, RoomOwnerHint},
        rooms::RoomRegistry,
    },
    config::Config,
    errors::AppError,
    state::AppState,
    storage::{
        AbyssiniandbSnapshotStore, AeternusdbSnapshotStore, AgdbSnapshotStore,
        AmandineSnapshotStore, ApexStoreSnapshotStore, AppendKvSnapshotStore, ArmdbSnapshotStore,
        AssystemSnapshotStore, BitaskSnapshotStore, BitcaskEngineSnapshotStore,
        BitkvRsSnapshotStore, BlazeupSnapshotStore, BlockbucketSnapshotStore,
        BtreeStoreSnapshotStore, CandystoreSnapshotStore, CanopydbSnapshotStore,
        CavesSnapshotStore, CelerixStoreSnapshotStore, CitadeldbSnapshotStore, CkydbSnapshotStore,
        ColonDbSnapshotStore, CrepeDbSnapshotStore, CrystalSnapshotStore, CuendillarSnapshotStore,
        DatastackSnapshotStore, DbRsSnapshotStore, DblessSnapshotStore, DbliteSnapshotStore,
        DharmadbSnapshotStore, DocDbSnapshotStore, DocumentSnapshot, EightSnapshotStore,
        EpochDbSnapshotStore, EtchdbSnapshotStore, FeoxdbSnapshotStore, FerrumdbSnapshotStore,
        FileSnapshotStore, FjallSnapshotStore, FlashKvSnapshotStore, GhaladbSnapshotStore,
        GrausDbSnapshotStore, GrebedbSnapshotStore, GrumpydbSnapshotStore, HeedSnapshotStore,
        HighlandcowsIsamSnapshotStore, HightowerKvSnapshotStore, HmdbSnapshotStore,
        IcefalldbSnapshotStore, InMemorySnapshotStore, InfusedbSnapshotStore, IpjdbSnapshotStore,
        JammdbSnapshotStore, JanqlSnapshotStore, JasondbSnapshotStore, JasonisnthappySnapshotStore,
        JfsSnapshotStore, JoydbSnapshotStore, JsonStoreSnapshotStore, JsondbSnapshotStore,
        KafiSnapshotStore, KoitSnapshotStore, KopperdbSnapshotStore, KstoneSnapshotStore,
        KvSnapshotStore, LedgerKvSnapshotStore, LiteDbSnapshotStore, LogKvSnapshotStore,
        LoroKvSnapshotStore, LsmEngineSnapshotStore, LsmStorageEngineSnapshotStore,
        LsmTreeSnapshotStore, LsmdbSnapshotStore, LuckdbSnapshotStore, MaceSnapshotStore,
        ManagedSnapshotStore, MhdbSnapshotStore, MicroKvSnapshotStore, MindbSnapshotStore,
        MmdbSnapshotStore, NanodbSnapshotStore, NativeDbSnapshotStore, NebariSnapshotStore,
        NikidbSnapshotStore, NodbSnapshotStore, OkofdbSnapshotStore, ParityDbSnapshotStore,
        PersistentKvSnapshotStore, PersySnapshotStore, PickleDbSnapshotStore, RaindbSnapshotStore,
        RcaskSnapshotStore, ReadbSnapshotStore, RedbSnapshotStore, RoughdbSnapshotStore,
        RskeySnapshotStore, RubinSnapshotStore, RumDbSnapshotStore, RustbreakSnapshotStore,
        RustcaskSnapshotStore, RustliteSnapshotStore, RustyLeveldbSnapshotStore, S3SnapshotStore,
        SaberdbSnapshotStore, SanakirjaSnapshotStore, ScdbSnapshotStore, ShorterDbSnapshotStore,
        SiamesedbSnapshotStore, SimpleDbSnapshotStore, SkvSnapshotStore, SledSnapshotStore,
        SmolldbSnapshotStore, SnaildbSnapshotStore, SnapshotStore, SqliteSnapshotStore,
        StructsySnapshotStore, SurrealkvSnapshotStore, ThetadbSnapshotStore,
        ThunderdbSnapshotStore, TinkvSnapshotStore, TinybaseSnapshotStore, TinydbSnapshotStore,
        TinykvSnapshotStore, VsdbSnapshotStore, YakvSnapshotStore, YakvdbSnapshotStore,
        YedbSnapshotStore,
    },
};
use chrono::{Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    fs,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{net::TcpListener, task::JoinHandle};
use uuid::Uuid;
use yrs::{
    Doc, GetString, ReadTxn, StateVector, Text, Transact, Update,
    sync::{AwarenessUpdate, Message, SyncMessage, awareness::AwarenessUpdateEntry},
    updates::{decoder::Decode, encoder::Encode},
};

fn test_config() -> Config {
    Config {
        host: "127.0.0.1".to_owned(),
        port: 4000,
        frontend_origin: "http://localhost:3000".to_owned(),
        rust_log: "backend=debug".to_owned(),
        api_token: "test-admin-token".to_owned(),
        snapshot_store: "memory".to_owned(),
        snapshot_dir: "./data/test-snapshots".to_owned(),
        snapshot_agdb_path: "./data/test-snapshots.agdb".to_owned(),
        snapshot_amandine_path: "./data/test-snapshots.amandine".to_owned(),
        snapshot_apex_store_path: "./data/test-snapshots.apex_store".to_owned(),
        snapshot_armdb_path: "./data/test-snapshots.armdb".to_owned(),
        snapshot_assystem_path: "./data/test-snapshots.assystem".to_owned(),
        snapshot_colon_db_path: "./data/test-snapshots.colon_db".to_owned(),
        snapshot_flash_kv_path: "./data/test-snapshots.flash_kv".to_owned(),
        snapshot_ghaladb_path: "./data/test-snapshots.ghaladb".to_owned(),
        snapshot_blockbucket_path: "./data/test-snapshots.blockbucket".to_owned(),
        snapshot_grebedb_path: "./data/test-snapshots.grebedb".to_owned(),
        snapshot_grumpydb_path: "./data/test-snapshots.grumpydb".to_owned(),
        snapshot_highlandcows_isam_path: "./data/test-snapshots.highlandcows_isam".to_owned(),
        snapshot_simple_db_path: "./data/test-snapshots.simple_db".to_owned(),
        snapshot_docdb_path: "./data/test-snapshots.docdb.json".to_owned(),
        snapshot_eight_path: "./data/test-snapshots.eight".to_owned(),
        snapshot_epoch_db_path: "./data/test-snapshots.epoch_db".to_owned(),
        snapshot_etchdb_path: "./data/test-snapshots.etchdb".to_owned(),
        snapshot_ferrumdb_path: "./data/test-snapshots.ferrumdb".to_owned(),
        snapshot_rumdb_path: "./data/test-snapshots.rumdb".to_owned(),
        snapshot_rubin_path: "./data/test-snapshots.rubin.json".to_owned(),
        snapshot_shorterdb_path: "./data/test-snapshots.shorterdb".to_owned(),
        snapshot_sqlite_path: "./data/test-snapshots.sqlite3".to_owned(),
        snapshot_heed_path: "./data/test-snapshots.heed".to_owned(),
        snapshot_hightower_kv_path: "./data/test-snapshots.hightower_kv".to_owned(),
        snapshot_hmdb_path: "./data/test-snapshots.hmdb".to_owned(),
        snapshot_icefalldb_path: "./data/test-snapshots.icefalldb".to_owned(),
        snapshot_bitask_path: "./data/test-snapshots.bitask".to_owned(),
        snapshot_bitkv_rs_path: "./data/test-snapshots.bitkv_rs".to_owned(),
        snapshot_bitcask_engine_path: "./data/test-snapshots.bitcask_engine".to_owned(),
        snapshot_blazeup_path: "./data/test-snapshots.blazeup".to_owned(),
        snapshot_candystore_path: "./data/test-snapshots.candystore".to_owned(),
        snapshot_celerix_store_path: "./data/test-snapshots.celerix_store".to_owned(),
        snapshot_citadeldb_path: "./data/test-snapshots.citadeldb".to_owned(),
        snapshot_citadeldb_passphrase: "test-citadel-snapshot-passphrase".to_owned(),
        snapshot_cuendillar_path: "./data/test-snapshots.cuendillar".to_owned(),
        snapshot_datastack_path: "./data/test-snapshots.datastack".to_owned(),
        snapshot_jammdb_path: "./data/test-snapshots.jammdb".to_owned(),
        snapshot_mace_path: "./data/test-snapshots.mace".to_owned(),
        snapshot_janql_path: "./data/test-snapshots.janql".to_owned(),
        snapshot_jasondb_path: "./data/test-snapshots.jasondb".to_owned(),
        snapshot_jasonisnthappy_path: "./data/test-snapshots.jasonisnthappy".to_owned(),
        snapshot_jfs_path: "./data/test-snapshots.jfs.json".to_owned(),
        snapshot_json_store_path: "./data/test-snapshots.json_store.jsonl".to_owned(),
        snapshot_feoxdb_path: "./data/test-snapshots.feoxdb".to_owned(),
        snapshot_jsondb_path: "./data/test-snapshots.jsondb.json".to_owned(),
        snapshot_kopperdb_path: "./data/test-snapshots.kopperdb".to_owned(),
        snapshot_kv_path: "./data/test-snapshots.kv".to_owned(),
        snapshot_koit_path: "./data/test-snapshots.koit.json".to_owned(),
        snapshot_lite_db_path: "./data/test-snapshots.lite_db".to_owned(),
        snapshot_log_kv_path: "./data/test-snapshots.log_kv".to_owned(),
        snapshot_append_kv_path: "./data/test-snapshots.append_kv".to_owned(),
        snapshot_mhdb_path: "./data/test-snapshots.mhdb".to_owned(),
        snapshot_loro_kv_path: "./data/test-snapshots.loro_kv".to_owned(),
        snapshot_luckdb_path: "./data/test-snapshots.luckdb.json".to_owned(),
        snapshot_ipjdb_path: "./data/test-snapshots.ipjdb".to_owned(),
        snapshot_lsm_engine_path: "./data/test-snapshots.lsm_engine".to_owned(),
        snapshot_lsm_storage_engine_path: "./data/test-snapshots.lsm_storage_engine".to_owned(),
        snapshot_lsmdb_path: "./data/test-snapshots.lsmdb".to_owned(),
        snapshot_lsm_tree_path: "./data/test-snapshots.lsm_tree".to_owned(),
        snapshot_mindb_path: "./data/test-snapshots.mindb".to_owned(),
        snapshot_mmdb_path: "./data/test-snapshots.mmdb".to_owned(),
        snapshot_nanodb_path: "./data/test-snapshots.nanodb.json".to_owned(),
        snapshot_graus_db_path: "./data/test-snapshots.graus_db".to_owned(),
        snapshot_fjall_path: "./data/test-snapshots.fjall".to_owned(),
        snapshot_persy_path: "./data/test-snapshots.persy".to_owned(),
        snapshot_persistent_kv_path: "./data/test-snapshots.persistent_kv".to_owned(),
        snapshot_native_db_path: "./data/test-snapshots.native_db".to_owned(),
        snapshot_nebari_path: "./data/test-snapshots.nebari".to_owned(),
        snapshot_nikidb_path: "./data/test-snapshots.nikidb".to_owned(),
        snapshot_nodb_path: "./data/test-snapshots.nodb".to_owned(),
        snapshot_okofdb_path: "./data/test-snapshots.okofdb".to_owned(),
        snapshot_parity_db_path: "./data/test-snapshots.parity_db".to_owned(),
        snapshot_pickledb_path: "./data/test-snapshots.pickledb".to_owned(),
        snapshot_rcask_path: "./data/test-snapshots.rcask".to_owned(),
        snapshot_microkv_path: "./data/test-snapshots_microkv".to_owned(),
        snapshot_redb_path: "./data/test-snapshots.redb".to_owned(),
        snapshot_rskey_path: "./data/test-snapshots.rskey".to_owned(),
        snapshot_readb_path: "./data/test-snapshots.readb".to_owned(),
        snapshot_rustlite_path: "./data/test-snapshots.rustlite".to_owned(),
        snapshot_rustcask_path: "./data/test-snapshots.rustcask".to_owned(),
        snapshot_rusty_leveldb_path: "./data/test-snapshots.rusty_leveldb".to_owned(),
        snapshot_canopydb_path: "./data/test-snapshots.canopydb".to_owned(),
        snapshot_caves_path: "./data/test-snapshots.caves".to_owned(),
        snapshot_ckydb_path: "./data/test-snapshots.ckydb".to_owned(),
        snapshot_crepedb_path: "./data/test-snapshots.crepedb".to_owned(),
        snapshot_crystal_path: "./data/test-snapshots.crystal".to_owned(),
        snapshot_scdb_path: "./data/test-snapshots.scdb".to_owned(),
        snapshot_skv_path: "./data/test-snapshots.skv".to_owned(),
        snapshot_surrealkv_path: "./data/test-snapshots.surrealkv".to_owned(),
        snapshot_sled_path: "./data/test-snapshots.sled".to_owned(),
        snapshot_rustbreak_path: "./data/test-snapshots.rustbreak".to_owned(),
        snapshot_yedb_path: "./data/test-snapshots.yedb".to_owned(),
        snapshot_btree_store_path: "./data/test-snapshots.btree_store".to_owned(),
        snapshot_siamesedb_path: "./data/test-snapshots.siamesedb".to_owned(),
        snapshot_structsy_path: "./data/test-snapshots.structsy".to_owned(),
        snapshot_abyssiniandb_path: "./data/test-snapshots.abyssiniandb".to_owned(),
        snapshot_aeternusdb_path: "./data/test-snapshots.aeternusdb".to_owned(),
        snapshot_thunderdb_path: "./data/test-snapshots.thunderdb".to_owned(),
        snapshot_thetadb_path: "./data/test-snapshots.thetadb".to_owned(),
        snapshot_tinybase_path: "./data/test-snapshots.tinybase".to_owned(),
        snapshot_tinydb_path: "./data/test-snapshots.tinydb".to_owned(),
        snapshot_dblite_path: "./data/test-snapshots.dblite".to_owned(),
        snapshot_dbless_path: "./data/test-snapshots.dbless".to_owned(),
        snapshot_db_rs_path: "./data/test-snapshots.db_rs".to_owned(),
        snapshot_dharmadb_path: "./data/test-snapshots.dharmadb".to_owned(),
        snapshot_sanakirja_path: "./data/test-snapshots.sanakirja".to_owned(),
        snapshot_snaildb_path: "./data/test-snapshots.snaildb".to_owned(),
        snapshot_tinykv_path: "./data/test-snapshots.tinykv.json".to_owned(),
        snapshot_vsdb_path: "./data/test-snapshots.vsdb".to_owned(),
        snapshot_yakv_path: "./data/test-snapshots.yakv".to_owned(),
        snapshot_yakvdb_path: "./data/test-snapshots.yakvdb".to_owned(),
        snapshot_saberdb_path: "./data/test-snapshots.saberdb.json".to_owned(),
        snapshot_smolldb_path: "./data/test-snapshots.smolldb".to_owned(),
        snapshot_kstone_path: "./data/test-snapshots.kstone".to_owned(),
        snapshot_roughdb_path: "./data/test-snapshots.roughdb".to_owned(),
        snapshot_raindb_path: "./data/test-snapshots.raindb".to_owned(),
        snapshot_infusedb_path: "./data/test-snapshots.infusedb".to_owned(),
        snapshot_kafi_path: "./data/test-snapshots.kafi".to_owned(),
        snapshot_tinkv_path: "./data/test-snapshots.tinkv".to_owned(),
        snapshot_ledger_kv_path: "./data/test-snapshots.ledger_kv".to_owned(),
        snapshot_joydb_path: "./data/test-snapshots.joydb.json".to_owned(),
        snapshot_s3_endpoint: None,
        snapshot_s3_region: "us-east-1".to_owned(),
        snapshot_s3_bucket: None,
        snapshot_s3_prefix: "snapshots/".to_owned(),
        snapshot_s3_access_key_id: None,
        snapshot_s3_secret_access_key: None,
        snapshot_s3_session_token: None,
        snapshot_s3_timeout_secs: 5,
        snapshot_s3_path_style: true,
        snapshot_managed_base_url: None,
        snapshot_managed_auth_token: None,
        snapshot_managed_timeout_secs: 5,
        room_locator: "local".to_owned(),
        room_coordinator: "noop".to_owned(),
        room_coordinator_state_dir: "./data/test-room-coordinator".to_owned(),
        room_coordinator_sqlite_path: "./data/test-room-coordinator.sqlite3".to_owned(),
        room_coordinator_heartbeat_interval_secs: 10,
        room_coordinator_lease_ttl_secs: 30,
        room_coordination_managed_base_url: None,
        room_coordination_managed_auth_token: None,
        room_coordination_managed_timeout_secs: 5,
        node_id: "test-node".to_owned(),
        node_base_url: None,
        room_owner_hints_path: None,
    }
}

fn temp_snapshot_dir(test_name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("backend-{test_name}-{}", Uuid::new_v4()))
}

fn configure_shared_sqlite_collaboration(
    config: &mut Config,
    root: &std::path::Path,
    node_id: &str,
    node_base_url: &str,
) {
    config.snapshot_store = "sqlite".to_owned();
    config.snapshot_sqlite_path = root
        .join("snapshots.sqlite3")
        .to_string_lossy()
        .into_owned();
    config.room_locator = "sqlite".to_owned();
    config.room_coordinator = "sqlite".to_owned();
    config.room_coordinator_sqlite_path = root
        .join("room-coordinator.sqlite3")
        .to_string_lossy()
        .into_owned();
    config.node_id = node_id.to_owned();
    config.node_base_url = Some(node_base_url.to_owned());
}

fn configure_managed_snapshot_store(
    config: &mut Config,
    managed_base_url: &str,
    managed_auth_token: Option<&str>,
) {
    config.snapshot_store = "managed".to_owned();
    config.snapshot_managed_base_url = Some(managed_base_url.to_owned());
    config.snapshot_managed_auth_token = managed_auth_token.map(str::to_owned);
    config.snapshot_managed_timeout_secs = 5;
}

fn configure_s3_snapshot_store(
    config: &mut Config,
    endpoint: &str,
    bucket: &str,
    access_key_id: &str,
    secret_access_key: &str,
) {
    config.snapshot_store = "s3".to_owned();
    config.snapshot_s3_endpoint = Some(endpoint.to_owned());
    config.snapshot_s3_region = "us-east-1".to_owned();
    config.snapshot_s3_bucket = Some(bucket.to_owned());
    config.snapshot_s3_prefix = "snapshots/test-suite/".to_owned();
    config.snapshot_s3_access_key_id = Some(access_key_id.to_owned());
    config.snapshot_s3_secret_access_key = Some(secret_access_key.to_owned());
    config.snapshot_s3_session_token = None;
    config.snapshot_s3_timeout_secs = 5;
    config.snapshot_s3_path_style = true;
}

fn configure_redb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "redb".to_owned();
    config.snapshot_redb_path = root.join("snapshots.redb").to_string_lossy().into_owned();
}

fn configure_agdb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "agdb".to_owned();
    config.snapshot_agdb_path = root.join("snapshots.agdb").to_string_lossy().into_owned();
}

fn configure_amandine_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "amandine".to_owned();
    config.snapshot_amandine_path = root
        .join("snapshots.amandine")
        .to_string_lossy()
        .into_owned();
}

fn configure_flash_kv_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "flash_kv".to_owned();
    config.snapshot_flash_kv_path = root
        .join("snapshots.flash_kv")
        .to_string_lossy()
        .into_owned();
}

fn configure_blockbucket_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "blockbucket".to_owned();
    config.snapshot_blockbucket_path = root
        .join("snapshots.blockbucket")
        .to_string_lossy()
        .into_owned();
}

fn configure_grebedb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "grebedb".to_owned();
    config.snapshot_grebedb_path = root
        .join("snapshots.grebedb")
        .to_string_lossy()
        .into_owned();
}

fn configure_grumpydb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "grumpydb".to_owned();
    config.snapshot_grumpydb_path = root
        .join("snapshots.grumpydb")
        .to_string_lossy()
        .into_owned();
}

fn configure_highlandcows_isam_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "highlandcows_isam".to_owned();
    config.snapshot_highlandcows_isam_path = root
        .join("snapshots.highlandcows_isam")
        .to_string_lossy()
        .into_owned();
}

fn configure_eight_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "eight".to_owned();
    config.snapshot_eight_path = root.join("snapshots.eight").to_string_lossy().into_owned();
}

fn configure_epoch_db_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "epoch_db".to_owned();
    config.snapshot_epoch_db_path = root
        .join("snapshots.epoch_db")
        .to_string_lossy()
        .into_owned();
}

fn configure_rumdb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "rumdb".to_owned();
    config.snapshot_rumdb_path = root.join("snapshots.rumdb").to_string_lossy().into_owned();
}

fn configure_simple_db_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "simple_db".to_owned();
    config.snapshot_simple_db_path = root
        .join("snapshots.simple_db")
        .to_string_lossy()
        .into_owned();
}

fn configure_docdb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "docdb".to_owned();
    config.snapshot_docdb_path = root
        .join("snapshots.docdb.json")
        .to_string_lossy()
        .into_owned();
}

fn configure_shorterdb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "shorterdb".to_owned();
    config.snapshot_shorterdb_path = root
        .join("snapshots.shorterdb")
        .to_string_lossy()
        .into_owned();
}

fn configure_fjall_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "fjall".to_owned();
    config.snapshot_fjall_path = root.join("snapshots.fjall").to_string_lossy().into_owned();
}

fn configure_persy_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "persy".to_owned();
    config.snapshot_persy_path = root.join("snapshots.persy").to_string_lossy().into_owned();
}

fn configure_persistent_kv_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "persistent_kv".to_owned();
    config.snapshot_persistent_kv_path = root
        .join("snapshots.persistent_kv")
        .to_string_lossy()
        .into_owned();
}

fn configure_native_db_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "native_db".to_owned();
    config.snapshot_native_db_path = root
        .join("snapshots.native_db")
        .to_string_lossy()
        .into_owned();
}

fn configure_nebari_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "nebari".to_owned();
    config.snapshot_nebari_path = root.join("snapshots.nebari").to_string_lossy().into_owned();
}

fn configure_nodb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "nodb".to_owned();
    config.snapshot_nodb_path = root.join("snapshots.nodb").to_string_lossy().into_owned();
}

fn configure_okofdb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "okofdb".to_owned();
    config.snapshot_okofdb_path = root.join("snapshots.okofdb").to_string_lossy().into_owned();
}

fn configure_celerix_store_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "celerix_store".to_owned();
    config.snapshot_celerix_store_path = root
        .join("snapshots.celerix_store")
        .to_string_lossy()
        .into_owned();
}

fn configure_citadeldb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "citadeldb".to_owned();
    config.snapshot_citadeldb_path = root
        .join("snapshots.citadeldb")
        .to_string_lossy()
        .into_owned();
    config.snapshot_citadeldb_passphrase = "test-citadel-snapshot-passphrase".to_owned();
}

fn configure_nikidb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "nikidb".to_owned();
    config.snapshot_nikidb_path = root.join("snapshots.nikidb").to_string_lossy().into_owned();
}

fn configure_parity_db_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "parity_db".to_owned();
    config.snapshot_parity_db_path = root
        .join("snapshots.parity_db")
        .to_string_lossy()
        .into_owned();
}

fn configure_jammdb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "jammdb".to_owned();
    config.snapshot_jammdb_path = root.join("snapshots.jammdb").to_string_lossy().into_owned();
}

fn configure_janql_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "janql".to_owned();
    config.snapshot_janql_path = root.join("snapshots.janql").to_string_lossy().into_owned();
}

fn configure_jasondb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "jasondb".to_owned();
    config.snapshot_jasondb_path = root
        .join("snapshots.jasondb")
        .to_string_lossy()
        .into_owned();
}

fn configure_jasonisnthappy_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "jasonisnthappy".to_owned();
    config.snapshot_jasonisnthappy_path = root
        .join("snapshots.jasonisnthappy")
        .to_string_lossy()
        .into_owned();
}

fn configure_datastack_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "datastack".to_owned();
    config.snapshot_datastack_path = root
        .join("snapshots.datastack")
        .to_string_lossy()
        .into_owned();
}

fn configure_crystal_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "crystal".to_owned();
    config.snapshot_crystal_path = root
        .join("snapshots.crystal")
        .to_string_lossy()
        .into_owned();
}

fn configure_assystem_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "assystem".to_owned();
    config.snapshot_assystem_path = root
        .join("snapshots.assystem")
        .to_string_lossy()
        .into_owned();
}

fn configure_colon_db_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "colon_db".to_owned();
    config.snapshot_colon_db_path = root
        .join("snapshots.colon_db")
        .to_string_lossy()
        .into_owned();
}

fn configure_mace_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "mace".to_owned();
    config.snapshot_mace_path = root.join("snapshots.mace").to_string_lossy().into_owned();
}

fn configure_heed_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "heed".to_owned();
    config.snapshot_heed_path = root.join("snapshots.heed").to_string_lossy().into_owned();
}

fn configure_hightower_kv_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "hightower_kv".to_owned();
    config.snapshot_hightower_kv_path = root
        .join("snapshots.hightower_kv")
        .to_string_lossy()
        .into_owned();
}

fn configure_sled_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "sled".to_owned();
    config.snapshot_sled_path = root.join("snapshots.sled").to_string_lossy().into_owned();
}

fn configure_readb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "readb".to_owned();
    config.snapshot_readb_path = root.join("snapshots.readb").to_string_lossy().into_owned();
}

fn configure_rskey_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "rskey".to_owned();
    config.snapshot_rskey_path = root.join("snapshots.rskey").to_string_lossy().into_owned();
}

fn configure_rustlite_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "rustlite".to_owned();
    config.snapshot_rustlite_path = root
        .join("snapshots.rustlite")
        .to_string_lossy()
        .into_owned();
}

fn configure_rusty_leveldb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "rusty_leveldb".to_owned();
    config.snapshot_rusty_leveldb_path = root
        .join("snapshots.rusty_leveldb")
        .to_string_lossy()
        .into_owned();
}

fn configure_canopydb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "canopydb".to_owned();
    config.snapshot_canopydb_path = root
        .join("snapshots.canopydb")
        .to_string_lossy()
        .into_owned();
}

fn configure_caves_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "caves".to_owned();
    config.snapshot_caves_path = root.join("snapshots.caves").to_string_lossy().into_owned();
}

fn configure_ckydb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "ckydb".to_owned();
    config.snapshot_ckydb_path = root.join("snapshots.ckydb").to_string_lossy().into_owned();
}

fn configure_crepedb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "crepedb".to_owned();
    config.snapshot_crepedb_path = root
        .join("snapshots.crepedb")
        .to_string_lossy()
        .into_owned();
}

fn configure_scdb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "scdb".to_owned();
    config.snapshot_scdb_path = root.join("snapshots.scdb").to_string_lossy().into_owned();
}

fn configure_skv_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "skv".to_owned();
    config.snapshot_skv_path = root.join("snapshots.skv").to_string_lossy().into_owned();
}

fn configure_surrealkv_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "surrealkv".to_owned();
    config.snapshot_surrealkv_path = root
        .join("snapshots.surrealkv")
        .to_string_lossy()
        .into_owned();
}

fn configure_pickledb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "pickledb".to_owned();
    config.snapshot_pickledb_path = root
        .join("snapshots.pickledb")
        .to_string_lossy()
        .into_owned();
}

fn configure_microkv_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "microkv".to_owned();
    config.snapshot_microkv_path = root
        .join("snapshots_microkv")
        .to_string_lossy()
        .into_owned();
}

fn configure_rustbreak_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "rustbreak".to_owned();
    config.snapshot_rustbreak_path = root
        .join("snapshots.rustbreak")
        .to_string_lossy()
        .into_owned();
}

fn configure_rustcask_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "rustcask".to_owned();
    config.snapshot_rustcask_path = root
        .join("snapshots.rustcask")
        .to_string_lossy()
        .into_owned();
}

fn configure_yedb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "yedb".to_owned();
    config.snapshot_yedb_path = root.join("snapshots.yedb").to_string_lossy().into_owned();
}

fn configure_btree_store_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "btree_store".to_owned();
    config.snapshot_btree_store_path = root
        .join("snapshots.btree_store")
        .to_string_lossy()
        .into_owned();
}

fn configure_siamesedb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "siamesedb".to_owned();
    config.snapshot_siamesedb_path = root
        .join("snapshots.siamesedb")
        .to_string_lossy()
        .into_owned();
}

fn configure_structsy_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "structsy".to_owned();
    config.snapshot_structsy_path = root
        .join("snapshots.structsy")
        .to_string_lossy()
        .into_owned();
}

fn configure_abyssiniandb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "abyssiniandb".to_owned();
    config.snapshot_abyssiniandb_path = root
        .join("snapshots.abyssiniandb")
        .to_string_lossy()
        .into_owned();
}

fn configure_thunderdb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "thunderdb".to_owned();
    config.snapshot_thunderdb_path = root
        .join("snapshots.thunderdb")
        .to_string_lossy()
        .into_owned();
}

fn configure_tinybase_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "tinybase".to_owned();
    config.snapshot_tinybase_path = root
        .join("snapshots.tinybase")
        .to_string_lossy()
        .into_owned();
}

fn configure_tinydb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "tinydb".to_owned();
    config.snapshot_tinydb_path = root.join("snapshots.tinydb").to_string_lossy().into_owned();
}

fn configure_vsdb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "vsdb".to_owned();
    config.snapshot_vsdb_path = root.join("snapshots.vsdb").to_string_lossy().into_owned();
}

fn configure_thetadb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "thetadb".to_owned();
    config.snapshot_thetadb_path = root
        .join("snapshots.thetadb")
        .to_string_lossy()
        .into_owned();
}

fn configure_dblite_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "dblite".to_owned();
    config.snapshot_dblite_path = root.join("snapshots.dblite").to_string_lossy().into_owned();
}

fn configure_dbless_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "dbless".to_owned();
    config.snapshot_dbless_path = root.join("snapshots.dbless").to_string_lossy().into_owned();
}

fn configure_aeternusdb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "aeternusdb".to_owned();
    config.snapshot_aeternusdb_path = root
        .join("snapshots.aeternusdb")
        .to_string_lossy()
        .into_owned();
}

fn configure_sanakirja_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "sanakirja".to_owned();
    config.snapshot_sanakirja_path = root
        .join("snapshots.sanakirja")
        .to_string_lossy()
        .into_owned();
}

fn configure_snaildb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "snaildb".to_owned();
    config.snapshot_snaildb_path = root
        .join("snapshots.snaildb")
        .to_string_lossy()
        .into_owned();
}

fn configure_tinykv_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "tinykv".to_owned();
    config.snapshot_tinykv_path = root
        .join("snapshots.tinykv.json")
        .to_string_lossy()
        .into_owned();
}

fn configure_yakv_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "yakv".to_owned();
    config.snapshot_yakv_path = root.join("snapshots.yakv").to_string_lossy().into_owned();
}

fn configure_yakvdb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "yakvdb".to_owned();
    config.snapshot_yakvdb_path = root.join("snapshots.yakvdb").to_string_lossy().into_owned();
}

fn configure_saberdb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "saberdb".to_owned();
    config.snapshot_saberdb_path = root
        .join("snapshots.saberdb.json")
        .to_string_lossy()
        .into_owned();
}

fn configure_smolldb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "smolldb".to_owned();
    config.snapshot_smolldb_path = root
        .join("snapshots.smolldb")
        .to_string_lossy()
        .into_owned();
}

fn configure_ghaladb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "ghaladb".to_owned();
    config.snapshot_ghaladb_path = root
        .join("snapshots.ghaladb")
        .to_string_lossy()
        .into_owned();
}

fn configure_apex_store_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "apex_store".to_owned();
    config.snapshot_apex_store_path = root
        .join("snapshots.apex_store")
        .to_string_lossy()
        .into_owned();
}

fn configure_kstone_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "kstone".to_owned();
    config.snapshot_kstone_path = root.join("snapshots.kstone").to_string_lossy().into_owned();
}

fn configure_roughdb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "roughdb".to_owned();
    config.snapshot_roughdb_path = root
        .join("snapshots.roughdb")
        .to_string_lossy()
        .into_owned();
}

fn configure_raindb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "raindb".to_owned();
    config.snapshot_raindb_path = root.join("snapshots.raindb").to_string_lossy().into_owned();
}

fn configure_infusedb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "infusedb".to_owned();
    config.snapshot_infusedb_path = root
        .join("snapshots.infusedb")
        .to_string_lossy()
        .into_owned();
}

fn configure_kafi_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "kafi".to_owned();
    config.snapshot_kafi_path = root.join("snapshots.kafi").to_string_lossy().into_owned();
}

fn configure_tinkv_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "tinkv".to_owned();
    config.snapshot_tinkv_path = root.join("snapshots.tinkv").to_string_lossy().into_owned();
    config.snapshot_ledger_kv_path = root
        .join("snapshots.ledger_kv")
        .to_string_lossy()
        .into_owned();
}

fn configure_ledger_kv_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "ledger_kv".to_owned();
    config.snapshot_ledger_kv_path = root
        .join("snapshots.ledger_kv")
        .to_string_lossy()
        .into_owned();
}

fn configure_joydb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "joydb".to_owned();
    config.snapshot_joydb_path = root
        .join("snapshots.joydb.json")
        .to_string_lossy()
        .into_owned();
}

fn configure_bitcask_engine_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "bitcask_engine".to_owned();
    config.snapshot_bitcask_engine_path = root
        .join("snapshots.bitcask_engine")
        .to_string_lossy()
        .into_owned();
}

fn configure_blazeup_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "blazeup".to_owned();
    config.snapshot_blazeup_path = root
        .join("snapshots.blazeup")
        .to_string_lossy()
        .into_owned();
}

fn configure_feoxdb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "feoxdb".to_owned();
    config.snapshot_feoxdb_path = root.join("snapshots.feoxdb").to_string_lossy().into_owned();
}

fn configure_db_rs_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "db_rs".to_owned();
    config.snapshot_db_rs_path = root.join("snapshots.db_rs").to_string_lossy().into_owned();
}

fn configure_dharmadb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "dharmadb".to_owned();
    config.snapshot_dharmadb_path = root
        .join("snapshots.dharmadb")
        .to_string_lossy()
        .into_owned();
}

fn configure_jsondb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "jsondb".to_owned();
    config.snapshot_jsondb_path = root
        .join("snapshots.jsondb.json")
        .to_string_lossy()
        .into_owned();
}

fn configure_kopperdb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "kopperdb".to_owned();
    config.snapshot_kopperdb_path = root
        .join("snapshots.kopperdb")
        .to_string_lossy()
        .into_owned();
}

fn configure_rcask_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "rcask".to_owned();
    config.snapshot_rcask_path = root.join("snapshots.rcask").to_string_lossy().into_owned();
}

fn configure_jfs_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "jfs".to_owned();
    config.snapshot_jfs_path = root
        .join("snapshots.jfs.json")
        .to_string_lossy()
        .into_owned();
}

fn configure_koit_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "koit".to_owned();
    config.snapshot_koit_path = root
        .join("snapshots.koit.json")
        .to_string_lossy()
        .into_owned();
}

fn configure_lite_db_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "lite_db".to_owned();
    config.snapshot_lite_db_path = root
        .join("snapshots.lite_db")
        .to_string_lossy()
        .into_owned();
}

fn configure_log_kv_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "log_kv".to_owned();
    config.snapshot_log_kv_path = root.join("snapshots.log_kv").to_string_lossy().into_owned();
}

fn configure_append_kv_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "append_kv".to_owned();
    config.snapshot_append_kv_path = root
        .join("snapshots.append_kv")
        .to_string_lossy()
        .into_owned();
}

fn configure_mhdb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "mhdb".to_owned();
    config.snapshot_mhdb_path = root.join("snapshots.mhdb").to_string_lossy().into_owned();
}

fn configure_loro_kv_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "loro_kv".to_owned();
    config.snapshot_loro_kv_path = root
        .join("snapshots.loro_kv")
        .to_string_lossy()
        .into_owned();
}

fn configure_luckdb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "luckdb".to_owned();
    config.snapshot_luckdb_path = root
        .join("snapshots.luckdb.json")
        .to_string_lossy()
        .into_owned();
}

fn configure_ipjdb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "ipjdb".to_owned();
    config.snapshot_ipjdb_path = root.join("snapshots.ipjdb").to_string_lossy().into_owned();
}

fn configure_rubin_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "rubin".to_owned();
    config.snapshot_rubin_path = root
        .join("snapshots.rubin.json")
        .to_string_lossy()
        .into_owned();
}

fn configure_lsm_engine_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "lsm_engine".to_owned();
    config.snapshot_lsm_engine_path = root
        .join("snapshots.lsm_engine")
        .to_string_lossy()
        .into_owned();
}

fn configure_etchdb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "etchdb".to_owned();
    config.snapshot_etchdb_path = root.join("snapshots.etchdb").to_string_lossy().into_owned();
}

fn configure_lsm_storage_engine_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "lsm_storage_engine".to_owned();
    config.snapshot_lsm_storage_engine_path = root
        .join("snapshots.lsm_storage_engine")
        .to_string_lossy()
        .into_owned();
}

fn configure_lsmdb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "lsmdb".to_owned();
    config.snapshot_lsmdb_path = root.join("snapshots.lsmdb").to_string_lossy().into_owned();
}

fn configure_lsm_tree_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "lsm_tree".to_owned();
    config.snapshot_lsm_tree_path = root
        .join("snapshots.lsm_tree")
        .to_string_lossy()
        .into_owned();
}

fn configure_ferrumdb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "ferrumdb".to_owned();
    config.snapshot_ferrumdb_path = root
        .join("snapshots.ferrumdb")
        .to_string_lossy()
        .into_owned();
}

fn configure_mindb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "mindb".to_owned();
    config.snapshot_mindb_path = root.join("snapshots.mindb").to_string_lossy().into_owned();
}

fn configure_mmdb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "mmdb".to_owned();
    config.snapshot_mmdb_path = root.join("snapshots.mmdb").to_string_lossy().into_owned();
}

fn configure_nanodb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "nanodb".to_owned();
    config.snapshot_nanodb_path = root
        .join("snapshots.nanodb.json")
        .to_string_lossy()
        .into_owned();
}

fn configure_graus_db_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "graus_db".to_owned();
    config.snapshot_graus_db_path = root
        .join("snapshots.graus_db")
        .to_string_lossy()
        .into_owned();
}

fn configure_kv_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "kv".to_owned();
    config.snapshot_kv_path = root.join("snapshots.kv").to_string_lossy().into_owned();
}

fn configure_json_store_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "json_store".to_owned();
    config.snapshot_json_store_path = root
        .join("snapshots.json_store.jsonl")
        .to_string_lossy()
        .into_owned();
}

fn configure_hmdb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "hmdb".to_owned();
    config.snapshot_hmdb_path = root.join("snapshots.hmdb").to_string_lossy().into_owned();
}

fn configure_icefalldb_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "icefalldb".to_owned();
    config.snapshot_icefalldb_path = root
        .join("snapshots.icefalldb")
        .to_string_lossy()
        .into_owned();
}

fn configure_bitask_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "bitask".to_owned();
    config.snapshot_bitask_path = root.join("snapshots.bitask").to_string_lossy().into_owned();
}

fn configure_bitkv_rs_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "bitkv_rs".to_owned();
    config.snapshot_bitkv_rs_path = root
        .join("snapshots.bitkv_rs")
        .to_string_lossy()
        .into_owned();
}

fn configure_candystore_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "candystore".to_owned();
    config.snapshot_candystore_path = root
        .join("snapshots.candystore")
        .to_string_lossy()
        .into_owned();
}

fn configure_cuendillar_snapshot_store(config: &mut Config, root: &std::path::Path) {
    config.snapshot_store = "cuendillar".to_owned();
    config.snapshot_cuendillar_path = root
        .join("snapshots.cuendillar")
        .to_string_lossy()
        .into_owned();
}

fn configure_managed_coordination_with_shared_sqlite_snapshots(
    config: &mut Config,
    root: &std::path::Path,
    node_id: &str,
    node_base_url: &str,
    managed_base_url: &str,
    managed_auth_token: Option<&str>,
) {
    config.snapshot_store = "sqlite".to_owned();
    config.snapshot_sqlite_path = root
        .join("snapshots.sqlite3")
        .to_string_lossy()
        .into_owned();
    config.room_locator = "managed".to_owned();
    config.room_coordinator = "managed".to_owned();
    config.room_coordination_managed_base_url = Some(managed_base_url.to_owned());
    config.room_coordination_managed_auth_token = managed_auth_token.map(str::to_owned);
    config.room_coordinator_heartbeat_interval_secs = 1;
    config.room_coordinator_lease_ttl_secs = 3;
    config.node_id = node_id.to_owned();
    config.node_base_url = Some(node_base_url.to_owned());
}

fn configure_managed_coordination_with_managed_snapshots(
    config: &mut Config,
    node_id: &str,
    node_base_url: &str,
    coordination_base_url: &str,
    snapshot_base_url: &str,
    managed_auth_token: Option<&str>,
) {
    configure_managed_snapshot_store(config, snapshot_base_url, managed_auth_token);
    config.room_locator = "managed".to_owned();
    config.room_coordinator = "managed".to_owned();
    config.room_coordination_managed_base_url = Some(coordination_base_url.to_owned());
    config.room_coordination_managed_auth_token = managed_auth_token.map(str::to_owned);
    config.room_coordinator_heartbeat_interval_secs = 1;
    config.room_coordinator_lease_ttl_secs = 3;
    config.node_id = node_id.to_owned();
    config.node_base_url = Some(node_base_url.to_owned());
}

fn admin_auth_header(config: &Config) -> String {
    format!("Bearer {}", config.api_token)
}

fn document_auth_header(access_token: &str) -> String {
    format!("Bearer {access_token}")
}

#[derive(Debug, Clone, Default)]
struct MockS3ServiceState {
    objects: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    last_authorization: Arc<Mutex<Option<String>>>,
}

impl MockS3ServiceState {
    fn object(&self, key: &str) -> Option<Vec<u8>> {
        self.objects
            .lock()
            .expect("mock s3 object store should not be poisoned")
            .get(key)
            .cloned()
    }

    fn last_authorization(&self) -> Option<String> {
        self.last_authorization
            .lock()
            .expect("mock s3 auth state should not be poisoned")
            .clone()
    }
}

struct MockS3Harness {
    state: MockS3ServiceState,
    endpoint: String,
    bucket: String,
    access_key_id: String,
    secret_access_key: String,
    task: JoinHandle<()>,
}

impl Drop for MockS3Harness {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn spawn_mock_s3_snapshot_service() -> MockS3Harness {
    let state = MockS3ServiceState::default();
    let bucket = "backend-test-snapshots".to_owned();
    let access_key_id = "test-access-key".to_owned();
    let secret_access_key = "test-secret-key".to_owned();
    let app = Router::new()
        .fallback(mock_s3_dispatch)
        .with_state(state.clone());

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("mock s3 listener should bind");
    let addr = listener
        .local_addr()
        .expect("mock s3 listener should expose local addr");
    let task = tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("mock s3 service should serve");
    });

    MockS3Harness {
        state,
        endpoint: format!("http://{addr}"),
        bucket,
        access_key_id,
        secret_access_key,
        task,
    }
}

async fn mock_s3_dispatch(
    State(state): State<MockS3ServiceState>,
    method: Method,
    uri: Uri,
    Query(query): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let authorization = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    *state
        .last_authorization
        .lock()
        .expect("mock s3 auth state should not be poisoned") = authorization;

    let path = uri.path().trim_start_matches('/');
    if path.is_empty() {
        return mock_s3_xml_error(StatusCode::NOT_FOUND, "NoSuchBucket", "bucket not found");
    }

    let (bucket, key) = match path.split_once('/') {
        Some((bucket, "")) => (bucket, None),
        Some((bucket, key)) => (bucket, Some(key)),
        None => (path, None),
    };

    if bucket.trim().is_empty() {
        return mock_s3_xml_error(StatusCode::NOT_FOUND, "NoSuchBucket", "bucket not found");
    }

    if key.is_none() && query.get("list-type").map(String::as_str) == Some("2") {
        let prefix = query.get("prefix").cloned().unwrap_or_default();
        let mut objects = state
            .objects
            .lock()
            .expect("mock s3 object store should not be poisoned")
            .iter()
            .filter(|(key, _)| key.starts_with(&prefix))
            .map(|(key, bytes)| (key.clone(), bytes.len()))
            .collect::<Vec<_>>();
        objects.sort_by(|left, right| left.0.cmp(&right.0));

        let contents = objects
            .into_iter()
            .map(|(key, size)| {
                format!(
                    "<Contents><Key>{key}</Key><Size>{size}</Size><StorageClass>STANDARD</StorageClass></Contents>"
                )
            })
            .collect::<String>();
        let xml = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?><ListBucketResult><Name>{bucket}</Name><Prefix>{prefix}</Prefix><KeyCount>{key_count}</KeyCount><MaxKeys>1000</MaxKeys><IsTruncated>false</IsTruncated>{contents}</ListBucketResult>",
            key_count = contents.matches("<Contents>").count(),
        );
        return mock_s3_xml_response(StatusCode::OK, xml);
    }

    let Some(key) = key else {
        return mock_s3_xml_error(StatusCode::NOT_FOUND, "NoSuchKey", "object not found");
    };

    match method {
        Method::GET => match state.object(key) {
            Some(bytes) => (StatusCode::OK, bytes).into_response(),
            None => mock_s3_xml_error(StatusCode::NOT_FOUND, "NoSuchKey", "object not found"),
        },
        Method::PUT => {
            state
                .objects
                .lock()
                .expect("mock s3 object store should not be poisoned")
                .insert(key.to_owned(), body.to_vec());
            StatusCode::OK.into_response()
        }
        Method::DELETE => {
            state
                .objects
                .lock()
                .expect("mock s3 object store should not be poisoned")
                .remove(key);
            StatusCode::NO_CONTENT.into_response()
        }
        _ => StatusCode::METHOD_NOT_ALLOWED.into_response(),
    }
}

fn mock_s3_xml_response(status: StatusCode, body: String) -> Response {
    let mut response = (status, body).into_response();
    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("application/xml"),
    );
    response
}

fn mock_s3_xml_error(status: StatusCode, code: &str, message: &str) -> Response {
    mock_s3_xml_response(
        status,
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?><Error><Code>{code}</Code><Message>{message}</Message></Error>"
        ),
    )
}

#[derive(Debug, Default)]
struct RemoteRoomLocator;

impl RoomLocator for RemoteRoomLocator {
    fn resolve(&self, doc_id: &Uuid) -> Result<ResolvedRoom, RoomLocatorError> {
        Ok(ResolvedRoom::Remote(RoomOwnerHint {
            node_id: format!("node-for-{doc_id}"),
            base_url: Some("http://node-b.internal:4000".to_owned()),
        }))
    }
}

#[derive(Debug, Default)]
struct RecordingRoomCoordinator {
    events: Mutex<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MockManagedLeaseRecord {
    doc_id: Uuid,
    node_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    base_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    lease_id: Option<Uuid>,
    #[serde(default)]
    epoch: u64,
    activated_at: chrono::DateTime<Utc>,
    #[serde(default, alias = "updated_at", skip_serializing_if = "Option::is_none")]
    renewed_at: Option<chrono::DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    expires_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
struct MockManagedAcquireRequest {
    node_id: String,
    base_url: Option<String>,
    lease_ttl_secs: u64,
}

#[derive(Debug, Deserialize)]
struct MockManagedRenewRequest {
    node_id: String,
    lease_id: Uuid,
    epoch: u64,
    lease_ttl_secs: u64,
}

#[derive(Debug, Deserialize)]
struct MockManagedReleaseRequest {
    node_id: String,
    lease_id: Uuid,
    epoch: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MockManagedSnapshotPayload {
    document: MockManagedSnapshotDocument,
    update: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MockManagedSnapshotDocument {
    id: Uuid,
    title: String,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
    access_token: String,
}

#[derive(Debug, Serialize)]
struct MockManagedSnapshotCatalogResponse {
    documents: Vec<MockManagedSnapshotDocument>,
}

impl From<DocumentSnapshot> for MockManagedSnapshotPayload {
    fn from(snapshot: DocumentSnapshot) -> Self {
        let document = snapshot.document;
        let access_token = document.access_token().to_owned();
        Self {
            document: MockManagedSnapshotDocument {
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

#[derive(Debug, Clone)]
struct MockManagedCoordinationServiceState {
    leases: Arc<Mutex<HashMap<Uuid, MockManagedLeaseRecord>>>,
    snapshots: Arc<Mutex<HashMap<Uuid, MockManagedSnapshotPayload>>>,
    auth_token: Option<String>,
}

impl MockManagedCoordinationServiceState {
    fn lease(&self, doc_id: &Uuid) -> Option<MockManagedLeaseRecord> {
        self.leases
            .lock()
            .expect("managed coordination lease store should not be poisoned")
            .get(doc_id)
            .cloned()
    }

    fn snapshot(&self, doc_id: &Uuid) -> Option<MockManagedSnapshotPayload> {
        self.snapshots
            .lock()
            .expect("managed snapshot store should not be poisoned")
            .get(doc_id)
            .cloned()
    }
}

struct MockManagedCoordinationHarness {
    state: MockManagedCoordinationServiceState,
    base_url: String,
    snapshot_base_url: String,
    task: JoinHandle<()>,
}

impl Drop for MockManagedCoordinationHarness {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn spawn_mock_managed_coordination_service(
    auth_token: Option<&str>,
) -> MockManagedCoordinationHarness {
    let state = MockManagedCoordinationServiceState {
        leases: Arc::new(Mutex::new(HashMap::new())),
        snapshots: Arc::new(Mutex::new(HashMap::new())),
        auth_token: auth_token.map(str::to_owned),
    };
    let app = Router::new()
        .route("/coord/v1/leases/{doc_id}", get(mock_managed_lookup_lease))
        .route(
            "/coord/v1/leases/{doc_id}/acquire",
            post(mock_managed_acquire_lease),
        )
        .route(
            "/coord/v1/leases/{doc_id}/renew",
            post(mock_managed_renew_lease),
        )
        .route(
            "/coord/v1/leases/{doc_id}/release",
            post(mock_managed_release_lease),
        )
        .route("/snapshot/v1/snapshots", get(mock_managed_list_snapshots))
        .route(
            "/snapshot/v1/snapshots/{doc_id}",
            get(mock_managed_get_snapshot)
                .put(mock_managed_put_snapshot)
                .delete(mock_managed_delete_snapshot),
        )
        .with_state(state.clone());

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("managed coordination listener should bind");
    let addr = listener
        .local_addr()
        .expect("managed coordination listener should expose local addr");
    let task = tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("managed coordination service should serve");
    });

    MockManagedCoordinationHarness {
        state,
        base_url: format!("http://{addr}/coord"),
        snapshot_base_url: format!("http://{addr}/snapshot"),
        task,
    }
}

fn mock_managed_authorize(
    headers: &HeaderMap,
    state: &MockManagedCoordinationServiceState,
) -> Result<(), StatusCode> {
    let Some(expected_auth_token) = state.auth_token.as_deref() else {
        return Ok(());
    };
    let Some(header_value) = headers.get(axum::http::header::AUTHORIZATION) else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    let Ok(header_value) = header_value.to_str() else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    if header_value == format!("Bearer {expected_auth_token}") {
        Ok(())
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

async fn mock_managed_lookup_lease(
    State(state): State<MockManagedCoordinationServiceState>,
    Path(doc_id): Path<Uuid>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(status) = mock_managed_authorize(&headers, &state) {
        return status.into_response();
    }

    match state.lease(&doc_id) {
        Some(lease) => (StatusCode::OK, Json(lease)).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn mock_managed_acquire_lease(
    State(state): State<MockManagedCoordinationServiceState>,
    Path(doc_id): Path<Uuid>,
    headers: HeaderMap,
    Json(payload): Json<MockManagedAcquireRequest>,
) -> impl IntoResponse {
    if let Err(status) = mock_managed_authorize(&headers, &state) {
        return status.into_response();
    }

    let now = Utc::now();
    let ttl = ChronoDuration::seconds(payload.lease_ttl_secs as i64);
    let mut leases = state
        .leases
        .lock()
        .expect("managed coordination lease store should not be poisoned");

    if let Some(existing) = leases.get(&doc_id) {
        let active_remote_owner = existing.node_id.trim() != payload.node_id.trim()
            && existing
                .expires_at
                .map(|expires_at| expires_at > now)
                .unwrap_or(true);
        if active_remote_owner {
            return (StatusCode::CONFLICT, Json(existing.clone())).into_response();
        }
    }

    let epoch = leases
        .get(&doc_id)
        .map(|lease| lease.epoch.saturating_add(1))
        .unwrap_or(1);
    let lease = MockManagedLeaseRecord {
        doc_id,
        node_id: payload.node_id.trim().to_owned(),
        base_url: payload.base_url,
        lease_id: Some(Uuid::new_v4()),
        epoch,
        activated_at: now,
        renewed_at: Some(now),
        expires_at: Some(now + ttl),
    };
    leases.insert(doc_id, lease.clone());

    (StatusCode::OK, Json(lease)).into_response()
}

async fn mock_managed_renew_lease(
    State(state): State<MockManagedCoordinationServiceState>,
    Path(doc_id): Path<Uuid>,
    headers: HeaderMap,
    Json(payload): Json<MockManagedRenewRequest>,
) -> impl IntoResponse {
    if let Err(status) = mock_managed_authorize(&headers, &state) {
        return status.into_response();
    }

    let now = Utc::now();
    let ttl = ChronoDuration::seconds(payload.lease_ttl_secs as i64);
    let mut leases = state
        .leases
        .lock()
        .expect("managed coordination lease store should not be poisoned");
    let Some(existing) = leases.get_mut(&doc_id) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    if existing.node_id.trim() != payload.node_id.trim()
        || existing.lease_id != Some(payload.lease_id)
        || existing.epoch != payload.epoch
    {
        return (StatusCode::CONFLICT, Json(existing.clone())).into_response();
    }

    existing.renewed_at = Some(now);
    existing.expires_at = Some(now + ttl);
    (StatusCode::OK, Json(existing.clone())).into_response()
}

async fn mock_managed_release_lease(
    State(state): State<MockManagedCoordinationServiceState>,
    Path(doc_id): Path<Uuid>,
    headers: HeaderMap,
    Json(payload): Json<MockManagedReleaseRequest>,
) -> impl IntoResponse {
    if let Err(status) = mock_managed_authorize(&headers, &state) {
        return status.into_response();
    }

    let mut leases = state
        .leases
        .lock()
        .expect("managed coordination lease store should not be poisoned");
    let Some(existing) = leases.get(&doc_id) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    if existing.node_id.trim() != payload.node_id.trim()
        || existing.lease_id != Some(payload.lease_id)
        || existing.epoch != payload.epoch
    {
        return (StatusCode::CONFLICT, Json(existing.clone())).into_response();
    }

    leases.remove(&doc_id);
    StatusCode::NO_CONTENT.into_response()
}

async fn mock_managed_list_snapshots(
    State(state): State<MockManagedCoordinationServiceState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(status) = mock_managed_authorize(&headers, &state) {
        return status.into_response();
    }

    let snapshots = state
        .snapshots
        .lock()
        .expect("managed snapshot store should not be poisoned");
    let documents = snapshots
        .values()
        .map(|snapshot| snapshot.document.clone())
        .collect::<Vec<_>>();

    (
        StatusCode::OK,
        Json(MockManagedSnapshotCatalogResponse { documents }),
    )
        .into_response()
}

async fn mock_managed_get_snapshot(
    State(state): State<MockManagedCoordinationServiceState>,
    Path(doc_id): Path<Uuid>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(status) = mock_managed_authorize(&headers, &state) {
        return status.into_response();
    }

    match state.snapshot(&doc_id) {
        Some(snapshot) => (StatusCode::OK, Json(snapshot)).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn mock_managed_put_snapshot(
    State(state): State<MockManagedCoordinationServiceState>,
    Path(doc_id): Path<Uuid>,
    headers: HeaderMap,
    Json(payload): Json<MockManagedSnapshotPayload>,
) -> impl IntoResponse {
    if let Err(status) = mock_managed_authorize(&headers, &state) {
        return status.into_response();
    }

    if payload.document.id != doc_id {
        return StatusCode::BAD_REQUEST.into_response();
    }

    state
        .snapshots
        .lock()
        .expect("managed snapshot store should not be poisoned")
        .insert(doc_id, payload);
    StatusCode::NO_CONTENT.into_response()
}

async fn mock_managed_delete_snapshot(
    State(state): State<MockManagedCoordinationServiceState>,
    Path(doc_id): Path<Uuid>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(status) = mock_managed_authorize(&headers, &state) {
        return status.into_response();
    }

    state
        .snapshots
        .lock()
        .expect("managed snapshot store should not be poisoned")
        .remove(&doc_id);
    StatusCode::NO_CONTENT.into_response()
}

impl RecordingRoomCoordinator {
    fn snapshot(&self) -> Vec<String> {
        self.events
            .lock()
            .expect("recording coordinator mutex should not be poisoned")
            .clone()
    }
}

impl RoomCoordinator for RecordingRoomCoordinator {
    fn mode(&self) -> &'static str {
        "recording"
    }

    fn room_activated(&self, doc_id: &Uuid) -> Result<(), RoomCoordinatorError> {
        self.events
            .lock()
            .expect("recording coordinator mutex should not be poisoned")
            .push(format!("activate:{doc_id}"));
        Ok(())
    }

    fn room_deactivated(&self, doc_id: &Uuid) -> Result<(), RoomCoordinatorError> {
        self.events
            .lock()
            .expect("recording coordinator mutex should not be poisoned")
            .push(format!("deactivate:{doc_id}"));
        Ok(())
    }
}

#[derive(Debug, Default)]
struct FailingRoomCoordinator;

impl RoomCoordinator for FailingRoomCoordinator {
    fn mode(&self) -> &'static str {
        "failing"
    }

    fn room_activated(&self, doc_id: &Uuid) -> Result<(), RoomCoordinatorError> {
        Err(RoomCoordinatorError::Operation(format!(
            "unable to acquire lease for {doc_id}"
        )))
    }

    fn room_deactivated(&self, _doc_id: &Uuid) -> Result<(), RoomCoordinatorError> {
        Ok(())
    }
}

fn decode_sync_message(payload: impl AsRef<[u8]>) -> SyncMessage {
    let message = Message::decode_v1(payload.as_ref()).expect("websocket payload should decode");
    match message {
        Message::Sync(message) => message,
        other => panic!("expected sync message, received {other:?}"),
    }
}

async fn wait_for_sqlite_room_lease_release(sqlite_path: &std::path::Path, doc_id: Uuid) {
    for _ in 0..20 {
        let connection = rusqlite::Connection::open(sqlite_path)
            .expect("sqlite coordinator file should open while waiting for release");
        let remaining: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM room_leases WHERE doc_id = ?1",
                [doc_id.to_string()],
                |row| row.get(0),
            )
            .unwrap_or(0);
        if remaining == 0 {
            return;
        }

        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    panic!("sqlite room lease for `{doc_id}` should be released after handoff");
}

async fn wait_for_managed_room_lease_release(
    state: &MockManagedCoordinationServiceState,
    doc_id: Uuid,
) {
    for _ in 0..100 {
        if state.lease(&doc_id).is_none() {
            return;
        }

        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    panic!("managed room lease for `{doc_id}` should be released after handoff");
}

async fn wait_for_managed_room_lease_owner(
    state: &MockManagedCoordinationServiceState,
    doc_id: Uuid,
    expected_node_id: &str,
) {
    for _ in 0..100 {
        if state
            .lease(&doc_id)
            .map(|lease| lease.node_id == expected_node_id)
            .unwrap_or(false)
        {
            return;
        }

        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    panic!("managed room lease for `{doc_id}` should be owned by `{expected_node_id}`");
}

#[tokio::test]
async fn health_endpoint_returns_ok_payload() {
    let config = test_config();
    let app = build_app(
        &config,
        AppState::from_config(&config).expect("state should initialize"),
    )
    .expect("app should build");
    let server = TestServer::new(app);
    let response = server.get("/api/health").await;

    response.assert_status_ok();

    let payload = response.json::<Value>();
    assert_eq!(payload["status"], "ok");
    assert_eq!(payload["service"], "backend");
    assert!(payload["timestamp"].as_str().is_some());
}

#[tokio::test]
async fn documents_endpoint_returns_documents_array() {
    let config = test_config();
    let app = build_app(
        &config,
        AppState::from_config(&config).expect("state should initialize"),
    )
    .expect("app should build");
    let server = TestServer::new(app);
    let response = server
        .get("/api/documents")
        .add_header("Authorization", admin_auth_header(&config).as_str())
        .await;

    response.assert_status_ok();

    let payload = response.json::<Value>();
    assert!(payload["documents"].as_array().is_some());
}

#[tokio::test]
async fn create_document_endpoint_creates_document_and_lists_it() {
    let config = test_config();
    let app = build_app(
        &config,
        AppState::from_config(&config).expect("state should initialize"),
    )
    .expect("app should build");
    let server = TestServer::new(app);

    let response = server
        .post("/api/documents")
        .add_header("Authorization", admin_auth_header(&config).as_str())
        .json(&serde_json::json!({
            "title": "Design notes"
        }))
        .await;

    response.assert_status(StatusCode::CREATED);

    let payload = response.json::<Value>();
    let created_id = payload["document"]["id"]
        .as_str()
        .expect("created document id should be returned");
    let access_token = payload["credentials"]["access_token"]
        .as_str()
        .expect("document access token should be returned");
    assert_eq!(payload["document"]["title"].as_str(), Some("Design notes"));
    assert!(payload["document"]["created_at"].as_str().is_some());
    assert!(payload["document"]["updated_at"].as_str().is_some());

    let detail_response = server
        .get(&format!("/api/documents/{created_id}"))
        .add_header("Authorization", document_auth_header(access_token).as_str())
        .await;
    detail_response.assert_status_ok();

    let detail_payload = detail_response.json::<Value>();
    assert_eq!(detail_payload["document"]["id"].as_str(), Some(created_id));
    assert!(detail_payload["document"]["access_token"].is_null());

    let list_response = server
        .get("/api/documents")
        .add_header("Authorization", admin_auth_header(&config).as_str())
        .await;
    list_response.assert_status_ok();

    let list_payload = list_response.json::<Value>();
    let documents = list_payload["documents"]
        .as_array()
        .expect("documents should be returned as an array");

    assert_eq!(documents.len(), 1);
    assert_eq!(documents[0]["id"].as_str(), Some(created_id));
    assert!(documents[0]["access_token"].is_null());
}

#[tokio::test]
async fn documents_endpoint_lists_snapshot_backed_documents_after_room_eviction() {
    let config = test_config();
    let state = AppState::from_config(&config).expect("state should initialize");
    let document = state
        .rooms()
        .create_document(Some("Evicted but listed".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have an active room");
    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("idle room eviction should succeed");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);
    assert!(state.rooms().get(&document.id).is_none());

    let app = build_app(&config, state).expect("app should build");
    let server = TestServer::new(app);

    let response = server
        .get("/api/documents")
        .add_header("Authorization", admin_auth_header(&config).as_str())
        .await;

    response.assert_status_ok();

    let payload = response.json::<Value>();
    let documents = payload["documents"]
        .as_array()
        .expect("documents should be returned as an array");

    assert_eq!(documents.len(), 1);
    assert_eq!(
        documents[0]["id"].as_str(),
        Some(document.id.to_string().as_str())
    );
    assert_eq!(documents[0]["title"].as_str(), Some("Evicted but listed"));
}

#[tokio::test]
async fn delete_document_endpoint_removes_existing_document() {
    let config = test_config();
    let app = build_app(
        &config,
        AppState::from_config(&config).expect("state should initialize"),
    )
    .expect("app should build");
    let server = TestServer::new(app);

    let create_response = server
        .post("/api/documents")
        .add_header("Authorization", admin_auth_header(&config).as_str())
        .json(&serde_json::json!({
            "title": "Disposable"
        }))
        .await;
    create_response.assert_status(StatusCode::CREATED);

    let payload = create_response.json::<Value>();
    let created_id = payload["document"]["id"]
        .as_str()
        .expect("created document id should be returned")
        .to_owned();
    let access_token = payload["credentials"]["access_token"]
        .as_str()
        .expect("document access token should be returned")
        .to_owned();

    let delete_response = server
        .delete(&format!("/api/documents/{created_id}"))
        .add_header(
            "Authorization",
            document_auth_header(&access_token).as_str(),
        )
        .await;
    delete_response.assert_status(StatusCode::NO_CONTENT);

    let get_response = server
        .get(&format!("/api/documents/{created_id}"))
        .add_header(
            "Authorization",
            document_auth_header(&access_token).as_str(),
        )
        .await;
    get_response.assert_status_not_found();
}

#[tokio::test]
async fn delete_document_endpoint_rejects_documents_with_active_websocket_sessions() {
    let config = test_config();
    let state = AppState::from_config(&config).expect("state should initialize");
    let document = state
        .rooms()
        .create_document(Some("Busy delete".to_owned()))
        .expect("document should be created");
    let app = build_app(&config, state).expect("app should build");
    let server = TestServer::builder().http_transport().build(app);

    let websocket = server
        .get_websocket(&format!("/ws/{}", document.id))
        .add_header("Origin", config.frontend_origin.as_str())
        .add_header(
            "Authorization",
            document_auth_header(document.access_token()).as_str(),
        )
        .await
        .into_websocket()
        .await;

    let delete_response = server
        .delete(&format!("/api/documents/{}", document.id))
        .add_header(
            "Authorization",
            document_auth_header(document.access_token()).as_str(),
        )
        .await;

    delete_response.assert_status(StatusCode::CONFLICT);
    let payload = delete_response.json::<Value>();
    assert_eq!(payload["error"], "conflict");
    assert_eq!(
        payload["message"],
        format!(
            "document `{}` cannot be deleted while collaboration sessions are active",
            document.id
        )
    );

    websocket.close().await;
}

#[tokio::test]
async fn delete_document_endpoint_allows_delete_after_websocket_session_closes() {
    let config = test_config();
    let state = AppState::from_config(&config).expect("state should initialize");
    let document = state
        .rooms()
        .create_document(Some("Delete after close".to_owned()))
        .expect("document should be created");
    let app = build_app(&config, state).expect("app should build");
    let server = TestServer::builder().http_transport().build(app);

    let websocket = server
        .get_websocket(&format!("/ws/{}", document.id))
        .add_header("Origin", config.frontend_origin.as_str())
        .add_header(
            "Authorization",
            document_auth_header(document.access_token()).as_str(),
        )
        .await
        .into_websocket()
        .await;

    websocket.close().await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    let delete_response = server
        .delete(&format!("/api/documents/{}", document.id))
        .add_header(
            "Authorization",
            document_auth_header(document.access_token()).as_str(),
        )
        .await;
    delete_response.assert_status(StatusCode::NO_CONTENT);

    let detail_response = server
        .get(&format!("/api/documents/{}", document.id))
        .add_header(
            "Authorization",
            document_auth_header(document.access_token()).as_str(),
        )
        .await;
    detail_response.assert_status_not_found();
}

#[tokio::test]
async fn document_detail_endpoint_rejects_missing_document_with_json_error() {
    let config = test_config();
    let app = build_app(
        &config,
        AppState::from_config(&config).expect("state should initialize"),
    )
    .expect("app should build");
    let server = TestServer::new(app);
    let doc_id = Uuid::nil();

    let response = server
        .get(&format!("/api/documents/{doc_id}"))
        .add_header("Authorization", "Bearer missing-doc-token")
        .await;

    response.assert_status_not_found();

    let payload = response.json::<Value>();
    assert_eq!(payload["error"], "not_found");
    assert_eq!(
        payload["message"],
        format!("document `{doc_id}` was not found")
    );
}

#[tokio::test]
async fn document_detail_endpoint_rejects_non_local_room_owner() {
    let config = test_config();
    let state = AppState::with_snapshot_store_and_locator(
        config.frontend_origin.clone(),
        config.api_token.clone(),
        Arc::new(InMemorySnapshotStore::new()),
        Arc::new(RemoteRoomLocator),
    )
    .expect("state should initialize with rejecting locator");
    let document = state
        .rooms()
        .create_document(Some("Remote owner".to_owned()))
        .expect("document should be created");
    let app = build_app(&config, state).expect("app should build");
    let server = TestServer::new(app);

    let response = server
        .get(&format!("/api/documents/{}", document.id))
        .add_header(
            "Authorization",
            document_auth_header(document.access_token()).as_str(),
        )
        .await;

    response.assert_status(StatusCode::CONFLICT);
    response.assert_header(
        "x-collab-owner-node-id",
        format!("node-for-{}", document.id),
    );
    response.assert_header("x-collab-owner-base-url", "http://node-b.internal:4000");
    response.assert_header(
        "x-collab-redirect-location",
        format!("http://node-b.internal:4000/api/documents/{}", document.id),
    );
    response.assert_header(
        "location",
        format!("http://node-b.internal:4000/api/documents/{}", document.id),
    );

    let payload = response.json::<Value>();
    assert_eq!(payload["error"], "conflict");
    assert_eq!(
        payload["message"],
        format!(
            "document `{}` is owned by another collaboration node",
            document.id
        )
    );
    assert_eq!(
        payload["owner"]["node_id"],
        format!("node-for-{}", document.id)
    );
    assert_eq!(payload["owner"]["base_url"], "http://node-b.internal:4000");
}

#[tokio::test]
async fn document_detail_endpoint_rejects_non_local_file_room_owner() {
    let mut config = test_config();
    let coordinator_dir = temp_snapshot_dir("file-room-locator");
    config.room_locator = "file".to_owned();
    config.room_coordinator_state_dir = coordinator_dir.display().to_string();
    config.node_id = "node-a".to_owned();

    let state = AppState::from_config(&config).expect("state should initialize with file locator");
    let document = state
        .rooms()
        .create_document(Some("Remote file owner".to_owned()))
        .expect("document should be created");

    fs::write(
        coordinator_dir.join(format!("{}.json", document.id)),
        serde_json::to_vec(&serde_json::json!({
            "doc_id": document.id,
            "node_id": "node-b",
            "activated_at": "2026-04-20T00:00:00Z",
            "updated_at": "2026-04-20T00:00:00Z"
        }))
        .expect("file room state should serialize"),
    )
    .expect("file room state should be written");

    let app = build_app(&config, state).expect("app should build");
    let server = TestServer::new(app);

    let response = server
        .get(&format!("/api/documents/{}", document.id))
        .add_header(
            "Authorization",
            document_auth_header(document.access_token()).as_str(),
        )
        .await;

    response.assert_status(StatusCode::CONFLICT);

    let payload = response.json::<Value>();
    assert_eq!(payload["error"], "conflict");
    assert_eq!(
        payload["message"],
        format!(
            "document `{}` is owned by another collaboration node",
            document.id
        )
    );
    assert_eq!(payload["owner"]["node_id"], "node-b");
    assert!(payload["owner"]["base_url"].is_null());

    fs::remove_dir_all(coordinator_dir).expect("test state directory should be cleaned up");
}

#[tokio::test]
async fn document_detail_endpoint_includes_base_url_for_non_local_file_room_owner() {
    let mut config = test_config();
    let coordinator_dir = temp_snapshot_dir("file-room-locator-with-base-url");
    config.room_locator = "file".to_owned();
    config.room_coordinator_state_dir = coordinator_dir.display().to_string();
    config.node_id = "node-a".to_owned();

    let state = AppState::from_config(&config).expect("state should initialize with file locator");
    let document = state
        .rooms()
        .create_document(Some("Remote file owner with base url".to_owned()))
        .expect("document should be created");

    fs::write(
        coordinator_dir.join(format!("{}.json", document.id)),
        serde_json::to_vec(&serde_json::json!({
            "doc_id": document.id,
            "node_id": "node-b",
            "base_url": "http://node-b.internal:5001/",
            "activated_at": "2026-04-20T00:00:00Z",
            "updated_at": "2026-04-20T00:00:00Z"
        }))
        .expect("file room state should serialize"),
    )
    .expect("file room state should be written");

    let app = build_app(&config, state).expect("app should build");
    let server = TestServer::new(app);

    let response = server
        .get(&format!("/api/documents/{}", document.id))
        .add_header(
            "Authorization",
            document_auth_header(document.access_token()).as_str(),
        )
        .await;

    response.assert_status(StatusCode::CONFLICT);

    let payload = response.json::<Value>();
    assert_eq!(payload["error"], "conflict");
    assert_eq!(payload["owner"]["node_id"], "node-b");
    assert_eq!(payload["owner"]["base_url"], "http://node-b.internal:5001");

    fs::remove_dir_all(coordinator_dir).expect("test state directory should be cleaned up");
}

#[tokio::test]
async fn document_detail_endpoint_rejects_non_local_sqlite_room_owner() {
    let mut config = test_config();
    let coordinator_dir = temp_snapshot_dir("sqlite-room-locator");
    let sqlite_path = coordinator_dir.join("room-coordinator.sqlite3");
    config.room_locator = "sqlite".to_owned();
    config.room_coordinator_sqlite_path = sqlite_path.to_string_lossy().into_owned();
    config.node_id = "node-a".to_owned();

    let state =
        AppState::from_config(&config).expect("state should initialize with sqlite locator");
    let document = state
        .rooms()
        .create_document(Some("Remote sqlite owner".to_owned()))
        .expect("document should be created");

    let connection = rusqlite::Connection::open(&sqlite_path).expect("sqlite file should open");
    connection
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS room_leases (
                doc_id TEXT PRIMARY KEY,
                node_id TEXT NOT NULL,
                base_url TEXT,
                lease_id TEXT NOT NULL,
                epoch INTEGER NOT NULL,
                activated_at TEXT NOT NULL,
                renewed_at TEXT NOT NULL,
                expires_at TEXT NOT NULL
            );",
        )
        .expect("sqlite schema should initialize");
    let now = Utc::now();
    connection
        .execute(
            "INSERT INTO room_leases (
                doc_id,
                node_id,
                base_url,
                lease_id,
                epoch,
                activated_at,
                renewed_at,
                expires_at
            ) VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                document.id.to_string(),
                "node-b",
                Uuid::new_v4().to_string(),
                2_i64,
                now.to_rfc3339(),
                now.to_rfc3339(),
                (now + ChronoDuration::seconds(30)).to_rfc3339(),
            ],
        )
        .expect("sqlite room lease should be written");

    let app = build_app(&config, state).expect("app should build");
    let server = TestServer::new(app);

    let response = server
        .get(&format!("/api/documents/{}", document.id))
        .add_header(
            "Authorization",
            document_auth_header(document.access_token()).as_str(),
        )
        .await;

    response.assert_status(StatusCode::CONFLICT);

    let payload = response.json::<Value>();
    assert_eq!(payload["error"], "conflict");
    assert_eq!(payload["owner"]["node_id"], "node-b");
    assert!(payload["owner"]["base_url"].is_null());

    fs::remove_dir_all(coordinator_dir).expect("test sqlite directory should be cleaned up");
}

#[tokio::test]
async fn document_detail_endpoint_includes_base_url_for_non_local_sqlite_room_owner() {
    let mut config = test_config();
    let coordinator_dir = temp_snapshot_dir("sqlite-room-locator-with-base-url");
    let sqlite_path = coordinator_dir.join("room-coordinator.sqlite3");
    config.room_locator = "sqlite".to_owned();
    config.room_coordinator_sqlite_path = sqlite_path.to_string_lossy().into_owned();
    config.node_id = "node-a".to_owned();

    let state =
        AppState::from_config(&config).expect("state should initialize with sqlite locator");
    let document = state
        .rooms()
        .create_document(Some("Remote sqlite owner with base url".to_owned()))
        .expect("document should be created");

    let connection = rusqlite::Connection::open(&sqlite_path).expect("sqlite file should open");
    connection
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS room_leases (
                doc_id TEXT PRIMARY KEY,
                node_id TEXT NOT NULL,
                base_url TEXT,
                lease_id TEXT NOT NULL,
                epoch INTEGER NOT NULL,
                activated_at TEXT NOT NULL,
                renewed_at TEXT NOT NULL,
                expires_at TEXT NOT NULL
            );",
        )
        .expect("sqlite schema should initialize");
    let now = Utc::now();
    connection
        .execute(
            "INSERT INTO room_leases (
                doc_id,
                node_id,
                base_url,
                lease_id,
                epoch,
                activated_at,
                renewed_at,
                expires_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                document.id.to_string(),
                "node-b",
                "http://node-b.internal:5100/",
                Uuid::new_v4().to_string(),
                3_i64,
                now.to_rfc3339(),
                now.to_rfc3339(),
                (now + ChronoDuration::seconds(30)).to_rfc3339(),
            ],
        )
        .expect("sqlite room lease should be written");

    let app = build_app(&config, state).expect("app should build");
    let server = TestServer::new(app);

    let response = server
        .get(&format!("/api/documents/{}", document.id))
        .add_header(
            "Authorization",
            document_auth_header(document.access_token()).as_str(),
        )
        .await;

    response.assert_status(StatusCode::CONFLICT);
    response.assert_header("x-collab-owner-node-id", "node-b");
    response.assert_header("x-collab-owner-base-url", "http://node-b.internal:5100");
    response.assert_header(
        "x-collab-redirect-location",
        format!("http://node-b.internal:5100/api/documents/{}", document.id),
    );
    response.assert_header(
        "location",
        format!("http://node-b.internal:5100/api/documents/{}", document.id),
    );

    let payload = response.json::<Value>();
    assert_eq!(payload["error"], "conflict");
    assert_eq!(payload["owner"]["node_id"], "node-b");
    assert_eq!(payload["owner"]["base_url"], "http://node-b.internal:5100");

    fs::remove_dir_all(coordinator_dir).expect("test sqlite directory should be cleaned up");
}

#[tokio::test]
async fn document_detail_endpoint_allows_expired_file_room_owner_state() {
    let mut config = test_config();
    let coordinator_dir = temp_snapshot_dir("file-room-locator-expired");
    config.room_locator = "file".to_owned();
    config.room_coordinator_state_dir = coordinator_dir.display().to_string();
    config.node_id = "node-a".to_owned();

    let state = AppState::from_config(&config).expect("state should initialize with file locator");
    let document = state
        .rooms()
        .create_document(Some("Expired remote file owner".to_owned()))
        .expect("document should be created");

    fs::write(
        coordinator_dir.join(format!("{}.json", document.id)),
        serde_json::to_vec(&serde_json::json!({
            "doc_id": document.id,
            "node_id": "node-b",
            "lease_id": Uuid::new_v4(),
            "epoch": 2,
            "activated_at": (Utc::now() - ChronoDuration::seconds(5)).to_rfc3339(),
            "renewed_at": (Utc::now() - ChronoDuration::seconds(4)).to_rfc3339(),
            "expires_at": (Utc::now() - ChronoDuration::seconds(1)).to_rfc3339()
        }))
        .expect("file room state should serialize"),
    )
    .expect("file room state should be written");

    let app = build_app(&config, state).expect("app should build");
    let server = TestServer::new(app);

    let response = server
        .get(&format!("/api/documents/{}", document.id))
        .add_header(
            "Authorization",
            document_auth_header(document.access_token()).as_str(),
        )
        .await;

    response.assert_status_ok();

    let payload = response.json::<Value>();
    assert_eq!(payload["document"]["id"], document.id.to_string());

    fs::remove_dir_all(coordinator_dir).expect("test state directory should be cleaned up");
}

#[tokio::test]
async fn websocket_endpoint_accepts_document_connections() {
    let config = test_config();
    let app = build_app(
        &config,
        AppState::from_config(&config).expect("state should initialize"),
    )
    .expect("app should build");
    let server = TestServer::builder().http_transport().build(app);
    let create_response = server
        .post("/api/documents")
        .add_header("Authorization", admin_auth_header(&config).as_str())
        .json(&serde_json::json!({}))
        .await;
    create_response.assert_status(StatusCode::CREATED);

    let payload = create_response.json::<Value>();
    let doc_id = payload["document"]["id"]
        .as_str()
        .expect("created document id should be returned")
        .to_owned();
    let access_token = payload["credentials"]["access_token"]
        .as_str()
        .expect("document access token should be returned")
        .to_owned();

    let websocket = server
        .get_websocket(&format!("/ws/{doc_id}"))
        .add_header("Origin", config.frontend_origin.as_str())
        .add_header(
            "Authorization",
            document_auth_header(&access_token).as_str(),
        )
        .await
        .into_websocket()
        .await;

    websocket.close().await;
}

#[tokio::test]
async fn websocket_room_coordinator_tracks_first_and_last_session() {
    let config = test_config();
    let coordinator = Arc::new(RecordingRoomCoordinator::default());
    let state = AppState::with_snapshot_store_locator_and_coordinator(
        config.frontend_origin.clone(),
        config.api_token.clone(),
        Arc::new(InMemorySnapshotStore::new()),
        Arc::new(backend::collab::locator::LocalRoomLocator),
        coordinator.clone(),
    )
    .expect("state should initialize with recording coordinator");
    let document = state
        .rooms()
        .create_document(Some("Tracked room".to_owned()))
        .expect("document should be created");
    let app = build_app(&config, state).expect("app should build");
    let server = TestServer::builder().http_transport().build(app);

    let websocket_a = server
        .get_websocket(&format!("/ws/{}", document.id))
        .add_header("Origin", config.frontend_origin.as_str())
        .add_header(
            "Authorization",
            document_auth_header(document.access_token()).as_str(),
        )
        .await
        .into_websocket()
        .await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    assert_eq!(
        coordinator.snapshot(),
        vec![format!("activate:{}", document.id)]
    );

    let websocket_b = server
        .get_websocket(&format!("/ws/{}", document.id))
        .add_header("Origin", config.frontend_origin.as_str())
        .add_header(
            "Authorization",
            document_auth_header(document.access_token()).as_str(),
        )
        .await
        .into_websocket()
        .await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    assert_eq!(
        coordinator.snapshot(),
        vec![format!("activate:{}", document.id)]
    );

    websocket_a.close().await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    assert_eq!(
        coordinator.snapshot(),
        vec![format!("activate:{}", document.id)]
    );

    websocket_b.close().await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    assert_eq!(
        coordinator.snapshot(),
        vec![
            format!("activate:{}", document.id),
            format!("deactivate:{}", document.id),
        ]
    );
}

#[tokio::test]
async fn websocket_room_activation_failure_does_not_leak_active_sessions() {
    let config = test_config();
    let state = AppState::with_snapshot_store_locator_and_coordinator(
        config.frontend_origin.clone(),
        config.api_token.clone(),
        Arc::new(InMemorySnapshotStore::new()),
        Arc::new(backend::collab::locator::LocalRoomLocator),
        Arc::new(FailingRoomCoordinator),
    )
    .expect("state should initialize with failing coordinator");
    let document = state
        .rooms()
        .create_document(Some("Failed coordinator activation".to_owned()))
        .expect("document should be created");
    let app = build_app(&config, state.clone()).expect("app should build");
    let server = TestServer::builder().http_transport().build(app);

    let websocket = server
        .get_websocket(&format!("/ws/{}", document.id))
        .add_header("Origin", config.frontend_origin.as_str())
        .add_header(
            "Authorization",
            document_auth_header(document.access_token()).as_str(),
        )
        .await
        .into_websocket()
        .await;

    tokio::time::sleep(Duration::from_millis(50)).await;
    websocket.close().await;

    let room = state
        .rooms()
        .get_or_restore(&document.id)
        .expect("room lookup should succeed")
        .expect("room should remain recoverable after activation failure");
    assert_eq!(room.active_sessions(), 0);
}

#[tokio::test]
async fn document_detail_endpoint_rejects_invalid_uuid_with_json_error() {
    let config = test_config();
    let app = build_app(
        &config,
        AppState::from_config(&config).expect("state should initialize"),
    )
    .expect("app should build");
    let server = TestServer::new(app);

    let response = server.get("/api/documents/not-a-uuid").await;

    response.assert_status_bad_request();

    let payload = response.json::<Value>();
    assert_eq!(payload["error"], "bad_request");
    assert_eq!(
        payload["message"],
        "id must be a valid UUID, received `not-a-uuid`"
    );
}

#[tokio::test]
async fn websocket_endpoint_rejects_missing_origin_with_json_error() {
    let config = test_config();
    let app = build_app(
        &config,
        AppState::from_config(&config).expect("state should initialize"),
    )
    .expect("app should build");
    let server = TestServer::builder().http_transport().build(app);
    let doc_id = Uuid::new_v4();

    let response = server.get_websocket(&format!("/ws/{doc_id}")).await;

    response.assert_status_forbidden();

    let payload = response.json::<Value>();
    assert_eq!(payload["error"], "forbidden");
    assert_eq!(
        payload["message"],
        "Origin header is required for websocket connections"
    );
}

#[tokio::test]
async fn websocket_endpoint_rejects_disallowed_origin_with_json_error() {
    let config = test_config();
    let app = build_app(
        &config,
        AppState::from_config(&config).expect("state should initialize"),
    )
    .expect("app should build");
    let server = TestServer::builder().http_transport().build(app);
    let doc_id = Uuid::new_v4();

    let response = server
        .get_websocket(&format!("/ws/{doc_id}"))
        .add_header("Origin", "http://evil.example")
        .add_header("Authorization", "Bearer test-doc-token")
        .await;

    response.assert_status_forbidden();

    let payload = response.json::<Value>();
    assert_eq!(payload["error"], "forbidden");
    assert_eq!(
        payload["message"],
        "Origin `http://evil.example` is not allowed for websocket connections"
    );
}

#[tokio::test]
async fn websocket_endpoint_rejects_invalid_uuid_with_json_error() {
    let config = test_config();
    let app = build_app(
        &config,
        AppState::from_config(&config).expect("state should initialize"),
    )
    .expect("app should build");
    let server = TestServer::builder().http_transport().build(app);

    let response = server
        .get_websocket("/ws/not-a-uuid")
        .add_header("Origin", config.frontend_origin.as_str())
        .add_header("Authorization", "Bearer test-doc-token")
        .await;

    response.assert_status_bad_request();

    let payload = response.json::<Value>();
    assert_eq!(payload["error"], "bad_request");
    assert_eq!(
        payload["message"],
        "doc_id must be a valid UUID, received `not-a-uuid`"
    );
}

#[tokio::test]
async fn websocket_endpoint_rejects_missing_document_with_json_error() {
    let config = test_config();
    let app = build_app(
        &config,
        AppState::from_config(&config).expect("state should initialize"),
    )
    .expect("app should build");
    let server = TestServer::builder().http_transport().build(app);
    let doc_id = Uuid::new_v4();

    let response = server
        .get_websocket(&format!("/ws/{doc_id}"))
        .add_header("Origin", config.frontend_origin.as_str())
        .add_header("Authorization", "Bearer test-doc-token")
        .await;

    response.assert_status_not_found();

    let payload = response.json::<Value>();
    assert_eq!(payload["error"], "not_found");
    assert_eq!(
        payload["message"],
        format!("document `{doc_id}` was not found")
    );
}

#[tokio::test]
async fn websocket_endpoint_rejects_non_local_owner_with_redirect_headers() {
    let config = test_config();
    let state = AppState::with_snapshot_store_and_locator(
        config.frontend_origin.clone(),
        config.api_token.clone(),
        Arc::new(InMemorySnapshotStore::new()),
        Arc::new(RemoteRoomLocator),
    )
    .expect("state should initialize with rejecting locator");
    let document = state
        .rooms()
        .create_document(Some("Remote websocket owner".to_owned()))
        .expect("document should be created");
    let app = build_app(&config, state).expect("app should build");
    let server = TestServer::builder().http_transport().build(app);

    let response = server
        .get_websocket(&format!("/ws/{}?source=edge", document.id))
        .add_header("Origin", config.frontend_origin.as_str())
        .add_header(
            "Authorization",
            document_auth_header(document.access_token()).as_str(),
        )
        .await;

    response.assert_status(StatusCode::CONFLICT);
    response.assert_header(
        "x-collab-owner-node-id",
        format!("node-for-{}", document.id),
    );
    response.assert_header("x-collab-owner-base-url", "http://node-b.internal:4000");
    response.assert_header(
        "x-collab-redirect-location",
        format!("http://node-b.internal:4000/ws/{}?source=edge", document.id),
    );
    response.assert_header(
        "location",
        format!("http://node-b.internal:4000/ws/{}?source=edge", document.id),
    );

    let payload = response.json::<Value>();
    assert_eq!(payload["error"], "conflict");
    assert_eq!(
        payload["message"],
        format!(
            "document `{}` is owned by another collaboration node",
            document.id
        )
    );
    assert_eq!(
        payload["owner"]["node_id"],
        format!("node-for-{}", document.id)
    );
    assert_eq!(payload["owner"]["base_url"], "http://node-b.internal:4000");
}

#[tokio::test]
async fn documents_endpoint_rejects_missing_admin_token() {
    let config = test_config();
    let app = build_app(
        &config,
        AppState::from_config(&config).expect("state should initialize"),
    )
    .expect("app should build");
    let server = TestServer::new(app);

    let response = server.get("/api/documents").await;

    response.assert_status(StatusCode::UNAUTHORIZED);

    let payload = response.json::<Value>();
    assert_eq!(payload["error"], "unauthorized");
    assert_eq!(payload["message"], "Authorization header is required");
}

#[tokio::test]
async fn document_detail_endpoint_rejects_invalid_document_token() {
    let config = test_config();
    let app = build_app(
        &config,
        AppState::from_config(&config).expect("state should initialize"),
    )
    .expect("app should build");
    let server = TestServer::new(app);

    let create_response = server
        .post("/api/documents")
        .add_header("Authorization", admin_auth_header(&config).as_str())
        .json(&serde_json::json!({
            "title": "Restricted"
        }))
        .await;
    create_response.assert_status(StatusCode::CREATED);

    let payload = create_response.json::<Value>();
    let created_id = payload["document"]["id"]
        .as_str()
        .expect("created document id should be returned");

    let response = server
        .get(&format!("/api/documents/{created_id}"))
        .add_header("Authorization", "Bearer invalid-doc-token")
        .await;

    response.assert_status_forbidden();

    let payload = response.json::<Value>();
    assert_eq!(payload["error"], "forbidden");
    assert_eq!(
        payload["message"],
        format!("provided token does not grant access to document `{created_id}`")
    );
}

#[tokio::test]
async fn websocket_endpoint_rejects_missing_document_token() {
    let config = test_config();
    let app = build_app(
        &config,
        AppState::from_config(&config).expect("state should initialize"),
    )
    .expect("app should build");
    let server = TestServer::builder().http_transport().build(app);
    let create_response = server
        .post("/api/documents")
        .add_header("Authorization", admin_auth_header(&config).as_str())
        .json(&serde_json::json!({}))
        .await;
    create_response.assert_status(StatusCode::CREATED);

    let payload = create_response.json::<Value>();
    let doc_id = payload["document"]["id"]
        .as_str()
        .expect("created document id should be returned")
        .to_owned();

    let response = server
        .get_websocket(&format!("/ws/{doc_id}"))
        .add_header("Origin", config.frontend_origin.as_str())
        .await;

    response.assert_status(StatusCode::UNAUTHORIZED);

    let payload = response.json::<Value>();
    assert_eq!(payload["error"], "unauthorized");
    assert_eq!(payload["message"], "Authorization header is required");
}

#[tokio::test]
async fn websocket_endpoint_supports_yrs_sync_handshake_and_update_broadcast() {
    let config = test_config();
    let state = AppState::from_config(&config).expect("state should initialize");
    let document = state
        .rooms()
        .create_document(Some("Provider compatibility".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have an active room");
    {
        let server_doc = room.awareness().write().await.doc().clone();
        let text = server_doc.get_or_insert_text("content");
        let mut txn = server_doc.transact_mut();
        text.insert(&mut txn, 0, "seed");
    }

    let app = build_app(&config, state).expect("app should build");
    let server = TestServer::builder().http_transport().build(app);

    let mut first_client = server
        .get_websocket(&format!("/ws/{}", document.id))
        .add_header("Origin", config.frontend_origin.as_str())
        .add_header(
            "Authorization",
            document_auth_header(document.access_token()).as_str(),
        )
        .await
        .into_websocket()
        .await;

    first_client
        .send_message(WsMessage::Binary(
            Message::Sync(SyncMessage::SyncStep1(StateVector::default()))
                .encode_v1()
                .into(),
        ))
        .await;

    let sync_reply = decode_sync_message(first_client.receive_bytes().await);
    let SyncMessage::SyncStep2(update) = sync_reply else {
        panic!("expected SyncStep2 during initial handshake");
    };

    let first_client_doc = Doc::new();
    let first_client_text = first_client_doc.get_or_insert_text("content");
    let mut first_client_txn = first_client_doc.transact_mut();
    first_client_txn
        .apply_update(Update::decode_v1(update.as_slice()).expect("sync payload should decode"));
    drop(first_client_txn);
    assert_eq!(
        first_client_text.get_string(&first_client_doc.transact()),
        "seed"
    );

    let mut second_client = server
        .get_websocket(&format!("/ws/{}", document.id))
        .add_header("Origin", config.frontend_origin.as_str())
        .add_header(
            "Authorization",
            document_auth_header(document.access_token()).as_str(),
        )
        .await
        .into_websocket()
        .await;

    second_client
        .send_message(WsMessage::Binary(
            Message::Sync(SyncMessage::SyncStep1(StateVector::default()))
                .encode_v1()
                .into(),
        ))
        .await;
    let second_sync_reply = decode_sync_message(second_client.receive_bytes().await);
    let SyncMessage::SyncStep2(second_initial_update) = second_sync_reply else {
        panic!("expected SyncStep2 during second client handshake");
    };
    let second_client_doc = Doc::new();
    let second_client_text = second_client_doc.get_or_insert_text("content");
    let mut second_client_txn = second_client_doc.transact_mut();
    second_client_txn.apply_update(
        Update::decode_v1(second_initial_update.as_slice())
            .expect("second sync payload should decode"),
    );
    drop(second_client_txn);

    let mut update_txn = first_client_doc.transact_mut();
    first_client_text.insert(&mut update_txn, 4, " + provider");
    let client_update = update_txn.encode_update_v1();
    drop(update_txn);

    first_client
        .send_message(WsMessage::Binary(
            Message::Sync(SyncMessage::Update(client_update.clone()))
                .encode_v1()
                .into(),
        ))
        .await;

    let broadcast = decode_sync_message(second_client.receive_bytes().await);
    let SyncMessage::Update(update) = broadcast else {
        panic!("expected broadcast update for subscribed client");
    };
    let mut second_client_txn = second_client_doc.transact_mut();
    second_client_txn
        .apply_update(Update::decode_v1(update.as_slice()).expect("update payload should decode"));
    drop(second_client_txn);
    assert_eq!(
        second_client_text.get_string(&second_client_doc.transact()),
        "seed + provider"
    );

    first_client.close().await;
    second_client.close().await;
}

#[tokio::test]
async fn websocket_endpoint_restores_latest_sqlite_snapshot_after_owner_handoff() {
    let shared_root = temp_snapshot_dir("sqlite-owner-handoff");

    let mut node_a_config = test_config();
    configure_shared_sqlite_collaboration(
        &mut node_a_config,
        &shared_root,
        "node-a",
        "http://node-a.internal:4300/",
    );

    let mut node_b_config = test_config();
    configure_shared_sqlite_collaboration(
        &mut node_b_config,
        &shared_root,
        "node-b",
        "http://node-b.internal:4301/",
    );

    let node_a_state =
        AppState::from_config(&node_a_config).expect("node-a state should initialize");
    let node_a_app =
        build_app(&node_a_config, node_a_state.clone()).expect("node-a app should build");
    let node_a_server = TestServer::builder().http_transport().build(node_a_app);

    let create_response = node_a_server
        .post("/api/documents")
        .add_header("Authorization", admin_auth_header(&node_a_config).as_str())
        .json(&serde_json::json!({
            "title": "Handoff document"
        }))
        .await;
    create_response.assert_status(StatusCode::CREATED);

    let payload = create_response.json::<Value>();
    let doc_id = payload["document"]["id"]
        .as_str()
        .expect("created document id should be returned")
        .to_owned();
    let doc_uuid = Uuid::parse_str(&doc_id).expect("created document id should be a UUID");
    let access_token = payload["credentials"]["access_token"]
        .as_str()
        .expect("document access token should be returned")
        .to_owned();

    let mut node_a_client = node_a_server
        .get_websocket(&format!("/ws/{doc_id}"))
        .add_header("Origin", node_a_config.frontend_origin.as_str())
        .add_header(
            "Authorization",
            document_auth_header(&access_token).as_str(),
        )
        .await
        .into_websocket()
        .await;

    node_a_client
        .send_message(WsMessage::Binary(
            Message::Sync(SyncMessage::SyncStep1(StateVector::default()))
                .encode_v1()
                .into(),
        ))
        .await;
    let initial_sync = decode_sync_message(node_a_client.receive_bytes().await);
    let SyncMessage::SyncStep2(initial_update) = initial_sync else {
        panic!("expected SyncStep2 during initial node-a handshake");
    };

    let node_a_doc = Doc::new();
    let node_a_text = node_a_doc.get_or_insert_text("content");
    let mut node_a_txn = node_a_doc.transact_mut();
    node_a_txn.apply_update(
        Update::decode_v1(initial_update.as_slice()).expect("initial sync payload should decode"),
    );
    node_a_text.insert(&mut node_a_txn, 0, "hello handoff");
    let client_update = node_a_txn.encode_update_v1();
    drop(node_a_txn);

    node_a_client
        .send_message(WsMessage::Binary(
            Message::Sync(SyncMessage::Update(client_update))
                .encode_v1()
                .into(),
        ))
        .await;

    let node_b_state =
        AppState::from_config(&node_b_config).expect("node-b state should initialize");
    assert!(
        node_b_state.rooms().get(&doc_uuid).is_none(),
        "distributed sqlite mode should not eagerly hydrate rooms on startup"
    );

    let node_b_app =
        build_app(&node_b_config, node_b_state.clone()).expect("node-b app should build");
    let node_b_server = TestServer::builder().http_transport().build(node_b_app);

    let standby_response = node_b_server
        .get(&format!("/api/documents/{doc_id}?probe=standby"))
        .add_header(
            "Authorization",
            document_auth_header(&access_token).as_str(),
        )
        .await;
    standby_response.assert_status(StatusCode::CONFLICT);
    standby_response.assert_header("x-collab-owner-node-id", "node-a");
    standby_response.assert_header("x-collab-owner-base-url", "http://node-a.internal:4300");
    standby_response.assert_header(
        "x-collab-redirect-location",
        format!("http://node-a.internal:4300/api/documents/{doc_id}?probe=standby"),
    );

    node_a_client.close().await;
    let lease_path = shared_root.join("room-coordinator.sqlite3");
    wait_for_sqlite_room_lease_release(&lease_path, doc_uuid).await;

    let detail_response = {
        let mut last_status = None;
        let mut response = None;

        for _ in 0..100 {
            let next_response = node_b_server
                .get(&format!("/api/documents/{doc_id}"))
                .add_header(
                    "Authorization",
                    document_auth_header(&access_token).as_str(),
                )
                .await;
            let status = next_response.status_code();
            if status == StatusCode::OK {
                response = Some(next_response);
                break;
            }

            last_status = Some(status);
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        response.unwrap_or_else(|| {
            panic!(
                "node-b detail restore should become available after managed handoff, last status was {:?}",
                last_status
            )
        })
    };
    detail_response.assert_status_ok();

    let mut node_b_client = node_b_server
        .get_websocket(&format!("/ws/{doc_id}"))
        .add_header("Origin", node_b_config.frontend_origin.as_str())
        .add_header(
            "Authorization",
            document_auth_header(&access_token).as_str(),
        )
        .await
        .into_websocket()
        .await;

    node_b_client
        .send_message(WsMessage::Binary(
            Message::Sync(SyncMessage::SyncStep1(StateVector::default()))
                .encode_v1()
                .into(),
        ))
        .await;
    let handoff_sync = decode_sync_message(node_b_client.receive_bytes().await);
    let SyncMessage::SyncStep2(handoff_update) = handoff_sync else {
        panic!("expected SyncStep2 during node-b handoff handshake");
    };

    let node_b_doc = Doc::new();
    let node_b_text = node_b_doc.get_or_insert_text("content");
    let mut node_b_txn = node_b_doc.transact_mut();
    node_b_txn.apply_update(
        Update::decode_v1(handoff_update.as_slice()).expect("handoff sync payload should decode"),
    );
    drop(node_b_txn);

    assert_eq!(
        node_b_text.get_string(&node_b_doc.transact()),
        "hello handoff"
    );

    node_b_client.close().await;
    fs::remove_dir_all(shared_root).expect("shared sqlite handoff directory should be cleaned up");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn document_detail_restores_latest_sqlite_snapshot_after_managed_owner_handoff() {
    let shared_root = temp_snapshot_dir("managed-owner-handoff");
    let harness = spawn_mock_managed_coordination_service(Some("managed-secret")).await;

    let mut node_a_config = test_config();
    configure_managed_coordination_with_shared_sqlite_snapshots(
        &mut node_a_config,
        &shared_root,
        "node-a",
        "http://node-a.internal:4300/",
        harness.base_url.as_str(),
        Some("managed-secret"),
    );

    let mut node_b_config = test_config();
    configure_managed_coordination_with_shared_sqlite_snapshots(
        &mut node_b_config,
        &shared_root,
        "node-b",
        "http://node-b.internal:4301/",
        harness.base_url.as_str(),
        Some("managed-secret"),
    );

    let node_a_state =
        AppState::from_config(&node_a_config).expect("node-a state should initialize");
    let node_a_app =
        build_app(&node_a_config, node_a_state.clone()).expect("node-a app should build");
    let node_a_server = TestServer::builder().http_transport().build(node_a_app);

    let create_response = node_a_server
        .post("/api/documents")
        .add_header("Authorization", admin_auth_header(&node_a_config).as_str())
        .json(&serde_json::json!({
            "title": "Managed handoff document"
        }))
        .await;
    create_response.assert_status(StatusCode::CREATED);

    let payload = create_response.json::<Value>();
    let doc_id = payload["document"]["id"]
        .as_str()
        .expect("created document id should be returned")
        .to_owned();
    let doc_uuid = Uuid::parse_str(&doc_id).expect("created document id should be a UUID");
    let access_token = payload["credentials"]["access_token"]
        .as_str()
        .expect("document access token should be returned")
        .to_owned();

    let node_a_room = node_a_state
        .rooms()
        .get(&doc_uuid)
        .expect("created document should have an active room");
    {
        let node_a_doc = node_a_room.awareness().write().await.doc().clone();
        let node_a_text = node_a_doc.get_or_insert_text("content");
        let mut node_a_txn = node_a_doc.transact_mut();
        node_a_text.insert(&mut node_a_txn, 0, "hello managed handoff");
    }

    assert_eq!(node_a_room.start_session(), 1);
    node_a_state
        .room_coordinator()
        .room_activated(&doc_uuid)
        .expect("node-a should acquire the managed lease");

    wait_for_managed_room_lease_owner(&harness.state, doc_uuid, "node-a").await;

    let node_b_state =
        AppState::from_config(&node_b_config).expect("node-b state should initialize");
    assert!(
        node_b_state.rooms().get(&doc_uuid).is_none(),
        "distributed managed mode should not eagerly hydrate rooms on startup"
    );

    let node_b_app =
        build_app(&node_b_config, node_b_state.clone()).expect("node-b app should build");
    let node_b_server = TestServer::builder().http_transport().build(node_b_app);

    let standby_response = {
        let mut standby_response = None;
        let mut last_status = None;

        for _ in 0..100 {
            let response = node_b_server
                .get(&format!("/api/documents/{doc_id}?probe=managed-standby"))
                .add_header(
                    "Authorization",
                    document_auth_header(&access_token).as_str(),
                )
                .await;
            let status = response.status_code();
            if status == StatusCode::CONFLICT {
                standby_response = Some(response);
                break;
            }

            last_status = Some(status);
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        standby_response.unwrap_or_else(|| {
            panic!(
                "node-b standby detail should observe managed remote owner, last status was {:?}",
                last_status
            )
        })
    };
    standby_response.assert_status(StatusCode::CONFLICT);
    standby_response.assert_header("x-collab-owner-node-id", "node-a");
    standby_response.assert_header("x-collab-owner-base-url", "http://node-a.internal:4300");
    standby_response.assert_header(
        "x-collab-redirect-location",
        format!("http://node-a.internal:4300/api/documents/{doc_id}?probe=managed-standby"),
    );

    let teardown = node_a_state
        .rooms()
        .persist_and_evict_if_idle(&doc_uuid, &node_a_room)
        .expect("node-a should persist the sqlite snapshot before handoff");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);
    node_a_state
        .room_coordinator()
        .room_deactivated(&doc_uuid)
        .expect("node-a should release the managed lease after persisting");
    wait_for_managed_room_lease_release(&harness.state, doc_uuid).await;

    let detail_response = {
        let mut detail_response = None;
        let mut last_status = None;

        for _ in 0..100 {
            let response = node_b_server
                .get(&format!("/api/documents/{doc_id}"))
                .add_header(
                    "Authorization",
                    document_auth_header(&access_token).as_str(),
                )
                .await;
            let status = response.status_code();
            if status == StatusCode::OK {
                detail_response = Some(response);
                break;
            }

            last_status = Some(status);
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        detail_response.unwrap_or_else(|| {
            panic!(
                "node-b detail restore should become available after managed handoff, last status was {:?}",
                last_status
            )
        })
    };
    detail_response.assert_status_ok();

    let restored_room = node_b_state
        .rooms()
        .get(&doc_uuid)
        .expect("detail restore should hydrate the room on node-b");
    let node_b_doc = Doc::new();
    let node_b_text = node_b_doc.get_or_insert_text("content");
    let restored_snapshot = restored_room
        .snapshot()
        .expect("restored room should snapshot after managed handoff");
    let mut restored_txn = node_b_doc.transact_mut();
    restored_txn.apply_update(
        Update::decode_v1(restored_snapshot.update.as_slice())
            .expect("managed handoff snapshot should decode"),
    );
    drop(restored_txn);

    assert_eq!(
        node_b_text.get_string(&node_b_doc.transact()),
        "hello managed handoff"
    );

    fs::remove_dir_all(shared_root).expect("managed handoff directory should be cleaned up");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn app_state_restores_latest_managed_snapshot_after_managed_owner_handoff() {
    let harness = spawn_mock_managed_coordination_service(Some("managed-secret")).await;

    let mut node_a_config = test_config();
    configure_managed_coordination_with_managed_snapshots(
        &mut node_a_config,
        "node-a",
        "http://node-a.internal:4300/",
        harness.base_url.as_str(),
        harness.snapshot_base_url.as_str(),
        Some("managed-secret"),
    );

    let mut node_b_config = test_config();
    configure_managed_coordination_with_managed_snapshots(
        &mut node_b_config,
        "node-b",
        "http://node-b.internal:4301/",
        harness.base_url.as_str(),
        harness.snapshot_base_url.as_str(),
        Some("managed-secret"),
    );

    let node_a_state =
        AppState::from_config(&node_a_config).expect("node-a state should initialize");
    let document = node_a_state
        .rooms()
        .create_document(Some("Managed durability handoff document".to_owned()))
        .expect("document should be created");
    let doc_uuid = document.id;
    let node_a_room = node_a_state
        .rooms()
        .get(&doc_uuid)
        .expect("created document should have an active room");
    {
        let node_a_doc = node_a_room.awareness().write().await.doc().clone();
        let node_a_text = node_a_doc.get_or_insert_text("content");
        let mut node_a_txn = node_a_doc.transact_mut();
        node_a_text.insert(&mut node_a_txn, 0, "hello managed durability handoff");
    }

    assert_eq!(node_a_room.start_session(), 1);
    node_a_state
        .room_coordinator()
        .room_activated(&doc_uuid)
        .expect("node-a should acquire the managed lease");

    wait_for_managed_room_lease_owner(&harness.state, doc_uuid, "node-a").await;

    let node_b_state =
        AppState::from_config(&node_b_config).expect("node-b state should initialize");
    assert!(
        node_b_state.rooms().get(&doc_uuid).is_none(),
        "distributed managed mode should not eagerly hydrate rooms on startup"
    );
    let listed_documents = node_b_state
        .rooms()
        .list_documents()
        .expect("managed snapshot catalog should load while the room stays cold");
    assert_eq!(listed_documents.len(), 1);
    assert_eq!(listed_documents[0].id, doc_uuid);

    let error = node_b_state
        .ensure_local_room_owner(&doc_uuid)
        .expect_err("node-b should observe node-a as the active managed owner");
    match error {
        AppError::RemoteOwner {
            owner_node_id,
            owner_base_url,
            ..
        } => {
            assert_eq!(owner_node_id, "node-a");
            assert_eq!(
                owner_base_url.as_deref(),
                Some("http://node-a.internal:4300")
            );
        }
        other => panic!("expected remote owner error, received {other:?}"),
    }

    let teardown = node_a_state
        .rooms()
        .persist_and_evict_if_idle(&doc_uuid, &node_a_room)
        .expect("node-a should persist the managed snapshot before handoff");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);
    node_a_state
        .room_coordinator()
        .room_deactivated(&doc_uuid)
        .expect("node-a should release the managed lease after persisting");
    wait_for_managed_room_lease_release(&harness.state, doc_uuid).await;

    node_b_state
        .ensure_local_room_owner(&doc_uuid)
        .expect("node-b should resolve locally after the managed lease is released");

    let restored_room = node_b_state
        .rooms()
        .get_or_restore(&doc_uuid)
        .expect("node-b restore should query the managed snapshot store")
        .expect("managed snapshot should restore after owner handoff");
    let node_b_doc = Doc::new();
    let node_b_text = node_b_doc.get_or_insert_text("content");
    let restored_snapshot = restored_room
        .snapshot()
        .expect("restored room should snapshot after managed-managed handoff");
    let mut node_b_txn = node_b_doc.transact_mut();
    node_b_txn.apply_update(
        Update::decode_v1(restored_snapshot.update.as_slice())
            .expect("managed-managed handoff snapshot should decode"),
    );
    drop(node_b_txn);

    assert_eq!(
        node_b_text.get_string(&node_b_doc.transact()),
        "hello managed durability handoff"
    );
}

#[tokio::test]
async fn websocket_endpoint_rejects_invalid_awareness_payload_updates() {
    let config = test_config();
    let state = AppState::from_config(&config).expect("state should initialize");
    let document = state
        .rooms()
        .create_document(Some("Awareness validation".to_owned()))
        .expect("document should be created");

    let app = build_app(&config, state.clone()).expect("app should build");
    let server = TestServer::builder().http_transport().build(app);

    let mut client = server
        .get_websocket(&format!("/ws/{}", document.id))
        .add_header("Origin", config.frontend_origin.as_str())
        .add_header(
            "Authorization",
            document_auth_header(document.access_token()).as_str(),
        )
        .await
        .into_websocket()
        .await;

    let invalid_awareness = AwarenessUpdate {
        clients: HashMap::from([(
            7,
            AwarenessUpdateEntry {
                clock: 1,
                json: r#"{"user":{"id":"user-7","name":"Kim","color":"blue"},"client":{"id":"session-3","kind":"editor"}}"#
                    .to_owned(),
            },
        )]),
    };

    client
        .send_message(WsMessage::Binary(
            Message::Awareness(invalid_awareness).encode_v1().into(),
        ))
        .await;

    tokio::time::sleep(Duration::from_millis(50)).await;

    let room = state
        .rooms()
        .get_or_restore(&document.id)
        .expect("room lookup should succeed")
        .expect("document room should restore after the invalid update path");
    let awareness_ref = room.awareness();
    let awareness = awareness_ref.read().await;

    assert!(!awareness.clients().contains_key(&7));

    client.close().await;
}

#[tokio::test]
async fn app_state_hydrates_snapshot_backed_rooms_on_startup() {
    let config = test_config();
    let snapshot_store: Arc<dyn SnapshotStore> = Arc::new(InMemorySnapshotStore::new());
    let bootstrap_registry = RoomRegistry::new(snapshot_store.clone());
    let document = bootstrap_registry
        .create_document(Some("Hydrated at startup".to_owned()))
        .expect("document should be created");
    let room = bootstrap_registry
        .get(&document.id)
        .expect("created document should have an active room");

    {
        let server_doc = room.awareness().write().await.doc().clone();
        let text = server_doc.get_or_insert_text("content");
        let mut txn = server_doc.transact_mut();
        text.insert(&mut txn, 0, "restored");
    }

    assert_eq!(room.start_session(), 1);
    let teardown = bootstrap_registry
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("idle room eviction should succeed");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);
    assert!(bootstrap_registry.get(&document.id).is_none());

    let state = AppState::with_snapshot_store(
        config.frontend_origin.clone(),
        config.api_token.clone(),
        snapshot_store,
    )
    .expect("state should hydrate rooms from snapshot store");

    let hydrated_room = state
        .rooms()
        .get(&document.id)
        .expect("room should be present after startup hydration");
    let hydrated_doc = hydrated_room.awareness().read().await.doc().clone();
    let hydrated_text = hydrated_doc.get_or_insert_text("content");

    assert_eq!(
        hydrated_text.get_string(&hydrated_doc.transact()),
        "restored"
    );
}

#[tokio::test]
async fn app_state_uses_file_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("file-store");
    config.snapshot_store = "file".to_owned();
    config.snapshot_dir = snapshot_dir.to_string_lossy().into_owned();

    let state = AppState::from_config(&config).expect("state should initialize with file store");
    let document = state
        .rooms()
        .create_document(Some("Persisted to disk".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to disk on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted file snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_dir.join(format!("{}.json", document.id)).exists());

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[tokio::test]
async fn app_state_uses_sqlite_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("sqlite-store-config");
    let snapshot_path = snapshot_dir.join("snapshots.sqlite3");
    config.snapshot_store = "sqlite".to_owned();
    config.snapshot_sqlite_path = snapshot_path.to_string_lossy().into_owned();

    let state = AppState::from_config(&config).expect("state should initialize with sqlite store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to sqlite".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to sqlite on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted sqlite snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_jammdb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("jammdb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.jammdb");
    configure_jammdb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with jammdb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to jammdb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to jammdb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted jammdb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_janql_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("janql-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.janql");
    configure_janql_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with janql store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to janql".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to janql on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted janql snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.join("wal.log").exists() || snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_jasondb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("jasondb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.jasondb");
    configure_jasondb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with jasondb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to jasondb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to jasondb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted jasondb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_jasonisnthappy_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("jasonisnthappy-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.jasonisnthappy");
    configure_jasonisnthappy_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with jasonisnthappy store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to jasonisnthappy".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to jasonisnthappy on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state = AppState::from_config(&config)
        .expect("state should reload persisted jasonisnthappy snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_datastack_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("datastack-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.datastack");
    configure_datastack_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with datastack store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to datastack".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to datastack on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted datastack snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_crystal_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("crystal-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.crystal");
    configure_crystal_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with crystal store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to crystal".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to crystal on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted crystal snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_assystem_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("assystem-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.assystem");
    configure_assystem_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with assystem store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to assystem".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to assystem on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted assystem snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_colon_db_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("colon-db-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.colon_db");
    configure_colon_db_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with colon_db store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to colon_db".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to colon_db on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted colon_db snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_mace_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("mace-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.mace");
    configure_mace_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with mace store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to mace".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to mace on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted mace snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_heed_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("heed-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.heed");
    configure_heed_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with heed store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to heed".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to heed on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted heed snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_hightower_kv_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("hightower-kv-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.hightower_kv");
    configure_hightower_kv_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with hightower_kv store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to hightower_kv".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to hightower_kv on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state = AppState::from_config(&config)
        .expect("state should reload persisted hightower_kv snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_fjall_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("fjall-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.fjall");
    configure_fjall_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with fjall store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to fjall".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to fjall on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted fjall snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_persy_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("persy-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.persy");
    configure_persy_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with persy store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to persy".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to persy on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted persy snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_rejects_managed_snapshot_store_without_base_url() {
    let mut config = test_config();
    config.snapshot_store = "managed".to_owned();

    let error = match AppState::from_config(&config) {
        Ok(_) => panic!("managed snapshot store should require base url"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("SNAPSHOT_MANAGED_BASE_URL is required when SNAPSHOT_STORE=managed"),
        "unexpected error: {error}"
    );
}

#[test]
fn app_state_uses_agdb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("agdb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.agdb");
    configure_agdb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with agdb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to agdb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to agdb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted agdb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_amandine_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("amandine-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.amandine");
    configure_amandine_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with amandine store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to amandine".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to amandine on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted amandine snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.join("snapshots.json").exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_armdb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("armdb-store-config");
    let snapshot_path = snapshot_dir.join("snapshots.armdb");
    config.snapshot_store = "armdb".to_owned();
    config.snapshot_armdb_path = snapshot_path.to_string_lossy().into_owned();

    let state = AppState::from_config(&config).expect("state should initialize with armdb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to armdb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to armdb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted armdb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_redb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("redb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.redb");
    configure_redb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with redb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to redb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to redb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted redb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_sled_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("sled-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.sled");
    configure_sled_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with sled store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to sled".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to sled on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted sled snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_pickledb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("pickledb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.pickledb");
    configure_pickledb_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with pickledb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to pickledb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to pickledb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted pickledb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_microkv_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("microkv-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots_microkv.kv");
    configure_microkv_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with microkv store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to microkv".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to microkv on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted microkv snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_rustbreak_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("rustbreak-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.rustbreak");
    configure_rustbreak_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with rustbreak store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to rustbreak".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to rustbreak on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted rustbreak snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_yedb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("yedb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.yedb");
    configure_yedb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with yedb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to yedb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to yedb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted yedb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_btree_store_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("btree-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.btree_store");
    configure_btree_store_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config)
        .expect("state should initialize with btree_store snapshot store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to btree_store".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to btree_store on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted btree_store snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_siamesedb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("siamesedb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.siamesedb");
    configure_siamesedb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config)
        .expect("state should initialize with siamesedb snapshot store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to siamesedb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to siamesedb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted siamesedb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_readb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("readb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.readb");
    configure_readb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with readb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to readb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to readb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted readb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_rskey_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("rskey-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.rskey");
    configure_rskey_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with rskey store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to rskey".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to rskey on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted rskey snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_rustlite_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("rustlite-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.rustlite");
    configure_rustlite_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with rustlite store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to rustlite".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to rustlite on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted rustlite snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_canopydb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("canopydb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.canopydb");
    configure_canopydb_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with canopydb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to canopydb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to canopydb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted canopydb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_structsy_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("structsy-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.structsy");
    configure_structsy_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with structsy store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to structsy".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to structsy on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted structsy snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_abyssiniandb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("abyssiniandb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.abyssiniandb");
    configure_abyssiniandb_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with abyssiniandb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to abyssiniandb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to abyssiniandb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state = AppState::from_config(&config)
        .expect("state should reload persisted abyssiniandb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_ckydb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("ckydb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.ckydb");
    configure_ckydb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with ckydb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to ckydb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to ckydb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted ckydb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_crepedb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("crepedb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.crepedb");
    configure_crepedb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with crepedb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to crepedb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to crepedb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted crepedb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_scdb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("scdb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.scdb");
    configure_scdb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with scdb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to scdb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to scdb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted scdb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_skv_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("skv-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    configure_skv_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with skv store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to skv".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to skv on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted skv snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_dir.join("snapshots.skv.data").exists());
    assert!(snapshot_dir.join("snapshots.skv.index").exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_surrealkv_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("surrealkv-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.surrealkv");
    configure_surrealkv_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with surrealkv store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to surrealkv".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to surrealkv on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted surrealkv snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_thunderdb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("thunderdb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.thunderdb");
    configure_thunderdb_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with thunderdb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to thunderdb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to thunderdb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted thunderdb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_tinybase_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("tinybase-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.tinybase");
    configure_tinybase_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with tinybase store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to tinybase".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to tinybase on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted tinybase snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_tinydb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("tinydb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.tinydb");
    configure_tinydb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with tinydb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to tinydb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to tinydb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted tinydb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_dblite_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("dblite-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.dblite");
    configure_dblite_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with dblite store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to dblite".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to dblite on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted dblite snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_aeternusdb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("aeternusdb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.aeternusdb");
    configure_aeternusdb_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with aeternusdb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to aeternusdb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to aeternusdb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted aeternusdb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_dbless_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("dbless-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.dbless");
    configure_dbless_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with dbless store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to dbless".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to dbless on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted dbless snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_sanakirja_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("sanakirja-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.sanakirja");
    configure_sanakirja_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with sanakirja store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to sanakirja".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to sanakirja on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted sanakirja snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_flash_kv_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("flash-kv-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.flash_kv");
    configure_flash_kv_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with flash_kv store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to flash_kv".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to flash_kv on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted flash_kv snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_highlandcows_isam_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("highlandcows-isam-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.highlandcows_isam");
    configure_highlandcows_isam_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config)
        .expect("state should initialize with highlandcows-isam store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to highlandcows-isam".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to highlandcows-isam on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state = AppState::from_config(&config)
        .expect("state should reload persisted highlandcows-isam snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.with_extension("idb").exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_simple_db_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("simple-db-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.simple_db");
    configure_simple_db_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with simple_db store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to simple_db".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to simple_db on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted simple_db snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_snaildb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("snaildb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.snaildb");
    configure_snaildb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with snaildb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to snaildb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to snaildb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted snaildb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_docdb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("docdb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.docdb.json");
    configure_docdb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with docdb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to docdb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to docdb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted docdb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_shorterdb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("shorterdb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.shorterdb");
    configure_shorterdb_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with shorterdb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to shorterdb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to shorterdb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted shorterdb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_tinykv_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("tinykv-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.tinykv.json");
    configure_tinykv_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with tinykv store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to tinykv".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to tinykv on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted tinykv snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_yakv_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("yakv-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.yakv");
    configure_yakv_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with yakv store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to yakv".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to yakv on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted yakv snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_yakvdb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("yakvdb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.yakvdb");
    configure_yakvdb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with yakvdb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to yakvdb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to yakvdb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted yakvdb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_rustcask_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("rustcask-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.rustcask");
    configure_rustcask_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with rustcask store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to rustcask".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to rustcask on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted rustcask snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_rusty_leveldb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("rusty-leveldb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.rusty_leveldb");
    configure_rusty_leveldb_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with rusty-leveldb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to rusty-leveldb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to rusty-leveldb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state = AppState::from_config(&config)
        .expect("state should reload persisted rusty-leveldb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_saberdb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("saberdb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.saberdb.json");
    configure_saberdb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with saberdb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to saberdb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to saberdb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted saberdb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_smolldb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("smolldb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.smolldb");
    configure_smolldb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with smolldb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to smolldb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to smolldb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted smolldb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_kstone_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("kstone-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.kstone");
    configure_kstone_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with kstone store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to kstone".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to kstone on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted kstone snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_ghaladb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("ghaladb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.ghaladb");
    configure_ghaladb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with ghaladb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to ghaladb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to ghaladb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted ghaladb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_apex_store_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("apex-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.apex_store");
    configure_apex_store_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with apex_store store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to ApexStore".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to apex_store on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted apex_store snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_roughdb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("roughdb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.roughdb");
    configure_roughdb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with roughdb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to roughdb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to roughdb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted roughdb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_raindb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("raindb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.raindb");
    configure_raindb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with raindb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to raindb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to raindb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted raindb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_infusedb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("infusedb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.infusedb");
    configure_infusedb_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with infusedb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to infusedb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to infusedb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted infusedb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_kafi_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("kafi-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.kafi");
    configure_kafi_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with kafi store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to kafi".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to kafi on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted kafi snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_tinkv_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("tinkv-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    configure_tinkv_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with tinkv store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to tinkv".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to tinkv on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted tinkv snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_ledger_kv_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("ledger-kv-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    configure_ledger_kv_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with ledger_kv store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to ledger_kv".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to ledger_kv on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted ledger_kv snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(
        snapshot_dir
            .join("snapshots.ledger_kv")
            .join("snapshots.bin")
            .exists()
    );
    assert!(
        snapshot_dir
            .join("snapshots.ledger_kv")
            .join("snapshots.meta")
            .exists()
    );

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_joydb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("joydb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    configure_joydb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with joydb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to joydb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to joydb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted joydb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_dir.join("snapshots.joydb.json").exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_bitcask_engine_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("bitcask-engine-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.bitcask_engine");
    configure_bitcask_engine_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with bitcask-engine store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to bitcask-engine".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to bitcask-engine on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state = AppState::from_config(&config)
        .expect("state should reload persisted bitcask-engine snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_blazeup_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("blazeup-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.blazeup");
    configure_blazeup_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with blazeup store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to blazeup".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to blazeup on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted blazeup snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_feoxdb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("feoxdb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.feoxdb");
    configure_feoxdb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with feoxdb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to feoxdb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to feoxdb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted feoxdb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_db_rs_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("db-rs-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.db_rs");
    configure_db_rs_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with db_rs store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to db_rs".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to db_rs on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted db_rs snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_dharmadb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("dharmadb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.dharmadb");
    configure_dharmadb_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with dharmadb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to dharmadb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to dharmadb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted dharmadb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_jsondb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("jsondb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.jsondb.json");
    configure_jsondb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with jsondb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to jsondb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to jsondb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted jsondb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_kopperdb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("kopperdb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.kopperdb");
    configure_kopperdb_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with kopperdb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to kopperdb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to kopperdb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted kopperdb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_rcask_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("rcask-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.rcask");
    configure_rcask_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with rcask store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to rcask".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to rcask on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted rcask snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_jfs_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("jfs-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.jfs.json");
    configure_jfs_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with jfs store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to jfs".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to jfs on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted jfs snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_json_store_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("json-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.json_store.jsonl");
    configure_json_store_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with json_store backend");

    let document = state
        .rooms()
        .create_document(Some("Persisted to json_store".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to json_store on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted json_store snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_koit_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("koit-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.koit.json");
    configure_koit_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with koit store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to koit".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to koit on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted koit snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_lite_db_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("lite-db-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.lite_db");
    configure_lite_db_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with lite_db store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to lite_db".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to lite_db on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted lite_db snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_log_kv_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("log-kv-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.log_kv");
    configure_log_kv_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with log_kv store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to log_kv".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to log_kv on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted log_kv snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_append_kv_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("append-kv-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.append_kv");
    configure_append_kv_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with append_kv store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to append_kv".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to append_kv on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted append_kv snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().title, document.title);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_mhdb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("mhdb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.mhdb");
    configure_mhdb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with mhdb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to mhdb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to mhdb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted mhdb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.with_extension("pag").exists());
    assert!(snapshot_path.with_extension("dir").exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_loro_kv_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("loro-kv-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.loro_kv");
    configure_loro_kv_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with loro_kv store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to loro_kv".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to loro_kv on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted loro_kv snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_luckdb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("luckdb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.luckdb.json");
    configure_luckdb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with luckdb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to luckdb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to luckdb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted luckdb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_ipjdb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("ipjdb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.ipjdb");
    configure_ipjdb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with ipjdb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to ipjdb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to ipjdb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted ipjdb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_rubin_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("rubin-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.rubin.json");
    configure_rubin_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with rubin store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to rubin".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to rubin on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted rubin snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_lsm_storage_engine_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("lsm-storage-engine-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.lsm_storage_engine");
    configure_lsm_storage_engine_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config)
        .expect("state should initialize with lsm_storage_engine store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to lsm_storage_engine".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to lsm_storage_engine on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state = AppState::from_config(&config)
        .expect("state should reload persisted lsm_storage_engine snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_etchdb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("etchdb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.etchdb");
    configure_etchdb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with etchdb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to etchdb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to etchdb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted etchdb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_lsm_engine_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("lsm-engine-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.lsm_engine");
    configure_lsm_engine_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with lsm_engine store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to lsm_engine".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to lsm_engine on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted lsm_engine snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_lsmdb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("lsmdb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.lsmdb");
    configure_lsmdb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with lsmdb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to lsmdb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to lsmdb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted lsmdb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_lsm_tree_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("lsm-tree-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.lsm_tree");
    configure_lsm_tree_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with lsm_tree store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to lsm_tree".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to lsm_tree on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted lsm_tree snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.join("current").exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_ferrumdb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("ferrumdb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.ferrumdb");
    configure_ferrumdb_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with ferrumdb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to ferrumdb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to ferrumdb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted ferrumdb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_mindb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("mindb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.mindb");
    configure_mindb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with mindb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to mindb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to mindb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted mindb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_mmdb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("mmdb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.mmdb");
    configure_mmdb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with mmdb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to mmdb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to mmdb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted mmdb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_nanodb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("nanodb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.nanodb.json");
    configure_nanodb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with nanodb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to nanodb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to nanodb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted nanodb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_graus_db_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("graus_db-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.graus_db");
    configure_graus_db_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with graus_db store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to graus_db".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to graus_db on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted graus_db snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_kv_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("kv-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.kv");
    configure_kv_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with kv store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to kv".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to kv on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted kv snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_eight_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("eight-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.eight");
    configure_eight_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with eight store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to eight".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to eight on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted eight snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_epoch_db_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("epoch-db-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.epoch_db");
    configure_epoch_db_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with epoch-db store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to epoch-db".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to epoch-db on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted epoch-db snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_rumdb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("rumdb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.rumdb");
    configure_rumdb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with rumdb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to rumdb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to rumdb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted rumdb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_hmdb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("hmdb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    configure_hmdb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with hmdb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to hmdb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to hmdb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted hmdb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(
        fs::read_dir(&snapshot_dir)
            .expect("hmdb snapshot directory should exist")
            .next()
            .is_some()
    );

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_icefalldb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("icefalldb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.icefalldb");
    configure_icefalldb_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with icefalldb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to icefalldb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to icefalldb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted icefalldb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_bitask_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("bitask-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.bitask");
    configure_bitask_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with bitask store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to bitask".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to bitask on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted bitask snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_bitkv_rs_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("bitkv-rs-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.bitkv_rs");
    configure_bitkv_rs_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with bitkv-rs store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to bitkv-rs".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to bitkv-rs on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted bitkv-rs snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_candystore_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("candystore-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.candystore");
    configure_candystore_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with candystore store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to candystore".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to candystore on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted candystore snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_cuendillar_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("cuendillar-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.cuendillar");
    configure_cuendillar_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with cuendillar store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to cuendillar".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to cuendillar on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted cuendillar snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());
    assert!(snapshot_path.join("wal").exists());
    assert!(snapshot_path.join("sstable").exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_caves_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("caves-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_root = snapshot_dir.join("snapshots.caves");
    configure_caves_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with caves store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to caves".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to caves on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted caves snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_root.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_persistent_kv_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("persistent-kv-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.persistent_kv");
    configure_persistent_kv_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with persistent_kv store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to persistent_kv".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to persistent_kv on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state = AppState::from_config(&config)
        .expect("state should reload persisted persistent_kv snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_native_db_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("native-db-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.native_db");
    configure_native_db_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with native_db store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to native_db".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to native_db on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted native_db snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_nebari_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("nebari-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_root = snapshot_dir.join("snapshots.nebari");
    configure_nebari_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with nebari store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to nebari".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to nebari on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted nebari snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_root.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_nodb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("nodb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.nodb");
    configure_nodb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with nodb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to nodb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to nodb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted nodb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_okofdb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("okofdb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.okofdb");
    configure_okofdb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with okofdb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to okofdb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to okofdb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted okofdb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_celerix_store_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("celerix-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.celerix_store");
    configure_celerix_store_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with celerix_store store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to celerix_store".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to celerix_store on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state = AppState::from_config(&config)
        .expect("state should reload persisted celerix_store snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.join("snapshots.json").exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_citadeldb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("citadeldb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.citadeldb");
    configure_citadeldb_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with citadeldb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to citadeldb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to citadeldb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted citadeldb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());
    assert!(
        snapshot_dir
            .join("snapshots.citadeldb.citadel-keys")
            .exists()
    );

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_thetadb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("thetadb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.thetadb");
    configure_thetadb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with thetadb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to thetadb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to thetadb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted thetadb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_vsdb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("vsdb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_root = snapshot_dir.join("snapshots.vsdb");
    configure_vsdb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with vsdb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to vsdb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to vsdb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted vsdb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_root.join("store.meta.json").exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_grebedb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("grebedb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.grebedb");
    configure_grebedb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with grebedb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to grebedb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to grebedb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted grebedb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_grumpydb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("grumpydb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.grumpydb");
    configure_grumpydb_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with grumpydb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to grumpydb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to grumpydb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted grumpydb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_blockbucket_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("blockbucket-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.blockbucket");
    configure_blockbucket_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with blockbucket store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to blockbucket".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to blockbucket on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted blockbucket snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_nikidb_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("nikidb-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.nikidb");
    configure_nikidb_snapshot_store(&mut config, &snapshot_dir);

    let state = AppState::from_config(&config).expect("state should initialize with nikidb store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to nikidb".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to nikidb on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted nikidb snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_uses_parity_db_snapshot_store_from_config() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("parity-db-store-config");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.parity_db");
    configure_parity_db_snapshot_store(&mut config, &snapshot_dir);

    let state =
        AppState::from_config(&config).expect("state should initialize with parity_db store");

    let document = state
        .rooms()
        .create_document(Some("Persisted to parity_db".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to parity_db on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    drop(room);
    drop(state);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted parity_db snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(snapshot_path.exists());

    drop(restored_room);
    drop(reloaded_state);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn app_state_rejects_s3_snapshot_store_without_endpoint() {
    let mut config = test_config();
    config.snapshot_store = "s3".to_owned();

    let error = match AppState::from_config(&config) {
        Ok(_) => panic!("s3 snapshot store should require endpoint"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("SNAPSHOT_S3_ENDPOINT is required when SNAPSHOT_STORE=s3"),
        "unexpected error: {error}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn app_state_uses_managed_snapshot_store_from_config() {
    let harness = spawn_mock_managed_coordination_service(Some("snapshot-secret")).await;

    let mut config = test_config();
    configure_managed_snapshot_store(
        &mut config,
        &harness.snapshot_base_url,
        Some("snapshot-secret"),
    );

    let state = AppState::from_config(&config)
        .expect("state should initialize with managed snapshot store");
    let document = state
        .rooms()
        .create_document(Some("Persisted to managed store".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to managed store on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    let persisted_snapshot = harness
        .state
        .snapshot(&document.id)
        .expect("managed snapshot service should store the snapshot");
    assert_eq!(persisted_snapshot.document.id, document.id);

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted managed snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn app_state_uses_s3_snapshot_store_from_config() {
    let harness = spawn_mock_s3_snapshot_service().await;

    let mut config = test_config();
    configure_s3_snapshot_store(
        &mut config,
        &harness.endpoint,
        &harness.bucket,
        &harness.access_key_id,
        &harness.secret_access_key,
    );

    let state =
        AppState::from_config(&config).expect("state should initialize with s3 snapshot store");
    let document = state
        .rooms()
        .create_document(Some("Persisted to s3".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("snapshot should persist to s3 on eviction");
    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    let object_key = format!("snapshots/test-suite/{}.json", document.id);
    let persisted_snapshot = harness
        .state
        .object(&object_key)
        .expect("mock s3 service should store the snapshot object");
    assert!(!persisted_snapshot.is_empty());

    let reloaded_state =
        AppState::from_config(&config).expect("state should reload persisted s3 snapshot");
    let restored_room = reloaded_state
        .rooms()
        .get(&document.id)
        .expect("persisted room should hydrate on startup");

    assert_eq!(restored_room.document().id, document.id);
    assert!(
        harness
            .state
            .last_authorization()
            .is_some_and(|header| header.contains("Credential=test-access-key/")),
        "s3 requests should be signed with the configured access key"
    );
}

#[tokio::test]
async fn app_state_skips_startup_room_hydration_in_distributed_sqlite_mode() {
    let shared_root = temp_snapshot_dir("sqlite-distributed-skip-hydrate");

    let mut writer_config = test_config();
    configure_shared_sqlite_collaboration(
        &mut writer_config,
        &shared_root,
        "node-a",
        "http://node-a.internal:4400/",
    );
    let writer_state =
        AppState::from_config(&writer_config).expect("writer state should initialize");
    let document = writer_state
        .rooms()
        .create_document(Some("Distributed hydrate guard".to_owned()))
        .expect("document should be created");

    let mut reader_config = test_config();
    configure_shared_sqlite_collaboration(
        &mut reader_config,
        &shared_root,
        "node-b",
        "http://node-b.internal:4401/",
    );
    let reader_state =
        AppState::from_config(&reader_config).expect("reader state should initialize");

    assert!(
        reader_state.rooms().get(&document.id).is_none(),
        "distributed sqlite mode should leave rooms cold until ownership is checked"
    );
    let listed_documents = reader_state
        .rooms()
        .list_documents()
        .expect("document catalog should still load from shared snapshot store");
    assert_eq!(listed_documents.len(), 1);
    assert_eq!(listed_documents[0].id, document.id);
    assert_eq!(listed_documents[0].title, document.title);

    fs::remove_dir_all(shared_root).expect("shared sqlite test directory should be cleaned up");
}

#[tokio::test]
async fn app_state_uses_logging_room_coordinator_from_config() {
    let mut config = test_config();
    config.room_coordinator = "logging".to_owned();

    let state =
        AppState::from_config(&config).expect("state should initialize with logging coordinator");
    assert_eq!(state.room_coordinator().mode(), "logging");

    let document = state
        .rooms()
        .create_document(Some("Logged room".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("logging coordinator should not affect snapshot persistence");

    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);
}

#[tokio::test]
async fn app_state_uses_file_room_coordinator_from_config() {
    let mut config = test_config();
    let coordinator_dir = temp_snapshot_dir("file-room-coordinator");
    config.room_coordinator = "file".to_owned();
    config.room_coordinator_state_dir = coordinator_dir.display().to_string();
    config.node_base_url = Some("http://node-a.internal:4100/".to_owned());

    let state =
        AppState::from_config(&config).expect("state should initialize with file coordinator");
    assert_eq!(state.room_coordinator().mode(), "file");

    let document = state
        .rooms()
        .create_document(Some("File coordinated room".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    state
        .room_coordinator()
        .room_activated(&document.id)
        .expect("file coordinator should persist active room state");
    let state_path = coordinator_dir.join(format!("{}.json", document.id));
    assert!(state_path.exists());

    let persisted_state: Value = serde_json::from_slice(
        &fs::read(&state_path).expect("file coordinator should persist active room state"),
    )
    .expect("file room coordinator state should deserialize");
    assert_eq!(persisted_state["base_url"], "http://node-a.internal:4100");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("file coordinator should not affect snapshot persistence");

    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    state
        .room_coordinator()
        .room_deactivated(&document.id)
        .expect("file coordinator should remove active room state");
    assert!(!state_path.exists());

    fs::remove_dir_all(coordinator_dir).expect("test coordinator directory should be cleaned up");
}

#[tokio::test]
async fn app_state_uses_sqlite_room_coordinator_from_config() {
    let mut config = test_config();
    let coordinator_dir = temp_snapshot_dir("sqlite-room-coordinator");
    let sqlite_path = coordinator_dir.join("room-coordinator.sqlite3");
    config.room_coordinator = "sqlite".to_owned();
    config.room_coordinator_sqlite_path = sqlite_path.to_string_lossy().into_owned();
    config.node_base_url = Some("http://node-a.internal:4200/".to_owned());

    let state =
        AppState::from_config(&config).expect("state should initialize with sqlite coordinator");
    assert_eq!(state.room_coordinator().mode(), "sqlite");

    let document = state
        .rooms()
        .create_document(Some("Sqlite coordinated room".to_owned()))
        .expect("document should be created");
    let room = state
        .rooms()
        .get(&document.id)
        .expect("created document should have a room");

    state
        .room_coordinator()
        .room_activated(&document.id)
        .expect("sqlite coordinator should persist active room state");
    let connection = rusqlite::Connection::open(&sqlite_path).expect("sqlite file should open");
    let persisted_state: Value = connection
        .query_row(
            "SELECT json_object(
                'doc_id', doc_id,
                'node_id', node_id,
                'base_url', base_url,
                'lease_id', lease_id,
                'epoch', epoch,
                'activated_at', activated_at,
                'renewed_at', renewed_at,
                'expires_at', expires_at
            )
             FROM room_leases
             WHERE doc_id = ?1",
            [document.id.to_string()],
            |row| row.get::<_, String>(0),
        )
        .map(|json| serde_json::from_str(&json).expect("sqlite room lease json should parse"))
        .expect("sqlite coordinator should persist active room state");
    assert_eq!(persisted_state["base_url"], "http://node-a.internal:4200");

    assert_eq!(room.start_session(), 1);
    let teardown = state
        .rooms()
        .persist_and_evict_if_idle(&document.id, &room)
        .expect("sqlite coordinator should not affect snapshot persistence");

    assert!(teardown.evicted);
    assert_eq!(teardown.remaining_sessions, 0);

    state
        .room_coordinator()
        .room_deactivated(&document.id)
        .expect("sqlite coordinator should remove active room state");
    let remaining: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM room_leases WHERE doc_id = ?1",
            [document.id.to_string()],
            |row| row.get(0),
        )
        .expect("sqlite coordinator should query room lease count");
    assert_eq!(remaining, 0);

    fs::remove_dir_all(coordinator_dir).expect("test coordinator directory should be cleaned up");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn app_state_uses_managed_room_coordination_from_config() {
    let harness = spawn_mock_managed_coordination_service(Some("managed-secret")).await;

    let mut writer_config = test_config();
    writer_config.room_locator = "managed".to_owned();
    writer_config.room_coordinator = "managed".to_owned();
    writer_config.room_coordination_managed_base_url = Some(harness.base_url.clone());
    writer_config.room_coordination_managed_auth_token = Some("managed-secret".to_owned());
    writer_config.room_coordinator_heartbeat_interval_secs = 1;
    writer_config.room_coordinator_lease_ttl_secs = 3;
    writer_config.node_id = "node-a".to_owned();
    writer_config.node_base_url = Some("http://node-a.internal:4300/".to_owned());

    let writer_state =
        AppState::from_config(&writer_config).expect("state should initialize with managed mode");
    assert_eq!(writer_state.room_coordinator().mode(), "managed");

    let document = writer_state
        .rooms()
        .create_document(Some("Managed coordinated room".to_owned()))
        .expect("document should be created");

    writer_state
        .room_coordinator()
        .room_activated(&document.id)
        .expect("managed coordinator should persist active room state");

    let initial_lease = harness
        .state
        .lease(&document.id)
        .expect("managed coordination service should store the acquired lease");
    assert_eq!(initial_lease.node_id, "node-a");
    assert_eq!(
        initial_lease.base_url,
        Some("http://node-a.internal:4300".to_owned())
    );

    tokio::time::sleep(Duration::from_millis(1_100)).await;

    let renewed_lease = harness
        .state
        .lease(&document.id)
        .expect("managed coordination service should keep the renewed lease");
    assert_eq!(renewed_lease.lease_id, initial_lease.lease_id);
    assert_eq!(renewed_lease.epoch, initial_lease.epoch);
    assert!(
        renewed_lease.renewed_at > initial_lease.renewed_at,
        "managed coordinator heartbeat should advance renewed_at"
    );

    let mut reader_config = test_config();
    reader_config.room_locator = "managed".to_owned();
    reader_config.room_coordination_managed_base_url = Some(harness.base_url.clone());
    reader_config.room_coordination_managed_auth_token = Some("managed-secret".to_owned());
    reader_config.node_id = "node-b".to_owned();

    let reader_state =
        AppState::from_config(&reader_config).expect("reader state should initialize");
    let error = reader_state
        .ensure_local_room_owner(&document.id)
        .expect_err("managed locator should report the remote owner while the lease is active");

    match error {
        AppError::RemoteOwner {
            owner_node_id,
            owner_base_url,
            ..
        } => {
            assert_eq!(owner_node_id, "node-a");
            assert_eq!(
                owner_base_url.as_deref(),
                Some("http://node-a.internal:4300")
            );
        }
        other => panic!("expected remote owner error, received {other:?}"),
    }

    writer_state
        .room_coordinator()
        .room_deactivated(&document.id)
        .expect("managed coordinator should release active room state");
    assert!(
        harness.state.lease(&document.id).is_none(),
        "managed coordination service should remove the lease after release"
    );
    reader_state
        .ensure_local_room_owner(&document.id)
        .expect("managed locator should resolve locally after lease release");
}

#[tokio::test]
async fn app_state_with_file_store_skips_corrupt_snapshots_during_startup() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("file-store-corrupt-startup");
    config.snapshot_store = "file".to_owned();
    config.snapshot_dir = snapshot_dir.to_string_lossy().into_owned();

    let store = FileSnapshotStore::new(&snapshot_dir).expect("file store should initialize");
    let valid_document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Healthy".to_owned()));
    let valid_update = Doc::new()
        .transact()
        .encode_state_as_update_v1(&StateVector::default());
    store
        .save_snapshot(DocumentSnapshot::new(valid_document.clone(), valid_update))
        .expect("valid snapshot should save");

    let corrupt_doc_id = Uuid::new_v4();
    fs::write(
        snapshot_dir.join(format!("{corrupt_doc_id}.json")),
        b"{not-json",
    )
    .expect("corrupt snapshot fixture should be written");

    let state = AppState::from_config(&config)
        .expect("startup hydration should continue past corrupt snapshots");
    let hydrated_documents = state
        .rooms()
        .list_documents()
        .expect("document catalog should still be available");

    assert!(state.rooms().get(&valid_document.id).is_some());
    assert_eq!(hydrated_documents, vec![valid_document]);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[tokio::test]
async fn app_state_with_file_store_cleans_matching_stale_temp_snapshots_during_startup() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("file-store-stale-temp-startup");
    config.snapshot_store = "file".to_owned();
    config.snapshot_dir = snapshot_dir.to_string_lossy().into_owned();

    let store = FileSnapshotStore::new(&snapshot_dir).expect("file store should initialize");
    let valid_document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Healthy".to_owned()));
    let valid_update = Doc::new()
        .transact()
        .encode_state_as_update_v1(&StateVector::default());
    store
        .save_snapshot(DocumentSnapshot::new(valid_document.clone(), valid_update))
        .expect("valid snapshot should save");

    let stale_temp_path =
        snapshot_dir.join(format!("{}.json.{}.tmp", valid_document.id, Uuid::new_v4()));
    fs::write(&stale_temp_path, br#"{"partial":true}"#)
        .expect("stale temp snapshot fixture should be written");

    let state = AppState::from_config(&config)
        .expect("startup hydration should clean stale temp files and restore valid snapshots");
    let hydrated_documents = state
        .rooms()
        .list_documents()
        .expect("document catalog should still be available");

    assert!(state.rooms().get(&valid_document.id).is_some());
    assert_eq!(hydrated_documents, vec![valid_document]);
    assert!(!stale_temp_path.exists());

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[tokio::test]
async fn app_state_with_file_store_cleans_orphan_stale_temp_snapshots_during_startup() {
    let mut config = test_config();
    let snapshot_dir = temp_snapshot_dir("file-store-orphan-stale-temp-startup");
    config.snapshot_store = "file".to_owned();
    config.snapshot_dir = snapshot_dir.to_string_lossy().into_owned();

    let orphan_doc_id = Uuid::new_v4();
    let stale_temp_path = snapshot_dir.join(format!("{orphan_doc_id}.json.{}.tmp", Uuid::new_v4()));
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    fs::write(&stale_temp_path, br#"{"partial":true}"#)
        .expect("stale temp snapshot fixture should be written");

    let state =
        AppState::from_config(&config).expect("startup hydration should clean orphan temp files");
    let hydrated_documents = state
        .rooms()
        .list_documents()
        .expect("document catalog should remain empty");

    assert!(state.rooms().get(&orphan_doc_id).is_none());
    assert!(hydrated_documents.is_empty());
    assert!(!stale_temp_path.exists());

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn file_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("file-store-unit");
    let store = FileSnapshotStore::new(&snapshot_dir).expect("file store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Disk".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to file store");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from file store");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from file store")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn sqlite_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("sqlite-store-unit");
    let snapshot_path = snapshot_dir.join("snapshots.sqlite3");
    let store = SqliteSnapshotStore::new(&snapshot_path).expect("sqlite store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Sqlite".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to sqlite store");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from sqlite store");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from sqlite store")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn jammdb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("jammdb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.jammdb");
    let store =
        JammdbSnapshotStore::new(&snapshot_path).expect("jammdb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Jammdb".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to jammdb store");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from jammdb store");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from jammdb store")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn janql_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("janql-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.janql");
    let store =
        JanqlSnapshotStore::new(&snapshot_path).expect("janql snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("JanQL".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to janql");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from janql");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from janql")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened =
        JanqlSnapshotStore::new(&snapshot_path).expect("janql snapshot store should reopen");
    let reopened_documents = reopened
        .list_documents()
        .expect("document catalog should reload from janql");
    let reopened_snapshot = reopened
        .load_snapshot(&document.id)
        .expect("snapshot should reload from janql")
        .expect("snapshot should exist after reopen");

    assert_eq!(reopened_documents, vec![document.clone()]);
    assert_eq!(reopened_snapshot.document, document);
    assert_eq!(reopened_snapshot.update, vec![1, 2, 3]);

    drop(reopened);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn jasondb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("jasondb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.jasondb");
    let store = JasondbSnapshotStore::new(&snapshot_path)
        .expect("jasondb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("JasonDB".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to jasondb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from jasondb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from jasondb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store =
        JasondbSnapshotStore::new(&snapshot_path).expect("jasondb snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from jasondb"),
        vec![document.clone()]
    );
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("snapshot should reload from jasondb")
            .is_some()
    );

    reopened_store
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from jasondb");
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("deleted snapshot lookup should succeed")
            .is_none()
    );
    assert!(
        reopened_store
            .list_documents()
            .expect("document catalog should reflect jasondb deletion")
            .is_empty()
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn jasonisnthappy_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("jasonisnthappy-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.jasonisnthappy");
    let store = JasonisnthappySnapshotStore::new(&snapshot_path)
        .expect("jasonisnthappy snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("JasonIsntHappy".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to jasonisnthappy");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from jasonisnthappy");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from jasonisnthappy")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store = JasonisnthappySnapshotStore::new(&snapshot_path)
        .expect("jasonisnthappy snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from jasonisnthappy"),
        vec![document.clone()]
    );
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("snapshot should reload from jasonisnthappy")
            .is_some()
    );

    reopened_store
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from jasonisnthappy");
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("deleted snapshot lookup should succeed")
            .is_none()
    );
    assert!(
        reopened_store
            .list_documents()
            .expect("document catalog should reflect jasonisnthappy deletion")
            .is_empty()
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn datastack_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("datastack-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.datastack");
    let store = DatastackSnapshotStore::new(&snapshot_path)
        .expect("datastack snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("DataStack".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to datastack");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from datastack");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from datastack")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store = DatastackSnapshotStore::new(&snapshot_path)
        .expect("datastack snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from datastack"),
        vec![document.clone()]
    );
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("snapshot should reload from datastack")
            .is_some()
    );

    reopened_store
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from datastack");
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("deleted snapshot lookup should succeed")
            .is_none()
    );
    assert!(
        reopened_store
            .list_documents()
            .expect("document catalog should reflect datastack deletion")
            .is_empty()
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn mace_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("mace-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.mace");
    let store =
        MaceSnapshotStore::new(&snapshot_path).expect("mace snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Mace".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to mace");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from mace");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from mace")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened =
        MaceSnapshotStore::new(&snapshot_path).expect("mace snapshot store should reopen");
    let reopened_documents = reopened
        .list_documents()
        .expect("document catalog should reload from mace");
    let reopened_snapshot = reopened
        .load_snapshot(&document.id)
        .expect("snapshot should reload from mace")
        .expect("snapshot should exist after reopen");

    assert_eq!(reopened_documents, vec![document.clone()]);
    assert_eq!(reopened_snapshot.document, document);
    assert_eq!(reopened_snapshot.update, vec![1, 2, 3]);

    drop(reopened);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn heed_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("heed-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.heed");
    let store =
        HeedSnapshotStore::new(&snapshot_path).expect("heed snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Heed".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to heed store");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from heed store");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from heed store")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn hightower_kv_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("hightower-kv-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.hightower_kv");
    let store = HightowerKvSnapshotStore::new(&snapshot_path)
        .expect("hightower_kv snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Hightower KV".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to hightower_kv");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from hightower_kv");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from hightower_kv")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn fjall_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("fjall-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.fjall");
    let store =
        FjallSnapshotStore::new(&snapshot_path).expect("fjall snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Fjall".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to fjall store");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from fjall store");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from fjall store")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn persy_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("persy-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.persy");
    let store =
        PersySnapshotStore::new(&snapshot_path).expect("persy snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Persy".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to persy store");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from persy store");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from persy store")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn managed_snapshot_store_round_trips_document_catalog() {
    let runtime = tokio::runtime::Runtime::new().expect("tokio runtime should initialize");
    let harness = runtime.block_on(spawn_mock_managed_coordination_service(Some(
        "snapshot-secret",
    )));
    let store = ManagedSnapshotStore::new(
        &harness.snapshot_base_url,
        Some("snapshot-secret".to_owned()),
        Duration::from_secs(5),
    )
    .expect("managed snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Managed".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to managed store");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from managed store");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from managed store")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);
}

#[test]
fn redb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("redb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.redb");
    let store =
        RedbSnapshotStore::new(&snapshot_path).expect("redb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Redb".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to redb store");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from redb store");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from redb store")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn sled_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("sled-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.sled");
    let store =
        SledSnapshotStore::new(&snapshot_path).expect("sled snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Sled".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to sled store");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from sled store");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from sled store")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn pickledb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("pickledb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.pickledb");
    let store = PickleDbSnapshotStore::new(&snapshot_path)
        .expect("pickledb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("PickleDB".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to pickledb store");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from pickledb store");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from pickledb store")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn microkv_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("microkv-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots_microkv");
    let store = MicroKvSnapshotStore::new(&snapshot_path)
        .expect("microkv snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("MicroKV".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to microkv store");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from microkv store");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from microkv store")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn rustbreak_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("rustbreak-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.rustbreak");
    let store = RustbreakSnapshotStore::new(&snapshot_path)
        .expect("rustbreak snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Rustbreak".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to rustbreak store");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from rustbreak store");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from rustbreak store")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn yedb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("yedb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.yedb");
    let store =
        YedbSnapshotStore::new(&snapshot_path).expect("yedb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Yedb".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to yedb store");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from yedb store");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from yedb store")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn btree_store_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("btree-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.btree_store");
    let store = BtreeStoreSnapshotStore::new(&snapshot_path)
        .expect("btree_store snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("BtreeStore".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to btree_store");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from btree_store");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from btree_store")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn siamesedb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("siamesedb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.siamesedb");
    let store = SiamesedbSnapshotStore::new(&snapshot_path)
        .expect("siamesedb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Siamesedb".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to siamesedb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from siamesedb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from siamesedb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn readb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("readb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.readb");
    let store =
        ReadbSnapshotStore::new(&snapshot_path).expect("readb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Readb".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to readb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from readb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from readb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn rustlite_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("rustlite-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.rustlite");
    let store = RustliteSnapshotStore::new(&snapshot_path)
        .expect("rustlite snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Rustlite".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to rustlite");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from rustlite");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from rustlite")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn canopydb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("canopydb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.canopydb");
    let store = CanopydbSnapshotStore::new(&snapshot_path)
        .expect("canopydb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Canopydb".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to canopydb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from canopydb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from canopydb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn caves_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("caves-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_root = snapshot_dir.join("snapshots.caves");
    let store =
        CavesSnapshotStore::new(&snapshot_root).expect("caves snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Caves".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to caves");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from caves");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from caves")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn structsy_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("structsy-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.structsy");
    let store = StructsySnapshotStore::new(&snapshot_path)
        .expect("structsy snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Structsy".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to structsy");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from structsy");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from structsy")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn abyssiniandb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("abyssiniandb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.abyssiniandb");
    let store = AbyssiniandbSnapshotStore::new(&snapshot_path)
        .expect("abyssiniandb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Abyssiniandb".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to abyssiniandb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from abyssiniandb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from abyssiniandb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn ckydb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("ckydb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.ckydb");
    let store =
        CkydbSnapshotStore::new(&snapshot_path).expect("ckydb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Ckydb".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to ckydb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from ckydb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from ckydb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn crepedb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("crepedb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.crepedb");
    let store = CrepeDbSnapshotStore::new(&snapshot_path)
        .expect("crepedb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("CrepeDB".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to crepedb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from crepedb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from crepedb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened =
        CrepeDbSnapshotStore::new(&snapshot_path).expect("crepedb snapshot store should reopen");
    assert_eq!(
        reopened
            .list_documents()
            .expect("document catalog should reload from crepedb"),
        vec![loaded_snapshot.document.clone()]
    );

    drop(reopened);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn crystal_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("crystal-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let store =
        CrystalSnapshotStore::new(&snapshot_dir).expect("crystal snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Crystal".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to crystal");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from crystal");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from crystal")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened =
        CrystalSnapshotStore::new(&snapshot_dir).expect("crystal snapshot store should reopen");
    assert_eq!(
        reopened
            .list_documents()
            .expect("document catalog should reload from crystal"),
        vec![document.clone()]
    );
    assert!(
        reopened
            .load_snapshot(&document.id)
            .expect("snapshot should reload from crystal")
            .is_some()
    );

    reopened
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from crystal");
    assert!(
        reopened
            .load_snapshot(&document.id)
            .expect("deleted snapshot lookup should succeed")
            .is_none()
    );
    assert!(
        reopened
            .list_documents()
            .expect("document catalog should reflect crystal deletion")
            .is_empty()
    );

    drop(reopened);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn assystem_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("assystem-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.assystem");
    let store = AssystemSnapshotStore::new(&snapshot_path)
        .expect("assystem snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Assystem".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to assystem");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from assystem");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from assystem")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened =
        AssystemSnapshotStore::new(&snapshot_path).expect("assystem snapshot store should reopen");
    assert_eq!(
        reopened
            .list_documents()
            .expect("document catalog should reload from assystem"),
        vec![document.clone()]
    );
    assert!(
        reopened
            .load_snapshot(&document.id)
            .expect("snapshot should reload from assystem")
            .is_some()
    );

    reopened
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from assystem");
    assert!(
        reopened
            .load_snapshot(&document.id)
            .expect("deleted snapshot lookup should succeed")
            .is_none()
    );
    assert!(
        reopened
            .list_documents()
            .expect("document catalog should reflect assystem deletion")
            .is_empty()
    );

    drop(reopened);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn colon_db_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("colon-db-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.colon_db");
    let store = ColonDbSnapshotStore::new(&snapshot_path)
        .expect("colon_db snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Colon DB".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to colon_db");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from colon_db");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from colon_db")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened =
        ColonDbSnapshotStore::new(&snapshot_path).expect("colon_db snapshot store should reopen");
    assert_eq!(
        reopened
            .list_documents()
            .expect("document catalog should reload from colon_db"),
        vec![document.clone()]
    );
    assert!(
        reopened
            .load_snapshot(&document.id)
            .expect("snapshot should reload from colon_db")
            .is_some()
    );

    reopened
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from colon_db");
    assert!(
        reopened
            .load_snapshot(&document.id)
            .expect("deleted snapshot lookup should succeed")
            .is_none()
    );
    assert!(
        reopened
            .list_documents()
            .expect("document catalog should reflect colon_db deletion")
            .is_empty()
    );

    drop(reopened);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn rskey_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("rskey-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.rskey");
    let store =
        RskeySnapshotStore::new(&snapshot_path).expect("rskey snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Rskey".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to rskey");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from rskey");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from rskey")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn scdb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("scdb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.scdb");
    let store =
        ScdbSnapshotStore::new(&snapshot_path).expect("scdb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Scdb".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to scdb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from scdb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from scdb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn skv_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("skv-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.skv");
    let store =
        SkvSnapshotStore::new(&snapshot_path).expect("skv snapshot store should initialize");
    let document = backend::models::document::Document::new(Uuid::new_v4(), Some("Skv".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to skv");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from skv");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from skv")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn surrealkv_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("surrealkv-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.surrealkv");
    let store = SurrealkvSnapshotStore::new(&snapshot_path)
        .expect("surrealkv snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("SurrealKV".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to surrealkv");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from surrealkv");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from surrealkv")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn thunderdb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("thunderdb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.thunderdb");
    let store = ThunderdbSnapshotStore::new(&snapshot_path)
        .expect("thunderdb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Thunderdb".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to thunderdb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from thunderdb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from thunderdb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn thetadb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("thetadb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.thetadb");
    let store = ThetadbSnapshotStore::new(&snapshot_path)
        .expect("thetadb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("ThetaDb".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to thetadb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from thetadb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from thetadb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn vsdb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("vsdb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_root = snapshot_dir.join("snapshots.vsdb");
    let store =
        VsdbSnapshotStore::new(&snapshot_root).expect("vsdb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Vsdb".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to vsdb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from vsdb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from vsdb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);
    assert!(snapshot_root.join("store.meta.json").exists());

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn tinybase_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("tinybase-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.tinybase");
    let store = TinybaseSnapshotStore::new(&snapshot_path)
        .expect("tinybase snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Tinybase".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to tinybase");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from tinybase");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from tinybase")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn tinydb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("tinydb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.tinydb");
    let store =
        TinydbSnapshotStore::new(&snapshot_path).expect("tinydb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Tinydb".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to tinydb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from tinydb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from tinydb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);
    assert!(snapshot_path.exists());

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn dblite_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("dblite-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.dblite");
    let store =
        DbliteSnapshotStore::new(&snapshot_path).expect("dblite snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Dblite".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to dblite");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from dblite");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from dblite")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn aeternusdb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("aeternusdb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.aeternusdb");
    let store = AeternusdbSnapshotStore::new(&snapshot_path)
        .expect("aeternusdb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("AeternusDB".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to aeternusdb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from aeternusdb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from aeternusdb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn dbless_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("dbless-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.dbless");
    let store =
        DblessSnapshotStore::new(&snapshot_path).expect("dbless snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Dbless".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to dbless");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from dbless");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from dbless")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn sanakirja_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("sanakirja-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.sanakirja");
    let store = SanakirjaSnapshotStore::new(&snapshot_path)
        .expect("sanakirja snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Sanakirja".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to sanakirja");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from sanakirja");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from sanakirja")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn flash_kv_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("flash-kv-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.flash_kv");
    let store = FlashKvSnapshotStore::new(&snapshot_path)
        .expect("flash_kv snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("FlashKV".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to flash_kv");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from flash_kv");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from flash_kv")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn grebedb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("grebedb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.grebedb");
    let store = GrebedbSnapshotStore::new(&snapshot_path)
        .expect("grebedb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("GrebeDB".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to grebedb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from grebedb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from grebedb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn grumpydb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("grumpydb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.grumpydb");
    let store = GrumpydbSnapshotStore::new(&snapshot_path)
        .expect("grumpydb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("GrumpyDB".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to grumpydb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from grumpydb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from grumpydb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    store
        .delete_snapshot(&loaded_snapshot.document.id)
        .expect("snapshot should delete from grumpydb");
    assert!(
        store
            .load_snapshot(&loaded_snapshot.document.id)
            .expect("deleted snapshot lookup should succeed")
            .is_none()
    );

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn highlandcows_isam_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("highlandcows-isam-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.highlandcows_isam");
    let store = HighlandcowsIsamSnapshotStore::new(&snapshot_path)
        .expect("highlandcows-isam snapshot store should initialize");
    let document = backend::models::document::Document::new(
        Uuid::new_v4(),
        Some("Highlandcows ISAM".to_owned()),
    );
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to highlandcows-isam");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from highlandcows-isam");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from highlandcows-isam")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn simple_db_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("simple-db-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.simple_db");
    let store = SimpleDbSnapshotStore::new(&snapshot_path)
        .expect("simple_db snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("SimpleDB".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to simple_db");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from simple_db");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from simple_db")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn snaildb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("snaildb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.snaildb");
    let store = SnaildbSnapshotStore::new(&snapshot_path)
        .expect("snaildb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("SnailDb".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to snaildb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from snaildb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from snaildb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn docdb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("docdb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.docdb.json");
    let store =
        DocDbSnapshotStore::new(&snapshot_path).expect("docdb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("DocDb".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to docdb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from docdb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from docdb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn shorterdb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("shorterdb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.shorterdb");
    let store = ShorterDbSnapshotStore::new(&snapshot_path)
        .expect("shorterdb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("ShorterDB".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to shorterdb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from shorterdb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from shorterdb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn tinykv_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("tinykv-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.tinykv.json");
    let store =
        TinykvSnapshotStore::new(&snapshot_path).expect("tinykv snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("TinyKV".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to tinykv");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from tinykv");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from tinykv")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn saberdb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("saberdb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.saberdb.json");
    let store = SaberdbSnapshotStore::new(&snapshot_path)
        .expect("saberdb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("SaberDB".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to saberdb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from saberdb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from saberdb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn smolldb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("smolldb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.smolldb");
    let store = SmolldbSnapshotStore::new(&snapshot_path)
        .expect("smolldb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("SmollDB".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to smolldb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from smolldb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from smolldb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store =
        SmolldbSnapshotStore::new(&snapshot_path).expect("smolldb snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from smolldb"),
        vec![document]
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn kstone_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("kstone-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.kstone");
    let store =
        KstoneSnapshotStore::new(&snapshot_path).expect("kstone snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Kstone".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to kstone");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from kstone");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from kstone")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store =
        KstoneSnapshotStore::new(&snapshot_path).expect("kstone snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from kstone"),
        vec![document.clone()]
    );
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("snapshot should reload from kstone")
            .is_some()
    );

    reopened_store
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from kstone");
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("deleted snapshot lookup should succeed")
            .is_none()
    );
    assert!(
        reopened_store
            .list_documents()
            .expect("document catalog should reflect kstone deletion")
            .is_empty()
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn ghaladb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("ghaladb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.ghaladb");
    let store = GhaladbSnapshotStore::new(&snapshot_path)
        .expect("ghaladb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("GhalaDB".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to ghaladb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from ghaladb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from ghaladb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store =
        GhaladbSnapshotStore::new(&snapshot_path).expect("ghaladb snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from ghaladb"),
        vec![document.clone()]
    );
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("snapshot should reload from ghaladb")
            .is_some()
    );

    reopened_store
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from ghaladb");
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("deleted snapshot lookup should succeed")
            .is_none()
    );
    assert!(
        reopened_store
            .list_documents()
            .expect("document catalog should reflect ghaladb deletion")
            .is_empty()
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn roughdb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("roughdb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.roughdb");
    let store = RoughdbSnapshotStore::new(&snapshot_path)
        .expect("roughdb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("RoughDB".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to roughdb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from roughdb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from roughdb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store =
        RoughdbSnapshotStore::new(&snapshot_path).expect("roughdb snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from roughdb"),
        vec![document.clone()]
    );
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("snapshot should reload from roughdb")
            .is_some()
    );

    reopened_store
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from roughdb");
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("deleted snapshot lookup should succeed")
            .is_none()
    );
    assert!(
        reopened_store
            .list_documents()
            .expect("document catalog should reflect roughdb deletion")
            .is_empty()
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn raindb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("raindb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.raindb");
    let store =
        RaindbSnapshotStore::new(&snapshot_path).expect("raindb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("RainDB".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to raindb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from raindb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from raindb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store =
        RaindbSnapshotStore::new(&snapshot_path).expect("raindb snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from raindb"),
        vec![document.clone()]
    );
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("snapshot should reload from raindb")
            .is_some()
    );

    reopened_store
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from raindb");
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("deleted snapshot lookup should succeed")
            .is_none()
    );
    assert!(
        reopened_store
            .list_documents()
            .expect("document catalog should reflect raindb deletion")
            .is_empty()
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn infusedb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("infusedb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.infusedb");
    let store = InfusedbSnapshotStore::new(&snapshot_path)
        .expect("infusedb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("InfuseDB".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to infusedb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from infusedb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from infusedb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store =
        InfusedbSnapshotStore::new(&snapshot_path).expect("infusedb snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from infusedb"),
        vec![document.clone()]
    );
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("snapshot should reload from infusedb")
            .is_some()
    );

    reopened_store
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from infusedb");
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("deleted snapshot lookup should succeed")
            .is_none()
    );
    assert!(
        reopened_store
            .list_documents()
            .expect("document catalog should reflect infusedb deletion")
            .is_empty()
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn kafi_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("kafi-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.kafi");
    let store =
        KafiSnapshotStore::new(&snapshot_path).expect("kafi snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Kafi".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to kafi");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from kafi");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from kafi")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store =
        KafiSnapshotStore::new(&snapshot_path).expect("kafi snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from kafi"),
        vec![document.clone()]
    );
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("snapshot should reload from kafi")
            .is_some()
    );

    reopened_store
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from kafi");
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("deleted snapshot lookup should succeed")
            .is_none()
    );
    assert!(
        reopened_store
            .list_documents()
            .expect("document catalog should reflect kafi deletion")
            .is_empty()
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn tinkv_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("tinkv-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.tinkv");
    let store =
        TinkvSnapshotStore::new(&snapshot_path).expect("tinkv snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Tinkv".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to tinkv");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from tinkv");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from tinkv")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store =
        TinkvSnapshotStore::new(&snapshot_path).expect("tinkv snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from tinkv"),
        vec![document.clone()]
    );
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("snapshot should reload from tinkv")
            .is_some()
    );

    reopened_store
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from tinkv");
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("deleted snapshot lookup should succeed")
            .is_none()
    );
    assert!(
        reopened_store
            .list_documents()
            .expect("document catalog should reflect tinkv deletion")
            .is_empty()
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn ledger_kv_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("ledger-kv-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let store = LedgerKvSnapshotStore::new(&snapshot_dir)
        .expect("ledger_kv snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("LedgerKV".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to ledger_kv");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from ledger_kv");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from ledger_kv")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store =
        LedgerKvSnapshotStore::new(&snapshot_dir).expect("ledger_kv snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from ledger_kv"),
        vec![document.clone()]
    );
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("snapshot should reload from ledger_kv")
            .is_some()
    );

    reopened_store
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from ledger_kv");
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("deleted snapshot lookup should succeed")
            .is_none()
    );
    assert!(
        reopened_store
            .list_documents()
            .expect("document catalog should reflect ledger_kv deletion")
            .is_empty()
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn joydb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("joydb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.joydb.json");
    let store =
        JoydbSnapshotStore::new(&snapshot_path).expect("joydb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Joydb".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to joydb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from joydb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from joydb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store =
        JoydbSnapshotStore::new(&snapshot_path).expect("joydb snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from joydb"),
        vec![document.clone()]
    );
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("snapshot should reload from joydb")
            .is_some()
    );

    reopened_store
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from joydb");
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("deleted snapshot lookup should succeed")
            .is_none()
    );
    assert!(
        reopened_store
            .list_documents()
            .expect("document catalog should reflect joydb deletion")
            .is_empty()
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn lsm_tree_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("lsm-tree-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.lsm_tree");
    let store = LsmTreeSnapshotStore::new(&snapshot_path)
        .expect("lsm_tree snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("LsmTree".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to lsm_tree");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from lsm_tree");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from lsm_tree")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store =
        LsmTreeSnapshotStore::new(&snapshot_path).expect("lsm_tree snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from lsm_tree"),
        vec![document.clone()]
    );
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("snapshot should reload from lsm_tree")
            .is_some()
    );

    reopened_store
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from lsm_tree");
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("deleted snapshot lookup should succeed")
            .is_none()
    );
    assert!(
        reopened_store
            .list_documents()
            .expect("document catalog should reflect lsm_tree deletion")
            .is_empty()
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn bitcask_engine_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("bitcask-engine-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.bitcask_engine");
    let store = BitcaskEngineSnapshotStore::new(&snapshot_path)
        .expect("bitcask-engine snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Bitcask Engine".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to bitcask-engine");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from bitcask-engine");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from bitcask-engine")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store = BitcaskEngineSnapshotStore::new(&snapshot_path)
        .expect("bitcask-engine snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from bitcask-engine"),
        vec![document.clone()]
    );
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("snapshot should reload from bitcask-engine")
            .is_some()
    );

    reopened_store
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from bitcask-engine");
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("deleted snapshot lookup should succeed")
            .is_none()
    );
    assert!(
        reopened_store
            .list_documents()
            .expect("document catalog should reflect bitcask-engine deletion")
            .is_empty()
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn feoxdb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("feoxdb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.feoxdb");
    let store =
        FeoxdbSnapshotStore::new(&snapshot_path).expect("feoxdb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("FeOxDB".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to feoxdb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from feoxdb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from feoxdb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store =
        FeoxdbSnapshotStore::new(&snapshot_path).expect("feoxdb snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from feoxdb"),
        vec![document]
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn agdb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("agdb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.agdb");
    let store =
        AgdbSnapshotStore::new(&snapshot_path).expect("agdb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Agdb".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to agdb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from agdb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from agdb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store =
        AgdbSnapshotStore::new(&snapshot_path).expect("agdb snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from agdb"),
        vec![document]
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn amandine_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("amandine-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.amandine");
    let store = AmandineSnapshotStore::new(&snapshot_path)
        .expect("amandine snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Amandine".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to amandine");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from amandine");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from amandine")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store =
        AmandineSnapshotStore::new(&snapshot_path).expect("amandine snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from amandine"),
        vec![document.clone()]
    );
    let loaded_snapshot = reopened_store
        .load_snapshot(&document.id)
        .expect("snapshot should reload from amandine")
        .expect("snapshot should exist after reopen");
    assert_eq!(loaded_snapshot.document, document);

    reopened_store
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from amandine");
    assert!(
        reopened_store
            .list_documents()
            .expect("document catalog should be empty after amandine delete")
            .is_empty()
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn armdb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("armdb-store-roundtrip");
    let snapshot_path = snapshot_dir.join("snapshots.armdb");
    let store =
        ArmdbSnapshotStore::new(&snapshot_path).expect("armdb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("ArmDB".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to armdb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from armdb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from armdb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store =
        ArmdbSnapshotStore::new(&snapshot_path).expect("armdb snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from armdb"),
        vec![document.clone()]
    );

    reopened_store
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from armdb");
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("deleted snapshot lookup should succeed")
            .is_none()
    );
    assert!(
        reopened_store
            .list_documents()
            .expect("document catalog should reflect armdb deletion")
            .is_empty()
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn db_rs_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("db-rs-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.db_rs");
    let store =
        DbRsSnapshotStore::new(&snapshot_path).expect("db_rs snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("DbRs".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to db_rs");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from db_rs");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from db_rs")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn dharmadb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("dharmadb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.dharmadb");
    let store = DharmadbSnapshotStore::new(&snapshot_path)
        .expect("dharmadb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("DharmaDB".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to dharmadb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from dharmadb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from dharmadb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn jsondb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("jsondb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.jsondb.json");
    let store =
        JsondbSnapshotStore::new(&snapshot_path).expect("jsondb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("JsonDb".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to jsondb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from jsondb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from jsondb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn kopperdb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("kopperdb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.kopperdb");
    let store = KopperdbSnapshotStore::new(&snapshot_path)
        .expect("kopperdb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("KopperDB".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to kopperdb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from kopperdb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from kopperdb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened =
        KopperdbSnapshotStore::new(&snapshot_path).expect("kopperdb snapshot store should reopen");
    let reopened_documents = reopened
        .list_documents()
        .expect("document catalog should reload from kopperdb");
    let reopened_snapshot = reopened
        .load_snapshot(&document.id)
        .expect("snapshot should reload from kopperdb")
        .expect("snapshot should exist after reopen");

    assert_eq!(reopened_documents, vec![document.clone()]);
    assert_eq!(reopened_snapshot.document, document);
    assert_eq!(reopened_snapshot.update, vec![1, 2, 3]);

    drop(reopened);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn rcask_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("rcask-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.rcask");
    let store =
        RcaskSnapshotStore::new(&snapshot_path).expect("rcask snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("RCask".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to rcask");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from rcask");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from rcask")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened =
        RcaskSnapshotStore::new(&snapshot_path).expect("rcask snapshot store should reopen");
    let reopened_documents = reopened
        .list_documents()
        .expect("document catalog should reload from rcask");
    let reopened_snapshot = reopened
        .load_snapshot(&document.id)
        .expect("snapshot should reload from rcask")
        .expect("snapshot should exist after reopen");

    assert_eq!(reopened_documents, vec![document.clone()]);
    assert_eq!(reopened_snapshot.document, document);
    assert_eq!(reopened_snapshot.update, vec![1, 2, 3]);

    drop(reopened);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn jfs_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("jfs-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.jfs.json");
    let store =
        JfsSnapshotStore::new(&snapshot_path).expect("jfs snapshot store should initialize");
    let document = backend::models::document::Document::new(Uuid::new_v4(), Some("JFS".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to jfs");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from jfs");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from jfs")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn json_store_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("json-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.json_store.jsonl");
    let store = JsonStoreSnapshotStore::new(&snapshot_path)
        .expect("json_store snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("JsonStore".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to json_store");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from json_store");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from json_store")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn koit_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("koit-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.koit.json");
    let store =
        KoitSnapshotStore::new(&snapshot_path).expect("koit snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Koit".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to koit");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from koit");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from koit")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn lite_db_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("lite-db-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.lite_db");
    let store =
        LiteDbSnapshotStore::new(&snapshot_path).expect("lite_db snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("LiteDb".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to lite_db");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from lite_db");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from lite_db")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store =
        LiteDbSnapshotStore::new(&snapshot_path).expect("lite_db snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from lite_db"),
        vec![document]
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn log_kv_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("log-kv-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.log_kv");
    let store =
        LogKvSnapshotStore::new(&snapshot_path).expect("log_kv snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("LogKV".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to log_kv");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from log_kv");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from log_kv")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store =
        LogKvSnapshotStore::new(&snapshot_path).expect("log_kv snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from log_kv"),
        vec![document.clone()]
    );
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("snapshot should reload from log_kv")
            .is_some()
    );

    reopened_store
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from log_kv");
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("deleted snapshot lookup should succeed")
            .is_none()
    );
    assert!(
        reopened_store
            .list_documents()
            .expect("document catalog should reflect log_kv deletion")
            .is_empty()
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn append_kv_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("append-kv-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.append_kv");
    let store = AppendKvSnapshotStore::new(&snapshot_path)
        .expect("append_kv snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("AppendKV".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to append_kv");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from append_kv");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from append_kv")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store =
        AppendKvSnapshotStore::new(&snapshot_path).expect("append_kv snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from append_kv"),
        vec![document.clone()]
    );
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("snapshot should reload from append_kv")
            .is_some()
    );

    reopened_store
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from append_kv");
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("deleted snapshot lookup should succeed")
            .is_none()
    );
    assert!(
        reopened_store
            .list_documents()
            .expect("document catalog should reflect append_kv deletion")
            .is_empty()
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn mhdb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("mhdb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.mhdb");
    let store =
        MhdbSnapshotStore::new(&snapshot_path).expect("mhdb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("MHdb".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1; 2048]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to mhdb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from mhdb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from mhdb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1; 2048]);

    drop(store);

    let reopened_store =
        MhdbSnapshotStore::new(&snapshot_path).expect("mhdb snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from mhdb"),
        vec![document.clone()]
    );
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("snapshot should reload from mhdb")
            .is_some()
    );

    reopened_store
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from mhdb");
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("deleted snapshot lookup should succeed")
            .is_none()
    );
    assert!(
        reopened_store
            .list_documents()
            .expect("document catalog should reflect mhdb deletion")
            .is_empty()
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn loro_kv_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("loro-kv-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.loro_kv");
    let store =
        LoroKvSnapshotStore::new(&snapshot_path).expect("loro_kv snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("LoroKV".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to loro_kv");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from loro_kv");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from loro_kv")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store =
        LoroKvSnapshotStore::new(&snapshot_path).expect("loro_kv snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from loro_kv"),
        vec![document.clone()]
    );
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("snapshot should reload from loro_kv")
            .is_some()
    );

    reopened_store
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from loro_kv");
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("deleted snapshot lookup should succeed")
            .is_none()
    );
    assert!(
        reopened_store
            .list_documents()
            .expect("document catalog should reflect loro_kv deletion")
            .is_empty()
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn luckdb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("luckdb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.luckdb.json");
    let store =
        LuckdbSnapshotStore::new(&snapshot_path).expect("luckdb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("LuckDB".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to luckdb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from luckdb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from luckdb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store =
        LuckdbSnapshotStore::new(&snapshot_path).expect("luckdb snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from luckdb"),
        vec![document.clone()]
    );
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("snapshot should reload from luckdb")
            .is_some()
    );

    reopened_store
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from luckdb");
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("deleted snapshot lookup should succeed")
            .is_none()
    );
    assert!(
        reopened_store
            .list_documents()
            .expect("document catalog should reflect luckdb deletion")
            .is_empty()
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn ipjdb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("ipjdb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.ipjdb");
    let store =
        IpjdbSnapshotStore::new(&snapshot_path).expect("ipjdb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("ipjdb".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to ipjdb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from ipjdb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from ipjdb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store =
        IpjdbSnapshotStore::new(&snapshot_path).expect("ipjdb snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from ipjdb"),
        vec![document.clone()]
    );
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("snapshot should reload from ipjdb")
            .is_some()
    );

    reopened_store
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from ipjdb");
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("deleted snapshot lookup should succeed")
            .is_none()
    );
    assert!(
        reopened_store
            .list_documents()
            .expect("document catalog should reflect ipjdb deletion")
            .is_empty()
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn rubin_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("rubin-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.rubin.json");
    let store =
        RubinSnapshotStore::new(&snapshot_path).expect("rubin snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Rubin".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to rubin");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from rubin");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from rubin")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store =
        RubinSnapshotStore::new(&snapshot_path).expect("rubin snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from rubin"),
        vec![document.clone()]
    );
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("snapshot should reload from rubin")
            .is_some()
    );

    reopened_store
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from rubin");
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("deleted snapshot lookup should succeed")
            .is_none()
    );
    assert!(
        reopened_store
            .list_documents()
            .expect("document catalog should reflect rubin deletion")
            .is_empty()
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn lsm_storage_engine_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("lsm-storage-engine-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.lsm_storage_engine");
    let store = LsmStorageEngineSnapshotStore::new(&snapshot_path)
        .expect("lsm_storage_engine snapshot store should initialize");
    let document = backend::models::document::Document::new(
        Uuid::new_v4(),
        Some("LSM storage engine".to_owned()),
    );
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to lsm_storage_engine");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from lsm_storage_engine");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from lsm_storage_engine")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store = LsmStorageEngineSnapshotStore::new(&snapshot_path)
        .expect("lsm_storage_engine snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from lsm_storage_engine"),
        vec![document]
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn lsm_engine_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("lsm-engine-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.lsm_engine");
    let store = LsmEngineSnapshotStore::new(&snapshot_path)
        .expect("lsm_engine snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("LSM Engine".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to lsm_engine");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from lsm_engine");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from lsm_engine")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store = LsmEngineSnapshotStore::new(&snapshot_path)
        .expect("lsm_engine snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from lsm_engine"),
        vec![document.clone()]
    );
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("snapshot should reload from lsm_engine")
            .is_some()
    );

    reopened_store
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from lsm_engine");
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("deleted snapshot lookup should succeed")
            .is_none()
    );
    assert!(
        reopened_store
            .list_documents()
            .expect("document catalog should reflect lsm_engine deletion")
            .is_empty()
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn etchdb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("etchdb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.etchdb");
    let store =
        EtchdbSnapshotStore::new(&snapshot_path).expect("etchdb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("EtchDB".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to etchdb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from etchdb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from etchdb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store =
        EtchdbSnapshotStore::new(&snapshot_path).expect("etchdb snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from etchdb"),
        vec![document.clone()]
    );
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("snapshot should reload from etchdb")
            .is_some()
    );

    reopened_store
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from etchdb");
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("deleted snapshot lookup should succeed")
            .is_none()
    );
    assert!(
        reopened_store
            .list_documents()
            .expect("document catalog should reflect etchdb deletion")
            .is_empty()
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn apex_store_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("apex-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.apex_store");
    let store = ApexStoreSnapshotStore::new(&snapshot_path)
        .expect("apex_store snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("ApexStore".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to apex_store");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from apex_store");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from apex_store")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store = ApexStoreSnapshotStore::new(&snapshot_path)
        .expect("apex_store snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from apex_store"),
        vec![document.clone()]
    );
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("snapshot should reload from apex_store")
            .is_some()
    );

    reopened_store
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from apex_store");
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("deleted snapshot lookup should succeed")
            .is_none()
    );
    assert!(
        reopened_store
            .list_documents()
            .expect("document catalog should reflect apex_store deletion")
            .is_empty()
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn lsmdb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("lsmdb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.lsmdb");
    let store =
        LsmdbSnapshotStore::new(&snapshot_path).expect("lsmdb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("LSMDB".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to lsmdb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from lsmdb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from lsmdb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store =
        LsmdbSnapshotStore::new(&snapshot_path).expect("lsmdb snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from lsmdb"),
        vec![document.clone()]
    );
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("snapshot should reload from lsmdb")
            .is_some()
    );

    reopened_store
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from lsmdb");
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("deleted snapshot lookup should succeed")
            .is_none()
    );
    assert!(
        reopened_store
            .list_documents()
            .expect("document catalog should reflect lsmdb deletion")
            .is_empty()
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn ferrumdb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("ferrumdb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.ferrumdb");
    let store = FerrumdbSnapshotStore::new(&snapshot_path)
        .expect("ferrumdb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("FerrumDB".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to ferrumdb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from ferrumdb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from ferrumdb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store =
        FerrumdbSnapshotStore::new(&snapshot_path).expect("ferrumdb snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from ferrumdb"),
        vec![document.clone()]
    );

    reopened_store
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from ferrumdb");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should be empty after ferrumdb delete"),
        Vec::new()
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn mmdb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("mmdb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.mmdb");
    let store =
        MmdbSnapshotStore::new(&snapshot_path).expect("mmdb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("MMDB".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to mmdb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from mmdb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from mmdb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store =
        MmdbSnapshotStore::new(&snapshot_path).expect("mmdb snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from mmdb"),
        vec![document.clone()]
    );

    reopened_store
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from mmdb");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should be empty after mmdb delete"),
        Vec::new()
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn mindb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("mindb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.mindb");
    let store =
        MindbSnapshotStore::new(&snapshot_path).expect("mindb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Mindb".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to mindb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from mindb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from mindb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store =
        MindbSnapshotStore::new(&snapshot_path).expect("mindb snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from mindb"),
        vec![document.clone()]
    );

    reopened_store
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from mindb");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should be empty after mindb delete"),
        Vec::new()
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn nanodb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("nanodb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.nanodb.json");
    let store =
        NanodbSnapshotStore::new(&snapshot_path).expect("nanodb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("NanoDB".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to nanodb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from nanodb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from nanodb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store =
        NanodbSnapshotStore::new(&snapshot_path).expect("nanodb snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from nanodb"),
        vec![document.clone()]
    );

    reopened_store
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from nanodb");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should be empty after nanodb delete"),
        Vec::new()
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn graus_db_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("graus_db-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.graus_db");
    let store = GrausDbSnapshotStore::new(&snapshot_path)
        .expect("graus_db snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("GrausDb".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to graus_db");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from graus_db");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from graus_db")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store =
        GrausDbSnapshotStore::new(&snapshot_path).expect("graus_db snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from graus_db"),
        vec![document.clone()]
    );

    reopened_store
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from graus_db");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should be empty after graus_db delete"),
        Vec::new()
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn yakv_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("yakv-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.yakv");
    let store =
        YakvSnapshotStore::new(&snapshot_path).expect("yakv snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("YAKV".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to yakv");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from yakv");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from yakv")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn yakvdb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("yakvdb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.yakvdb");
    let store =
        YakvdbSnapshotStore::new(&snapshot_path).expect("yakvdb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("YAKVDB".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to yakvdb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from yakvdb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from yakvdb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store = YakvdbSnapshotStore::new(&snapshot_path)
        .expect("yakvdb snapshot store should reopen existing database");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should survive yakvdb reopen"),
        vec![document.clone()]
    );

    reopened_store
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from yakvdb");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should be empty after yakvdb delete"),
        Vec::new()
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn kv_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("kv-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.kv");
    let store = KvSnapshotStore::new(&snapshot_path).expect("kv snapshot store should initialize");
    let document = backend::models::document::Document::new(Uuid::new_v4(), Some("KV".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to kv");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from kv");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from kv")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn eight_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("eight-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.eight");
    let store =
        EightSnapshotStore::new(&snapshot_path).expect("eight snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Eight".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to eight");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from eight");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from eight")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn epoch_db_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("epoch-db-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.epoch_db");
    let store = EpochDbSnapshotStore::new(&snapshot_path)
        .expect("epoch-db snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("EpochDB".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to epoch-db");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from epoch-db");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from epoch-db")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn rumdb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("rumdb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.rumdb");
    let store =
        RumDbSnapshotStore::new(&snapshot_path).expect("rumdb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("RumDB".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to rumdb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from rumdb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from rumdb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn rustcask_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("rustcask-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.rustcask");
    let store = RustcaskSnapshotStore::new(&snapshot_path)
        .expect("rustcask snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Rustcask".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to rustcask");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from rustcask");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from rustcask")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn rusty_leveldb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("rusty-leveldb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.rusty_leveldb");
    let store = RustyLeveldbSnapshotStore::new(&snapshot_path)
        .expect("rusty-leveldb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Rusty LevelDB".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to rusty-leveldb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from rusty-leveldb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from rusty-leveldb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn hmdb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("hmdb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let store =
        HmdbSnapshotStore::new(&snapshot_dir).expect("hmdb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("HmDb".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to hmdb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from hmdb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from hmdb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn icefalldb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("icefalldb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.icefalldb");
    let store = IcefalldbSnapshotStore::new(&snapshot_path)
        .expect("icefalldb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("IcefallDB".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to icefalldb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from icefalldb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from icefalldb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened = IcefalldbSnapshotStore::new(&snapshot_path)
        .expect("icefalldb snapshot store should reopen");
    let reopened_documents = reopened
        .list_documents()
        .expect("document catalog should reload from icefalldb");
    let reopened_snapshot = reopened
        .load_snapshot(&document.id)
        .expect("snapshot should reload from icefalldb")
        .expect("snapshot should exist after reopen");

    assert_eq!(reopened_documents, vec![document.clone()]);
    assert_eq!(reopened_snapshot.document, document);
    assert_eq!(reopened_snapshot.update, vec![1, 2, 3]);

    drop(reopened);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn blockbucket_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("blockbucket-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.blockbucket");
    let store = BlockbucketSnapshotStore::new(&snapshot_path)
        .expect("blockbucket snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Blockbucket".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to blockbucket");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from blockbucket");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from blockbucket")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened = BlockbucketSnapshotStore::new(&snapshot_path)
        .expect("blockbucket snapshot store should reopen");
    let reopened_documents = reopened
        .list_documents()
        .expect("document catalog should reload from blockbucket");
    let reopened_snapshot = reopened
        .load_snapshot(&document.id)
        .expect("snapshot should reload from blockbucket")
        .expect("snapshot should exist after reopen");

    assert_eq!(reopened_documents, vec![document.clone()]);
    assert_eq!(reopened_snapshot.document, document);
    assert_eq!(reopened_snapshot.update, vec![1, 2, 3]);

    drop(reopened);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn blazeup_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("blazeup-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.blazeup");
    let store = BlazeupSnapshotStore::new(&snapshot_path)
        .expect("blazeup snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Blazeup".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to blazeup");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from blazeup");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from blazeup")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store =
        BlazeupSnapshotStore::new(&snapshot_path).expect("blazeup snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from blazeup"),
        vec![document.clone()]
    );
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("snapshot should reload from blazeup")
            .is_some()
    );

    reopened_store
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from blazeup");
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("deleted snapshot lookup should succeed")
            .is_none()
    );
    assert!(
        reopened_store
            .list_documents()
            .expect("document catalog should reflect blazeup deletion")
            .is_empty()
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn bitask_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("bitask-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let store =
        BitaskSnapshotStore::new(&snapshot_dir).expect("bitask snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Bitask".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to bitask");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from bitask");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from bitask")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn bitkv_rs_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("bitkv-rs-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let store = BitkvRsSnapshotStore::new(&snapshot_dir)
        .expect("bitkv-rs snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Bitkv-rs".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to bitkv-rs");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from bitkv-rs");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from bitkv-rs")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn candystore_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("candystore-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.candystore");
    let store = CandystoreSnapshotStore::new(&snapshot_path)
        .expect("candystore snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Candystore".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to candystore");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from candystore");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from candystore")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn cuendillar_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("cuendillar-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.cuendillar");
    let store = CuendillarSnapshotStore::new(&snapshot_path)
        .expect("cuendillar snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Cuendillar".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to cuendillar");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from cuendillar");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from cuendillar")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened = CuendillarSnapshotStore::new(&snapshot_path)
        .expect("cuendillar snapshot store should reopen");
    let reopened_documents = reopened
        .list_documents()
        .expect("document catalog should reload from cuendillar");
    let reopened_snapshot = reopened
        .load_snapshot(&document.id)
        .expect("snapshot should reload from cuendillar")
        .expect("snapshot should exist after reopen");

    assert_eq!(reopened_documents, vec![document.clone()]);
    assert_eq!(reopened_snapshot.document, document);
    assert_eq!(reopened_snapshot.update, vec![1, 2, 3]);

    drop(reopened);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn native_db_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("native-db-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.native_db");
    let store = NativeDbSnapshotStore::new(&snapshot_path)
        .expect("native_db snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Native DB".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to native_db store");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from native_db store");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from native_db store")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn nebari_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("nebari-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_root = snapshot_dir.join("snapshots.nebari");
    let store =
        NebariSnapshotStore::new(&snapshot_root).expect("nebari snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Nebari".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to nebari");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from nebari");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from nebari")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn persistent_kv_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("persistent-kv-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.persistent_kv");
    let store = PersistentKvSnapshotStore::new(&snapshot_path)
        .expect("persistent_kv snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("PersistentKV".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to persistent_kv");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from persistent_kv");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from persistent_kv")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn nodb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("nodb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.nodb");
    let store =
        NodbSnapshotStore::new(&snapshot_path).expect("nodb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("NoDb".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to nodb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from nodb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from nodb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn okofdb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("okofdb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.okofdb");
    let store =
        OkofdbSnapshotStore::new(&snapshot_path).expect("okofdb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Okofdb".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to okofdb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from okofdb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from okofdb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn celerix_store_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("celerix-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.celerix_store");
    let store = CelerixStoreSnapshotStore::new(&snapshot_path)
        .expect("celerix_store snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Celerix".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to celerix_store");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from celerix_store");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from celerix_store")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    store
        .delete_snapshot(&loaded_snapshot.document.id)
        .expect("snapshot should delete from celerix_store");
    assert!(
        store
            .load_snapshot(&loaded_snapshot.document.id)
            .expect("deleted snapshot lookup should succeed")
            .is_none()
    );

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn citadeldb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("citadeldb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.citadeldb");
    let store = CitadeldbSnapshotStore::new(
        &snapshot_path,
        b"test-citadel-snapshot-passphrase".as_slice(),
    )
    .expect("citadeldb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("CitadelDB".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to citadeldb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from citadeldb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from citadeldb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document.clone());
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    let reopened_store = CitadeldbSnapshotStore::new(
        &snapshot_path,
        b"test-citadel-snapshot-passphrase".as_slice(),
    )
    .expect("citadeldb snapshot store should reopen");
    assert_eq!(
        reopened_store
            .list_documents()
            .expect("document catalog should reload from citadeldb"),
        vec![document.clone()]
    );
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("snapshot should reload from citadeldb")
            .is_some()
    );

    reopened_store
        .delete_snapshot(&document.id)
        .expect("snapshot should delete from citadeldb");
    assert!(
        reopened_store
            .load_snapshot(&document.id)
            .expect("deleted snapshot lookup should succeed")
            .is_none()
    );
    assert!(
        reopened_store
            .list_documents()
            .expect("document catalog should reflect citadeldb deletion")
            .is_empty()
    );

    drop(reopened_store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn nikidb_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("nikidb-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.nikidb");
    let store =
        NikidbSnapshotStore::new(&snapshot_path).expect("nikidb snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("NikiDb".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to nikidb");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from nikidb");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from nikidb")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn parity_db_snapshot_store_round_trips_document_catalog() {
    let snapshot_dir = temp_snapshot_dir("parity-db-store-roundtrip");
    fs::create_dir_all(&snapshot_dir).expect("test snapshot directory should be created");
    let snapshot_path = snapshot_dir.join("snapshots.parity_db");
    let store = ParityDbSnapshotStore::new(&snapshot_path)
        .expect("parity_db snapshot store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("ParityDB".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to parity_db store");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from parity_db store");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from parity_db store")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);

    drop(store);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn s3_snapshot_store_round_trips_document_catalog() {
    let runtime = tokio::runtime::Runtime::new().expect("tokio runtime should initialize");
    let harness = runtime.block_on(spawn_mock_s3_snapshot_service());
    let store = S3SnapshotStore::new(
        &harness.endpoint,
        "us-east-1",
        &harness.bucket,
        "snapshots/unit-tests/",
        &harness.access_key_id,
        &harness.secret_access_key,
        None,
        Duration::from_secs(5),
        true,
    )
    .expect("s3 snapshot store should initialize");
    let document = backend::models::document::Document::new(Uuid::new_v4(), Some("S3".to_owned()));
    let snapshot = DocumentSnapshot::new(document.clone(), vec![1, 2, 3]);

    store
        .save_snapshot(snapshot)
        .expect("snapshot should save to s3 store");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should load from s3 store");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from s3 store")
        .expect("snapshot should exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);
    assert!(
        harness
            .state
            .last_authorization()
            .is_some_and(|header| header.contains("Credential=test-access-key/")),
        "s3 requests should be signed with the configured access key"
    );
}

#[test]
fn sqlite_snapshot_store_skips_corrupt_rows_when_listing_documents() {
    let snapshot_dir = temp_snapshot_dir("sqlite-store-corrupt-catalog");
    let snapshot_path = snapshot_dir.join("snapshots.sqlite3");
    let store = SqliteSnapshotStore::new(&snapshot_path).expect("sqlite store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Catalog".to_owned()));

    store
        .save_snapshot(DocumentSnapshot::new(document.clone(), vec![7, 8, 9]))
        .expect("valid snapshot should save");

    let corrupt_doc_id = Uuid::new_v4();
    let connection =
        rusqlite::Connection::open(&snapshot_path).expect("sqlite file should be writable");
    connection
        .execute(
            "INSERT INTO snapshots (doc_id, title, created_at, updated_at, access_token, update_bytes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                corrupt_doc_id.to_string(),
                "Corrupt",
                "not-a-timestamp",
                "not-a-timestamp",
                "token",
                vec![1_u8, 2, 3]
            ],
        )
        .expect("corrupt sqlite snapshot row should be written");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should skip corrupt sqlite rows");
    let corrupt_snapshot_error = store
        .load_snapshot(&corrupt_doc_id)
        .expect_err("directly loading a corrupt sqlite snapshot should still fail");

    assert_eq!(listed_documents, vec![document]);
    assert!(matches!(
        corrupt_snapshot_error,
        backend::storage::StorageError::CorruptSnapshot(id) if id == corrupt_doc_id
    ));

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn file_snapshot_store_replaces_existing_snapshot_without_leaking_temp_files() {
    let snapshot_dir = temp_snapshot_dir("file-store-atomic-save");
    let store = FileSnapshotStore::new(&snapshot_dir).expect("file store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Atomic".to_owned()));

    store
        .save_snapshot(DocumentSnapshot::new(document.clone(), vec![1, 2, 3]))
        .expect("initial snapshot should save");
    store
        .save_snapshot(DocumentSnapshot::new(document.clone(), vec![9, 8, 7]))
        .expect("replacement snapshot should save");

    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should load from file store")
        .expect("snapshot should exist");
    let directory_entries = fs::read_dir(&snapshot_dir)
        .expect("snapshot directory should be readable")
        .map(|entry| entry.expect("snapshot entry should be readable").path())
        .collect::<Vec<_>>();
    let json_entries = directory_entries
        .iter()
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .count();
    let temp_entries = directory_entries
        .iter()
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("tmp"))
        .count();

    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![9, 8, 7]);
    assert_eq!(json_entries, 1);
    assert_eq!(temp_entries, 0);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn file_snapshot_store_ignores_stale_temp_files_when_listing_documents() {
    let snapshot_dir = temp_snapshot_dir("file-store-stale-temp");
    let store = FileSnapshotStore::new(&snapshot_dir).expect("file store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Atomic".to_owned()));

    store
        .save_snapshot(DocumentSnapshot::new(document.clone(), vec![4, 5, 6]))
        .expect("snapshot should save");

    let stale_temp_path = snapshot_dir.join(format!("{}.json.{}.tmp", document.id, Uuid::new_v4()));
    fs::write(&stale_temp_path, br#"{"partial":true}"#)
        .expect("stale temp snapshot fixture should be written");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should ignore stale temp files");
    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("snapshot should still load from file store")
        .expect("snapshot should still exist");

    assert_eq!(listed_documents, vec![document.clone()]);
    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![4, 5, 6]);
    assert!(stale_temp_path.exists());

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn file_snapshot_store_delete_snapshot_removes_matching_stale_temp_files() {
    let snapshot_dir = temp_snapshot_dir("file-store-delete-stale-temp");
    let store = FileSnapshotStore::new(&snapshot_dir).expect("file store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Atomic".to_owned()));
    let unrelated_doc_id = Uuid::new_v4();

    store
        .save_snapshot(DocumentSnapshot::new(document.clone(), vec![4, 5, 6]))
        .expect("snapshot should save");

    let matching_temp_path =
        snapshot_dir.join(format!("{}.json.{}.tmp", document.id, Uuid::new_v4()));
    let unrelated_temp_path =
        snapshot_dir.join(format!("{}.json.{}.tmp", unrelated_doc_id, Uuid::new_v4()));
    fs::write(&matching_temp_path, br#"{"partial":true}"#)
        .expect("matching stale temp snapshot fixture should be written");
    fs::write(&unrelated_temp_path, br#"{"partial":true}"#)
        .expect("unrelated stale temp snapshot fixture should be written");

    store
        .delete_snapshot(&document.id)
        .expect("delete should remove snapshot and matching temp files");

    assert!(
        store
            .load_snapshot(&document.id)
            .expect("snapshot lookup should succeed")
            .is_none()
    );
    assert!(!matching_temp_path.exists());
    assert!(unrelated_temp_path.exists());

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[cfg(unix)]
#[test]
fn file_snapshot_store_preserves_previous_snapshot_when_atomic_replace_cannot_write_temp_file() {
    use std::os::unix::fs::PermissionsExt;

    let snapshot_dir = temp_snapshot_dir("file-store-atomic-save-failure");
    let store = FileSnapshotStore::new(&snapshot_dir).expect("file store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Atomic".to_owned()));

    store
        .save_snapshot(DocumentSnapshot::new(document.clone(), vec![1, 2, 3]))
        .expect("initial snapshot should save");

    let original_permissions = fs::metadata(&snapshot_dir)
        .expect("snapshot directory metadata should be readable")
        .permissions();
    let mut readonly_permissions = original_permissions.clone();
    readonly_permissions.set_mode(0o555);
    fs::set_permissions(&snapshot_dir, readonly_permissions)
        .expect("snapshot directory should become read-only");

    let failed_save = store.save_snapshot(DocumentSnapshot::new(document.clone(), vec![9, 8, 7]));

    fs::set_permissions(&snapshot_dir, original_permissions)
        .expect("snapshot directory permissions should be restored");

    assert!(matches!(
        failed_save,
        Err(backend::storage::StorageError::Io(message)) if message.contains(".tmp")
    ));

    let loaded_snapshot = store
        .load_snapshot(&document.id)
        .expect("original snapshot should still load")
        .expect("original snapshot should still exist");
    let temp_entries = fs::read_dir(&snapshot_dir)
        .expect("snapshot directory should be readable")
        .map(|entry| entry.expect("snapshot entry should be readable").path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("tmp"))
        .count();

    assert_eq!(loaded_snapshot.document, document);
    assert_eq!(loaded_snapshot.update, vec![1, 2, 3]);
    assert_eq!(temp_entries, 0);

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}

#[test]
fn file_snapshot_store_skips_corrupt_snapshots_when_listing_documents() {
    let snapshot_dir = temp_snapshot_dir("file-store-corrupt-catalog");
    let store = FileSnapshotStore::new(&snapshot_dir).expect("file store should initialize");
    let document =
        backend::models::document::Document::new(Uuid::new_v4(), Some("Catalog".to_owned()));

    store
        .save_snapshot(DocumentSnapshot::new(document.clone(), vec![7, 8, 9]))
        .expect("valid snapshot should save");

    let corrupt_doc_id = Uuid::new_v4();
    fs::write(snapshot_dir.join(format!("{corrupt_doc_id}.json")), b"[]")
        .expect("corrupt snapshot fixture should be written");

    let listed_documents = store
        .list_documents()
        .expect("document catalog should skip corrupt snapshots");
    let corrupt_snapshot_error = store
        .load_snapshot(&corrupt_doc_id)
        .expect_err("directly loading a corrupt snapshot should still fail");

    assert_eq!(listed_documents, vec![document]);
    assert!(matches!(
        corrupt_snapshot_error,
        backend::storage::StorageError::CorruptSnapshot(id) if id == corrupt_doc_id
    ));

    fs::remove_dir_all(snapshot_dir).expect("test snapshot directory should be cleaned up");
}
