use crate::{sha256_hex, ProofBundle};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde_json::Value;

#[derive(Debug, thiserror::Error)]
pub enum VerifyError {
    #[error("unsupported canon_version: {0}")]
    UnsupportedCanonVersion(String),

    #[error("base64 decode failed: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("json decode failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("signature decode failed")]
    BadSignatureEncoding,

    #[error("signature verify failed: {0}")]
    SigVerify(#[from] ed25519_dalek::SignatureError),

    #[error("intent canonical bytes mismatch")]
    IntentCanonMismatch,

    #[error("decision canonical bytes mismatch")]
    DecisionCanonMismatch,

    #[error("log hash mismatch")]
    LogHashMismatch,
}

pub fn verify_proof_bundle(proof: &ProofBundle, vk: &VerifyingKey) -> Result<(), VerifyError> {
    if proof.canon_version != "rfc8785-jcs-v1" {
        return Err(VerifyError::UnsupportedCanonVersion(
            proof.canon_version.clone(),
        ));
    }

    let intent_bytes = B64.decode(&proof.intent_canon)?;
    let decision_bytes = B64.decode(&proof.decision_canon)?;

    let intent_val: Value = serde_json::from_slice(&intent_bytes)?;
    let decision_val: Value = serde_json::from_slice(&decision_bytes)?;

    let intent_canon_re = serde_jcs::to_vec(&intent_val)?;
    let decision_canon_re = serde_jcs::to_vec(&decision_val)?;

    if intent_canon_re != intent_bytes {
        return Err(VerifyError::IntentCanonMismatch);
    }
    if decision_canon_re != decision_bytes {
        return Err(VerifyError::DecisionCanonMismatch);
    }

    let mut entry_payload = Vec::new();
    entry_payload.extend_from_slice(proof.prev_log_hash.as_bytes());
    entry_payload.extend_from_slice(&intent_bytes);
    entry_payload.extend_from_slice(&decision_bytes);
    entry_payload.extend_from_slice(proof.execution.digest.as_bytes());

    let expected_log_hash = sha256_hex(&entry_payload);
    if expected_log_hash != proof.log_hash {
        return Err(VerifyError::LogHashMismatch);
    }

    let sig_bytes = B64.decode(&proof.signing.signature)?;
    let sig_arr: [u8; 64] = sig_bytes
        .try_into()
        .map_err(|_| VerifyError::BadSignatureEncoding)?;
    let sig = Signature::from_bytes(&sig_arr);

    vk.verify(proof.log_hash.as_bytes(), &sig)?;

    Ok(())
}
