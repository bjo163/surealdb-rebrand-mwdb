# MW-DB Documentation

## Core

- [Blueprint](./MWDB-BLUEPRINT.md) — product thesis, architecture layers, evolution path, local-first contract, trust model.
- [Milestones](./MWDB-MILESTONES.md) — M0-M10 roadmap and exit criteria.
- [Architecture](./MWDB-ARCHITECTURE.md) — boundaries for data API, local state, changes, sync, verification, federation, and storage.
- [Issue Plan](./MWDB-ISSUE-PLAN.md) — implementation backlog prepared as GitHub issues; currently mirrored here because Issues are disabled.
- [ADR-0001](./MWDB-ADR-0001-architecture-strategy.md) — decision to extend/reuse mature database primitives before building a native engine.

## Current strategy

1. Keep SurrealDB as the initial technical substrate.
2. Make MW-DB differentiation happen at the local-first, sync, event/version, verification, branch, and federation layers.
3. Keep query language and existing SDKs initially; avoid premature MWQL/ORM rewrites.
4. Defer native-engine replacement until evidence supports it.

## Next execution target

Start with M0-01 and M1-01, then parallelize M2 local-first work once the contracts are frozen.
