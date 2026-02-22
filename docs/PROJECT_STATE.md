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

## Next Milestone

v0.2 — Seatbelt Policy Engine

Goal:
Move from "log everything" to "enforce before execution".

Sentinel will become an active gate, not just a ledger.