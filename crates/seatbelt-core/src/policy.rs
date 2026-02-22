use sentinel_core::{sha256_hex, TransitionIntent};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub version: String,
    pub default: String,
    pub allow: Vec<AllowRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllowRule {
    pub capability: Option<String>,
    pub mcp_server: Option<String>,
    pub tool_name: Option<String>,
}

#[derive(Debug)]
pub struct Evaluation {
    pub decision: String,
    pub reason: String,
    pub policy_hash: String,
}

impl Policy {
    pub fn load(path: &str) -> Result<Self, String> {
        let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let policy: Policy = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
        Ok(policy)
    }

    pub fn evaluate(&self, intent: &TransitionIntent) -> Evaluation {
        let policy_hash = sha256_hex(
            serde_json::to_string(self)
                .expect("policy serializable")
                .as_bytes(),
        );

        for rule in &self.allow {
            if Self::matches(rule, intent) {
                return Evaluation {
                    decision: "allow".to_string(),
                    reason: "matched allow rule".to_string(),
                    policy_hash,
                };
            }
        }

        Evaluation {
            decision: self.default.clone(),
            reason: "no matching rule".to_string(),
            policy_hash,
        }
    }

    fn matches(rule: &AllowRule, intent: &TransitionIntent) -> bool {
        if let Some(cap) = &rule.capability {
            if &intent.capability != cap {
                return false;
            }
        }

        if let Some(server) = &rule.mcp_server {
            if &intent.target.mcp_server != server {
                return false;
            }
        }

        if let Some(tool) = &rule.tool_name {
            if &intent.target.tool_name != tool {
                return false;
            }
        }

        true
    }
}
