use std::{collections::HashMap, fs, path::Path, sync::Arc};

use axum::http::Uri;
use serde::Deserialize;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    config::Config,
    errors::{AppError, AppResult},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedRoom {
    Local,
    Remote(RoomOwnerHint),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RoomOwnerHint {
    pub node_id: String,
    #[serde(default)]
    pub base_url: Option<String>,
}

impl RoomOwnerHint {
    fn normalized(mut self) -> Result<Self, RoomLocatorError> {
        let node_id = self.node_id.trim();
        if node_id.is_empty() {
            return Err(RoomLocatorError::Config(
                "room owner hint node_id cannot be empty".to_owned(),
            ));
        }
        self.node_id = node_id.to_owned();

        if let Some(base_url) = self.base_url.take() {
            let base_url = base_url.trim();
            if base_url.is_empty() {
                return Err(RoomLocatorError::Config(
                    "room owner hint base_url cannot be empty".to_owned(),
                ));
            }

            self.base_url = Some(normalize_owner_base_url(base_url)?);
        }

        Ok(self)
    }
}

fn normalize_owner_base_url(base_url: &str) -> Result<String, RoomLocatorError> {
    let invalid_message = || {
        RoomLocatorError::Config(format!(
            "room owner hint base_url must be an origin-only absolute http/https URL without path/query, received `{base_url}`"
        ))
    };

    let uri: Uri = base_url.parse().map_err(|_| {
        RoomLocatorError::Config(format!(
            "room owner hint base_url must be an origin-only absolute http/https URL without path/query, received `{base_url}`"
        ))
    })?;

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

#[derive(Debug, Error)]
pub enum RoomLocatorError {
    #[error("room ownership locator is misconfigured: {0}")]
    Config(String),
    #[error("room ownership lookup failed: {0}")]
    LookupFailed(String),
}

pub trait RoomLocator: Send + Sync {
    fn resolve(&self, doc_id: &Uuid) -> Result<ResolvedRoom, RoomLocatorError>;
}

#[derive(Debug, Default)]
pub struct LocalRoomLocator;

impl RoomLocator for LocalRoomLocator {
    fn resolve(&self, _doc_id: &Uuid) -> Result<ResolvedRoom, RoomLocatorError> {
        Ok(ResolvedRoom::Local)
    }
}

#[derive(Debug)]
pub struct StaticRoomLocator {
    current_node_id: String,
    document_owners: HashMap<Uuid, RoomOwnerHint>,
}

#[derive(Debug, Deserialize)]
struct StaticRoomOwnerHints {
    #[serde(default)]
    documents: HashMap<Uuid, RoomOwnerHint>,
}

impl StaticRoomLocator {
    pub fn new(
        current_node_id: impl Into<String>,
        document_owners: HashMap<Uuid, RoomOwnerHint>,
    ) -> Result<Self, RoomLocatorError> {
        let current_node_id = current_node_id.into();
        let current_node_id = current_node_id.trim();
        if current_node_id.is_empty() {
            return Err(RoomLocatorError::Config(
                "NODE_ID cannot be empty when ROOM_LOCATOR=static".to_owned(),
            ));
        }

        let document_owners = document_owners
            .into_iter()
            .map(|(doc_id, owner)| owner.normalized().map(|owner| (doc_id, owner)))
            .collect::<Result<HashMap<_, _>, _>>()?;

        Ok(Self {
            current_node_id: current_node_id.to_owned(),
            document_owners,
        })
    }

    pub fn from_json_file(
        current_node_id: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<Self, RoomLocatorError> {
        let path = path.as_ref();
        let bytes = fs::read(path).map_err(|error| {
            RoomLocatorError::LookupFailed(format!("{}: {error}", path.display()))
        })?;
        let hints: StaticRoomOwnerHints = serde_json::from_slice(&bytes)
            .map_err(|error| RoomLocatorError::Config(format!("{}: {error}", path.display())))?;

        Self::new(current_node_id, hints.documents)
    }
}

impl RoomLocator for StaticRoomLocator {
    fn resolve(&self, doc_id: &Uuid) -> Result<ResolvedRoom, RoomLocatorError> {
        match self.document_owners.get(doc_id) {
            Some(owner) if owner.node_id != self.current_node_id => {
                Ok(ResolvedRoom::Remote(owner.clone()))
            }
            _ => Ok(ResolvedRoom::Local),
        }
    }
}

pub fn local_room_locator() -> Arc<dyn RoomLocator> {
    Arc::new(LocalRoomLocator)
}

pub fn room_locator_from_config(config: &Config) -> AppResult<Arc<dyn RoomLocator>> {
    match config.room_locator.as_str() {
        "local" => Ok(local_room_locator()),
        "static" => {
            let path = config.room_owner_hints_path.as_ref().ok_or_else(|| {
                AppError::Config(
                    "ROOM_OWNER_HINTS_PATH is required when ROOM_LOCATOR=static".to_owned(),
                )
            })?;
            let locator = StaticRoomLocator::from_json_file(config.node_id.clone(), path).map_err(
                |error| match error {
                    RoomLocatorError::Config(message) => AppError::Config(message),
                    other => AppError::from(anyhow::Error::from(other)),
                },
            )?;
            Ok(Arc::new(locator))
        }
        other => Err(AppError::Config(format!(
            "ROOM_LOCATOR must be `local` or `static`, received `{other}`"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DEFAULT_NODE_ID;
    use std::path::PathBuf;

    fn temp_hints_path(test_name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("backend-{test_name}-{}.json", Uuid::new_v4()))
    }

    #[test]
    fn local_room_locator_marks_every_document_as_local() {
        let locator = LocalRoomLocator;
        let doc_id = Uuid::new_v4();

        assert_eq!(
            locator
                .resolve(&doc_id)
                .expect("local locator should always resolve locally"),
            ResolvedRoom::Local
        );
    }

    #[test]
    fn static_room_locator_marks_other_node_document_as_remote() {
        let doc_id = Uuid::new_v4();
        let locator = StaticRoomLocator::new(
            "node-a",
            HashMap::from([(
                doc_id,
                RoomOwnerHint {
                    node_id: "node-b".to_owned(),
                    base_url: Some("http://node-b.internal:4000".to_owned()),
                },
            )]),
        )
        .expect("static locator should initialize");

        assert_eq!(
            locator
                .resolve(&doc_id)
                .expect("doc should resolve through static locator"),
            ResolvedRoom::Remote(RoomOwnerHint {
                node_id: "node-b".to_owned(),
                base_url: Some("http://node-b.internal:4000".to_owned()),
            })
        );
    }

    #[test]
    fn static_room_locator_marks_current_node_document_as_local() {
        let doc_id = Uuid::new_v4();
        let locator = StaticRoomLocator::new(
            "node-a",
            HashMap::from([(
                doc_id,
                RoomOwnerHint {
                    node_id: "node-a".to_owned(),
                    base_url: Some("http://node-a.internal:4000".to_owned()),
                },
            )]),
        )
        .expect("static locator should initialize");

        assert_eq!(
            locator
                .resolve(&doc_id)
                .expect("doc should resolve through static locator"),
            ResolvedRoom::Local
        );
    }

    #[test]
    fn static_room_locator_rejects_invalid_owner_base_url() {
        let doc_id = Uuid::new_v4();
        let error = StaticRoomLocator::new(
            "node-a",
            HashMap::from([(
                doc_id,
                RoomOwnerHint {
                    node_id: "node-b".to_owned(),
                    base_url: Some("node-b.internal:4000".to_owned()),
                },
            )]),
        )
        .expect_err("relative owner base_url should be rejected");

        assert_eq!(
            error.to_string(),
            "room ownership locator is misconfigured: room owner hint base_url must be an origin-only absolute http/https URL without path/query, received `node-b.internal:4000`"
        );
    }

    #[test]
    fn static_room_locator_trims_owner_hint_node_id_and_base_url() {
        let doc_id = Uuid::new_v4();
        let locator = StaticRoomLocator::new(
            " node-a ",
            HashMap::from([(
                doc_id,
                RoomOwnerHint {
                    node_id: "  node-b  ".to_owned(),
                    base_url: Some("  https://node-b.internal:4000/  ".to_owned()),
                },
            )]),
        )
        .expect("static locator should initialize");

        assert_eq!(
            locator
                .resolve(&doc_id)
                .expect("doc should resolve through normalized static locator"),
            ResolvedRoom::Remote(RoomOwnerHint {
                node_id: "node-b".to_owned(),
                base_url: Some("https://node-b.internal:4000".to_owned()),
            })
        );
    }

    #[test]
    fn room_locator_from_config_loads_static_owner_hints_file() {
        let doc_id = Uuid::new_v4();
        let hints_path = temp_hints_path("static-room-locator");
        fs::write(
            &hints_path,
            format!(
                r#"{{
  "documents": {{
    "{doc_id}": {{
      "node_id": " node-b ",
      "base_url": " http://node-b.internal:4000/ "
    }}
  }}
}}"#
            ),
        )
        .expect("static room locator hints file should be written");

        let config = Config {
            host: "127.0.0.1".to_owned(),
            port: 4000,
            frontend_origin: "http://localhost:3000".to_owned(),
            rust_log: "backend=debug".to_owned(),
            api_token: "test-admin-token".to_owned(),
            snapshot_store: "memory".to_owned(),
            snapshot_dir: "./data/test-snapshots".to_owned(),
            room_locator: "static".to_owned(),
            room_coordinator: "noop".to_owned(),
            node_id: "node-a".to_owned(),
            room_owner_hints_path: Some(hints_path.to_string_lossy().into_owned()),
        };

        let locator =
            room_locator_from_config(&config).expect("config should produce a static room locator");
        assert_eq!(
            locator
                .resolve(&doc_id)
                .expect("doc should resolve through config-backed locator"),
            ResolvedRoom::Remote(RoomOwnerHint {
                node_id: "node-b".to_owned(),
                base_url: Some("http://node-b.internal:4000".to_owned()),
            })
        );

        fs::remove_file(hints_path).expect("static room locator hints file should be removed");
    }

    #[test]
    fn room_locator_from_config_rejects_static_mode_without_hints_path() {
        let config = Config {
            host: "127.0.0.1".to_owned(),
            port: 4000,
            frontend_origin: "http://localhost:3000".to_owned(),
            rust_log: "backend=debug".to_owned(),
            api_token: "test-admin-token".to_owned(),
            snapshot_store: "memory".to_owned(),
            snapshot_dir: "./data/test-snapshots".to_owned(),
            room_locator: "static".to_owned(),
            room_coordinator: "noop".to_owned(),
            node_id: "node-a".to_owned(),
            room_owner_hints_path: None,
        };

        let error = match room_locator_from_config(&config) {
            Ok(_) => panic!("static locator without hints path should fail"),
            Err(error) => error,
        };

        match error {
            AppError::Config(message) => assert_eq!(
                message,
                "ROOM_OWNER_HINTS_PATH is required when ROOM_LOCATOR=static"
            ),
            other => panic!("expected config error, received {other:?}"),
        }
    }

    #[test]
    fn room_locator_from_config_rejects_invalid_owner_base_url() {
        let doc_id = Uuid::new_v4();
        let hints_path = temp_hints_path("static-room-locator-invalid-base-url");
        fs::write(
            &hints_path,
            format!(
                r#"{{
  "documents": {{
    "{doc_id}": {{
      "node_id": "node-b",
      "base_url": "ftp://node-b.internal:4000"
    }}
  }}
}}"#
            ),
        )
        .expect("static room locator hints file should be written");

        let config = Config {
            host: "127.0.0.1".to_owned(),
            port: 4000,
            frontend_origin: "http://localhost:3000".to_owned(),
            rust_log: "backend=debug".to_owned(),
            api_token: "test-admin-token".to_owned(),
            snapshot_store: "memory".to_owned(),
            snapshot_dir: "./data/test-snapshots".to_owned(),
            room_locator: "static".to_owned(),
            room_coordinator: "noop".to_owned(),
            node_id: "node-a".to_owned(),
            room_owner_hints_path: Some(hints_path.to_string_lossy().into_owned()),
        };

        let error = match room_locator_from_config(&config) {
            Ok(_) => panic!("invalid owner base_url should fail config loading"),
            Err(error) => error,
        };

        match error {
            AppError::Config(message) => assert_eq!(
                message,
                "room owner hint base_url must be an origin-only absolute http/https URL without path/query, received `ftp://node-b.internal:4000`"
            ),
            other => panic!("expected config error, received {other:?}"),
        }

        fs::remove_file(hints_path).expect("static room locator hints file should be removed");
    }

    #[test]
    fn room_locator_from_config_rejects_owner_base_url_with_path_or_query() {
        let doc_id = Uuid::new_v4();
        let hints_path = temp_hints_path("static-room-locator-invalid-base-url-path");
        fs::write(
            &hints_path,
            format!(
                r#"{{
  "documents": {{
    "{doc_id}": {{
      "node_id": "node-b",
      "base_url": "https://node-b.internal:4000/proxy?via=edge"
    }}
  }}
}}"#
            ),
        )
        .expect("static room locator hints file should be written");

        let config = Config {
            host: "127.0.0.1".to_owned(),
            port: 4000,
            frontend_origin: "http://localhost:3000".to_owned(),
            rust_log: "backend=debug".to_owned(),
            api_token: "test-admin-token".to_owned(),
            snapshot_store: "memory".to_owned(),
            snapshot_dir: "./data/test-snapshots".to_owned(),
            room_locator: "static".to_owned(),
            room_coordinator: "noop".to_owned(),
            node_id: "node-a".to_owned(),
            room_owner_hints_path: Some(hints_path.to_string_lossy().into_owned()),
        };

        let error = match room_locator_from_config(&config) {
            Ok(_) => panic!("owner base_url with path/query should fail config loading"),
            Err(error) => error,
        };

        match error {
            AppError::Config(message) => assert_eq!(
                message,
                "room owner hint base_url must be an origin-only absolute http/https URL without path/query, received `https://node-b.internal:4000/proxy?via=edge`"
            ),
            other => panic!("expected config error, received {other:?}"),
        }

        fs::remove_file(hints_path).expect("static room locator hints file should be removed");
    }

    #[test]
    fn room_locator_from_config_rejects_unknown_locator_mode() {
        let config = Config {
            host: "127.0.0.1".to_owned(),
            port: 4000,
            frontend_origin: "http://localhost:3000".to_owned(),
            rust_log: "backend=debug".to_owned(),
            api_token: "test-admin-token".to_owned(),
            snapshot_store: "memory".to_owned(),
            snapshot_dir: "./data/test-snapshots".to_owned(),
            room_locator: "unsupported".to_owned(),
            room_coordinator: "noop".to_owned(),
            node_id: DEFAULT_NODE_ID.to_owned(),
            room_owner_hints_path: None,
        };

        let error = match room_locator_from_config(&config) {
            Ok(_) => panic!("unknown room locator mode should fail"),
            Err(error) => error,
        };

        match error {
            AppError::Config(message) => assert_eq!(
                message,
                "ROOM_LOCATOR must be `local` or `static`, received `unsupported`"
            ),
            other => panic!("expected config error, received {other:?}"),
        }
    }
}
