# MW-DB Authenticated Replication Wave

Date: 2026-09-13

## Delivered

The `mwdb/replication` crate is the orchestration boundary for the previously isolated replication primitives. It now wires:

```text
Capability / protocol boundary
        ↓
Peer identity (Ed25519)
        ↓
Authenticated frame (shared-key prototype)
        ↓
Persistent replay window
        ↓
Signed sync batch
        ↓
Logical-change validation
        ↓
Idempotent durable apply
        ↓
Canonicalized batch evidence
        ↓
Merkle batch root
        ↓
Persistent peer cursor
        ↓
Sync observability metrics
```

## Durable state

`mwdb-replication` persists replay-window state and accepted change IDs in `replication-state.json`, while peer checkpoints remain in the existing cursor store. State writes use a temporary file followed by rename.

## Trust boundary

The current orchestration verifies the public-key signature and binds the signed peer identity to the authenticated frame `node_id`. It does **not** yet implement a persistent trust/allowlist store, key rotation, or revocation. Embedding a public key in a valid signature envelope is therefore not equivalent to authorization.

## Security status

The frame authenticator is still a shared-key prototype. The identity layer uses Ed25519 for message signatures. These layers are intentionally separate until a versioned key/trust protocol is frozen.

## Remaining P0

- GAP-01: freeze deterministic cross-language numeric canonicalization.
- GAP-02: independent canonical verifier / external vectors.
- GAP-05: persistent trust-store integration.
- GAP-10: wire signed logical-change identity into a stable signed-change/Merkle protocol.
- GAP-12: partition/federation test suite.
- GAP-15: provenance and reproducible baseline.

## Exit criteria for the next wave

1. Capability negotiation is a fail-closed state machine rather than only an intersection helper.
2. Peer trust is persisted and explicit.
3. Merkle proofs have versioned wire serialization and compatibility vectors.
4. A transport adapter can carry the authenticated frame without changing replication semantics.
5. A two-peer partition test proves convergence after drop, duplicate, reorder, reconnect, and restart.

This wave is a prototype integration milestone, not a production-distributed release.
