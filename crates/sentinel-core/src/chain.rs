use crate::{verify_proof_bundle, ProofBundle, VerifyError};
use ed25519_dalek::VerifyingKey;

#[derive(Debug, thiserror::Error)]
pub enum ChainError {
    #[error("proof verification failed at index {index}: {source}")]
    ProofVerify { index: usize, source: VerifyError },

    #[error("chain link mismatch at index {index}: expected prev_log_hash={expected}, got={got}")]
    LinkMismatch {
        index: usize,
        expected: String,
        got: String,
    },

    #[error("empty chain")]
    EmptyChain,
}

/// Verify a chain of proofs in order.
///
/// Rules:
/// - each proof must verify against vk
/// - for i>0: chain[i].prev_log_hash must equal chain[i-1].log_hash
pub fn verify_proof_chain(proofs: &[ProofBundle], vk: &VerifyingKey) -> Result<(), ChainError> {
    if proofs.is_empty() {
        return Err(ChainError::EmptyChain);
    }

    for (i, p) in proofs.iter().enumerate() {
        verify_proof_bundle(p, vk).map_err(|e| ChainError::ProofVerify {
            index: i,
            source: e,
        })?;

        if i > 0 {
            let expected = &proofs[i - 1].log_hash;
            let got = &p.prev_log_hash;
            if got != expected {
                return Err(ChainError::LinkMismatch {
                    index: i,
                    expected: expected.clone(),
                    got: got.clone(),
                });
            }
        }
    }

    Ok(())
}
