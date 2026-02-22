# Sentinel OS — Project State

## Version
v0.1.0 — Centralized Authoritative Ledger

---

## Overview

Sentinel OS is a **local-first trust control plane for agent systems**.

It ensures that every agent transition is:

- Canonicalized
- Cryptographically signed
- Hash-chained
- Offline verifiable
- Bound to a server identity

Sentinel is not just logging. It is the foundation for enforceable agent governance.

---

## Architecture (v0.1)

### Write Path (Authoritative)

Client → `POST /v1/transitions` → sentineld

Server:

1. Computes authoritative `prev_log_hash`
2. Builds canonical proof bundle (RFC8785-JCS)
3. Signs using Ed25519 server key
4. Appends to SQLite ledger
5. Returns signed proof

Clients:
- Do NOT construct proofs
- Do NOT choose `prev_log_hash`
- Do NOT sign

Server is the only signing authority.

---

### Verification Model

Verification loop:

1. Client fetches `/v1/keys/current`
2. Client fetches `/v1/chain`
3. Client verifies entire chain offline
4. Chain integrity requires:
   - Valid signature
   - Hash continuity
   - Correct public key

Verification works independently of server after retrieval.

---

## Security Invariants (v0.1)

- Only server signs proofs.
- Ledger is append-only.
- Database is bound to a single server public key.
- Canonicalization version is fixed.
- All transitions are signed before persistence.

---

## Database Identity Lock

Each `sentinel.db` is permanently bound to a single `server_pubkey_id`.

If the server key changes while the DB remains:

- Server refuses to start.
- Prevents chain corruption.
- Prevents mixed-signature ledgers.

---

## What Exists

- `sentinel-core` — canonicalization, hashing, signing, verification
- `sentineld` — authoritative signing service
- `sentinelctl` — client & verifier
- Hash-chained SQLite ledger
- Strict CI (fmt, clippy, tests)
- Public key endpoint
- Chain endpoint

---

## What Does NOT Exist Yet

- Policy enforcement
- Deny-by-default mode
- Capability gating
- Tool interception (MCP proxy)
- Human approval workflows
- Key rotation
- Multi-issuer ledgers
- Distributed consensus

---

## Threat Model (v0.1)

### Assumptions

- Agent is untrusted.
- Tools are untrusted.
- Sentinel is local trust root.
- Host OS integrity assumed.
- SQLite file integrity assumed unless tampered.
- Server private key stored locally.

### Out of Scope

- Compromised OS
- Distributed Byzantine networks
- Remote consensus
- Hardware key security

---

## Current Version
v0.2 — Seatbelt Policy Decisions (Centralized)

---

## Overview

Sentinel OS is a local-first trust control plane for agent systems.

It provides:

- Canonicalization (RFC8785-JCS)
- Cryptographic signing (Ed25519)
- Append-only hash-chained ledger
- Offline verification
- Server identity lock (DB bound to server public key)
- Policy decisions via Seatbelt (default deny)

---

## Architecture

### Core Contract

Intent → Policy Decision → Signed Proof → Append to Ledger → Offline Verification

This contract is universal. Adapters (MCP proxy, HTTP gateway, shell wrapper) translate real actions into Intent.

---

## What Exists Today

### sentinel-core
- Canonicalize intent and decision
- Hashing utilities
- Sign ProofBundle
- Verify signature and chain integrity

### seatbelt-core (v0.2)
- Loads local JSON policy
- Evaluates TransitionIntent deterministically
- Produces: decision, reason, policy_hash

### sentineld
- Authoritative server
- Computes authoritative prev_log_hash
- Evaluates Seatbelt policy (server-side)
- Builds and signs ProofBundle
- Appends to SQLite ledger
- Exposes endpoints:
  - POST /v1/transitions
  - GET /v1/keys/current
  - GET /v1/chain
  - GET /v1/chain/head

### sentinelctl
- Sends transitions
- Fetches key and chain
- Verifies chain offline

---

## Security Invariants (v0.2)

- Clients cannot provide authoritative decisions.
- Server computes policy decision and signs it.
- Ledger remains append-only and hash chained.
- Database is bound to a single server public key.
- Canonicalization version is fixed.

---

## Threat Model

Assumptions:
- Agent is untrusted.
- Tools are untrusted.
- Sentinel is local trust root.
- Host OS integrity assumed.

Out of scope:
- Compromised OS
- Distributed consensus / byzantine networks
- Hardware key security

---

## What Is Missing

- Tool interception and enforcement (blocking)
- Multi-adapter integrations (MCP, HTTP, shell, filesystem)
- Key rotation
- Policy explainability (rule identifiers)
- Human approval workflow
- Production hardening (rate limits, auth)

---

## Next Milestone

v0.3 — MCP Proxy Adapter (first enforcement integration)

Goal:
Intercept real tool calls, consult Sentinel, allow or deny, then log execution digest.

This will be the first end-to-end real agent integration tutorial in-repo.