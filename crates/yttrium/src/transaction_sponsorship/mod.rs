use {
    crate::{
        blockchain_api::{BLOCKCHAIN_API_URL, BUNDLER_ENDPOINT_PATH},
        bundler::{
            client::BundlerClient,
            config::BundlerConfig,
            pimlico::{self, paymaster::client::PaymasterClient},
        },
        call::Call,
        chain_abstraction::amount::Amount,
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
        hex::FromHex, network::{EthereumWallet, TransactionBuilder7702}, primitives::{eip191_hash_message, Address, Bytes, PrimitiveSignature, B256, U256}, rpc::types::{Authorization, UserOperationReceipt}, signers::local::{LocalSigner, PrivateKeySigner}, sol_types::SolCall, sol
    },
    alloy_provider::{Provider, ProviderBuilder},
    error::{
        CreateSponsoredUserOpError, PrepareDeployError, PrepareError, SendError,
    },
    relay_rpc::domain::ProjectId,
    reqwest::Url,
    std::{collections::HashMap, time::Duration},
};

pub mod error;

sol! {
    pragma solidity ^0.8.0;
    function transfer(address recipient, uint256 amount) external returns (bool);
}


#[cfg(test)]
mod tests;

// docs: https://github.com/reown-com/reown-docs/pull/201

#[derive(Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct Client {
    provider_pool: ProviderPool,
    bundler_url: Url,
    paymaster_url: Url,
}

#[cfg_attr(feature = "uniffi", uniffi::export(async_runtime = "tokio"))]
impl Client {
    #[cfg_attr(feature = "uniffi", uniffi::constructor)]
    pub fn new(project_id: ProjectId) -> Self {
        let bundler_url = BLOCKCHAIN_API_URL
            .parse::<Url>()
            .unwrap()
            .join(BUNDLER_ENDPOINT_PATH)
            .unwrap();

        Self {
            provider_pool: ProviderPool::new(project_id),
            paymaster_url: bundler_url.clone(),
            bundler_url,
        }
    }

    // #[cfg(feature = "uniffi")]
    pub fn with_rpc_overrides(
        &self,
        rpc_overrides: HashMap<String, Url>,
    ) -> Self {
        let mut s = self.clone();
        s.provider_pool.set_rpc_overrides(rpc_overrides);
        s
    }

    // #[cfg(feature = "uniffi")]
    pub fn with_4337_urls(&self, bundler_url: Url, paymaster_url: Url) -> Self {
        let mut s = self.clone();
        s.bundler_url = bundler_url;
        s.paymaster_url = paymaster_url;
        s
    }

    pub fn prepare_usdc_transfer_call(
        &self,
        chain_id: &str,
        to: Address,
        usdc_amount: U256,
    ) -> Call {

        let usdc_address = match chain_id {
            "eip155:10" => Address::from_hex("0x0b2c639c533813f4aa9d7837caf62653d097ff85")
                .expect("invalid USDC address for Optimism"),
            "eip155:42161" => Address::from_hex("0xaf88d065e77c8cC2239327C5EDb3A432268e5831")
                .expect("invalid USDC address for Arbitrum"),
            "eip155:8453" => Address::from_hex("0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913")
                .expect("invalid USDC address for Base"),
            _ => {
                panic!("Unsupported chain_id: {chain_id}");
            }
        };


        let encoded_data = transfer::TransferCall::new(to, usdc_amount).encode();


        Call {
            to: usdc_address,
            value: U256::ZERO, 
            // input: encoded_data.into(),
            input: Bytes::new(),
        }
    }

    // The above builder-pattern implementations are inefficient when used by regular Rust code.
    // The below functions are standard Rust efficient, however UniFFI doesn't like them even though they are feature flagged.
    // Not sure what the solution is, but for now we will use the sub-optimal implementations from Rust.
    // Consider making a separate Client that wraps this in uniffi_compat?

    // #[cfg(not(feature = "uniffi"))]
    // pub fn set_rpc_overrides(&mut self, rpc_overrides: HashMap<String, Url>) {
    //     self.provider_pool.set_rpc_overrides(rpc_overrides);
    // }

