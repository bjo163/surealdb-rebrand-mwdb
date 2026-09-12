# MW-DB Execution Matrix

This is the working dashboard for determining **where the project is now** and **what counts as done**.

## Current repository baseline

- Repository: `bjo163/surealdb-rebrand-mwdb`
- Branch: `main`
- Substrate: existing SurrealDB Rust workspace
- MW-DB feature substrate: `mwdb/local-first`
- Current posture: upstream-derived research/product fork with the first local-first and logical-change prototypes implemented.
- GitHub Issues: disabled; executable backlog remains in `docs/MWDB-ISSUE-PLAN.md`.

## Overall status

```text
M0  Baseline / provenance       NEXT
M1  Branding / packaging       NEXT
M2  Local-first                IN PROGRESS ← current feature gate
M3  Logical changes            IN PROGRESS
M4  Sync / conflicts            PLANNED
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
| M1-A | Brand | MW-DB terminology and README identity | NEXT | brand/spec | coherent |
| M1-B | Packaging | crate/binary/SDK naming map | NEXT | package map | coherent |
| M1-C | Compatibility | upstream compatibility policy | NEXT | compatibility matrix | explicit |
| M2-A | Local | local-first contract | DONE (prototype) | `mwdb/local-first` contract | tested |
| M2-B | Local | durable pending-change queue | DONE (prototype) | append-only journal + status | crash-safe |
| M2-C | Local | local subscriptions/reconnect semantics | DONE (prototype) | ordered local subscriptions + resubscribe fixture | deterministic |
| M2-D | Local | offline/recovery harness | IN PROGRESS | restart/replay fixtures + CI workflow | repeatable |
| M3-A | Events | logical change envelope | DONE (prototype) | `LogicalChangeV1` + deterministic hash | versioned |
| M3-B | Events | deterministic replay | PARTIAL (prototype) | startup journal replay | reproducible |
| M3-C | Events | change feed/export/import | PLANNED | API | stable |
| M4-A | Sync | push/pull/checkpoint/resume | PLANNED | sync v0 | convergent |
| M4-B | Conflicts | conflict taxonomy | PLANNED | policy spec | explicit |
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
              +-- status records
              +-- duplicate suppression
              +-- startup replay
              +-- ordered local subscriptions
```

The journal is written and synced before `set()` returns. Startup replays logical changes so a crash between journal and snapshot persistence can be recovered.

## Verification status

The prototype has six unit tests covering:

- offline local write
- restart persistence
- acknowledgement persistence
- duplicate change suppression
- journal replay recovery
- ordered local subscriptions / reconnect fixture
- deterministic logical-change hashing and verification

The repository now includes a dedicated GitHub Actions workflow for this prototype. The repository-level workflow run has not yet been observed through the available connector, so M2-D remains `IN PROGRESS` rather than being declared fully verified.

## Next execution target

```text
M2-D deterministic offline/reconnect harness
          ↓
M3-B replay + export/import + checkpoint
          ↓
M4-A sync protocol v0
          ↓
M4-B/M4-C conflict + CRDT
```

Do not start blockchain, P2P, or a native storage rewrite before the local-first and logical-change contracts are stable.