# MW-DB Audit Gap & Task Register

Status: ACTIVE — RELEASE TRACK

This register is the stable local task map for MW-DB. GitHub Issues remain unavailable in the repository API (issue creation returns HTTP 410), so the task IDs below are the authoritative issue-ready identifiers until Issues can be enabled.

## Audit finding

The recent MW-DB implementation work is executable prototype work, not documentation-only. The repository contains focused crates/tests for local-first durability, sync, replay/import, persistent cursors, observability, authenticated frames, canonical encoding, signed changes, Merkle roots/proofs, capability intersection, public-key peer identity, and replay-window checks. The main remaining gap is integration and production gates.

## Release-track evidence added

- `docs/MWDB-RELEASE-GATE.json` — machine-readable P0 release gate; current release status is intentionally `blocked`.
- `docs/MWDB-PROVENANCE.md` — upstream identity, attribution, and release provenance policy.
- `docs/canonical-vectors.json` — language-neutral canonicalization vectors and rules.
- `tools/canonical_verify.mjs` — independent Node.js vector implementation.
- `mwdb/integration/` — cross-crate integration harness covering canonicalization, public-key identity, signing, Merkle proof, authenticated replay protection, local write → sync → persistent cursor.
- `mwdb/canonical/src/lib.rs` — canonical integral-number normalization plus vector tests.
- CI fix commit `c73b53f51c81859399c26d08f1484a8abcd8be69` raises fuzz recursion limits for the structured executor targets; push workflows must complete successfully before release evidence is accepted.

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
Status: INTEGRATION TESTED (prototype)
Target: mwdb/canonical
Progress: integral-number normalization, language-neutral vectors, and independent Node vector verifier added.
Limit: cross-language hash agreement and full numeric edge-case contract remain open.

### MW-T08 — Signed change prototype
Status: DONE (prototype)
Target: mwdb/signing
Limit: production key lifecycle and full Merkle semantics remain open.

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
Status: INTEGRATION TESTED
Evidence: `docs/canonical-vectors.json`, `mwdb/canonical/src/lib.rs`
Close condition: independent implementation reproduces every vector and hash.

### GAP-02 — Independent canonical implementation
Priority: P0
Status: INTEGRATION TESTED
Evidence: `tools/canonical_verify.mjs`, `docs/canonical-vectors.json`
Close condition: CI verifies every vector; hash agreement is a separate release evidence requirement.

### GAP-03 — Sync/cursor integration
Priority: P0
Status: INTEGRATION TESTED
Evidence: `mwdb/integration/src/lib.rs`
Close condition: CI passes the two-replica local write → sync → persisted cursor fixture.

### GAP-04 — Observability integration
Priority: P1
Status: OPEN

### GAP-05 — Public-key peer identity integration
Priority: P0
Status: INTEGRATION TESTED
Evidence: `mwdb/integration/src/lib.rs`
Close condition: identity verification is enforced by the actual sync state machine, not only the harness.

### GAP-06 — Capability negotiation state machine
Priority: P1
Status: OPEN

### GAP-07 — Persistent replay protection
Priority: P0
Status: IN PROGRESS
Evidence: replay-window state serialization exists; integration harness verifies duplicate rejection.
Close condition: peer-scoped persistence and key lifecycle are enforced across restart and sync.

### GAP-08 — Key rotation / revocation
Priority: P1
Status: OPEN

### GAP-09 — Merkle proof serialization
Priority: P1
Status: OPEN

### GAP-10 — Signed-change + Merkle integration
Priority: P0
Status: INTEGRATION TESTED
Evidence: `mwdb/integration/src/lib.rs`
Close condition: signed logical changes become the authoritative Merkle leaves/state contribution in the production path.

### GAP-11 — Network transport adapter
Priority: P1
Status: OPEN

### GAP-12 — Partition/federation suite
Priority: P0
Status: OPEN

### GAP-13 — Arbitrary logical operations
Priority: P1
Status: OPEN

### GAP-14 — Snapshot / branch foundation
Priority: P2
Status: OPEN

### GAP-15 — M0 provenance baseline
Priority: P0
Status: INTEGRATION TESTED
Evidence: `docs/MWDB-PROVENANCE.md`
Close condition: release artifact records exact upstream baseline and compatibility delta.

### GAP-16 — Compatibility matrix
Priority: P1
Status: OPEN

### GAP-17 — Benchmark gate
Priority: P1
Status: OPEN

## Recommended execution order

GAP-15 -> GAP-01 -> GAP-02 -> GAP-03 -> GAP-05 -> GAP-07 -> GAP-10 -> GAP-12 -> P1 integration tasks -> GAP-14.

## Release rule

Production release is blocked until every P0 gate in `docs/MWDB-RELEASE-GATE.json` is green with reproducible CI evidence. Prototype presence is not sufficient evidence.

## Issue policy

Each GAP is intentionally written as issue-ready. When GitHub Issues are enabled, create one Issue per GAP and keep this register synchronized with issue numbers. Do not close a GAP merely because a prototype primitive exists; close only after the acceptance criteria and integration evidence are demonstrated.
