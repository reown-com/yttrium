use {
    super::smart_sessions::PolicyData,
    alloy::primitives::{address, Address, Bytes},
};

// https://github.com/rhinestonewtf/module-sdk/blob/main/src/module/smart-sessions/policies/sudo-policy/constants.ts
pub const SUDO_POLICY_ADDRESS: Address =
    address!("0000003111cD8e92337C100F22B7A9dbf8DEE301");

// https://github.com/rhinestonewtf/module-sdk/blob/1f2f2c5380614ad07b6e1ccbb5a9ed55374c673c/src/module/smart-sessions/policies/sudo-policy/installation.ts#L4
pub fn get_sudo_policy() -> PolicyData {
    PolicyData { policy: SUDO_POLICY_ADDRESS, initData: Bytes::default() }
}
