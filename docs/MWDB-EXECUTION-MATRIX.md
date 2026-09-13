# MW-DB Execution Matrix

This is the working dashboard for determining where the project is now and what counts as done.

## Current repository baseline

- Repository: `bjo163/surealdb-rebrand-mwdb`
- Branch: `main`
- Substrate: existing SurrealDB Rust workspace
- MW-DB feature substrate: `mwdb/local-first` + `mwdb/sync` + `mwdb/replay` + `mwdb/cursor` + `mwdb/observability` + `mwdb/auth` + `mwdb/canonical` + `mwdb/signing` + `mwdb/merkle` + `mwdb/identity` + `mwdb/capabilities`
- Current posture: upstream-derived research/product fork with a broad distributed/verifiable prototype substrate. Production readiness is not claimed.
- GitHub Issues: disabled; `docs/MWDB-ISSUE-PLAN.md` and `docs/MWDB-AUDIT-GAP-TASKS.md` are the authoritative issue-ready backlog.

## Overall status

```text
M0  Baseline / provenance       NEXT
M1  Branding / packaging       NEXT
M2  Local-first                DONE (prototype)
M3  Logical changes            IN PROGRESS
M4  Sync / conflicts            IN PROGRESS
M5  Branch / time travel       PLANNED
M6  Verification                IN PROGRESS
M7  Federation / distribution   IN PROGRESS (prototype foundations)
M8  Adaptive trust              PLANNED
M9  Agent-native                PLANNED
M10 Native-engine decision      GATE
```

## Detailed matrix

| ID | Area | Work | Status | Output | Gate |
|---|---|---|---|---|---|
| M0-A | Baseline | reproducible build/test/toolchain record | NEXT | baseline report | reproducible |
| M0-B | Legal | license/attribution inventory | NEXT | provenance map | reviewed |
| M0-C | Architecture | upstream/MW-DB ownership map | NEXT | ownership contract | reviewed |
| M1-A | Brand | MW-DB terminology and README identity | DONE (prototype) | brand/README baseline | coherent |
| M1-B | Packaging | crate/binary/SDK naming map | NEXT | package map | coherent |
| M1-C | Compatibility | upstream compatibility policy | NEXT | compatibility matrix | explicit |
| M2-A | Local | local-first contract | DONE (prototype) | local-first behavior | tested |
| M2-B | Local | durable pending-change queue | DONE (prototype) | append-only journal + status | crash-safe |
| M2-C | Local | local subscriptions/reconnect semantics | DONE (prototype) | ordered subscriptions + remote apply | deterministic |
| M2-D | Local | offline/recovery harness | DONE (prototype) | restart/replay/retry/fault recovery fixtures | repeatable |
| M3-A | Events | logical change envelope | DONE (prototype) | `LogicalChangeV1` + hash | versioned |
| M3-B | Events | deterministic replay | PARTIAL (prototype) | startup replay + standalone replay crate | reproducible |
| M3-C | Events | change feed/export/import | PARTIAL (prototype) | JSONL export/import + checkpoint delta | resumable |
| M3-D | Events | persistent peer cursor | DONE (prototype) | `mwdb/cursor` | restart-safe |
| M3-E | Events | canonical cross-language encoding | PARTIAL (prototype) | canonical boundary + vectors | cross-implementation |
| M4-A | Sync | push/pull/checkpoint/resume | DONE (prototype) | sync v1 primitives + convergence fixture | convergent |
| M4-B | Conflicts | conflict taxonomy | DONE (prototype) | classifier + conservative policy | explicit |
| M4-C | CRDT | mergeable-structure prototypes | DONE (prototype) | G-Counter + algebraic tests | deterministic |
| M4-D | Resilience | drop/duplicate/reorder fault harness | DONE (prototype) | transport-neutral fault model | repeatable |
| M4-E | Observability | sync state metrics | DONE (prototype) | semantic events + counters | measurable |
| M4-F | Transport | authenticated shared-key transport | DONE (prototype) | authenticated frame adapter | integrity |
| M5-A | Versioning | snapshots | PLANNED | API/storage mapping | restorable |
| M5-B | Versioning | named branches | PLANNED | API | isolated |
| M5-C | Versioning | diff | PLANNED | state/logical diff | accurate |
| M5-D | Versioning | merge/reject/rollback | PLANNED | workflow | safe |
| M6-A | Proof | hashes/content identity | DONE (prototype) | content hash substrate | deterministic |
| M6-B | Proof | Merkle state root | DONE (prototype) | deterministic ordered root | verifiable |
| M6-C | Proof | signed changes | DONE (prototype) | signed-change envelope | authentic |
| M6-D | Proof | inclusion/state proofs | DONE (prototype) | inclusion proof + verifier | independently verifiable |
| M7-A | Network | peer identity and capabilities | PARTIAL (prototype) | identity + capability primitives | authenticated |
| M7-B | Network | replication topology | PLANNED | topology model | tested |
| M7-C | Network | peer sync | PARTIAL (prototype) | auth frame boundary + sync primitives | resilient |
| M7-D | Network | failure/partition tests | PLANNED | partition/chaos suite | passes |
| M8-A | Trust | consistency policy model | PLANNED | schema/data policy | explicit |
| M8-B | Trust | adaptive enforcement | PLANNED | planner/runtime hooks | correct |
| M8-C | Trust | semantic consistency tests | PLANNED | test matrix | passing |
| M9-A | Agents | typed agent API | PLANNED | SDK/API | scoped |
| M9-B | Agents | agent permissions/sandbox | PLANNED | policy layer | isolated |
| M9-C | Agents | reviewable change-set format | PLANNED | replayable proposal | reviewable |
| M10-A | Engine | comparative benchmark suite | GATE | evidence report | reproducible |
| M10-B | Engine | upstream/native decision | GATE | ADR | approved |

