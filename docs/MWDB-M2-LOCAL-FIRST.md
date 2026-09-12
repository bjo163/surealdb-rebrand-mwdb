# M2 Local-First Implementation

Status: **IN PROGRESS**

This document tracks the first executable MW-DB differentiation slice.

## M2-A — Local-first contract

Implemented in `mwdb/local-first/src/lib.rs`.

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

## M2-C — Subscriptions / reconnect

Implemented for the prototype:

- local change notification
- ordered local subscription delivery
- subscriber cleanup after disconnect
- reopen/reconnect fixture that recovers pending changes

Not yet implemented:

- remote-originated notification
- transport-level deduplication
- explicit remote ordering contract
- network reconnect protocol

Status: **DONE for local prototype; PLANNED for transport integration**

## M2-D — Deterministic offline / recovery harness

Current prototype coverage:

- offline write persistence
- restart recovery
- acknowledgement persistence
- duplicate suppression
- journal replay
- local subscription ordering
- reconnect fixture

The repository also contains `.github/workflows/mwdb-local-first.yml` to verify this isolated crate.

Remaining:

- deterministic network fault injection
- reconnect retry schedule
- multi-replica fixtures
- CI result capture and regression gate

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

and all relevant failure/recovery tests pass in CI.
