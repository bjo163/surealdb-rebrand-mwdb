use std::{collections::BTreeMap, path::{Path, PathBuf}};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum CursorError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeerCursor {
    pub peer_id: String,
    pub last_change_id: Option<Uuid>,
    pub last_clock: u64,
}

#[derive(Debug)]
pub struct CursorStore {
    path: PathBuf,
    cursors: BTreeMap<String, PeerCursor>,
}

impl CursorStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, CursorError> {
        let path = path.as_ref().to_path_buf();
        let cursors = if path.exists() {
            let bytes = std::fs::read(&path)?;
            if bytes.is_empty() {
                BTreeMap::new()
            } else {
                serde_json::from_slice(&bytes)?
            }
        } else {
            BTreeMap::new()
        };
        Ok(Self { path, cursors })
    }

    pub fn get(&self, peer_id: &str) -> Option<&PeerCursor> {
        self.cursors.get(peer_id)
    }

    pub fn upsert(&mut self, cursor: PeerCursor) -> Result<(), CursorError> {
        self.cursors.insert(cursor.peer_id.clone(), cursor);
        self.persist()
    }

    pub fn remove(&mut self, peer_id: &str) -> Result<Option<PeerCursor>, CursorError> {
        let removed = self.cursors.remove(peer_id);
        self.persist()?;
        Ok(removed)
    }

    pub fn all(&self) -> impl Iterator<Item = &PeerCursor> {
        self.cursors.values()
    }

    fn persist(&self) -> Result<(), CursorError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let tmp = self.path.with_extension("json.tmp");
        let bytes = serde_json::to_vec_pretty(&self.cursors)?;
        std::fs::write(&tmp, bytes)?;
        std::fs::rename(tmp, &self.path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn cursor_survives_restart() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("peers.json");
        let change_id = Uuid::from_u128(42);
        {
            let mut store = CursorStore::open(&path).unwrap();
            store.upsert(PeerCursor {
                peer_id: "peer-b".into(),
                last_change_id: Some(change_id),
                last_clock: 9,
            }).unwrap();
        }
        let reopened = CursorStore::open(&path).unwrap();
        assert_eq!(reopened.get("peer-b").unwrap().last_change_id, Some(change_id));
        assert_eq!(reopened.get("peer-b").unwrap().last_clock, 9);
    }

    #[test]
    fn malformed_cursor_file_is_rejected() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("peers.json");
        std::fs::write(&path, b"not-json").unwrap();
        assert!(CursorStore::open(&path).is_err());
    }

    #[test]
    fn remove_is_persistent_and_idempotent() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("peers.json");
        let mut store = CursorStore::open(&path).unwrap();
        store.upsert(PeerCursor {
            peer_id: "peer-b".into(),
            last_change_id: None,
            last_clock: 0,
        }).unwrap();
        assert!(store.remove("peer-b").unwrap().is_some());
        assert!(store.remove("peer-b").unwrap().is_none());
        let reopened = CursorStore::open(&path).unwrap();
        assert!(reopened.get("peer-b").is_none());
    }
}
