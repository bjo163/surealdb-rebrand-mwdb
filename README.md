<div align="center">

# MW-DB

### Local-first data. Logical changes. Verifiable sync.

**A Rust-native research/product fork built on the mature SurrealDB substrate.**

<p>
  <a href="https://github.com/bjo163/surealdb-rebrand-mwdb/actions/workflows/mwdb-local-first.yml"><img src="https://img.shields.io/github/actions/workflow/status/bjo163/surealdb-rebrand-mwdb/mwdb-local-first.yml?branch=main&label=local-first%20CI&style=flat-square" alt="Local-first CI"></a>
  <a href="https://github.com/bjo163/surealdb-rebrand-mwdb/actions/workflows/mwdb-sync.yml"><img src="https://img.shields.io/github/actions/workflow/status/bjo163/surealdb-rebrand-mwdb/mwdb-sync.yml?branch=main&label=sync%20CI&style=flat-square" alt="Sync CI"></a>
  <a href="https://github.com/bjo163/surealdb-rebrand-mwdb/actions/workflows/mwdb-replay.yml"><img src="https://img.shields.io/github/actions/workflow/status/bjo163/surealdb-rebrand-mwdb/mwdb-replay.yml?branch=main&label=replay%20CI&style=flat-square" alt="Replay CI"></a>
  <a href="https://github.com/bjo163/surealdb-rebrand-mwdb"><img src="https://img.shields.io/github/stars/bjo163/surealdb-rebrand-mwdb?style=flat-square" alt="GitHub stars"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/built%20with-Rust-dea584?style=flat-square&logo=rust&logoColor=white" alt="Built with Rust"></a>
</p>

<p>
  <a href="#why-mw-db">Why MW-DB</a> ·
  <a href="#architecture">Architecture</a> ·
  <a href="#roadmap">Roadmap</a> ·
  <a href="#quick-start">Quick start</a> ·
  <a href="#status">Status</a>
</p>

</div>

> **Project status:** release-track prototype. MW-DB is not yet a drop-in replacement for SurrealDB and is not production-ready as a distributed database. See [`docs/MWDB-RELEASE-GATE.json`](docs/MWDB-RELEASE-GATE.json) for the blocking P0 gates.

## Why MW-DB?

MW-DB starts from a simple idea: **keep the proven database substrate, then build a stronger logical data layer around it.**

The project is exploring a database model where local writes remain useful offline, changes have stable logical identities, synchronization can resume from checkpoints, and data evolution can become verifiable and branchable.

The direction is deliberately incremental:

```text
mature SurrealDB substrate
          │
          ├── local-first durability
          ├── logical change envelopes
          ├── checkpointed sync
          ├── conflict classification
          ├── CRDT experiments
          ├── provenance / identity / signing
          ├── Merkle state + proofs
          ├── federation / transport
          ├── branching / time travel
          └── agent-native data operations
```

MW-DB is **not** starting with a storage-engine rewrite, blockchain, or custom P2P stack. Those are later decision gates that require benchmarks and evidence.

## What exists today

### Local-first core

`mwdb/local-first` provides the first executable MW-DB primitive:

- durable append-only logical journal;
- local writes that survive restart;
- snapshot rebuild from the journal;
- pending / acknowledged / rejected / conflicted status;
- ordered local subscriptions;
- deterministic logical-change hashing;
- checkpoint-based logical change selection;
- durable application of verified remote changes;
- duplicate change suppression.

### Sync core

`mwdb/sync` is a transport-neutral prototype for synchronization:

- `SyncHello` / `SyncBatch` / `SyncAck` envelopes;
- explicit protocol versioning;
- content-hash verification;
- checkpoint delta selection;
- idempotent batch application;
- persistent acknowledgement helpers;
- conservative conflict classification;
- deterministic two-replica offline/reconnect fixture;
- deterministic drop / duplicate / reorder fault harness.

### Verification substrate

The current prototype also contains focused crates for:

- canonical encoding;
- public-key peer identity;
- signing;
- Merkle state roots and inclusion proofs;
- replay-window checks;
- capability intersection.

These components are on the **integration path**, not presented as production security guarantees yet. Cross-language canonical vectors are maintained in [`docs/canonical-vectors.json`](docs/canonical-vectors.json).

### Conflict policy + CRDT lab

M4 includes an explicit conflict boundary:

| Situation | Prototype policy |
|---|---|
| causally ordered changes | apply in logical order |
| different objects | merge independently |
| concurrent same object | surface conflict; never silently overwrite |

There is also a standalone **G-Counter** experiment with per-actor state and max-based merge. These experiments are intentionally not wired into ordinary whole-object `set` mutations yet.

### Replay / import

`mwdb/replay` provides a small independent validation surface for logical histories:

- JSONL export helper;
- hash-verified import through `LocalFirstStore`;
- deterministic state reconstruction;
- idempotent re-import;
- malformed-input line reporting.

This keeps M3 replay/import independently testable without turning the main SurrealDB workspace into a heavy experimental dependency graph.

## Architecture

MW-DB keeps concerns separated:

