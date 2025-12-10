// Based off: https://github.com/rhinestonewtf/module-sdk-tutorials/blob/main/src/smart-sessions/permissionless-safe-7702.ts

#[cfg(any(feature = "test_local_bundler", feature = "test_pimlico_api"))]
use {
    crate::{
        bundler::{
            client::BundlerClient,
            config::BundlerConfig,
            pimlico::{self, paymaster::client::PaymasterClient},
        },
        call::Call,
        config::{LOCAL_BUNDLER_URL, LOCAL_PAYMASTER_URL, LOCAL_RPC_URL},
        entry_point::ENTRYPOINT_ADDRESS_V07,
        erc7579::{
            accounts::safe::encode_validator_key,
            addresses::RHINESTONE_ATTESTER_ADDRESS,
            ownable_validator::{
                OWNABLE_VALIDATOR_ADDRESS, encode_owners,
                get_ownable_validator, get_ownable_validator_mock_signature,
            },
            policy::get_sudo_policy,
            smart_sessions::{
                ActionData, ERC7739Data, Session, encode_use_signature,
                get_permission_id, get_smart_sessions_validator,
            },
        },
        smart_accounts::{
            nonce::get_nonce_with_key,
            safe::{
                AddSafe7579Contract, Owners, SAFE_4337_MODULE_ADDRESS,
                SAFE_ERC_7579_LAUNCHPAD_ADDRESS, SAFE_L2_SINGLETON_1_4_1,
                SetupContract, get_call_data,
            },
        },
        test_helpers::anvil_faucet,
        user_operation::{UserOperationV07, hash::get_user_operation_hash_v07},
    },
    alloy::{
        network::{EthereumWallet, TransactionBuilder7702},
        primitives::{
            Address, B256, Bytes, U256, address, eip191_hash_message,
            fixed_bytes,
        },
        rlp::Encodable,
        rpc::types::Authorization,
        signers::{
            SignerSync,
            local::{LocalSigner, PrivateKeySigner},
        },
        sol_types::SolCall,
    },
    alloy_provider::{Provider, ProviderBuilder},
    reqwest::Url,
    std::time::Duration,
};

// Odyssey onramp
// cast send 0x9228665c0D8f9Fc36843572bE50B716B81e042BA \
//     --value 0.00001ether \
//     --mnemonic $FAUCET_MNEMONIC \
//     --rpc-url https://gateway.tenderly.co/public/sepolia

#[tokio::test]
#[serial_test::serial(odyssey)]
#[cfg(feature = "test_pimlico_api")]
async fn test_pimlico() {
    use {crate::test_helpers::private_faucet, std::env};

    let chain_id = 911867; // Odyssey Testnet
    let rpc = "https://odyssey.ithaca.xyz".parse().unwrap();
    let pimlico_api_key = env::var("PIMLICO_API_KEY")
        .expect("You've not set the PIMLICO_API_KEY");
    let bundler_url = format!(
        "https://api.pimlico.io/v2/{chain_id}/rpc?apikey={pimlico_api_key}"
    )
    .parse::<Url>()
    .unwrap();

    let provider = ProviderBuilder::new().connect_http(rpc);
    let faucet = private_faucet();
    test_impl(provider, faucet, bundler_url.clone(), bundler_url).await
}

#[tokio::test]
#[cfg(feature = "test_local_bundler")]
async fn test_local() {
    let provider =
        ProviderBuilder::new().connect_http(LOCAL_RPC_URL.parse().unwrap());
    let faucet = anvil_faucet(&provider).await;

    test_impl(
        provider,
        faucet,
        LOCAL_BUNDLER_URL.parse().unwrap(),
        LOCAL_PAYMASTER_URL.parse().unwrap(),
    )
    .await
}

