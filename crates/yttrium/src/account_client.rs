use {
    crate::{
        bundler::{
            client::BundlerClient, config::BundlerConfig,
            pimlico::paymaster::client::PaymasterClient,
        },
        call::{
            Call,
            send::{
                do_send_transactions, prepare_send_transaction,
                safe_test::{
                    self, DoSendTransactionParams, OwnerSignature,
                    PreparedSendTransaction,
                },
            },
        },
        config::Config,
        smart_accounts::{
            account_address::AccountAddress,
            safe::{
                Owners, PreparedSignature, SignOutputEnum, SignStep3Params,
                prepare_sign, sign, sign_step_3,
            },
        },
    },
    alloy::{
        primitives::{B256, Bytes, U64, U256},
        providers::ProviderBuilder,
        rpc::types::UserOperationReceipt,
    },
};

#[cfg_attr(any(feature = "uniffi", feature = "uniffi_derive"), derive(uniffi_macros::Object))]
pub struct AccountClient {
    owner: AccountAddress,
    chain_id: u64,
    pub config: Config,
}

impl AccountClient {
    pub fn new(owner: AccountAddress, chain_id: u64, config: Config) -> Self {
        Self { owner, chain_id, config }
    }

    pub fn chain_id(&self) -> u64 {
        self.chain_id
    }

    pub async fn get_address(&self) -> eyre::Result<AccountAddress> {
        safe_test::get_address(self.owner, self.config.clone()).await
    }

    pub fn prepare_sign_message(
        &self,
        message_hash: B256,
    ) -> PreparedSignature {
        prepare_sign(
            // TODO refactor class to parse Address on AccountClient
            // initialization instead of lazily
            self.owner,
            U256::from(U64::from(self.chain_id)),
            message_hash,
        )
    }

    pub async fn do_sign_message(
        &self,
        signatures: Vec<OwnerSignature>,
    ) -> eyre::Result<SignOutputEnum> {
        // TODO refactor class to create Provider on AccountClient
        // initialization instead of lazily
        let provider = ProviderBuilder::new()
            .connect_http(self.config.endpoints.rpc.base_url.parse().unwrap());

        sign(
            Owners { owners: vec![self.owner.into()], threshold: 1 },
            self.get_address().await?,
            signatures,
            &provider,
            PaymasterClient::new(BundlerConfig::new(
                self.config.endpoints.paymaster.base_url.parse().unwrap(),
            )),
        )
        .await
    }

    pub async fn finalize_sign_message(
        &self,
        signatures: Vec<OwnerSignature>,
        sign_step_3_params: SignStep3Params,
    ) -> eyre::Result<Bytes> {
        sign_step_3(signatures, sign_step_3_params).await
    }

    pub async fn prepare_send_transactions(
        &self,
        calls: Vec<Call>,
    ) -> eyre::Result<PreparedSendTransaction> {
        prepare_send_transaction(
            calls,
            self.owner,
            self.chain_id,
            self.config.clone(),
        )
        .await
    }

    pub async fn do_send_transactions(
        &self,
        signatures: Vec<OwnerSignature>,
        do_send_transaction_params: DoSendTransactionParams,
    ) -> eyre::Result<Bytes> {
        do_send_transactions(
            signatures,
            do_send_transaction_params,
            self.chain_id,
            self.config.clone(),
        )
        .await
    }

    pub async fn wait_for_user_operation_receipt(
        &self,
        user_operation_hash: Bytes,
    ) -> eyre::Result<UserOperationReceipt> {
        println!("Querying for receipts...");

        let bundler_client = BundlerClient::new(BundlerConfig::new(
            self.config.endpoints.bundler.base_url.parse()?,
        ));
        let receipt = bundler_client
            .wait_for_user_operation_receipt(user_operation_hash)
            .await?;

        println!("Received User Operation receipt: {receipt:?}");

        let tx_hash = receipt.clone().receipt.transaction_hash;
        println!(
            "UserOperation included: https://sepolia.etherscan.io/tx/{tx_hash}"
        );
        Ok(receipt)
    }
}
