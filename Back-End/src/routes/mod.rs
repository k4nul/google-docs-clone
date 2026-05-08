use axum::{Router, routing::get};

use crate::state::AppState;

pub mod documents;
pub mod health;

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health::get_health))
        .route(
            "/documents",
            get(documents::list_documents).post(documents::create_document),
        )
        .route(
            "/documents/{id}",
            get(documents::get_document).delete(documents::delete_document),
        )
}
