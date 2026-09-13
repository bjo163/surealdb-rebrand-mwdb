# MW-DB Audit — 2026-09-13

## Scope

Audit covers the MW-DB overlay added on top of the SurrealDB substrate: local-first durability, logical changes, replay/import, cursors, sync, conflict/CRDT prototypes, observability, authenticated frames, canonical encoding, signing, Merkle proofs, and capability negotiation.

## Current conclusion

The repository has a substantial prototype foundation, but the system is not yet a production distributed database. The main remaining risks are cross-language canonicalization, authenticated peer identity/replay protection, integration of cursor + observability into the real sync path, proof/signature composition, transport abstraction, and partition/federation verification.

## P0 gaps

1. Canonical numeric normalization and published cross-language vectors.
2. Independent non-Rust canonical verifier.
3. Sync must own persistent peer cursor advancement and recovery.
4. Public-key peer identity and asymmetric signatures.
5. Replay-window / nonce / sequence protection for authenticated frames.
6. Signed-change and Merkle root/inclusion semantics must be one verifiable protocol.
7. Deterministic partition/federation suite with no silent divergence.
8. Provenance baseline: upstream commit, toolchain, target, reproduction command, license/attribution inventory.

## P1 gaps

1. Sync must emit observability metrics directly.
2. Capability negotiation must become a state machine with enforcement.
3. Key rotation/revocation lifecycle.
4. Merkle proof serialization and versioning.
5. Replaceable network transport adapter.
6. Expand logical operations beyond `set`.
7. Compatibility/migration matrix.
8. Benchmark/regression gate.

## P2 gaps

1. Snapshot / branch / diff / merge / rollback foundation.

## Evidence rule

A milestone is only marked DONE when its acceptance criteria are demonstrated by code and focused tests. A prototype crate or document alone does not close a production gate.

## Retroactive issue policy

All work already landed as prototype code/docs/tests is recorded separately from remaining gaps. GitHub Issues are now enabled in the active repository connection, so current and future gaps should be represented as actual issues as well as docs/tasks.
