# MW-DB Blueprint

Status: research/engineering blueprint, not a production compatibility promise.

## Mission

MW-DB is the working name for a data system that can evolve from embedded local-first use into cloud, replicated, federated, and eventually decentralized deployments without forcing application rewrites.

The project starts from the existing SurrealDB Rust codebase to avoid rebuilding mature query, storage, indexing, protocol, and runtime primitives from zero. MW-DB differentiation is built above and around those primitives first.

## Core thesis

> Build locally. Sync incrementally. Version everything. Verify what matters. Distribute when needed.

The architecture must preserve one application-facing data model while allowing multiple execution and trust modes:

`local -> replicated -> federated -> decentralized`

## Non-goals for the first releases

- Do not rewrite the storage engine prematurely.
- Do not invent a new query language before existing query capabilities are understood.
- Do not put every record on a blockchain.
- Do not force strong global consensus on data that can safely merge.
- Do not promise Supabase/Neon/Mongo/S3 feature parity before measurable requirements exist.
- Do not remove upstream attribution or license notices.

## Architecture layers

```text
Application / SDKs
        |
        v
MW-DB Data API
        |
        +-------------------------------+
        | Query / Model / Subscription   |
        +-------------------------------+
        | Local-first state              |
        | Event / change log             |
        | Sync / conflict policy         |
        | Branch / snapshot              |
        | Verification / provenance      |
        +-------------------------------+
        | Existing database engine(s)    |
        +-------------------------------+
        | Storage backends               |
        +-------------------------------+
```

## Evolution path

### M0 — Provenance and governance

Establish upstream mapping, license boundaries, branding boundaries, contribution policy, and a reproducible baseline.

### M1 — Rebrand-safe foundation

Define MW-DB naming surfaces, package boundaries, user-facing identity, and upstream compatibility policy without mass-renaming code that creates unnecessary churn.

### M2 — Local-first substrate

Define durable local state, deterministic change records, offline writes, reconnect semantics, and incremental synchronization contracts.

### M3 — Event and version model

Introduce a logical change/event model independent of the physical WAL implementation. Events need stable identifiers, causal metadata, hashes, actor identity, and replay semantics.

### M4 — Sync and conflict resolution

Add pull/push synchronization, idempotency, retry, causal ordering, and explicit conflict policies. CRDTs are used only for structures where merge semantics are well-defined.

### M5 — Branching and time travel

Add cheap snapshots/branches, compare/diff, rollback, and agent/user experimental branches.

### M6 — Verification and provenance

Add content hashes, Merkle structures, signatures, proof generation, and optional anchoring. Blockchain is an optional verification target, not the primary database.

### M7 — Federation and distributed execution

Add peer discovery, authenticated replication, topology-aware placement, shard/replica policy, and pluggable consensus where required.

### M8 — Adaptive trust/consistency

Allow table/object policies to select local merge, causal consistency, serializable coordination, or BFT-style coordination according to semantics.

### M9 — Agent-native data plane

Expose typed operations for AI agents: branch, inspect history, propose changes, test, merge, subscribe, and prove.

### M10 — Native engine evaluation

Only after benchmarks and operational evidence, decide which SurrealDB components should remain upstream dependencies and which MW-DB components justify a native implementation.

## Query and ORM strategy

Phase 1 keeps SurrealQL and existing SDKs as the compatibility surface. MW-DB extensions should prefer APIs/protocols before syntax extensions.

Long-term targets:

- SQL/document/graph/vector access through one coherent data model.
- A typed schema source that can generate SDK/ORM types.
- Local-first transactions and subscriptions in the client SDK.
- Branch/sync/prove/merge operations exposed as first-class APIs.

A future `MWQL`/EVO-style query layer is explicitly deferred until extension requirements are demonstrated by real workloads.

## Local-first contract

A conforming MW-DB client should:

1. Read local committed state without network access.
2. Perform permitted local writes while offline.
3. Persist writes durably before acknowledging them.
4. Record logical changes with stable IDs.
5. Resume synchronization after reconnect without duplicating effects.
6. Expose conflict outcomes rather than silently overwriting data.
7. Permit local verification of received state.

## Trust model

MW-DB supports progressive trust:

- `local`: single process/device is authoritative.
- `replicated`: a configured set of replicas share state.
- `federated`: independent operators exchange signed changes.
- `decentralized`: no single operator is assumed authoritative; consensus/verification is explicit.

## Success metrics

Every milestone should produce measurable evidence for:

- offline write success rate
- sync convergence time
- duplicate/replay safety
- conflict detection/merge correctness
- read/write latency
- recovery time after crash
- storage amplification
- bandwidth per synchronized change
- proof generation/verification cost
- interoperability with existing MW products

## Dependency strategy

SurrealDB is the initial upstream technical base. MW-DB must keep a clear boundary between reused upstream capabilities and original MW-DB work. A future engine replacement must be possible without changing the public MW-DB data/sync contracts.

## Commercial/legal guardrail

The current repository contains upstream SurrealDB licensing and attribution material. Any managed database service or commercial redistribution strategy must be reviewed against the exact license terms of each upstream version before launch. Branding changes do not change copyright or license obligations.
