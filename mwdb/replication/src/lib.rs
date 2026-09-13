use std::{collections::{BTreeMap, BTreeSet}, path::{Path, PathBuf}, time::{SystemTime, UNIX_EPOCH}};

use mwdb_auth::{AuthenticatedFrame, ReplayWindow, ReplayWindowState, SharedKeyAuthenticator};
use mwdb_canonical::canonicalize;
use mwdb_cursor::{CursorError, CursorStore, PeerCursor};
use mwdb_identity::{verify_signed_bytes, IdentityKey, PeerIdentity, SignedBytes};
use mwdb_local_first::{LocalFirstError, LocalFirstStore};
use mwdb_merkle::state_root;
use mwdb_observability::{SyncEvent, SyncMetrics};
use mwdb_sync::{apply_batch, encode_batch, select_since, SyncAck, SyncBatch, SyncError};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

pub const REPLICATION_PROTOCOL_V1: u16 = 1;
const REPLAY_WINDOW: u64 = 64;

#[derive(Debug, Error)]
pub enum ReplicationError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("canonicalization error: {0}")]
    Canonical(#[from] mwdb_canonical::CanonicalError),
    #[error("authentication error: {0}")]
    Auth(#[from] mwdb_auth::AuthError),
    #[error("identity verification failed: {0}")]
    Identity(#[from] mwdb_identity::IdentityError),
    #[error("sync error: {0}")]
    Sync(#[from] SyncError),
    #[error("local-first error: {0}")]
    LocalFirst(#[from] LocalFirstError),
    #[error("cursor error: {0}")]
    Cursor(#[from] CursorError),
    #[error("frame node id does not match signed peer identity")]
    IdentityBinding,
    #[error("unsupported replication protocol {0}")]
    UnsupportedProtocol(u16),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct WireSignedEnvelope {
    protocol: u16,
    peer_id: String,
    public_key: Vec<u8>,
    payload: Vec<u8>,
    signature: Vec<u8>,
}

impl From<SignedBytes> for WireSignedEnvelope {
    fn from(value: SignedBytes) -> Self {
        Self { protocol: value.protocol, peer_id: value.peer_id, public_key: value.public_key.to_vec(), payload: value.payload, signature: value.signature.to_vec() }
    }
}

impl TryFrom<WireSignedEnvelope> for SignedBytes {
    type Error = ReplicationError;
    fn try_from(value: WireSignedEnvelope) -> Result<Self, Self::Error> {
        let public_key: [u8; 32] = value.public_key.try_into().map_err(|_| mwdb_identity::IdentityError::InvalidVerifyingKey)?;
        let signature: [u8; 64] = value.signature.try_into().map_err(|_| mwdb_identity::IdentityError::InvalidSignature)?;
        Ok(Self { protocol: value.protocol, peer_id: value.peer_id, public_key, payload: value.payload, signature })
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct DurableSessionState {
    replay: BTreeMap<String, ReplayWindowState>,
    applied: BTreeSet<Uuid>,
}

fn persist_state(path: &Path, state: &DurableSessionState) -> Result<(), ReplicationError> {
    if let Some(parent) = path.parent() { std::fs::create_dir_all(parent)?; }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_vec_pretty(state)?)?;
    std::fs::rename(tmp, path)?;
    Ok(())
}

fn load_state(path: &Path) -> Result<DurableSessionState, ReplicationError> {
    if !path.exists() { return Ok(DurableSessionState::default()); }
    let bytes = std::fs::read(path)?;
    if bytes.is_empty() { return Ok(DurableSessionState::default()); }
    Ok(serde_json::from_slice(&bytes)?)
}

fn now_ms() -> u128 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceivedBatch {
    pub peer: PeerIdentity,
    pub ack: SyncAck,
    pub batch_root: [u8; 32],
    pub metrics: SyncMetrics,
}

pub struct ReplicationSession {
    node_id: String,
    identity: IdentityKey,
    auth: SharedKeyAuthenticator,
    store: LocalFirstStore,
    cursors: CursorStore,
    state_path: PathBuf,
    state: DurableSessionState,
    metrics: SyncMetrics,
    next_nonce: u64,
}

impl ReplicationSession {
    pub fn open(root: impl AsRef<Path>, node_id: impl Into<String>, seed: [u8; 32], shared_key: [u8; 32]) -> Result<Self, ReplicationError> {
        let root = root.as_ref();
        std::fs::create_dir_all(root)?;
        let node_id = node_id.into();
        let identity = IdentityKey::from_seed(seed);
        let state_path = root.join("replication-state.json");
        let state = load_state(&state_path)?;
        Ok(Self {
            node_id: node_id.clone(),
            identity,
            auth: SharedKeyAuthenticator::new(shared_key),
            store: LocalFirstStore::open(root.join("local"), &node_id)?,
            cursors: CursorStore::open(root.join("cursors.json"))?,
            state_path,
            state,
            metrics: SyncMetrics::default(),
            next_nonce: 1,
        })
    }

    pub fn node_id(&self) -> &str { &self.node_id }
    pub fn identity(&self) -> &PeerIdentity { self.identity.identity() }
    pub fn store(&self) -> &LocalFirstStore { &self.store }
    pub fn store_mut(&mut self) -> &mut LocalFirstStore { &mut self.store }
    pub fn metrics(&self) -> &SyncMetrics { &self.metrics }
    pub fn cursor(&self, peer_id: &str) -> Option<&PeerCursor> { self.cursors.get(peer_id) }

    pub fn make_frame(&mut self, checkpoint: Option<Uuid>) -> Result<AuthenticatedFrame, ReplicationError> {
        let batch = select_since(self.store.queue(), checkpoint)?;
        let payload = encode_batch(&batch)?;
        let signed = self.identity.sign(&payload);
        let wire = serde_json::to_vec(&WireSignedEnvelope::from(signed))?;
        let frame = self.auth.seal(&self.node_id, self.next_nonce, now_ms(), wire)?;
        self.next_nonce = self.next_nonce.saturating_add(1);
        self.metrics.observe(&SyncEvent::BatchSent { changes: batch.changes.len(), bytes: frame.payload.len() });
        Ok(frame)
    }

    pub fn receive_frame(&mut self, frame: &AuthenticatedFrame) -> Result<ReceivedBatch, ReplicationError> {
        let mut replay = self.state.replay.get(&frame.node_id).cloned().map(ReplayWindow::from_state).unwrap_or_else(|| ReplayWindow::new(REPLAY_WINDOW));
        self.auth.verify_and_accept(&mut replay, frame)?;
        let wire: WireSignedEnvelope = serde_json::from_slice(&frame.payload)?;
        let signed = SignedBytes::try_from(wire)?;
        let peer = verify_signed_bytes(&signed)?;
        if peer.peer_id != frame.node_id { return Err(ReplicationError::IdentityBinding); }
        let batch: SyncBatch = mwdb_sync::decode_batch(&signed.payload)?;
        let seen = &mut self.state.applied;
        let mut seen_hashes = BTreeSet::new();
        for id in seen.iter() { seen_hashes.insert(*id); }
        let mut seen_uuid = seen_hashes;
        let mut working_seen = seen_uuid.clone().into_iter().collect::<BTreeSet<_>>();
        let mut sync_seen = std::collections::HashSet::new();
        sync_seen.extend(working_seen.iter().copied());
        let ack = apply_batch(&mut self.store, &mut sync_seen, &batch)?;
        let mut accepted_ids = BTreeSet::new();
        for id in &ack.accepted { accepted_ids.insert(*id); }
        working_seen.extend(accepted_ids);
        *seen = working_seen;

        let root_items = batch.changes.iter().map(|change| {
            let value = serde_json::to_value(change)?;
            Ok(canonicalize(&value)?.into_bytes())
        }).collect::<Result<Vec<_>, ReplicationError>>()?;
        let batch_root = state_root(&root_items);

        self.state.replay.insert(peer.peer_id.clone(), replay.state());
        if let Some(checkpoint) = ack.checkpoint {
            let clock = batch.changes.iter().find(|change| change.change_id == checkpoint).map(|change| change.clock).unwrap_or_default();
            self.cursors.upsert(PeerCursor { peer_id: peer.peer_id.clone(), last_change_id: Some(checkpoint), last_clock: clock })?;
        }
        persist_state(&self.state_path, &self.state)?;

        self.metrics = SyncMetrics::default();
        self.metrics.observe(&SyncEvent::BatchReceived { changes: batch.changes.len(), bytes: frame.payload.len() });
        for _ in &ack.accepted { self.metrics.observe(&SyncEvent::ChangeApplied); }
        for _ in &ack.rejected { self.metrics.observe(&SyncEvent::ChangeRejected); }
        Ok(ReceivedBatch { peer, ack, batch_root, metrics: self.metrics.clone() })
    }

    pub fn acknowledge(&mut self, ack: &SyncAck) -> Result<(), ReplicationError> {
        mwdb_sync::mark_acknowledged(self.store.queue_mut(), ack)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn authenticated_replication_wires_identity_auth_replay_cursor_merkle_and_observability() {
        let a_dir = tempdir().unwrap();
        let b_dir = tempdir().unwrap();
        let mut a = ReplicationSession::open(a_dir.path(), "node-a", [1; 32], [9; 32]).unwrap();
        let mut b = ReplicationSession::open(b_dir.path(), "node-b", [2; 32], [9; 32]).unwrap();
        a.store_mut().set("doc:1", serde_json::json!({"v": 42})).unwrap();

        let frame = a.make_frame(None).unwrap();
        let first = b.receive_frame(&frame).unwrap();
        assert_eq!(first.peer.peer_id, a.identity().peer_id);
        assert_eq!(first.ack.accepted.len(), 1);
        assert_eq!(b.store().get("doc:1"), Some(&serde_json::json!({"v": 42})));
        assert_ne!(first.batch_root, [0u8; 32]);
        assert_eq!(first.metrics.batches_received, 1);
        assert!(b.receive_frame(&frame).is_err());

        drop(b);
        let mut b = ReplicationSession::open(b_dir.path(), "node-b", [2; 32], [9; 32]).unwrap();
        let frame2 = a.make_frame(None).unwrap();
        let second = b.receive_frame(&frame2).unwrap();
        assert!(second.ack.accepted.is_empty());
        assert_eq!(b.cursor(&a.identity().peer_id).unwrap().last_clock, 1);
    }

    #[test]
    fn forged_frame_context_is_rejected_before_payload_processing() {
        let a_dir = tempdir().unwrap();
        let b_dir = tempdir().unwrap();
        let mut a = ReplicationSession::open(a_dir.path(), "node-a", [3; 32], [8; 32]).unwrap();
        let mut b = ReplicationSession::open(b_dir.path(), "node-b", [4; 32], [8; 32]).unwrap();
        a.store_mut().set("doc:2", serde_json::json!({"v": 7})).unwrap();
        let mut frame = a.make_frame(None).unwrap();
        frame.node_id = "evil".into();
        assert!(matches!(b.receive_frame(&frame), Err(ReplicationError::Auth(mwdb_auth::AuthError::InvalidTag))));
    }
}
