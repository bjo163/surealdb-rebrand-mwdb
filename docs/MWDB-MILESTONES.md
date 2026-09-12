# MW-DB Milestones

This roadmap is intentionally staged so the project gains useful capabilities early while preserving a path toward decentralized data.

## M0 — Baseline & Upstream Boundary

Goal: make the fork/rebrand understandable and reproducible.

Deliverables:
- upstream commit/version record
- license and attribution inventory
- MW-DB naming policy
- architecture map
- benchmark baseline
- CI/release baseline

Exit: repository can be rebuilt and its upstream delta can be explained.

## M1 — Branding & Packaging

Goal: introduce MW-DB identity without destructive rename churn.

Deliverables:
- MW-DB README/docs identity
- package/binary naming plan
- compatibility matrix
- upstream vs MW-DB ownership map

Exit: a new contributor can tell what is upstream and what is MW-DB.

## M2 — Local-First

Goal: make local state a first-class execution mode.

Deliverables:
- local persistence contract
- offline reads/writes
- durable pending-change queue
- reconnect semantics
- client subscription model
- local-first integration tests

Exit: representative application workload works offline and converges after reconnect.

## M3 — Event / Version Graph

Goal: give every logical mutation an auditable identity.

Deliverables:
- stable event/change IDs
- causal parents or logical clocks
- actor identity
- event hash
- replay/rebuild tooling
- change feed API

Exit: state can be reconstructed from a logical change history in tested scenarios.

## M4 — Sync / Conflict Engine

Goal: turn local-first state into reliable multi-device state.

Deliverables:
- push/pull protocol
- idempotent application of changes
- resumable synchronization
- conflict classification
- merge policies
- CRDT experiments for safe structures

Exit: two or more replicas converge under partition/reconnect test suites.

## M5 — Branching / Time Travel

Goal: make database state branchable like source code.

Deliverables:
- snapshots
- named branches
- compare/diff
- restore/rollback
- branch merge
- agent sandbox branch

Exit: an application can create an isolated branch, mutate it, inspect differences, and merge/reject changes.

## M6 — Verification / Provenance

Goal: make received data independently verifiable.

Deliverables:
- content hashing
- Merkle tree/state root
- signed change records
- proof generation and verification
- optional external anchor interface

Exit: a verifier can prove inclusion/integrity without trusting the transport node.

## M7 — Federation / Distribution

Goal: support independent operators and distributed topology.

Deliverables:
- authenticated peers
- replication topology
- peer capabilities
- placement policy
- distributed failure tests
- pluggable consensus interface

Exit: independent nodes can exchange authenticated state under tested failure scenarios.

## M8 — Adaptive Trust & Consistency

Goal: use the weakest coordination model that preserves application invariants.

Modes:
- local
- eventual/mergeable
- causal
- serializable
- coordinated/BFT where required

Exit: consistency policy is explicit, testable, and attached to data semantics rather than hidden global assumptions.

## M9 — Agent-Native Data Plane

Goal: make database operations safe and useful for AI agents.

Operations:
- inspect
- branch
- query
- mutate
- test
- diff
- propose
- merge
- subscribe
- prove

Exit: an agent can work in an isolated branch and produce a reviewable, replayable change set.

## M10 — Native Engine Decision

Goal: decide what, if anything, should become MW-DB-native.

Decision inputs:
- benchmarks
- operational telemetry
- maintenance cost
- upstream dependency constraints
- licensing/commercial requirements
- workload gaps

Exit: an evidence-backed decision to continue with upstream components, replace specific layers, or begin a native engine track.

## Priority rule

The first competitive advantage is not raw benchmark speed. It is the combination of:

`local-first + incremental sync + versioning + verification + branching + federation`

Performance work follows real workloads and remains subject to regression benchmarks.
