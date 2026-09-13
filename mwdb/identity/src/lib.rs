use ed25519_dalek::{Signature, SigningKey, VerifyingKey, Signer, Verifier};
use thiserror::Error;

pub const IDENTITY_PROTOCOL_V1: u16 = 1;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum IdentityError {
    #[error("invalid signing key material")]
    InvalidSigningKey,
    #[error("invalid verifying key material")]
    InvalidVerifyingKey,
    #[error("invalid signature material")]
    InvalidSignature,
    #[error("signature verification failed")]
    VerificationFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerIdentity {
    pub protocol: u16,
    pub peer_id: String,
    pub public_key: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedBytes {
    pub protocol: u16,
    pub peer_id: String,
    pub public_key: [u8; 32],
    pub payload: Vec<u8>,
    pub signature: [u8; 64],
}

pub struct IdentityKey {
    signing: SigningKey,
    identity: PeerIdentity,
}

impl IdentityKey {
    pub fn from_seed(seed: [u8; 32]) -> Self {
        let signing = SigningKey::from_bytes(&seed);
        let public_key = signing.verifying_key().to_bytes();
        let peer_id = format!("ed25519-{}", hex::encode(&public_key[..8]));
        let identity = PeerIdentity { protocol: IDENTITY_PROTOCOL_V1, peer_id, public_key };
        Self { signing, identity }
    }

    pub fn identity(&self) -> &PeerIdentity {
        &self.identity
    }

    pub fn sign(&self, payload: &[u8]) -> SignedBytes {
        let signature = self.signing.sign(payload).to_bytes();
        SignedBytes {
            protocol: IDENTITY_PROTOCOL_V1,
            peer_id: self.identity.peer_id.clone(),
            public_key: self.identity.public_key,
            payload: payload.to_vec(),
            signature,
        }
    }
}

pub fn verify_signed_bytes(signed: &SignedBytes) -> Result<PeerIdentity, IdentityError> {
    if signed.protocol != IDENTITY_PROTOCOL_V1 {
        return Err(IdentityError::InvalidVerifyingKey);
    }
    let verifying = VerifyingKey::from_bytes(&signed.public_key).map_err(|_| IdentityError::InvalidVerifyingKey)?;
    let signature = Signature::from_bytes(&signed.signature);
    verifying.verify(&signed.payload, &signature).map_err(|_| IdentityError::VerificationFailed)?;
    let expected_peer_id = format!("ed25519-{}", hex::encode(&signed.public_key[..8]));
    if expected_peer_id != signed.peer_id {
        return Err(IdentityError::VerificationFailed);
    }
    Ok(PeerIdentity { protocol: signed.protocol, peer_id: signed.peer_id.clone(), public_key: signed.public_key })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_and_verify_round_trip() {
        let key = IdentityKey::from_seed([7; 32]);
        let signed = key.sign(b"hello");
        let identity = verify_signed_bytes(&signed).unwrap();
        assert_eq!(identity.peer_id, key.identity().peer_id);
    }

    #[test]
    fn tampered_payload_is_rejected() {
        let key = IdentityKey::from_seed([8; 32]);
        let mut signed = key.sign(b"hello");
        signed.payload[0] = b'H';
        assert_eq!(verify_signed_bytes(&signed), Err(IdentityError::VerificationFailed));
    }

    #[test]
    fn peer_id_is_bound_to_public_key() {
        let key = IdentityKey::from_seed([9; 32]);
        let mut signed = key.sign(b"hello");
        signed.peer_id = "ed25519-not-the-key".into();
        assert_eq!(verify_signed_bytes(&signed), Err(IdentityError::VerificationFailed));
    }
}
