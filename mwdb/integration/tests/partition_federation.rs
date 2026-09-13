use std::collections::HashSet;

use mwdb_auth::{AuthenticatedFrame, AuthError, ReplayStore, ReplayStoreError, SharedKeyAuthenticator};
use mwdb_cursor::{CursorStore, PeerCursor};
use mwdb_local_first::LocalFirstStore;
use mwdb_merkle::state_root;
use mwdb_sync::{apply_batch, encode_batch, select_since};
use uuid::Uuid;

fn deliver(
    source_id: &str,
    source: &LocalFirstStore,
    target: &mut LocalFirstStore,
    seen_target: &mut HashSet<Uuid>,
    replay_target: &mut ReplayStore,
    cursors_target: &mut CursorStore,
    auth: &SharedKeyAuthenticator,
    key_epoch: u64,
    nonce: u64,
) -> AuthenticatedFrame {
    let previous_checkpoint = cursors_target
        .get(source_id)
        .and_then(|cursor| cursor.last_change_id);
    let batch = select_since(source.queue(), previous_checkpoint).unwrap();
    let encoded = encode_batch(&batch).unwrap();
    let frame = auth
        .seal(source_id, nonce, nonce as u128, encoded)
        .unwrap();

    auth.verify_and_accept_persistent(replay_target, key_epoch, 64, &frame)
        .unwrap();
    let ack = apply_batch(target, seen_target, &batch).unwrap();
    let source_checkpoint = source.queue().checkpoint();
    cursors_target
        .upsert(PeerCursor {
            peer_id: source_id.to_owned(),
            last_change_id: ack.checkpoint.or(previous_checkpoint),
            last_clock: source_checkpoint.logical_clock,
        })
        .unwrap();
    frame
}

fn deterministic_state_root(store: &LocalFirstStore, keys: &[&str]) -> [u8; 32] {
    let leaves = keys
        .iter()
        .map(|key| {
            serde_json::to_vec(&serde_json::json!({
                "key": key,
                "value": store.get(key).cloned().unwrap_or(serde_json::Value::Null)
            }))
            .unwrap()
        })
        .collect::<Vec<_>>();
    state_root(&leaves)
}

