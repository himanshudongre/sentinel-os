mod api;
mod db;
mod keys;

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
    let key_path = std::env::var("SENTINEL_KEY").unwrap_or_else(|_| "sentinel.key".to_string());

    let db = Db::new(&db_path);
    db.init().expect("db init");

    let signing_key = keys::load_or_create_signing_key(&key_path).expect("load/create signing key");
    let verifying_key = signing_key.verifying_key();
    let server_pubkey_id = sentinel_core::sha256_hex(verifying_key.to_bytes().as_slice());

    // DB identity lock: one DB must correspond to one server pubkey_id
    let existing = db.get_meta("server_pubkey_id").expect("read meta");

    if let Some(db_pubkey_id) = existing {
        if db_pubkey_id != server_pubkey_id {
            eprintln!("FATAL: sentinel.db belongs to a different server key.");
            eprintln!("FATAL: DB belongs to a different server key.");
            eprintln!("DB path: {}", db_path);
            eprintln!("Key path: {}", key_path);
            eprintln!("DB server_pubkey_id:   {}", db_pubkey_id);
            eprintln!("Current server_pubkey_id: {}", server_pubkey_id);
            eprintln!("Fix: start sentineld with the original sentinel.key, or delete sentinel.db and sentinel.key to reset.");
            std::process::exit(1);
        }
    } else {
        // meta missing: if proofs exist, infer identity from head and lock it
        if let Some(head) = db.get_head().expect("read head") {
            let inferred = head.signing.pubkey_id.clone();
            if inferred != server_pubkey_id {
                eprintln!("FATAL: sentinel.db contains proofs signed by a different key.");
                eprintln!("DB inferred pubkey_id:    {}", inferred);
                eprintln!("Current server_pubkey_id: {}", server_pubkey_id);
                eprintln!("Fix: use matching sentinel.key or reset db.");
                std::process::exit(1);
            }
            db.set_meta("server_pubkey_id", &inferred)
                .expect("write meta");
        } else {
            // empty db: lock identity to current key
            db.set_meta("server_pubkey_id", &server_pubkey_id)
                .expect("write meta");
        }
    }

    let state = AppState {
        db,
        signing_key,
        verifying_key,
    };

    let app = Router::new()
        .route("/health", get(api::health))
        .route("/v1/chain/head", get(api::get_head))
        .route("/v1/chain", get(api::get_chain))
        .route("/v1/transitions", post(api::post_transition))
        .route("/v1/keys/current", get(api::get_current_key))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8787));
    tracing::info!("sentineld listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
