use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use ed25519_dalek::VerifyingKey;
use reqwest::blocking::Client;
use sentinel_core::*;
use uuid::Uuid;

#[derive(Debug, serde::Deserialize)]
struct CurrentKeyResponse {
    alg: String,
    pubkey_id: String,
    verifying_key_b64: String,
}

#[derive(Debug, serde::Serialize)]
struct TransitionRequest {
    intent: TransitionIntent,
    decision: Decision,
    execution: ExecutionInfo,
}

fn main() {
    let base = "http://127.0.0.1:8787";
    let client = Client::new();

    // 1) Build a transition request (no signing on client)
    let intent_id = Uuid::new_v4();

    let intent = TransitionIntent {
        id: intent_id,
        ts: "2026-02-22T00:00:00Z".to_string(),
        schema_version: "0.1".to_string(),
        actor: Actor {
            agent_name: "sentinelctl".to_string(),
            agent_version: "0.1".to_string(),
            runtime: "local".to_string(),
            host_fingerprint: "dev".to_string(),
        },
        transition_type: "action".to_string(),
        capability: "dangerous".to_string(),
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
        intent_id,
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

    let req = TransitionRequest {
        intent,
        decision,
        execution,
    };

    // 2) POST transition, receive server-signed proof bundle
    let created: ProofBundle = client
        .post(format!("{base}/v1/transitions"))
        .json(&req)
        .send()
        .expect("post transition")
        .error_for_status()
        .expect("transition status")
        .json()
        .expect("parse created proof");

    println!(
        "Created proof_id={} log_hash={}",
        created.proof_id, created.log_hash
    );

    // 3) Fetch server verifying key
    let key: CurrentKeyResponse = client
        .get(format!("{base}/v1/keys/current"))
        .send()
        .expect("get key")
        .error_for_status()
        .expect("key status")
        .json()
        .expect("parse key response");

    if key.alg != "ed25519" {
        panic!("unsupported key alg: {}", key.alg);
    }

    let vk_bytes = B64
        .decode(key.verifying_key_b64.trim())
        .expect("base64 decode verifying key");
    let vk_arr: [u8; 32] = vk_bytes.try_into().expect("verifying key must be 32 bytes");
    let vk = VerifyingKey::from_bytes(&vk_arr).expect("construct verifying key");

    // 4) Fetch entire chain and verify offline
    let chain: Vec<ProofBundle> = client
        .get(format!("{base}/v1/chain"))
        .send()
        .expect("get chain")
        .error_for_status()
        .expect("chain status")
        .json()
        .expect("parse chain");

    verify_proof_chain(&chain, &vk).expect("verify_proof_chain");

    let head = chain.last().map(|p| p.log_hash.clone()).unwrap_or_default();

    println!(
        "Chain verified successfully. len={} head={} pubkey_id={}",
        chain.len(),
        head,
        key.pubkey_id
    );
}
