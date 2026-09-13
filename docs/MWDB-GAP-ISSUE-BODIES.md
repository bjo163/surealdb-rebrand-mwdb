# Issue Bodies — MW-DB P0/P1/P2

These are canonical issue bodies for the audit gaps. They are intentionally implementation-focused.

## GAP-01 — Canonical numeric normalization
Define exact JSON number/edge-value rules, including integers, fractions, exponent forms, negative zero, overflow/underflow handling, and rejection policy. Publish byte/hash vectors and require an independent implementation to reproduce them.

**Acceptance:** every vector is reproduced byte-for-byte and hash-for-hash outside the canonical crate.

## GAP-02 — Independent canonical verifier
Implement a tiny verifier outside the Rust MW-DB implementation, preferably another language/runtime, using only published vectors and documented rules.

**Acceptance:** verifier passes all valid vectors and rejects altered bytes/numbers.

## GAP-03 — Sync/cursor integration
Move peer checkpoint selection/advancement into the sync engine. Persist the cursor only after successful durable application and make resume idempotent after crash.

**Acceptance:** interrupted transfer resumes from durable cursor with no duplicate effects and no skipped changes.

## GAP-05 — Public-key peer identity
Replace prototype symmetric peer authentication for federation with asymmetric identity. Bind peer ID to public key and sign logical changes or authenticated batches.

**Acceptance:** unsigned, altered, unknown, or revoked peer changes are rejected; identity persists across restart.

## GAP-07 — Replay protection
Add nonce/sequence semantics, replay window, and duplicate tracking to the authenticated transport envelope.

**Acceptance:** a captured valid frame is rejected outside the configured acceptance window; legitimate reordered frames remain safe where protocol semantics permit.

## GAP-10 — Signed-change + Merkle integration
Freeze the composition between canonical change bytes, signatures, Merkle leaves, roots, checkpoints, and proofs. Specify what a verifier proves.

**Acceptance:** independent verifier can validate change authenticity and inclusion against a published root.

## GAP-12 — Partition/federation suite
Create deterministic N-node tests for partition, delay, duplication, reordering, crash, restart, cursor resume, conflicts, and recovery.

**Acceptance:** all cases either converge to the defined state or surface an explicit unresolved conflict; never silently diverge.

## GAP-15 — Provenance/reproducible baseline
Record upstream commit, toolchain, target matrix, reproduction commands, license/attribution inventory, and MW-DB-owned code boundaries.

**Acceptance:** a clean checkout can reproduce the documented prototype baseline and inherited upstream components are traceable.

## GAP-04 — Sync observability integration
Emit metrics/events from actual sync paths rather than caller-managed reporting.

**Acceptance:** send/receive/apply/duplicate/reject/retry/conflict/cursor/queue-depth are emitted by one execution.

## GAP-06 — Capability negotiation state machine
Turn capability intersection into a versioned handshake and reject unsupported operations before mutation.

**Acceptance:** incompatible protocol/capability pairs fail closed before applying state.

## GAP-08 — Key rotation/revocation
Define key IDs, active/retired/revoked states, rotation ordering, and peer migration behavior.

**Acceptance:** rotation does not corrupt durable history and revoked keys cannot authenticate new operations.

## GAP-09 — Merkle proof serialization
Version proof format and add round-trip plus external-vector tests.

**Acceptance:** a proof can be serialized, transported, deserialized, and verified against a published root.

## GAP-11 — Network transport adapter
Define a transport trait independent of TCP/QUIC/WebSocket/etc., carrying authenticated frames and acknowledgements.

**Acceptance:** sync core can run over a test transport and a real transport without changing state logic.

## GAP-13 — Arbitrary logical operations
Expand the change model beyond `set` to a defined CRUD/patch operation subset with deterministic semantics.

**Acceptance:** operations replay identically across independent implementations.

## GAP-16 — Compatibility matrix
Document where MW-DB matches upstream SurrealDB and where semantics intentionally diverge.

**Acceptance:** every public divergence has migration/compatibility documentation.

## GAP-17 — Benchmark gate
Benchmark write, replay, sync, recovery, proof generation/verification, and payload overhead against an explicit baseline.

**Acceptance:** reproducible benchmark report with thresholds and regression detection.

## GAP-14 — Snapshot/branch foundation
Define logical snapshot IDs, named histories, diff, merge, reject, rollback, and branch isolation.

**Acceptance:** branch fixtures are isolated and deterministic and can be restored/merged without silent data loss.
