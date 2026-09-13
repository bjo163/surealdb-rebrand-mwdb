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
- checkpoint-based delta selection
- durable verified remote application
- standalone `mwdb/replay` import/export helpers
- replay/import tests for reconstruction, idempotent re-import, and malformed input

## Design rules

- `change_id` is the idempotency key.
- `actor_id` identifies the writer at the logical protocol layer.
- `parents` carry causal ancestry.
- `logical_clock` provides an ordering signal without claiming global consensus.
- `content_hash` is calculated from a deterministic canonical representation of the logical fields.
- physical WAL records remain an engine concern and are not exposed as the sync format.
- import must verify the complete logical change before mutating local state.

## Prototype limitations

The current JSON representation is deterministic for the Rust prototype, but cross-language canonicalization is not frozen yet. A future protocol spec must define canonical serialization byte-for-byte before signatures or federation depend on it.

The replay implementation currently supports the `set` operation used by the prototype. Unsupported logical operations are rejected rather than silently approximated.

Persistent checkpoint storage and a network-facing change-feed API are still pending.

## Remaining M3 work

1. formal versioned protocol specification;
2. canonical cross-language encoding;
3. replay engine for arbitrary supported operations;
4. persistent checkpoints/cursors;
5. network-facing change-feed API;
6. deterministic multi-implementation compatibility fixtures.

## Verification

`mwdb/local-first` covers durable journaling, startup replay, verified remote application, and checkpoint metadata. `mwdb/replay` validates JSONL export/import independently and ensures re-import is idempotent.

## Gate

M3 is complete when a known logical history can be exported, validated, imported, replayed, and reconstructed deterministically across supported implementations.
