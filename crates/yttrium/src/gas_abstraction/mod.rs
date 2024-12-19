use {
    crate::{
        bundler::{
            client::BundlerClient,
            config::BundlerConfig,
            models::user_operation_receipt::UserOperationReceipt,
            pimlico::{self, paymaster::client::PaymasterClient},
        },
        chain_abstraction::api::InitialTransaction,
        config::Config,
        entry_point::ENTRYPOINT_ADDRESS_V07,
        erc7579::addresses::{
            MOCK_ATTESTER_ADDRESS, RHINESTONE_ATTESTER_ADDRESS,
        },
        execution::send::safe_test::{
            encode_send_transactions, DoSendTransactionParams, OwnerSignature,
            PreparedSendTransaction,
        },
        provider_pool::ProviderPool,
        smart_accounts::{
            nonce::get_nonce,
            safe::{
                get_call_data, user_operation_to_safe_op, AddSafe7579Contract,
                Owners, SetupContract, SAFE_4337_MODULE_ADDRESS,
                SAFE_ERC_7579_LAUNCHPAD_ADDRESS, SAFE_L2_SINGLETON_1_4_1,
            },
        },
        test_helpers::anvil_faucet,
        user_operation::UserOperationV07,
    },
    alloy::{
        network::{EthereumWallet, TransactionBuilder7702},
        primitives::{
            aliases::U48, Address, Bytes, PrimitiveSignature, B256, U256, U64,
        },
        rpc::types::Authorization,
        sol_types::{SolCall, SolStruct},
    },
    alloy_provider::{Provider, ProviderBuilder},
    relay_rpc::domain::ProjectId,
};

#[cfg(test)]
mod tests;

// docs: https://github.com/reown-com/reown-docs/pull/201

#[derive(Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct Client {
    provider_pool: ProviderPool,
    config: Config,
}

#[cfg_attr(feature = "uniffi", uniffi::export(async_runtime = "tokio"))]
impl Client {
    #[cfg_attr(feature = "uniffi", uniffi::constructor)]
    // TODO remove `config` param
    pub fn new(project_id: ProjectId, config: Config) -> Self {
        Self {
            provider_pool: ProviderPool::new(project_id, config.clone()),
            config,
        }
    }

    // Or you can make chain-specific clients if you want to do it manually
    // async fn new(chain_id: Caip10, rpc_url: Url, bundler_url: Url, paymaster_url: Url) -> Self {}

