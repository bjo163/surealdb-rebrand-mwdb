# M2 Local-First Implementation

Status: **IN PROGRESS**

This document tracks the first executable MW-DB differentiation slice.

## M2-A — Local-first contract

**Implemented in prototype:** `mwdb/local-first/src/lib.rs`

Contract:

1. local reads do not require a network;
2. local writes are accepted while offline;
3. logical changes are journaled durably before `set()` returns success;
4. local state is reconstructible from the change journal;
5. restart replays the journal to repair/rebuild the state snapshot;
6. change IDs are stable and duplicate enqueue attempts are ignored;
7. acknowledgement state is persisted in the same journal.

Status: **DONE for prototype contract**

## M2-B — Durable pending-change queue

Implemented:

- append-only `changes.log`
- `Pending | Acknowledged | Rejected | Conflicted`
- UUID change IDs
- duplicate suppression
- durable status records
- crash/restart recovery

Status: **DONE for prototype**

## M2-C — Subscriptions/reconnect

Not implemented yet. The queue is intentionally transport-neutral.

Next implementation should define:

- local change notification
- remote-originated notification
- deduplication
- ordering guarantees
- resubscription after reconnect

Status: **PLANNED**

## M2-D — Deterministic offline harness

Prototype coverage currently includes restart, offline persistence, duplicate suppression, and acknowledgement recovery tests.

Remaining:

- deterministic network-off toggle
- reconnect simulation
- retry schedule
- transport fault injection
- multi-replica scenario fixtures

Status: **IN PROGRESS**

## Architecture decision

The logical change log is an MW-DB primitive. Physical WAL records inside the upstream engine remain an implementation detail and are not exposed as the sync protocol.

This keeps the later M3/M4 protocol independent of the current SurrealDB storage substrate.

## Exit gate

M2 is complete only when an end-to-end fixture can:

```text
open local
  -> write offline
  -> restart
  -> recover
  -> reconnect
  -> publish pending changes
  -> receive acknowledgement
  -> show deterministic subscription behavior
```

and all relevant failure/recovery tests pass.