    // #[cfg(not(feature = "uniffi"))]
    // pub fn set_4337_urls(
    //     &mut self,
    //     bundler_url: Url,
    //     paymaster_url: Url,
    // ) {
    //     self.bundler_url = bundler_url;
    //     self.paymaster_url = paymaster_url;
    // }

    // #[cfg(not(feature = "uniffi"))]
    // pub fn with_rpc_overrides(
    //     mut self,
    //     rpc_overrides: HashMap<String, Url>,
    // ) -> Self {
    //     self.set_rpc_overrides(rpc_overrides);
    //     self
    // }

    // #[cfg(not(feature = "uniffi"))]
    // pub fn with_4337_urls(
    //     mut self,
    //     bundler_url: Url,
    //     paymaster_url: Url,
    // ) -> Self {
    //     self.set_4337_urls(bundler_url, paymaster_url);
    //     self
    // }

    // TODO error type
    pub async fn prepare(
        &self,
        chain_id: String,
        from: Address,
        calls: Vec<Call>,
    ) -> Result<PreparedGasAbstraction, PrepareError> {
        let provider = self.provider_pool.get_provider(&chain_id).await;

        let code = provider
            .get_code_at(from)
            .await
            .map_err(PrepareError::CheckingAccountCode)?;
        // TODO check if the code is our contract, or something else
        // If no code, return 7702 txn and UserOp hash to sign
        // If our contract, return UserOp hash to sign
        // If something else then return an error

        if code.is_empty() {
            // Else assume it's our account for now
            // TODO throw error if it's not our account (how?)

            let auth = Authorization {
                chain_id: chain_id
                    .strip_prefix("eip155:")
                    .unwrap()
                    .parse()
                    .unwrap(),
                address: SAFE_L2_SINGLETON_1_4_1,
                // TODO should this be `pending` tag? https://github.com/wevm/viem/blob/a49c100a0b2878fbfd9f1c9b43c5cc25de241754/src/experimental/eip7702/actions/signAuthorization.ts#L149
                nonce: provider
                    .get_transaction_count(from)
                    .await
                    .map_err(PrepareError::GettingNonce)?,
            };

            let auth = PreparedGasAbstractionAuthorization {
                hash: auth.signature_hash(),
                auth,
            };

            Ok(PreparedGasAbstraction::DeploymentRequired {
                auth,
                prepare_deploy_params: PrepareDeployParams {
                    chain_id,
                    from,
                    calls,
                },
            })
        } else {
            let prepared_send = self
                .create_sponsored_user_op(chain_id, from, calls)
                .await
                .map_err(PrepareError::CreatingSponsoredUserOp)?;

            Ok(PreparedGasAbstraction::DeploymentNotRequired { prepared_send })
        }
    }

