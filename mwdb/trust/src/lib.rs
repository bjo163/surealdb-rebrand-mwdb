use std::{collections::BTreeMap, path::{Path, PathBuf}};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrustStatus { Active, Revoked }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrustedPeer {
    pub peer_id: String,
    pub public_key: [u8; 32],
    pub status: TrustStatus,
    pub label: Option<String>,
}

#[derive(Debug, Error)]
pub enum TrustError {
    #[error("io error: {0}")] Io(#[from] std::io::Error),
    #[error("serialization error: {0}")] Serialization(#[from] serde_json::Error),
    #[error("peer id is empty")] EmptyPeerId,
    #[error("peer key does not match peer id")]
    PeerIdMismatch,
}

#[derive(Debug)]
pub struct TrustStore {
    path: PathBuf,
    peers: BTreeMap<String, TrustedPeer>,
}

impl TrustStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, TrustError> {
        let path = path.as_ref().to_path_buf();
        let peers = if path.exists() {
            let bytes = std::fs::read(&path)?;
            if bytes.is_empty() { BTreeMap::new() } else { serde_json::from_slice(&bytes)? }
        } else { BTreeMap::new() };
        Ok(Self { path, peers })
    }

    pub fn trust(&mut self, peer_id: impl Into<String>, public_key: [u8; 32], label: Option<String>) -> Result<(), TrustError> {
        let peer_id = peer_id.into();
        if peer_id.is_empty() { return Err(TrustError::EmptyPeerId); }
        let expected = format!("ed25519-{}", hex::encode(&public_key[..8]));
        if peer_id != expected { return Err(TrustError::PeerIdMismatch); }
        self.peers.insert(peer_id.clone(), TrustedPeer { peer_id, public_key, status: TrustStatus::Active, label });
        self.persist()
    }

    pub fn revoke(&mut self, peer_id: &str) -> Result<bool, TrustError> {
        let Some(peer) = self.peers.get_mut(peer_id) else { return Ok(false); };
        peer.status = TrustStatus::Revoked;
        self.persist()?;
        Ok(true)
    }

    pub fn get(&self, peer_id: &str) -> Option<&TrustedPeer> { self.peers.get(peer_id) }

    pub fn is_trusted(&self, peer_id: &str, public_key: &[u8; 32]) -> bool {
        matches!(self.get(peer_id), Some(peer) if peer.status == TrustStatus::Active && &peer.public_key == public_key)
    }

    pub fn all(&self) -> impl Iterator<Item = &TrustedPeer> { self.peers.values() }

    fn persist(&self) -> Result<(), TrustError> {
        if let Some(parent) = self.path.parent() { std::fs::create_dir_all(parent)?; }
        let tmp = self.path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_vec_pretty(&self.peers)?)?;
        std::fs::rename(tmp, &self.path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn peer() -> ([u8; 32], String) {
        let key = [7u8; 32];
        (key, format!("ed25519-{}", hex::encode(&key[..8])))
    }

    #[test]
    fn trust_survives_restart_and_revoke_is_persistent() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("trust.json");
        let (key, id) = peer();
        { let mut s = TrustStore::open(&path).unwrap(); s.trust(id.clone(), key, Some("test".into())).unwrap(); }
        let mut s = TrustStore::open(&path).unwrap();
        assert!(s.is_trusted(&id, &key));
        s.revoke(&id).unwrap();
        drop(s);
        let s = TrustStore::open(&path).unwrap();
        assert!(!s.is_trusted(&id, &key));
    }

    #[test]
    fn mismatched_peer_id_is_rejected() {
        let dir = tempdir().unwrap();
        let mut s = TrustStore::open(dir.path().join("trust.json")).unwrap();
        assert!(matches!(
            s.trust("ed25519-bad", [7; 32], None),
            Err(TrustError::PeerIdMismatch)
        ));
    }
}
