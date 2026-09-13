# MW-DB M6 Verification Status

Status: IN PROGRESS — PROTOTYPE

## Completed in this slice

- `mwdb/canonical`: recursive object-key ordering, deterministic UTF-8 JSON representation, and BLAKE3 helper.
- `docs/MWDB-M6-TEST-VECTORS.md`: published canonical encoding vectors for independent implementations.
- `mwdb/signing`: signed-change envelope using a shared 32-byte BLAKE3 keyed MAC over change ID, actor ID, and payload.
- `mwdb/merkle`: deterministic ordered binary-tree state root with domain-separated leaf/parent hashing.
- `docs/MWDB-M6-PROOFING.md`: records scope and production limitations.

## Still open

- Freeze numeric normalization and final canonical wire specification.
- Add independent non-Rust vector implementation and frozen digest fixtures.
- Add public-key signatures, identity lifecycle, rotation, and replay policy.
- Add Merkle inclusion paths and an independent verifier.
- Integrate signing, roots, and cursor checks into the sync path.

## Release rule

These components are research/prototype primitives. They do not constitute production federation security until the open gates above are independently verified.
