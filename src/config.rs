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
pub const DEFAULT_SNAPSHOT_SQLITE_PATH: &str = "./data/snapshots.sqlite3";
pub const DEFAULT_SNAPSHOT_HEED_PATH: &str = "./data/snapshots.heed";
pub const DEFAULT_SNAPSHOT_JAMMDB_PATH: &str = "./data/snapshots.jammdb";
pub const DEFAULT_SNAPSHOT_FJALL_PATH: &str = "./data/snapshots.fjall";
pub const DEFAULT_SNAPSHOT_PERSY_PATH: &str = "./data/snapshots.persy";
pub const DEFAULT_SNAPSHOT_NATIVE_DB_PATH: &str = "./data/snapshots.native_db";
pub const DEFAULT_SNAPSHOT_PARITY_DB_PATH: &str = "./data/snapshots.parity_db";
pub const DEFAULT_SNAPSHOT_PICKLEDB_PATH: &str = "./data/snapshots.pickledb";
pub const DEFAULT_SNAPSHOT_MICROKV_PATH: &str = "./data/snapshots_microkv";
pub const DEFAULT_SNAPSHOT_REDB_PATH: &str = "./data/snapshots.redb";
pub const DEFAULT_SNAPSHOT_READB_PATH: &str = "./data/snapshots.readb";
pub const DEFAULT_SNAPSHOT_SLED_PATH: &str = "./data/snapshots.sled";
pub const DEFAULT_SNAPSHOT_RUSTBREAK_PATH: &str = "./data/snapshots.rustbreak";
pub const DEFAULT_SNAPSHOT_YEDB_PATH: &str = "./data/snapshots.yedb";
pub const DEFAULT_SNAPSHOT_BTREE_STORE_PATH: &str = "./data/snapshots.btree_store";
pub const DEFAULT_SNAPSHOT_SIAMESDB_PATH: &str = "./data/snapshots.siamesedb";
pub const DEFAULT_SNAPSHOT_STRUCTSY_PATH: &str = "./data/snapshots.structsy";
pub const DEFAULT_SNAPSHOT_ABYSSINIANDB_PATH: &str = "./data/snapshots.abyssiniandb";
pub const DEFAULT_SNAPSHOT_THUNDERDB_PATH: &str = "./data/snapshots.thunderdb";
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
    pub snapshot_sqlite_path: String,
    pub snapshot_heed_path: String,
    pub snapshot_jammdb_path: String,
    pub snapshot_fjall_path: String,
    pub snapshot_persy_path: String,
    pub snapshot_native_db_path: String,
    pub snapshot_parity_db_path: String,
    pub snapshot_pickledb_path: String,
    pub snapshot_microkv_path: String,
    pub snapshot_redb_path: String,
    pub snapshot_readb_path: String,
    pub snapshot_sled_path: String,
    pub snapshot_rustbreak_path: String,
    pub snapshot_yedb_path: String,
    pub snapshot_btree_store_path: String,
    pub snapshot_siamesedb_path: String,
    pub snapshot_structsy_path: String,
    pub snapshot_abyssiniandb_path: String,
    pub snapshot_thunderdb_path: String,
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
        let snapshot_sqlite_path =
            env_string("SNAPSHOT_SQLITE_PATH", DEFAULT_SNAPSHOT_SQLITE_PATH)?;
        let snapshot_heed_path = env_string("SNAPSHOT_HEED_PATH", DEFAULT_SNAPSHOT_HEED_PATH)?;
        let snapshot_jammdb_path =
            env_string("SNAPSHOT_JAMMDB_PATH", DEFAULT_SNAPSHOT_JAMMDB_PATH)?;
        let snapshot_fjall_path = env_string("SNAPSHOT_FJALL_PATH", DEFAULT_SNAPSHOT_FJALL_PATH)?;
        let snapshot_persy_path = env_string("SNAPSHOT_PERSY_PATH", DEFAULT_SNAPSHOT_PERSY_PATH)?;
        let snapshot_native_db_path =
            env_string("SNAPSHOT_NATIVE_DB_PATH", DEFAULT_SNAPSHOT_NATIVE_DB_PATH)?;
        let snapshot_parity_db_path =
            env_string("SNAPSHOT_PARITY_DB_PATH", DEFAULT_SNAPSHOT_PARITY_DB_PATH)?;
        let snapshot_pickledb_path =
            env_string("SNAPSHOT_PICKLEDB_PATH", DEFAULT_SNAPSHOT_PICKLEDB_PATH)?;
        let snapshot_microkv_path =
            env_string("SNAPSHOT_MICROKV_PATH", DEFAULT_SNAPSHOT_MICROKV_PATH)?;
        let snapshot_redb_path = env_string("SNAPSHOT_REDB_PATH", DEFAULT_SNAPSHOT_REDB_PATH)?;
        let snapshot_readb_path = env_string("SNAPSHOT_READB_PATH", DEFAULT_SNAPSHOT_READB_PATH)?;
        let snapshot_sled_path = env_string("SNAPSHOT_SLED_PATH", DEFAULT_SNAPSHOT_SLED_PATH)?;
        let snapshot_rustbreak_path =
            env_string("SNAPSHOT_RUSTBREAK_PATH", DEFAULT_SNAPSHOT_RUSTBREAK_PATH)?;
        let snapshot_yedb_path = env_string("SNAPSHOT_YEDB_PATH", DEFAULT_SNAPSHOT_YEDB_PATH)?;
        let snapshot_btree_store_path = env_string(
            "SNAPSHOT_BTREE_STORE_PATH",
            DEFAULT_SNAPSHOT_BTREE_STORE_PATH,
        )?;
        let snapshot_siamesedb_path =
            env_string("SNAPSHOT_SIAMESDB_PATH", DEFAULT_SNAPSHOT_SIAMESDB_PATH)?;
        let snapshot_structsy_path =
            env_string("SNAPSHOT_STRUCTSY_PATH", DEFAULT_SNAPSHOT_STRUCTSY_PATH)?;
        let snapshot_abyssiniandb_path = env_string(
            "SNAPSHOT_ABYSSINIANDB_PATH",
            DEFAULT_SNAPSHOT_ABYSSINIANDB_PATH,
        )?;
        let snapshot_thunderdb_path =
            env_string("SNAPSHOT_THUNDERDB_PATH", DEFAULT_SNAPSHOT_THUNDERDB_PATH)?;
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
            snapshot_sqlite_path,
            snapshot_heed_path,
            snapshot_jammdb_path,
            snapshot_fjall_path,
            snapshot_persy_path,
            snapshot_native_db_path,
            snapshot_parity_db_path,
            snapshot_pickledb_path,
            snapshot_microkv_path,
            snapshot_redb_path,
            snapshot_readb_path,
            snapshot_sled_path,
            snapshot_rustbreak_path,
            snapshot_yedb_path,
            snapshot_btree_store_path,
            snapshot_siamesedb_path,
            snapshot_structsy_path,
            snapshot_abyssiniandb_path,
            snapshot_thunderdb_path,
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
