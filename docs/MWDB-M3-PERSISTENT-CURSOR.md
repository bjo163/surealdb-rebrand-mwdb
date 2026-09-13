# MW-DB M3 — Persistent Peer Cursor v0

Status: **PROTOTYPE IMPLEMENTED**

This document defines the minimal durable resume state for peer synchronization without coupling it to a specific network transport.

## Contract

`mwdb-cursor::PeerCursor` stores:

- `peer_id`: stable logical identity for the replication peer.
- `last_change_id`: last acknowledged logical change, when known.
- `last_clock`: logical-clock watermark associated with that cursor.

`mwdb-cursor::CursorStore` persists a map of peer IDs to cursors in an atomic JSON replacement flow (`*.json.tmp` then rename).

## Recovery semantics

A replica may restart and reopen the cursor store without losing the last committed peer watermark. Removing a peer cursor is persistent and idempotent. Malformed cursor state fails closed rather than silently resetting replication progress.

The cursor is deliberately separate from transport/session state. It does not authenticate peers, authorize changes, or define the canonical cross-language change encoding.

## Verification

The isolated crate tests:

1. restart persistence of `peer_id`, `last_change_id`, and `last_clock`;
2. rejection of malformed persisted state;
3. persistent and idempotent cursor removal.

## Remaining gate

Before production federation, the cursor contract must be integrated with the network-facing sync loop, authenticated peer identity, durable ACK semantics, and canonical cross-language change encoding. Cursor persistence alone does not prove end-to-end sync correctness.
