# ADR-0001: Build MW-DB as an evolution layer before a new database engine

Status: Accepted for the research phase

## Context

The repository contains a substantial Rust database implementation derived from SurrealDB. Rebuilding storage, transactions, indexing, query execution, and transport from zero would delay the features that are intended to differentiate MW-DB: local-first state, sync, versioning, verification, federation, and decentralized trust.

## Decision

For milestones M0-M9, reuse mature upstream database capabilities wherever they satisfy requirements. Introduce MW-DB functionality behind explicit modules/contracts. Do not start a native storage-engine rewrite until milestone M10 has benchmark and operational evidence.

The initial query language remains SurrealQL. New functionality should be exposed through APIs/protocols first.

## Consequences

Positive:
- Faster usable prototypes.
- Lower early correctness risk in core database mechanics.
- Real workload evidence can determine where a native engine is justified.
- Application-facing MW-DB contracts can remain stable across future engine changes.

Negative:
- Upstream architecture constrains some early choices.
- Upstream updates must be tracked and merged carefully.
- Licensing and redistribution obligations remain material.

## Revisit conditions

Reconsider this decision when one or more of the following is demonstrated:

- a required workload cannot be implemented without invasive upstream changes;
- licensing/commercial constraints make the current substrate unsuitable;
- benchmarks show a stable MW-DB-specific engine design can materially outperform the current path on target workloads;
- local-first/sync semantics require storage primitives that the current engine cannot provide efficiently.
