use crate::ffi::FFIRouteError;
use serde_json;
use yttrium::chain_abstraction::api::Transaction;
use yttrium::chain_abstraction::client::Client;
use yttrium::chain_abstraction::error::RouteError;

pub struct FFIChainClient {
    client: Client,
}

impl FFIChainClient {
    pub fn new(project_id: String) -> Self {
        let client = Client::new(project_id.into()); // Convert String to ProjectId
        Self { client }
    }

    pub async fn route(
        &self,
        transaction: String,
    ) -> Result<String, FFIRouteError> {
        // Parse the transaction JSON string
        let transaction: Transaction = serde_json::from_str(&transaction)
            .map_err(|e| FFIRouteError::Request(e.to_string()))?;

        // Call the underlying client
        let response =
            self.client.route(transaction).await.map_err(|e| match e {
                RouteError::Request(e) => FFIRouteError::Request(e.to_string()),
                RouteError::RequestFailed(e) => {
                    let msg = e.unwrap_or_else(|err| err.to_string());
                    FFIRouteError::RequestFailed(msg)
                }
            })?;

        // Serialize the RouteResponse into a JSON string
        let json_response = serde_json::to_string(&response)
            .map_err(|e| FFIRouteError::Request(e.to_string()))?;

        Ok(json_response)
    }

    pub async fn status(
        &self,
        orchestration_id: String,
    ) -> Result<String, FFIRouteError> {
        // Call the underlying client
        let response = self.client.status(orchestration_id).await.map_err(
            |e| match e {
                RouteError::Request(e) => FFIRouteError::Request(e.to_string()),
                RouteError::RequestFailed(e) => {
                    let msg = e.unwrap_or_else(|err| err.to_string());
                    FFIRouteError::RequestFailed(msg)
                }
            },
        )?;

        // Serialize the response to JSON
        let json_response = serde_json::to_string(&response)
            .map_err(|e| FFIRouteError::Request(e.to_string()))?;

        // Return the JSON string
        Ok(json_response)
    }
}
