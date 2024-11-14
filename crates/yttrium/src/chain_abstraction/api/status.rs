use relay_rpc::domain::ProjectId;
use serde::{Deserialize, Serialize};

pub const STATUS_ENDPOINT_PATH: &str = "/v1/ca/orchestrator/status";

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StatusQueryParams {
    pub project_id: ProjectId,
    pub orchestration_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
#[serde(rename_all = "camelCase")]
pub struct StatusResponseSuccessPending {
    created_at: u32,
    /// Polling interval in ms for the client
    check_in: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
#[serde(rename_all = "camelCase")]
pub struct StatusResponseSuccessCompleted {
    created_at: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
#[serde(rename_all = "camelCase")]
pub struct StatusResponseSuccessError {
    created_at: u32,
    error_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Enum))]
#[serde(rename_all = "camelCase", tag = "status")]
pub enum StatusResponseSuccess {
    Pending(StatusResponseSuccessPending),
    Completed(StatusResponseSuccessCompleted),
    Error(StatusResponseSuccessError),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
#[serde(rename_all = "camelCase")]
pub struct StatusResponseError {
    pub error: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Enum))]
#[serde(untagged)]
pub enum StatusResponse {
    Success(StatusResponseSuccess),
    Error(StatusResponseError),
}
