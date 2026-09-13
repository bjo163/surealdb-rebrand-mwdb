use std::{
    collections::{HashMap, HashSet},
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, Sender},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum LocalFirstError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("change {0} does not exist")]
    UnknownChange(Uuid),
    #[error("unsupported operation: {0}")]
    UnsupportedOperation(String),
    #[error("invalid logical change: {0}")]
    InvalidLogicalChange(Uuid),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChangeStatus {
    Pending,
    Acknowledged,
    Rejected,
    Conflicted,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChangeEnvelope {
    pub change_id: Uuid,
    pub actor_id: String,
    pub object_id: String,
    pub operation: String,
    pub payload: serde_json::Value,
    pub created_at_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogicalChangeV1 {
    pub schema_version: u16,
    pub change_id: Uuid,
    pub actor_id: String,
    pub object_id: String,
    pub operation: String,
    pub payload: serde_json::Value,
    pub created_at_ms: u128,
    pub logical_clock: u64,
    pub parents: Vec<Uuid>,
    pub content_hash: String,
}

impl LogicalChangeV1 {
    pub fn new(
        change_id: Uuid,
        actor_id: impl Into<String>,
        object_id: impl Into<String>,
        operation: impl Into<String>,
        payload: serde_json::Value,
        created_at_ms: u128,
        logical_clock: u64,
        parents: Vec<Uuid>,
    ) -> Self {
        let mut change = Self {
            schema_version: 1,
            change_id,
            actor_id: actor_id.into(),
            object_id: object_id.into(),
            operation: operation.into(),
            payload,
            created_at_ms,
            logical_clock,
            parents,
            content_hash: String::new(),
        };
        change.content_hash = change.compute_hash();
        change
    }

    pub fn from_envelope(envelope: &ChangeEnvelope, logical_clock: u64, parents: Vec<Uuid>) -> Self {
        Self::new(
            envelope.change_id,
            envelope.actor_id.clone(),
            envelope.object_id.clone(),
            envelope.operation.clone(),
            envelope.payload.clone(),
            envelope.created_at_ms,
            logical_clock,
            parents,
        )
    }

    pub fn to_envelope(&self) -> ChangeEnvelope {
        ChangeEnvelope {
            change_id: self.change_id,
            actor_id: self.actor_id.clone(),
            object_id: self.object_id.clone(),
            operation: self.operation.clone(),
            payload: self.payload.clone(),
            created_at_ms: self.created_at_ms,
        }
    }

    pub fn compute_hash(&self) -> String {
        let canonical = serde_json::json!({
            "schema_version": self.schema_version,
            "change_id": self.change_id,
            "actor_id": self.actor_id,
            "object_id": self.object_id,
            "operation": self.operation,
            "payload": self.payload,
            "created_at_ms": self.created_at_ms,
            "logical_clock": self.logical_clock,
            "parents": self.parents,
        });
        blake3::hash(canonical.to_string().as_bytes()).to_hex().to_string()
    }

    pub fn verify(&self) -> bool {
        self.content_hash == self.compute_hash()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChangeCheckpoint {
    pub logical_clock: u64,
    pub last_change_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
enum QueueRecord {
    Change(ChangeEnvelope),
    LogicalChange(LogicalChangeV1),
    Status { change_id: Uuid, status: ChangeStatus },
}

#[derive(Debug)]
pub struct ChangeQueue {
    path: PathBuf,
    file: File,
    changes: Vec<ChangeEnvelope>,
    logical: Vec<LogicalChangeV1>,
    status: HashMap<Uuid, ChangeStatus>,
}

impl ChangeQueue {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, LocalFirstError> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let reader = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&path)?;
        let mut changes = Vec::new();
        let mut logical = Vec::new();
        let mut status = HashMap::new();

        for line in BufReader::new(reader).lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<QueueRecord>(&line)? {
                QueueRecord::Change(change) => {
                    if status.contains_key(&change.change_id) {
                        continue;
                    }
                    let logical_change = LogicalChangeV1::from_envelope(
                        &change,
                        logical.len() as u64 + 1,
                        logical
                            .last()
                            .map(|previous: &LogicalChangeV1| previous.change_id)
                            .into_iter()
                            .collect::<Vec<Uuid>>(),
                    );
                    status.insert(change.change_id, ChangeStatus::Pending);
                    changes.push(change);
                    logical.push(logical_change);
                }
                QueueRecord::LogicalChange(change) => {
                    if status.contains_key(&change.change_id) {
                        continue;
                    }
                    if change.schema_version != 1 || !change.verify() {
                        return Err(LocalFirstError::InvalidLogicalChange(change.change_id));
                    }
                    status.insert(change.change_id, ChangeStatus::Pending);
                    changes.push(change.to_envelope());
                    logical.push(change);
                }
                QueueRecord::Status { change_id, status: next } => {
                    status.insert(change_id, next);
                }
            }
        }

        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        Ok(Self { path, file, changes, logical, status })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn enqueue(&mut self, change: ChangeEnvelope) -> Result<bool, LocalFirstError> {
        if self.status.contains_key(&change.change_id) {
            return Ok(false);
        }
        let logical = LogicalChangeV1::from_envelope(
            &change,
            self.logical.len() as u64 + 1,
            self.logical
                .last()
                .map(|previous: &LogicalChangeV1| previous.change_id)
                .into_iter()
                .collect::<Vec<Uuid>>(),
        );
        self.enqueue_logical(logical)
    }

    pub fn enqueue_logical(&mut self, change: LogicalChangeV1) -> Result<bool, LocalFirstError> {
        if change.schema_version != 1 || !change.verify() {
            return Err(LocalFirstError::InvalidLogicalChange(change.change_id));
        }
        if self.status.contains_key(&change.change_id) {
            return Ok(false);
        }
        self.append(&QueueRecord::LogicalChange(change.clone()))?;
        self.status.insert(change.change_id, ChangeStatus::Pending);
        self.changes.push(change.to_envelope());
        self.logical.push(change);
        Ok(true)
    }

    pub fn set_status(&mut self, change_id: Uuid, next: ChangeStatus) -> Result<(), LocalFirstError> {
        if !self.status.contains_key(&change_id) {
            return Err(LocalFirstError::UnknownChange(change_id));
        }
        self.append(&QueueRecord::Status { change_id, status: next.clone() })?;
        self.status.insert(change_id, next);
        Ok(())
    }

    pub fn pending(&self) -> Vec<ChangeEnvelope> {
        self.changes
            .iter()
            .filter(|change| matches!(self.status.get(&change.change_id), Some(ChangeStatus::Pending)))
            .cloned()
            .collect()
    }

    pub fn status(&self, change_id: Uuid) -> Option<&ChangeStatus> {
        self.status.get(&change_id)
    }

    pub fn all(&self) -> &[ChangeEnvelope] {
        &self.changes
    }

    pub fn logical_changes(&self) -> Vec<LogicalChangeV1> {
        self.logical.clone()
    }

    pub fn logical_changes_since(&self, last_change_id: Option<Uuid>) -> Result<Vec<LogicalChangeV1>, LocalFirstError> {
        let start = match last_change_id {
            None => 0,
            Some(id) => self
                .logical
                .iter()
                .position(|change| change.change_id == id)
                .map(|index| index + 1)
                .ok_or(LocalFirstError::UnknownChange(id))?,
        };
        Ok(self.logical.iter().skip(start).cloned().collect())
    }

    pub fn checkpoint(&self) -> ChangeCheckpoint {
        ChangeCheckpoint {
            logical_clock: self.logical.last().map(|change| change.logical_clock).unwrap_or(0),
            last_change_id: self.logical.last().map(|change| change.change_id),
        }
    }

    pub fn export_logical_v1(&self) -> Result<String, LocalFirstError> {
        let mut output = String::new();
        for change in &self.logical {
            output.push_str(&serde_json::to_string(change)?);
            output.push('\n');
        }
        Ok(output)
    }

    pub fn verify_export(&self) -> Result<usize, LocalFirstError> {
        let exported = self.export_logical_v1()?;
        let mut verified = 0;
        for line in exported.lines() {
            let change: LogicalChangeV1 = serde_json::from_str(line)?;
            if change.verify() {
                verified += 1;
            }
        }
        Ok(verified)
    }

    fn append(&mut self, record: &QueueRecord) -> Result<(), LocalFirstError> {
        let encoded = serde_json::to_vec(record)?;
        self.file.write_all(&encoded)?;
        self.file.write_all(b"\n")?;
        self.file.sync_data()?;
        Ok(())
    }

    pub fn known_ids(&self) -> HashSet<Uuid> {
        self.status.keys().copied().collect()
    }
}

#[derive(Debug)]
pub struct LocalFirstStore {
    queue: ChangeQueue,
    state_path: PathBuf,
    state: serde_json::Map<String, serde_json::Value>,
    actor_id: String,
    subscribers: Vec<Sender<ChangeEnvelope>>,
}

impl LocalFirstStore {
    pub fn open(dir: impl AsRef<Path>, actor_id: impl Into<String>) -> Result<Self, LocalFirstError> {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir)?;
        let state_path = dir.join("state.json");
        let mut state = if state_path.exists() {
            let bytes = std::fs::read(&state_path)?;
            serde_json::from_slice::<serde_json::Map<String, serde_json::Value>>(&bytes)?
        } else {
            serde_json::Map::new()
        };
        let queue = ChangeQueue::open(dir.join("changes.log"))?;

        for change in queue.all() {
            if change.operation == "set" {
                state.insert(change.object_id.clone(), change.payload.clone());
            }
        }

        let store = Self {
            queue,
            state_path,
            state,
            actor_id: actor_id.into(),
            subscribers: Vec::new(),
        };
        store.persist_state()?;
        Ok(store)
    }

    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.state.get(key)
    }

    pub fn set(&mut self, key: impl Into<String>, value: serde_json::Value) -> Result<Uuid, LocalFirstError> {
        let key = key.into();
        let logical = LogicalChangeV1::new(
            Uuid::new_v4(),
            self.actor_id.clone(),
            key.clone(),
            "set",
            value.clone(),
            now_ms(),
            self.queue.logical.len() as u64 + 1,
            self.queue
                .logical
                .last()
                .map(|previous: &LogicalChangeV1| previous.change_id)
                .into_iter()
                .collect::<Vec<Uuid>>(),
        );
        let id = logical.change_id;
        self.queue.enqueue_logical(logical.clone())?;
        self.state.insert(key, value);
        self.persist_state()?;
        self.publish(logical.to_envelope());
        Ok(id)
    }

    pub fn apply_remote(&mut self, change: &LogicalChangeV1) -> Result<bool, LocalFirstError> {
        if change.schema_version != 1 || !change.verify() {
            return Err(LocalFirstError::InvalidLogicalChange(change.change_id));
        }
        if self.queue.known_ids().contains(&change.change_id) {
            return Ok(false);
        }
        if change.operation != "set" {
            return Err(LocalFirstError::UnsupportedOperation(change.operation.clone()));
        }
        self.queue.enqueue_logical(change.clone())?;
        self.queue.set_status(change.change_id, ChangeStatus::Acknowledged)?;
        self.state.insert(change.object_id.clone(), change.payload.clone());
        self.persist_state()?;
        self.publish(change.to_envelope());
        Ok(true)
    }

    pub fn subscribe(&mut self) -> Receiver<ChangeEnvelope> {
        let (tx, rx) = mpsc::channel();
        self.subscribers.push(tx);
        rx
    }

    fn publish(&mut self, change: ChangeEnvelope) {
        self.subscribers.retain(|subscriber| subscriber.send(change.clone()).is_ok());
    }

    pub fn queue(&self) -> &ChangeQueue {
        &self.queue
    }

    pub fn queue_mut(&mut self) -> &mut ChangeQueue {
        &mut self.queue
    }

    fn persist_state(&self) -> Result<(), LocalFirstError> {
        let tmp = self.state_path.with_extension("json.tmp");
        let bytes = serde_json::to_vec_pretty(&self.state)?;
        {
            let mut file = File::create(&tmp)?;
            file.write_all(&bytes)?;
            file.sync_data()?;
        }
        std::fs::rename(tmp, &self.state_path)?;
        Ok(())
    }
}

fn now_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn sample_envelope(id: Uuid, key: &str, value: serde_json::Value) -> ChangeEnvelope {
        ChangeEnvelope {
            change_id: id,
            actor_id: "a".into(),
            object_id: key.into(),
            operation: "set".into(),
            payload: value,
            created_at_ms: 1,
        }
    }

    #[test]
    fn offline_write_is_durable_and_pending_after_reopen() {
        let dir = tempdir().unwrap();
        let id;
        {
            let mut store = LocalFirstStore::open(dir.path(), "device-a").unwrap();
            id = store.set("user:1", serde_json::json!({"name":"Alice"})).unwrap();
            assert_eq!(store.get("user:1").unwrap()["name"], "Alice");
            assert_eq!(store.queue().pending().len(), 1);
        }
        let store = LocalFirstStore::open(dir.path(), "device-a").unwrap();
        assert_eq!(store.get("user:1").unwrap()["name"], "Alice");
        assert_eq!(store.queue().pending()[0].change_id, id);
    }

    #[test]
    fn acknowledgement_survives_reopen() {
        let dir = tempdir().unwrap();
        let id;
        {
            let mut store = LocalFirstStore::open(dir.path(), "device-a").unwrap();
            id = store.set("k", serde_json::json!(1)).unwrap();
            store.queue_mut().set_status(id, ChangeStatus::Acknowledged).unwrap();
        }
        let store = LocalFirstStore::open(dir.path(), "device-a").unwrap();
        assert!(store.queue().pending().is_empty());
        assert_eq!(store.queue().status(id), Some(&ChangeStatus::Acknowledged));
    }

    #[test]
    fn duplicate_change_id_is_not_reenqueued() {
        let dir = tempdir().unwrap();
        let mut queue = ChangeQueue::open(dir.path().join("changes.log")).unwrap();
        let id = Uuid::new_v4();
        let change = sample_envelope(id, "o", serde_json::json!(true));
        assert!(queue.enqueue(change.clone()).unwrap());
        assert!(!queue.enqueue(change).unwrap());
        assert_eq!(queue.all().len(), 1);
    }

    #[test]
    fn replay_repairs_state_snapshot() {
        let dir = tempdir().unwrap();
        let mut queue = ChangeQueue::open(dir.path().join("changes.log")).unwrap();
        let id = Uuid::new_v4();
        queue.enqueue(sample_envelope(id, "k", serde_json::json!(42))).unwrap();
        let store = LocalFirstStore::open(dir.path(), "a").unwrap();
        assert_eq!(store.get("k"), Some(&serde_json::json!(42)));
    }

    #[test]
    fn local_subscription_is_ordered() {
        let dir = tempdir().unwrap();
        let mut store = LocalFirstStore::open(dir.path(), "device-a").unwrap();
        let rx = store.subscribe();
        store.set("a", serde_json::json!(1)).unwrap();
        store.set("b", serde_json::json!(2)).unwrap();
        assert_eq!(rx.recv().unwrap().object_id, "a");
        assert_eq!(rx.recv().unwrap().object_id, "b");
    }

    #[test]
    fn remote_apply_is_durable_and_idempotent() {
        let source = tempdir().unwrap();
        let target = tempdir().unwrap();
        let mut a = LocalFirstStore::open(source.path(), "a").unwrap();
        let mut b = LocalFirstStore::open(target.path(), "b").unwrap();
        a.set("shared", serde_json::json!({"v":1})).unwrap();
        let change = a.queue().logical_changes()[0].clone();
        assert!(b.apply_remote(&change).unwrap());
        assert!(!b.apply_remote(&change).unwrap());
        assert_eq!(b.get("shared"), Some(&serde_json::json!({"v":1})));
        assert_eq!(b.queue().status(change.change_id), Some(&ChangeStatus::Acknowledged));
    }

    #[test]
    fn logical_change_hash_is_deterministic_and_verifiable() {
        let id = Uuid::nil();
        let a = LogicalChangeV1::new(id, "actor", "object", "set", serde_json::json!({"b":2,"a":1}), 10, 3, vec![]);
        let b = LogicalChangeV1::new(id, "actor", "object", "set", serde_json::json!({"b":2,"a":1}), 10, 3, vec![]);
        assert_eq!(a.content_hash, b.content_hash);
        assert!(a.verify());
    }

    #[test]
    fn export_is_replayable_and_checkpointed() {
        let dir = tempdir().unwrap();
        let mut queue = ChangeQueue::open(dir.path().join("changes.log")).unwrap();
        queue.enqueue(sample_envelope(Uuid::from_u128(1), "one", serde_json::json!(1))).unwrap();
        queue.enqueue(sample_envelope(Uuid::from_u128(2), "two", serde_json::json!(2))).unwrap();
        let export = queue.export_logical_v1().unwrap();
        let parsed = export.lines().map(|line| serde_json::from_str::<LogicalChangeV1>(line).unwrap()).collect::<Vec<_>>();
        assert_eq!(parsed.len(), 2);
        assert!(parsed.iter().all(LogicalChangeV1::verify));
        assert_eq!(queue.verify_export().unwrap(), 2);
        assert_eq!(queue.checkpoint().logical_clock, 2);
        assert_eq!(queue.logical_changes_since(Some(Uuid::from_u128(1))).unwrap().len(), 1);
    }
}
