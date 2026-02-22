use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use seatbelt_core::policy::Policy;
use sentinel_core::{Decision, PolicyRef};

use crate::db::Db;

#[derive(Debug, Clone, serde::Serialize)]
pub struct CurrentKeyResponse {
    pub alg: String,
    pub pubkey_id: String,
    pub verifying_key_b64: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TransitionRequest {
    pub intent: sentinel_core::TransitionIntent,
    pub execution: sentinel_core::ExecutionInfo,
}

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub signing_key: ed25519_dalek::SigningKey,
    pub verifying_key: ed25519_dalek::VerifyingKey,
    pub policy: Policy,
}

pub async fn health() -> &'static str {
    "ok"
}

pub async fn post_transition(
    State(st): State<AppState>,
    Json(req): Json<TransitionRequest>,
) -> impl IntoResponse {
    // Compute the authoritative prev_log_hash
    let prev = match st.db.expected_prev_log_hash() {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("db head read failed: {e}"),
            )
                .into_response();
        }
    };

    let eval = st.policy.evaluate(&req.intent);

    let server_decision = Decision {
        intent_id: req.intent.id,
        decision: eval.decision.clone(),
        reason: eval.reason.clone(),
        policy: PolicyRef {
            policy_id: "local".to_string(),
            policy_hash: eval.policy_hash.clone(),
            policy_version: st.policy.version.clone(),
        },
        constraints: None,
    };

    // Build proof on server, sign with server key
    let proof = match sentinel_core::build_proof_bundle(sentinel_core::ProofBuildInput {
        proof_id: uuid::Uuid::new_v4(),
        ts: req.intent.ts.clone(),
        intent: &req.intent,
        decision: &server_decision,
        execution: req.execution,
        prev_log_hash: prev,
        signing_key: &st.signing_key,
        verifying_key: &st.verifying_key,
    }) {
        Ok(p) => p,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    };

    // Store proof
    if let Err(e) = st.db.insert_proof(&proof) {
        return (StatusCode::CONFLICT, format!("db insert failed: {e}")).into_response();
    }

    (StatusCode::CREATED, Json(proof)).into_response()
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

pub async fn get_current_key(State(st): State<AppState>) -> impl IntoResponse {
    let vk_bytes = st.verifying_key.to_bytes();
    let verifying_key_b64 = B64.encode(vk_bytes);

    let resp = CurrentKeyResponse {
        alg: "ed25519".to_string(),
        pubkey_id: sentinel_core::sha256_hex(vk_bytes.as_slice()),
        verifying_key_b64,
    };

    (StatusCode::OK, Json(resp)).into_response()
}
