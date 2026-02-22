#!/usr/bin/env bash
set -euo pipefail

SENTINEL_URL="${SENTINEL_URL:-http://127.0.0.1:8787}"
PROXY_BIN="${PROXY_BIN:-target/debug/mcp-proxy}"

echo "[verify] sentinel: ${SENTINEL_URL}"
curl -sf "${SENTINEL_URL}/v1/chain/head" >/dev/null || {
  echo "[verify] ERROR: sentineld not reachable at ${SENTINEL_URL}"
  exit 1
}

echo "[verify] proxy: ${PROXY_BIN}"
test -x "${PROXY_BIN}" || {
  echo "[verify] ERROR: proxy binary not found or not executable: ${PROXY_BIN}"
  exit 1
}

REQ_FILE="./ZZZ_DENY_TEST_FROM_SCRIPT.txt"
rm -f "${REQ_FILE}" || true

cat <<'EOF' | "${PROXY_BIN}" | tee /tmp/mcp_proxy_verify_out.jsonl >/dev/null
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"sentinel-os-verify","version":"0.0.1"}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"write_file","arguments":{"path":"./ZZZ_DENY_TEST_FROM_SCRIPT.txt","content":"hello"}}}
EOF

if test -f "${REQ_FILE}"; then
  echo "[verify] ERROR: file was created but should have been denied: ${REQ_FILE}"
  exit 1
fi

echo "[verify] OK: file not created"

echo "[verify] ledger head intent:"
curl -s "${SENTINEL_URL}/v1/chain/head" \
| jq -r '.intent_canon | @base64d | fromjson | {capability, target}'

echo "[verify] ledger head decision:"
curl -s "${SENTINEL_URL}/v1/chain/head" \
| jq -r '.decision_canon | @base64d | fromjson | {decision, reason, policy}'