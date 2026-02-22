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

#[derive(Debug)]
pub struct ProofBuildInput<'a> {
    pub proof_id: Uuid,
    pub ts: String,
    pub intent: &'a TransitionIntent,
    pub decision: &'a Decision,
    pub execution: ExecutionInfo,
    pub prev_log_hash: String,
    pub signing_key: &'a SigningKey,
    pub verifying_key: &'a VerifyingKey,
    pub pubkey_id: String,
}

/// Build a ProofBundle that is verifiable offline.
///
/// Hash/signing rules (v0.1):
/// - intent_canon = base64(JCS(intent))
/// - decision_canon = base64(JCS(decision))
/// - entry_payload = prev_log_hash || intent_bytes || decision_bytes || execution.digest
/// - log_hash = sha256_hex(entry_payload)
/// - signature = ed25519_sign(log_hash_bytes)
pub fn build_proof_bundle(input: ProofBuildInput) -> Result<ProofBundle, String> {
    let intent_bytes = canonical_json_bytes(input.intent).map_err(|e| e.to_string())?;
    let decision_bytes = canonical_json_bytes(input.decision).map_err(|e| e.to_string())?;

    let intent_canon_b64 = B64.encode(&intent_bytes);
    let decision_canon_b64 = B64.encode(&decision_bytes);

    let mut entry_payload = Vec::new();
    entry_payload.extend_from_slice(input.prev_log_hash.as_bytes());
    entry_payload.extend_from_slice(&intent_bytes);
    entry_payload.extend_from_slice(&decision_bytes);
    entry_payload.extend_from_slice(input.execution.digest.as_bytes());

    let log_hash = sha256_hex(&entry_payload);

    let sig = crate::sign_bytes(input.signing_key, log_hash.as_bytes());
    let sig_b64 = B64.encode(sig.to_bytes());

    // Verify immediately as a sanity check (should never fail)
    let sig_arr: [u8; 64] = sig.to_bytes();
    let sig2 = Signature::from_bytes(&sig_arr);
    crate::verify_bytes(input.verifying_key, log_hash.as_bytes(), &sig2)
        .map_err(|e| e.to_string())?;

    Ok(ProofBundle {
        proof_id: input.proof_id,
        ts: input.ts,
        schema_version: "0.1".to_string(),
        canon_version: "rfc8785-jcs-v1".to_string(),
        intent_canon: intent_canon_b64,
        decision_canon: decision_canon_b64,
        execution: input.execution,
        prev_log_hash: input.prev_log_hash,
        log_hash,
        signing: SigningInfo {
            alg: "ed25519".to_string(),
            pubkey_id: input.pubkey_id,
            signature: sig_b64,
        },
    })
}
