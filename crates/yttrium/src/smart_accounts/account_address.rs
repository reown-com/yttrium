use {
    alloy::primitives::Address,
    serde::{Deserialize, Serialize},
};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Ord,
    PartialOrd,
    Serialize,
    Deserialize,
    Default,
)]
pub struct AccountAddress(Address);

impl AccountAddress {
    pub fn new(address: Address) -> Self {
        Self(address)
    }

    pub fn to_address(&self) -> Address {
        self.0
    }
}

impl From<AccountAddress> for Address {
    fn from(val: AccountAddress) -> Self {
        val.0
    }
}

impl From<Address> for AccountAddress {
    fn from(val: Address) -> Self {
        Self::new(val)
    }
}

impl std::fmt::Display for AccountAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
