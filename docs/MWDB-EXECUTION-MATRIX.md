# MW-DB Execution Matrix

This is the working dashboard for determining **where the project is now** and **what counts as done**.

## Current repository baseline

- Repository: `bjo163/surealdb-rebrand-mwdb`
- Branch: `main`
- Substrate: existing SurrealDB Rust workspace
- MW-DB feature substrate: `mwdb/local-first`
- Current posture: upstream-derived research/product fork with the first MW-DB local-first prototype implemented.
- GitHub Issues: disabled; executable backlog remains in `docs/MWDB-ISSUE-PLAN.md`.

## Overall status

```text
M0  Baseline / provenance       NEXT
M1  Branding / packaging       NEXT
M2  Local-first                IN PROGRESS ← current feature gate
M3  Logical changes            PLANNED
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
| M2-C | Local | local subscriptions/reconnect semantics | PLANNED | API + tests | deterministic |
| M2-D | Local | offline/recovery harness | IN PROGRESS | restart/replay tests | repeatable |
| M3-A | Events | logical change envelope | BASELINE (prototype) | `ChangeEnvelope` v0 | versioned |
| M3-B | Events | deterministic replay | PARTIAL (prototype) | startup journal replay | reproducible |
| M3-C | Events | change feed/export/import | PLANNED | API | stable |
| M4-A | Sync | push/pull/checkpoint/resume | PLANNED | sync v0 | convergent |
| M4-B | Conflicts | conflict taxonomy | PLANNED | policy spec | explicit |
| M4-C | CRDT | mergeable-structure prototypes | PLANNED | test suite | deterministic |
| M5-A | Versioning | snapshots | PLANNED | API/storage mapping | restorable |
| M5-B | Versioning | named branches | PLANNED | API | isolated |
| M5-C | Versioning | diff | PLANNED | change/state diff | accurate |
| M5-D | Versioning | merge/reject/rollback | PLANNED | workflow | safe |
| M6-A | Proof | hashes/content identity | PLANNED | implementation | deterministic |
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
              +-- ChangeEnvelope
              +-- status records
              +-- duplicate suppression
              +-- replay on startup
```

The journal is written and synced before `set()` returns. Startup replays logical changes so a crash between journal and snapshot persistence can be recovered.

## Verification status

Prototype tests cover:

- offline local write
- restart persistence
- acknowledgement persistence
- duplicate change suppression
- journal replay recovery

The prototype has **not yet been claimed as fully verified by CI**, because the repo is primarily an upstream workspace and the new crate is intentionally isolated. The next execution task is to add deterministic CI execution for `mwdb/local-first` and complete M2-C/D.

## Definition of Done for a MW-DB milestone

A milestone is `DONE` only when:

1. contract is documented;
2. implementation exists where required;
3. positive tests pass;
4. failure/recovery tests pass where relevant;
5. benchmark baseline exists;
6. compatibility/migration impact is documented;
7. relevant issue/ADR is updated;
8. next milestone has an explicit dependency boundary.

## Next execution target

```text
M2-C subscription semantics
       +
M2-D deterministic offline/reconnect harness
       ↓
M3-A change envelope versioning
       ↓
M3-B replay/export/import
       ↓
M4 sync protocol
```

Do not start blockchain, P2P, or a native storage rewrite before the local-first and logical-change contracts are stable.