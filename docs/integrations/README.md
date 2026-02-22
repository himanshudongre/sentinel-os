# Sentinel OS

A local, verifiable trust kernel for agent execution and state.

---

## Why This Project Exists

Agents are becoming capable of acting:

- Calling tools  
- Executing code  
- Modifying files  
- Making API requests  
- Managing infrastructure  
- Storing and retrieving memory  

But there is a gap between capability and responsibility.

When agents move from experimentation to operational systems, organizations face fundamental questions:

- How do we prove what the agent did?
- How do we enforce policy deterministically?
- How do we prevent silent misuse of tools?
- How do we audit memory mutations?
- How do we verify integrity independently of the model provider?
- How do we replay and reconstruct execution chains?

Today’s guardrails are typically:

- Prompt constraints  
- Provider moderation  
- Internal logs  
- Best-effort policy checks  

These mechanisms are useful, but they are not:

- Cryptographically verifiable  
- Vendor-neutral  
- Deterministically replayable  
- Tamper-evident  

Sentinel OS exists to explore a different approach:

Treat agent execution like critical infrastructure.

---

## What Sentinel OS Is

Sentinel OS is a local-first trust kernel for agent systems.

It provides three core components:

---

### 1. Sentinel (Integrity Layer)

A canonical, signed, append-only ledger of agent transitions.

Each transition follows a strict contract:

Intent → Policy Decision → Execution Digest → Signed Proof → Hash-Chained Ledger

Properties:

- RFC8785 canonicalization  
- SHA-256 hashing  
- Ed25519 signatures  
- Immutable hash chaining  
- Offline verification  
- Host identity binding  

This makes agent behavior:

- Tamper-evident  
- Auditable  
- Deterministically reproducible  
- Cryptographically verifiable  

---

### 2. Seatbelt (Policy Layer)

A deterministic policy engine with a default-deny model.

Seatbelt:

- Evaluates `TransitionIntent`  
- Produces authoritative allow/deny decisions  
- Generates canonical, signed policy references  
- Ensures clients cannot self-authorize actions  

Policy becomes:

- Explicit  
- Deterministic  
- Verifiable  
- Bound into the proof chain  

This separates “what the agent wants to do” from “what the system allows.”

---

### 3. Sentinel Memory (State Layer — Roadmap)

Memory is not just storage.

In agent systems, memory becomes:

- A source of future decisions  
- A vector for corruption  
- A replay surface  
- A policy boundary  

Sentinel Memory will:

- Record memory writes as signed transitions  
- Hash memory content  
- Audit retrieval queries  
- Bind memory namespaces to policy  
- Enable deterministic replay of memory state  

This makes agent state as verifiable as agent actions.

---

## The Problem Being Addressed

The limiting factor for serious agent adoption is not intelligence.

It is operational trust.

Organizations hesitate to delegate meaningful authority because they cannot:

- Prove policy compliance  
- Reconstruct action history deterministically  
- Detect tampering  
- Bind actions to identity  
- Audit cross-agent workflows  
- Verify integrity without trusting the model vendor  

Sentinel OS explores whether a cryptographic trust kernel can reduce that barrier.

---

## What This Is Not

Sentinel OS does not:

- Replace model guardrails  
- Improve reasoning quality  
- Provide distributed consensus  
- Solve alignment  
- Guarantee safety in adversarial environments  
- Replace OS-level security  

It focuses narrowly on:

Deterministic, verifiable control over agent transitions and state.

---

## Architecture Overview

### sentinel-core
- Canonicalization (RFC8785-JCS)  
- Hashing utilities  
- Signature generation  
- Proof verification  
- Chain verification  

### sentineld
- Authoritative signing server  
- Seatbelt policy evaluation  
- Append-only SQLite ledger  
- Identity lock (DB bound to server key)  

### seatbelt-core
- Deterministic policy engine  
- Default deny enforcement  
- Canonical policy hashing  

### sentinelctl
- Example client  
- Offline chain verification  

### mcp-proxy (v0.3)
- Tool call interception  
- Policy enforcement before execution  
- Execution digest capture  

---

## Design Principles

- Local-first  
- Deterministic  
- Default deny  
- Minimal dependencies  
- Cryptographically verifiable  
- Vendor-neutral  
- Explicit over implicit  

---

## Current Status

v0.2 — Server-side policy enforcement

- Clients cannot supply authoritative decisions  
- Policy decisions are signed  
- Ledger is append-only and hash chained  
- Offline verification works  

Next milestone:

v0.3 — Real enforcement via MCP proxy

---

## Who This May Be Useful For

- Teams running tool-enabled agents  
- Multi-agent systems  
- Infrastructure automation workflows  
- Research into trustworthy AI systems  
- Enterprises exploring operational governance for agents  
- Developers building agent platforms  

---

## Long-Term Vision

If agent systems become infrastructure, they will require:

- Integrity guarantees  
- Policy transparency  
- Deterministic replay  
- Vendor-independent verification  
- Auditable memory  

Sentinel OS explores how to provide that layer.