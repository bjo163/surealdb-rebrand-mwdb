# MW-DB Issue Plan

GitHub Issues are currently disabled on this repository, so the executable issue backlog is recorded here until issue tracking is enabled. Each entry is intended to become one GitHub issue without changing scope.

## M0-01 — Upstream baseline and provenance

Scope: record upstream commit/version, license/attribution inventory, and reproducible build/test baseline.

Acceptance: a contributor can identify the upstream base and reproduce the baseline.

## M1-01 — MW-DB branding and package map

Scope: define product naming, binary/package naming, and upstream-vs-MWDB ownership boundaries without destructive mass renaming.

Acceptance: docs and package map consistently describe MW-DB while preserving upstream notices.

## M2-01 — Local-first contract

Scope: define durable local state, offline reads/writes, acknowledgement rules, reconnect lifecycle, and subscription expectations.

Acceptance: contract is documented and a minimal end-to-end offline test passes.

## M2-02 — Durable change queue

Scope: persist logical local changes with stable IDs and replay-safe state.

Acceptance: restart and reconnect do not lose or duplicate acknowledged local changes.

## M3-01 — Logical event/change model

Scope: define change IDs, object identity, operation type, causal metadata, hash, actor, and replay semantics.

Acceptance: representative database mutations can be serialized, replayed, and reconstructed deterministically.

## M4-01 — Sync protocol v0

Scope: authenticated push/pull, idempotency, resumability, acknowledgements, and basic causal ordering.

Acceptance: two replicas synchronize after offline divergence without duplicate effects.

## M4-02 — Conflict policy and CRDT experiments

Scope: classify mergeable vs invariant-sensitive changes; prototype CRDTs only for well-defined structures.

Acceptance: conflict tests demonstrate deterministic outcomes and never silently discard changes.

## M5-01 — Branch/snapshot/time-travel

Scope: named database branches, snapshots, diff, restore, and merge/reject semantics.

Acceptance: a branch can diverge from a baseline, be inspected, and be merged or discarded safely.

## M6-01 — Verification/provenance

Scope: content hashes, Merkle state roots, signed logical changes, inclusion proofs, and optional anchor interface.

Acceptance: an independent verifier can validate change/state integrity without trusting the transport node.

## M7-01 — Federation and peer replication

Scope: peer identity, capabilities, authenticated replication, topology, and failure handling.

Acceptance: independent nodes can exchange and validate state under partition/reconnect tests.

## M8-01 — Adaptive trust and consistency policies

Scope: make consistency explicit per data semantics: local, mergeable/eventual, causal, serializable, or coordinated/BFT.

Acceptance: policies are represented in schema/data configuration and covered by semantic tests.

## M9-01 — Agent-native data operations

Scope: typed APIs for inspect, branch, propose, test, diff, merge, subscribe, and prove.

Acceptance: an agent can work entirely inside a branch and emit a reviewable, replayable change set.

## M10-01 — Native-engine decision gate

Scope: benchmark and review whether any substrate components should be replaced with MW-DB-native implementations.

Acceptance: decision is evidence-based and includes performance, maintenance, licensing, portability, and operational criteria.

## Priority order

`M0-01 -> M1-01 -> M2-01/M2-02 -> M3-01 -> M4-01/M4-02 -> M5-01 -> M6-01 -> M7-01 -> M8-01 -> M9-01 -> M10-01`

Parallel work is safe within a milestone when APIs/contracts are frozen first. Do not start M7+ implementation by assumption; each milestone requires passing its exit criteria.
