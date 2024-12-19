use {
    crate::{
        chain_abstraction::api::InitialTransaction,
        config::Config,
        gas_abstraction::{
            Client as GasAbstractionClient, SignedAuthorization,
        },
    },
    alloy::{
        network::Ethereum,
        primitives::{Bytes, U256},
        signers::{local::LocalSigner, SignerSync},
    },
    alloy_provider::{Provider, ReqwestProvider},
};

#[tokio::test]
async fn happy_path() {
    // TODO remove
    let config = Config::local();
    let provider = ReqwestProvider::<Ethereum>::new_http(
        config.endpoints.rpc.base_url.parse().unwrap(),
    );
    let chain_id = format!("eip155:{}", provider.get_chain_id().await.unwrap());

    // You have a GasAbstractionClient
    let project_id = std::env::var("REOWN_PROJECT_ID").unwrap().into();
    let client = GasAbstractionClient::new(project_id, config);

    // You have an EOA
    let eoa = LocalSigner::random();

    {
        // You have an incomming eth_sendTransaction
        let txn = InitialTransaction {
            chain_id: chain_id.clone(),
            from: eoa.address(),
            to: LocalSigner::random().address(),
            value: U256::ZERO,
            input: Bytes::new(),
        };
        let result = client.prepare(txn).await.unwrap();

        assert!(result.auth.is_some());

        // Sign the authorization
        let auth_sig = result.auth.map(|auth| SignedAuthorization {
            signature: eoa.sign_hash_sync(&auth.auth.signature_hash()).unwrap(),
            auth: auth.auth,
        });

        // Ask the user for approval and sign the UserOperation
        let signature = eoa
            .sign_typed_data_sync(
                &result.prepared_send_transaction.safe_op,
                &result.prepared_send_transaction.domain,
            )
            .unwrap();

        // Send the UserOperation and get the receipt
        let receipt = client
            .send(
                auth_sig,
                signature,
                result.prepared_send_transaction.do_send_transaction_params,
            )
            .await;
        println!("receipt: {:?}", receipt);
    }

    {
        // You have an incomming eth_sendTransaction
        let txn = InitialTransaction {
            chain_id,
            from: eoa.address(),
            to: LocalSigner::random().address(),
            value: U256::ZERO,
            input: Bytes::new(),
        };
        let result = client.prepare(txn).await.unwrap();

        assert!(result.auth.is_none());

        // Ask the user for approval and sign the UserOperation
        let signature = eoa
            .sign_typed_data_sync(
                &result.prepared_send_transaction.safe_op,
                &result.prepared_send_transaction.domain,
            )
            .unwrap();

        // Send the UserOperation and get the receipt
        let receipt = client
            .send(
                None,
                signature,
                result.prepared_send_transaction.do_send_transaction_params,
            )
            .await;
        println!("receipt: {:?}", receipt);
    }

    println!("success");
}
