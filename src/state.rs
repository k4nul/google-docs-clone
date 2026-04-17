use std::sync::Arc;

use crate::{
    collab::rooms::RoomRegistry,
    config::{Config, DEFAULT_FRONTEND_ORIGIN},
};

#[derive(Clone)]
pub struct AppState {
    rooms: Arc<RoomRegistry>,
    frontend_origin: Arc<str>,
}

impl AppState {
    pub fn new(frontend_origin: impl Into<String>) -> Self {
        Self {
            rooms: Arc::new(RoomRegistry::new()),
            frontend_origin: Arc::<str>::from(frontend_origin.into()),
        }
    }

    pub fn from_config(config: &Config) -> Self {
        Self::new(config.frontend_origin.clone())
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
}

impl Default for AppState {
    fn default() -> Self {
        Self::new(DEFAULT_FRONTEND_ORIGIN)
    }
}
