# MW-DB Architecture

Status: architecture v0.2.

## 1. Architectural thesis

MW-DB is a product/data-platform architecture first and an engine fork second. The current repository provides a mature Rust/SurrealDB substrate. MW-DB should add new capabilities at replaceable boundaries so the substrate can be changed later without rewriting application-facing contracts.

The core lifecycle is:

```text
LOCAL -> SYNC -> VERSION -> BRANCH -> VERIFY -> FEDERATE -> DECENTRALIZE
```

## 2. Layered architecture

```text
+-------------------------------------------------------------+
|                     Applications                            |
|  Radicle / MW products / Web / Mobile / AI agents          |
+-----------------------------+-------------------------------+
                              |
                              v
+-------------------------------------------------------------+
|                         MW-DB API                           |
| query | write | subscribe | branch | sync | prove | merge  |
+-----------------------------+-------------------------------+
                              |
              +---------------+---------------+
              |                               |
              v                               v
+-------------------------+       +-------------------------+
| Query / Model Plane     |       | Identity / Policy Plane |
| SurrealQL initially     |       | actor | auth | ACL      |
| relational/document     |       | trust mode | policy    |
| graph/vector access     |       +-------------------------+
+------------+------------+
             |
             v
+-------------------------------------------------------------+
|                       State Plane                           |
| MVCC | transactions | indexes | subscriptions | cache       |
+---------------------------+---------------------------------+
                            |
             +--------------+--------------+
             |                             |
             v                             v
+-------------------------+      +----------------------------+
| Local-First Plane       |      | Logical Change Plane       |
| durable local state     |      | change IDs                 |
| offline writes          |      | actor/causal metadata     |
| pending queue           |      | hashes/signatures         |
+-------------+-----------+      +-------------+--------------+
              |                                |
              +---------------+----------------+
                              v
                    +---------------------+
                    |     Sync Plane      |
                    | push/pull/resume    |
                    | idempotency         |
                    | conflict handling   |
                    +----------+----------+
                               |
              +----------------+----------------+
              |                                 |
              v                                 v
   +---------------------+           +----------------------+
   | Branch/Version     |           | Verification Plane   |
   | snapshots          |           | hashes               |
   | branches           |           | Merkle roots         |
   | diff/merge         |           | proofs               |
   | rollback           |           | optional anchoring   |
   +---------------------+           +----------------------+
              |
              +----------------+----------------+
                               v
                    +---------------------+
                    | Network / Federation|
                    | peers | topology     |
                    | replication | BFT    |
                    +----------+----------+
                               |
                               v
                    +---------------------+
                    | Storage Substrate   |
                    | existing engines    |
                    | local / object / KV |
                    +---------------------+
```

## 3. Planes and ownership

### 3.1 Query/model plane

Purpose: application data access. Keep SurrealQL and existing SDKs initially. MW-DB additions should first be exposed as typed APIs/protocol operations rather than a new query grammar.

### 3.2 State plane

Purpose: transactional local state. Reuse the upstream engine capabilities. Do not duplicate MVCC or indexing prematurely.

### 3.3 Logical change plane

Purpose: stable cross-node semantics. This is distinct from physical WAL. A logical change is an application-level replication object.

Minimum envelope:

```text
change_id
object/table identity
operation
payload or delta
actor
causal metadata
created_at
parents/change refs
content_hash
signature (when trust mode requires it)
```

### 3.4 Local-first plane

The local DB is a durable execution target, not a cache.

Required guarantees:
1. local read without network;
2. permitted offline writes;
3. durable persistence before acknowledgement;
4. restart-safe pending state;
5. deterministic local subscriptions.

### 3.5 Sync plane

Responsibilities:
- peer/session authentication,
- capability negotiation,
- checkpoints/cursors,
- push/pull,
- resumability,
- idempotency,
- acknowledgement,
- backpressure,
- causal ordering,
- conflict reporting.

### 3.6 Branch/version plane

Represents named logical database states. Branching is intended for users, tests, previews, and AI agents.

Required operations:
`create`, `snapshot`, `diff`, `restore`, `merge`, `reject`, `delete`.

### 3.7 Verification plane

Provides independent integrity checks. Cryptographic proofs are not required for every deployment.

Progression:

```text
hash -> Merkle root -> inclusion proof -> signed state -> optional external anchor
```

### 3.8 Network/federation plane

Later-stage protocol for independent operators. Network transport must remain separate from state semantics so local testing can exercise the same logical protocol without a live WAN.

## 4. Trust modes

```text
LOCAL
  One local authority. No network requirement.

REPLICATED
  A configured set of replicas share state under an operational coordinator.

FEDERATED
  Independent operators exchange authenticated/signed changes.

DECENTRALIZED
  No single operator is assumed authoritative; verification and coordination are protocol-level concerns.
```

Trust mode must not alter application data APIs.

## 5. Consistency model

MW-DB should not force one global consistency model.

```text
mergeable -> CRDT / deterministic merge
causal    -> causal ordering
critical  -> serializable/coordinated transaction
multi-org -> federation policy
```

The schema/data policy identifies the required semantics. The engine selects the minimum coordination necessary to preserve invariants.

## 6. Storage strategy

Initial MW-DB releases reuse existing storage engines. Storage remains an adapter boundary.

Potential backends:
- embedded/local filesystem,
- SSD/NVMe,
- distributed KV,
- object storage,
- content-addressed storage.

No application API may depend on a specific backend.

## 7. Query and ORM strategy

### Initial
- SurrealQL remains the query language.
- Existing SDKs remain valid.
- New capabilities are APIs/protocols first.

### Target
A canonical MW schema can generate typed clients for TypeScript, Rust, Go, and Python. Generated clients must preserve transaction, local-first, sync, conflict, and branch semantics.

### Deferred
A distinct MWQL syntax is deferred until real workloads prove that current query primitives cannot express a capability cleanly.

## 8. Radicle integration boundary

Radicle is an integration laboratory, not a dependency of the generic database core.

```text
Radicle repo state
      |
      v
  local MW-DB
      |
      v
 logical changes
      |
      v
   peer sync
      |
      v
 merge / review
      |
      v
 verification
```

## 9. Evidence gates

Every new subsystem must have:

- a written contract;
- deterministic tests;
- failure/recovery tests where applicable;
- performance baseline;
- interoperability test;
- migration/compatibility story.

No decentralized feature is considered complete merely because a demo works.

## 10. Architectural invariants

1. Local reads do not require a network.
2. Offline writes are durable before acknowledgement.
3. Logical changes have stable identifiers.
4. Sync is incremental, resumable, and idempotent.
5. Conflicts are explicit and policy-driven.
6. Verification can operate independently of the transport node.
7. Public MW-DB contracts do not permanently expose one storage implementation.
8. Upstream attribution/license obligations remain intact.
9. Radicle-specific concerns stay out of the generic core.
10. Native engine replacement is an evidence-based milestone, not an assumption.