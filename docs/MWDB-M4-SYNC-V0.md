# MW-DB M4 Sync Protocol v0

Status: **PROTOTYPE / IN PROGRESS**

M4 now has a transport-neutral sync core and a deterministic two-replica convergence fixture. Network transport, authentication, resumable sessions, and production conflict resolution remain future work.

## Purpose

M4 builds on the M3 logical-change format. The transport carries logical changes, not physical storage/WAL records.

## Protocol v1 envelope

```text
SyncHello
  protocol
  node_id
  last_change_id
  last_clock

SyncBatch
  protocol
  changes[]

SyncAck
  protocol
  accepted[]
  rejected[]
  checkpoint
```

## Required semantics

1. Protocol version is explicit.
2. A batch is rejected when a logical change fails content-hash verification.
3. Change IDs are idempotency keys.
4. The receiver applies verified changes only once.
5. Acknowledgements can be persisted into the local durable queue.
6. Checkpoints select only changes after a known anchor; an unknown checkpoint is an explicit error.
7. Remote changes are persisted through the same append-only logical journal used for local changes.
8. Transport is intentionally replaceable; HTTP/WebSocket/P2P are future adapters.

## Implemented prototype

`mwdb/sync` contains:

- protocol structures and JSON encoding/decoding;
- logical-change hash verification;
- checkpoint-based delta selection;
- idempotent batch application;
- persistent acknowledgement helpers;
- conservative conflict classification;
- deterministic two-replica offline divergence/reconnect testing.

`mwdb/local-first` additionally exposes:

- `logical_changes_since(checkpoint)`;
- durable `apply_remote(change)`;
- replay-safe duplicate suppression.

## Convergence fixture

The current test models two offline replicas:

```text
Replica A: doc:a = 1
Replica B: doc:b = 2
        │     │
        └─ sync both ways ─┘
               ↓
       A and B contain both
       retrying the same batch
       is idempotent
```

This proves the prototype data path, not production distributed consensus or conflict-free merging.

## Conflict policy v0

The classifier is intentionally conservative:

| Relationship | Classification | Automatic overwrite |
|---|---|---|
| one change names the other as a parent | `CausallyOrdered` | allowed only by higher-level policy |
| different object IDs | `Mergeable` | safe for this prototype model |
| same object, no causal relation | `ConcurrentSameObject` | **never silently overwrite** |

Current `set` semantics replace a whole object value, so same-object concurrent writes remain a conflict until a field-aware merge model is introduced.

## Not production-ready yet

- authenticated peer identity and session handshake;
- transport adapter with reconnect/resume;
- persisted remote checkpoint state;
- message-size/backpressure limits;
- network fault injection and partition tests;
- field-aware CRDT/merge structures;
- cross-language canonical serialization for hashes;
- signed changes and proof verification.

## Next gate

```text
M4-A  checkpoint + convergence core      DONE (prototype)
  ↓
M4-B  conflict taxonomy                  IN PROGRESS
  ↓
M4-C  mergeable / CRDT structures        NEXT
  ↓
M7    authenticated distributed transport
```

## Safety rule

Never weaken verification to make synchronization succeed. Invalid logical changes must remain rejected and observable, and concurrent same-object writes must never be silently treated as conflict-free.
