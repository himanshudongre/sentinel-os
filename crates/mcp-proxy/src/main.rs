use reqwest::Client;
use serde_json::Value;
use sha2::{Digest, Sha256};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use uuid::Uuid;

use base64::Engine;

use sentinel_core::{
    Actor, Decision, ExecutionInfo, ParamsDigest, PolicyRef, Target, TransitionIntent,
};

fn sha256_hex_json(v: &Value) -> String {
    let bytes = serde_json::to_vec(v).expect("json serialize");
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

fn jsonrpc_error(id: &Value, code: i64, message: &str) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id.clone(),
        "error": { "code": code, "message": message }
    })
}

// Extract the "decision" string from ProofBundle.decision_canon (base64 of canonical JSON)
fn decision_from_proof_bundle(proof: &Value) -> Result<String, String> {
    let decision_canon_b64 = proof
        .get("decision_canon")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing decision_canon".to_string())?;

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(decision_canon_b64)
        .map_err(|e| format!("base64 decode decision_canon failed: {e}"))?;

    let v: Value = serde_json::from_slice(&bytes)
        .map_err(|e| format!("parse decision_canon json failed: {e}"))?;

    let decision = v
        .get("decision")
        .and_then(|d| d.as_str())
        .ok_or_else(|| "decision missing in decision_canon".to_string())?;

    Ok(decision.to_string())
}