#[test]
fn three_replica_partition_restart_and_rejoin_converges() {
    let base = std::env::temp_dir().join(format!("mwdb-federation-{}", Uuid::new_v4()));
    let a_dir = base.join("a");
    let b_dir = base.join("b");
    let c_dir = base.join("c");
    std::fs::create_dir_all(&base).unwrap();

    let mut a = LocalFirstStore::open(&a_dir, "node-a").unwrap();
    let mut b = LocalFirstStore::open(&b_dir, "node-b").unwrap();
    let mut c = LocalFirstStore::open(&c_dir, "node-c").unwrap();

    let auth = SharedKeyAuthenticator::new([41; 32]);
    let mut replay_a = ReplayStore::open(base.join("replay-a.json")).unwrap();
    let mut replay_b = ReplayStore::open(base.join("replay-b.json")).unwrap();
    let mut replay_c = ReplayStore::open(base.join("replay-c.json")).unwrap();
    let mut cursors_a = CursorStore::open(base.join("cursor-a.json")).unwrap();
    let mut cursors_b = CursorStore::open(base.join("cursor-b.json")).unwrap();
    let mut cursors_c = CursorStore::open(base.join("cursor-c.json")).unwrap();
    let mut seen_a = HashSet::new();
    let mut seen_b = HashSet::new();
    let mut seen_c = HashSet::new();

    // Baseline federation before the partition.
    a.set("seed", serde_json::json!({"v": 0})).unwrap();
    let baseline_to_b = deliver(
        "node-a",
        &a,
        &mut b,
        &mut seen_b,
        &mut replay_b,
        &mut cursors_b,
        &auth,
        1,
        1,
    );
    deliver(
        "node-a",
        &a,
        &mut c,
        &mut seen_c,
        &mut replay_c,
        &mut cursors_c,
        &auth,
        1,
        1,
    );

    // B is now isolated. All three sides continue to accept local work.
    a.set("doc:a", serde_json::json!({"owner": "a", "v": 1}))
        .unwrap();
    b.set("doc:b", serde_json::json!({"owner": "b", "v": 2}))
        .unwrap();
    c.set("doc:c", serde_json::json!({"owner": "c", "v": 3}))
        .unwrap();

    // A and C remain connected while B is partitioned.
    deliver(
        "node-a",
        &a,
        &mut c,
        &mut seen_c,
        &mut replay_c,
        &mut cursors_c,
        &auth,
        1,
        2,
    );
    deliver(
        "node-c",
        &c,
        &mut a,
        &mut seen_a,
        &mut replay_a,
        &mut cursors_a,
        &auth,
        1,
        1,
    );

    // Restart the isolated replica and prove cursor + replay state survived.
    drop(b);
    drop(replay_b);
    drop(cursors_b);
    let mut b = LocalFirstStore::open(&b_dir, "node-b").unwrap();
    let mut replay_b = ReplayStore::open(base.join("replay-b.json")).unwrap();
    let mut cursors_b = CursorStore::open(base.join("cursor-b.json")).unwrap();
    seen_b = HashSet::new();
    assert!(cursors_b.get("node-a").unwrap().last_change_id.is_some());
    assert!(matches!(
        auth.verify_and_accept_persistent(&mut replay_b, 1, 64, &baseline_to_b),
        Err(ReplayStoreError::Auth(AuthError::Replay))
    ));

    // A newer key epoch may restart its nonce sequence; an older epoch may not return.
    let rotated = auth
        .seal("node-a", 1, 100, b"key-epoch-2".to_vec())
        .unwrap();
    auth.verify_and_accept_persistent(&mut replay_b, 2, 64, &rotated)
        .unwrap();
    let stale_epoch = auth
        .seal("node-a", 99, 101, b"stale-key-epoch".to_vec())
        .unwrap();
    assert!(matches!(
        auth.verify_and_accept_persistent(&mut replay_b, 1, 64, &stale_epoch),
        Err(ReplayStoreError::StaleKeyEpoch {
            current: 2,
            requested: 1
        })
    ));

    // Heal the partition. Cursor-based incremental delivery brings B up to date.
    deliver(
        "node-a",
        &a,
        &mut b,
        &mut seen_b,
        &mut replay_b,
        &mut cursors_b,
        &auth,
        2,
        2,
    );
    deliver(
        "node-c",
        &c,
        &mut b,
        &mut seen_b,
        &mut replay_b,
        &mut cursors_b,
        &auth,
        1,
        1,
    );

    // B now federates its offline write back into both connected replicas.
    let b_to_a = deliver(
        "node-b",
        &b,
        &mut a,
        &mut seen_a,
        &mut replay_a,
        &mut cursors_a,
        &auth,
        1,
        1,
    );
    deliver(
        "node-b",
        &b,
        &mut c,
        &mut seen_c,
        &mut replay_c,
        &mut cursors_c,
        &auth,
        1,
        1,
    );

    // Re-delivery of an already authenticated frame is rejected before mutation.
    assert!(matches!(
        auth.verify_and_accept_persistent(&mut replay_a, 1, 64, &b_to_a),
        Err(ReplayStoreError::Auth(AuthError::Replay))
    ));

    let keys = ["seed", "doc:a", "doc:b", "doc:c"];
    for key in keys {
        assert_eq!(a.get(key), b.get(key), "A/B diverged at {key}");
        assert_eq!(a.get(key), c.get(key), "A/C diverged at {key}");
    }

    let root_a = deterministic_state_root(&a, &keys);
    let root_b = deterministic_state_root(&b, &keys);
    let root_c = deterministic_state_root(&c, &keys);
    assert_eq!(root_a, root_b);
    assert_eq!(root_a, root_c);

    // Final restart proves the healed state is durable, not just in-memory convergence.
    drop(b);
    let b = LocalFirstStore::open(&b_dir, "node-b").unwrap();
    assert_eq!(deterministic_state_root(&b, &keys), root_a);

    let _ = std::fs::remove_dir_all(base);
}
