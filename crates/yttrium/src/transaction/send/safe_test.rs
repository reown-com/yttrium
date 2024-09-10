use crate::user_operation::UserOperationV07;
use core::fmt;

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

#[cfg(test)]
mod tests {
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
        user_operation::UserOperationV07,
    };
    use alloy::{
        dyn_abi::{DynSolValue, Eip712Domain},
        network::Ethereum,
        primitives::{
            aliases::U48, Address, Bytes, FixedBytes, Uint, U128, U256,
        },
        providers::{ext::AnvilApi, Provider, ReqwestProvider},
        signers::{k256::ecdsa::SigningKey, local::LocalSigner, SignerSync},
        sol,
        sol_types::{SolCall, SolValue},
    };
    use std::{ops::Not, str::FromStr};

    async fn send_transaction(
        execution_calldata: Vec<Execution>,
        owner: LocalSigner<SigningKey>,
    ) -> eyre::Result<String> {
        let config = crate::config::Config::local();

        let bundler_base_url = config.endpoints.bundler.base_url;
        let paymaster_base_url = config.endpoints.paymaster.base_url;

        let bundler_client =
            BundlerClient::new(BundlerConfig::new(bundler_base_url.clone()));

        let pimlico_client: PimlicoBundlerClient = PimlicoBundlerClient::new(
            BundlerConfig::new(bundler_base_url.clone()),
        );

        let chain = crate::chain::Chain::ETHEREUM_SEPOLIA_V07;
        let entry_point_config = chain.entry_point_config();

        let chain_id = chain.id.eip155_chain_id()?;

        let entry_point_address = entry_point_config.address();

        let rpc_url = config.endpoints.rpc.base_url;

        let rpc_url: reqwest::Url = rpc_url.parse()?;
        let provider = ReqwestProvider::<Ethereum>::new_http(rpc_url);

        let safe_factory_address_primitives: Address =
            SAFE_PROXY_FACTORY_ADDRESS;
        let safe_factory_address =
            FactoryAddress::new(safe_factory_address_primitives);

        let owners = Owners { owners: vec![owner.address()], threshold: 1 };

        let factory_data_value = factory_data(owners.clone()).abi_encode();

        let sender_address =
            get_account_address(provider.clone(), owners.clone()).await;

        let call_type = if execution_calldata.len() > 1 {
            CallType::BatchCall
        } else {
            CallType::Call
        };
        let revert_on_error = false;
        let selector = [0u8; 4];
        let context = [0u8; 22];

        enum CallType {
            Call,
            BatchCall,
            #[allow(dead_code)]
            DelegateCall,
        }
        impl CallType {
            fn as_byte(&self) -> u8 {
                match self {
                    CallType::Call => 0x00,
                    CallType::BatchCall => 0x01,
                    CallType::DelegateCall => 0xff,
                }
            }
        }

        let mode = DynSolValue::Tuple(vec![
            DynSolValue::Uint(Uint::from(call_type.as_byte()), 8),
            DynSolValue::Uint(Uint::from(revert_on_error as u8), 8),
            DynSolValue::Bytes(vec![0u8; 4]),
            DynSolValue::Bytes(selector.to_vec()),
            DynSolValue::Bytes(context.to_vec()),
        ])
        .abi_encode_packed();

        let call_data = Safe7579::executeCall {
            mode: FixedBytes::from_slice(&mode),
            executionCalldata: execution_calldata.abi_encode_packed().into(),
        }
        .abi_encode()
        .into();

        let deployed = provider.get_code_at(sender_address).await?.len() > 0;
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

        let gas_price =
            pimlico_client.estimate_user_operation_gas_price().await?;

        assert!(gas_price.fast.max_fee_per_gas > U256::from(1));

        let nonce = get_nonce(
            &provider,
            &SimpleAccountAddress::new(sender_address),
            &entry_point_address,
        )
        .await?;

        let user_op = UserOperationV07 {
            sender: sender_address,
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
            signature: Bytes::from_str(
                crate::smart_accounts::safe::DUMMY_SIGNATURE_HEX
                    .strip_prefix("0x")
                    .unwrap(),
            )?,
        };

        let paymaster_client = PaymasterClient::new(BundlerConfig::new(
            paymaster_base_url.clone(),
        ));

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
            op.paymaster_post_op_gas_limit =
                Some(s.paymaster_post_op_gas_limit);
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
            safe: sender_address,
            callData: sponsored_user_op.call_data.clone(),
            nonce: sponsored_user_op.nonce,
            initCode: deployed
                .not()
                .then(|| {
                    [
                        sponsored_user_op
                            .clone()
                            .factory
                            .unwrap()
                            .to_vec()
                            .into(),
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
                coerce_u256_to_u128(sponsored_user_op.call_gas_limit)
                    .to_be_bytes(),
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
        let signed_user_op =
            UserOperationV07 { signature, ..sponsored_user_op };

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
            "UserOperation included: https://sepolia.etherscan.io/tx/{}",
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

    #[tokio::test]
    async fn test_send_transaction() -> eyre::Result<()> {
        let rpc_url = Config::local().endpoints.rpc.base_url;
        let rpc_url: reqwest::Url = rpc_url.parse()?;
        let provider = ReqwestProvider::<Ethereum>::new_http(rpc_url);

        let destination = LocalSigner::random();
        let balance = provider.get_balance(destination.address()).await?;
        assert_eq!(balance, Uint::from(0));

        let owner = LocalSigner::random();
        let sender_address = get_account_address(
            provider.clone(),
            Owners { owners: vec![owner.address()], threshold: 1 },
        )
        .await;

        provider.anvil_set_balance(sender_address, U256::from(100)).await?;
        let transaction = vec![Execution {
            target: destination.address(),
            value: Uint::from(1),
            callData: Bytes::new(),
        }];

        let transaction_hash =
            send_transaction(transaction, owner.clone()).await?;

        println!("Transaction sent: {}", transaction_hash);

        let balance = provider.get_balance(destination.address()).await?;
        assert_eq!(balance, Uint::from(1));

        provider.anvil_set_balance(sender_address, U256::from(100)).await?;
        let transaction = vec![Execution {
            target: destination.address(),
            value: Uint::from(1),
            callData: Bytes::new(),
        }];

        let transaction_hash = send_transaction(transaction, owner).await?;

        println!("Transaction sent: {}", transaction_hash);

        let balance = provider.get_balance(destination.address()).await?;
        assert_eq!(balance, Uint::from(2));

        Ok(())
    }

    // TODO test/fix: if invalid call data (e.g. sending balance that you don't
    // have), the account will still be deployed but the transfer won't happen.
    // Why can't we detect this?
}
