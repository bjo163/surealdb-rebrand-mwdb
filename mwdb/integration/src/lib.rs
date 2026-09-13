#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use mwdb_auth::{ReplayStore, SharedKeyAuthenticator};
    use mwdb_canonical::canonicalize;
    use mwdb_cursor::{CursorStore, PeerCursor};
    use mwdb_identity::{verify_signed_bytes, IdentityKey};
    use mwdb_local_first::LocalFirstStore;
    use mwdb_merkle::{build_proof, hash_leaf, state_root, verify_proof};
    use mwdb_signing::{sign, verify};
    use mwdb_sync::{apply_batch, make_ack, select_since};

    #[test]
    fn canonical_identity_signature_merkle_auth_replay_pipeline() {
        let document = serde_json::json!({
            "z": {"b": 2, "a": 1},
            "operation": "set",
            "value": {"count": 1.0}
        });
        let canonical = canonicalize(&document).unwrap();
        assert_eq!(canonical, "{\"operation\":\"set\",\"value\":{\"count\":1},\"z\":{\"a\":1,\"b\":2}}");

        let identity = IdentityKey::from_seed([11; 32]);
        let identity_signed = identity.sign(canonical.as_bytes());
        let verified_identity = verify_signed_bytes(&identity_signed).unwrap();
        assert_eq!(verified_identity.peer_id, identity.identity().peer_id);

        let change_id = uuid::Uuid::from_u128(1);
        let keyed = [17u8; 32];
        let signed_change = sign(change_id, &verified_identity.peer_id, canonical.as_bytes(), &keyed);
        assert!(verify(&signed_change, &keyed));

        let leaves = vec![
            canonical.as_bytes().to_vec(),
            signed_change.payload.clone(),
            identity_signed.payload.clone(),
        ];
        let root = state_root(&leaves);
        let proof = build_proof(&leaves, 1).unwrap();
        assert_eq!(proof.leaf, hash_leaf(&signed_change.payload));
        assert!(verify_proof(root, &proof).unwrap());

        let auth = SharedKeyAuthenticator::new([23; 32]);
        let frame = auth
            .seal(verified_identity.peer_id.clone(), 7, 123, canonical.into_bytes())
            .unwrap();
        let replay_path = std::env::temp_dir().join(format!(
            "mwdb-integration-replay-{}.json",
            uuid::Uuid::new_v4()
        ));
        {
            let mut replay = ReplayStore::open(&replay_path).unwrap();
            auth.verify_and_accept_persistent(&mut replay, 1, 8, &frame)
                .unwrap();
        }
        let mut reopened = ReplayStore::open(&replay_path).unwrap();
        assert!(auth
            .verify_and_accept_persistent(&mut reopened, 1, 8, &frame)
            .is_err());
        let rotated = auth
            .seal(verified_identity.peer_id.clone(), 1, 124, b"rotated".to_vec())
            .unwrap();
        auth.verify_and_accept_persistent(&mut reopened, 2, 8, &rotated)
            .unwrap();
        assert_eq!(
            reopened.get(&verified_identity.peer_id).unwrap().key_epoch,
            2
        );
        drop(reopened);
        let reopened = ReplayStore::open(&replay_path).unwrap();
        assert_eq!(
            reopened.get(&verified_identity.peer_id).unwrap().key_epoch,
            2
        );
        let _ = std::fs::remove_file(replay_path);
    }

    #[test]
    fn changing_signed_or_proven_payload_breaks_the_chain() {
        let identity = IdentityKey::from_seed([31; 32]);
        let signed = identity.sign(b"canonical-v1");
        assert!(verify_signed_bytes(&signed).is_ok());

        let mut tampered = signed.clone();
        tampered.payload = b"canonical-v2".to_vec();
        assert!(verify_signed_bytes(&tampered).is_err());

        let leaves = vec![b"canonical-v1".to_vec(), b"other".to_vec()];
        let root = state_root(&leaves);
        let mut proof = build_proof(&leaves, 0).unwrap();
        proof.leaf[0] ^= 1;
        assert!(!verify_proof(root, &proof).unwrap());
    }

    #[test]
    fn local_write_syncs_and_persists_peer_cursor() {
        let base = std::env::temp_dir().join(format!("mwdb-integration-{}", uuid::Uuid::new_v4()));
        let a_dir = base.join("a");
        let b_dir = base.join("b");
        let cursor_path = base.join("peers.json");
        std::fs::create_dir_all(&base).unwrap();

        let mut a = LocalFirstStore::open(&a_dir, "node-a").unwrap();
        let mut b = LocalFirstStore::open(&b_dir, "node-b").unwrap();
        let mut seen_b = HashSet::new();

        let change_id = a.set("doc:1", serde_json::json!({"value": 42})).unwrap();
        let batch = select_since(a.queue(), None).unwrap();
        assert_eq!(batch.changes.len(), 1);
        assert_eq!(batch.changes[0].change_id, change_id);

        let ack = {
            let ids = batch.changes.iter().map(|change| change.change_id).collect();
            make_ack(ids, Vec::new())
        };
        apply_batch(&mut b, &mut seen_b, &batch).unwrap();
        assert_eq!(b.get("doc:1"), Some(&serde_json::json!({"value": 42})));

        let checkpoint = a.queue().logical_changes().last().unwrap();
        let mut cursors = CursorStore::open(&cursor_path).unwrap();
        cursors
            .upsert(PeerCursor {
                peer_id: "node-b".into(),
                last_change_id: ack.checkpoint,
                last_clock: checkpoint.logical_clock,
            })
            .unwrap();

        let reopened = CursorStore::open(&cursor_path).unwrap();
        let saved = reopened.get("node-b").unwrap();
        assert_eq!(saved.last_change_id, Some(change_id));
        assert_eq!(saved.last_clock, checkpoint.logical_clock);

        let _ = std::fs::remove_dir_all(base);
    }
}
