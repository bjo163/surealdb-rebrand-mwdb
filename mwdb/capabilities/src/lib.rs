use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const CAPABILITY_PROTOCOL_V1: u16 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityHello {
    pub protocol: u16,
    pub node_id: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityAgreement {
    pub protocol: u16,
    pub common: Vec<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CapabilityError {
    #[error("unsupported capability protocol {0}")]
    UnsupportedProtocol(u16),
    #[error("required capability is not negotiated: {0}")]
    Missing(String),
}

pub fn common(local: &[&str], remote: &[&str]) -> Vec<String> {
    let mut out = local.iter().filter(|v| remote.contains(v)).map(|v| (*v).to_string()).collect::<Vec<_>>();
    out.sort();
    out.dedup();
    out
}

pub fn negotiate(local: &CapabilityHello, remote: &CapabilityHello) -> Result<CapabilityAgreement, CapabilityError> {
    if local.protocol != CAPABILITY_PROTOCOL_V1 { return Err(CapabilityError::UnsupportedProtocol(local.protocol)); }
    if remote.protocol != CAPABILITY_PROTOCOL_V1 { return Err(CapabilityError::UnsupportedProtocol(remote.protocol)); }
    let local_refs = local.capabilities.iter().map(String::as_str).collect::<Vec<_>>();
    let remote_refs = remote.capabilities.iter().map(String::as_str).collect::<Vec<_>>();
    Ok(CapabilityAgreement { protocol: CAPABILITY_PROTOCOL_V1, common: common(&local_refs, &remote_refs) })
}

pub fn require(agreement: &CapabilityAgreement, capability: &str) -> Result<(), CapabilityError> {
    if agreement.protocol != CAPABILITY_PROTOCOL_V1 { return Err(CapabilityError::UnsupportedProtocol(agreement.protocol)); }
    if agreement.common.iter().any(|item| item == capability) { Ok(()) } else { Err(CapabilityError::Missing(capability.to_string())) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intersection_is_sorted_and_unique() {
        assert_eq!(common(&["sync", "proof", "sync"], &["proof", "sync"]), vec!["proof", "sync"]);
    }

    #[test]
    fn negotiation_is_fail_closed_for_protocol_and_capability() {
        let local = CapabilityHello { protocol: 1, node_id: "a".into(), capabilities: vec!["sync".into(), "proof".into()] };
        let remote = CapabilityHello { protocol: 1, node_id: "b".into(), capabilities: vec!["sync".into()] };
        let agreement = negotiate(&local, &remote).unwrap();
        assert!(require(&agreement, "sync").is_ok());
        assert_eq!(require(&agreement, "proof"), Err(CapabilityError::Missing("proof".into())));
    }

    #[test]
    fn incompatible_protocol_is_rejected() {
        let a = CapabilityHello { protocol: 1, node_id: "a".into(), capabilities: vec!["sync".into()] };
        let b = CapabilityHello { protocol: 2, node_id: "b".into(), capabilities: vec!["sync".into()] };
        assert_eq!(negotiate(&a, &b), Err(CapabilityError::UnsupportedProtocol(2)));
    }
}