#[tokio::main]
async fn main() {
    eprintln!("[mcp-proxy] starting");

    let sentinel_url =
        std::env::var("SENTINEL_URL").unwrap_or_else(|_| "http://127.0.0.1:8787".to_string());
    let root_dir = std::env::var("FS_ROOT").unwrap_or_else(|_| ".".to_string());

    // Allow overriding in case Claude runs with a different PATH
    let upstream_cmd = std::env::var("UPSTREAM_CMD").unwrap_or_else(|_| "npx".to_string());
    let upstream_pkg = std::env::var("UPSTREAM_PKG")
        .unwrap_or_else(|_| "@modelcontextprotocol/server-filesystem".to_string());

    let http = Client::new();

    // Spawn upstream filesystem MCP server (stdio)
    let mut child = Command::new(upstream_cmd)
        .arg(upstream_pkg)
        .arg(&root_dir)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to start upstream filesystem server");

    let child_stdin = child.stdin.take().expect("upstream stdin");
    let child_stdout = child.stdout.take().expect("upstream stdout");
    let child_stderr = child.stderr.take().expect("upstream stderr");

    let mut upstream_in = tokio::io::BufWriter::new(child_stdin);

    // Forward upstream stderr to our stderr for debugging
    tokio::spawn(async move {
        let mut lines = BufReader::new(child_stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            eprintln!("[upstream:stderr] {line}");
        }
    });

    // Forward upstream stdout -> our stdout
    tokio::spawn(async move {
        let mut lines = BufReader::new(child_stdout).lines();
        let mut out = tokio::io::BufWriter::new(tokio::io::stdout());
        while let Ok(Some(line)) = lines.next_line().await {
            if out.write_all(line.as_bytes()).await.is_err() {
                break;
            }
            if out.write_all(b"\n").await.is_err() {
                break;
            }
            if out.flush().await.is_err() {
                break;
            }
        }
    });

    // Read from Claude stdin
    let mut stdin_lines = BufReader::new(tokio::io::stdin()).lines();
    let mut out = tokio::io::BufWriter::new(tokio::io::stdout());

    loop {
        let line = match stdin_lines.next_line().await {
            Ok(Some(l)) => l,
            Ok(None) => break,
            Err(_) => break,
        };

        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            continue;
        }

        let msg: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => {
                eprintln!("[mcp-proxy] non-json from client, forwarding to upstream");
                if upstream_in.write_all(trimmed.as_bytes()).await.is_err() {
                    break;
                }
                if upstream_in.write_all(b"\n").await.is_err() {
                    break;
                }
                if upstream_in.flush().await.is_err() {
                    break;
                }
                continue;
            }
        };

        let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let id = msg.get("id").cloned().unwrap_or(Value::Null);

        if method == "tools/call" {
            let params = msg.get("params").cloned().unwrap_or(Value::Null);
            let tool_name = params
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown_tool")
                .to_string();
            let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);

            let intent_id = Uuid::new_v4();
            let ts = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

            let intent = TransitionIntent {
                id: intent_id,
                ts,
                schema_version: "0.1".to_string(),
                transition_type: "action".to_string(),
                capability: tool_name.clone(),
                params_digest: ParamsDigest {
                    alg: "sha256".to_string(),
                    value: sha256_hex_json(&arguments),
                },
                proposed_effect: None,
                actor: Actor {
                    agent_name: "claude-desktop".to_string(),
                    agent_version: "unknown".to_string(),
                    host_fingerprint: "local".to_string(),
                    runtime: "desktop".to_string(),
                },
                target: Target {
                    mcp_server: "filesystem_proxy".to_string(),
                    tool_name: tool_name.clone(),
                },
            };

            // Placeholder decision is ignored by sentineld v0.2+
            let placeholder_decision = Decision {
                intent_id,
                decision: "allow".to_string(),
                reason: "placeholder".to_string(),
                policy: PolicyRef {
                    policy_id: "placeholder".to_string(),
                    policy_hash: "0".repeat(64),
                    policy_version: "0.0".to_string(),
                },
                constraints: None,
            };

            let req = serde_json::json!({
                "intent": intent,
                "decision": placeholder_decision,
                "execution": ExecutionInfo {
                    status: "pending".to_string(),
                    digest_alg: "sha256".to_string(),
                    digest: "0".to_string(),
                }
            });

            // Ask sentineld for authoritative proof
            let resp = match http
                .post(format!("{sentinel_url}/v1/transitions"))
                .json(&req)
                .send()
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    let err = jsonrpc_error(&id, -32001, &format!("Sentinel unreachable: {e}"));
                    let _ = out.write_all(err.to_string().as_bytes()).await;
                    let _ = out.write_all(b"\n").await;
                    let _ = out.flush().await;
                    continue;
                }
            };

            let status = resp.status().as_u16();
            let proof_json: Value = match resp.json().await {
                Ok(v) => v,
                Err(e) => {
                    let err = jsonrpc_error(&id, -32001, &format!("Sentinel bad response: {e}"));
                    let _ = out.write_all(err.to_string().as_bytes()).await;
                    let _ = out.write_all(b"\n").await;
                    let _ = out.flush().await;
                    continue;
                }
            };

            if status != 201 {
                let err = jsonrpc_error(
                    &id,
                    -32000,
                    &format!("Sentinel rejected transition. http_status={status}"),
                );
                let _ = out.write_all(err.to_string().as_bytes()).await;
                let _ = out.write_all(b"\n").await;
                let _ = out.flush().await;
                continue;
            }

            // IMPORTANT: 201 does not mean allow. Check decision inside the proof.
            let decision = match decision_from_proof_bundle(&proof_json) {
                Ok(d) => d,
                Err(e) => {
                    let err = jsonrpc_error(&id, -32001, &format!("Failed to parse proof: {e}"));
                    let _ = out.write_all(err.to_string().as_bytes()).await;
                    let _ = out.write_all(b"\n").await;
                    let _ = out.flush().await;
                    continue;
                }
            };

            let proof_id = proof_json
                .get("proof_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            let log_hash = proof_json
                .get("log_hash")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            if decision != "allow" {
                eprintln!(
                    "[mcp-proxy] DENY tools/call name={} proof_id={} log_hash={} decision={}",
                    tool_name, proof_id, log_hash, decision
                );

                let err = jsonrpc_error(
                    &id,
                    -32000,
                    &format!(
                        "Blocked by Seatbelt policy. decision={} proof_id={} log_hash={}",
                        decision, proof_id, log_hash
                    ),
                );

                let _ = out.write_all(err.to_string().as_bytes()).await;
                let _ = out.write_all(b"\n").await;
                let _ = out.flush().await;
                continue;
            } else {
                eprintln!(
                    "[mcp-proxy] ALLOW tools/call name={} proof_id={} log_hash={}",
                    tool_name, proof_id, log_hash
                );
            }
        }

        // Allowed or not a tool call: forward to upstream
        if method == "tools/call" {
            // This log should appear only when the call is actually forwarded
            let tool = msg
                .get("params")
                .and_then(|p| p.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown_tool");
            eprintln!("[mcp-proxy] FORWARD tools/call name={tool}");
        } else {
            eprintln!("[mcp-proxy] FORWARD method={method}");
        }

        if upstream_in.write_all(trimmed.as_bytes()).await.is_err() {
            break;
        }
        if upstream_in.write_all(b"\n").await.is_err() {
            break;
        }
        if upstream_in.flush().await.is_err() {
            break;
        }
    }

    let _ = child.kill().await;
    eprintln!("[mcp-proxy] exiting");
}
