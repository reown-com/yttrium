use {
    crate::{
        config::Config,
        gas_abstraction::{Client as GasAbstractionClient, Transaction},
    },
    alloy::{
        primitives::{Bytes, U256, U64},
        signers::local::LocalSigner,
    },
};

#[tokio::test]
async fn prepares() {
    // You have an EOA
    let eoa = LocalSigner::random();

    // You have an incomming eth_sendTransaction
    let txn = Transaction {
        chain_id: U64::from(1), // TODO
        from: eoa.address(),
        to: LocalSigner::random().address(),
        value: U256::ZERO,
        input: Bytes::new(),
    };

    // You have a GasAbstractionClient
    let project_id = std::env::var("REOWN_PROJECT_ID").unwrap().into();
    let client = GasAbstractionClient::new(project_id, Config::local());
    let _result = client.prepare_gas_abstraction(txn).await;

    // let PreparedGasAbstraction {
    //     hashes_to_sign,
    //     fields
    // } = client.prepare_gas_abstraction(txn).await?;

    // ask_user_for_approval(fields).await?;

    // let signatures = hashes_to_sign.iter().map(|hash| account.sign(hash)).collect();

    // let receipt = client.execute_transaction(txn, Params {
    //     signatures
    // }).await?;

    // display_success(receipt)
}
