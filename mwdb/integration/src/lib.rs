#[cfg(test)]
mod tests {
    use mwdb_auth::{ReplayWindow, SharedKeyAuthenticator};
    use mwdb_canonical::canonicalize;
    use mwdb_identity::{verify_signed_bytes, IdentityKey};
    use mwdb_merkle::{build_proof, hash_leaf, state_root, verify_proof};
    use mwdb_signing::{sign, verify};

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
        assert_eq!(proof.leaf, hash_leaf(canonical.as_bytes()));
        assert!(verify_proof(root, &proof).unwrap());

        let auth = SharedKeyAuthenticator::new([23; 32]);
        let frame = auth
            .seal(verified_identity.peer_id.clone(), 7, 123, canonical.into_bytes())
            .unwrap();
        let mut replay = ReplayWindow::new(8);
        auth.verify_and_accept(&mut replay, &frame).unwrap();
        assert!(auth.verify_and_accept(&mut replay, &frame).is_err());
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
}
