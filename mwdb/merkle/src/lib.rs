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
    if items.is_empty() {
        return hash_leaf(b"");
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_state_has_same_root() {
        let items = vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec()];
        assert_eq!(state_root(&items), state_root(&items));
    }

    #[test]
    fn changed_state_changes_root() {
        let a = vec![b"a".to_vec(), b"b".to_vec()];
        let b = vec![b"a".to_vec(), b"c".to_vec()];
        assert_ne!(state_root(&a), state_root(&b));
    }

    #[test]
    fn order_changes_root() {
        let a = vec![b"a".to_vec(), b"b".to_vec()];
        let b = vec![b"b".to_vec(), b"a".to_vec()];
        assert_ne!(state_root(&a), state_root(&b));
    }
}
