use serde_json;
use yttrium::chain_abstraction::api::Transaction;
use yttrium::chain_abstraction::client::Client;
use yttrium::chain_abstraction::error::RouteError;
use crate::ffi::{FFIRouteResponse, FFIRouteResponseSuccess, FFIRouteError};
use yttrium::chain_abstraction::api::route::{RouteResponse, RouteResponseSuccess};


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
    ) -> Result<FFIRouteResponse, FFIRouteError> {
        // Parse the transaction JSON string into a Transaction
        let transaction: Transaction = serde_json::from_str(&transaction)
            .map_err(|e| FFIRouteError::RequestFailed(e.to_string()))?;
    
        // Call the underlying client
        let route_response = self.client.route(transaction).await
            .map_err(|route_error| match route_error {
                RouteError::Request(reqwest_error) => {
                    FFIRouteError::Request(reqwest_error.to_string())
                }
                RouteError::RequestFailed(result) => {
                    let message = match result {
                        Ok(response_body) => response_body,
                        Err(reqwest_error) => reqwest_error.to_string(),
                    };
                    FFIRouteError::RequestFailed(message)
                }
            })?;
    
        // Map the RouteResponse to FFIRouteResponse
        let ffi_route_response = match route_response {
            RouteResponse::Success(success) => {
                let ffi_success = match success {
                    RouteResponseSuccess::Available(available) => {
                        // Serialize `available` into a JSON string
                        let json_string = serde_json::to_string(&available)
                            .map_err(|e| FFIRouteError::RequestFailed(e.to_string()))?;
                        FFIRouteResponseSuccess::Available(json_string)
                    }
                    RouteResponseSuccess::NotRequired(not_required) => {
                        // Serialize `not_required` into a JSON string
                        let json_string = serde_json::to_string(&not_required)
                            .map_err(|e| FFIRouteError::RequestFailed(e.to_string()))?;
                        FFIRouteResponseSuccess::NotRequired(json_string)
                    }
                };
                FFIRouteResponse::Success(ffi_success)
            }
            RouteResponse::Error(error) => {
                let error_message = format!("{:?}", error.error); // Convert error to string
                FFIRouteResponse::Error(error_message)
            }
        };
    
        Ok(ffi_route_response)
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
