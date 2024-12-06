use {
    alloy::primitives::U256,
    serde::{Deserialize, Serialize},
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EstimateResult {
    pub call_gas_limit: U256,
    pub pre_verification_gas: U256,
    pub verification_gas_limit: U256,
    pub paymaster_verification_gas_limit: Option<U256>,
    pub paymaster_post_op_gas_limit: Option<U256>,
}
