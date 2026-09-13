use std::collections::HashSet;

use mwdb_local_first::{ChangeEnvelope, LocalFirstStore};
use mwdb_sync::{apply_batch, select_since, FaultHarness, SyncBatch, SYNC_PROTOCOL_V1};
use tempfile::tempdir;
use uuid::Uuid;

#[test]
fn eventual_delivery_recovers_after_drop_duplicate_and_reorder() {
    let a_dir = tempdir().unwrap();
    let b_dir = tempdir().unwrap();
    let mut a = LocalFirstStore::open(a_dir.path(), "a").unwrap();
    let mut b = LocalFirstStore::open(b_dir.path(), "b").unwrap();
    let mut seen_b = HashSet::new();

    a.set("doc:a", serde_json::json!({"v": 1})).unwrap();
    a.set("doc:b", serde_json::json!({"v": 2})).unwrap();

    let mut batches = select_since(a.queue(), None)
        .unwrap()
        .changes
        .into_iter()
        .map(|change| SyncBatch {
            protocol: SYNC_PROTOCOL_V1,
            changes: vec![change],
        })
        .collect::<Vec<_>>();

    let mut harness = FaultHarness::default();
    let damaged = harness.deliver(&mut batches, true, true, true);
    for batch in &damaged {
        apply_batch(&mut b, &mut seen_b, batch).unwrap();
    }

    // The dropped first batch is retried from the authoritative source.
    apply_batch(&mut b, &mut seen_b, &batches[0]).unwrap();
    // Replaying every source batch is safe and exercises duplicate suppression.
    for batch in &batches {
        apply_batch(&mut b, &mut seen_b, batch).unwrap();
    }

    assert_eq!(b.get("doc:a"), Some(&serde_json::json!({"v": 1})));
    assert_eq!(b.get("doc:b"), Some(&serde_json::json!({"v": 2})));
    assert_eq!(harness.dropped_batches, 1);
    assert_eq!(harness.duplicated_batches, 1);
    assert_eq!(harness.reordered_batches, 1);
}

#[test]
fn rejected_change_can_be_retried_without_poisoning_seen_set() {
    let a_dir = tempdir().unwrap();
    let b_dir = tempdir().unwrap();
    let mut a = LocalFirstStore::open(a_dir.path(), "a").unwrap();
    let mut b = LocalFirstStore::open(b_dir.path(), "b").unwrap();
    let mut seen_b = HashSet::new();

    let id = Uuid::from_u128(9001);
    a.queue_mut()
        .enqueue(ChangeEnvelope {
            change_id: id,
            actor_id: "a".into(),
            object_id: "doc:retry".into(),
            operation: "unsupported-op".into(),
            payload: serde_json::json!(true),
            created_at_ms: 1,
        })
        .unwrap();

    let batch = select_since(a.queue(), None).unwrap();
    let ack = apply_batch(&mut b, &mut seen_b, &batch);
    assert!(ack.is_err());
    assert!(!seen_b.contains(&id));

    let retry = apply_batch(&mut b, &mut seen_b, &batch);
    assert!(retry.is_err());
    assert!(!seen_b.contains(&id));
}
