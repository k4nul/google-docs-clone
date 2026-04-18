use anyhow::Context;
use backend::{app::build_app, config::Config, state::AppState};
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env()?;
    init_tracing(&config)?;

    let state = AppState::from_config(&config)?;
    let app = build_app(&config, state)?;

    let bind_address = config.bind_address();
    let listener = tokio::net::TcpListener::bind(&bind_address)
        .await
        .with_context(|| format!("failed to bind TCP listener on {bind_address}"))?;
    let local_addr = listener
        .local_addr()
        .context("failed to resolve listener local address")?;

    info!(%local_addr, "collaborative backend listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("axum server exited with an error")?;

    Ok(())
}

fn init_tracing(config: &Config) -> anyhow::Result<()> {
    let env_filter = EnvFilter::try_new(config.rust_log.clone())
        .or_else(|_| EnvFilter::try_from_default_env())
        .context("invalid RUST_LOG value")?;

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .try_init()
        .map_err(|error| anyhow::anyhow!("failed to initialize tracing subscriber: {error}"))?;

    Ok(())
}

async fn shutdown_signal() {
    match tokio::signal::ctrl_c().await {
        Ok(()) => info!("shutdown signal received"),
        Err(error) => tracing::warn!(%error, "failed to install Ctrl+C handler"),
    }
}
