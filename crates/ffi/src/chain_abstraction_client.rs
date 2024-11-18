use alloy::providers::Provider;
use yttrium::chain_abstraction::api::route::RouteResponse;
use yttrium::chain_abstraction::api::status::StatusResponse;
use yttrium::chain_abstraction::api::Transaction as CATransaction;

use alloy::{network::Ethereum, providers::ReqwestProvider};
use relay_rpc::domain::ProjectId;
use yttrium::chain_abstraction::client::Client;

pub struct FFIChainClient {
    client: Client,
}

impl FFIChainClient {
    pub fn new(config: FFIChainClientConfig) -> Self {
        let project_id = ProjectId::new(config.project_id);
        let mut client = Client::new(project_id);
        
        if let Some(base_url) = config.base_url {
            client.base_url = base_url.parse().unwrap();
        }

        Self { client }
    }

    pub async fn route(
        &self,
        transaction: String,
    ) -> Result<FFIRouteResponse, FFIRouteError> {
        // Parse the transaction JSON string
        let transaction: Transaction = serde_json::from_str(&transaction)
            .map_err(|e| FFIRouteError::Request(e.to_string()))?;

        // Call the underlying client
        let response = self.client.route(transaction).await
            .map_err(|e| match e {
                RouteError::Request(e) => FFIRouteError::Request(e.to_string()),
                RouteError::RequestFailed(e) => FFIRouteError::RequestFailed(e),
            })?;

        // Convert to FFI response
        Ok(FFIRouteResponse {
            orchestration_id: response.orchestration_id,
            status: response.status,
        })
    }

    pub async fn status(
        &self,
        orchestration_id: String,
    ) -> Result<FFIStatusResponse, FFIRouteError> {
        // Call the underlying client
        let response = self.client.status(orchestration_id).await
            .map_err(|e| match e {
                RouteError::Request(e) => FFIRouteError::Request(e.to_string()),
                RouteError::RequestFailed(e) => FFIRouteError::RequestFailed(e),
            })?;

        // Convert to FFI response
        Ok(FFIStatusResponse {
            status: response.status,
            result: response.result,
            error: response.error,
        })
    }
}