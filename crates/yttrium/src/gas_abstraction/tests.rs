use {
    crate::{
        chain_abstraction::api::InitialTransaction,
        config::{LOCAL_BUNDLER_URL, LOCAL_PAYMASTER_URL, LOCAL_RPC_URL},
        gas_abstraction::{
            Client as GasAbstractionClient, PreparedGasAbstraction,
            SignedAuthorization,
        },
    },
    alloy::{
        network::Ethereum,
        primitives::{Bytes, U256},
        signers::{local::LocalSigner, SignerSync},
    },
    alloy_provider::{Provider, ReqwestProvider},
    std::collections::HashMap,
};

#[tokio::test]
async fn happy_path() {
    let chain_id = format!(
        "eip155:{}",
        ReqwestProvider::<Ethereum>::new_http(LOCAL_RPC_URL.parse().unwrap(),)
            .get_chain_id()
            .await
            .unwrap()
    );

    // You have a GasAbstractionClient
    // TODO allow Sponsor EOA as configuration - for non-Anvil usage i.e. TODO Pimlico test case against testnet
    // let project_id = std::env::var("REOWN_PROJECT_ID").unwrap().into();
    let project_id = "".into();
    let client = GasAbstractionClient::new(project_id)
        .with_rpc_overrides(HashMap::from([(
            chain_id.clone(),
            LOCAL_RPC_URL.parse().unwrap(),
        )]))
        .with_4337_urls(
            LOCAL_BUNDLER_URL.to_owned(),
            LOCAL_PAYMASTER_URL.to_owned(),
        );

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

        let result = client.prepare(txn).await;
        assert!(matches!(
            result,
            PreparedGasAbstraction::DeploymentRequired { .. }
        ));
        let (auth, prepare_deploy_params) = match result {
            PreparedGasAbstraction::DeploymentRequired {
                auth,
                prepare_deploy_params,
            } => (auth, prepare_deploy_params),
            PreparedGasAbstraction::DeploymentNotRequired { .. } => {
                panic!("unexpected")
            }
        };

        // Display disclaimer info to the user
        // User approved? Yes

        let auth_sig = SignedAuthorization {
            signature: eoa.sign_hash_sync(&auth.auth.signature_hash()).unwrap(),
            auth: auth.auth,
        };
        let prepared_send =
            client.prepare_deploy(auth_sig, prepare_deploy_params, None).await;

        // Display fee information to the user: prepare_deploy_result.fees
        // User approved? Yes

        let signature: alloy::signers::Signature =
            eoa.sign_hash_sync(&prepared_send.hash).unwrap();
        let receipt = client.send(signature, prepared_send.send_params).await;
        println!("receipt: {:?}", receipt);
    }

    // Second eth_sendTransaction
    {
        // You have an incomming eth_sendTransaction
        let txn = InitialTransaction {
            chain_id,
            from: eoa.address(),
            to: LocalSigner::random().address(),
            value: U256::ZERO,
            input: Bytes::new(),
        };

        let result = client.prepare(txn).await;
        assert!(matches!(
            result,
            PreparedGasAbstraction::DeploymentNotRequired { .. }
        ));
        let prepared_send = match result {
            PreparedGasAbstraction::DeploymentNotRequired { prepared_send } => {
                prepared_send
            }
            PreparedGasAbstraction::DeploymentRequired { .. } => {
                panic!("unexpected")
            }
        };

        // Display fee information to the user: prepare.fees
        // User approved? Yes

        let signature = eoa.sign_hash_sync(&prepared_send.hash).unwrap();
        let receipt = client.send(signature, prepared_send.send_params).await;
        println!("receipt: {:?}", receipt);
    }

    println!("success");
}
