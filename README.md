<div align="center">

# MW-DB

### Local-first data. Logical changes. Verifiable sync.

**A Rust-native research/product fork built on the mature SurrealDB substrate.**

<p>
  <a href="https://github.com/bjo163/surealdb-rebrand-mwdb/actions/workflows/mwdb-local-first.yml"><img src="https://img.shields.io/github/actions/workflow/status/bjo163/surealdb-rebrand-mwdb/mwdb-local-first.yml?branch=main&label=local-first%20CI&style=flat-square" alt="Local-first CI"></a>
  <a href="https://github.com/bjo163/surealdb-rebrand-mwdb/actions/workflows/mwdb-sync.yml"><img src="https://img.shields.io/github/actions/workflow/status/bjo163/surealdb-rebrand-mwdb/mwdb-sync.yml?branch=main&label=sync%20CI&style=flat-square" alt="Sync CI"></a>
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

> **Project status:** experimental / prototype. MW-DB is not yet a drop-in replacement for SurrealDB and is not production-ready as a distributed database.

## Why MW-DB?

MW-DB starts from a simple idea: **keep the proven database substrate, then build a stronger logical data layer around it.**

The project is exploring a database model where local writes remain useful offline, changes have stable logical identities, synchronization can resume from checkpoints, and data evolution can eventually become verifiable and branchable.

The direction is deliberately incremental:

```text
mature SurrealDB substrate
          │
          ├── local-first durability
          ├── logical change envelopes
          ├── checkpointed sync
          ├── conflict classification
          ├── CRDT experiments
          ├── provenance / verification
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

### Conflict policy + CRDT lab

M4 now includes an explicit conflict boundary:

| Situation | Prototype policy |
|---|---|
| causally ordered changes | apply in logical order |
| different objects | merge independently |
| concurrent same object | surface conflict; never silently overwrite |

There is also a standalone **G-Counter** experiment with per-actor state and max-based merge. Its tests cover the core CRDT properties of idempotence, commutativity, associativity, and preservation of the highest observed actor state.

These CRDT experiments are intentionally **not** wired into ordinary whole-object `set` mutations yet.

## Architecture

MW-DB keeps concerns separated:

| Layer | Responsibility |
|---|---|
| **SurrealDB substrate** | mature database/query/storage capabilities |
| **Local-first** | durable local state + logical journal |
| **Logical changes** | stable change identity + causal metadata |
| **Sync** | checkpointed exchange + idempotent application |
| **Conflict layer** | classify concurrent / causal relationships |
| **CRDT lab** | typed experiments with deterministic merge semantics |
| **Verification** | hashes, proofs, signatures (future) |
| **Federation** | authenticated distributed transport (future) |
| **Agent layer** | capability-scoped data operations (future) |

### Important design rules

**Physical WAL is not the sync protocol.**

The sync layer operates on explicit logical changes so transport, recovery, verification, and future federation do not become coupled to one physical storage implementation.

**Conflict is preferable to silent overwrite.**

The prototype treats concurrent writes to the same object conservatively until a domain-specific merge rule is proven safe.

## Roadmap

```text
M0  Baseline / provenance       ────────────────┐
M1  Branding / packaging                        │
M2  Local-first                 ██████████░░     │ active
M3  Logical changes             ████████░░░░     │ active
M4  Sync / conflicts            ████████░░░░     │ prototype
M5  Branch / time travel        ░░░░░░░░░░░░     │
M6  Verification                ░░░░░░░░░░░░     │
M7  Federation / distribution   ░░░░░░░░░░░░     │
M8  Adaptive consistency        ░░░░░░░░░░░░     │
M9  Agent-native data plane     ░░░░░░░░░░░░     │
M10 Native-engine decision      ◆ evidence gate │
```

### Current execution order

`M2-022 → M3-031/032 → M4-042 → M4-043 → M7`

The executable backlog lives in [`docs/MWDB-ISSUE-PLAN.md`](docs/MWDB-ISSUE-PLAN.md), while the architecture decision is recorded in [`docs/MWDB-ADR-0001-architecture-strategy.md`](docs/MWDB-ADR-0001-architecture-strategy.md).

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

### Run both

```bash
cd mwdb/local-first && cargo test && cargo check
cd ../sync && cargo test && cargo check
```

## Repository map

```text
.
├── docs/
│   ├── MWDB-BLUEPRINT.md
│   ├── MWDB-MILESTONES.md
│   ├── MWDB-ARCHITECTURE.md
│   ├── MWDB-EXECUTION-MATRIX.md
│   ├── MWDB-ISSUE-PLAN.md
│   ├── MWDB-M2-LOCAL-FIRST.md
│   ├── MWDB-M3-CHANGE-MODEL.md
│   ├── MWDB-M4-SYNC-V0.md
│   ├── MWDB-M4-CONFLICTS-CRDT-V0.md
│   └── MWDB-ADR-0001-architecture-strategy.md
├── mwdb/
│   ├── local-first/      # local durability + logical journal
│   └── sync/             # checkpoint sync + conflicts + CRDT lab
└── .github/workflows/    # focused MW-DB CI
```

## Compatibility and provenance

MW-DB is derived from the upstream SurrealDB project. The repository intentionally preserves upstream provenance while adding MW-DB-specific contracts and experimental crates.

User-facing rebranding does **not** imply that upstream copyright, licensing, trademarks, or third-party components have been removed. See the repository's license and provenance documentation before redistribution.

## Current limitations

MW-DB still needs authenticated peer identity, production network transport adapters, persisted remote resume state, real transport fault injection, field-aware merge/CRDT integration, canonical cross-language hashing, signed changes, and independent proof verification.

The M4 transport-neutral fault harness is test infrastructure, not a network simulator or production transport.

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
