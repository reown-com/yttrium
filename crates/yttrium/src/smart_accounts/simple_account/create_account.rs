use alloy::{primitives::U256, sol, sol_types::SolCall};

sol!(
    #[allow(missing_docs)]
    #[derive(Debug, PartialEq, Eq)]
    type SimpleAccount is address;
    function createAccount(address owner,uint256 salt) public returns (SimpleAccount ret);
);

pub struct SimpleAccountCreate(createAccountCall);

impl SimpleAccountCreate {
    pub fn new(
        owner: alloy::primitives::Address,
        salt: alloy::primitives::U256,
    ) -> Self {
        Self(createAccountCall { owner: owner, salt: salt })
    }

    pub fn new_u64(owner: alloy::primitives::Address, salt: u64) -> Self {
        let salt = U256::from(salt);
        Self(createAccountCall { owner: owner, salt: salt })
    }

    pub fn encode(&self) -> Vec<u8> {
        createAccountCall::abi_encode(&self.0)
    }
}
