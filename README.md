# Sentinel OS

Local-first trust control plane for agentic systems.

## What it does (v0.1)
- Intercepts MCP tool invocations through a proxy
- Evaluates policy (deny by default)
- Requires approvals when configured
- Emits cryptographically verifiable proof bundles
- Stores proofs in an append-only, tamper-evident log

## Repository layout
- `crates/sentinel-core`: schemas, canonicalization, hashing, signing, verification
- `crates/sentineld`: local daemon API, policy evaluation, proof log
- `crates/sentinelctl`: CLI to approve, inspect, verify
- `crates/mcp-proxy`: MCP firewall proxy integration
- `docs/`: spec, threat model, invariants, roadmap

## Quick start
See `docs/spec-v0.1.md` and `docs/ROADMAP.md`.