| Layer | Responsibility |
|---|---|
| **SurrealDB substrate** | mature database/query/storage capabilities |
| **Local-first** | durable local state + logical journal |
| **Logical changes** | stable change identity + causal metadata |
| **Sync** | checkpointed exchange + idempotent application |
| **Conflict layer** | classify concurrent / causal relationships |
| **Verification** | hashes, signatures, Merkle roots, proofs |
| **Identity / trust** | public-key peer identity and trust boundaries |
| **Replay protection** | peer-scoped duplicate/replay control |
| **Federation** | authenticated distributed transport |
| **Agent layer** | capability-scoped data operations |

### Important design rules

**Physical WAL is not the sync protocol.**

The sync layer operates on explicit logical changes so transport, recovery, verification, and future federation do not become coupled to one physical storage implementation.

**Conflict is preferable to silent overwrite.**

The prototype treats concurrent writes to the same object conservatively until a domain-specific merge rule is proven safe.

## Roadmap

```text
M0  Baseline / provenance       ████░░░░░░░░     release gate
M1  Branding / packaging        ██████████░░     prototype
M2  Local-first                 ██████████░░     prototype
M3  Logical changes              █████████░░░     prototype
M4  Sync / conflicts             █████████░░░     prototype
M5  Branch / time travel         ░░░░░░░░░░░░     next
M6  Verification                 █████░░░░░░░     integration
M7  Federation / distribution    █████░░░░░░░     integration
M8  Adaptive consistency         ░░░░░░░░░░░░     later
M9  Agent-native data plane      ░░░░░░░░░░░░     later
M10 Native-engine decision       ◆ evidence gate
```

### Current execution order

`P0 integration → canonical interoperability → identity/replay → signing/Merkle → transport/partition → benchmark/provenance → M5 branch/time-travel`

The executable backlog lives in [`docs/MWDB-ISSUE-PLAN.md`](docs/MWDB-ISSUE-PLAN.md), while the architecture decision is recorded in [`docs/MWDB-ADR-0001-architecture-strategy.md`](docs/MWDB-ADR-0001-architecture-strategy.md). The current release gate is [`docs/MWDB-RELEASE-GATE.json`](docs/MWDB-RELEASE-GATE.json).

## Quick start

The main repository is a large upstream-derived Rust workspace. MW-DB experimental crates are intentionally isolated under `mwdb/` so they can iterate without forcing every contributor to build the full upstream workspace.

### Run the local-first prototype

```bash
cd mwdb/local-first
cargo test
cargo check
```

### Run the sync + CRDT prototype

```bash
cd mwdb/sync
cargo test
cargo check
```

### Run the replay/import prototype

```bash
cd mwdb/replay
cargo test
cargo check
```

### Run the canonical vectors

The same vector file is intended to be implemented independently by another language/runtime before GAP-02 can close.

## Repository map

```text
.
├── docs/
│   ├── MWDB-BLUEPRINT.md
│   ├── MWDB-MILESTONES.md
│   ├── MWDB-ARCHITECTURE.md
│   ├── MWDB-EXECUTION-MATRIX.md
│   ├── MWDB-ISSUE-PLAN.md
│   ├── MWDB-RELEASE-GATE.json
│   ├── MWDB-PROVENANCE.md
│   ├── MWDB-AUDIT-GAP-TASKS.md
│   ├── canonical-vectors.json
│   └── MWDB-ADR-0001-architecture-strategy.md
├── mwdb/
│   ├── auth/
│   ├── canonical/
│   ├── capabilities/
│   ├── cursor/
│   ├── identity/
│   ├── local-first/
│   ├── merkle/
│   ├── observability/
│   ├── replay/
│   ├── signing/
│   ├── sync/
│   └── trust/
└── .github/workflows/    # focused MW-DB and upstream CI
```

## Compatibility and provenance

MW-DB is derived from the upstream SurrealDB project. The repository intentionally preserves upstream provenance while adding MW-DB-specific contracts and experimental crates. See [`docs/MWDB-PROVENANCE.md`](docs/MWDB-PROVENANCE.md).

User-facing rebranding does **not** imply that upstream copyright, licensing, trademarks, or third-party components have been removed. See the repository's license and provenance documentation before redistribution.

## Current limitations

MW-DB still needs an independent canonical implementation, full sync/cursor wiring, enforced public-key identity, persistent replay/key lifecycle, signed-change + Merkle integration, a production network transport adapter, and a partition/federation test suite.

The current transport-neutral fault harness is test infrastructure, not a network simulator or production transport.

M3 replay currently supports the prototype's `set` operation. Unsupported operations are rejected rather than approximated.

Most importantly, **same-object concurrent writes are not silently merged today**. The prototype classifier treats them conservatively as `ConcurrentSameObject` and surfaces them for explicit resolution.

## Contributing

Keep changes small, testable, and isolated by milestone. Prefer a focused crate under `mwdb/` over modifying the upstream core unless the feature genuinely belongs in the shared substrate.

Every milestone should leave behind:

1. a documented contract;
2. executable tests, including failure/recovery tests where relevant;
3. a benchmark baseline when performance is part of the decision;
4. compatibility/provenance notes;
5. an updated roadmap/issue entry.

## License

See [`LICENSE`](LICENSE) and the provenance/architecture documents for licensing and upstream attribution details.
