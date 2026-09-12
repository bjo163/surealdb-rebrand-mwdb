use std::{collections::{HashMap, HashSet}, fs::{File, OpenOptions}, io::{BufRead, BufReader, Write}, path::{Path, PathBuf}};

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
enum QueueRecord {
    Change(ChangeEnvelope),
    Status { change_id: Uuid, status: ChangeStatus },
}

#[derive(Debug)]
pub struct ChangeQueue {
    path: PathBuf,
    file: File,
    changes: Vec<ChangeEnvelope>,
    status: HashMap<Uuid, ChangeStatus>,
}

impl ChangeQueue {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, LocalFirstError> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let reader = OpenOptions::new().create(true).read(true).open(&path)?;
        let mut changes = Vec::new();
        let mut status = HashMap::new();
        for line in BufReader::new(reader).lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<QueueRecord>(&line)? {
                QueueRecord::Change(change) => {
                    status.entry(change.change_id).or_insert(ChangeStatus::Pending);
                    changes.push(change);
                }
                QueueRecord::Status { change_id, status: next } => {
                    status.insert(change_id, next);
                }
            }
        }
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        Ok(Self { path, file, changes, status })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn enqueue(&mut self, change: ChangeEnvelope) -> Result<bool, LocalFirstError> {
        if self.status.contains_key(&change.change_id) {
            return Ok(false);
        }
        self.append(&QueueRecord::Change(change.clone()))?;
        self.status.insert(change.change_id, ChangeStatus::Pending);
        self.changes.push(change);
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
            .filter(|c| matches!(self.status.get(&c.change_id), Some(ChangeStatus::Pending)))
            .cloned()
            .collect()
    }

    pub fn status(&self, change_id: Uuid) -> Option<&ChangeStatus> {
        self.status.get(&change_id)
    }

    pub fn all(&self) -> &[ChangeEnvelope] {
        &self.changes
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

        // The logical change log is authoritative for local mutations. This replay
        // repairs a state snapshot that was written before a crash/restart boundary.
        for change in queue.all() {
            if change.operation == "set" {
                state.insert(change.object_id.clone(), change.payload.clone());
            }
        }

        let store = Self { queue, state_path, state, actor_id: actor_id.into() };
        store.persist_state()?;
        Ok(store)
    }

    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.state.get(key)
    }

    pub fn set(&mut self, key: impl Into<String>, value: serde_json::Value) -> Result<Uuid, LocalFirstError> {
        let key = key.into();
        let change = ChangeEnvelope {
            change_id: Uuid::new_v4(),
            actor_id: self.actor_id.clone(),
            object_id: key.clone(),
            operation: "set".into(),
            payload: value.clone(),
            created_at_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
        };

        // Journal first: once the write is acknowledged by this API, the logical
        // mutation is durable. The state snapshot can be reconstructed from it.
        let id = change.change_id;
        self.queue.enqueue(change)?;
        self.state.insert(key, value);
        self.persist_state()?;
        Ok(id)
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

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
        {
            let store = LocalFirstStore::open(dir.path(), "device-a").unwrap();
            assert_eq!(store.get("user:1").unwrap()["name"], "Alice");
            assert_eq!(store.queue().pending()[0].change_id, id);
        }
    }

    #[test]
    fn acknowledgement_survives_reopen() {
        let dir = tempdir().unwrap();
        let id;
        {
            let mut store = LocalFirstStore::open(dir.path(), "device-a").unwrap();
            id = store.set("k", serde_json::json!(1)).unwrap();
            store.queue_mut().set_status(id, ChangeStatus::Acknowledged).unwrap();
            assert!(store.queue().pending().is_empty());
        }
        let store = LocalFirstStore::open(dir.path(), "device-a").unwrap();
        assert!(store.queue().pending().is_empty());
        assert_eq!(store.queue().status(id), Some(&ChangeStatus::Acknowledged));
    }

    #[test]
    fn duplicate_change_id_is_not_reenqueued() {
        let dir = tempdir().unwrap();
        let mut q = ChangeQueue::open(dir.path().join("changes.log")).unwrap();
        let id = Uuid::new_v4();
        let c = ChangeEnvelope {
            change_id: id,
            actor_id: "a".into(),
            object_id: "o".into(),
            operation: "set".into(),
            payload: serde_json::json!(true),
            created_at_ms: 1,
        };
        assert!(q.enqueue(c.clone()).unwrap());
        assert!(!q.enqueue(c).unwrap());
        assert_eq!(q.all().len(), 1);
    }

    #[test]
    fn replay_repairs_state_snapshot() {
        let dir = tempdir().unwrap();
        let id;
        {
            let mut q = ChangeQueue::open(dir.path().join("changes.log")).unwrap();
            id = Uuid::new_v4();
            q.enqueue(ChangeEnvelope {
                change_id: id,
                actor_id: "a".into(),
                object_id: "k".into(),
                operation: "set".into(),
                payload: serde_json::json!(42),
                created_at_ms: 1,
            }).unwrap();
            // Intentionally no state.json: recovery must rebuild it from the journal.
        }
        let store = LocalFirstStore::open(dir.path(), "a").unwrap();
        assert_eq!(store.get("k"), Some(&serde_json::json!(42)));
        assert_eq!(store.queue().all()[0].change_id, id);
    }
}
