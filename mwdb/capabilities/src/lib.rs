pub const CAPABILITY_PROTOCOL_V1: u16 = 1;

pub fn common(local: &[&str], remote: &[&str]) -> Vec<String> {
    let mut out = local.iter().filter(|v| remote.contains(v)).map(|v| (*v).to_string()).collect::<Vec<_>>();
    out.sort();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intersection_is_sorted_and_unique() {
        assert_eq!(common(&["sync", "proof", "sync"], &["proof", "sync"]), vec!["proof", "sync"]);
    }
}
