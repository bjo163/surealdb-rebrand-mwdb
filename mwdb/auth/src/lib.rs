use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const AUTH_FRAME_V1: u16 = 1;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AuthError {
    #[error("invalid frame version")]
    InvalidVersion,
    #[error("authentication tag mismatch")]
    InvalidTag,
    #[error("empty node id")]
    EmptyNodeId,
    #[error("replayed nonce")]
    Replay,
    #[error("nonce is outside replay window")]
    StaleNonce,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticatedFrame {
    pub version: u16,
    pub node_id: String,
    pub nonce: u64,
    pub issued_at_ms: u128,
    pub payload: Vec<u8>,
    pub tag: [u8; 32],
}

pub struct SharedKeyAuthenticator {
    key: [u8; 32],
}

impl SharedKeyAuthenticator {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    pub fn seal(
        &self,
        node_id: impl Into<String>,
        nonce: u64,
        issued_at_ms: u128,
        payload: Vec<u8>,
    ) -> Result<AuthenticatedFrame, AuthError> {
        let node_id = node_id.into();
        if node_id.is_empty() {
            return Err(AuthError::EmptyNodeId);
        }
        let tag = self.tag(AUTH_FRAME_V1, &node_id, nonce, issued_at_ms, &payload);
        Ok(AuthenticatedFrame {
            version: AUTH_FRAME_V1,
            node_id,
            nonce,
            issued_at_ms,
            payload,
            tag,
        })
    }

    pub fn verify(&self, frame: &AuthenticatedFrame) -> Result<(), AuthError> {
        if frame.version != AUTH_FRAME_V1 {
            return Err(AuthError::InvalidVersion);
        }
        let expected = self.tag(
            frame.version,
            &frame.node_id,
            frame.nonce,
            frame.issued_at_ms,
            &frame.payload,
        );
        if expected != frame.tag {
            return Err(AuthError::InvalidTag);
        }
        Ok(())
    }

    pub fn verify_and_accept(
        &self,
        window: &mut ReplayWindow,
        frame: &AuthenticatedFrame,
    ) -> Result<(), AuthError> {
        self.verify(frame)?;
        window.accept(frame.nonce)
    }

    pub fn verify_and_accept_persistent(
        &self,
        store: &mut ReplayStore,
        key_epoch: u64,
        replay_width: u64,
        frame: &AuthenticatedFrame,
    ) -> Result<(), ReplayStoreError> {
        self.verify(frame)?;
        store.accept(&frame.node_id, key_epoch, replay_width, frame.nonce)
    }

    fn tag(
        &self,
        version: u16,
        node_id: &str,
        nonce: u64,
        issued_at_ms: u128,
        payload: &[u8],
    ) -> [u8; 32] {
        let mut hasher = Hasher::new_keyed(&self.key);
        hasher.update(b"mwdb-auth-frame-v1\0");
        hasher.update(&version.to_be_bytes());
        hasher.update(&(node_id.len() as u64).to_be_bytes());
        hasher.update(node_id.as_bytes());
        hasher.update(&nonce.to_be_bytes());
        hasher.update(&issued_at_ms.to_be_bytes());
        hasher.update(&(payload.len() as u64).to_be_bytes());
        hasher.update(payload);
        *hasher.finalize().as_bytes()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplayWindowState {
    pub width: u64,
    pub highest: Option<u64>,
    pub seen: BTreeSet<u64>,
}

#[derive(Debug, Clone)]
pub struct ReplayWindow {
    width: u64,
    highest: Option<u64>,
    seen: BTreeSet<u64>,
}

impl ReplayWindow {
    pub fn new(width: u64) -> Self {
        assert!(width > 0, "replay window must be non-zero");
        Self {
            width,
            highest: None,
            seen: BTreeSet::new(),
        }
    }

    pub fn from_state(state: ReplayWindowState) -> Self {
        assert!(state.width > 0, "replay window must be non-zero");
        let mut window = Self {
            width: state.width,
            highest: state.highest,
            seen: state.seen,
        };
        window.prune();
        window
    }

    pub fn state(&self) -> ReplayWindowState {
        ReplayWindowState {
            width: self.width,
            highest: self.highest,
            seen: self.seen.clone(),
        }
    }

    /// Accept a nonce exactly once while it is inside the configured window.
    pub fn accept(&mut self, nonce: u64) -> Result<(), AuthError> {
        if self.seen.contains(&nonce) {
            return Err(AuthError::Replay);
        }
        if let Some(highest) = self.highest {
            let floor = highest.saturating_sub(self.width.saturating_sub(1));
            if nonce < floor {
                return Err(AuthError::StaleNonce);
            }
            if nonce > highest {
                self.highest = Some(nonce);
            }
        } else {
            self.highest = Some(nonce);
        }
        self.seen.insert(nonce);
        self.prune();
        Ok(())
    }

    pub fn highest(&self) -> Option<u64> {
        self.highest
    }

    fn prune(&mut self) {
        let Some(highest) = self.highest else {
            return;
        };
        let floor = highest.saturating_sub(self.width.saturating_sub(1));
        self.seen.retain(|nonce| *nonce >= floor);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeerReplayState {
    pub peer_id: String,
    pub key_epoch: u64,
    pub window: ReplayWindowState,
}

#[derive(Debug, Error)]
pub enum ReplayStoreError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error("empty peer id")]
    EmptyPeerId,
    #[error("invalid persisted peer replay record")]
    InvalidPeerRecord,
    #[error("stale key epoch {requested}; current epoch is {current}")]
    StaleKeyEpoch { current: u64, requested: u64 },
    #[error("replay width mismatch: stored {stored}, requested {requested}")]
    ReplayWidthMismatch { stored: u64, requested: u64 },
}

#[derive(Debug)]
pub struct ReplayStore {
    path: PathBuf,
    peers: BTreeMap<String, PeerReplayState>,
}

impl ReplayStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, ReplayStoreError> {
        let path = path.as_ref().to_path_buf();
        let peers: BTreeMap<String, PeerReplayState> = if path.exists() {
            let bytes = std::fs::read(&path)?;
            if bytes.is_empty() {
                BTreeMap::new()
            } else {
                serde_json::from_slice(&bytes)?
            }
        } else {
            BTreeMap::new()
        };

        if peers
            .iter()
            .any(|(peer_id, state)| peer_id.is_empty() || peer_id != &state.peer_id || state.window.width == 0)
        {
            return Err(ReplayStoreError::InvalidPeerRecord);
        }

        Ok(Self { path, peers })
    }

    pub fn get(&self, peer_id: &str) -> Option<&PeerReplayState> {
        self.peers.get(peer_id)
    }

    pub fn accept(
        &mut self,
        peer_id: &str,
        key_epoch: u64,
        replay_width: u64,
        nonce: u64,
    ) -> Result<(), ReplayStoreError> {
        if peer_id.is_empty() {
            return Err(ReplayStoreError::EmptyPeerId);
        }
        assert!(replay_width > 0, "replay window must be non-zero");

        let mut window = match self.peers.get(peer_id) {
            Some(state) if key_epoch < state.key_epoch => {
                return Err(ReplayStoreError::StaleKeyEpoch {
                    current: state.key_epoch,
                    requested: key_epoch,
                });
            }
            Some(state) if key_epoch == state.key_epoch => {
                if state.window.width != replay_width {
                    return Err(ReplayStoreError::ReplayWidthMismatch {
                        stored: state.window.width,
                        requested: replay_width,
                    });
                }
                ReplayWindow::from_state(state.window.clone())
            }
            _ => ReplayWindow::new(replay_width),
        };

        window.accept(nonce)?;
        let next = PeerReplayState {
            peer_id: peer_id.to_owned(),
            key_epoch,
            window: window.state(),
        };
        self.replace_and_persist(peer_id.to_owned(), next)
    }

    pub fn remove(&mut self, peer_id: &str) -> Result<Option<PeerReplayState>, ReplayStoreError> {
        let removed = self.peers.remove(peer_id);
        if let Err(error) = self.persist() {
            if let Some(state) = removed.clone() {
                self.peers.insert(peer_id.to_owned(), state);
            }
            return Err(error);
        }
        Ok(removed)
    }

    pub fn all(&self) -> impl Iterator<Item = &PeerReplayState> {
        self.peers.values()
    }

    fn replace_and_persist(
        &mut self,
        peer_id: String,
        state: PeerReplayState,
    ) -> Result<(), ReplayStoreError> {
        let previous = self.peers.insert(peer_id.clone(), state);
        if let Err(error) = self.persist() {
            match previous {
                Some(previous) => {
                    self.peers.insert(peer_id, previous);
                }
                None => {
                    self.peers.remove(&peer_id);
                }
            }
            return Err(error);
        }
        Ok(())
    }

    fn persist(&self) -> Result<(), ReplayStoreError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let tmp = self.path.with_extension("json.tmp");
        let bytes = serde_json::to_vec_pretty(&self.peers)?;
        std::fs::write(&tmp, bytes)?;
        std::fs::rename(tmp, &self.path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn auth() -> SharedKeyAuthenticator {
        SharedKeyAuthenticator::new([7; 32])
    }

    fn replay_path(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("mwdb-auth-{name}-{}-{unique}.json", std::process::id()))
    }

    #[test]
    fn seal_and_verify_round_trip() {
        let frame = auth().seal("node-a", 1, 42, b"hello".to_vec()).unwrap();
        assert!(auth().verify(&frame).is_ok());
    }

    #[test]
    fn tampering_is_rejected() {
        let mut frame = auth().seal("node-a", 1, 42, b"hello".to_vec()).unwrap();
        frame.payload[0] = b'H';
        assert_eq!(auth().verify(&frame), Err(AuthError::InvalidTag));
    }

    #[test]
    fn context_fields_are_authenticated() {
        let mut frame = auth().seal("node-a", 1, 42, b"hello".to_vec()).unwrap();
        frame.nonce = 2;
        assert_eq!(auth().verify(&frame), Err(AuthError::InvalidTag));
        let mut frame = auth().seal("node-a", 1, 42, b"hello".to_vec()).unwrap();
        frame.node_id = "node-b".into();
        assert_eq!(auth().verify(&frame), Err(AuthError::InvalidTag));
    }

    #[test]
    fn empty_node_is_rejected() {
        assert_eq!(auth().seal("", 1, 42, vec![]).unwrap_err(), AuthError::EmptyNodeId);
    }

    #[test]
    fn replay_window_rejects_duplicate_and_stale_nonces() {
        let mut window = ReplayWindow::new(4);
        assert!(window.accept(10).is_ok());
        assert_eq!(window.accept(10), Err(AuthError::Replay));
        assert!(window.accept(8).is_ok());
        assert_eq!(window.accept(6), Err(AuthError::StaleNonce));
        assert!(window.accept(14).is_ok());
        assert_eq!(window.highest(), Some(14));
    }

    #[test]
    fn replay_window_state_round_trip() {
        let mut window = ReplayWindow::new(8);
        window.accept(9).unwrap();
        window.accept(10).unwrap();
        let restored = ReplayWindow::from_state(window.state());
        assert_eq!(restored.highest(), Some(10));
        assert!(restored.seen.contains(&9));
    }

    #[test]
    fn authenticated_accept_combines_tag_and_replay_checks() {
        let mut window = ReplayWindow::new(8);
        let frame = auth().seal("node-a", 9, 42, b"hello".to_vec()).unwrap();
        assert!(auth().verify_and_accept(&mut window, &frame).is_ok());
        assert_eq!(auth().verify_and_accept(&mut window, &frame), Err(AuthError::Replay));
    }

    #[test]
    fn peer_replay_state_survives_restart() {
        let path = replay_path("restart");
        let frame = auth().seal("peer-a", 9, 42, b"hello".to_vec()).unwrap();

        {
            let mut store = ReplayStore::open(&path).unwrap();
            auth()
                .verify_and_accept_persistent(&mut store, 1, 8, &frame)
                .unwrap();
        }

        let mut reopened = ReplayStore::open(&path).unwrap();
        assert!(matches!(
            auth().verify_and_accept_persistent(&mut reopened, 1, 8, &frame),
            Err(ReplayStoreError::Auth(AuthError::Replay))
        ));
        assert_eq!(reopened.get("peer-a").unwrap().window.highest, Some(9));

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn replay_state_is_peer_scoped() {
        let path = replay_path("peers");
        let mut store = ReplayStore::open(&path).unwrap();
        store.accept("peer-a", 1, 8, 7).unwrap();
        store.accept("peer-b", 1, 8, 7).unwrap();
        assert_eq!(store.all().count(), 2);
        assert!(matches!(
            store.accept("peer-a", 1, 8, 7),
            Err(ReplayStoreError::Auth(AuthError::Replay))
        ));

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn newer_key_epoch_resets_window_and_old_epoch_is_rejected() {
        let path = replay_path("epoch");
        let mut store = ReplayStore::open(&path).unwrap();
        store.accept("peer-a", 4, 8, 100).unwrap();
        store.accept("peer-a", 5, 8, 1).unwrap();
        assert_eq!(store.get("peer-a").unwrap().key_epoch, 5);
        assert_eq!(store.get("peer-a").unwrap().window.highest, Some(1));
        assert!(matches!(
            store.accept("peer-a", 4, 8, 101),
            Err(ReplayStoreError::StaleKeyEpoch {
                current: 5,
                requested: 4
            })
        ));

        drop(store);
        let reopened = ReplayStore::open(&path).unwrap();
        assert_eq!(reopened.get("peer-a").unwrap().key_epoch, 5);

        let _ = std::fs::remove_file(path);
    }
}
