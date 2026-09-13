# MW-DB Provenance Baseline

Status: ACTIVE

## Upstream identity

MW-DB is a product/research fork of the SurrealDB repository and retains the upstream Rust workspace as its database substrate.

- Upstream repository: `https://github.com/surrealdb/surrealdb`
- MW-DB repository: `https://github.com/bjo163/surealdb-rebrand-mwdb`
- Current upstream-derived workspace version: `3.3.0-nightly`
- Current MW-DB branch: `main`

## Provenance policy

1. Upstream-derived code remains attributable to its original project and authors.
2. MW-DB-specific code is isolated under `mwdb/` and documented as an additional layer unless a change genuinely belongs in the shared substrate.
3. User-facing rebranding does not remove upstream licensing, copyright, trademark, or third-party obligations.
4. Release notes must identify the upstream baseline and the MW-DB-specific delta.
5. No production release is permitted without a reproducible source baseline and compatibility record.

## Release evidence

The machine-readable release gate is `docs/MWDB-RELEASE-GATE.json`.
Canonical interoperability vectors are in `docs/canonical-vectors.json`.

## Scope boundary

The current strategy is explicitly **not** a storage-engine rewrite. MW-DB adds local-first durability, logical changes, synchronization, identity, verification, and federation-oriented contracts around the existing database substrate.
