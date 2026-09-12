# MW-DB Architecture

## Boundary model

MW-DB is a product architecture first and an engine fork second. The current codebase provides a mature Rust database substrate. New MW-DB functionality should be introduced at explicit boundaries so the substrate can be replaced later if evidence requires it.

```text
Clients / Apps
      |
      v
+---------------------+
| MW-DB API / SDK     |
+----------+----------+
           |
   +-------+--------+
   | Query / Model  |
   | Subscription   |
   +-------+--------+
           |
   +-------+------------------------------+
   |                                      |
   v                                      v
Local State                         Change/Event Log
   |                                      |
   +----------------+---------------------+
                    |
                    v
               Sync Engine
                    |
          +---------+---------+
          |                   |
       Remote              Peer/Federation
          |                   |
          +---------+---------+
                    |
               Verification
                    |
              Storage Engine
                    |
             Storage Backends
```

## Components

### 1. Data API

Stable application-facing API for reads, writes, queries, subscriptions, snapshots, branches, synchronization, and proofs.

### 2. Local state

The local database is not treated as a cache. It is a durable execution target that must support useful work without a network connection.

### 3. Logical change log

Separate logical changes from physical WAL records. Physical WAL remains an engine implementation detail; MW-DB changes need stable identity and replication semantics.

Minimum logical change fields:

```text
change_id
object/table identity
operation
payload or delta
actor
causal metadata
created_at
parent/change references
content hash
signature (when trust mode requires it)
```

### 4. Sync engine

Responsibilities include discovery, push/pull, resumability, idempotency, backpressure, acknowledgement, causal ordering, and conflict handling.

### 5. Conflict engine

Conflict policy is explicit. Mergeable state may use CRDT techniques; invariant-sensitive transactions must use stronger coordination.

### 6. Version/branch engine

Snapshots and branches are logical database states. They enable experimentation, review, rollback, and agent sandboxes.

### 7. Verification engine

Supports hashes, Merkle state, signed changes, inclusion proofs, and optional external anchoring.

### 8. Network/federation

A later layer for authenticated peers, independent operators, topology management, and consensus/coordination.

## Trust modes

```text
LOCAL
  single authority; no network dependency

REPLICATED
  configured replicas; operationally coordinated

FEDERATED
  independent operators; signed changes and explicit trust

DECENTRALIZED
  no assumed single authority; consensus/verification are protocol concerns
```

## Storage strategy

The initial release should use existing storage engines. Possible future adapters include local files, SSD/NVMe, object storage, and content-addressed storage. Storage choice must not leak into the application API.

## Query strategy

Keep SurrealQL as the initial query language. Add new capability through APIs/protocols first. A new query dialect is deferred until there is a demonstrated need for syntax that cannot be expressed cleanly by current primitives.

## ORM/SDK strategy

Use the existing SDK ecosystem initially. Long term, define an MW schema representation that can generate typed clients for TypeScript, Rust, Go, and Python. Generated clients must preserve the same local-first/sync semantics.

## Radicle integration target

Radicle is an early integration laboratory:

```text
repository state
      -> local MW-DB
      -> logical changes
      -> peer sync
      -> merge/review
      -> verification
```

Radicle-specific logic must remain an integration consumer, not become part of the generic database core.

## Architectural invariants

1. Local reads do not require a network.
2. Offline writes are durable before acknowledgement.
3. Logical changes have stable identifiers.
4. Synchronization is incremental and resumable.
5. Conflicts are observable and policy-driven.
6. Verification does not depend on the transport node being trusted.
7. Public MW-DB contracts do not expose an irreversible dependency on one storage backend.
8. Upstream license/attribution notices remain intact.
