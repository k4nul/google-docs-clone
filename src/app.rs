use axum::{
    Router,
    http::{HeaderValue, Method},
    routing::get,
};
use tower_http::{
    cors::{AllowOrigin, Any, CorsLayer},
    trace::TraceLayer,
};

use crate::{collab::ws, config::Config, errors::AppResult, routes, state::AppState};

pub fn build_app(config: &Config, state: AppState) -> AppResult<Router> {
    let cors = build_cors(config)?;

    Ok(Router::new()
        .nest("/api", routes::api_router())
        .route("/ws/{doc_id}", get(ws::ws_handler))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state))
}

fn build_cors(config: &Config) -> AppResult<CorsLayer> {
    let origin: HeaderValue = config.frontend_origin_header()?;

    Ok(CorsLayer::new()
        .allow_origin(AllowOrigin::exact(origin))
        .allow_methods([Method::GET])
        .allow_headers(Any))
}
