# MW-DB Authenticated Transport Boundary

Status: PROTOTYPE

`mwdb/auth` defines a transport-neutral authenticated frame using a 32-byte shared secret and keyed BLAKE3. The authenticated domain includes protocol version, node ID, nonce, issue time, payload length, and payload bytes.

## Security boundary

This proves message integrity and possession of a shared key for the prototype. It does not provide public-key identity, key rotation, forward secrecy, certificate validation, authorization, replay-window enforcement, or production peer federation.

## Required production gate

Before federated deployment, replace or wrap this adapter with an explicit peer-identity design, replay policy, key lifecycle, capability negotiation, and independent interoperability/security tests.
