# MW-DB M6 Canonical Encoding

Status: PROTOTYPE

This document freezes the current design boundary for cross-implementation change hashing.

## Scope

The prototype canonical form is UTF-8 JSON with recursively sorted object keys and deterministic array order. Numbers must be JSON numbers accepted by serde_json; non-JSON values are not representable.

This is a repository-local prototype contract, not yet a claim of RFC 8785/JCS interoperability.

## Gate

Before production federation, publish test vectors in at least two independent implementations and verify identical canonical bytes and BLAKE3 hashes.

## Non-goals

No storage-engine change, network transport, signing, or blockchain anchoring is implied by this contract.
