use std::collections::HashSet;

use mwdb_local_first::{
    ChangeEnvelope, ChangeQueue, ChangeStatus, LocalFirstStore, LogicalChangeV1, LocalFirstError,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("invalid change: {0}")]
    InvalidChange(String),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("local-first error: {0}")]
    LocalFirst(#[from] LocalFirstError),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncHello {
    pub protocol: u16,
    pub node_id: String,
    pub last_change_id: Option<Uuid>,
    pub last_clock: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncBatch {
    pub protocol: u16,
    pub changes: Vec<LogicalChangeV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncAck {
    pub protocol: u16,
    pub accepted: Vec<Uuid>,
    pub rejected: Vec<Uuid>,
    pub checkpoint: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ApplyResult {
    Applied,
    Duplicate,
    Rejected(String),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConflictKind {
    CausallyOrdered,
    Mergeable,
    ConcurrentSameObject,
}

pub fn make_hello(node_id: impl Into<String>, changes: &[ChangeEnvelope]) -> SyncHello {
    SyncHello {
        protocol: 1,
        node_id: node_id.into(),
        last_change_id: changes.last().map(|c| c.change_id),
        last_clock: changes.len() as u64,
    }
}

pub fn changes_since(
    queue: &ChangeQueue,
    checkpoint: Option<Uuid>,
) -> Result<Vec<LogicalChangeV1>, SyncError> {
    Ok(queue.logical_changes_since(checkpoint)?)
}

pub fn encode_batch(changes: &[LogicalChangeV1]) -> Result<Vec<u8>, SyncError> {
    serde_json::to_vec(&SyncBatch { protocol: 1, changes: changes.to_vec() }).map_err(Into::into)
}

pub fn decode_batch(bytes: &[u8]) -> Result<SyncBatch, SyncError> {
    let batch: SyncBatch = serde_json::from_slice(bytes)?;
    if batch.protocol != 1 {
        return Err(SyncError::InvalidChange(format!("unsupported protocol {}", batch.protocol)));
    }
    for change in &batch.changes {
        if !change.verify() {
            return Err(SyncError::InvalidChange(change.change_id.to_string()));
        }
    }
    Ok(batch)
}

pub fn make_ack(accepted: Vec<Uuid>, rejected: Vec<Uuid>) -> SyncAck {
    SyncAck {
        protocol: 1,
        checkpoint: accepted.last().copied(),
        accepted,
        rejected,
    }
}

pub fn apply_idempotently(
    seen: &mut HashSet<Uuid>,
    change: &LogicalChangeV1,
) -> ApplyResult {
    if !change.verify() {
        return ApplyResult::Rejected("content hash verification failed".into());
    }
    if !seen.insert(change.change_id) {
        return ApplyResult::Duplicate;
    }
    ApplyResult::Applied
}

pub fn apply_batch(
    store: &mut LocalFirstStore,
    batch: &SyncBatch,
) -> Result<SyncAck, SyncError> {
    if batch.protocol != 1 {
        return Err(SyncError::InvalidChange(format!("unsupported protocol {}", batch.protocol)));
    }

    let mut accepted = Vec::new();
    let mut rejected = Vec::new();
    for change in &batch.changes {
        if !change.verify() {
            rejected.push(change.change_id);
            continue;
        }
        if store.apply_remote(change)? {
            accepted.push(change.change_id);
        }
    }
    Ok(make_ack(accepted, rejected))
}

pub fn classify(a: &LogicalChangeV1, b: &LogicalChangeV1) -> ConflictKind {
    if a.parents.contains(&b.change_id) || b.parents.contains(&a.change_id) {
        ConflictKind::CausallyOrdered
    } else if a.object_id != b.object_id {
        ConflictKind::Mergeable
    } else {
        ConflictKind::ConcurrentSameObject
    }
}

pub fn mark_acknowledged(
    queue: &mut ChangeQueue,
    ack: &SyncAck,
) -> Result<(), LocalFirstError> {
    for id in &ack.accepted {
        queue.set_status(*id, ChangeStatus::Acknowledged)?;
    }
    for id in &ack.rejected {
        queue.set_status(*id, ChangeStatus::Rejected)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn sample_change(id: Uuid) -> LogicalChangeV1 {
        LogicalChangeV1::new(id, "node-a", "doc:1", "set", serde_json::json!({"v":1}), 1, 1, vec![])
    }

    #[test]
    fn batch_round_trip_and_hash_verification() {
        let change = sample_change(Uuid::from_u128(1));
        let bytes = encode_batch(&[change.clone()]).unwrap();
        let decoded = decode_batch(&bytes).unwrap();
        assert_eq!(decoded.changes, vec![change]);
    }

    #[test]
    fn tampered_batch_is_rejected() {
        let mut change = sample_change(Uuid::from_u128(2));
        change.payload = serde_json::json!({"v":999});
        let batch = SyncBatch { protocol: 1, changes: vec![change] };
        let bytes = serde_json::to_vec(&batch).unwrap();
        assert!(matches!(decode_batch(&bytes), Err(SyncError::InvalidChange(_))));
    }

    #[test]
    fn duplicate_changes_are_idempotent() {
        let mut seen = HashSet::new();
        let change = sample_change(Uuid::from_u128(3));
        assert_eq!(apply_idempotently(&mut seen, &change), ApplyResult::Applied);
        assert_eq!(apply_idempotently(&mut seen, &change), ApplyResult::Duplicate);
    }

    #[test]
    fn acknowledgement_updates_persistent_queue_state() {
        let dir = tempdir().unwrap();
        let mut queue = ChangeQueue::open(dir.path().join("changes.log")).unwrap();
        let envelope = ChangeEnvelope {
            change_id: Uuid::from_u128(4),
            actor_id: "node-a".into(),
            object_id: "doc:4".into(),
            operation: "set".into(),
            payload: serde_json::json!(true),
            created_at_ms: 1,
        };
        queue.enqueue(envelope.clone()).unwrap();
        let ack = make_ack(vec![envelope.change_id], vec![]);
        mark_acknowledged(&mut queue, &ack).unwrap();
        assert_eq!(queue.status(envelope.change_id), Some(&ChangeStatus::Acknowledged));
    }

    #[test]
    fn checkpoint_delta_excludes_already_seen_changes() {
        let dir = tempdir().unwrap();
        let mut queue = ChangeQueue::open(dir.path().join("changes.log")).unwrap();
        for i in 1..=3 {
            queue.enqueue(ChangeEnvelope {
                change_id: Uuid::from_u128(i),
                actor_id: "node-a".into(),
                object_id: format!("doc:{i}"),
                operation: "set".into(),
                payload: serde_json::json!(i),
                created_at_ms: i as u128,
            }).unwrap();
        }
        let delta = changes_since(&queue, Some(Uuid::from_u128(1))).unwrap();
        assert_eq!(delta.iter().map(|c| c.change_id).collect::<Vec<_>>(), vec![Uuid::from_u128(2), Uuid::from_u128(3)]);
    }

    #[test]
    fn two_replicas_converge_after_offline_divergence_and_retry() {
        let a_dir = tempdir().unwrap();
        let b_dir = tempdir().unwrap();
        let mut a = LocalFirstStore::open(a_dir.path(), "node-a").unwrap();
        let mut b = LocalFirstStore::open(b_dir.path(), "node-b").unwrap();

        a.set("doc:a", serde_json::json!({"v": 1})).unwrap();
        b.set("doc:b", serde_json::json!({"v": 2})).unwrap();

        let a_batch = SyncBatch { protocol: 1, changes: changes_since(a.queue(), None).unwrap() };
        let b_batch = SyncBatch { protocol: 1, changes: changes_since(b.queue(), None).unwrap() };

        let a_to_b = apply_batch(&mut b, &a_batch).unwrap();
        let b_to_a = apply_batch(&mut a, &b_batch).unwrap();
        assert_eq!(a_to_b.accepted.len(), 1);
        assert_eq!(b_to_a.accepted.len(), 1);

        let retry = apply_batch(&mut b, &a_batch).unwrap();
        assert!(retry.accepted.is_empty());
        assert_eq!(b.get("doc:a"), Some(&serde_json::json!({"v": 1})));
        assert_eq!(a.get("doc:b"), Some(&serde_json::json!({"v": 2})));
    }

    #[test]
    fn conflict_classifier_is_conservative() {
        let a = sample_change(Uuid::from_u128(10));
        let b = LogicalChangeV1::new(Uuid::from_u128(11), "node-b", "doc:2", "set", serde_json::json!(2), 2, 1, vec![]);
        assert_eq!(classify(&a, &b), ConflictKind::Mergeable);

        let c = LogicalChangeV1::new(Uuid::from_u128(12), "node-b", "doc:1", "set", serde_json::json!(2), 2, 1, vec![]);
        assert_eq!(classify(&a, &c), ConflictKind::ConcurrentSameObject);

        let d = LogicalChangeV1::new(Uuid::from_u128(13), "node-a", "doc:1", "set", serde_json::json!(3), 3, 2, vec![a.change_id]);
        assert_eq!(classify(&a, &d), ConflictKind::CausallyOrdered);
    }
}
