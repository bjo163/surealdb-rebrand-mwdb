# MW-DB M4 Sync Protocol v0

Status: **PROTOTYPE / IN PROGRESS**

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
2. A batch is rejected when any logical change fails content-hash verification.
3. Change IDs are idempotency keys.
4. Receiver tracks applied IDs before reporting success.
5. Acknowledgements can be persisted into the local durable queue.
6. Checkpoints identify the last accepted change for resumable exchange.
7. Transport is intentionally replaceable; HTTP/WebSocket/P2P are future adapters.

## Current implementation

`mwdb/sync` contains protocol structures, encode/decode, verification, idempotent apply, and acknowledgement helpers.

This is not yet a network transport implementation and does not claim replica convergence.

## Next steps

- incremental batch selection from checkpoints;
- authenticated handshake;
- resumable transport adapter;
- two-replica convergence fixture;
- conflict classification before automatic merge;
- backpressure and observability.

## Safety rule

Never weaken verification to make synchronization succeed. Invalid logical changes must remain rejected and observable.
