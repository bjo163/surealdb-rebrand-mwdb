use thiserror::Error;

#[derive(Debug, Error)]
pub enum MerkleError {
    #[error("invalid merkle proof")]
    InvalidProof,
    #[error("leaf index {0} is out of range")]
    InvalidIndex(usize),
}

pub fn hash_leaf(data: &[u8]) -> [u8; 32] {
    *blake3::hash(data).as_bytes()
}

pub fn combine(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut bytes = [0u8; 64];
    bytes[..32].copy_from_slice(left);
    bytes[32..].copy_from_slice(right);
    *blake3::hash(&bytes).as_bytes()
}

pub fn state_root(items: &[Vec<u8>]) -> [u8; 32] {
    if items.is_empty() { return hash_leaf(b""); }
    let mut level: Vec<[u8; 32]> = items.iter().map(|v| hash_leaf(v)).collect();
    while level.len() > 1 {
        let mut next = Vec::new();
        let mut index = 0;
        while index < level.len() {
            let left = level[index];
            let right = if index + 1 < level.len() { level[index + 1] } else { left };
            next.push(combine(&left, &right));
            index += 2;
        }
        level = next;
    }
    level[0]
}

pub mod proofs;
pub use proofs::{build_proof, verify_proof, MerkleProof, MerkleProofStep};

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn same_state_has_same_root() { let items = vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec()]; assert_eq!(state_root(&items), state_root(&items)); }
    #[test] fn changed_state_changes_root() { let a = vec![b"a".to_vec(), b"b".to_vec()]; let b = vec![b"a".to_vec(), b"c".to_vec()]; assert_ne!(state_root(&a), state_root(&b)); }
    #[test] fn order_changes_root() { let a = vec![b"a".to_vec(), b"b".to_vec()]; let b = vec![b"b".to_vec(), b"a".to_vec()]; assert_ne!(state_root(&a), state_root(&b)); }
    #[test] fn inclusion_proof_round_trip() { let items = vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec(), b"d".to_vec(), b"e".to_vec()]; let root = state_root(&items); let proof = build_proof(&items, 3).unwrap(); assert!(verify_proof(root, &proof).unwrap()); }
    #[test] fn tampered_proof_fails() { let items = vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec()]; let root = state_root(&items); let mut proof = build_proof(&items, 1).unwrap(); proof.leaf[0] ^= 1; assert!(!verify_proof(root, &proof).unwrap()); }
}
