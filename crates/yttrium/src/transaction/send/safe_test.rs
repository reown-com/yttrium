use crate::{
    bundler::{
        client::BundlerClient,
        config::BundlerConfig,
        pimlico::{
            client::BundlerClient as PimlicoBundlerClient,
            paymaster::client::PaymasterClient,
        },
    },
    config::Config,
    smart_accounts::{
        nonce::get_nonce,
        safe::{
            factory_data, get_account_address, Execution, Owners, Safe7579,
            Safe7579Launchpad, SAFE_4337_MODULE_ADDRESS,
            SAFE_ERC_7579_LAUNCHPAD_ADDRESS, SAFE_PROXY_FACTORY_ADDRESS,
            SEPOLIA_SAFE_ERC_7579_SINGLETON_ADDRESS,
        },
        simple_account::{factory::FactoryAddress, SimpleAccountAddress},
    },
    user_operation::{Authorization, UserOperationV07},
};
use alloy::{
    dyn_abi::{
        DynSolCall, DynSolReturns, DynSolType, DynSolValue, Eip712Domain,
    },
    network::Ethereum,
    primitives::{
        aliases::U48, keccak256, Address, Bytes, FixedBytes, Uint, U128, U256,
    },
    providers::{Provider, ReqwestProvider},
    signers::{k256::ecdsa::SigningKey, local::LocalSigner, SignerSync},
    sol,
    sol_types::{SolCall, SolValue},
};
use core::fmt;
use serde_json::json;
use std::{ops::Not, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct UserOperationEstimated(UserOperationV07);

impl From<UserOperationEstimated> for UserOperationV07 {
    fn from(val: UserOperationEstimated) -> Self {
        val.0
    }
}

#[derive(Debug, Clone)]
pub struct SignedUserOperation(UserOperationV07);

impl From<SignedUserOperation> for UserOperationV07 {
    fn from(val: SignedUserOperation) -> Self {
        val.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SentUserOperationHash(String);

impl From<SentUserOperationHash> for String {
    fn from(user_operation_hash: SentUserOperationHash) -> Self {
        user_operation_hash.0
    }
}

impl fmt::Display for SentUserOperationHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub async fn get_address(
    owner: LocalSigner<SigningKey>,
    config: Config,
) -> eyre::Result<Address> {
    let rpc_url = config.endpoints.rpc.base_url;
    let rpc_url: reqwest::Url = rpc_url.parse()?;
    let provider = ReqwestProvider::<Ethereum>::new_http(rpc_url);

    let owners = Owners { owners: vec![owner.address()], threshold: 1 };

    Ok(get_account_address(provider.clone(), owners.clone()).await)
}

pub async fn send_transaction(
    execution_calldata: Vec<Execution>,
    owner: LocalSigner<SigningKey>,
    address: Option<Address>,
    authorization_list: Option<Vec<Authorization>>,
    config: Config,
) -> eyre::Result<String> {
    let bundler_base_url = config.endpoints.bundler.base_url;
    let paymaster_base_url = config.endpoints.paymaster.base_url;

    let bundler_client =
        BundlerClient::new(BundlerConfig::new(bundler_base_url.clone()));

    let pimlico_client: PimlicoBundlerClient =
        PimlicoBundlerClient::new(BundlerConfig::new(bundler_base_url.clone()));

    let chain = crate::chain::Chain::ETHEREUM_SEPOLIA_V07;
    let entry_point_config = chain.entry_point_config();

    let chain_id = chain.id.eip155_chain_id();

    let entry_point_address = entry_point_config.address();

    let rpc_url = config.endpoints.rpc.base_url;

    let rpc_url: reqwest::Url = rpc_url.parse()?;
    let provider = ReqwestProvider::<Ethereum>::new_http(rpc_url);

    let safe_factory_address_primitives: Address = SAFE_PROXY_FACTORY_ADDRESS;
    let safe_factory_address =
        FactoryAddress::new(safe_factory_address_primitives);

    let owners = Owners { owners: vec![owner.address()], threshold: 1 };

    let factory_data_value = factory_data(owners.clone()).abi_encode();

    let contract_address =
        get_account_address(provider.clone(), owners.clone()).await;
    let account_address =
        if let Some(address) = address { address } else { contract_address };

    let call_data = {
        let batch = execution_calldata.len() != 1;
        let revert_on_error = false;
        let selector = [0u8; 4];
        let context = [0u8; 22];

        let mode =
            DynSolValue::Tuple(vec![
                DynSolValue::Uint(
                    Uint::from(if batch { 0x01 } else { 0x00 }),
                    8,
                ), // DelegateCall is 0xFF
                DynSolValue::Uint(Uint::from(revert_on_error as u8), 8),
                DynSolValue::Bytes(vec![0u8; 4]),
                DynSolValue::Bytes(selector.to_vec()),
                DynSolValue::Bytes(context.to_vec()),
            ])
            .abi_encode_packed();

        let execution_calldata = if batch {
            DynSolCall::new(
                FixedBytes::from_slice(
                    &keccak256("executionBatch(tuple[])")[..4],
                ),
                vec![DynSolType::Array(Box::new(DynSolType::Tuple(vec![
                    DynSolType::Address,
                    DynSolType::Uint(32),
                    DynSolType::Bytes,
                ])))],
                None,
                DynSolReturns::new(vec![]),
            )
            .abi_encode_input(&[DynSolValue::Array(
                execution_calldata
                    .iter()
                    .map(|execution| {
                        DynSolValue::Tuple(vec![
                            DynSolValue::Address(execution.target),
                            DynSolValue::Uint(Uint::from(execution.value), 32),
                            DynSolValue::Bytes(execution.callData.to_vec()),
                        ])
                    })
                    .collect(),
            )])?[4..]
                .to_vec()
        } else {
            execution_calldata.abi_encode_packed()
        }
        .into();

        Safe7579::executeCall {
            mode: FixedBytes::from_slice(&mode),
            executionCalldata: execution_calldata,
        }
        .abi_encode()
        .into()
    };

    let deployed = provider.get_code_at(account_address).await?.len() > 0;
    println!("Deployed: {}", deployed);
    // permissionless: signerToSafeSmartAccount -> encodeCallData
    let call_data = if deployed {
        call_data
    } else {
        Safe7579Launchpad::setupSafeCall {
            initData: Safe7579Launchpad::InitData {
                singleton: SEPOLIA_SAFE_ERC_7579_SINGLETON_ADDRESS,
                owners: owners.owners,
                threshold: U256::from(owners.threshold),
                setupTo: SAFE_ERC_7579_LAUNCHPAD_ADDRESS,
                setupData: Safe7579Launchpad::initSafe7579Call {
                    safe7579: SAFE_4337_MODULE_ADDRESS,
                    executors: vec![],
                    fallbacks: vec![],
                    hooks: vec![],
                    attesters: vec![],
                    threshold: 0,
                }
                .abi_encode()
                .into(),
                safe7579: SAFE_4337_MODULE_ADDRESS,
                callData: call_data,
                validators: vec![],
            },
        }
        .abi_encode()
        .into()
    };

    let gas_price = pimlico_client.estimate_user_operation_gas_price().await?;

    assert!(gas_price.fast.max_fee_per_gas > U256::from(1));

    let nonce = get_nonce(
        &provider,
        &SimpleAccountAddress::new(account_address),
        &entry_point_address,
    )
    .await?;

    let user_op = UserOperationV07 {
        sender: account_address,
        nonce: U256::from(nonce),
        factory: deployed.not().then(|| safe_factory_address.to_address()),
        factory_data: deployed.not().then(|| factory_data_value.into()),
        call_data,
        call_gas_limit: U256::from(0),
        verification_gas_limit: U256::from(0),
        pre_verification_gas: U256::from(0),
        max_fee_per_gas: gas_price.fast.max_fee_per_gas,
        max_priority_fee_per_gas: gas_price.fast.max_priority_fee_per_gas,
        paymaster: None,
        paymaster_verification_gas_limit: None,
        paymaster_post_op_gas_limit: None,
        paymaster_data: None,
        // authorization_list: None,
        signature: Bytes::from_str(
            crate::smart_accounts::safe::DUMMY_SIGNATURE_HEX
                .strip_prefix("0x")
                .unwrap(),
        )?,
    };

    if let Some(authorization_list) = authorization_list {
        let response = reqwest::Client::new()
                .post(bundler_base_url.clone())
                .json(&json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "method": "eth_prepareSendUserOperation7702",
                        "params": [
                            format!("{}:{}:{}", user_op.sender, user_op.nonce, user_op.call_data),
                            authorization_list,
                        ],
                }))
                .send()
                .await
                .unwrap();
        let success = response.status().is_success();
        println!("response: {:?}", response.text().await);

        assert!(success);
    }

    let paymaster_client =
        PaymasterClient::new(BundlerConfig::new(paymaster_base_url.clone()));

    let sponsor_user_op_result = paymaster_client
        .sponsor_user_operation_v07(
            &user_op.clone().into(),
            &entry_point_address,
            None,
        )
        .await?;

    let sponsored_user_op = {
        let s = sponsor_user_op_result.clone();
        let mut op = user_op.clone();

        op.call_gas_limit = s.call_gas_limit;
        op.verification_gas_limit = s.verification_gas_limit;
        op.pre_verification_gas = s.pre_verification_gas;
        op.paymaster = Some(s.paymaster);
        op.paymaster_verification_gas_limit =
            Some(s.paymaster_verification_gas_limit);
        op.paymaster_post_op_gas_limit = Some(s.paymaster_post_op_gas_limit);
        op.paymaster_data = Some(s.paymaster_data);

        op
    };

    let valid_after = U48::from(0);
    let valid_until = U48::from(0);

    sol!(
        struct SafeOp {
            address safe;
            uint256 nonce;
            bytes initCode;
            bytes callData;
            uint128 verificationGasLimit;
            uint128 callGasLimit;
            uint256 preVerificationGas;
            uint128 maxPriorityFeePerGas;
            uint128 maxFeePerGas;
            bytes paymasterAndData;
            uint48 validAfter;
            uint48 validUntil;
            address entryPoint;
        }
    );

    // TODO handle panic
    fn coerce_u256_to_u128(u: U256) -> U128 {
        U128::from(u)
    }

    let message = SafeOp {
        safe: account_address,
        callData: sponsored_user_op.call_data.clone(),
        nonce: sponsored_user_op.nonce,
        initCode: deployed
            .not()
            .then(|| {
                [
                    sponsored_user_op.clone().factory.unwrap().to_vec().into(),
                    sponsored_user_op.clone().factory_data.unwrap(),
                ]
                .concat()
                .into()
            })
            .unwrap_or(Bytes::new()),
        maxFeePerGas: u128::from_be_bytes(
            coerce_u256_to_u128(sponsored_user_op.max_fee_per_gas)
                .to_be_bytes(),
        ),
        maxPriorityFeePerGas: u128::from_be_bytes(
            coerce_u256_to_u128(sponsored_user_op.max_priority_fee_per_gas)
                .to_be_bytes(),
        ),
        preVerificationGas: sponsored_user_op.pre_verification_gas,
        verificationGasLimit: u128::from_be_bytes(
            coerce_u256_to_u128(sponsored_user_op.verification_gas_limit)
                .to_be_bytes(),
        ),
        callGasLimit: u128::from_be_bytes(
            coerce_u256_to_u128(sponsored_user_op.call_gas_limit).to_be_bytes(),
        ),
        // signerToSafeSmartAccount -> getPaymasterAndData
        paymasterAndData: sponsored_user_op
            .paymaster
            .map(|paymaster| {
                [
                    paymaster.to_vec(),
                    coerce_u256_to_u128(
                        sponsored_user_op
                            .paymaster_verification_gas_limit
                            .unwrap_or(Uint::from(0)),
                    )
                    .to_be_bytes_vec(),
                    coerce_u256_to_u128(
                        sponsored_user_op
                            .paymaster_post_op_gas_limit
                            .unwrap_or(Uint::from(0)),
                    )
                    .to_be_bytes_vec(),
                    sponsored_user_op
                        .paymaster_data
                        .clone()
                        .unwrap_or(Bytes::new())
                        .to_vec(),
                ]
                .concat()
                .into()
            })
            .unwrap_or(Bytes::new()),
        validAfter: valid_after,
        validUntil: valid_until,
        entryPoint: entry_point_address.to_address(),
    };

    let erc7579_launchpad_address = true;
    let verifying_contract = if erc7579_launchpad_address && !deployed {
        sponsored_user_op.sender
    } else {
        SAFE_4337_MODULE_ADDRESS
    };

    // TODO loop per-owner
    let signature = owner.sign_typed_data_sync(
        &message,
        &Eip712Domain {
            chain_id: Some(Uint::from(chain_id)),
            verifying_contract: Some(verifying_contract),
            ..Default::default()
        },
    )?;
    // TODO sort by (lowercase) owner address not signature data
    let mut signatures =
        [signature].iter().map(|sig| sig.as_bytes()).collect::<Vec<_>>();
    signatures.sort();
    let signature_bytes = signatures.concat();

    let signature = DynSolValue::Tuple(vec![
        DynSolValue::Uint(Uint::from(valid_after), 48),
        DynSolValue::Uint(Uint::from(valid_until), 48),
        DynSolValue::Bytes(signature_bytes),
    ])
    .abi_encode_packed()
    .into();
    let signed_user_op = UserOperationV07 { signature, ..sponsored_user_op };

    println!("Generated signature: {:?}", signed_user_op.signature);

    let user_operation_hash = bundler_client
        .send_user_operation(
            entry_point_address.to_address(),
            signed_user_op.clone(),
        )
        .await?;

    println!("Received User Operation hash: {:?}", user_operation_hash);

    println!("Querying for receipts...");

    let receipt = bundler_client
        .wait_for_user_operation_receipt(user_operation_hash.clone())
        .await?;

    let tx_hash = receipt.receipt.transaction_hash;
    println!(
        "SAFE UserOperation included: https://sepolia.etherscan.io/tx/{}",
        tx_hash
    );

    // Some extra calls to wait for/get the actual transaction. But these
    // aren't required since eth_getUserOperationReceipt already waits
    // let tx_hash = FixedBytes::from_slice(
    //     &hex::decode(tx_hash.strip_prefix("0x").unwrap()).unwrap(),
    // );
    // let pending_txn = provider
    //     .watch_pending_transaction(PendingTransactionConfig::new(tx_hash))
    //     .await?;
    // pending_txn.await.unwrap();
    // let transaction = provider.get_transaction_by_hash(tx_hash).await?;
    // println!("Transaction included: {:?}", transaction);
    // let transaction_receipt =
    //     provider.get_transaction_receipt(tx_hash).await?;
    // println!("Transaction receipt: {:?}", transaction_receipt);

    Ok(user_operation_hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::ChainId;
    use alloy::{
        consensus::{SignableTransaction, TxEip7702},
        network::{EthereumWallet, TransactionBuilder, TxSignerSync},
        primitives::U64,
        providers::{ext::AnvilApi, PendingTransactionConfig, ProviderBuilder},
        rpc::types::TransactionRequest,
    };

    async fn use_faucet(
        provider: ReqwestProvider,
        faucet: LocalSigner<SigningKey>,
        amount: U256,
        to: Address,
    ) -> eyre::Result<()> {
        // Basic check (which we can tune) to make sure we don't use excessive
        // amounts (e.g. 0.1) of test ETH. It is not infinite, so we should use
        // the minimum amount necessary.
        assert!(amount < U256::from(20), "You probably don't need that much");

        ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(EthereumWallet::new(faucet))
            .on_provider(provider.clone())
            .send_transaction(
                TransactionRequest::default().with_to(to).with_value(amount),
            )
            .await?
            .watch()
            .await?;
        let balance = provider.get_balance(to).await?;
        assert_eq!(balance, amount);

        Ok(())
    }

    async fn test_send_transaction(
        config: Config,
        faucet: LocalSigner<SigningKey>,
    ) -> eyre::Result<()> {
        let provider = ReqwestProvider::<Ethereum>::new_http(
            config.endpoints.rpc.base_url.parse()?,
        );

        let destination = LocalSigner::random();
        let balance = provider.get_balance(destination.address()).await?;
        assert_eq!(balance, Uint::from(0));

        let owner = LocalSigner::random();
        let sender_address = get_account_address(
            provider.clone(),
            Owners { owners: vec![owner.address()], threshold: 1 },
        )
        .await;

        use_faucet(
            provider.clone(),
            faucet.clone(),
            U256::from(2),
            sender_address,
        )
        .await?;

        let transaction = vec![Execution {
            target: destination.address(),
            value: Uint::from(1),
            callData: Bytes::new(),
        }];

        let transaction_hash = send_transaction(
            transaction,
            owner.clone(),
            None,
            None,
            config.clone(),
        )
        .await?;

        println!("Transaction sent: {}", transaction_hash);

        let balance = provider.get_balance(destination.address()).await?;
        assert_eq!(balance, Uint::from(1));

        let transaction = vec![Execution {
            target: destination.address(),
            value: Uint::from(1),
            callData: Bytes::new(),
        }];

        let transaction_hash =
            send_transaction(transaction, owner, None, None, config).await?;

        println!("Transaction sent: {}", transaction_hash);

        let balance = provider.get_balance(destination.address()).await?;
        assert_eq!(balance, Uint::from(2));

        Ok(())
    }

    async fn anvil_faucet(config: Config) -> LocalSigner<SigningKey> {
        let faucet = LocalSigner::random();
        let provider = ReqwestProvider::<Ethereum>::new_http(
            config.endpoints.rpc.base_url.parse().unwrap(),
        );
        provider.anvil_set_balance(faucet.address(), U256::MAX).await.unwrap();
        faucet
    }

    #[tokio::test]
    async fn test_send_transaction_local() {
        let config = Config::local();
        let faucet = anvil_faucet(config.clone()).await;
        test_send_transaction(config, faucet).await.unwrap();
    }

    #[cfg(feature = "test_pimlico_api")]
    fn pimlico_faucet() -> LocalSigner<SigningKey> {
        use alloy::signers::local::{coins_bip39::English, MnemonicBuilder};
        MnemonicBuilder::<English>::default()
            .phrase(
                std::env::var("FAUCET_MNEMONIC")
                    .expect("You've not set the FAUCET_MNEMONIC"),
            )
            .build()
            .unwrap()
    }

    #[tokio::test]
    #[cfg(feature = "test_pimlico_api")]
    async fn test_send_transaction_pimlico() {
        let config = Config::pimlico();
        let faucet = pimlico_faucet();
        test_send_transaction(config, faucet).await.unwrap();
    }

    #[tokio::test]
    async fn test_send_transaction_just_deploy() -> eyre::Result<()> {
        let config = Config::local();
        let faucet = anvil_faucet(config.clone()).await;

        let provider = ReqwestProvider::<Ethereum>::new_http(
            config.endpoints.rpc.base_url.parse()?,
        );

        let owner = LocalSigner::random();
        let sender_address = get_account_address(
            provider.clone(),
            Owners { owners: vec![owner.address()], threshold: 1 },
        )
        .await;
        assert!(provider.get_code_at(sender_address).await.unwrap().is_empty());

        use_faucet(
            provider.clone(),
            faucet.clone(),
            U256::from(3),
            sender_address,
        )
        .await?;

        let transaction = vec![];

        let transaction_hash = send_transaction(
            transaction,
            owner.clone(),
            None,
            None,
            config.clone(),
        )
        .await?;

        println!("Transaction sent: {}", transaction_hash);

        assert!(!provider
            .get_code_at(sender_address)
            .await
            .unwrap()
            .is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_send_transaction_batch() -> eyre::Result<()> {
        let config = Config::local();
        let faucet = anvil_faucet(config.clone()).await;

        let provider = ReqwestProvider::<Ethereum>::new_http(
            config.endpoints.rpc.base_url.parse()?,
        );

        let destination1 = LocalSigner::random();
        let destination2 = LocalSigner::random();

        let owner = LocalSigner::random();
        let sender_address = get_account_address(
            provider.clone(),
            Owners { owners: vec![owner.address()], threshold: 1 },
        )
        .await;

        use_faucet(
            provider.clone(),
            faucet.clone(),
            U256::from(3),
            sender_address,
        )
        .await?;

        let transaction = vec![
            Execution {
                target: destination1.address(),
                value: Uint::from(1),
                callData: Bytes::new(),
            },
            Execution {
                target: destination2.address(),
                value: Uint::from(2),
                callData: Bytes::new(),
            },
        ];

        let transaction_hash = send_transaction(
            transaction,
            owner.clone(),
            None,
            None,
            config.clone(),
        )
        .await?;

        println!("Transaction sent: {}", transaction_hash);

        assert_eq!(
            provider.get_balance(destination1.address()).await?,
            Uint::from(1)
        );
        assert_eq!(
            provider.get_balance(destination2.address()).await?,
            Uint::from(2)
        );

        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_send_transaction_7702() -> eyre::Result<()> {
        let config = Config::local();
        let provider = ReqwestProvider::<Ethereum>::new_http(
            config.endpoints.rpc.base_url.parse()?,
        );

        let destination = LocalSigner::random();
        let balance = provider.get_balance(destination.address()).await?;
        assert_eq!(balance, Uint::from(0));

        let owner = LocalSigner::random();
        let contract_address = get_account_address(
            provider.clone(),
            Owners { owners: vec![owner.address()], threshold: 1 },
        )
        .await;
        // TODO remove when testing 7702; the contract address doesn't need
        // funds, just the authority
        // provider.anvil_set_balance(contract_address, U256::MAX).await?;

        let authority = LocalSigner::random();
        provider.anvil_set_balance(authority.address(), U256::MAX).await?;

        let chain_id = ChainId::ETHEREUM_SEPOLIA.eip155_chain_id();
        let auth_7702 = alloy::rpc::types::Authorization {
            chain_id: U256::from(chain_id),
            address: contract_address,
            nonce: provider.get_transaction_count(authority.address()).await?,
        };

        // Sign the authorization
        let sig = authority.sign_hash_sync(&auth_7702.signature_hash())?;
        let auth = auth_7702.into_signed(sig);

        let authorization_list = vec![Authorization {
            contract_address: auth.address,
            chain_id: u64::from_be_bytes(
                U64::from(auth.chain_id).to_be_bytes(),
            ),
            nonce: auth.nonce,
            y_parity: auth.signature().v().y_parity_byte(),
            r: format!(
                "0x{}",
                hex::encode(auth.signature().r().to_be_bytes_vec())
            ),
            s: format!(
                "0x{}",
                hex::encode(auth.signature().s().to_be_bytes_vec())
            ),
        }];

        let transaction = vec![];
        let transaction_hash = send_transaction(
            transaction,
            owner.clone(),
            Some(authority.address()),
            Some(authorization_list.clone()),
            // None,
            config.clone(),
        )
        .await?;
        println!("Transaction sent: {}", transaction_hash);

        println!("contract address: {}", contract_address);
        println!(
            "contract code: {}",
            provider.get_code_at(contract_address).await?
        );
        println!(
            "authority code: {}",
            provider.get_code_at(authority.address()).await?
        );

        let transaction = vec![Execution {
            target: destination.address(),
            value: Uint::from(1),
            callData: Bytes::new(),
        }];

        let transaction_hash = send_transaction(
            transaction,
            owner,
            Some(authority.address()),
            // None,
            // Some(authorization_list.clone()),
            None,
            config,
        )
        .await?;

        println!("Transaction sent: {}", transaction_hash);

        let balance = provider.get_balance(destination.address()).await?;
        assert_eq!(balance, Uint::from(1));

        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_send_transaction_7702_vanilla_bundler() -> eyre::Result<()> {
        let config = Config::local();
        let provider = ReqwestProvider::<Ethereum>::new_http(
            config.endpoints.rpc.base_url.parse()?,
        );

        let destination = LocalSigner::random();
        let balance = provider.get_balance(destination.address()).await?;
        assert_eq!(balance, Uint::from(0));

        let owner = LocalSigner::random();
        let contract_address = get_account_address(
            provider.clone(),
            Owners { owners: vec![owner.address()], threshold: 1 },
        )
        .await;

        let transaction = vec![];
        let transaction_hash = send_transaction(
            transaction,
            owner.clone(),
            None,
            None,
            config.clone(),
        )
        .await?;
        println!("Transaction sent: {}", transaction_hash);

        let authority = LocalSigner::random();
        provider.anvil_set_balance(authority.address(), U256::MAX).await?;

        let (dummy_contract_address, dummy_contract_calldata) = {
            // Codegen from embedded Solidity code and precompiled bytecode.
            // solc v0.8.25 Log.sol --via-ir --optimize --bin
            sol!(
                #[allow(missing_docs)]
                #[sol(rpc, bytecode = "6080806040523460135760c9908160188239f35b5f80fdfe6004361015600b575f80fd5b5f3560e01c80637b3ab2d014605f57639ee1a440146027575f80fd5b34605b575f366003190112605b577f2d67bb91f17bca05af6764ab411e86f4ddf757adb89fcec59a7d21c525d417125f80a1005b5f80fd5b34605b575f366003190112605b577fbcdfe0d5b27dd186282e187525415c57ea3077c34efb39148111e4d342e7ab0e5f80a100fea2646970667358221220f6b42b522bc9fb2b4c7d7e611c7c3e995d057ecab7fd7be4179712804c886b4f64736f6c63430008190033")]
                contract Log {
                    #[derive(Debug)]
                    event Hello();
                    event World();

                    function emitHello() public {
                        emit Hello();
                    }

                    function emitWorld() public {
                        emit World();
                    }
                }
            );
            let contract = Log::deploy(&provider).await?;
            let call = contract.emitHello();
            (*contract.address(), call.calldata().to_owned())
        };

        let chain_id = ChainId::ETHEREUM_SEPOLIA.eip155_chain_id();
        let auth_7702 = alloy::rpc::types::Authorization {
            chain_id: U256::from(chain_id),
            address: contract_address,
            nonce: provider.get_transaction_count(authority.address()).await?,
        };

        // Sign the authorization
        let sig = authority.sign_hash_sync(&auth_7702.signature_hash())?;
        let auth = auth_7702.into_signed(sig);

        // Estimate the EIP1559 fees
        let eip1559_est = provider.estimate_eip1559_fees(None).await?;

        // Build the transaction
        // let sender = authority.clone(); // The one sending the txn can be
        // different form the EOA being delgated
        let sender = LocalSigner::random();
        provider.anvil_set_balance(sender.address(), U256::MAX).await?;
        let mut tx = TxEip7702 {
            to: dummy_contract_address,
            authorization_list: vec![auth],
            input: dummy_contract_calldata,
            nonce: provider.get_transaction_count(sender.address()).await?,
            chain_id,
            gas_limit: 1000000,
            max_fee_per_gas: eip1559_est.max_fee_per_gas,
            max_priority_fee_per_gas: eip1559_est.max_priority_fee_per_gas,
            ..Default::default()
        };

        // Sign and Encode the transaction
        let sig = sender.sign_transaction_sync(&mut tx)?;
        let tx = tx.into_signed(sig);
        let mut encoded = Vec::new();
        tx.tx().encode_with_signature(tx.signature(), &mut encoded, false);
        let receipt = provider
            .send_raw_transaction(&encoded)
            .await?
            .get_receipt()
            .await?;

        assert!(receipt.status());
        assert_eq!(receipt.inner.logs().len(), 1);
        assert_eq!(receipt.inner.logs()[0].address(), dummy_contract_address);

        provider
            .watch_pending_transaction(PendingTransactionConfig::new(
                receipt.transaction_hash,
            ))
            .await?
            .await?;

        println!("contract address: {}", contract_address);
        println!(
            "contract code: {}",
            provider.get_code_at(contract_address).await?
        );
        println!(
            "authority code: {}",
            provider.get_code_at(authority.address()).await?
        );

        let transaction = vec![Execution {
            target: destination.address(),
            value: Uint::from(1),
            callData: Bytes::new(),
        }];

        let transaction_hash = send_transaction(
            transaction,
            owner,
            Some(authority.address()),
            None,
            config,
        )
        .await?;

        println!("Transaction sent: {}", transaction_hash);

        let balance = provider.get_balance(destination.address()).await?;
        assert_eq!(balance, Uint::from(2));

        Ok(())
    }

    // TODO test/fix: if invalid call data (e.g. sending balance that you don't
    // have), the account will still be deployed but the transfer won't happen.
    // Why can't we detect this?
}
