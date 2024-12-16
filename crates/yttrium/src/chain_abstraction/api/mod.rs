use {
    alloy::primitives::{Address, Bytes, U256, U64},
    serde::{Deserialize, Serialize},
};

pub mod fungible_price;
pub mod route;
pub mod status;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub from: Address,
    pub to: Address,
    pub value: U256,
    pub gas: U64,
    pub data: Bytes,
    pub nonce: U64,
    // CAIP-2 chain ID
    pub chain_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
#[serde(rename_all = "camelCase")]
pub struct InitialTransaction {
    pub from: Address,
    pub to: Address,
    pub value: U256,
    pub data: Bytes,
    pub nonce: U64,
    // CAIP-2 chain ID
    pub chain_id: String,
}
