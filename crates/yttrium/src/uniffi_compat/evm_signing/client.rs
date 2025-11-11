use {
    super::{
        sign_and_send_transaction, EvmSigningError, SignAndSendParams,
        SignAndSendResult,
    },
    crate::{
        blockchain_api::BLOCKCHAIN_API_URL_PROD, provider_pool::ProviderPool,
        pulse::PulseMetadata,
    },
    alloy::signers::local::PrivateKeySigner,
    relay_rpc::domain::ProjectId,
    reqwest::{Client as ReqwestClient, Url},
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
}
