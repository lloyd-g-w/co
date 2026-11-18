//! The co server: serves `/ws` (the protocol of `co-protocol`, one JSON
//! message per text frame) and `/healthz`.

use std::net::SocketAddr;

use axum::{Router, extract::State, routing::get};
use co_hosts::Registry;
use co_store::Store;
use tower_http::trace::TraceLayer;

mod ws;

pub struct Config {
    pub listen: SocketAddr,
    pub database_url: String,
}

#[derive(Clone)]
pub struct AppState {
    pub store: Store,
    pub hosts: Registry,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/ws", get(ws::upgrade))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn healthz(State(state): State<AppState>) -> &'static str {
    match sqlx_ping(&state.store).await {
        true => "ok",
        false => "db unavailable",
    }
}

async fn sqlx_ping(store: &Store) -> bool {
    store.pool().acquire().await.is_ok()
}

/// Connects to the database, applies migrations and serves until shutdown.
pub async fn serve(config: Config) -> anyhow::Result<()> {
    let store = Store::connect(&config.database_url).await?;
    store.migrate().await?;
    let state = AppState {
        store,
        hosts: Registry::new(),
    };
    let listener = tokio::net::TcpListener::bind(config.listen).await?;
    tracing::info!("co server listening on {}", listener.local_addr()?);
    axum::serve(listener, router(state))
        .with_graceful_shutdown(shutdown())
        .await?;
    Ok(())
}

async fn shutdown() {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("shutting down");
}
