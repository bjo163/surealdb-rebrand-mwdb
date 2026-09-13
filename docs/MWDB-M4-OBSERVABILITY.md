# MW-DB M4 — Sync Observability v0

Status: PROTOTYPE / IN PROGRESS

## Goal

Define a transport-neutral observation contract for sync behavior without coupling MW-DB to one telemetry backend.

## Event contract

`mwdb-observability` defines:

- BatchSent { changes, bytes }
- BatchReceived { changes, bytes }
- ChangeApplied
- DuplicateIgnored
- ChangeRejected
- Retry
- Conflict
- QueueDepth { depth }

These events are semantic sync events. Future adapters may export them to logs, OpenTelemetry, Prometheus, or another backend.

## Counters

`SyncMetrics` tracks batch counts, change results, retries, conflicts, bytes, and current queue depth. Independent snapshots can be merged.

## Acceptance criteria

1. Every event updates deterministic counters.
2. Metrics serialize through serde.
3. Counter snapshots merge without losing totals.
4. The crate remains outside the root workspace.
5. CI runs cargo test and cargo check for the crate.

## Boundary

Next work integrates event emission into sync operations, defines peer identity/authentication, and verifies metrics during retry, reject, conflict, and partition scenarios.
