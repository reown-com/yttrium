use {
    super::{
        EvmSigningError, SignAndSendParams, SignAndSendResult,
        sign_and_send_transaction, sign_typed_data,
    },
    crate::{
        blockchain_api::BLOCKCHAIN_API_URL_PROD, erc20::ERC20,
        provider_pool::ProviderPool, pulse::PulseMetadata,
    },
    alloy::{primitives::Address, signers::local::PrivateKeySigner},
    alloy_provider::Provider,
    relay_rpc::domain::ProjectId,
    reqwest::{Client as ReqwestClient, Url},
    std::str::FromStr,
};

#[derive(uniffi::Object)]
pub struct EvmSigningClient {
    provider_pool: ProviderPool,
    #[allow(unused)]
    http_client: ReqwestClient,
    #[allow(unused)]
    project_id: ProjectId,
    #[allow(unused)]
    pulse_metadata: PulseMetadata,
}

#[uniffi::export(async_runtime = "tokio")]
impl EvmSigningClient {
    #[uniffi::constructor]
    pub fn new(project_id: ProjectId, pulse_metadata: PulseMetadata) -> Self {
        Self::with_blockchain_api_url(
            project_id,
            pulse_metadata,
            BLOCKCHAIN_API_URL_PROD.parse().unwrap(),
        )
    }

    #[uniffi::constructor]
    pub fn with_blockchain_api_url(
        project_id: ProjectId,
        pulse_metadata: PulseMetadata,
        blockchain_api_base_url: Url,
    ) -> Self {
        let client = ReqwestClient::builder().build().unwrap();
        Self {
            provider_pool: ProviderPool::new(
                project_id.clone(),
                client.clone(),
                pulse_metadata.clone(),
                blockchain_api_base_url,
            ),
            http_client: client,
            project_id,
            pulse_metadata,
        }
    }

    pub async fn sign_and_send(
        &self,
        params: SignAndSendParams,
        signer: &PrivateKeySigner,
    ) -> Result<SignAndSendResult, EvmSigningError> {
        sign_and_send_transaction(&self.provider_pool, params, signer).await
    }

    pub fn sign_typed_data(
        &self,
        json_data: String,
        signer: &PrivateKeySigner,
    ) -> Result<String, EvmSigningError> {
        sign_typed_data(json_data, signer)
    }

    /// Get the balance of an ERC20 token for a wallet address.
    /// Returns the balance as a decimal string (raw units, not formatted).
    pub async fn get_token_balance(
        &self,
        chain_id: String,
        contract_address: String,
        wallet_address: String,
    ) -> Result<String, EvmSigningError> {
        let contract = Address::from_str(&contract_address)
            .map_err(|e| EvmSigningError::InvalidAddress(e.to_string()))?;
        let wallet = Address::from_str(&wallet_address)
            .map_err(|e| EvmSigningError::InvalidAddress(e.to_string()))?;
        let provider = self.provider_pool.get_provider(&chain_id).await;
        let erc20 = ERC20::new(contract, provider);
        let balance = erc20
            .balanceOf(wallet)
            .call()
            .await
            .map_err(|e| EvmSigningError::BalanceFetch(e.to_string()))?;
        Ok(balance.to_string())
    }

    /// Get the native token balance (ETH, MATIC, etc.) for a wallet address.
    /// Returns the balance as a decimal string in wei.
    pub async fn get_native_balance(
        &self,
        chain_id: String,
        wallet_address: String,
    ) -> Result<String, EvmSigningError> {
        let wallet = Address::from_str(&wallet_address)
            .map_err(|e| EvmSigningError::InvalidAddress(e.to_string()))?;
        let provider = self.provider_pool.get_provider(&chain_id).await;
        let balance = provider
            .get_balance(wallet)
            .await
            .map_err(|e| EvmSigningError::BalanceFetch(e.to_string()))?;
        Ok(balance.to_string())
    }
}
