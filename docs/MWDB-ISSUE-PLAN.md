# MW-DB Issue Plan

GitHub Issues are disabled in this repository at the time of writing, so this file is the executable backlog. When Issues are enabled, each `MW-*` entry can become one issue without changing scope.

## Status key

- `DONE (prototype)`: executable implementation exists with focused tests.
- `PARTIAL`: part of the contract exists, but the milestone exit gate is not met.
- `PLANNED`: not implemented yet.
- `GATE`: evidence-based decision.

## M0 — Baseline / provenance

### MW-001 — Record upstream baseline
Status: PLANNED
Scope: pin upstream repository, source commit/version, Rust toolchain, supported targets, and reproducible build/test commands.
Acceptance: a fresh contributor can reproduce the baseline from documented steps.

### MW-002 — Build/test baseline report
Status: PLANNED
Scope: run and record core build, unit/integration, language-test, and benchmark baseline.
Acceptance: green checks are recorded as green and failures are recorded with explicit disposition.

### MW-003 — License and attribution inventory
Status: PLANNED
Scope: map inherited licenses, notices, trademarks, generated assets, and redistribution constraints.
Acceptance: every redistributed upstream component has a traceable license/notice source.

### MW-004 — Upstream/MW-DB ownership map
Status: PLANNED
Scope: classify directories/files as upstream, MW-DB, or integration-owned.
Acceptance: future diffs can be classified without ambiguity.

## M1 — Branding / packaging

### MW-010 — MW-DB brand specification
Status: PLANNED
Scope: product naming, terminology, README identity, CLI/help wording, and documentation vocabulary.
Acceptance: user-facing docs consistently identify MW-DB without falsely removing upstream provenance.

### MW-011 — Package/binary naming map
Status: PLANNED
Scope: Rust crate names, binary names, SDK package naming, compatibility aliases, and migration path.
Acceptance: names are documented before mass package renames occur.

### MW-012 — Compatibility matrix
Status: PLANNED
Scope: document what remains compatible with upstream SurrealDB and what is intentionally MW-DB-specific.
Acceptance: each public divergence has an explicit compatibility note.

## M2 — Local-first

### MW-020 — Local-first contract
Status: DONE (prototype)
Scope: local reads, offline writes, durability-before-ack, restart behavior, reconnect lifecycle, and subscriptions.
Acceptance: contract is testable and a minimal end-to-end offline scenario passes.

### MW-021 — Durable change queue
Status: DONE (prototype)
Scope: persisted pending/sent/acknowledged/rejected/conflicted states with stable IDs.
Acceptance: crash/restart does not lose acknowledged local changes or duplicate them.

### MW-022 — Offline/online test harness
Status: PARTIAL
Scope: deterministic network-off, reconnect, retry, and crash simulation.
Acceptance: CI can execute representative offline scenarios deterministically.
Current: restart/replay/reconnect fixtures exist; transport fault injection and multi-replica harness remain.

### MW-023 — Subscription semantics
Status: DONE (prototype)
Scope: local events, remote events, deduplication, ordering, and resubscription after reconnect.
Acceptance: subscribers see deterministic, documented behavior.
Current: ordered local subscriptions and reconnect resubscription fixture exist; remote-originated event semantics remain in M4.

## M3 — Logical changes / version graph

### MW-030 — Change envelope v0
Status: DONE (prototype)
Scope: `change_id`, object/table identity, operation, payload/delta, actor, causal metadata, timestamps, parents, content hash.
Acceptance: mutations serialize and deserialize with stable semantics.

### MW-031 — Deterministic replay
Status: PARTIAL
Scope: apply logical changes to rebuild representative state.
Acceptance: same ordered change history yields byte-equivalent canonical state representation.
Current: startup journal replay exists; full independent replay/export fixture remains.

### MW-032 — Change feed API
Status: PARTIAL
Scope: inspect/export/import logical changes and checkpoints.
Acceptance: a consumer can resume from a checkpoint without replaying already acknowledged changes.
Current: logical JSONL export, validation, and checkpoint metadata exist; full import/resume API remains.

## M4 — Sync / conflicts

### MW-040 — Sync protocol v0
Status: PARTIAL
Scope: authenticated session, push/pull, checkpoint, acknowledgement, retry, resume, idempotency.
Acceptance: replicas converge after disconnect/reconnect without duplicate effects.
Current: protocol envelope, encode/decode validation, idempotent apply, and persistent acknowledgement helpers exist; transport and convergence are next.

### MW-041 — Conflict taxonomy
Status: PLANNED
Scope: define mergeable, causally ordered, and invariant-sensitive conflicts.
Acceptance: every tested conflict class has an explicit resolution policy.

### MW-042 — CRDT experiments
Status: PLANNED
Scope: prototype only well-defined mergeable structures.
Acceptance: merges are deterministic and tests prove no silent data loss.

### MW-043 — Sync observability
Status: PLANNED
Scope: sync latency, queue depth, retries, conflicts, bytes transferred, convergence time.
Acceptance: metrics exist for every sync state transition.

