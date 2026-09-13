use super::{combine, hash_leaf, MerkleError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerkleProofStep {
    pub sibling: [u8; 32],
    pub sibling_on_left: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerkleProof {
    pub leaf: [u8; 32],
    pub steps: Vec<MerkleProofStep>,
}

pub fn build_proof(items: &[Vec<u8>], index: usize) -> Result<MerkleProof, MerkleError> {
    if index >= items.len() {
        return Err(MerkleError::InvalidIndex(index));
    }

    let leaf = hash_leaf(&items[index]);
    let mut level: Vec<[u8; 32]> = items.iter().map(|item| hash_leaf(item)).collect();
    let mut current = index;
    let mut steps = Vec::new();

    while level.len() > 1 {
        let sibling_index = if current % 2 == 0 {
            if current + 1 < level.len() {
                current + 1
            } else {
                current
            }
        } else {
            current - 1
        };

        steps.push(MerkleProofStep {
            sibling: level[sibling_index],
            sibling_on_left: current % 2 == 1,
        });

        let mut next = Vec::with_capacity(level.len().div_ceil(2));
        let mut pair = 0;
        while pair < level.len() {
            let left = level[pair];
            let right = if pair + 1 < level.len() {
                level[pair + 1]
            } else {
                left
            };
            next.push(combine(&left, &right));
            pair += 2;
        }

        level = next;
        current /= 2;
    }

    Ok(MerkleProof { leaf, steps })
}

pub fn verify_proof(root: [u8; 32], proof: &MerkleProof) -> Result<bool, MerkleError> {
    let mut current = proof.leaf;
    for step in &proof.steps {
        current = if step.sibling_on_left {
            combine(&step.sibling, &current)
        } else {
            combine(&current, &step.sibling)
        };
    }
    Ok(current == root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_root;

    #[test]
    fn proof_tracks_requested_leaf() {
        let items = vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec()];
        let proof = build_proof(&items, 1).unwrap();
        assert_eq!(proof.leaf, hash_leaf(b"b"));
        assert!(verify_proof(state_root(&items), &proof).unwrap());
    }

    #[test]
    fn odd_leaf_duplication_matches_root_builder() {
        let items = vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec()];
        let proof = build_proof(&items, 2).unwrap();
        assert!(verify_proof(state_root(&items), &proof).unwrap());
    }

    #[test]
    fn out_of_range_index_is_rejected() {
        let items = vec![b"a".to_vec()];
        assert!(matches!(build_proof(&items, 1), Err(MerkleError::InvalidIndex(1))));
    }
}
