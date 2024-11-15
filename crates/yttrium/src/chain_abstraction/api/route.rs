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
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub funding_from: Vec<FundingMetadata>,
    pub check_in: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BridgingStatus {
    BridgingAvailable,
    BridgingNotAvailable,
    BridgingNotRequired,
    InsufficientFunds,
    InsufficientGasFunds,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FundingMetadata {
    pub chain_id: String,
    pub token_contract: Address,
    pub symbol: String,
    pub amount: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteResponseBridging {
    pub orchestration_id: String,
    pub transactions: Vec<Transaction>,
    pub metadata: Metadata,
    pub status: BridgingStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteResponseError {
    pub status: BridgingStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RouteResponse {
    Success(RouteResponseBridging),
    Error(RouteResponseError),
}

impl RouteResponse {
    pub fn into_result(
        self,
    ) -> Result<RouteResponseBridging, RouteResponseError> {
        match self {
            Self::Success(success) => Ok(success),
            Self::Error(error) => Err(error),
        }
    }
}
