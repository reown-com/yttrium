use {
    super::error::L1DataFeeError,
    alloy::{
        network::TransactionBuilder,
        primitives::{address, Address, U256},
        rlp::Encodable,
        rpc::types::TransactionRequest,
        sol,
    },
    alloy_provider::Provider,
    tracing::warn,
};

// https://docs.optimism.io/builders/app-developers/transactions/fees#l1-data-fee
sol! {
    #[sol(rpc)]
    contract GasPriceOracle {
        function getL1Fee(bytes memory _data) public view returns (uint256);
    }
}
// https://github.com/wevm/viem/blob/ae3b8aeab22d56b2cf6d3b05e4f9eeaab7cf81fe/src/op-stack/contracts.ts#L8
const ORACLE_ADDRESS: Address =
    address!("420000000000000000000000000000000000000F");

pub async fn get_l1_data_fee(
    txn: TransactionRequest,
    provider: &impl Provider,
) -> Result<U256, L1DataFeeError> {
    let oracle = GasPriceOracle::new(ORACLE_ADDRESS, provider);
    let x = txn.build_unsigned().unwrap();
    let txn = x.eip1559().unwrap();
    let mut buf = Vec::with_capacity(txn.length());
    txn.encode(&mut buf);
    // txn.build_unsigned().unwrap().eip1559().unwrap().
    // let built = txn.build(wallet).await.unwrap();
    // let mut buf = Vec::with_capacity(built.eip2718_encoded_length());
    // built.as_eip1559().unwrap().rlp_encode(&mut buf);
    // println!("l1_data_fee txn: {:?}", txn);
    // println!("l1_data_fee txn hash: {}", hex::encode(keccak256(&buf)));
    let current_l1_fee = match oracle.getL1Fee(buf.into()).call().await {
        Ok(fee) => fee._0,
        Err(e) => {
            // TODO check if this error comes from the contract not existing,
            // because that means it's a chain w/o an L2 data fee
            warn!("error getting L1 data fee: {e}");
            return Ok(U256::from(0));
        }
    };
    // The fee can change a maximum of 12.5% per mainnet block: https://docs.optimism.io/builders/app-developers/transactions/fees#mechanism
    // Multiplying by 2 gives us 6 blocks of buffer, and also is simpler
    // to implement here w/ integers (vs floats)
    Ok(current_l1_fee * U256::from(2))

    // TODO also consider "blob fee" (max_fee_per_blob_gas): https://docs.optimism.io/builders/app-developers/transactions/fees#mechanism
}
