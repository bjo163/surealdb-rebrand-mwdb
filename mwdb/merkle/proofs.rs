use super::{combine, hash_leaf, MerkleError};
use serde::{Deserialize, Serialize};

pub const MERKLE_PROOF_V1: u16 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MerkleProofStep {
    pub sibling: [u8; 32],
    pub sibling_on_left: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MerkleProof {
    pub protocol: u16,
    pub leaf: [u8; 32],
    pub index: usize,
    pub steps: Vec<MerkleProofStep>,
}

impl MerkleProof {
    pub fn encode(&self) -> Result<Vec<u8>, serde_json::Error> { serde_json::to_vec(self) }
    pub fn decode(bytes: &[u8]) -> Result<Self, MerkleError> {
        let proof: Self = serde_json::from_slice(bytes).map_err(|_| MerkleError::InvalidProof)?;
        if proof.protocol != MERKLE_PROOF_V1 { return Err(MerkleError::InvalidProof); }
        Ok(proof)
    }
}

pub fn build_proof(items: &[Vec<u8>], target: usize) -> Result<MerkleProof, MerkleError> {
    if items.is_empty() || target >= items.len() { return Err(MerkleError::InvalidIndex(target)); }
    let mut level: Vec<[u8; 32]> = items.iter().map(|v| hash_leaf(v)).collect();
    let leaf = level[target];
    let mut index = target;
    let mut steps = Vec::new();
    while level.len() > 1 {
        let sibling_index = if index % 2 == 0 { (index + 1).min(level.len() - 1) } else { index - 1 };
        steps.push(MerkleProofStep { sibling: level[sibling_index], sibling_on_left: sibling_index < index });
        let mut next = Vec::new();
        let mut i = 0;
        while i < level.len() {
            let left = level[i];
            let right = if i + 1 < level.len() { level[i + 1] } else { left };
            next.push(combine(&left, &right));
            i += 2;
        }
        level = next;
        index /= 2;
    }
    Ok(MerkleProof { protocol: MERKLE_PROOF_V1, leaf, index: target, steps })
}

pub fn verify_proof(root: [u8; 32], proof: &MerkleProof) -> Result<bool, MerkleError> {
    if proof.protocol != MERKLE_PROOF_V1 { return Err(MerkleError::InvalidProof); }
    let mut current = proof.leaf;
    let mut index = proof.index;
    for step in &proof.steps {
        current = if step.sibling_on_left { combine(&step.sibling, &current) } else { combine(&current, &step.sibling) };
        index /= 2;
    }
    if index != 0 { return Err(MerkleError::InvalidProof); }
    Ok(current == root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proof_serialization_round_trip_is_versioned() {
        let items = vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec(), b"d".to_vec()];
        let proof = build_proof(&items, 2).unwrap();
        let bytes = proof.encode().unwrap();
        let restored = MerkleProof::decode(&bytes).unwrap();
        assert_eq!(restored, proof);
        assert!(verify_proof(super::super::state_root(&items), &restored).unwrap());
    }
}
