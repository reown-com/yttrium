use alloy::primitives::{
    b256, Address, BlockHash, Bytes, TxHash, B256, U128, U64, U8,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionReceipt {
    pub transaction_hash: TxHash,
    pub transaction_index: String,
    pub block_hash: BlockHash,
    pub block_number: U64,
    pub from: Address,
    pub to: Address,
    pub cumulative_gas_used: String,
    pub gas_used: U128,
    pub contract_address: Option<String>,
    pub status: U8,
    pub logs_bloom: String,
    // pub r#type: String,
    pub effective_gas_price: String,
}

// TODO replace with alloy's UserOperationReceipt
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOperationReceipt {
    pub user_op_hash: B256,
    pub entry_point: Address,
    pub sender: Address,
    pub nonce: String,
    pub paymaster: String,
    pub actual_gas_cost: String,
    pub actual_gas_used: String,
    pub success: bool,
    // pub reason: String,
    pub receipt: TransactionReceipt,
    pub logs: Vec<TransactionLog>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionLog {
    pub address: Address,
    pub topics: Vec<B256>,
    pub data: Bytes,
}

impl UserOperationReceipt {
    pub fn mock() -> Self {
        UserOperationReceipt {
            user_op_hash: b256!("93c06f3f5909cc2b192713ed9bf93e3e1fde4b22fcd2466304fa404f9b80ff90"),
            entry_point: "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789"
                .parse()
                .unwrap(),
            sender: "0x9E1276a4A64D064256E7347cdA4d8C8039b1bc48".parse().unwrap(),
            nonce: "0x3".to_string(),
            paymaster: "0xb80bCD1Bcf735238EAB64ffc3431076605A21D61".to_string(),
            actual_gas_cost: "0x11bed797b2d5c8".to_string(),
            actual_gas_used: "0x20725".to_string(),
            success: true,
            // reason: "".to_string(),
            receipt: TransactionReceipt {
                transaction_hash: b256!("68b5465c1efe05e5a29f8551c3808e5fd3b0a46e7abb007e11c586632cf46c23"),
                transaction_index: "0x85".to_string(),
                block_hash: b256!("0b95eb450c36397458e77e38420b89f0b6336b7c61b7bbb9898e0318da0f4cd0"),
                block_number: "0x113fc81".parse().unwrap(),
                from: "0x374a2c4dcb38ecbb606117ae1bfe402a52176ec1".parse().unwrap(),
                to: "0x5ff137d4b0fdcd49dca30c7cf57e578a026d2789".parse().unwrap(),
                cumulative_gas_used: "0x12bafe6".to_string(),
                gas_used: "0x20d07".parse().unwrap(),
                contract_address: None,
                status: U8::from(1),
                logs_bloom: "0x04400000000040002000000000000000000000000000000000000000000000000008000000000000000200010000000000100000000000000000020000000000000000000000000000000008000000000100000000000000000000000000000000000000080000000008000000000000000000000000000000000010000000000000000000040040100088000000000000000000000000000000000000000000000000000000000100400000000008000000000000000000000002000000000000000002000000100001000000000000000000002000000000000040000000000000000000000000200000000000000000000000000000000000000000000010".to_string(),
                // r#type: "0x2".to_string(),
                effective_gas_price: "0x86cb70a28".to_string(),
            },
            logs: vec![],
        }
    }
}
