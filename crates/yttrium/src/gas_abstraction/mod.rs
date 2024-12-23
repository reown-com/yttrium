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
        erc7579::{
            accounts::safe::encode_validator_key,
            addresses::RHINESTONE_ATTESTER_ADDRESS,
            ownable_validator::{
                get_ownable_validator, get_ownable_validator_mock_signature,
            },
        },
        provider_pool::ProviderPool,
        smart_accounts::{
            nonce::get_nonce_with_key,
            safe::{
                get_call_data, AddSafe7579Contract, Owners, SetupContract,
                SAFE_4337_MODULE_ADDRESS, SAFE_ERC_7579_LAUNCHPAD_ADDRESS,
                SAFE_L2_SINGLETON_1_4_1,
            },
        },
        test_helpers::anvil_faucet,
        user_operation::{hash::get_user_operation_hash_v07, UserOperationV07},
    },
    alloy::{
        network::{EthereumWallet, TransactionBuilder7702},
        primitives::{
            eip191_hash_message, Address, Bytes, PrimitiveSignature, B256, U256,
        },
        rpc::types::Authorization,
        signers::local::LocalSigner,
        sol_types::SolCall,
    },
    alloy_provider::{Provider, ProviderBuilder},
    relay_rpc::domain::ProjectId,
    std::collections::HashMap,
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
    // TODO remove `config` param, do similar with_*_overrides pattern
    // Actually, add these to ProviderPool as other types of providers (e.g. get_bundler())
    pub fn new(
        project_id: ProjectId,
        chain_id: String,
        config: Config,
    ) -> Self {
        Self {
            provider_pool: ProviderPool::new(project_id).with_rpc_overrides(
                HashMap::from([(
                    chain_id,
                    config.endpoints.rpc.base_url.parse().unwrap(),
                )]),
            ),
            config,
        }
    }

    // TODO error type
    pub async fn prepare(
        &self,
        transaction: InitialTransaction,
    ) -> Result<PreparedGasAbstraction, String> {
        let provider =
            self.provider_pool.get_provider(&transaction.chain_id).await;

        let code = provider.get_code_at(transaction.from).await.unwrap();
        // TODO check if the code is our contract, or something else
        // If no code, return 7702 txn and UserOp hash to sign
        // If our contract, return UserOp hash to sign
        // If something else then return an error

        if code.is_empty() {
            // Else assume it's our account for now
            // TODO throw error if it's not our account (how?)

            let auth = Authorization {
                chain_id: transaction
                    .chain_id
                    .strip_prefix("eip155:")
                    .unwrap()
                    .parse()
                    .unwrap(),
                address: SAFE_L2_SINGLETON_1_4_1,
                // TODO should this be `pending` tag? https://github.com/wevm/viem/blob/a49c100a0b2878fbfd9f1c9b43c5cc25de241754/src/experimental/eip7702/actions/signAuthorization.ts#L149
                nonce: provider
                    .get_transaction_count(transaction.from)
                    .await
                    .unwrap(),
            };

            let auth = PreparedGasAbstractionAuthorization {
                hash: auth.signature_hash(),
                auth,
            };

            Ok(PreparedGasAbstraction::DeploymentRequired {
                auth,
                prepare_deploy_params: PrepareDeployParams { transaction },
            })
        } else {
            let prepared_send =
                self.create_sponsored_user_op(transaction).await;

            Ok(PreparedGasAbstraction::DeploymentNotRequired { prepared_send })
        }
    }

    // TODO error type
    async fn create_sponsored_user_op(
        &self,
        transaction: InitialTransaction,
    ) -> PreparedSend {
        let InitialTransaction { chain_id, from, to, value, input } =
            transaction;

        // TODO don't look this up a second time
        let provider = self.provider_pool.get_provider(&chain_id).await;

        let owners = Owners { threshold: 1, owners: vec![from] };
        let ownable_validator = get_ownable_validator(&owners, None);
        let nonce = get_nonce_with_key(
            &provider,
            from.into(),
            &ENTRYPOINT_ADDRESS_V07.into(),
            encode_validator_key(ownable_validator.address),
        )
        .await
        .unwrap();

        let mock_signature = get_ownable_validator_mock_signature(&owners);

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
            signature: mock_signature,
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

        let message: B256 = get_user_operation_hash_v07(
            &user_op,
            &ENTRYPOINT_ADDRESS_V07,
            chain_id.strip_prefix("eip155:").unwrap().parse().unwrap(),
        )
        .into();

        let hash = eip191_hash_message(message);

        PreparedSend {
            message: message.into(),
            hash,
            send_params: SendParams { user_op },
        }
    }

    // TODO error type
    // When bundlers support sending 7702 natively, this name will be accurate (noting is sent until `send()`)
    // For now, `prepare_deploy()` also will execute a 7702 txn by itself since this function sponsors the UserOp
    pub async fn prepare_deploy(
        &self,
        // FIXME can't pass Authorization through FFI
        auth_sig: SignedAuthorization,
        params: PrepareDeployParams,
    ) -> PreparedSend {
        let account = params.transaction.from;
        let SignedAuthorization { auth, signature } = auth_sig;
        let chain_id = auth.chain_id;
        let auth = auth.into_signed(signature);

        let faucet = anvil_faucet(&self.config.endpoints.rpc.base_url).await;
        let faucet_wallet = EthereumWallet::new(faucet);
        let faucet_provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(faucet_wallet)
            .on_provider(
                self.provider_pool
                    .get_provider(&format!("eip155:{chain_id}"))
                    .await,
            );

        let safe_owners = Owners {
            threshold: 1,
            owners: vec![LocalSigner::random().address()],
        };

        let owners = Owners { threshold: 1, owners: vec![account] };

        let ownable_validator = get_ownable_validator(&owners, None);

        // TODO do this in the UserOp as a factory
        assert!(SetupContract::new(account, faucet_provider)
            .setup(
                safe_owners.owners,
                U256::from(safe_owners.threshold),
                SAFE_ERC_7579_LAUNCHPAD_ADDRESS,
                AddSafe7579Contract::addSafe7579Call {
                    safe7579: SAFE_4337_MODULE_ADDRESS,
                    validators: vec![
                        AddSafe7579Contract::ModuleInit {
                            module: ownable_validator.address,
                            initData: ownable_validator.init_data,
                        },
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

        self.create_sponsored_user_op(params.transaction).await
    }

    // TODO error type
    // TODO check if receipt is actually OK, return error result if not (with the receipt still)
    pub async fn send(
        &self,
        signature: PrimitiveSignature,
        params: SendParams,
    ) -> UserOperationReceipt {
        let bundler_client = BundlerClient::new(BundlerConfig::new(
            self.config.endpoints.bundler.base_url.clone(),
        ));
        let signature = signature.as_bytes().into();
        let user_op = UserOperationV07 { signature, ..params.user_op };
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
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum PreparedGasAbstraction {
    DeploymentRequired {
        auth: PreparedGasAbstractionAuthorization,
        prepare_deploy_params: PrepareDeployParams,
    },
    DeploymentNotRequired {
        prepared_send: PreparedSend,
    },
}

#[derive(Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PrepareDeployParams {
    pub transaction: InitialTransaction,
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

#[derive(Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PreparedSend {
    pub message: Bytes,
    pub hash: B256,
    pub send_params: SendParams,
}

#[derive(Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct SendParams {
    pub user_op: UserOperationV07,
}
