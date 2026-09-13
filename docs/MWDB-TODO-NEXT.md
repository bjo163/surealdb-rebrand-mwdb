# MW-DB Next Execution TODO

## P0 — execute now

- [ ] P0-01 Canonical numeric normalization + cross-language test vectors
- [ ] P0-02 Independent non-Rust canonical verifier
- [ ] P0-03 Integrate persistent peer cursor into sync engine
- [ ] P0-04 Public-key peer identity + asymmetric change signatures
- [ ] P0-05 Replay protection: nonce/sequence/window + duplicate detection
- [ ] P0-06 Integrate signed changes with Merkle state/inclusion proofs
- [ ] P0-07 Deterministic partition/federation suite
- [ ] P0-08 Provenance/reproducible baseline

## P1 — next wave

- [ ] P1-01 Wire observability into sync engine
- [ ] P1-02 Capability negotiation state machine + enforcement
- [ ] P1-03 Key rotation/revocation
- [ ] P1-04 Proof serialization/versioning
- [ ] P1-05 Replaceable network transport adapter
- [ ] P1-06 Logical operations: create/update/delete
- [ ] P1-07 Compatibility/migration matrix
- [ ] P1-08 Benchmark + regression gate

## P2

- [ ] P2-01 Snapshot/branch/time-travel/merge

## Exit rule

Do not call MW-DB distributed production-ready until P0 items are implemented or explicitly waived with evidence. Do not call the cryptographic layer production-ready while it uses prototype symmetric signing or non-versioned proof formats.
