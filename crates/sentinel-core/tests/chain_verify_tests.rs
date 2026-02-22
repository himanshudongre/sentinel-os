use ed25519_dalek::VerifyingKey;
use sentinel_core::*;
use uuid::Uuid;

fn dummy_intent() -> TransitionIntent {
    TransitionIntent {
        id: Uuid::new_v4(),
        ts: "2026-02-22T00:00:00Z".to_string(),
        schema_version: "0.1".to_string(),
        actor: Actor {
            agent_name: "test-agent".to_string(),
            agent_version: "0.0.1".to_string(),
            runtime: "test".to_string(),
            host_fingerprint: "host123".to_string(),
        },
        transition_type: "action".to_string(),
        capability: "mcp.tool.invoke".to_string(),
        target: Target {
            mcp_server: "mock".to_string(),
            tool_name: "do_thing".to_string(),
        },
        params_digest: ParamsDigest {
            alg: "sha256".to_string(),
            value: "00".repeat(32),
        },
        proposed_effect: Some("test".to_string()),
    }
}

fn dummy_decision(intent_id: uuid::Uuid) -> Decision {
    Decision {
        intent_id,
        decision: "allow".to_string(),
        reason: "ok".to_string(),
        policy: PolicyRef {
            policy_id: "default".to_string(),
            policy_hash: "11".repeat(32),
            policy_version: "0.1".to_string(),
        },
        constraints: None,
    }
}

fn make_proof(
    sk: &ed25519_dalek::SigningKey,
    vk: &VerifyingKey,
    prev_log_hash: String,
) -> ProofBundle {
    let intent = dummy_intent();
    let decision = dummy_decision(intent.id);

    let execution = ExecutionInfo {
        status: "ok".to_string(),
        digest_alg: "sha256".to_string(),
        digest: sha256_hex(b"RESULT_OK"),
    };

    build_proof_bundle(ProofBuildInput {
        proof_id: Uuid::new_v4(),
        ts: "2026-02-22T00:00:01Z".to_string(),
        intent: &intent,
        decision: &decision,
        execution,
        prev_log_hash,
        signing_key: sk,
        verifying_key: vk,
    })
    .expect("build_proof_bundle")
}

#[test]
fn chain_verifies_ok() {
    let sk = generate_signing_key();
    let vk: VerifyingKey = sk.verifying_key();

    let p1 = make_proof(&sk, &vk, "00".repeat(32));
    let p2 = make_proof(&sk, &vk, p1.log_hash.clone());
    let p3 = make_proof(&sk, &vk, p2.log_hash.clone());

    let proofs = vec![p1, p2, p3];
    verify_proof_chain(&proofs, &vk).expect("verify_proof_chain");
}

#[test]
fn chain_link_mismatch_fails() {
    let sk = generate_signing_key();
    let vk: VerifyingKey = sk.verifying_key();

    let p1 = make_proof(&sk, &vk, "00".repeat(32));
    let mut p2 = make_proof(&sk, &vk, p1.log_hash.clone());

    // Break the link
    p2.prev_log_hash = "ff".repeat(32);

    let proofs = vec![p1, p2];
    assert!(verify_proof_chain(&proofs, &vk).is_err());
}
