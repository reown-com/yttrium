use crate::{
    bundler::{
        client::BundlerClient,
        config::BundlerConfig,
        models::user_operation_receipt::UserOperationReceipt,
        pimlico::{
            client::BundlerClient as PimlicoBundlerClient,
            paymaster::client::PaymasterClient,
        },
    },
    chain::ChainId,
    config::Config,
    entry_point::EntryPointVersion,
    smart_accounts::{
        account_address::AccountAddress,
        nonce::get_nonce,
        safe::{
            factory_data, get_account_address, get_call_data,
            get_call_data_with_try, Owners, Safe7579Launchpad, SafeOp,
            DUMMY_SIGNATURE, SAFE_4337_MODULE_ADDRESS,
            SAFE_ERC_7579_LAUNCHPAD_ADDRESS, SAFE_PROXY_FACTORY_ADDRESS,
            SEPOLIA_SAFE_ERC_7579_SINGLETON_ADDRESS,
        },
    },
    transaction::Transaction,
    user_operation::{Authorization, UserOperationV07},
};
use alloy::{
    dyn_abi::{DynSolValue, Eip712Domain},
    network::Ethereum,
    primitives::{aliases::U48, Bytes, Uint, B256, U128, U160, U256},
    providers::{Provider, ReqwestProvider},
    signers::{k256::ecdsa::SigningKey, local::LocalSigner, SignerSync},
    sol_types::{SolCall, SolStruct},
    transports::Transport,
};
use alloy_provider::Network;
use core::fmt;
use serde::{Deserialize, Serialize};
use std::ops::Not;

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
    owner_address: String,
    config: Config,
) -> eyre::Result<AccountAddress> {
    let rpc_url = config.endpoints.rpc.base_url;
    let rpc_url: reqwest::Url = rpc_url.parse()?;
    let provider = ReqwestProvider::<Ethereum>::new_http(rpc_url);

    let owners =
        Owners { owners: vec![owner_address.parse().unwrap()], threshold: 1 };

    Ok(get_account_address(provider, owners).await)
}

