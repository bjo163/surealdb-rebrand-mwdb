# MW-DB Milestones

Status: active engineering roadmap.

MW-DB is currently an upstream-derived research/product track. The repository already contains a large Rust/SurrealDB substrate. The milestone status below distinguishes **existing upstream capability** from **MW-DB-owned capability**.

## Status legend

- `DONE`: evidenced in the current repository/docs and requires no new MW-DB implementation for that item.
- `BASELINE`: existing upstream capability that MW-DB can reuse, but not yet an MW-DB contract.
- `NEXT`: highest-priority MW-DB work.
- `PLANNED`: not started as an MW-DB feature.
- `GATE`: decision milestone; must be evidence-driven.

## M0 — Baseline, provenance, and upstream boundary

**Status: NEXT**

Objective: make the repository reproducible and define exactly what is inherited versus newly owned.

### M0-A Repository baseline
- Record upstream repository and pinned base commit/version.
- Record toolchain/platform matrix.
- Record build, unit, integration, language-test, and benchmark commands.
- Record current green/red checks without hiding failures.

### M0-B License/provenance map
- Inventory LICENSE, NOTICE, copyright, trademarks, generated assets, and upstream links.
- Document the exact license boundary for each inherited component.
- Explicitly prohibit presenting upstream work as MW-DB-original work.

### M0-C Ownership map
- Mark code/docs under `upstream`, `MW-DB`, and `integration` ownership.
- Define the rule for upstream sync/cherry-pick versus MW-DB feature work.

**Exit criteria:** a clean baseline report exists and a contributor can explain the upstream delta and license boundary.

## M1 — Branding and packaging

**Status: NEXT**

Objective: establish MW-DB as the product identity without destructive renaming.

### M1-A Brand surface
- README identity.
- Documentation identity.
- CLI/help/version presentation plan.
- Terminology glossary: MW-DB, engine, node, sync, branch, proof, peer.

### M1-B Package map
- Decide canonical Rust crate names.
- Decide binary names and compatibility aliases.
- Decide npm/Python/Go/Rust SDK naming conventions.

### M1-C Upstream compatibility
- Keep upstream references where legally/technically required.
- Define compatibility claims versus intentional divergence.

**Exit criteria:** a new contributor can distinguish MW-DB from upstream SurrealDB within five minutes.

## M2 — Local-first substrate

**Status: NEXT — FIRST FEATURE MILESTONE**

Objective: make local durable state the primary execution target rather than a cache.

### M2-A Local state contract
- Local reads require no network.
- Offline writes are permitted by policy.
- Successful acknowledgement means durable local persistence.
- Define restart behavior.

### M2-B Durable pending changes
- Stable local queue.
- Persisted status: pending/sent/acknowledged/rejected/conflicted.
- Replay-safe IDs.

### M2-C Subscription semantics
- Local subscription events.
- Remote-originated events.
- Reconnect/resubscription behavior.

### M2-D Test harness
- Offline/online toggle.
- Crash/restart simulation.
- Deterministic fixture workload.

**Exit criteria:** a representative workload can read/write offline, restart safely, reconnect, and preserve acknowledged effects.

## M3 — Logical event and version graph

**Status: PLANNED**

Objective: create a logical change model independent from physical WAL records.

### M3-A Change envelope
Minimum fields:
`change_id`, `object_id`, `operation`, `payload/delta`, `actor`, `causal metadata`, `created_at`, `parents`, `content_hash`.

### M3-B Replay engine
- Serialize logical changes.
- Apply changes deterministically.
- Reconstruct state from a known history.

### M3-C Change feed
- Local change stream.
- Export/import changes.
- Version/history inspection.

**Exit criteria:** representative mutations can be exported, replayed, and reconstructed deterministically.

## M4 — Sync and conflict engine

**Status: PLANNED**

Objective: synchronize independent local replicas safely.

