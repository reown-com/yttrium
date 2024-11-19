use super::Transaction;
use alloy::primitives::Address;
use relay_rpc::domain::ProjectId;
use serde::{Deserialize, Serialize};

pub const ROUTE_ENDPOINT_PATH: &str = "/v1/ca/orchestrator/route";

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RouteQueryParams {
    pub project_id: ProjectId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteRequest {
    pub transaction: Transaction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub funding_from: Vec<FundingMetadata>,
    pub check_in: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
#[serde(rename_all = "camelCase")]
pub struct FundingMetadata {
    pub chain_id: String,
    pub token_contract: Address,
    pub symbol: String,
    pub amount: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
#[serde(rename_all = "camelCase")]
pub struct RouteResponseAvailable {
    pub orchestration_id: String,
    pub transactions: Vec<Transaction>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
#[serde(rename_all = "camelCase")]
pub struct RouteResponseNotRequired {
    #[serde(rename = "transactions")]
    _flag: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Enum))]
#[serde(untagged)]
pub enum RouteResponseSuccess {
    Available(RouteResponseAvailable),
    NotRequired(RouteResponseNotRequired),
}

impl RouteResponseSuccess {
    pub fn into_option(self) -> Option<RouteResponseAvailable> {
        match self {
            Self::Available(a) => Some(a),
            Self::NotRequired(_) => None,
        }
    }
}

/// Bridging check error response that should be returned as a normal HTTP 200
/// response
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
pub struct RouteResponseError {
    pub error: BridgingError,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Enum))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BridgingError {
    NoRoutesAvailable,
    InsufficientFunds,
    InsufficientGasFunds,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Enum))]
#[serde(untagged)]
pub enum RouteResponse {
    Success(RouteResponseSuccess),
    Error(RouteResponseError),
}

impl RouteResponse {
    pub fn into_result(
        self,
    ) -> Result<RouteResponseSuccess, RouteResponseError> {
        match self {
            Self::Success(success) => Ok(success),
            Self::Error(error) => Err(error),
        }
    }
}
