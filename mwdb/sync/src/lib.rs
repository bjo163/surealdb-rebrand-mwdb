use std::collections::{BTreeMap, HashSet};

use mwdb_local_first::{
    ChangeEnvelope, ChangeQueue, ChangeStatus, LocalFirstError, LocalFirstStore, LogicalChangeV1,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

pub const SYNC_PROTOCOL_V1: u16 = 1;

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("invalid change: {0}")]
    InvalidChange(String),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("local-first error: {0}")]
    LocalFirst(#[from] LocalFirstError),
    #[error("checkpoint is not known by this replica: {0}")]
    UnknownCheckpoint(Uuid),
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
pub enum ConflictClass {
    CausallyOrdered,
    Mergeable,
    ConcurrentSameObject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictPolicy {
    ApplyOrdered,
    AutoMerge,
    SurfaceConflict,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct GCounter {
    counts: BTreeMap<String, u64>,
}

impl GCounter {
    pub fn increment(&mut self, actor_id: impl Into<String>, amount: u64) {
        let actor_id = actor_id.into();
        let current = self.counts.get(&actor_id).copied().unwrap_or(0);
        self.counts.insert(actor_id, current.saturating_add(amount));
    }

    pub fn value(&self) -> u64 {
        self.counts.values().copied().sum()
    }

    pub fn merge(&self, other: &Self) -> Self {
        let mut merged = self.clone();
        for (actor, value) in &other.counts {
            let current = merged.counts.get(actor).copied().unwrap_or(0);
            merged.counts.insert(actor.clone(), current.max(*value));
        }
        merged
    }

    pub fn actor_value(&self, actor_id: &str) -> u64 {
        self.counts.get(actor_id).copied().unwrap_or(0)
    }
}

pub fn make_hello(
    node_id: impl Into<String>,
    checkpoint: mwdb_local_first::ChangeCheckpoint,
) -> SyncHello {
    SyncHello {
        protocol: SYNC_PROTOCOL_V1,
        node_id: node_id.into(),
        last_change_id: checkpoint.last_change_id,
        last_clock: checkpoint.logical_clock,
    }
}

pub fn select_since(queue: &ChangeQueue, checkpoint: Option<Uuid>) -> Result<SyncBatch, SyncError> {
    let changes = queue.logical_changes_since(checkpoint)?;
    Ok(SyncBatch {
        protocol: SYNC_PROTOCOL_V1,
        changes,
    })
}

pub fn encode_batch(batch: &SyncBatch) -> Result<Vec<u8>, SyncError> {
    if batch.protocol != SYNC_PROTOCOL_V1 {
        return Err(SyncError::InvalidChange(format!(
            "unsupported protocol {}",
            batch.protocol
        )));
    }
    for change in &batch.changes {
        validate_change(change)?;
    }
    Ok(serde_json::to_vec(batch)?)
}

pub fn decode_batch(bytes: &[u8]) -> Result<SyncBatch, SyncError> {
    let batch: SyncBatch = serde_json::from_slice(bytes)?;
    if batch.protocol != SYNC_PROTOCOL_V1 {
        return Err(SyncError::InvalidChange(format!(
            "unsupported protocol {}",
            batch.protocol
        )));
    }
    for change in &batch.changes {
        validate_change(change)?;
    }
    Ok(batch)
}

fn validate_change(change: &LogicalChangeV1) -> Result<(), SyncError> {
    if change.schema_version != 1 || !change.verify() {
        return Err(SyncError::InvalidChange(change.change_id.to_string()));
    }
    Ok(())
}

pub fn make_ack(accepted: Vec<Uuid>, rejected: Vec<Uuid>) -> SyncAck {
    SyncAck {
        protocol: SYNC_PROTOCOL_V1,
        checkpoint: accepted.last().copied(),
        accepted,
        rejected,
    }
}

pub fn apply_idempotently(seen: &mut HashSet<Uuid>, change: &LogicalChangeV1) -> ApplyResult {
    if let Err(error) = validate_change(change) {
        return ApplyResult::Rejected(error.to_string());
    }
    if !seen.insert(change.change_id) {
        return ApplyResult::Duplicate;
    }
    ApplyResult::Applied
}

pub fn apply_batch(
    store: &mut LocalFirstStore,
    seen: &mut HashSet<Uuid>,
    batch: &SyncBatch,
) -> Result<SyncAck, SyncError> {
    if batch.protocol != SYNC_PROTOCOL_V1 {
        return Err(SyncError::InvalidChange(format!(
            "unsupported protocol {}",
            batch.protocol
        )));
    }

    let mut accepted = Vec::new();
    let mut rejected = Vec::new();
    for change in &batch.changes {
        match apply_idempotently(seen, change) {
            ApplyResult::Applied => match store.apply_remote(change) {
                Ok(true) | Ok(false) => accepted.push(change.change_id),
                Err(error) => {
                    seen.remove(&change.change_id);
                    rejected.push(change.change_id);
                    return Err(SyncError::LocalFirst(error));
                }
            },
            ApplyResult::Duplicate => accepted.push(change.change_id),
            ApplyResult::Rejected(_) => rejected.push(change.change_id),
        }
    }
    Ok(make_ack(accepted, rejected))
}

pub fn mark_acknowledged(queue: &mut ChangeQueue, ack: &SyncAck) -> Result<(), LocalFirstError> {
    for id in &ack.accepted {
        if queue.status(*id).is_some() {
            queue.set_status(*id, ChangeStatus::Acknowledged)?;
        }
    }
    for id in &ack.rejected {
        if queue.status(*id).is_some() {
            queue.set_status(*id, ChangeStatus::Rejected)?;
        }
    }
    Ok(())
}

pub fn classify(a: &LogicalChangeV1, b: &LogicalChangeV1) -> ConflictClass {
    if a.parents.contains(&b.change_id) || b.parents.contains(&a.change_id) {
        ConflictClass::CausallyOrdered
    } else if a.object_id != b.object_id {
        ConflictClass::Mergeable
    } else {
        ConflictClass::ConcurrentSameObject
    }
}

pub fn policy_for(class: ConflictClass) -> ConflictPolicy {
    match class {
        ConflictClass::CausallyOrdered => ConflictPolicy::ApplyOrdered,
        ConflictClass::Mergeable => ConflictPolicy::AutoMerge,
        ConflictClass::ConcurrentSameObject => ConflictPolicy::SurfaceConflict,
    }
}

#[derive(Debug, Default)]
pub struct FaultHarness {
    pub dropped_batches: usize,
    pub duplicated_batches: usize,
    pub reordered_batches: usize,
}

impl FaultHarness {
    pub fn deliver(
        &mut self,
        batches: &mut Vec<SyncBatch>,
        drop_first: bool,
        duplicate_first: bool,
        reverse: bool,
    ) -> Vec<SyncBatch> {
        let mut out = batches.clone();
        if drop_first && !out.is_empty() {
            out.remove(0);
            self.dropped_batches += 1;
        }
        if duplicate_first && !out.is_empty() {
            let first = out[0].clone();
            out.insert(0, first);
            self.duplicated_batches += 1;
        }
        if reverse {
            out.reverse();
            self.reordered_batches += 1;
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn sample_change(
        id: Uuid,
        object: &str,
        value: i64,
        clock: u64,
        parents: Vec<Uuid>,
    ) -> LogicalChangeV1 {
        LogicalChangeV1::new(
            id,
            "node-a",
            object,
            "set",
            serde_json::json!({"v": value}),
            clock as u128,
            clock,
            parents,
        )
    }

    #[test]
    fn batch_round_trip_and_hash_verification() {
        let change = sample_change(Uuid::from_u128(1), "doc:1", 1, 1, vec![]);
        let batch = SyncBatch {
            protocol: SYNC_PROTOCOL_V1,
            changes: vec![change.clone()],
        };
        let bytes = encode_batch(&batch).unwrap();
        assert_eq!(decode_batch(&bytes).unwrap().changes, vec![change]);
    }

    #[test]
    fn tampered_batch_is_rejected() {
        let mut change = sample_change(Uuid::from_u128(2), "doc:1", 1, 1, vec![]);
        change.payload = serde_json::json!({"v": 999});
        let batch = SyncBatch {
            protocol: SYNC_PROTOCOL_V1,
            changes: vec![change],
        };
        assert!(encode_batch(&batch).is_err());
    }

    #[test]
    fn duplicate_changes_are_idempotent() {
        let mut seen = HashSet::new();
        let change = sample_change(Uuid::from_u128(3), "doc:1", 1, 1, vec![]);
        assert_eq!(apply_idempotently(&mut seen, &change), ApplyResult::Applied);
        assert_eq!(apply_idempotently(&mut seen, &change), ApplyResult::Duplicate);
    }

    #[test]
    fn checkpoint_selection_is_incremental() {
        let dir = tempdir().unwrap();
        let mut queue = ChangeQueue::open(dir.path().join("changes.log")).unwrap();
        queue.enqueue(ChangeEnvelope { change_id: Uuid::from_u128(10), actor_id: "a".into(), object_id: "a".into(), operation: "set".into(), payload: serde_json::json!(1), created_at_ms: 1 }).unwrap();
        queue.enqueue(ChangeEnvelope { change_id: Uuid::from_u128(11), actor_id: "a".into(), object_id: "b".into(), operation: "set".into(), payload: serde_json::json!(2), created_at_ms: 2 }).unwrap();
        let batch = select_since(&queue, Some(Uuid::from_u128(10))).unwrap();
        assert_eq!(batch.changes.len(), 1);
        assert_eq!(batch.changes[0].change_id, Uuid::from_u128(11));
    }

    #[test]
    fn two_replicas_converge_after_offline_writes() {
        let a_dir = tempdir().unwrap();
        let b_dir = tempdir().unwrap();
        let mut a = LocalFirstStore::open(a_dir.path(), "a").unwrap();
        let mut b = LocalFirstStore::open(b_dir.path(), "b").unwrap();
        let mut seen_a = HashSet::new();
        let mut seen_b = HashSet::new();
        a.set("doc:a", serde_json::json!({"v": 1})).unwrap();
        b.set("doc:b", serde_json::json!({"v": 2})).unwrap();
        let batch_a = select_since(a.queue(), None).unwrap();
        let batch_b = select_since(b.queue(), None).unwrap();
        apply_batch(&mut b, &mut seen_b, &batch_a).unwrap();
        apply_batch(&mut a, &mut seen_a, &batch_b).unwrap();
        assert_eq!(a.get("doc:a"), b.get("doc:a"));
        assert_eq!(a.get("doc:b"), b.get("doc:b"));
        apply_batch(&mut b, &mut seen_b, &batch_a).unwrap();
        apply_batch(&mut a, &mut seen_a, &batch_b).unwrap();
        assert_eq!(b.get("doc:a"), Some(&serde_json::json!({"v": 1})));
    }

    #[test]
    fn fault_harness_models_drop_duplicate_and_reorder() {
        let a = sample_change(Uuid::from_u128(20), "a", 1, 1, vec![]);
        let b = sample_change(Uuid::from_u128(21), "b", 2, 2, vec![a.change_id]);
        let mut batches = vec![
            SyncBatch { protocol: SYNC_PROTOCOL_V1, changes: vec![a] },
            SyncBatch { protocol: SYNC_PROTOCOL_V1, changes: vec![b] },
        ];
        let mut harness = FaultHarness::default();
        let delivered = harness.deliver(&mut batches, true, true, true);
        assert_eq!(delivered.len(), 1);
        assert_eq!(harness.dropped_batches, 1);
        assert_eq!(harness.duplicated_batches, 1);
        assert_eq!(harness.reordered_batches, 1);
    }

    #[test]
    fn conflict_policy_is_explicit_and_conservative() {
        let a = sample_change(Uuid::from_u128(30), "doc:a", 1, 1, vec![]);
        let b = sample_change(Uuid::from_u128(31), "doc:b", 2, 1, vec![]);
        let c = sample_change(Uuid::from_u128(32), "doc:a", 3, 1, vec![]);
        let d = sample_change(Uuid::from_u128(33), "doc:a", 4, 2, vec![a.change_id]);
        assert_eq!(classify(&a, &b), ConflictClass::Mergeable);
        assert_eq!(policy_for(ConflictClass::Mergeable), ConflictPolicy::AutoMerge);
        assert_eq!(classify(&a, &c), ConflictClass::ConcurrentSameObject);
        assert_eq!(policy_for(ConflictClass::ConcurrentSameObject), ConflictPolicy::SurfaceConflict);
        assert_eq!(classify(&a, &d), ConflictClass::CausallyOrdered);
        assert_eq!(policy_for(ConflictClass::CausallyOrdered), ConflictPolicy::ApplyOrdered);
    }

    #[test]
    fn g_counter_merge_is_idempotent_commutative_and_associative() {
        let mut a = GCounter::default();
        a.increment("a", 2);
        let mut b = GCounter::default();
        b.increment("b", 3);
        let mut c = GCounter::default();
        c.increment("c", 5);
        assert_eq!(a.merge(&a), a);
        assert_eq!(a.merge(&b), b.merge(&a));
        assert_eq!(a.merge(&b).merge(&c), a.merge(&b.merge(&c)));
        assert_eq!(a.merge(&b).value(), 5);
        assert_eq!(a.merge(&b).merge(&c).value(), 10);
    }

    #[test]
    fn g_counter_preserves_highest_per_actor_state() {
        let mut low = GCounter::default();
        low.increment("node-a", 2);
        let mut high = low.clone();
        high.increment("node-a", 5);
        let merged = low.merge(&high);
        assert_eq!(merged.actor_value("node-a"), 7);
        assert_eq!(merged, high);
    }

    #[test]
    fn acknowledgement_updates_persistent_queue_state() {
        let dir = tempdir().unwrap();
        let mut queue = ChangeQueue::open(dir.path().join("changes.log")).unwrap();
        let envelope = ChangeEnvelope { change_id: Uuid::from_u128(40), actor_id: "node-a".into(), object_id: "doc:4".into(), operation: "set".into(), payload: serde_json::json!(true), created_at_ms: 1 };
        queue.enqueue(envelope.clone()).unwrap();
        mark_acknowledged(&mut queue, &make_ack(vec![envelope.change_id], vec![])).unwrap();
        assert_eq!(queue.status(envelope.change_id), Some(&ChangeStatus::Acknowledged));
    }
}
