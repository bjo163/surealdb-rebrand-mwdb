# MW-DB Audit Gap & Task Register

Status: ACTIVE

This register exists because GitHub Issues are disabled in this repository. Every task below is intended to be issue-ready if Issues are enabled later.

## Audit finding

The recent MW-DB implementation work is not documentation-only. The repository contains executable prototype crates and focused tests for local-first durability, sync, replay/import, persistent cursors, observability, authenticated frames, canonical encoding, signed changes, Merkle roots/proofs, and capability intersection. The audit found that the main gap is integration and production gates rather than absence of primitives.

## Completed implementation tasks

### MW-T01 — Local-first durable journal
Status: DONE (prototype)
Target: mwdb/local-first
Output: durable changes.log, restart replay, pending/ack/reject/conflict state, subscriptions, remote apply.
Verification: focused unit tests and dedicated CI workflow.

### MW-T02 — Sync v1 prototype
Status: DONE (prototype)
Target: mwdb/sync
Output: hello/batch/ack, checkpoint delta selection, verification, idempotent apply, conflict policy, CRDT lab, fault harness.
Verification: focused unit tests and dedicated CI workflow.

### MW-T03 — Replay/import surface
Status: DONE (prototype)
Target: mwdb/replay
Output: JSONL export/import, verified replay, idempotent re-import, malformed-input tests.
Verification: dedicated CI workflow.

### MW-T04 — Persistent peer cursor
Status: DONE (prototype)
Target: mwdb/cursor
Output: durable peer checkpoints, restart recovery, malformed-state rejection, idempotent removal.
Verification: focused tests and isolated crate.

### MW-T05 — Sync observability
Status: DONE (prototype)
Target: mwdb/observability
Output: typed sync events, counters, snapshots, deterministic merge.
Verification: focused tests and isolated CI.

### MW-T06 — Authenticated frame boundary
Status: DONE (prototype)
Target: mwdb/auth
Output: protocol-versioned shared-key authenticated frame with keyed BLAKE3 integrity checks.
Limit: not public-key identity, not replay-window enforcement, not key rotation.

### MW-T07 — Canonical encoding boundary
Status: DONE (prototype)
Target: mwdb/canonical + docs/MWDB-M6-CANONICAL-ENCODING.md
Output: recursively sorted JSON prototype representation and hash boundary.
Limit: numeric normalization and independent cross-language verification are still open.

### MW-T08 — Signed change prototype
Status: DONE (prototype)
Target: mwdb/signing
Output: signed logical-change envelope and tamper detection using current keyed integrity design.
Limit: not asymmetric/public-key signatures.

### MW-T09 — Merkle state root
Status: DONE (prototype)
Target: mwdb/merkle
Output: deterministic leaf hashing, ordered tree root, changed-state root tests.

### MW-T10 — Merkle inclusion proof
Status: DONE (prototype)
Target: mwdb/merkle/proofs.rs
Output: proof builder + independent verifier + tamper test.
Limit: proof serialization/versioning and external verifier vectors remain.

### MW-T11 — Capability intersection
Status: DONE (prototype)
Target: mwdb/capabilities
Output: protocol-versioned deterministic capability intersection.
Limit: no wire-level negotiation state machine or enforcement hook yet.

## Gaps found by audit

### GAP-01 — Canonical numeric normalization
Priority: P0
Task: freeze representation of numbers and edge values; publish byte/hash vectors.
Acceptance: independent implementation reproduces every vector.

### GAP-02 — Independent canonical implementation
Priority: P0
Task: implement a tiny non-Rust reference/vector verifier outside mwdb/canonical.
Acceptance: same canonical bytes and BLAKE3 hashes for all published vectors.

### GAP-03 — Sync/cursor integration
Priority: P0
Task: make sync selection/resume read and advance durable per-peer cursors automatically.
Acceptance: interrupted transfer resumes from persisted cursor without duplicate effects.

### GAP-04 — Observability integration
Priority: P1
Task: emit observability events from actual sync/apply paths rather than requiring callers to report events manually.
Acceptance: send/receive/apply/duplicate/reject/retry/conflict/queue-depth transitions are observable from one sync execution.

### GAP-05 — Public-key peer identity
Priority: P0
Task: add asymmetric identity and signature verification for federated peers.
Acceptance: identity is bound to a peer, unsigned/tampered changes are rejected, and identity metadata survives restart.

### GAP-06 — Capability negotiation state machine
Priority: P1
Task: turn capability intersection into a handshake with protocol compatibility and unsupported-operation rejection before mutation.
Acceptance: incompatible or unauthorized capabilities fail before state application.

### GAP-07 — Replay protection
Priority: P0
Task: add nonce/sequence/window semantics to authenticated transport.
Acceptance: captured frames cannot be accepted outside the configured replay window.

### GAP-08 — Key rotation / revocation
Priority: P1
Task: version keys and define rotation/revocation lifecycle.
Acceptance: old keys can be revoked without corrupting existing durable state and active peers migrate deterministically.

### GAP-09 — Merkle proof serialization
Priority: P1
Task: version and serialize proofs for external verifiers.
Acceptance: proof JSON/bytes round-trip and verifies against a published root vector.

### GAP-10 — Signed-change + Merkle integration
Priority: P0
Task: define what exactly is signed and what is committed to the Merkle root.
Acceptance: verifier can authenticate change identity and independently verify inclusion in the state/change root.

### GAP-11 — Network transport adapter
Priority: P1
Task: bind authenticated frames to a replaceable socket/stream abstraction.
Acceptance: transport can send batches, receive acknowledgements, retry, and report metrics without embedding a specific networking library in core logic.

### GAP-12 — Partition/federation suite
Priority: P0
Task: multi-node deterministic suite covering partition, delay, duplication, reordering, crash, resume, conflict, and recovery.
Acceptance: convergence or explicit unresolved conflict; no silent divergence in all defined scenarios.

### GAP-13 — Arbitrary logical operations
Priority: P1
Task: expand replay/apply beyond `set` while preserving deterministic operation semantics.
Acceptance: create/update/delete or chosen operation set replays identically across implementations.

### GAP-14 — Snapshot / branch foundation
Priority: P2
Task: define logical snapshots, named histories, diff, merge/reject/rollback.
Acceptance: isolated branch mutation and deterministic restore/merge fixtures.

### GAP-15 — M0 provenance baseline
Priority: P0
Task: record upstream source commit, toolchain, targets, reproducible commands, license/attribution inventory, and ownership map.
Acceptance: clean contributor can reproduce the documented baseline and trace inherited components.

### GAP-16 — Compatibility matrix
Priority: P1
Task: document upstream SurrealDB compatibility and intentional MW-DB divergence.
Acceptance: each public divergence has a migration/compatibility statement.

### GAP-17 — Benchmark gate
Priority: P1
Task: benchmark local write/replay/sync/recovery bandwidth and latency before any native-engine decision.
Acceptance: reproducible report with baseline and regression threshold.

## Recommended execution order

P0-15 -> P0-01/02 -> P0-03 -> P0-07 -> P0-05 -> P0-10 -> P0-12 -> P1-04/06/08/09/11/13 -> P2-14.

## Important classification

The completed MW-T01..T11 items are implementation tasks already landed in code/docs/tests; they are recorded here retroactively so the project has an issue-ready audit trail.

The GAP items are not claims that the primitives are missing. Several are integration, interoperability, security-hardening, or production-exit-gate work that remains after the prototype layer.