    async fn prepare(
        &self,
        transaction: InitialTransaction,
    ) -> Result<PreparedGasAbstraction, String> {
        let InitialTransaction { chain_id, from, to, value, input } =
            transaction;

        let provider = self.provider_pool.get_provider(&chain_id).await;

        let code = provider.get_code_at(from).await.unwrap();
        // TODO check if the code is our contract, or something else
        // If no code, return 7702 txn and UserOp hash to sign
        // If our contract, return UserOp hash to sign
        // If something else then return an error

        let auth = if code.is_empty() {
            let auth = Authorization {
                chain_id: chain_id
                    .strip_prefix("eip155:")
                    .unwrap()
                    .parse()
                    .unwrap(),
                address: SAFE_L2_SINGLETON_1_4_1,
                // TODO should this be `pending` tag? https://github.com/wevm/viem/blob/a49c100a0b2878fbfd9f1c9b43c5cc25de241754/src/experimental/eip7702/actions/signAuthorization.ts#L149
                nonce: provider.get_transaction_count(from).await.unwrap(),
            };

            Some(PreparedGasAbstractionAuthorization {
                hash: auth.signature_hash(),
                auth,
            })
        } else {
            None
        };

        // Else assume it's our account for now

        // TODO with_key if has modules
        let nonce =
            get_nonce(&provider, from.into(), &ENTRYPOINT_ADDRESS_V07.into())
                .await
                .unwrap();

        // let permission_id = get_permission_id(&session);
        // let smart_session_dummy_signature = encode_use_signature(
        //     permission_id,
        //     get_ownable_validator_mock_signature(1),
        // );

        // TODO refactor to reuse these clients as part of the pool thingie
        let pimlico_client = pimlico::client::BundlerClient::new(
            BundlerConfig::new(self.config.endpoints.bundler.base_url.clone()),
        );
        let paymaster_client = PaymasterClient::new(BundlerConfig::new(
            self.config.endpoints.paymaster.base_url.clone(),
        ));

        let gas_price = pimlico_client
            .estimate_user_operation_gas_price()
            .await
            .unwrap()
            .fast;
        let user_op = UserOperationV07 {
            sender: from.into(),
            nonce,
            factory: None,
            factory_data: None,
            call_data: get_call_data(vec![crate::execution::Execution {
                to,
                value,
                data: input,
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
            signature: Bytes::new(),
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
                pre_verification_gas: sponsor_user_op_result
                    .pre_verification_gas,
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
        let (safe_op, domain) = user_operation_to_safe_op(
            &user_op,
            ENTRYPOINT_ADDRESS_V07,
            U64::from(
                chain_id
                    .strip_prefix("eip155:")
                    .unwrap()
                    .parse::<u64>()
                    .unwrap(),
            ),
            valid_after,
            valid_until,
        );
        let hash = safe_op.eip712_signing_hash(&domain);

        Ok(PreparedGasAbstraction {
            auth,
            prepared_send_transaction: PreparedSendTransaction {
                safe_op,
                domain,
                hash,
                do_send_transaction_params: DoSendTransactionParams {
                    user_op,
                    valid_after,
                    valid_until,
                },
            },
        })
    }

    // TODO error type
    // TODO check if receipt is actually OK, return error result if not (with the receipt still)
    async fn send(
        &self,
        // FIXME can't pass Authorization through FFI
        auth_sig: Option<SignedAuthorization>,
        signature: PrimitiveSignature,
        params: DoSendTransactionParams,
    ) -> UserOperationReceipt {
        let account = params.user_op.sender;
        // TODO put this all inside the UserOperation once it's available

        if let Some(SignedAuthorization { auth, signature }) = auth_sig {
            let chain_id = auth.chain_id;
            let auth = auth.into_signed(signature);

            let faucet =
                anvil_faucet(&self.config.endpoints.rpc.base_url).await;
            let wallet = EthereumWallet::new(faucet);
            let wallet_provider = ProviderBuilder::new()
                .with_recommended_fillers()
                .wallet(wallet)
                .on_provider(
                    self.provider_pool
                        .get_provider(&format!("eip155:{chain_id}"))
                        .await,
                );
            let owners = Owners { threshold: 1, owners: vec![account.into()] };
            assert!(SetupContract::new(
                account.into(),
                wallet_provider.clone()
            )
            .setup(
                owners.owners,
                U256::from(owners.threshold),
                SAFE_ERC_7579_LAUNCHPAD_ADDRESS,
                AddSafe7579Contract::addSafe7579Call {
                    safe7579: SAFE_4337_MODULE_ADDRESS,
                    validators: vec![
                        // AddSafe7579Contract::ModuleInit {
                        //     module: ownable_validator.address,
                        //     initData: ownable_validator.init_data,
                        // },
                        // AddSafe7579Contract::ModuleInit {
                        //     module: smart_sessions.address,
                        //     initData: smart_sessions.init_data,
                        // },
                    ],
                    executors: vec![],
                    fallbacks: vec![],
                    hooks: vec![],
                    attesters: vec![
                        RHINESTONE_ATTESTER_ADDRESS,
                        MOCK_ATTESTER_ADDRESS,
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
                t.set_authorization_list(vec![auth]);
                t
            })
            .send()
            .await
            .unwrap()
            .get_receipt()
            .await
            .unwrap()
            .status());
        }

        let bundler_client = BundlerClient::new(BundlerConfig::new(
            self.config.endpoints.bundler.base_url.clone(),
        ));
        let user_op = encode_send_transactions(
            vec![OwnerSignature {
                owner: params.user_op.sender.into(),
                signature,
            }],
            params,
        )
        .await
        .unwrap();
        let hash = bundler_client
            .send_user_operation(ENTRYPOINT_ADDRESS_V07.into(), user_op.clone())
            .await
            .unwrap();

        bundler_client.wait_for_user_operation_receipt(hash).await.unwrap()
    }

    // Signature creation
    // async fn ..... COPY signature creation functions from current SDK

    // == Future APIs ==
    // async fn send_user_operation(?) - ?;
    // async fn create_smart_session(?) -> ?;
}

#[derive(Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PreparedGasAbstraction {
    pub auth: Option<PreparedGasAbstractionAuthorization>,
    pub prepared_send_transaction: PreparedSendTransaction,
}

#[derive(Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PreparedGasAbstractionAuthorization {
    pub auth: Authorization,
    pub hash: B256,
}

#[derive(Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct SignedAuthorization {
    // FIXME cannot pass this through FFI like this
    pub auth: Authorization,
    pub signature: PrimitiveSignature,
}
