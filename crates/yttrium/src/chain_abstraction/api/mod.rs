use alloy::primitives::{Address, Bytes, U256, U64};
use serde::{Deserialize, Serialize};

pub mod fungible_price;
pub mod route;
pub mod status;

#[cfg(feature = "uniffi")]
uniffi::custom_type!(Address, String, {
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| obj.to_string(),
});

#[cfg(feature = "uniffi")]
uniffi::custom_type!(U256, String, {
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| obj.to_string(),
});

#[cfg(feature = "uniffi")]
uniffi::custom_type!(U64, String, {
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| obj.to_string(),
});

#[cfg(feature = "uniffi")]
uniffi::custom_type!(Bytes, String, {
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| obj.to_string(),
});

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

    // deprecated
    pub gas_price: U256,
    pub max_fee_per_gas: U256,
    pub max_priority_fee_per_gas: U256,
}