    // TODO error type
    async fn create_sponsored_user_op(
        &self,
        chain_id: String,
        from: Address,
        calls: Vec<Call>,
    ) -> Result<PreparedSend, CreateSponsoredUserOpError> {
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
        .map_err(CreateSponsoredUserOpError::GettingNonce)?;

        let mock_signature = get_ownable_validator_mock_signature(&owners);

        // TODO refactor to reuse these clients as part of the pool thingie
        let pimlico_client = pimlico::client::BundlerClient::new(
            BundlerConfig::new(self.bundler_url.clone()),
        );
        let paymaster_client = PaymasterClient::new(BundlerConfig::new(
            self.paymaster_url.clone(),
        ));

        let gas_price = pimlico_client
            .estimate_user_operation_gas_price()
            .await
            .map_err(CreateSponsoredUserOpError::GettingUserOperationGasPrice)?
            .fast;
        let user_op = UserOperationV07 {
            sender: from.into(),
            nonce,
            factory: None,
            factory_data: None,
            call_data: get_call_data(calls),
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
                .map_err(CreateSponsoredUserOpError::SponsoringUserOperation)?;

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

        let fees = {
            let total_gas = user_op.call_gas_limit
                + user_op.verification_gas_limit
                + user_op.pre_verification_gas
                + user_op.paymaster_verification_gas_limit.unwrap_or_default()
                + user_op.paymaster_post_op_gas_limit.unwrap_or_default();
            let gas_fee = total_gas * gas_price.max_fee_per_gas;

            // TODO calculate local_total and local_total_sponsored based on Zerion cost API
            PreparedSendFees {
                gas_fee,
                local_total: Amount::zero(),
                local_total_sponsored: Amount::zero(),
            }
        };

        Ok(PreparedSend {
            message: message.into(),
            hash,
            send_params: SendParams { user_op },
            fees,
        })
    }

    // TODO error type
    // When bundlers support sending 7702 natively, this name will be accurate (nothing is sent until `send()`)
    // For now, `prepare_deploy()` also will execute a 7702 txn by itself since this function sponsors the UserOp
    pub async fn prepare_deploy(
        &self,
        auth_sig: SignedAuthorization, // TODO replace this with the alloy type with the same name; and deal with the UniFFI conversion separately
        params: PrepareDeployParams,
        // TODO remove this `sponsor` param once 4337 supports sponsoring 7702 txns
        // Pass None to use anvil faucet
        sponsor: Option<PrivateKeySigner>,
    ) -> Result<PreparedSend, PrepareDeployError> {
        let account = params.from;
        let SignedAuthorization { auth, signature } = auth_sig;
        let chain_id = auth.chain_id;
        let auth = auth.into_signed(signature);

        let provider = self
            .provider_pool
            .get_provider(&format!("eip155:{chain_id}"))
            .await;

        let sponsor = if let Some(sponsor) = sponsor {
            sponsor
        } else {
            anvil_faucet(&provider).await
        };
        let sponsor_wallet = EthereumWallet::new(sponsor);
        let sponsor_provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(sponsor_wallet)
            .on_provider(provider);

        let safe_owners = Owners {
            threshold: 1,
            owners: vec![LocalSigner::random().address()],
        };

        let owners = Owners { threshold: 1, owners: vec![account] };

        let ownable_validator = get_ownable_validator(&owners, None);

        // TODO do this in the UserOp as a factory
        // https://linear.app/reown/issue/WK-474/blocked-7702-refactor-to-run-init-transaction-as-the-factory-of-the
        let receipt = SetupContract::new(account, sponsor_provider)
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
            .map_err(PrepareDeployError::SendingDelegationTransaction)?
            .with_timeout(Some(Duration::from_secs(30)))
            .get_receipt()
            .await
            .map_err(PrepareDeployError::GettingDelegationTransactionReceipt)?;

        if !receipt.status() {
            return Err(PrepareDeployError::DelegationTransactionFailed(
                receipt,
            ));
        }

        self.create_sponsored_user_op(
            params.chain_id,
            params.from,
            params.calls,
        )
        .await
        .map_err(PrepareDeployError::CreatingSponsoredUserOp)
    }

    // TODO error type
    // TODO check if receipt is actually OK, return error result if not (with the receipt still)
    pub async fn send(
        &self,
        signature: PrimitiveSignature,
        params: SendParams,
    ) -> Result<UserOperationReceipt, SendError> {
        let bundler_client =
            BundlerClient::new(BundlerConfig::new(self.bundler_url.clone()));
        let signature = signature.as_bytes().into();
        let user_op = UserOperationV07 { signature, ..params.user_op };
        let hash = bundler_client
            .send_user_operation(ENTRYPOINT_ADDRESS_V07.into(), user_op.clone())
            .await
            .map_err(SendError::SendingUserOperation)?;

        bundler_client
            .wait_for_user_operation_receipt(hash)
            .await
            .map_err(SendError::WaitingForUserOperationReceipt)
    }

    // Signature creation
    // async fn ..... COPY signature creation functions from current SDK

    // == Future APIs ==
    // async fn send_user_operation(?) - ?;
    // async fn create_smart_session(?) -> ?;
}

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
    pub chain_id: String,
    pub from: Address,
    pub calls: Vec<Call>,
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

#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PreparedSend {
    pub message: Bytes,
    pub hash: B256,
    pub send_params: SendParams,
    pub fees: PreparedSendFees,
}

#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PreparedSendFees {
    pub gas_fee: U256,
    pub local_total: Amount,
    pub local_total_sponsored: Amount,
}

#[derive(Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct SendParams {
    pub user_op: UserOperationV07,
}
