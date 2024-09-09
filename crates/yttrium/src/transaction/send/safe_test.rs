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
        entry_point::get_sender_address::get_sender_address_v07,
        smart_accounts::{
            nonce::get_nonce,
            safe::{
                factory_data, Execution, Owners, Safe7579, Safe7579Launchpad,
                SAFE_4337_MODULE_ADDRESS, SAFE_ERC_7579_LAUNCHPAD_ADDRESS,
                SAFE_PROXY_FACTORY_ADDRESS,
                SEPOLIA_SAFE_ERC_7579_SINGLETON_ADDRESS,
            },
            simple_account::{factory::FactoryAddress, SimpleAccountAddress},
        },
        transaction::Transaction,
        user_operation::UserOperationV07,
    };
    use alloy::{
        dyn_abi::{DynSolValue, Eip712Domain},
        network::EthereumWallet,
        primitives::{Address, Bytes, FixedBytes, Uint, U128, U256},
        providers::ProviderBuilder,
        signers::{local::LocalSigner, SignerSync},
        sol,
        sol_types::{SolCall, SolValue},
    };
    use std::{ops::Not, str::FromStr, time::Duration};

    async fn send_transaction(
        transaction: Transaction,
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

        // Create a provider

        let alloy_signer = LocalSigner::random();
        let ethereum_wallet = EthereumWallet::new(alloy_signer.clone());

        let rpc_url: reqwest::Url = rpc_url.parse()?;
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(ethereum_wallet.clone())
            .on_http(rpc_url);

        let safe_factory_address_primitives: Address =
            SAFE_PROXY_FACTORY_ADDRESS;
        let safe_factory_address =
            FactoryAddress::new(safe_factory_address_primitives);

        let owner = ethereum_wallet.clone().default_signer();
        let owner_address = owner.address();
        let owners = Owners { owners: vec![owner_address], threshold: 1 };

        let factory_data_value = factory_data(owners.clone()).abi_encode();

        let factory_data_value_hex = hex::encode(factory_data_value.clone());

        let factory_data_value_hex_prefixed =
            format!("0x{}", factory_data_value_hex);

        println!(
            "Generated factory_data: {:?}",
            factory_data_value_hex_prefixed.clone()
        );

        // 5. Calculate the sender address

        let sender_address = get_sender_address_v07(
            &provider,
            safe_factory_address.into(),
            factory_data_value.clone().into(),
            entry_point_address,
        )
        .await?;

        println!("Calculated sender address: {:?}", sender_address);

        let to: Address = transaction.to.parse()?;
        let value: alloy::primitives::Uint<256, 4> =
            transaction.value.parse()?;
        let data_hex = transaction.data.strip_prefix("0x").unwrap();
        let data: Bytes = Bytes::from_str(data_hex)?;

        // let execution_calldata =
        //     vec![Execution { target: to, value, callData: data }];
        // let execution_calldata =
        //     Execution { target: to, value, callData: data };
        let execution_calldata =
            [to.to_vec(), value.to_be_bytes_vec(), data.to_vec()]
                .concat()
                .into();

        // let call_type = if execution_calldata.len() > 1 {
        //     CallType::BatchCall
        // } else {
        //     CallType::Call
        // };
        let call_type = CallType::Call;
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

        let mut mode = Vec::with_capacity(32);
        mode.push(call_type.as_byte());
        mode.push(if revert_on_error { 0x01 } else { 0x00 });
        mode.extend_from_slice(&[0u8; 4]);
        mode.extend_from_slice(&selector);
        mode.extend_from_slice(&context);
        let mode = FixedBytes::from_slice(&mode);

        // let mode = DynSolValue::Tuple(vec![
        //     DynSolValue::Uint(Uint::from(call_type.as_byte()), 8),
        //     DynSolValue::Uint(Uint::from(revert_on_error as u8), 8),
        //     DynSolValue::Bytes(selector.to_vec().into()),
        //     DynSolValue::Bytes(context.to_vec().into()),
        // ]).abi_encode_packed().into();

        let call_data = Safe7579::executeCall {
            mode,
            // executionCalldata: execution_calldata.abi_encode().into(),
            executionCalldata: execution_calldata,
        }
        .abi_encode()
        .into();

        let deployed = false;
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

        // Fill out remaining UserOperation values

        let gas_price =
            pimlico_client.estimate_user_operation_gas_price().await?;

        assert!(gas_price.fast.max_fee_per_gas > U256::from(1));

        println!("Gas price: {:?}", gas_price);

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

        println!("sponsor_user_op_result: {:?}", sponsor_user_op_result);

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

        println!("Received paymaster sponsor result: {:?}", sponsored_user_op);

        // Sign the UserOperation

        let valid_after = 0;
        let valid_until = 0;

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
            SAFE_ERC_7579_LAUNCHPAD_ADDRESS
        };

        // TODO loop per-owner
        let signature = alloy_signer.sign_typed_data_sync(
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

        Ok(user_operation_hash)
    }

    #[tokio::test]
    async fn test_send_transaction() -> eyre::Result<()> {
        let transaction = Transaction::mock();

        let transaction_hash = send_transaction(transaction).await?;

        println!("Transaction sent: {}", transaction_hash);

        Ok(())
    }
}