### M4-A Sync protocol v0
- Authenticated handshake.
- Push/pull.
- Cursor/checkpoint.
- Acknowledgement.
- Retry/resume.
- Idempotent application.

### M4-B Conflict classification
Classify changes as:
- mergeable,
- ordered but mergeable,
- invariant-sensitive/conflicting.

### M4-C CRDT experiments
Start only with structures where deterministic merge semantics are well-defined.

**Exit criteria:** two or more replicas converge after partition/reconnect and no conflict is silently discarded.

## M5 — Branch, snapshot, diff, and time travel

**Status: PLANNED**

Objective: treat database state as branchable history.

### M5-A Snapshot primitives
### M5-B Named branches
### M5-C State/change diff
### M5-D Restore/rollback
### M5-E Merge/reject
### M5-F Agent sandbox branch

**Exit criteria:** a branch can diverge, be inspected, tested, merged, rejected, and safely discarded.

## M6 — Verification and provenance

**Status: PLANNED**

Objective: make data independently verifiable.

### M6-A Hash identity
### M6-B Merkle state root
### M6-C Signed changes
### M6-D Inclusion/state proofs
### M6-E Optional external anchoring API

**Exit criteria:** an independent verifier can validate the integrity/inclusion of a received state or change without trusting the transport node.

## M7 — Federation and distributed execution

**Status: PLANNED**

Objective: support independently operated nodes.

### M7-A Peer identity/capabilities
### M7-B Replication topology
### M7-C Authenticated peer transport
### M7-D Placement and routing policy
### M7-E Partition/failure test matrix
### M7-F Pluggable coordination/consensus interface

**Exit criteria:** independent nodes can exchange authenticated state through tested partition/recovery scenarios.

## M8 — Adaptive trust and consistency

**Status: PLANNED**

Objective: make consistency proportional to data semantics.

Modes:
- local,
- mergeable/eventual,
- causal,
- serializable,
- coordinated/BFT.

### M8-A Schema/data consistency policy
### M8-B Policy enforcement
### M8-C Semantic test suite
### M8-D Operational observability

**Exit criteria:** consistency policy is explicit and testable rather than an implicit global setting.

## M9 — Agent-native data plane

**Status: PLANNED**

Objective: make database history, isolation, testing, and review safe for AI agents.

Operations:
`inspect`, `query`, `mutate`, `branch`, `test`, `diff`, `propose`, `merge`, `subscribe`, `prove`.

### M9-A Typed agent API
### M9-B Agent permissions/sandbox
### M9-C Reviewable change-set format
### M9-D Agent replay/provenance

**Exit criteria:** an agent can work in an isolated branch and emit a reviewable, replayable change set.

## M10 — Native-engine decision gate

**Status: GATE**

Objective: decide whether any subsystem should become MW-DB-native.

Inputs:
- performance benchmarks,
- real workload telemetry,
- maintenance cost,
- upstream roadmap/dependency risk,
- license/commercial constraints,
- portability,
- storage/network cost.

Possible outcomes:
1. remain upstream-derived,
2. replace one specific subsystem,
3. create a parallel MW-DB-native engine track.

**Exit criteria:** documented decision with measurements and a migration plan, if any.

## Execution order

```text
M0 baseline
  -> M1 identity
  -> M2 local-first
  -> M3 logical changes
  -> M4 sync/conflicts
  -> M5 branches/time travel
  -> M6 verification
  -> M7 federation
  -> M8 adaptive trust
  -> M9 agents
  -> M10 native-engine gate
```

Work may run in parallel **inside** a milestone after its contract is frozen. Do not parallelize cross-milestone implementations that can create incompatible data or protocol formats.

## Competitive principle

MW-DB does not win by cloning every feature of PostgreSQL, MongoDB, S3, Supabase, or Neon. The differentiation target is a coherent lifecycle:

`local -> sync -> version -> branch -> verify -> federate -> decentralize`

Performance competition follows real workload evidence.