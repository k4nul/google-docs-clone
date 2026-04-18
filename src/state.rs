use std::sync::Arc;

use crate::{
    collab::rooms::RoomRegistry,
    config::{Config, DEFAULT_FRONTEND_ORIGIN},
    errors::{AppError, AppResult},
    storage::{SnapshotStore, in_memory_snapshot_store},
};

#[derive(Clone)]
pub struct AppState {
    rooms: Arc<RoomRegistry>,
    frontend_origin: Arc<str>,
    api_token: Arc<str>,
}

impl AppState {
    pub fn new(frontend_origin: impl Into<String>, api_token: impl Into<String>) -> Self {
        Self {
            rooms: Arc::new(RoomRegistry::default()),
            frontend_origin: Arc::<str>::from(frontend_origin.into()),
            api_token: Arc::<str>::from(api_token.into()),
        }
    }

    pub fn with_snapshot_store(
        frontend_origin: impl Into<String>,
        api_token: impl Into<String>,
        snapshot_store: Arc<dyn SnapshotStore>,
    ) -> AppResult<Self> {
        let rooms = Arc::new(RoomRegistry::new(snapshot_store));
        let hydrated_rooms = rooms
            .hydrate_from_store()
            .map_err(anyhow::Error::from)
            .map_err(AppError::from)?;

        tracing::info!(
            hydrated_rooms,
            "initialized room registry from snapshot store"
        );

        Ok(Self {
            rooms,
            frontend_origin: Arc::<str>::from(frontend_origin.into()),
            api_token: Arc::<str>::from(api_token.into()),
        })
    }

    pub fn from_config(config: &Config) -> AppResult<Self> {
        Self::with_snapshot_store(
            config.frontend_origin.clone(),
            config.api_token.clone(),
            in_memory_snapshot_store(),
        )
    }

    pub fn rooms(&self) -> &RoomRegistry {
        self.rooms.as_ref()
    }

    pub fn rooms_registry(&self) -> Arc<RoomRegistry> {
        Arc::clone(&self.rooms)
    }

    pub fn frontend_origin(&self) -> &str {
        &self.frontend_origin
    }

    pub fn api_token(&self) -> &str {
        &self.api_token
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::with_snapshot_store(
            DEFAULT_FRONTEND_ORIGIN,
            crate::config::DEFAULT_API_TOKEN,
            in_memory_snapshot_store(),
        )
        .expect("default app state should initialize room registry")
    }
}
