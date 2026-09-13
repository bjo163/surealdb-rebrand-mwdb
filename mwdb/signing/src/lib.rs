use blake3::Hasher;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedChange {
    pub change_id: Uuid,
    pub actor_id: String,
    pub payload: Vec<u8>,
    pub mac: String,
}

fn bytes(change_id: Uuid, actor_id: &str, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(change_id.as_bytes());
    out.extend_from_slice(actor_id.as_bytes());
    out.push(0);
    out.extend_from_slice(payload);
    out
}

pub fn sign(change_id: Uuid, actor_id: &str, payload: &[u8], key: &[u8; 32]) -> SignedChange {
    let mut h = Hasher::new_keyed(key);
    h.update(&bytes(change_id, actor_id, payload));
    SignedChange { change_id, actor_id: actor_id.into(), payload: payload.to_vec(), mac: h.finalize().to_hex().to_string() }
}

pub fn verify(value: &SignedChange, key: &[u8; 32]) -> bool {
    let mut h = Hasher::new_keyed(key);
    h.update(&bytes(value.change_id, &value.actor_id, &value.payload));
    h.finalize().to_hex().as_str() == value.mac
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn round_trip() {
        let key = [7u8; 32];
        let v = sign(Uuid::from_u128(1), "node-a", b"payload", &key);
        assert!(verify(&v, &key));
    }
    #[test]
    fn changed_payload_is_rejected() {
        let key = [7u8; 32];
        let mut v = sign(Uuid::from_u128(2), "node-a", b"payload", &key);
        v.payload = b"other".to_vec();
        assert!(!verify(&v, &key));
    }
}
