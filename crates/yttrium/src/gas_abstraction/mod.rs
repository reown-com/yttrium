use {
    crate::{
        bundler::{
            client::BundlerClient,
            config::BundlerConfig,
            pimlico::{self, paymaster::client::PaymasterClient},
        },
        config::Config,
        entry_point::ENTRYPOINT_ADDRESS_V07,
        execution::send::safe_test::{
            encode_send_transactions, DoSendTransactionParams, OwnerSignature,
            PreparedSendTransaction,
        },
        provider_pool::ProviderPool,
        smart_accounts::{
            nonce::get_nonce,
            safe::{get_call_data, user_operation_to_safe_op},
        },
        user_operation::UserOperationV07,
    },
    alloy::{
        primitives::{aliases::U48, Address, Bytes, U256, U64},
        rpc::types::TransactionRequest,
        sol_types::SolStruct,
    },
    alloy_provider::Provider,
    relay_rpc::domain::ProjectId,
};

// #[cfg(test)]
// mod test_helpers;

#[cfg(test)]
#[cfg(feature = "test_blockchain_api")]
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
        Self { provider_pool: ProviderPool::new(project_id), config }
    }

    // Or you can make chain-specific clients if you want to do it manually
    // async fn new(chain_id: Caip10, rpc_url: Url, bundler_url: Url, paymaster_url: Url) -> Self {}

    // Prepare and send gas-abstracted transactions
    async fn prepare_gas_abstraction(
        &self,
        transaction: Transaction,
    ) -> Result<PreparedGasAbstraction, String> {
        let Transaction { chain_id, from, to, value, input } = transaction;

        let provider = self
            .provider_pool
            .get_provider(format!("eip155:{chain_id}",))
            .await;

        let code = provider.get_code_at(from).await.unwrap();
        // TODO check if the code is our contract, or something else
        // If no code, return 7702 txn and UserOp hash to sign
        // If our contract, return UserOp hash to sign
        // If something else then return an error

        // let mut hashes = Vec::with_capacity(2);
        if code.is_empty() {
            // return 7702 txn
        }

        // Else assume it's our account for now

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
            chain_id,
            valid_after,
            valid_until,
        );
        let hash = safe_op.eip712_signing_hash(&domain);

        Ok(PreparedGasAbstraction {
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

    async fn execute_transaction(
        &self,
        signatures: Vec<OwnerSignature>,
        params: DoSendTransactionParams,
    ) {
        let bundler_client = BundlerClient::new(BundlerConfig::new(
            self.config.endpoints.bundler.base_url.clone(),
        ));
        let user_op =
            encode_send_transactions(signatures, params).await.unwrap();
        let _user_operation_hash = bundler_client
            .send_user_operation(ENTRYPOINT_ADDRESS_V07.into(), user_op.clone())
            .await
            .unwrap();
    }

    // Signature creation
    // async fn ..... COPY signature creation functions from current SDK

    // == Future APIs ==
    // async fn send_user_operation(?) - ?;
    // async fn create_smart_session(?) -> ?;
}

#[derive(Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct Transaction {
    pub chain_id: U64,
    pub from: Address,
    pub to: Address,
    pub value: U256,
    pub input: Bytes,
}

#[derive(Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct PreparedGasAbstraction {
    // hashes_to_sign: Vec<B256>,
    // fees: GasAbstractionFees, // similar to RouteUiFields -> fee and other UI info for user display
    prepared_send_transaction: PreparedSendTransaction,
}

#[derive(Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct Params {
    /// EOA signatures by the transaction's sender of `hashes_to_sign`
    signatures: Vec<Vec<OwnerSignature>>,
    params: DoSendTransactionParams,
    initial_transaction: TransactionRequest,
}
