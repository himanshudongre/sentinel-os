use ed25519_dalek::VerifyingKey;
use reqwest::blocking::Client;
use sentinel_core::*;
use uuid::Uuid;

fn main() {
    let base = "http://127.0.0.1:8787";
    let client = Client::new();

    let sk = generate_signing_key();
    let vk: VerifyingKey = sk.verifying_key();

    let intent = TransitionIntent {
        id: Uuid::new_v4(),
        ts: "2026-02-22T00:00:00Z".to_string(),
        schema_version: "0.1".to_string(),
        actor: Actor {
            agent_name: "ctl".to_string(),
            agent_version: "0.1".to_string(),
            runtime: "local".to_string(),
            host_fingerprint: "dev".to_string(),
        },
        transition_type: "action".to_string(),
        capability: "demo".to_string(),
        target: Target {
            mcp_server: "none".to_string(),
            tool_name: "none".to_string(),
        },
        params_digest: ParamsDigest {
            alg: "sha256".to_string(),
            value: "00".repeat(32),
        },
        proposed_effect: None,
    };

    let decision = Decision {
        intent_id: intent.id,
        decision: "allow".to_string(),
        reason: "demo".to_string(),
        policy: PolicyRef {
            policy_id: "default".to_string(),
            policy_hash: "11".repeat(32),
            policy_version: "0.1".to_string(),
        },
        constraints: None,
    };

    let execution = ExecutionInfo {
        status: "ok".to_string(),
        digest_alg: "sha256".to_string(),
        digest: sha256_hex(b"DEMO_RESULT"),
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
    .expect("build proof");

    client
        .post(format!("{base}/v1/proofs"))
        .json(&proof)
        .send()
        .expect("post proof");

    let chain: Vec<ProofBundle> = client
        .get(format!("{base}/v1/chain"))
        .send()
        .expect("get chain")
        .json()
        .expect("parse chain");

    verify_proof_chain(&chain, &vk).expect("verify chain");

    println!("Chain verified successfully. Length={}", chain.len());
}
