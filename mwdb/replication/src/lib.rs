use std::{collections::BTreeSet, path::{Path, PathBuf}, time::{SystemTime, UNIX_EPOCH}};

use mwdb_auth::{AuthenticatedFrame, ReplayStore, ReplayStoreError, SharedKeyAuthenticator};
use mwdb_canonical::canonicalize;
use mwdb_capabilities::{negotiate, require, CapabilityAgreement, CapabilityError, CapabilityHello};
use mwdb_cursor::{CursorError, CursorStore, PeerCursor};
use mwdb_identity::{verify_signed_bytes, IdentityKey, PeerIdentity, SignedBytes};
use mwdb_local_first::{LocalFirstError, LocalFirstStore};
use mwdb_merkle::state_root;
use mwdb_observability::{SyncEvent, SyncMetrics};
use mwdb_sync::{apply_batch, encode_batch, select_since, SyncAck, SyncBatch, SyncError};
use mwdb_trust::{TrustError, TrustStore};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

pub const REPLICATION_PROTOCOL_V1: u16 = 1;
const REPLAY_WINDOW: u64 = 64;
pub const CAPABILITIES_V1: &[&str] = &["sync.v1", "identity.ed25519.v1", "auth.frame.v1", "proof.merkle.v1"];

#[derive(Debug, Error)]
pub enum ReplicationError {
    #[error("I/O error: {0}")] Io(#[from] std::io::Error),
    #[error("serialization error: {0}")] Serialization(#[from] serde_json::Error),
    #[error("canonicalization error: {0}")] Canonical(#[from] mwdb_canonical::CanonicalError),
    #[error("authentication error: {0}")] Auth(#[from] mwdb_auth::AuthError),
    #[error("persistent replay error: {0}")] Replay(#[from] ReplayStoreError),
    #[error("identity verification failed: {0}")] Identity(#[from] mwdb_identity::IdentityError),
    #[error("sync error: {0}")] Sync(#[from] SyncError),
    #[error("local-first error: {0}")] LocalFirst(#[from] LocalFirstError),
    #[error("cursor error: {0}")] Cursor(#[from] CursorError),
    #[error("trust error: {0}")] Trust(#[from] TrustError),
    #[error("capability negotiation failed: {0}")] Capability(#[from] CapabilityError),
    #[error("frame node id does not match signed peer identity")] IdentityBinding,
    #[error("peer is not trusted: {0}")] UntrustedPeer(String),
    #[error("unsupported replication protocol {0}")] UnsupportedProtocol(u16),
    #[error("key epoch must increase monotonically: current={current}, requested={requested}")]
    InvalidKeyEpoch { current: u64, requested: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct WireSignedEnvelope {
    protocol: u16,
    peer_id: String,
    public_key: Vec<u8>,
    capabilities: Vec<String>,
    #[serde(default = "default_key_epoch")]
    key_epoch: u64,
    payload: Vec<u8>,
    signature: Vec<u8>,
}

impl From<SignedBytes> for WireSignedEnvelope {
    fn from(value: SignedBytes) -> Self {
        Self {
            protocol: value.protocol,
            peer_id: value.peer_id,
            public_key: value.public_key.to_vec(),
            capabilities: Vec::new(),
            key_epoch: default_key_epoch(),
            payload: value.payload,
            signature: value.signature.to_vec(),
        }
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

fn default_next_nonce() -> u64 { 1 }
fn default_key_epoch() -> u64 { 1 }

#[derive(Debug, Serialize, Deserialize)]
struct DurableSessionState {
    applied: BTreeSet<Uuid>,
    #[serde(default = "default_next_nonce")]
    next_nonce: u64,
    #[serde(default = "default_key_epoch")]
    key_epoch: u64,
}

impl Default for DurableSessionState {
    fn default() -> Self {
        Self { applied: BTreeSet::new(), next_nonce: default_next_nonce(), key_epoch: default_key_epoch() }
    }
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

fn capability_hello(peer_id: &str) -> CapabilityHello {
    CapabilityHello { protocol: mwdb_capabilities::CAPABILITY_PROTOCOL_V1, node_id: peer_id.to_string(), capabilities: CAPABILITIES_V1.iter().map(|value| (*value).to_string()).collect() }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceivedBatch {
    pub peer: PeerIdentity,
    pub agreement: CapabilityAgreement,
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
    trust: TrustStore,
    replay: ReplayStore,
    state_path: PathBuf,
    state: DurableSessionState,
    metrics: SyncMetrics,
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
            store: LocalFirstStore::open(root.join("local"), node_id.clone())?,
            cursors: CursorStore::open(root.join("cursors.json"))?,
            trust: TrustStore::open(root.join("trust.json"))?,
            replay: ReplayStore::open(root.join("replay.json"))?,
            state_path,
            metrics: SyncMetrics::default(),
            state,
        })
    }

    pub fn node_id(&self) -> &str { &self.node_id }
    pub fn identity(&self) -> &PeerIdentity { self.identity.identity() }
    pub fn capabilities(&self) -> &[&str] { CAPABILITIES_V1 }
    pub fn store(&self) -> &LocalFirstStore { &self.store }
    pub fn store_mut(&mut self) -> &mut LocalFirstStore { &mut self.store }
    pub fn metrics(&self) -> &SyncMetrics { &self.metrics }
    pub fn cursor(&self, peer_id: &str) -> Option<&PeerCursor> { self.cursors.get(peer_id) }
    pub fn key_epoch(&self) -> u64 { self.state.key_epoch }
    pub fn trust_peer(&mut self, peer: &PeerIdentity, label: Option<String>) -> Result<(), ReplicationError> { self.trust.trust(peer.peer_id.clone(), peer.public_key, label)?; Ok(()) }
    pub fn revoke_peer(&mut self, peer_id: &str) -> Result<bool, ReplicationError> { Ok(self.trust.revoke(peer_id)?) }

    pub fn advance_key_epoch(&mut self, next_epoch: u64) -> Result<(), ReplicationError> {
        if next_epoch <= self.state.key_epoch {
            return Err(ReplicationError::InvalidKeyEpoch { current: self.state.key_epoch, requested: next_epoch });
        }
        self.state.key_epoch = next_epoch;
        self.state.next_nonce = default_next_nonce();
        persist_state(&self.state_path, &self.state)
    }

    pub fn make_frame(&mut self, checkpoint: Option<Uuid>) -> Result<AuthenticatedFrame, ReplicationError> {
        let batch = select_since(self.store.queue(), checkpoint)?;
        let payload = encode_batch(&batch)?;
        let signed = self.identity.sign(&payload);
        let mut wire = WireSignedEnvelope::from(signed);
        wire.capabilities = CAPABILITIES_V1.iter().map(|value| (*value).to_string()).collect();
        wire.key_epoch = self.state.key_epoch;
        let bytes = serde_json::to_vec(&wire)?;
        let nonce = self.state.next_nonce;
        let frame = self.auth.seal(self.identity.identity().peer_id.clone(), nonce, now_ms(), bytes)?;
        self.state.next_nonce = nonce.saturating_add(1);
        persist_state(&self.state_path, &self.state)?;
        self.metrics.observe(&SyncEvent::BatchSent { changes: batch.changes.len(), bytes: frame.payload.len() });
        Ok(frame)
    }

    pub fn receive_frame(&mut self, frame: &AuthenticatedFrame) -> Result<ReceivedBatch, ReplicationError> {
        self.auth.verify(frame)?;
        let wire: WireSignedEnvelope = serde_json::from_slice(&frame.payload)?;
        let signed = SignedBytes::try_from(wire.clone())?;
        let peer = verify_signed_bytes(&signed)?;
        if peer.peer_id != frame.node_id { return Err(ReplicationError::IdentityBinding); }
        if !self.trust.is_trusted(&peer.peer_id, &peer.public_key) { return Err(ReplicationError::UntrustedPeer(peer.peer_id)); }

        let remote_hello = CapabilityHello { protocol: mwdb_capabilities::CAPABILITY_PROTOCOL_V1, node_id: peer.peer_id.clone(), capabilities: wire.capabilities.clone() };
        let agreement = negotiate(&capability_hello(self.identity.identity().peer_id.as_str()), &remote_hello)?;
        require(&agreement, "sync.v1")?;
        require(&agreement, "identity.ed25519.v1")?;
        require(&agreement, "auth.frame.v1")?;
        require(&agreement, "proof.merkle.v1")?;

        let batch: SyncBatch = mwdb_sync::decode_batch(&signed.payload)?;
        self.auth.verify_and_accept_persistent(&mut self.replay, wire.key_epoch, REPLAY_WINDOW, frame)?;

        let mut sync_seen = std::collections::HashSet::new();
        sync_seen.extend(self.state.applied.iter().copied());
        let ack = apply_batch(&mut self.store, &mut sync_seen, &batch)?;
        self.state.applied.extend(ack.accepted.iter().copied());

        let root_items = batch.changes.iter().map(|change| {
            let value = serde_json::to_value(change)?;
            Ok(canonicalize(&value)?.into_bytes())
        }).collect::<Result<Vec<_>, ReplicationError>>()?;
        let batch_root = state_root(&root_items);

        if let Some(checkpoint) = ack.checkpoint {
            let clock = batch.changes.iter().find(|change| change.change_id == checkpoint).map(|change| change.logical_clock).unwrap_or_default();
            self.cursors.upsert(PeerCursor { peer_id: peer.peer_id.clone(), last_change_id: Some(checkpoint), last_clock: clock })?;
        }
        persist_state(&self.state_path, &self.state)?;

        self.metrics = SyncMetrics::default();
        self.metrics.observe(&SyncEvent::BatchReceived { changes: batch.changes.len(), bytes: frame.payload.len() });
        for _ in &ack.accepted { self.metrics.observe(&SyncEvent::ChangeApplied); }
        for _ in &ack.rejected { self.metrics.observe(&SyncEvent::ChangeRejected); }
        Ok(ReceivedBatch { peer, agreement, ack, batch_root, metrics: self.metrics.clone() })
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

    fn sessions(seed_a: u8, seed_b: u8) -> (tempfile::TempDir, tempfile::TempDir, ReplicationSession, ReplicationSession) {
        let a_dir = tempdir().unwrap();
        let b_dir = tempdir().unwrap();
        let a = ReplicationSession::open(a_dir.path(), "node-a", [seed_a; 32], [9; 32]).unwrap();
        let b = ReplicationSession::open(b_dir.path(), "node-b", [seed_b; 32], [9; 32]).unwrap();
        (a_dir, b_dir, a, b)
    }

    #[test]
    fn authenticated_replication_wires_capability_identity_auth_replay_cursor_merkle_and_observability() {
        let (_a_dir, b_dir, mut a, mut b) = sessions(1, 2);
        b.trust_peer(a.identity(), Some("test-peer".into())).unwrap();
        a.store_mut().set("doc:1", serde_json::json!({"v": 42})).unwrap();
        let frame = a.make_frame(None).unwrap();
        let first = b.receive_frame(&frame).unwrap();
        assert_eq!(first.peer.peer_id, a.identity().peer_id);
        assert_eq!(first.agreement.common.len(), 4);
        assert_eq!(first.ack.accepted.len(), 1);
        assert_eq!(b.store().get("doc:1"), Some(&serde_json::json!({"v": 42})));
        assert_ne!(first.batch_root, [0u8; 32]);
        assert_eq!(first.metrics.batches_received, 1);
        assert!(matches!(
            b.receive_frame(&frame),
            Err(ReplicationError::Replay(ReplayStoreError::Auth(mwdb_auth::AuthError::Replay)))
        ));
        let checkpoint = first.ack.checkpoint;
        drop(b);
        let mut b = ReplicationSession::open(b_dir.path(), "node-b", [2; 32], [9; 32]).unwrap();
        assert!(matches!(
            b.receive_frame(&frame),
            Err(ReplicationError::Replay(ReplayStoreError::Auth(mwdb_auth::AuthError::Replay)))
        ));
        let frame2 = a.make_frame(checkpoint).unwrap();
        let second = b.receive_frame(&frame2).unwrap();
        assert!(second.ack.accepted.is_empty());
        assert_eq!(b.cursor(&a.identity().peer_id).unwrap().last_clock, 1);
    }

    #[test]
    fn untrusted_peer_is_rejected_after_signature_verification() {
        let (_a_dir, _b_dir, mut a, mut b) = sessions(3, 4);
        a.store_mut().set("doc:2", serde_json::json!({"v": 7})).unwrap();
        let frame = a.make_frame(None).unwrap();
        assert!(matches!(b.receive_frame(&frame), Err(ReplicationError::UntrustedPeer(_))));
    }

    #[test]
    fn revoked_peer_is_rejected() {
        let (_a_dir, _b_dir, mut a, mut b) = sessions(5, 6);
        b.trust_peer(a.identity(), None).unwrap();
        a.store_mut().set("doc:3", serde_json::json!({"v": 8})).unwrap();
        let frame = a.make_frame(None).unwrap();
        b.revoke_peer(a.identity().peer_id.as_str()).unwrap();
        assert!(matches!(b.receive_frame(&frame), Err(ReplicationError::UntrustedPeer(_))));
    }

    #[test]
    fn sender_nonce_survives_restart() {
        let (a_dir, _b_dir, _a, mut b) = sessions(7, 8);
        let f1 = {
            let mut a = ReplicationSession::open(a_dir.path(), "node-a", [7; 32], [9; 32]).unwrap();
            a.store_mut().set("doc:1", serde_json::json!(1)).unwrap();
            a.make_frame(None).unwrap()
        };
        let mut a_key = ReplicationSession::open(a_dir.path(), "node-a", [7; 32], [9; 32]).unwrap();
        b.trust_peer(a_key.identity(), None).unwrap();
        b.receive_frame(&f1).unwrap();
        let f2 = a_key.make_frame(None).unwrap();
        assert!(f2.nonce > f1.nonce);
    }

    #[test]
    fn key_epoch_advances_monotonically_and_rejects_stale_epoch_after_rotation() {
        let (_a_dir, _b_dir, mut a, mut b) = sessions(11, 12);
        b.trust_peer(a.identity(), None).unwrap();
        a.store_mut().set("doc:1", serde_json::json!(1)).unwrap();
        let epoch1 = a.make_frame(None).unwrap();
        let first = b.receive_frame(&epoch1).unwrap();

        a.advance_key_epoch(2).unwrap();
        a.store_mut().set("doc:2", serde_json::json!(2)).unwrap();
        let epoch2 = a.make_frame(first.ack.checkpoint).unwrap();
        b.receive_frame(&epoch2).unwrap();
        assert_eq!(a.key_epoch(), 2);
        assert!(matches!(
            a.advance_key_epoch(2),
            Err(ReplicationError::InvalidKeyEpoch { current: 2, requested: 2 })
        ));
        assert!(matches!(
            b.receive_frame(&epoch1),
            Err(ReplicationError::Replay(ReplayStoreError::StaleKeyEpoch { current: 2, requested: 1 }))
        ));
    }
}
