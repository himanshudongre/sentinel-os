# Sentinel OS – Project State Snapshot (v0.1.0)

## Vision

Sentinel OS is a cryptographic trust control plane for agentic systems.
It provides tamper-evident logging, policy enforcement scaffolding, and verifiable action chains for AI agents and automation workflows.

Core philosophy:
- Deterministic
- Cryptographically verifiable
- Minimal trust assumptions
- Local-first
- Extensible toward external anchoring

---

## Implemented Components

### sentinel-core

- RFC 8785 JCS canonical JSON
- SHA256 log hashing
- ed25519 signing
- ProofBundle construction
- Proof verification
- Chain verification
- Integration tests
- Clippy + fmt enforced in CI

### sentineld

- SQLite append-only proof store
- WAL mode
- Server-enforced prev_log_hash
- HTTP API:
  - POST /v1/proofs
  - GET /v1/proofs/:id
  - GET /v1/chain
  - GET /v1/chain/head
- Chain consistency enforcement

### sentinelctl

- Generates demo proofs
- Posts to sentineld
- Retrieves chain
- Verifies chain offline

---

## Security Model (Current)

Model: Client-signed proofs, server-enforced chain order.

Server enforces:
- Single canonical chain
- No fork insertion
- Proper prev_log_hash

Client:
- Builds and signs ProofBundle

---

## Milestones Completed

v0.1.0-proof-pipeline
- End-to-end cryptographic ledger working
- CI green
- Formatting enforced
- Chain verification complete

---

## Hardening In Progress

- Remove client control of prev_log_hash
- Eventually move proof construction fully server-side
- Evaluate centralized signing vs decentralized signing

---

## Future Directions

Phase 1:
- Server-built proofs
- Transition API instead of proof API
- Remove trust from client

Phase 2:
- MCP proxy integration
- Policy engine

Phase 3:
- Optional external anchoring (blockchain or transparency logs)

---

## Architectural Principles

- Canonicalization locked
- Deterministic hashing
- Append-only ledger
- Verification must be offline-capable
- No network dependencies in core cryptography

---

## Immediate Next Step

Update sentinelctl to:
1. GET /v1/chain/head
2. Use returned log_hash as prev_log_hash
3. Only use 00... if chain empty

Then refactor toward server-built proofs.