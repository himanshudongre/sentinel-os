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

#[test]
fn proof_verifies_ok() {
    let sk = generate_signing_key();
    let vk: VerifyingKey = sk.verifying_key();

    let intent = dummy_intent();
    let decision = dummy_decision(intent.id);

    let execution = ExecutionInfo {
        status: "ok".to_string(),
        digest_alg: "sha256".to_string(),
        digest: sha256_hex(b"RESULT_OK"),
    };

    let proof = build_proof_bundle(ProofBuildInput {
        proof_id: Uuid::new_v4(),
        ts: "2026-02-22T00:00:01Z".to_string(),
        intent: &intent,
        decision: &decision,
        execution,
        prev_log_hash: "00".repeat(32),
        signing_key: &sk,
        verifying_key: &vk,
    })
    .expect("build_proof_bundle");

    verify_proof_bundle(&proof, &vk).expect("verify_proof_bundle");
}

#[test]
fn tampering_breaks_verification() {
    let sk = generate_signing_key();
    let vk: VerifyingKey = sk.verifying_key();

    let intent = dummy_intent();
    let decision = dummy_decision(intent.id);

    let execution = ExecutionInfo {
        status: "ok".to_string(),
        digest_alg: "sha256".to_string(),
        digest: sha256_hex(b"RESULT_OK"),
    };

    let mut proof = build_proof_bundle(ProofBuildInput {
        proof_id: Uuid::new_v4(),
        ts: "2026-02-22T00:00:01Z".to_string(),
        intent: &intent,
        decision: &decision,
        execution,
        prev_log_hash: "00".repeat(32),
        signing_key: &sk,
        verifying_key: &vk,
    })
    .expect("build_proof_bundle");

    // Tamper with execution digest
    proof.execution.digest = sha256_hex(b"EVIL");

    assert!(verify_proof_bundle(&proof, &vk).is_err());
}
