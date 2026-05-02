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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> Config {
        Config {
            host: "127.0.0.1".to_owned(),
            port: 4000,
            frontend_origin: "http://localhost:3000".to_owned(),
            rust_log: "backend=debug".to_owned(),
            api_token: "test-token".to_owned(),
            snapshot_store: "memory".to_owned(),
            snapshot_dir: "./data/snapshots".to_owned(),
            room_locator: "local".to_owned(),
            node_id: "local-node".to_owned(),
            room_owner_hints_path: None,
        }
    }

    #[test]
    fn bind_address_formats_host_and_port() {
        assert_eq!(test_config().bind_address(), "127.0.0.1:4000");
    }

    #[test]
    fn bind_address_uses_custom_host_and_port() {
        let config = Config {
            host: "0.0.0.0".to_owned(),
            port: 8080,
            ..test_config()
        };
        assert_eq!(config.bind_address(), "0.0.0.0:8080");
    }

    #[test]
    fn frontend_origin_header_returns_header_value_matching_origin_string() {
        let config = test_config();
        let header = config
            .frontend_origin_header()
            .expect("valid origin should produce a header value");
        assert_eq!(header.to_str().unwrap(), "http://localhost:3000");
    }

    #[test]
    fn frontend_origin_header_rejects_origin_containing_newline() {
        let config = Config {
            frontend_origin: "http://bad-origin\nX-Injected: evil".to_owned(),
            ..test_config()
        };
        let error = config
            .frontend_origin_header()
            .expect_err("origin with newline should be rejected");
        assert!(matches!(error, AppError::Config(_)));
    }
}
