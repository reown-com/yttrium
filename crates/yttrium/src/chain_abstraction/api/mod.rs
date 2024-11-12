use alloy::primitives::Address;
use serde::{Deserialize, Serialize};

pub mod route;
pub mod status;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub from: Address,
    pub to: Address,
    pub value: String,
    pub gas: String,
    pub gas_price: String,
    pub data: String,
    pub nonce: String,
    pub max_fee_per_gas: String,
    pub max_priority_fee_per_gas: String,
    pub chain_id: String,
}