## M5 — Branching / time travel

### MW-050 — Snapshot abstraction
Status: PLANNED
Scope: logical snapshot metadata and restore semantics.
Acceptance: snapshot can be created and restored in test fixtures.

### MW-051 — Named branches
Status: PLANNED
Scope: isolated database histories.
Acceptance: branch mutations do not alter parent state until merge.

### MW-052 — Diff engine
Status: PLANNED
Scope: state and logical-change comparison.
Acceptance: diff identifies additions, removals, updates, and conflicts correctly.

### MW-053 — Merge/reject/rollback
Status: PLANNED
Scope: branch review lifecycle.
Acceptance: merge, reject, and rollback are atomic and recoverable in tests.

### MW-054 — Agent sandbox branches
Status: PLANNED
Scope: branch permissions and lifecycle for agent experimentation.
Acceptance: agent changes stay isolated and are emitted as reviewable change sets.

## M6 — Verification / provenance

### MW-060 — Canonical content hashing
Status: PARTIAL
Scope: deterministic canonicalization and hash calculation.
Acceptance: identical logical state produces identical hashes across supported targets.
Current: deterministic prototype hashing exists; canonical cross-language encoding still needs freezing.

### MW-061 — Merkle state root
Status: PLANNED
Scope: state/change Merkle construction and root calculation.
Acceptance: roots are deterministic and efficiently recomputable.

### MW-062 — Signed changes
Status: PLANNED
Scope: actor identity and signatures for trusted/federated modes.
Acceptance: tampered changes fail verification.

### MW-063 — Inclusion/state proofs
Status: PLANNED
Scope: generate and verify proofs independent of transport node.
Acceptance: an external verifier can validate an included record/change.

### MW-064 — External anchor interface
Status: PLANNED
Scope: optional interface for anchoring state roots externally.
Acceptance: anchoring is pluggable and not required for ordinary local operation.

## M7 — Federation / distribution

### MW-070 — Peer identity and capabilities
Status: PLANNED
Scope: authenticated peer IDs and capability negotiation.
Acceptance: unsupported operations are rejected before state mutation.

### MW-071 — Replication topology
Status: PLANNED
Scope: peer graph, roles, placement, and replica policy.
Acceptance: topology can be represented and validated before deployment.

### MW-072 — Federated replication
Status: PLANNED
Scope: exchange signed logical changes across independently operated nodes.
Acceptance: partition/reconnect scenarios converge or expose explicit unresolved conflicts.

### MW-073 — Failure/partition test suite
Status: PLANNED
Scope: simulated partitions, delays, duplicates, reordering, and node recovery.
Acceptance: no unbounded silent divergence in tested scenarios.

### MW-074 — Coordination/consensus abstraction
Status: PLANNED
Scope: pluggable coordination interface for data requiring stronger ordering.
Acceptance: consensus dependency is explicit and isolated from local-first mode.

## M8 — Adaptive trust / consistency

### MW-080 — Consistency policy schema
Status: PLANNED
Scope: represent local, eventual/mergeable, causal, serializable, coordinated/BFT policies.
Acceptance: policy is stored/configured alongside data semantics.

### MW-081 — Policy enforcement
Status: PLANNED
Scope: reject operations that violate selected consistency guarantees.
Acceptance: invariant-sensitive fixtures cannot be corrupted by a weaker policy.

### MW-082 — Semantic consistency tests
Status: PLANNED
Scope: cross-mode consistency test matrix.
Acceptance: each policy has positive and negative tests.

## M9 — Agent-native data plane

### MW-090 — Typed agent API
Status: PLANNED
Scope: inspect, query, mutate, branch, test, diff, propose, merge, subscribe, prove.
Acceptance: API is capability-scoped and type-safe.

### MW-091 — Agent permissions/sandbox
Status: PLANNED
Scope: least privilege and branch isolation.
Acceptance: agent cannot mutate outside its granted scope.

### MW-092 — Reviewable change-set format
Status: PLANNED
Scope: human-readable and machine-replayable proposed changes.
Acceptance: changes can be reviewed before merge and replayed deterministically.

## M10 — Native-engine decision gate

### MW-100 — Comparative benchmark suite
Status: GATE
Scope: compare representative MW-DB workloads against relevant incumbent configurations.
Acceptance: results are reproducible and include latency, throughput, storage, bandwidth, and recovery metrics.

### MW-101 — Substrate gap analysis
Status: GATE
Scope: identify workload or architectural gaps that upstream reuse cannot solve cleanly.
Acceptance: each proposed native component has evidence, estimated maintenance cost, and migration impact.

### MW-102 — Native-engine decision ADR
Status: GATE
Scope: decide continue/replace/selectively-reimplement.
Acceptance: decision cites benchmark and legal/operational constraints.

## Current execution priority

`M2-022 -> M3-031/032 -> M4-040 -> M4-041/042 -> M4-043`

M2 local-first and M3 logical-change prototypes are now the active foundation. Do not claim M4 convergence until a real two-replica reconnect fixture passes.
