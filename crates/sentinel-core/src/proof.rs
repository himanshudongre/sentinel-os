use crate::{canonical_json_bytes, sha256_hex, Decision, TransitionIntent};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use ed25519_dalek::{Signature, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionInfo {
    pub status: String,
    pub digest_alg: String,
    pub digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningInfo {
    pub alg: String,
    pub pubkey_id: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofBundle {
    pub proof_id: Uuid,
    pub ts: String,
    pub schema_version: String,
    pub canon_version: String,
    pub intent_canon: String,
    pub decision_canon: String,
    pub execution: ExecutionInfo,
    pub prev_log_hash: String,
    pub log_hash: String,
    pub signing: SigningInfo,
}

pub fn make_not_executed_digest() -> String {
    sha256_hex(b"NOT_EXECUTED")
}

pub fn build_proof_bundle(
    proof_id: Uuid,
    ts: String,
    intent: &TransitionIntent,
    decision: &Decision,
    execution: ExecutionInfo,
    prev_log_hash: String,
    sk: &SigningKey,
    vk: &VerifyingKey,
    pubkey_id: String,
) -> Result<ProofBundle, String> {
    let intent_bytes = canonical_json_bytes(intent).map_err(|e| e.to_string())?;
    let decision_bytes = canonical_json_bytes(decision).map_err(|e| e.to_string())?;

    let intent_canon_b64 = B64.encode(&intent_bytes);
    let decision_canon_b64 = B64.encode(&decision_bytes);

    let mut entry_payload = Vec::new();
    entry_payload.extend_from_slice(prev_log_hash.as_bytes());
    entry_payload.extend_from_slice(&intent_bytes);
    entry_payload.extend_from_slice(&decision_bytes);
    entry_payload.extend_from_slice(execution.digest.as_bytes());

    let log_hash = sha256_hex(&entry_payload);

    let sig = crate::sign_bytes(sk, log_hash.as_bytes());
    let sig_b64 = B64.encode(sig.to_bytes());

    crate::verify_bytes(
        vk,
        log_hash.as_bytes(),
        &Signature::from_bytes(&sig.to_bytes()),
    )
    .map_err(|e| e.to_string())?;

    Ok(ProofBundle {
        proof_id,
        ts,
        schema_version: "0.1".to_string(),
        canon_version: "rfc8785-jcs-v1".to_string(),
        intent_canon: intent_canon_b64,
        decision_canon: decision_canon_b64,
        execution,
        prev_log_hash,
        log_hash,
        signing: SigningInfo {
            alg: "ed25519".to_string(),
            pubkey_id,
            signature: sig_b64,
        },
    })
}
