use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sentinel_core::ProofBundle;

use crate::db::Db;

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
}

pub async fn health() -> &'static str {
    "ok"
}

pub async fn post_proof(
    State(st): State<AppState>,
    Json(proof): Json<ProofBundle>,
) -> impl IntoResponse {
    match st.db.insert_proof(&proof) {
        Ok(_) => (StatusCode::CREATED, Json(proof.proof_id.to_string())).into_response(),
        Err(e) => {
            let msg = format!("db insert failed: {e}");
            (StatusCode::CONFLICT, msg).into_response()
        }
    }
}

pub async fn get_proof(
    State(st): State<AppState>,
    Path(proof_id): Path<String>,
) -> impl IntoResponse {
    match st.db.get_proof_by_id(&proof_id) {
        Ok(Some(p)) => (StatusCode::OK, Json(p)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "not found".to_string()).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}")).into_response(),
    }
}

pub async fn get_head(State(st): State<AppState>) -> impl IntoResponse {
    match st.db.get_head() {
        Ok(Some(p)) => (StatusCode::OK, Json(p)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "empty".to_string()).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}")).into_response(),
    }
}

pub async fn get_chain(State(st): State<AppState>) -> impl IntoResponse {
    // v0.1: fixed limit, later add query params (since_seq, limit)
    match st.db.list_chain(10_000) {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}")).into_response(),
    }
}
