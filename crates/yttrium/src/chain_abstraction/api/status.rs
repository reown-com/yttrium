use {
    relay_rpc::domain::ProjectId,
    serde::{Deserialize, Serialize},
};

pub const STATUS_ENDPOINT_PATH: &str = "/v1/ca/orchestrator/status";

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StatusQueryParams {
    pub project_id: ProjectId,
    pub orchestration_id: String,
    pub session_id: Option<String>,
    #[serde(rename = "st")]
    pub sdk_type: Option<String>,
    #[serde(rename = "sv")]
    pub sdk_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(any(feature = "uniffi", feature = "uniffi_derive"), derive(uniffi_macros::Record))]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub struct StatusResponsePendingObject {
    pub created_at: u64,
    /// Polling interval in ms for the client
    pub check_in: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(any(feature = "uniffi", feature = "uniffi_derive"), derive(uniffi_macros::Record))]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub struct StatusResponseCompleted {
    pub created_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(any(feature = "uniffi", feature = "uniffi_derive"), derive(uniffi_macros::Record))]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub struct StatusResponseError {
    pub created_at: u64,
    pub error: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(any(feature = "uniffi", feature = "uniffi_derive"), derive(uniffi_macros::Enum))]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "UPPERCASE", tag = "status")]
pub enum StatusResponse {
    Pending(StatusResponsePendingObject),
    Completed(StatusResponseCompleted),
    Error(StatusResponseError),
}