pub async fn send_transactions(
    execution_calldata: Vec<Transaction>,
    owner: LocalSigner<SigningKey>,
    address: Option<AccountAddress>,
    authorization_list: Option<Vec<Authorization>>,
    config: Config,
) -> eyre::Result<UserOperationReceipt> {
    let owner_address = owner.address();
    let PreparedSendTransaction {
        safe_op,
        domain,
        hash,
        do_send_transaction_params,
    } = prepare_send_transactions(
        execution_calldata,
        owner_address,
        address,
        authorization_list,
        config.clone(),
    )
    .await?;

    // TODO loop per-owner
    let signature = owner.sign_typed_data_sync(&safe_op, &domain)?;
    let signature2 = owner.sign_hash_sync(&hash)?;
    assert_eq!(signature, signature2);

    let user_operation_hash = do_send_transactions(
        vec![OwnerSignature { owner: owner_address, signature }],
        do_send_transaction_params,
        config.clone(),
    )
    .await?;

    println!("Querying for receipts...");

    let bundler_client = BundlerClient::new(BundlerConfig::new(
        config.endpoints.bundler.base_url.clone(),
    ));
    let receipt = bundler_client
        .wait_for_user_operation_receipt(user_operation_hash)
        .await?;

    println!(
        "SAFE UserOperation included: https://sepolia.etherscan.io/tx/{}",
        receipt.receipt.transaction_hash
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

    Ok(receipt)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreparedSendTransaction {
    pub safe_op: SafeOp,
    pub domain: Eip712Domain,
    pub hash: B256,
    pub do_send_transaction_params: DoSendTransactionParams,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoSendTransactionParams {
    pub user_op: UserOperationV07,
    pub valid_after: U48,
    pub valid_until: U48,
}

pub async fn prepare_send_transactions(
    execution_calldata: Vec<Transaction>,
    owner: Address,
    address: Option<AccountAddress>,
    authorization_list: Option<Vec<Authorization>>,
    config: Config,
) -> eyre::Result<PreparedSendTransaction> {
    let pimlico_client = PimlicoBundlerClient::new(BundlerConfig::new(
        config.endpoints.bundler.base_url.clone(),
    ));

    let provider = ReqwestProvider::<Ethereum>::new_http(
        config.endpoints.rpc.base_url.parse()?,
    );

    let paymaster_client = PaymasterClient::new(BundlerConfig::new(
        config.endpoints.paymaster.base_url.clone(),
    ));

    let gas_price = pimlico_client.estimate_user_operation_gas_price().await?;
    assert!(gas_price.fast.max_fee_per_gas > U256::from(1));

    prepare_send_transactions_inner(
        execution_calldata,
        Owners { owners: vec![owner], threshold: 1 },
        address,
        authorization_list,
        &provider,
        gas_price.fast.max_fee_per_gas,
        gas_price.fast.max_priority_fee_per_gas,
        paymaster_client,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn prepare_send_transactions_inner<P, T, N>(
    execution_calldata: Vec<Transaction>,
    owners: Owners,
    address: Option<AccountAddress>,
    authorization_list: Option<Vec<Authorization>>,
    provider: &P,
    max_fee_per_gas: U256,
    max_priority_fee_per_gas: U256,
    paymaster_client: PaymasterClient,
) -> eyre::Result<PreparedSendTransaction>
where
    T: Transport + Clone,
    P: Provider<T, N>,
    N: Network,
{
    let chain_id = provider.get_chain_id().await?;
    let chain = crate::chain::Chain::new(
        ChainId::new_eip155(chain_id),
        EntryPointVersion::V07,
    );
    let chain_id = chain.id.eip155_chain_id();

    let entry_point_config = chain.entry_point_config();
    let entry_point_address = entry_point_config.address();

    let factory_data_value = factory_data(owners.clone()).abi_encode();

    let contract_address = get_account_address(provider, owners.clone()).await;
    let account_address =
        if let Some(address) = address { address } else { contract_address };

    let deployed =
        provider.get_code_at(account_address.into()).await?.len() > 0;
    println!("Deployed: {}", deployed);
    // permissionless: signerToSafeSmartAccount -> encodeCallData
    let call_data = if deployed
        && provider
            .get_storage_at(account_address.into(), Uint::from(0))
            .await?
            == U256::from(U160::from_be_bytes(
                SEPOLIA_SAFE_ERC_7579_SINGLETON_ADDRESS.into_array(),
            )) {
        get_call_data(execution_calldata)
    } else {
        // Note about using `try` mode for get_call_data & needing to check
        // storage above. This is due to an issue in the Safe7579Launchpad
        // contract where a revert will cause the Safe7579Launchpad::setupSafe
        // to be reverted too. This leaves the account in a bricked state. To
        // workaround, we use the `try` mode to ensure that the reverted
        // execution does not revert the setupSafe call too. This unfortunately
        // has the side-effect that the UserOp will be successful, which is
        // misleading.
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
                callData: get_call_data_with_try(execution_calldata, true),
                validators: vec![],
            },
        }
        .abi_encode()
        .into()
    };

    let nonce =
        get_nonce(&provider, account_address, &entry_point_address).await?;

    let user_op = UserOperationV07 {
        sender: account_address,
        nonce,
        factory: deployed.not().then_some(SAFE_PROXY_FACTORY_ADDRESS),
        factory_data: deployed.not().then(|| factory_data_value.into()),
        call_data,
        call_gas_limit: U256::ZERO,
        verification_gas_limit: U256::ZERO,
        pre_verification_gas: U256::ZERO,
        max_fee_per_gas,
        max_priority_fee_per_gas,
        paymaster: None,
        paymaster_verification_gas_limit: None,
        paymaster_post_op_gas_limit: None,
        paymaster_data: None,
        // authorization_list: None,
        signature: DUMMY_SIGNATURE,
    };

    if let Some(_authorization_list) = authorization_list {
        // let response = reqwest::Client::new()
        //         .post(config.endpoints.paymaster.base_url.clone())
        //         .json(&json!({
        //                 "jsonrpc": "2.0",
        //                 "id": 1,
        //                 "method": "eth_prepareSendUserOperation7702",
        //                 "params": [
        //                     format!("{}:{}:{}", user_op.sender,
        // user_op.nonce, user_op.call_data),
        // authorization_list,                 ],
        //         }))
        //         .send()
        //         .await
        //         .unwrap();
        // let success = response.status().is_success();
        // println!("response: {:?}", response.text().await);

        // assert!(success);
        unimplemented!("need to refactor provider config to re-enable this")
    }

    let user_op = {
        let sponsor_user_op_result = paymaster_client
            .sponsor_user_operation_v07(
                &user_op.clone().into(),
                &entry_point_address,
                None,
            )
            .await?;

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

    let valid_after = U48::from(0);
    let valid_until = U48::from(0);

    // TODO handle panic
    fn coerce_u256_to_u128(u: U256) -> U128 {
        U128::from(u)
    }

    let safe_op = SafeOp {
        safe: account_address.into(),
        callData: user_op.call_data.clone(),
        nonce: user_op.nonce,
        initCode: deployed
            .not()
            .then(|| {
                [
                    user_op.clone().factory.unwrap().to_vec().into(),
                    user_op.clone().factory_data.unwrap(),
                ]
                .concat()
                .into()
            })
            .unwrap_or(Bytes::new()),
        maxFeePerGas: u128::from_be_bytes(
            coerce_u256_to_u128(user_op.max_fee_per_gas).to_be_bytes(),
        ),
        maxPriorityFeePerGas: u128::from_be_bytes(
            coerce_u256_to_u128(user_op.max_priority_fee_per_gas).to_be_bytes(),
        ),
        preVerificationGas: user_op.pre_verification_gas,
        verificationGasLimit: u128::from_be_bytes(
            coerce_u256_to_u128(user_op.verification_gas_limit).to_be_bytes(),
        ),
        callGasLimit: u128::from_be_bytes(
            coerce_u256_to_u128(user_op.call_gas_limit).to_be_bytes(),
        ),
        // signerToSafeSmartAccount -> getPaymasterAndData
        paymasterAndData: user_op
            .paymaster
            .map(|paymaster| {
                [
                    paymaster.to_vec(),
                    coerce_u256_to_u128(
                        user_op
                            .paymaster_verification_gas_limit
                            .unwrap_or(Uint::from(0)),
                    )
                    .to_be_bytes_vec(),
                    coerce_u256_to_u128(
                        user_op
                            .paymaster_post_op_gas_limit
                            .unwrap_or(Uint::from(0)),
                    )
                    .to_be_bytes_vec(),
                    user_op.paymaster_data.clone().unwrap_or_default().to_vec(),
                ]
                .concat()
                .into()
            })
            .unwrap_or_default(),
        validAfter: valid_after,
        validUntil: valid_until,
        entryPoint: entry_point_address.to_address(),
    };

    let erc7579_launchpad_address = true;
    let verifying_contract = if erc7579_launchpad_address && !deployed {
        user_op.sender.into()
    } else {
        SAFE_4337_MODULE_ADDRESS
    };

    let domain = Eip712Domain {
        chain_id: Some(Uint::from(chain_id)),
        verifying_contract: Some(verifying_contract),
        ..Default::default()
    };
    let hash = safe_op.eip712_signing_hash(&domain);

    Ok(PreparedSendTransaction {
        safe_op,
        domain,
        hash,
        do_send_transaction_params: DoSendTransactionParams {
            user_op,
            valid_after,
            valid_until,
        },
    })
}

pub use alloy::primitives::Address;
pub use alloy::primitives::PrimitiveSignature;
pub struct OwnerSignature {
    pub owner: Address,
    pub signature: PrimitiveSignature,
}

pub async fn encode_send_transactions(
    signatures: Vec<OwnerSignature>,
    DoSendTransactionParams {
        user_op,
        valid_after,
        valid_until,
    }: DoSendTransactionParams,
) -> eyre::Result<UserOperationV07> {
    if signatures.len() != 1 {
        return Err(eyre::eyre!("Only one signature is supported for now"));
    }

    let mut signatures = signatures;
    signatures.sort_by(|a, b| a.owner.cmp(&b.owner));
    let signature_bytes = signatures
        .into_iter()
        .map(|s| s.signature.as_bytes())
        .collect::<Vec<_>>()
        .concat();

    let signature = DynSolValue::Tuple(vec![
        DynSolValue::Uint(Uint::from(valid_after), 48),
        DynSolValue::Uint(Uint::from(valid_until), 48),
        DynSolValue::Bytes(signature_bytes),
    ])
    .abi_encode_packed()
    .into();

    Ok(UserOperationV07 { signature, ..user_op })
}

pub async fn do_send_transactions(
    signatures: Vec<OwnerSignature>,
    params: DoSendTransactionParams,
    config: Config,
) -> eyre::Result<B256> {
    let user_op = encode_send_transactions(signatures, params).await?;

    let provider = ReqwestProvider::<Ethereum>::new_http(
        config.endpoints.rpc.base_url.parse()?,
    );
    let chain_id = provider.get_chain_id().await?;
    let chain = crate::chain::Chain::new(
        ChainId::new_eip155(chain_id),
        EntryPointVersion::V07,
    );
    let entry_point_config = chain.entry_point_config();
    let entry_point_address = entry_point_config.address();
    let bundler_client = BundlerClient::new(BundlerConfig::new(
        config.endpoints.bundler.base_url.clone(),
    ));
    let user_operation_hash = bundler_client
        .send_user_operation(entry_point_address, user_op.clone())
        .await?;

    println!("Received User Operation hash: {:?}", user_operation_hash);

    Ok(user_operation_hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::smart_accounts::safe::sign_step_3;
    use crate::smart_accounts::safe::SignOutputEnum;
    use crate::{
        chain::ChainId,
        smart_accounts::safe::{prepare_sign, sign, PreparedSignature},
        test_helpers::{self, use_faucet},
        transaction::Transaction,
    };
    use alloy::network::{TransactionBuilder, TransactionBuilder7702};
    use alloy::rpc::types::TransactionRequest;
    use alloy::{
        primitives::{eip191_hash_message, fixed_bytes, U160, U64},
        providers::{ext::AnvilApi, PendingTransactionConfig},
        sol,
    };

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
            sender_address.into(),
        )
        .await;

        let transaction = vec![Transaction {
            to: destination.address(),
            value: Uint::from(1),
            data: Bytes::new(),
        }];

        let receipt = send_transactions(
            transaction,
            owner.clone(),
            None,
            None,
            config.clone(),
        )
        .await?;
        assert!(receipt.success);

        let balance = provider.get_balance(destination.address()).await?;
        assert_eq!(balance, Uint::from(1));

        let transaction = vec![Transaction {
            to: destination.address(),
            value: Uint::from(1),
            data: Bytes::new(),
        }];

        let receipt =
            send_transactions(transaction, owner, None, None, config).await?;
        assert!(receipt.success);

        let balance = provider.get_balance(destination.address()).await?;
        assert_eq!(balance, Uint::from(2));

        Ok(())
    }

    async fn anvil_faucet(config: Config) -> LocalSigner<SigningKey> {
        test_helpers::anvil_faucet(config.endpoints.rpc.base_url).await
    }

    #[tokio::test]
    async fn test_send_transaction_local() {
        let config = Config::local();
        let faucet = anvil_faucet(config.clone()).await;
        test_send_transaction(config, faucet).await.unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "test_pimlico_api")]
    async fn test_send_transaction_pimlico() {
        use crate::test_helpers::private_faucet;
        let config = Config::pimlico();
        let faucet = private_faucet();
        test_send_transaction(config, faucet).await.unwrap();
    }

    #[tokio::test]
    async fn test_send_transaction_first_reverted_local() {
        let config = Config::local();
        let faucet = anvil_faucet(config.clone()).await;
        test_send_transaction_first_reverted(config, faucet).await;
    }

    #[tokio::test]
    #[ignore = "not useful currently, can do same test locally"]
    #[cfg(feature = "test_pimlico_api")]
    async fn test_send_transaction_first_reverted_pimlico() {
        use crate::test_helpers::private_faucet;

        let config = Config::pimlico();
        let faucet = private_faucet();
        test_send_transaction_first_reverted(config, faucet).await;
    }

    async fn test_send_transaction_first_reverted(
        config: Config,
        faucet: LocalSigner<SigningKey>,
    ) {
        let provider = ReqwestProvider::<Ethereum>::new_http(
            config.endpoints.rpc.base_url.parse().unwrap(),
        );

        let destination = LocalSigner::random();
        let balance =
            provider.get_balance(destination.address()).await.unwrap();
        assert_eq!(balance, Uint::from(0));

        let owner = LocalSigner::random();
        let sender_address = get_account_address(
            provider.clone(),
            Owners { owners: vec![owner.address()], threshold: 1 },
        )
        .await;

        let transaction = vec![Transaction {
            to: destination.address(),
            value: Uint::from(1),
            data: Bytes::new(),
        }];

        let receipt = send_transactions(
            transaction,
            owner.clone(),
            None,
            None,
            config.clone(),
        )
        .await
        .unwrap();
        // The UserOp is successful, but the transaction actually failed. See
        // note above near `Safe7579Launchpad::setupSafe`
        assert!(receipt.success);
        assert!(
            provider.get_code_at(sender_address.into()).await.unwrap().len()
                > 0
        );
        assert_eq!(
            provider
                .get_storage_at(sender_address.into(), Uint::from(0))
                .await
                .unwrap(),
            U256::from(U160::from_be_bytes(
                SEPOLIA_SAFE_ERC_7579_SINGLETON_ADDRESS.into_array()
            ))
        );

        let balance =
            provider.get_balance(destination.address()).await.unwrap();
        assert_eq!(balance, Uint::from(0));
        use_faucet(
            provider.clone(),
            faucet.clone(),
            U256::from(1),
            sender_address.into(),
        )
        .await;

        let transaction = vec![Transaction {
            to: destination.address(),
            value: Uint::from(1),
            data: Bytes::new(),
        }];

        let receipt = send_transactions(transaction, owner, None, None, config)
            .await
            .unwrap();
        assert!(receipt.success);
        assert_eq!(
            provider
                .get_storage_at(sender_address.into(), Uint::from(0))
                .await
                .unwrap(),
            U256::from(U160::from_be_bytes(
                SEPOLIA_SAFE_ERC_7579_SINGLETON_ADDRESS.into_array()
            ))
        );

        let balance =
            provider.get_balance(destination.address()).await.unwrap();
        assert_eq!(balance, Uint::from(1));
    }

    #[tokio::test]
    #[ignore]
    async fn test_send_transaction_reverted() {
        let config = Config::local();
        let provider = ReqwestProvider::<Ethereum>::new_http(
            config.endpoints.rpc.base_url.parse().unwrap(),
        );

        let destination = LocalSigner::random();
        let balance =
            provider.get_balance(destination.address()).await.unwrap();
        assert_eq!(balance, Uint::from(0));

        let owner = LocalSigner::random();
        let sender_address = get_account_address(
            provider.clone(),
            Owners { owners: vec![owner.address()], threshold: 1 },
        )
        .await;

        let transaction = vec![Transaction {
            to: destination.address(),
            value: Uint::from(1),
            data: Bytes::new(),
        }];

        let receipt = send_transactions(
            transaction,
            LocalSigner::random(),
            Some(sender_address),
            None,
            config.clone(),
        )
        .await
        .unwrap();
        // The UserOp is successful, but the transaction actually failed. See
        // note above near `Safe7579Launchpad::setupSafe`
        assert!(!receipt.success);
        // assert!(
        //     provider.get_code_at(sender_address.into()).await.unwrap().len()
        //         > 0
        // );
        // assert_eq!(
        //     provider
        //         .get_storage_at(sender_address.into(), Uint::from(0))
        //         .await
        //         .unwrap(),
        //     U256::from(U160::from_be_bytes(
        //         SEPOLIA_SAFE_ERC_7579_SINGLETON_ADDRESS.into_array()
        //     ))
        // );

        // let balance =
        //     provider.get_balance(destination.address()).await.unwrap();
        // assert_eq!(balance, Uint::from(0));

        // let transaction = vec![Transaction {
        //     to: destination.address(),
        //     value: Uint::from(1),
        //     data: Bytes::new(),
        // }];

        // let receipt = send_transaction(transaction, owner, None, None,
        // config)     .await
        //     .unwrap();
        // assert!(receipt.success);
        // assert_eq!(
        //     provider
        //         .get_storage_at(sender_address.into(), Uint::from(0))
        //         .await
        //         .unwrap(),
        //     U256::from(U160::from_be_bytes(
        //         SEPOLIA_SAFE_ERC_7579_SINGLETON_ADDRESS.into_array()
        //     ))
        // );

        // let balance =
        //     provider.get_balance(destination.address()).await.unwrap();
        // assert_eq!(balance, Uint::from(1));
    }

    #[tokio::test]
    async fn test_send_transaction_just_deploy() -> eyre::Result<()> {
        let config = Config::local();

        let provider = ReqwestProvider::<Ethereum>::new_http(
            config.endpoints.rpc.base_url.parse()?,
        );

        let owner = LocalSigner::random();
        let sender_address = get_account_address(
            provider.clone(),
            Owners { owners: vec![owner.address()], threshold: 1 },
        )
        .await;
        assert!(provider
            .get_code_at(sender_address.into())
            .await
            .unwrap()
            .is_empty());

        let transaction = vec![];

        let receipt = send_transactions(
            transaction,
            owner.clone(),
            None,
            None,
            config.clone(),
        )
        .await?;
        assert!(receipt.success);

        assert!(!provider
            .get_code_at(sender_address.into())
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
            sender_address.into(),
        )
        .await;

        let transaction = vec![
            Transaction {
                to: destination1.address(),
                value: Uint::from(1),
                data: Bytes::new(),
            },
            Transaction {
                to: destination2.address(),
                value: Uint::from(2),
                data: Bytes::new(),
            },
        ];

        let receipt = send_transactions(
            transaction,
            owner.clone(),
            None,
            None,
            config.clone(),
        )
        .await?;
        assert!(receipt.success);

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
    async fn test_sign_message_deployed() {
        let config = Config::local();
        let provider = ReqwestProvider::<Ethereum>::new_http(
            config.endpoints.rpc.base_url.parse().unwrap(),
        );

        let owner = LocalSigner::random();
        let owner_address = owner.address();
        let owners = Owners { owners: vec![owner.address()], threshold: 1 };

        let sender_address =
            get_account_address(provider.clone(), owners.clone()).await;

        let receipt = send_transactions(
            vec![],
            owner.clone(),
            None,
            None,
            config.clone(),
        )
        .await
        .unwrap();
        assert!(receipt.success);
        assert!(!provider
            .get_code_at(sender_address.into())
            .await
            .unwrap()
            .is_empty());

        let message = "test message";
        let message_hash = eip191_hash_message(message);

        let chain_id = provider.get_chain_id().await.unwrap();
        let PreparedSignature { safe_message, domain } =
            prepare_sign(sender_address, U256::from(chain_id), message_hash);

        let signature =
            owner.sign_typed_data_sync(&safe_message, &domain).unwrap();

        let signature = sign(
            owners,
            sender_address,
            vec![OwnerSignature { owner: owner_address, signature }],
            &provider,
            PaymasterClient::new(BundlerConfig::new(
                config.endpoints.paymaster.base_url.parse().unwrap(),
            )),
        )
        .await
        .unwrap();

        let signature = match signature {
            SignOutputEnum::Signature(s) => s,
            SignOutputEnum::SignOutput(so) => {
                let signature = owner
                    .sign_typed_data_sync(
                        &so.to_sign.safe_op,
                        &so.to_sign.domain,
                    )
                    .unwrap();
                sign_step_3(
                    vec![OwnerSignature { owner: owner.address(), signature }],
                    so.sign_step_3_params,
                )
                .await
                .unwrap()
            }
        };

        sol! {
            #[sol(rpc)]
            contract Eip1271 {
                function isValidSignature(bytes32 hash, bytes calldata signature) external view returns (bytes4 magicValue);
            }
        };

        let magic_value = Eip1271::new(sender_address.into(), provider.clone())
            .isValidSignature(message_hash, signature.clone())
            .call()
            .await
            .unwrap();
        assert_eq!(magic_value.magicValue, fixed_bytes!("1626ba7e"));

        assert!(erc6492::verify_signature(
            signature,
            sender_address.into(),
            message_hash,
            &provider
        )
        .await
        .unwrap()
        .is_valid());
    }

    #[tokio::test]
    async fn test_sign_message_not_deployed() {
        let config = Config::local();
        let provider = ReqwestProvider::<Ethereum>::new_http(
            config.endpoints.rpc.base_url.parse().unwrap(),
        );

        let owner = LocalSigner::random();
        let owner_address = owner.address();
        let owners = Owners { owners: vec![owner.address()], threshold: 1 };

        let sender_address =
            get_account_address(provider.clone(), owners.clone()).await;

        assert!(provider
            .get_code_at(sender_address.into())
            .await
            .unwrap()
            .is_empty());

        let message = "test message";
        let message_hash = eip191_hash_message(message);

        let chain_id = provider.get_chain_id().await.unwrap();
        let PreparedSignature { safe_message, domain } =
            prepare_sign(sender_address, U256::from(chain_id), message_hash);

        let signature =
            owner.sign_typed_data_sync(&safe_message, &domain).unwrap();

        let signature = sign(
            owners,
            sender_address,
            vec![OwnerSignature { owner: owner_address, signature }],
            &provider,
            PaymasterClient::new(BundlerConfig::new(
                config.endpoints.paymaster.base_url.parse().unwrap(),
            )),
        )
        .await
        .unwrap();

        let signature = match signature {
            SignOutputEnum::Signature(s) => s,
            SignOutputEnum::SignOutput(so) => {
                let signature = owner
                    .sign_typed_data_sync(
                        &so.to_sign.safe_op,
                        &so.to_sign.domain,
                    )
                    .unwrap();
                sign_step_3(
                    vec![OwnerSignature { owner: owner.address(), signature }],
                    so.sign_step_3_params,
                )
                .await
                .unwrap()
            }
        };

        assert!(provider
            .get_code_at(sender_address.into())
            .await
            .unwrap()
            .is_empty());

        assert!(erc6492::verify_signature(
            signature,
            sender_address.into(),
            message_hash,
            &provider
        )
        .await
        .unwrap()
        .is_valid());

        assert!(provider
            .get_code_at(sender_address.into())
            .await
            .unwrap()
            .is_empty());
    }

    #[tokio::test]
    async fn test_sign_message_partial_deployed() {
        let config = Config::local();
        let provider = ReqwestProvider::<Ethereum>::new_http(
            config.endpoints.rpc.base_url.parse().unwrap(),
        );

        let owner = LocalSigner::random();
        let owner_address = owner.address();
        let owners = Owners { owners: vec![owner.address()], threshold: 1 };

        let sender_address =
            get_account_address(provider.clone(), owners.clone()).await;

        let destination = LocalSigner::random();
        let balance =
            provider.get_balance(destination.address()).await.unwrap();
        assert_eq!(balance, Uint::from(0));
        let receipt = send_transactions(
            vec![Transaction {
                to: destination.address(),
                value: Uint::from(1),
                data: Bytes::new(),
            }],
            owner.clone(),
            None,
            None,
            config.clone(),
        )
        .await
        .unwrap();
        assert!(receipt.success);
        assert!(!provider
            .get_code_at(sender_address.into())
            .await
            .unwrap()
            .is_empty());
        assert_eq!(
            provider
                .get_storage_at(sender_address.into(), Uint::from(0))
                .await
                .unwrap(),
            U256::from(U160::from_be_bytes(
                SEPOLIA_SAFE_ERC_7579_SINGLETON_ADDRESS.into_array()
            ))
        );

        let message = "test message";
        let message_hash = eip191_hash_message(message);

        let chain_id = provider.get_chain_id().await.unwrap();
        let PreparedSignature { safe_message, domain } =
            prepare_sign(sender_address, U256::from(chain_id), message_hash);

        let signature =
            owner.sign_typed_data_sync(&safe_message, &domain).unwrap();

        let signature = sign(
            owners,
            sender_address,
            vec![OwnerSignature { owner: owner_address, signature }],
            &provider,
            PaymasterClient::new(BundlerConfig::new(
                config.endpoints.paymaster.base_url.parse().unwrap(),
            )),
        )
        .await
        .unwrap();

        let signature = match signature {
            SignOutputEnum::Signature(s) => s,
            SignOutputEnum::SignOutput(so) => {
                let signature = owner
                    .sign_typed_data_sync(
                        &so.to_sign.safe_op,
                        &so.to_sign.domain,
                    )
                    .unwrap();
                sign_step_3(
                    vec![OwnerSignature { owner: owner.address(), signature }],
                    so.sign_step_3_params,
                )
                .await
                .unwrap()
            }
        };

        sol! {
            #[sol(rpc)]
            contract Eip1271 {
                function isValidSignature(bytes32 hash, bytes calldata signature) external view returns (bytes4 magicValue);
            }
        };

        let magic_value = Eip1271::new(sender_address.into(), provider.clone())
            .isValidSignature(message_hash, signature.clone())
            .call()
            .await
            .unwrap();
        assert_eq!(magic_value.magicValue, fixed_bytes!("1626ba7e"));

        assert!(erc6492::verify_signature(
            signature,
            sender_address.into(),
            message_hash,
            &provider
        )
        .await
        .unwrap()
        .is_valid());

        let balance =
            provider.get_balance(destination.address()).await.unwrap();
        assert_eq!(balance, Uint::from(0));
        let faucet = anvil_faucet(config.clone()).await;
        use_faucet(
            provider.clone(),
            faucet.clone(),
            U256::from(1),
            sender_address.into(),
        )
        .await;

        let transaction = vec![Transaction {
            to: destination.address(),
            value: Uint::from(1),
            data: Bytes::new(),
        }];

        let receipt = send_transactions(transaction, owner, None, None, config)
            .await
            .unwrap();
        assert!(receipt.success);
        assert_eq!(
            provider
                .get_storage_at(sender_address.into(), Uint::from(0))
                .await
                .unwrap(),
            U256::from(U160::from_be_bytes(
                SEPOLIA_SAFE_ERC_7579_SINGLETON_ADDRESS.into_array()
            ))
        );

        let balance =
            provider.get_balance(destination.address()).await.unwrap();
        assert_eq!(balance, Uint::from(1));
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
            chain_id,
            address: contract_address.into(),
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
            y_parity: auth.y_parity(),
            r: auth.r(),
            s: auth.s(),
        }];

        let transaction = vec![];
        let receipt = send_transactions(
            transaction,
            owner.clone(),
            Some(authority.address().into()),
            Some(authorization_list.clone()),
            // None,
            config.clone(),
        )
        .await?;
        assert!(receipt.success);

        println!("contract address: {}", contract_address);
        println!(
            "contract code: {}",
            provider.get_code_at(contract_address.into()).await?
        );
        println!(
            "authority code: {}",
            provider.get_code_at(authority.address()).await?
        );

        let transaction = vec![Transaction {
            to: destination.address(),
            value: Uint::from(1),
            data: Bytes::new(),
        }];

        let receipt = send_transactions(
            transaction,
            owner,
            Some(authority.address().into()),
            // None,
            // Some(authorization_list.clone()),
            None,
            config,
        )
        .await?;
        assert!(receipt.success);

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
        let receipt = send_transactions(
            transaction,
            owner.clone(),
            None,
            None,
            config.clone(),
        )
        .await?;
        assert!(receipt.success);

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
            chain_id,
            address: contract_address.into(),
            nonce: provider.get_transaction_count(authority.address()).await?,
        };

        // Sign the authorization
        let sig = authority.sign_hash_sync(&auth_7702.signature_hash())?;
        let auth = auth_7702.into_signed(sig);

        // Estimate the EIP1559 fees
        // let eip1559_est = provider.estimate_eip1559_fees(None).await?;

        // Build the transaction
        // let sender = authority.clone(); // The one sending the txn can be
        // different form the EOA being delgated
        let sender = LocalSigner::random();
        provider.anvil_set_balance(sender.address(), U256::MAX).await?;

        let tx = TransactionRequest::default()
            .with_to(dummy_contract_address)
            .with_authorization_list(vec![auth])
            .with_input(dummy_contract_calldata);
        // let mut tx = TxEip7702 {
        //     to: dummy_contract_address,
        //     authorization_list: vec![auth],
        //     input: dummy_contract_calldata,
        //     nonce: provider.get_transaction_count(sender.address()).await?,
        //     chain_id,
        //     gas_limit: 1000000,
        //     max_fee_per_gas: eip1559_est.max_fee_per_gas,
        //     max_priority_fee_per_gas: eip1559_est.max_priority_fee_per_gas,
        //     ..Default::default()
        // };

        // Sign and Encode the transaction
        // let sig = sender.sign_transaction_sync(&mut tx)?;
        // let tx = tx.into_signed(sig);
        // let mut encoded = Vec::new();
        // tx.tx().encode_with_signature(tx.signature(), &mut encoded, false);
        // let receipt = provider
        //     .send_raw_transaction(&encoded)
        //     .await?
        //     .get_receipt()
        //     .await?;

        let receipt =
            provider.send_transaction(tx).await?.get_receipt().await?;

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
            provider.get_code_at(contract_address.into()).await?
        );
        println!(
            "authority code: {}",
            provider.get_code_at(authority.address()).await?
        );

        let transaction: Vec<_> = vec![Transaction {
            to: destination.address(),
            value: Uint::from(1),
            data: Bytes::new(),
        }];

        let receipt = send_transactions(
            transaction,
            owner,
            Some(authority.address().into()),
            None,
            config,
        )
        .await?;
        assert!(receipt.success);

        let balance = provider.get_balance(destination.address()).await?;
        assert_eq!(balance, Uint::from(2));

        Ok(())
    }

    // TODO test/fix: if invalid call data (e.g. sending balance that you don't
    // have), the account will still be deployed but the transfer won't happen.
    // Why can't we detect this?
}
