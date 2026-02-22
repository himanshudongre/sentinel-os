mod api;
mod db;

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::api::AppState;
use crate::db::Db;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_path = std::env::var("SENTINEL_DB").unwrap_or_else(|_| "sentinel.db".to_string());
    let db = Db::new(db_path);
    db.init().expect("db init");

    let state = AppState { db };

    let app = Router::new()
        .route("/health", get(api::health))
        .route("/v1/proofs", post(api::post_proof))
        .route("/v1/proofs/:proof_id", get(api::get_proof))
        .route("/v1/chain/head", get(api::get_head))
        .route("/v1/chain", get(api::get_chain))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8787));
    tracing::info!("sentineld listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
