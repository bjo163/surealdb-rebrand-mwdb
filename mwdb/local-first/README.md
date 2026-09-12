# MW-DB Local-First Prototype

This crate is the first executable MW-DB feature slice. It intentionally sits outside the large upstream workspace so the experimental contract can evolve without invasive changes to the SurrealDB substrate.

## Current capabilities

- durable local key/value state
- append-only logical change journal
- `sync_data()` before a local write is acknowledged
- stable change IDs with duplicate suppression
- persisted change status
- journal replay on startup
- restart/recovery tests

## Run

```bash
cargo test --manifest-path mwdb/local-first/Cargo.toml
```

## Files

- `src/lib.rs` — local-first store and durable change queue
- `Cargo.toml` — isolated research crate

## Contract

A successful `set()` means the logical change is durable in `changes.log`. `state.json` is a rebuildable snapshot, not the sole source of truth. The network is not required for local reads or writes.

## Deliberate limitations

This is not yet the M4 sync protocol. There is no network transport, remote acknowledgement, CRDT merge, cryptographic identity, or conflict engine here. Those are downstream milestones and should consume the stable logical-change envelope rather than inventing another format.