#[cfg(any(feature = "test_local_bundler", feature = "test_pimlico_api"))]
async fn test_impl(
    provider: impl Provider + Clone,
    faucet: PrivateKeySigner,
    bundler_url: Url,
    paymaster_url: Url,
) {
    let chain_id = provider.get_chain_id().await.unwrap();
    println!("chain_id: {chain_id:?}");

    let account = LocalSigner::random();
    let safe_owner = LocalSigner::random();
    let owners = Owners { threshold: 1, owners: vec![safe_owner.address()] };

    // TODO ownableValidator
    // https://github.com/rhinestonewtf/module-sdk-tutorials/blob/656c52e200329c40ce633485bb8824df6c96ba20/src/smart-sessions/permissionless-safe-7702.ts#L80
    // https://github.com/rhinestonewtf/module-sdk/blob/main/src/module/ownable-validator/installation.ts
    let ownable_validator = get_ownable_validator(&owners, None);

    let session_owner = LocalSigner::random();
    let session_owners =
        Owners { threshold: 1, owners: vec![session_owner.address()] };

    let session = Session {
        sessionValidator: OWNABLE_VALIDATOR_ADDRESS,
        sessionValidatorInitData: encode_owners(&session_owners),
        salt: B256::default(),
        userOpPolicies: vec![get_sudo_policy()],
        erc7739Policies: ERC7739Data {
            allowedERC7739Content: vec![],
            erc1271Policies: vec![],
        },
        actions: vec![ActionData {
            actionTarget: address!("a564cB165815937967a7d018B7F34B907B52fcFd"), /* an address as the target of the session execution */
            actionTargetSelector: fixed_bytes!("00000000"), /* function selector to be used in the execution, in this case no function selector is used */
            actionPolicies: vec![get_sudo_policy()],
        }],
        permitERC4337Paymaster: true,
    };

    let smart_sessions =
        get_smart_sessions_validator(std::slice::from_ref(&session), None);

    let auth_7702 = Authorization {
        chain_id: U256::from(chain_id),
        address: SAFE_L2_SINGLETON_1_4_1,
        // TODO should this be `pending` tag? https://github.com/wevm/viem/blob/a49c100a0b2878fbfd9f1c9b43c5cc25de241754/src/experimental/eip7702/actions/signAuthorization.ts#L149
        nonce: provider.get_transaction_count(account.address()).await.unwrap(),
    };

    // Sign the authorization
    let sig = account.sign_hash_sync(&auth_7702.signature_hash()).unwrap();
    let auth = auth_7702.into_signed(sig);

    println!("using faucet: {}", faucet.address());
    println!("faucet private key: {}", hex::encode(faucet.to_bytes()));
    let faucet_wallet = EthereumWallet::new(faucet);
    let faucet_provider = ProviderBuilder::new()
        .wallet(faucet_wallet)
        .connect_provider(provider.clone());

    println!("account address: {}", account.address());
    // let mut buf = Vec::new();
    // auth.encode(&mut buf);
    // println!("auth: {}", hex::encode(buf));
    // let sent_txn = faucet_provider
    //     .send_transaction(
    //         TransactionRequest::default()
    //             .with_to(account.address())
    //             .with_input(Bytes::new())
    //             .with_authorization_list(vec![auth]),
    //     )
    //     .await
    //     .unwrap();
    // println!("txn hash: {}", sent_txn.tx_hash());
    // let receipt = sent_txn
    //     .with_timeout(Some(Duration::from_secs(20)))
    //     .get_receipt()
    //     .await
    //     .unwrap();
    // println!("receipt: {:?}", receipt);
    // assert!(receipt.status());
    // return;

    let sent_txn =
        SetupContract::new(account.address(), faucet_provider.clone())
            .setup(
                owners.owners,
                U256::from(owners.threshold),
                SAFE_ERC_7579_LAUNCHPAD_ADDRESS,
                AddSafe7579Contract::addSafe7579Call {
                    safe7579: SAFE_4337_MODULE_ADDRESS,
                    validators: vec![
                        AddSafe7579Contract::ModuleInit {
                            module: ownable_validator.address,
                            initData: ownable_validator.init_data,
                        },
                        AddSafe7579Contract::ModuleInit {
                            module: smart_sessions.address,
                            initData: smart_sessions.init_data,
                        },
                    ],
                    executors: vec![],
                    fallbacks: vec![],
                    hooks: vec![],
                    attesters: vec![
                        RHINESTONE_ATTESTER_ADDRESS,
                        // MOCK_ATTESTER_ADDRESS,
                    ],
                    threshold: 1,
                }
                .abi_encode()
                .into(),
                SAFE_4337_MODULE_ADDRESS,
                Address::ZERO,
                U256::ZERO,
                Address::ZERO,
            )
            .map(|mut t| {
                println!("t: {t:?}");
                println!("t.chain_id: {:?}", t.chain_id);
                let mut buf = Vec::new();
                auth.encode(&mut buf);
                println!("auth: {}", hex::encode(buf));
                t.set_authorization_list(vec![auth]);
                // t.set_nonce(1);
                // t.set_gas_limit(1000000);
                // t.set_max_fee_per_gas(252);
                // t.set_max_priority_fee_per_gas(0);
                // t.set_chain_id(chain_id);
                t
            })
            .send()
            .await
            .unwrap();
    println!("txn hash: {}", sent_txn.tx_hash());
    let receipt = sent_txn
        .with_timeout(Some(Duration::from_secs(20)))
        .get_receipt()
        .await
        .unwrap();
    println!("receipt: {receipt:?}");
    assert!(receipt.status());

    let nonce = get_nonce_with_key(
        &provider,
        account.address().into(),
        &ENTRYPOINT_ADDRESS_V07.into(),
        encode_validator_key(smart_sessions.address),
    )
    .await
    .unwrap();

    let permission_id = get_permission_id(&session);
    let smart_session_dummy_signature = encode_use_signature(
        permission_id,
        get_ownable_validator_mock_signature(&session_owners),
    );

    let pimlico_client = pimlico::client::BundlerClient::new(
        BundlerConfig::new(bundler_url.clone()),
    );
    let bundler_client = BundlerClient::new(BundlerConfig::new(bundler_url));
    let paymaster_client =
        PaymasterClient::new(BundlerConfig::new(paymaster_url));

    let gas_price =
        pimlico_client.estimate_user_operation_gas_price().await.unwrap().fast;
    let user_op = UserOperationV07 {
        sender: account.address().into(),
        nonce,
        factory: None,
        factory_data: None,
        call_data: get_call_data(vec![Call {
            to: session.actions[0].actionTarget,
            value: U256::ZERO,
            input: session.actions[0].actionTargetSelector.into(),
        }]),
        call_gas_limit: U256::ZERO,
        verification_gas_limit: U256::ZERO,
        pre_verification_gas: U256::ZERO,
        max_fee_per_gas: gas_price.max_fee_per_gas,
        max_priority_fee_per_gas: gas_price.max_priority_fee_per_gas,
        paymaster: None,
        paymaster_verification_gas_limit: None,
        paymaster_post_op_gas_limit: None,
        paymaster_data: None,
        signature: smart_session_dummy_signature,
    };

    let user_op = {
        let sponsor_user_op_result = paymaster_client
            .sponsor_user_operation_v07(
                &user_op.clone().into(),
                &ENTRYPOINT_ADDRESS_V07.into(),
                None,
            )
            .await
            .unwrap();

        UserOperationV07 {
            call_gas_limit: sponsor_user_op_result.call_gas_limit,
            verification_gas_limit: sponsor_user_op_result
                .verification_gas_limit,
            pre_verification_gas: sponsor_user_op_result.pre_verification_gas,
            paymaster: Some(sponsor_user_op_result.paymaster),
            paymaster_verification_gas_limit: Some(
                sponsor_user_op_result.paymaster_verification_gas_limit,
            ),
            paymaster_post_op_gas_limit: Some(
                sponsor_user_op_result.paymaster_post_op_gas_limit,
            ),
            paymaster_data: Some(sponsor_user_op_result.paymaster_data),
            ..user_op
        }
    };

    let user_op_hash_to_sign = get_user_operation_hash_v07(
        &user_op,
        &ENTRYPOINT_ADDRESS_V07,
        chain_id,
    );

    let user_op = UserOperationV07 {
        signature: encode_use_signature(
            permission_id,
            session_owner
                .sign_hash_sync(&eip191_hash_message(user_op_hash_to_sign.0))
                .unwrap()
                .as_bytes()
                .into(),
        ),
        ..user_op
    };

    println!("User operation: {user_op:?}");
    let user_op_hash = bundler_client
        .send_user_operation(ENTRYPOINT_ADDRESS_V07.into(), user_op.clone())
        .await
        .unwrap();
    println!("User operation hash: {user_op_hash:?}");
    assert_eq!(Bytes::from(user_op_hash_to_sign.0), user_op_hash);

    let receipt = bundler_client
        .wait_for_user_operation_receipt(user_op_hash)
        .await
        .unwrap();
    println!("User operation receipt: {receipt:?}");
    assert!(receipt.success);
}
