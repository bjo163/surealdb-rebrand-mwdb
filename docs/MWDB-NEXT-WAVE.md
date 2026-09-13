# MW-DB Next Wave

## Objective

Turn the current collection of working prototypes into one verifiable replication path before adding more product surface.

## Pipeline target

Capability negotiation → authenticated peer → canonical change decoding → signature verification → replay-window validation → conflict classification → durable apply → Merkle/update checkpoint → persistent cursor → observability event.

## Exit criteria

- one end-to-end deterministic two-peer fixture exercises the full path;
- failures are fail-closed and retry-safe;
- cursor advances only after durable acceptance;
- metrics are emitted by the actual sync path;
- independent vectors verify canonical bytes, signatures, and Merkle proofs.
