use crate::ffi::{
    FFIEip1559Estimation, FFIError, FFIEthTransaction, FFIRouteError,
    FFIRouteResponse, FFIRouteResponseSuccess, FFIStatusResponse,
    FFIStatusResponseSuccess,
};
use alloy::primitives::Address;
use alloy::providers::Provider;
use alloy::{network::Ethereum, providers::ReqwestProvider};
use serde_json;
use yttrium::chain_abstraction::api::route::{
    RouteResponse, RouteResponseSuccess,
};
use yttrium::chain_abstraction::api::status::{
    StatusResponse, StatusResponseSuccess,
};
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
        transaction: FFIEthTransaction,
    ) -> Result<FFIRouteResponse, FFIRouteError> {
        // Map FFIEthTransaction to Transaction

        // Parse the 'from' and 'to' addresses
        let from_address =
            transaction.from.parse::<Address>().map_err(|e| {
                FFIRouteError::Request(format!("Invalid 'from' address: {}", e))
            })?;
        let to_address = transaction.to.parse::<Address>().map_err(|e| {
            FFIRouteError::Request(format!("Invalid 'to' address: {}", e))
        })?;

        // Construct the Transaction
        let transaction = Transaction {
            from: from_address,
            to: to_address,
            value: transaction.value,
            gas: transaction.gas,
            gas_price: transaction.gas_price,
            data: transaction.data,
            nonce: transaction.nonce,
            max_fee_per_gas: transaction.max_fee_per_gas,
            max_priority_fee_per_gas: transaction.max_priority_fee_per_gas,
            chain_id: transaction.chain_id,
        };

        // Call the underlying client
        let route_response =
            self.client.route(transaction).await.map_err(|route_error| {
                match route_error {
                    RouteError::Request(reqwest_error) => {
                        FFIRouteError::Request(reqwest_error.to_string())
                    }
                    RouteError::RequestFailed(result) => {
                        let message =
                            result.unwrap_or_else(|err| err.to_string());
                        FFIRouteError::RequestFailed(message)
                    }
                }
            })?;

        // Map the RouteResponse to FFIRouteResponse
        let ffi_route_response = match route_response {
            RouteResponse::Success(success) => {
                let ffi_success = match success {
                    RouteResponseSuccess::Available(available) => {
                        // Serialize `available` into a JSON string
                        let json_string = serde_json::to_string(&available)
                            .map_err(|e| {
                                FFIRouteError::RequestFailed(e.to_string())
                            })?;
                        FFIRouteResponseSuccess::Available(json_string)
                    }
                    RouteResponseSuccess::NotRequired(not_required) => {
                        // Serialize `not_required` into a JSON string
                        let json_string = serde_json::to_string(&not_required)
                            .map_err(|e| {
                                FFIRouteError::RequestFailed(e.to_string())
                            })?;
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
    ) -> Result<FFIStatusResponse, FFIRouteError> {
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

        // Map the StatusResponse to FFIStatusResponse
        let ffi_status_response = match response {
            StatusResponse::Success(success) => {
                let ffi_success = match success {
                    StatusResponseSuccess::Pending(pending) => {
                        // Serialize `pending` into a JSON string
                        let json_string = serde_json::to_string(&pending)
                            .map_err(|e| {
                                FFIRouteError::RequestFailed(e.to_string())
                            })?;
                        FFIStatusResponseSuccess::Pending(json_string)
                    }
                    StatusResponseSuccess::Completed(completed) => {
                        // Serialize `completed` into a JSON string
                        let json_string = serde_json::to_string(&completed)
                            .map_err(|e| {
                                FFIRouteError::RequestFailed(e.to_string())
                            })?;
                        FFIStatusResponseSuccess::Completed(json_string)
                    }
                    StatusResponseSuccess::Error(error) => {
                        // Serialize `error` into a JSON string
                        let json_string = serde_json::to_string(&error)
                            .map_err(|e| {
                                FFIRouteError::RequestFailed(e.to_string())
                            })?;
                        FFIStatusResponseSuccess::Error(json_string)
                    }
                };
                FFIStatusResponse::Success(ffi_success)
            }
            StatusResponse::Error(error) => {
                // Serialize `error` into a JSON string
                let json_string = serde_json::to_string(&error)
                    .map_err(|e| FFIRouteError::RequestFailed(e.to_string()))?;
                FFIStatusResponse::Error(json_string)
            }
        };

        Ok(ffi_status_response)
    }

    pub async fn estimate_fees(
        &self,
        chain_id: String,
    ) -> Result<FFIEip1559Estimation, FFIError> {
        let url = format!(
            "https://rpc.walletconnect.com/v1?chainId={chain_id}&projectId={}",
            self.client.project_id
        )
        .parse()
        .expect("Invalid RPC URL");
        let provider = ReqwestProvider::<Ethereum>::new_http(url);
        provider
            .estimate_eip1559_fees(None)
            .await
            .map_err(|e| FFIError::Unknown(e.to_string()))
            .map(|fees| FFIEip1559Estimation {
                max_fee_per_gas: fees.max_fee_per_gas as i64,
                max_priority_fee_per_gas: fees.max_priority_fee_per_gas as i64,
            })
    }
}
