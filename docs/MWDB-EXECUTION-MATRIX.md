# MW-DB Execution Matrix

This is the working dashboard for determining **where the project is now** and **what counts as done**.

## Current repository baseline

- Repository: `bjo163/surealdb-rebrand-mwdb`
- Branch: `main`
- Substrate: existing SurrealDB Rust workspace
- Current posture: upstream-derived research/product fork
- GitHub Issues: disabled at time of roadmap authoring; executable issue plan is kept in `docs/MWDB-ISSUE-PLAN.md`.

## Overall status

```text
M0  Baseline / provenance       NEXT
M1  Branding / packaging       NEXT
M2  Local-first                NEXT ← first feature gate
M3  Logical changes            PLANNED
M4  Sync / conflicts            PLANNED
M5  Branch / time travel       PLANNED
M6  Verification                PLANNED
M7  Federation / distribution   PLANNED
M8  Adaptive trust              PLANNED
M9  Agent-native                PLANNED
M10 Native-engine decision      GATE
```

The upstream substrate already supplies many mature database capabilities. Those are marked as **BASELINE**, not as completed MW-DB differentiation.

## Detailed matrix

| ID | Area | Work | Status | Output | Gate |
|---|---|---|---|---|---|
| M0-A | Baseline | reproducible build/test/toolchain record | NEXT | baseline report | reproducible |
| M0-B | Legal | license/attribution inventory | NEXT | provenance map | reviewed |
| M0-C | Architecture | upstream/MW-DB ownership map | NEXT | ownership contract | reviewed |
| M1-A | Brand | MW-DB terminology and README identity | NEXT | brand/spec | coherent |
| M1-B | Packaging | crate/binary/SDK naming map | NEXT | package map | coherent |
| M1-C | Compatibility | upstream compatibility policy | NEXT | compatibility matrix | explicit |
| M2-A | Local | local-first contract | NEXT | protocol/spec | tested |
| M2-B | Local | durable pending-change queue | PLANNED | implementation | crash-safe |
| M2-C | Local | local subscriptions/reconnect semantics | PLANNED | tests/API | deterministic |
| M2-D | Local | offline harness | PLANNED | integration tests | repeatable |
| M3-A | Events | logical change envelope | PLANNED | schema | versioned |
| M3-B | Events | deterministic replay | PLANNED | replay tool/tests | reproducible |
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

## What is already reusable from upstream

The current repository already contains a substantial Rust database platform including the core/query/server/types/workspace structure. This should be reused until evidence demonstrates a bottleneck or strategic constraint.

Potentially reusable areas include:

- query and parser infrastructure;
- transactional/state engine;
- indexes and storage backends;
- server/network stack;
- SDK/type layers;
- WASM/embedded support;
- existing language/integration tests.

These items are **not counted as MW-DB milestones complete** merely because they exist upstream.

## Definition of Done for a MW-DB milestone

A milestone is `DONE` only when:

1. the contract is documented;
2. implementation exists where required;
3. positive tests pass;
4. failure/recovery tests pass where relevant;
5. benchmark baseline exists;
6. compatibility/migration impact is documented;
7. the relevant issue/ADR is updated;
8. the next milestone has an explicit dependency boundary.

## First implementation target

Do not start with blockchain, P2P, or a native storage rewrite.

The first meaningful MW-DB feature slice is:

```text
M0 baseline
  -> M1 identity
  -> M2 local-first
  -> M3 logical change
  -> M4 sync
```

This gives a usable vertical slice before decentralized complexity is introduced.