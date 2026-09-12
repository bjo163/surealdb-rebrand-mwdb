# M3 Logical Change Model

Status: **PROTOTYPE / PARTIAL**

## Purpose

M3 defines a logical change format that is independent of the physical WAL/storage implementation. The format is intended to become the stable unit for future synchronization, replay, branching, verification, and federation.

## `LogicalChangeV1`

```text
schema_version
change_id
actor_id
object_id
action/operation
payload
created_at_ms
logical_clock
parents[]
content_hash
```

The prototype currently lives in `mwdb/local-first/src/lib.rs`.

## Design rules

- `change_id` is the idempotency key.
- `actor_id` identifies the writer at the logical protocol layer.
- `parents` carry causal ancestry.
- `logical_clock` provides an ordering signal without claiming global consensus.
- `content_hash` is calculated from a deterministic canonical representation of the logical fields.
- verification must fail when the signed/hashed content is modified.
- physical WAL records remain an engine concern and are not exposed as the sync format.

## Current evidence

The prototype contains deterministic hash generation and self-verification tests. Full replay/export/import remains M3 work.

## Next

1. freeze the V1 schema in a versioned protocol document;
2. add canonical export/import;
3. add deterministic replay against a small state machine;
4. add checkpoint/cursor semantics;
5. feed the resulting format into M4 sync.
