# M3 Logical Change Model

Status: **IN PROGRESS — PROTOTYPE**

M3 defines a logical change format independent from the physical WAL/storage implementation. The format is the planned stable unit for synchronization, replay, branching, verification, and federation.

## `LogicalChangeV1`

```text
schema_version
change_id
actor_id
object_id
operation
payload
created_at_ms
logical_clock
parents[]
content_hash
```

Implemented in `mwdb/local-first/src/lib.rs`.

## Implemented now

- versioned `LogicalChangeV1` envelope
- UUID change identity
- actor/object/operation/payload fields
- logical clock and causal parents
- deterministic BLAKE3 content hash
- hash verification
- conversion from durable local `ChangeEnvelope`
- logical JSONL export
- export verification
- checkpoint metadata
- deterministic export/replay test fixture

## Design rules

- `change_id` is the idempotency key.
- `actor_id` identifies the writer at the logical protocol layer.
- `parents` carry causal ancestry.
- `logical_clock` provides an ordering signal without claiming global consensus.
- `content_hash` is calculated from a deterministic canonical representation of the logical fields.
- physical WAL records remain an engine concern and are not exposed as the sync format.

## Important prototype limitation

The current JSON representation is deterministic for the Rust prototype, but cross-language canonicalization is not frozen yet. A future protocol spec must define canonical serialization byte-for-byte before signatures or federation depend on it.

## Remaining M3 work

1. formal versioned protocol specification;
2. canonical cross-language encoding;
3. replay engine for arbitrary supported operations, not only `set`;
4. import validation and idempotent ingestion;
5. persistent checkpoints/cursors;
6. change-feed API.

## Gate

M3 is complete when a known logical history can be exported, validated, imported, replayed, and reconstructed deterministically across supported implementations.
