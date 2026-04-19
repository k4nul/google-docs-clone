use std::env;

use axum::http::HeaderValue;

use crate::errors::{AppError, AppResult};

pub const DEFAULT_HOST: &str = "127.0.0.1";
pub const DEFAULT_PORT: u16 = 4000;
pub const DEFAULT_FRONTEND_ORIGIN: &str = "http://localhost:3000";
pub const DEFAULT_RUST_LOG: &str = "backend=debug,tower_http=info";
pub const DEFAULT_API_TOKEN: &str = "dev-admin-token";
pub const DEFAULT_SNAPSHOT_STORE: &str = "memory";
pub const DEFAULT_SNAPSHOT_DIR: &str = "./data/snapshots";
pub const DEFAULT_ROOM_LOCATOR: &str = "local";
pub const DEFAULT_ROOM_COORDINATOR: &str = "noop";
pub const DEFAULT_ROOM_COORDINATOR_STATE_DIR: &str = "./data/room-coordinator";
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
    pub room_locator: String,
    pub room_coordinator: String,
    pub room_coordinator_state_dir: String,
    pub node_id: String,
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
        let room_locator = env_string("ROOM_LOCATOR", DEFAULT_ROOM_LOCATOR)?;
        let room_coordinator = env_string("ROOM_COORDINATOR", DEFAULT_ROOM_COORDINATOR)?;
        let room_coordinator_state_dir = env_string(
            "ROOM_COORDINATOR_STATE_DIR",
            DEFAULT_ROOM_COORDINATOR_STATE_DIR,
        )?;
        let node_id = env_string("NODE_ID", DEFAULT_NODE_ID)?;
        let room_owner_hints_path = env_optional_string("ROOM_OWNER_HINTS_PATH")?;

        Ok(Self {
            host,
            port,
            frontend_origin,
            rust_log,
            api_token,
            snapshot_store,
            snapshot_dir,
            room_locator,
            room_coordinator,
            room_coordinator_state_dir,
            node_id,
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
