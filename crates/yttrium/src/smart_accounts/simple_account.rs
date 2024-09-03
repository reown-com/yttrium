use alloy::{sol, sol_types::SolCall};

sol!(
    #[allow(missing_docs)]
    function execute(address dest, uint256 value, bytes calldata func);
);

pub mod create_account;
pub mod factory;

pub struct SimpleAccountExecute(executeCall);

impl SimpleAccountExecute {
    pub fn new(
        address: alloy::primitives::Address,
        value: alloy::primitives::U256,
        func: alloy::primitives::Bytes,
    ) -> Self {
        Self(executeCall { dest: address, value: value, func: func })
    }

    pub fn encode(&self) -> Vec<u8> {
        executeCall::abi_encode(&self.0)
    }
}

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    SimpleAccount,
    "src/contracts/artifacts/contracts/samples/SimpleAccount.sol/SimpleAccount.json"
);

pub const DUMMY_SIGNATURE_HEX: &str = "0xfffffffffffffffffffffffffffffff0000000000000000000000000000000007aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa1c";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimpleAccountAddress(alloy::primitives::Address);

impl SimpleAccountAddress {
    pub fn new(address: alloy::primitives::Address) -> Self {
        Self(address)
    }

    pub fn to_address(&self) -> alloy::primitives::Address {
        self.0
    }
}

impl Into<alloy::primitives::Address> for SimpleAccountAddress {
    fn into(self) -> alloy::primitives::Address {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OwnerAddress(alloy::primitives::Address);

impl OwnerAddress {
    pub fn new(address: alloy::primitives::Address) -> Self {
        Self(address)
    }

    pub fn to_address(&self) -> alloy::primitives::Address {
        self.0
    }
}

impl Into<alloy::primitives::Address> for OwnerAddress {
    fn into(self) -> alloy::primitives::Address {
        self.0
    }
}
