use {
    super::{
        api::{
            prepare::{PrepareResponseError, PrepareResponseNotRequired},
            status::{StatusResponseError, StatusResponsePendingObject},
        },
        ui_fields::UiFields,
    },
    alloy::{
        primitives::B256,
        transports::{RpcError, TransportErrorKind},
    },
    alloy_provider::PendingTransactionError,
    reqwest::StatusCode,
    serde::{Deserialize, Serialize},
    thiserror::Error,
};

#[derive(thiserror::Error, Debug)]
pub enum PrepareError {
    /// Retryable error
    #[error("HTTP request: {0}")]
    Request(reqwest::Error),

    /// Retryable error
    #[error("HTTP request failed: {0}, {1}")]
    RequestFailed(StatusCode, String),

    /// Retryable error
    #[error("HTTP request text failed: {0}, {1}")]
    RequestFailedText(StatusCode, reqwest::Error),

    /// Retryable error
    #[error("Decoding response as text failed: {0}, {1}")]
    DecodingText(StatusCode, reqwest::Error),

    /// Retryable error
    #[error("Decoding response as json failed: {0}, {1}, {2}")]
    DecodingJson(StatusCode, serde_json::Error, String),
}

#[derive(thiserror::Error, Debug)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
#[cfg_attr(feature = "wasm", derive(derive_jserror::JsError))]
pub enum StatusError {
    /// Retryable error
    #[error("HTTP request: {0}")]
    Request(reqwest::Error),

    /// Retryable error
    #[error("HTTP request failed: {0}, {1}")]
    RequestFailed(StatusCode, String),

    /// Retryable error
    #[error("HTTP request text failed: {0}, {1}")]
    RequestFailedText(StatusCode, reqwest::Error),

    /// Retryable error
    #[error("Decoding response as text failed: {0}, {1}")]
    DecodingText(StatusCode, reqwest::Error),

    /// Retryable error
    #[error("Decoding response as json failed: {0}, {1}, {2}")]
    DecodingJson(StatusCode, serde_json::Error, String),
}

#[derive(thiserror::Error, Debug)]
#[cfg_attr(feature = "wasm", derive(derive_jserror::JsError))]
pub enum UiFieldsError {
    /// Retryable error
    #[error("Fungibles HTTP request: {0}")]
    FungiblesRequest(reqwest::Error),

    /// Retryable error
    #[error("Fungibles HTTP request failed: {0}")]
    FungiblesRequestFailed(StatusCode, Result<String, reqwest::Error>),

    /// Retryable error
    #[error("Fungibles Json request: {0}")]
    FungiblesJson(reqwest::Error),

    /// Retryable error
    #[error("Eip1559Estimation: {0}")]
    Eip1559Estimation(RpcError<TransportErrorKind>),

    /// Retryable error
    #[error("L1DataFee: {0}")]
    L1DataFee(L1DataFeeError),
}

#[derive(thiserror::Error, Debug)]
#[cfg_attr(feature = "wasm", derive(derive_jserror::JsError))]
pub enum L1DataFeeError {}

#[derive(thiserror::Error, Debug)]
pub enum PrepareDetailedError {
    #[error("Prepare Error: {0}")]
    Prepare(PrepareError),

    #[error("UiFieldsError: {0}")]
    UiFields(UiFieldsError),
}

// TODO this response type shouldn't be in `error` module
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Enum))]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub enum PrepareDetailedResponse {
    Success(PrepareDetailedResponseSuccess),
    Error(PrepareResponseError),
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Enum))]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub enum PrepareDetailedResponseSuccess {
    Available(UiFields),
    NotRequired(PrepareResponseNotRequired),
}

impl PrepareDetailedResponseSuccess {
    pub fn into_option(self) -> Option<UiFields> {
        match self {
            Self::Available(a) => Some(a),
            Self::NotRequired(_) => None,
        }
    }
}

impl PrepareDetailedResponse {
    pub fn into_result(
        self,
    ) -> Result<PrepareDetailedResponseSuccess, PrepareResponseError> {
        match self {
            Self::Success(success) => Ok(success),
            Self::Error(error) => Err(error),
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
#[cfg_attr(feature = "wasm", derive(derive_jserror::JsError))]
pub enum WaitForSuccessError {
    #[error("Status: {0}")]
    Status(StatusError),

    #[error("StatusResponseError: {0:?}")]
    StatusResponseError(StatusResponseError),

    #[error("StatusResponsePending: {0:?}")]
    // renamed to `Object` to avoid conflicts: https://github.com/mozilla/uniffi-rs/issues/2402
    StatusResponsePending(StatusResponsePendingObject),
}

#[derive(thiserror::Error, Debug)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum ExecuteError {
    #[error("Execute Error: orchestration_id:{orchestration_id} - {reason}")]
    WithOrchestrationId { orchestration_id: String, reason: ExecuteErrorReason },
}

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum ExecuteErrorReason {
    #[error("Route: {0}")]
    Route(SendTransactionError),
    #[error("Bridge: {0}")]
    Bridge(WaitForSuccessError),
    #[error("Initial: {0}")]
    Initial(SendTransactionError),
}

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum SendTransactionError {
    #[error("Rpc: {0}")]
    Rpc(RpcError<TransportErrorKind>),

    #[error("PendingTransaction: {0}")]
    PendingTransaction(PendingTransactionError),

    #[error("Failed, txn hash: {txn_hash}")]
    Failed { txn_hash: B256 },
}
