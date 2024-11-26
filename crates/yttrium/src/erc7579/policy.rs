use super::smart_sessions::PolicyData;
use alloy::primitives::{address, Address, Bytes};

pub const SUDO_POLICY_ADDRESS: Address =
    address!("529Ad04F4D83aAb25144a90267D4a1443B84f5A6");

// https://github.com/rhinestonewtf/module-sdk/blob/1f2f2c5380614ad07b6e1ccbb5a9ed55374c673c/src/module/smart-sessions/policies/sudo-policy/installation.ts#L4
pub fn get_sudo_policy() -> PolicyData {
    PolicyData { policy: SUDO_POLICY_ADDRESS, initData: Bytes::default() }
}
