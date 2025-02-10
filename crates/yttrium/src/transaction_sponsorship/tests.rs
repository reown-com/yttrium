use {
    crate::{
        call::Call,
        chain_abstraction::pulse::PulseMetadata,
        config::{LOCAL_BUNDLER_URL, LOCAL_PAYMASTER_URL, LOCAL_RPC_URL},
        transaction_sponsorship::{
            Client as GasAbstractionClient, PreparedGasAbstraction,
            SignedAuthorization,
        },
    },
    alloy::{
        network::Ethereum,
        primitives::{Bytes, U256},
        signers::{
            local::{LocalSigner, PrivateKeySigner},
            SignerSync,
        },
    },
    alloy_provider::{Provider, ReqwestProvider},
    std::collections::HashMap,
};

#[tokio::test]
#[serial_test::serial(odyssey)]
#[cfg(feature = "test_pimlico_api")]
async fn happy_path_pimlico() {
    use {crate::test_helpers::private_faucet, reqwest::Url, std::env};

    let chain_id = 911867; // Odyssey Testnet
    let rpc = "https://odyssey.ithaca.xyz".parse::<Url>().unwrap();
    let pimlico_api_key = env::var("PIMLICO_API_KEY")
        .expect("You've not set the PIMLICO_API_KEY");
    let bundler_url = format!(
        "https://api.pimlico.io/v2/{chain_id}/rpc?apikey={pimlico_api_key}"
    )
    .parse::<Url>()
    .unwrap();
    let chain_id = format!("eip155:{chain_id}");

    let project_id = "".into();
    let client = GasAbstractionClient::new(
        project_id,
        PulseMetadata {
            url: None,
            bundle_id: None,
            package_name: None,
            sdk_version: "yttrium-tests-0.0.0".to_owned(),
            sdk_platform: "desktop".to_owned(),
        },
    )
    .with_rpc_overrides(HashMap::from([(chain_id.clone(), rpc)]))
    .with_4337_urls(bundler_url.clone(), bundler_url);

    let faucet = private_faucet();
    happy_path_impl(chain_id, client, Some(faucet)).await
}

#[tokio::test]
async fn happy_path_local() {
    let chain_id = format!(
        "eip155:{}",
        ReqwestProvider::<Ethereum>::new_http(LOCAL_RPC_URL.parse().unwrap())
            .get_chain_id()
            .await
            .unwrap()
    );

    let project_id = "".into();
    let client = GasAbstractionClient::new(
        project_id,
        PulseMetadata {
            url: None,
            bundle_id: None,
            package_name: None,
            sdk_version: "yttrium-tests-0.0.0".to_owned(),
            sdk_platform: "desktop".to_owned(),
        },
    )
    .with_rpc_overrides(HashMap::from([(
        chain_id.clone(),
        LOCAL_RPC_URL.parse().unwrap(),
    )]))
    .with_4337_urls(
        LOCAL_BUNDLER_URL.parse().unwrap(),
        LOCAL_PAYMASTER_URL.parse().unwrap(),
    );

    happy_path_impl(chain_id, client, None).await
}

async fn happy_path_impl(
    chain_id: String,
    client: GasAbstractionClient,
    sponsor: Option<PrivateKeySigner>,
) {
    // You have an EOA
    let eoa = LocalSigner::random();
    let from = eoa.address();

    {
        // You have an incomming eth_sendTransaction
        let txn = vec![Call {
            to: LocalSigner::random().address(),
            value: U256::ZERO,
            input: Bytes::new(),
        }];

        let result = client.prepare(chain_id.clone(), from, txn).await.unwrap();
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
        let prepared_send = client
            .prepare_deploy(auth_sig, prepare_deploy_params, sponsor)
            .await
            .unwrap();

        // Display fee information to the user: prepare_deploy_result.fees
        // User approved? Yes

        let signature = eoa.sign_hash_sync(&prepared_send.hash).unwrap();
        let receipt = client.send(signature, prepared_send.send_params).await;
        println!("receipt: {:?}", receipt);
    }

    // Second eth_sendTransaction
    {
        // You have an incomming eth_sendTransaction
        let calls = vec![Call {
            to: LocalSigner::random().address(),
            value: U256::ZERO,
            input: Bytes::new(),
        }];

        let result =
            client.prepare(chain_id.clone(), from, calls).await.unwrap();
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

    // Third eth_sendTransaction (2 calls)
    {
        // You have an incomming eth_sendTransaction
        let calls = vec![
            Call {
                to: LocalSigner::random().address(),
                value: U256::ZERO,
                input: Bytes::new(),
            },
            Call {
                to: LocalSigner::random().address(),
                value: U256::ZERO,
                input: Bytes::new(),
            },
        ];

        let result = client.prepare(chain_id, from, calls).await.unwrap();
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
