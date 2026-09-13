# MW-DB Audit Gap & Task Register

Status: ACTIVE

This register is the stable local task map for MW-DB. GitHub Issues remain unavailable in the repository API (issue creation returns HTTP 410), so the task IDs below are the authoritative issue-ready identifiers until Issues can be enabled.

## Audit finding

The recent MW-DB implementation work is executable prototype work, not documentation-only. The repository contains focused crates/tests for local-first durability, sync, replay/import, persistent cursors, observability, authenticated frames, canonical encoding, signed changes, Merkle roots/proofs, capability intersection, public-key peer identity, and replay-window checks. The main remaining gap is integration and production gates.

## Completed implementation tasks

### MW-T01 — Local-first durable journal
Status: DONE (prototype)
Target: mwdb/local-first

### MW-T02 — Sync v1 prototype
Status: DONE (prototype)
Target: mwdb/sync

### MW-T03 — Replay/import surface
Status: DONE (prototype)
Target: mwdb/replay

### MW-T04 — Persistent peer cursor
Status: DONE (prototype)
Target: mwdb/cursor

### MW-T05 — Sync observability
Status: DONE (prototype)
Target: mwdb/observability

### MW-T06 — Authenticated frame boundary
Status: DONE (prototype)
Target: mwdb/auth
Limit: shared-key authentication only; public-key identity and key lifecycle are separate.

### MW-T07 — Canonical encoding boundary
Status: DONE (prototype)
Target: mwdb/canonical
Limit: numeric normalization and independent verification remain open.

### MW-T08 — Signed change prototype
Status: DONE (prototype)
Target: mwdb/signing
Limit: integration with public identity and Merkle semantics remains open.

### MW-T09 — Merkle state root
Status: DONE (prototype)
Target: mwdb/merkle

### MW-T10 — Merkle inclusion proof
Status: DONE (prototype)
Target: mwdb/merkle
Limit: external serialization/vector contract remains open.

### MW-T11 — Capability intersection
Status: DONE (prototype)
Target: mwdb/capabilities
Limit: wire state machine/enforcement remains open.

### MW-T12 — Public-key peer identity
Status: DONE (prototype)
Target: mwdb/identity
Limit: trust store, revocation, persistence integration, and sync enforcement remain open.

### MW-T13 — Replay-window enforcement
Status: DONE (prototype)
Target: mwdb/auth
Limit: persistent peer-scoped replay state and key lifecycle remain open.

## Gaps found by audit

### GAP-01 — Canonical numeric normalization
Priority: P0

### GAP-02 — Independent canonical implementation
Priority: P0

### GAP-03 — Sync/cursor integration
Priority: P0

### GAP-04 — Observability integration
Priority: P1

### GAP-05 — Public-key peer identity integration
Priority: P0

### GAP-06 — Capability negotiation state machine
Priority: P1

### GAP-07 — Persistent replay protection
Priority: P0

### GAP-08 — Key rotation / revocation
Priority: P1

### GAP-09 — Merkle proof serialization
Priority: P1

### GAP-10 — Signed-change + Merkle integration
Priority: P0

### GAP-11 — Network transport adapter
Priority: P1

### GAP-12 — Partition/federation suite
Priority: P0

### GAP-13 — Arbitrary logical operations
Priority: P1

### GAP-14 — Snapshot / branch foundation
Priority: P2

### GAP-15 — M0 provenance baseline
Priority: P0

### GAP-16 — Compatibility matrix
Priority: P1

### GAP-17 — Benchmark gate
Priority: P1

## Recommended execution order

GAP-15 -> GAP-01 -> GAP-02 -> GAP-03 -> GAP-05 -> GAP-07 -> GAP-10 -> GAP-12 -> P1 integration tasks -> GAP-14.

## Issue policy

Each GAP is intentionally written as issue-ready. When GitHub Issues are enabled, create one Issue per GAP and keep this register synchronized with issue numbers. Do not close a GAP merely because a prototype primitive exists; close only after the acceptance criteria and integration evidence are demonstrated.
