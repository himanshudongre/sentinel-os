
---

# 📄 Updated `README.md`

```markdown
# Sentinel OS

Local-first trust control plane for agent systems.

Sentinel OS ensures every agent action is:

- Canonicalized
- Cryptographically signed
- Hash-chained
- Offline verifiable
- Bound to a server identity

This is not just logging.
This is enforceable trust infrastructure.

---

## Why Sentinel Exists

Modern AI agents can:

- Call tools
- Modify files
- Access networks
- Execute shell commands
- Spend money
- Exfiltrate data

Most systems log after the fact.

Sentinel logs before execution and enforces policy (coming in v0.2).

---

## Core Properties

- Server-authoritative signing
- Ed25519 cryptographic proofs
- RFC8785 canonical JSON
- Append-only hash chain
- Database identity lock
- Offline chain verification
- Strict CI enforcement

---

## Architecture

Agent → Seatbelt (policy) → Sentinel → Ledger

- Seatbelt decides.
- Sentinel signs.
- Ledger proves.
- Clients verify.

---

## Version Status

v0.1 — Centralized Authoritative Ledger  
v0.2 — Policy Enforcement (Seatbelt)  
v0.3 — MCP Proxy Integration  
v0.4 — Agent Adapters  
v1.0 — Production Hardened Trust Plane

---

## Quickstart

```bash
cargo run -p sentineld
cargo run -p sentinelctl