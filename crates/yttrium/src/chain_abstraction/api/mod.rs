use {
    alloy::{
        network::TransactionBuilder,
        primitives::{Address, Bytes, U128, U256, U64},
        rpc::types::TransactionRequest,
    },
    alloy_provider::utils::Eip1559Estimation,
    serde::{Deserialize, Serialize},
};

pub mod fungible_price;
pub mod prepare;
pub mod status;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
#[serde(rename_all = "camelCase")]
pub struct InitialTransaction {
    // CAIP-2 chain ID
    pub chain_id: String,

    pub from: Address,
    pub to: Address,
    pub value: U256,
    pub input: Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    // CAIP-2 chain ID
    pub chain_id: String,

    pub from: Address,
    pub to: Address,
    pub value: U256,
    pub input: Bytes,

    pub gas_limit: U64,
    pub nonce: U64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
#[serde(rename_all = "camelCase")]
pub struct FeeEstimatedTransaction {
    // CAIP-2 chain ID
    pub chain_id: String,

    pub from: Address,
    pub to: Address,
    pub value: U256,
    pub input: Bytes,

    pub gas_limit: U64,
    pub nonce: U64,

    pub max_fee_per_gas: U128,
    pub max_priority_fee_per_gas: U128,
}

impl FeeEstimatedTransaction {
    pub fn from_transaction_and_estimate(
        transaction: Transaction,
        estimate: Eip1559Estimation,
    ) -> Self {
        Self {
            chain_id: transaction.chain_id,
            from: transaction.from,
            to: transaction.to,
            value: transaction.value,
            input: transaction.input,
            gas_limit: transaction.gas_limit,
            nonce: transaction.nonce,
            max_fee_per_gas: U128::from(estimate.max_fee_per_gas),
            max_priority_fee_per_gas: U128::from(
                estimate.max_priority_fee_per_gas,
            ),
        }
    }

    pub fn into_transaction_request(self) -> TransactionRequest {
        let chain_id = self
            .chain_id
            .strip_prefix("eip155:")
            .unwrap()
            .parse::<U64>()
            .unwrap();
        TransactionRequest::default()
            .with_chain_id(chain_id.to())
            .with_from(self.from)
            .with_to(self.to)
            .with_value(self.value)
            .with_input(self.input)
            .with_gas_limit(self.gas_limit.to())
            .with_nonce(self.nonce.to())
            .with_max_fee_per_gas(self.max_fee_per_gas.to())
            .with_max_priority_fee_per_gas(self.max_priority_fee_per_gas.to())
    }
}
