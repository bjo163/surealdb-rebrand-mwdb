use super::{hash_leaf, hash_parent, MerkleError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerkleProofStep {
    pub sibling: [u8; 32],
    pub sibling_on_left: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerkleProof {
    pub leaf: [u8; 32],
    pub index: usize,
    pub steps: Vec<MerkleProofStep>,
}

pub fn verify_proof(root: [u8; 32], proof: &MerkleProof) -> Result<bool, MerkleError> {
    let mut current = proof.leaf;
    let mut index = proof.index;
    for step in &proof.steps {
        current = if step.sibling_on_left {
            hash_parent(&step.sibling, &current)
        } else {
            hash_parent(&current, &step.sibling)
        };
        index /= 2;
    }
    if index != 0 {
        return Err(MerkleError::InvalidProof);
    }
    Ok(current == root)
}
