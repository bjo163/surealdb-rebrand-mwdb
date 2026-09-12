use mwdb_local_first::{ChangeEnvelope, ChangeStatus, LogicalChangeV1};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("invalid change: {0}")]
    InvalidChange(String),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
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

pub fn make_hello(node_id: impl Into<String>, changes: &[ChangeEnvelope]) -> SyncHello {
    SyncHello {
        protocol: 1,
        node_id: node_id.into(),
        last_change_id: changes.last().map(|c| c.change_id),
        last_clock: changes.len() as u64,
    }
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
    seen: &mut std::collections::HashSet<Uuid>,
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

pub fn mark_acknowledged(
    queue: &mut mwdb_local_first::ChangeQueue,
    ack: &SyncAck,
) -> Result<(), mwdb_local_first::LocalFirstError> {
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
        let mut seen = std::collections::HashSet::new();
        let change = sample_change(Uuid::from_u128(3));
        assert_eq!(apply_idempotently(&mut seen, &change), ApplyResult::Applied);
        assert_eq!(apply_idempotently(&mut seen, &change), ApplyResult::Duplicate);
    }

    #[test]
    fn acknowledgement_updates_persistent_queue_state() {
        let dir = tempdir().unwrap();
        let mut queue = mwdb_local_first::ChangeQueue::open(dir.path().join("changes.log")).unwrap();
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
}
