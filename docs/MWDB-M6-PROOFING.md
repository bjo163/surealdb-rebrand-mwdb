# MW-DB M6 Proofing

Status: PROTOTYPE / IN PROGRESS.

## Canonical encoding

`mwdb/canonical` provides deterministic JSON encoding for the current Rust prototype. Object keys are sorted recursively and array order remains semantic.

The encoding is intentionally not yet declared a final cross-language standard. Numeric normalization, interoperability vectors, and a frozen wire specification remain exit-gate work.

## Signed changes

`mwdb/signing` provides a keyed BLAKE3 MAC over change ID, actor ID, and payload. This proves payload integrity when both peers already share the secret key.

This is not public-key identity, key rotation, certificate validation, or a production federation identity system.

## Merkle state root

`mwdb/merkle` provides a deterministic binary tree root over ordered leaf payloads. Duplicate final leaves are used when a level has odd cardinality.

Inclusion paths and independent verifier tooling remain future work.

## Exit gate

M6 is not production-ready until canonical test vectors are frozen, independent implementations reproduce the same digests, signed changes have explicit key/identity lifecycle rules, and state-root/inclusion verification is independently tested.
