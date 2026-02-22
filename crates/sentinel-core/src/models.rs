use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actor {
    pub agent_name: String,
    pub agent_version: String,
    pub runtime: String,
    pub host_fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamsDigest {
    pub alg: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    pub mcp_server: String,
    pub tool_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionIntent {
    pub id: Uuid,
    pub ts: String,
    pub schema_version: String,
    pub actor: Actor,
    pub transition_type: String,
    pub capability: String,
    pub target: Target,
    pub params_digest: ParamsDigest,
    pub proposed_effect: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRef {
    pub policy_id: String,
    pub policy_hash: String,
    pub policy_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub intent_id: Uuid,
    pub decision: String,
    pub reason: String,
    pub policy: PolicyRef,
    pub constraints: Option<serde_json::Value>,
}
