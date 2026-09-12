# MW-DB Execution Matrix

This is the working dashboard for determining **where the project is now** and **what counts as done**.

## Current repository baseline

- Repository: `bjo163/surealdb-rebrand-mwdb`
- Branch: `main`
- Substrate: existing SurrealDB Rust workspace
- MW-DB feature substrate: `mwdb/local-first` + `mwdb/sync`
- Current posture: upstream-derived research/product fork with local-first, logical-change, checkpoint-sync, and deterministic two-replica prototypes implemented.
- GitHub Issues: disabled; executable backlog remains in `docs/MWDB-ISSUE-PLAN.md`.

## Overall status

```text
M0  Baseline / provenance       NEXT
M1  Branding / packaging       NEXT
M2  Local-first                IN PROGRESS
M3  Logical changes            IN PROGRESS
M4  Sync / conflicts            IN PROGRESS ← current feature gate
M5  Branch / time travel       PLANNED
M6  Verification                PLANNED
M7  Federation / distribution   PLANNED
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
| M2-A | Local | local-first contract | DONE (prototype) | `mwdb/local-first` contract | tested |
| M2-B | Local | durable pending-change queue | DONE (prototype) | append-only journal + status | crash-safe |
| M2-C | Local | local subscriptions/reconnect semantics | DONE (prototype) | ordered local subscriptions + remote apply | deterministic |
| M2-D | Local | offline/recovery harness | IN PROGRESS | restart/replay/reconnect fixtures + CI | repeatable |
| M3-A | Events | logical change envelope | DONE (prototype) | `LogicalChangeV1` + deterministic hash | versioned |
| M3-B | Events | deterministic replay | PARTIAL (prototype) | startup journal replay | reproducible |
| M3-C | Events | change feed/export/import | PARTIAL (prototype) | export + checkpoint delta selection | resumable |
| M4-A | Sync | push/pull/checkpoint/resume | DONE (prototype) | sync v0 + two-replica fixture | convergent |
| M4-B | Conflicts | conflict taxonomy | PARTIAL (prototype) | conservative classifier | explicit |
| M4-C | CRDT | mergeable-structure prototypes | PLANNED | test suite | deterministic |
| M5-A | Versioning | snapshots | PLANNED | API/storage mapping | restorable |
| M5-B | Versioning | named branches | PLANNED | API | isolated |
| M5-C | Versioning | diff | PLANNED | change/state diff | accurate |
| M5-D | Versioning | merge/reject/rollback | PLANNED | workflow | safe |
| M6-A | Proof | hashes/content identity | BASELINE (prototype) | BLAKE3 content hash | deterministic |
| M6-B | Proof | Merkle state | PLANNED | root/proof | verifiable |
| M6-C | Proof | signed changes | PLANNED | signature envelope | authentic |
| M6-D | Proof | inclusion/state proofs | PLANNED | verifier | independent |
| M7-A | Network | peer identity | PLANNED | protocol | authenticated |
| M7-B | Network | replication topology | PLANNED | topology model | tested |
| M7-C | Network | peer sync | PLANNED | transport integration | resilient |
| M7-D | Network | failure/partition tests | PLANNED | chaos/fault suite | passes |
| M8-A | Trust | consistency policy model | PLANNED | schema/data policy | explicit |
| M8-B | Trust | adaptive enforcement | PLANNED | planner/runtime hooks | correct |
| M8-C | Trust | semantic consistency tests | PLANNED | test matrix | passing |
| M9-A | Agents | typed agent API | PLANNED | SDK/API | scoped |
| M9-B | Agents | sandbox/permissions | PLANNED | policy layer | isolated |
| M9-C | Agents | reviewable change sets | PLANNED | format/UI contract | replayable |
| M10-A | Engine | benchmark comparison | GATE | report | evidence |
| M10-B | Engine | upstream/native decision | GATE | ADR | approved |

## Current implemented slice

```text
LocalFirstStore
      |
      +-- state.json (rebuildable snapshot)
      |
      +-- changes.log (durable logical journal)
              |
              +-- ChangeEnvelope v0
              +-- LogicalChangeV1
              +-- checkpoint delta selection
              +-- status records
              +-- duplicate suppression
              +-- startup replay
              +-- ordered subscriptions
              +-- verified remote apply

mwdb/sync
      |
      +-- SyncHello / SyncBatch / SyncAck
      +-- hash verification
      +-- idempotent batch apply
      +-- persistent acknowledgement
      +-- conflict classification
      +-- two-replica convergence fixture
```

The journal is written and synced before `set()` returns. Startup replays logical changes so a crash between journal and snapshot persistence can be recovered.

## Verification status

The current prototypes have focused tests covering offline writes, restart persistence, acknowledgement persistence, duplicate suppression, journal replay, ordered subscriptions, deterministic hashing, checkpoint deltas, verified remote apply, two-replica convergence, retry idempotency, and conservative conflict classification.

Dedicated GitHub Actions workflows exist for the local-first and sync crates. CI should be treated as the verification source of truth; no production-readiness claim is made from unit tests alone.

## Next execution target

```text
M2-D  deterministic fault/recovery harness
          ↓
M3-B/C  independent replay + import/resume
          ↓
M4-B/C  conflict policy + CRDT experiments
          ↓
M4 observability / transport adapter
          ↓
M7 authenticated federation
```

Do not start blockchain, public P2P, or a native storage rewrite before the local-first, logical-change, and sync contracts are stable and supported by benchmarks.
