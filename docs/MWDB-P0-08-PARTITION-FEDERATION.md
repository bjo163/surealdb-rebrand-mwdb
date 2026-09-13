# MW-DB P0-08 / GAP-12 — Partition & Federation Convergence Suite

Status: RELEASE BLOCKER
Priority: P0

Production release remains blocked until this suite is reproducibly green in CI.

## Goal

Prove that MW-DB replicas converge deterministically through partition, restart, replay, and federation/rejoin scenarios using the real local-first, sync, cursor, and persistent auth/replay primitives rather than isolated unit mocks.

## Required fixture

Create a deterministic three-replica fixture: `node-a`, `node-b`, and `node-c`. Each replica must use persistent local storage, peer cursors, and replay state.

### Scenario A — Partition + concurrent writes

1. Start A/B/C from empty state.
2. Establish baseline sync.
3. Partition B from A/C.
4. Write different documents or values on both sides of the partition.
5. Sync A <-> C while B remains isolated.
6. Heal the partition and exchange batches in both directions.
7. Repeat delivery of at least one previously accepted batch.
8. Assert every replica converges to the same deterministic logical state and Merkle root; duplicate delivery must be idempotent or rejected according to protocol semantics.

### Scenario B — Restart during partition

1. Persist cursors and replay windows.
2. Restart one isolated replica.
3. Reopen local store, peer cursor, and replay store from disk.
4. Heal the partition and resume sync.
5. Assert no accepted change is lost and no already-accepted authenticated frame is accepted twice.

### Scenario C — Federation / rejoin boundary

1. Treat two replica groups as temporarily independent peers.
2. Exchange changes only after explicit rejoin.
3. Reject stale cursor, replay, and key-epoch state.
4. Assert deterministic convergence after rejoin.

## Acceptance criteria

- Real `mwdb-local-first`, `mwdb-sync`, `mwdb-cursor`, and persistent auth/replay primitives are exercised.
- Minimum three replicas.
- Partition/heal schedule is deterministic; no wall-clock sleeps.
- Concurrent writes are covered.
- Duplicate/replayed delivery is covered.
- Restart with persisted cursor and replay state is covered.
- Stale key epoch on rejoin is covered.
- Final state/root equality is asserted across every replica.
- Fixture runs in the release-gate workflow on pull requests and `main`.
- Failures emit replica, cursor, batch/change, replay, and root diagnostics sufficient to reproduce locally.
- `docs/MWDB-AUDIT-GAP-TASKS.md` and `docs/MWDB-RELEASE-GATE.json` are updated to green only after CI evidence succeeds.

## Non-goals

This task does not need to solve production networking, service discovery, or WAN transport. It proves deterministic engine-level partition/federation semantics.

## Release rule

Do not mark P0-08 green from code presence alone. Green requires this suite to pass in CI with reproducible evidence.