## Current implemented slice

```text
LocalFirstStore
  ├─ state snapshot + durable logical journal
  ├─ change status / duplicate suppression / replay
  ├─ ordered subscriptions / verified remote apply
  └─ checkpoint delta selection

Proof + identity substrate
  ├─ canonical JSON prototype
  ├─ signed-change prototype
  ├─ Merkle state root
  ├─ Merkle inclusion proof + verifier
  ├─ public-key identity prototype
  ├─ replay-window checks
  └─ capability intersection

Sync substrate
  ├─ SyncHello / SyncBatch / SyncAck
  ├─ hash verification + idempotent apply
  ├─ persistent acknowledgement
  ├─ conflict classifier / G-Counter lab
  ├─ fault harness + convergence fixtures
  ├─ persistent peer cursor
  ├─ sync metrics
  └─ authenticated frame boundary
```

## Verification status

Focused prototype tests cover local durability, restart recovery, logical-hash verification, checkpoint deltas, replay/import, cursor persistence, sync convergence/retry/fault scenarios, conflict classification, CRDT algebraic properties, sync metrics, authenticated frames, canonical ordering, signed-change integrity, Merkle roots/proofs, identity checks, and capability intersection.

CI is the source of truth for repository-wide verification. A prototype test passing is not a production-readiness claim.

## Release gates

A production federation release requires, at minimum:

1. canonical cross-language vectors including numeric normalization;
2. sync/cursor/observability integration;
3. persistent replay protection and key lifecycle;
4. public-key identity enforcement in sync;
5. signed-change/Merkle integration with serializable proofs;
6. real transport adapter and partition/failure suite;
7. arbitrary logical-operation replay;
8. provenance, compatibility, and benchmark evidence.

Do not start a native storage-engine rewrite or blockchain/public-P2P work before these gates are supported by reproducible evidence.

## Next execution target

```text
P0 integration pipeline
    ↓
canonical interoperability freeze
    ↓
identity + replay/key lifecycle
    ↓
signed/Merkle/proof integration
    ↓
network + partition suite
    ↓
benchmark / provenance gate
    ↓
M5 branch + time travel
```
