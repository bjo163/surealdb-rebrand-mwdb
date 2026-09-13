# MW-DB Next Wave Status — 2026-09-13

## Completed in this wave

- Capability negotiation is now versioned, deterministic, and fail-closed.
- Persistent peer trust store supports active/revoked states and restart recovery.
- Replication session enforces trusted Ed25519 peer identity before applying changes.
- Replication frames advertise the required MW-DB v1 capabilities.
- Merkle inclusion proofs now have a versioned JSON wire representation.
- Sender nonce state is durable across process restart.
- End-to-end replication tests cover capability agreement, trust rejection, revocation, replay, cursor persistence, Merkle evidence, and observability.

## Current gates

| Area | Status |
|---|---|
| Local-first durability | Prototype complete |
| Logical change + replay/import | Prototype complete |
| Sync + conflict/CRDT | Prototype complete |
| Authenticated replication pipeline | Prototype complete |
| Capability enforcement | Implemented |
| Peer trust / revocation | Implemented |
| Versioned Merkle proof | Implemented |
| Canonical numeric cross-language contract | P0 open |
| Independent canonical verifier/vector suite | P0 open |
| Real network transport | P1 open |
| Partition/federation suite | P0 open |
| Key rotation | P1 open |
| Branch/time travel | P2 open |
| Benchmark gate | P1 open |

MW-DB is not declared production-ready distributed infrastructure until the remaining P0 gates are closed or explicitly waived.
