use std::collections::BTreeSet;

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
    pub fn new(key: [u8; 32]) -> Self { Self { key } }

    pub fn seal(&self, node_id: impl Into<String>, nonce: u64, issued_at_ms: u128, payload: Vec<u8>) -> Result<AuthenticatedFrame, AuthError> {
        let node_id = node_id.into();
        if node_id.is_empty() { return Err(AuthError::EmptyNodeId); }
        let tag = self.tag(AUTH_FRAME_V1, &node_id, nonce, issued_at_ms, &payload);
        Ok(AuthenticatedFrame { version: AUTH_FRAME_V1, node_id, nonce, issued_at_ms, payload, tag })
    }

    pub fn verify(&self, frame: &AuthenticatedFrame) -> Result<(), AuthError> {
        if frame.version != AUTH_FRAME_V1 { return Err(AuthError::InvalidVersion); }
        let expected = self.tag(frame.version, &frame.node_id, frame.nonce, frame.issued_at_ms, &frame.payload);
        if expected != frame.tag { return Err(AuthError::InvalidTag); }
        Ok(())
    }

    pub fn verify_and_accept(&self, window: &mut ReplayWindow, frame: &AuthenticatedFrame) -> Result<(), AuthError> {
        self.verify(frame)?;
        window.accept(frame.nonce)
    }

    fn tag(&self, version: u16, node_id: &str, nonce: u64, issued_at_ms: u128, payload: &[u8]) -> [u8; 32] {
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
        Self { width, highest: None, seen: BTreeSet::new() }
    }

    pub fn from_state(state: ReplayWindowState) -> Self {
        assert!(state.width > 0, "replay window must be non-zero");
        let mut window = Self { width: state.width, highest: state.highest, seen: state.seen };
        window.prune();
        window
    }

    pub fn state(&self) -> ReplayWindowState {
        ReplayWindowState { width: self.width, highest: self.highest, seen: self.seen.clone() }
    }

    /// Accept a nonce exactly once while it is inside the configured window.
    pub fn accept(&mut self, nonce: u64) -> Result<(), AuthError> {
        if self.seen.contains(&nonce) { return Err(AuthError::Replay); }
        if let Some(highest) = self.highest {
            let floor = highest.saturating_sub(self.width.saturating_sub(1));
            if nonce < floor { return Err(AuthError::StaleNonce); }
            if nonce > highest { self.highest = Some(nonce); }
        } else {
            self.highest = Some(nonce);
        }
        self.seen.insert(nonce);
        self.prune();
        Ok(())
    }

    pub fn highest(&self) -> Option<u64> { self.highest }

    fn prune(&mut self) {
        let Some(highest) = self.highest else { return; };
        let floor = highest.saturating_sub(self.width.saturating_sub(1));
        self.seen.retain(|nonce| *nonce >= floor);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn auth() -> SharedKeyAuthenticator { SharedKeyAuthenticator::new([7; 32]) }

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
        assert_eq!(restored.seen.contains(&9), true);
    }

    #[test]
    fn authenticated_accept_combines_tag_and_replay_checks() {
        let mut window = ReplayWindow::new(8);
        let frame = auth().seal("node-a", 9, 42, b"hello".to_vec()).unwrap();
        assert!(auth().verify_and_accept(&mut window, &frame).is_ok());
        assert_eq!(auth().verify_and_accept(&mut window, &frame), Err(AuthError::Replay));
    }
}
