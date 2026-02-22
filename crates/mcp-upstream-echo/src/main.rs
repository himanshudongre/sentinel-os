use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct McpToolCall {
    name: String,
    arguments: serde_json::Value,
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/tools/call", post(tool_call));

    let addr = SocketAddr::from(([127, 0, 0, 1], 9000));
    println!("mcp-upstream-echo on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn tool_call(Json(call): Json<McpToolCall>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "ok": true,
        "echo": {
            "name": call.name,
            "arguments": call.arguments
        }
    }))
}
