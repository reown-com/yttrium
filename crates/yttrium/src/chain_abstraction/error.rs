use {
    super::{
        api::{
            prepare::{PrepareResponseError, PrepareResponseNotRequired},
            status::{StatusResponseError, StatusResponsePending},
        },
        ui_fields::UiFields,
    },
    reqwest::StatusCode,
};

#[derive(thiserror::Error, Debug)]
pub enum PrepareError {
    /// Retryable error
    #[error("HTTP request: {0}")]
    Request(reqwest::Error),

    /// Retryable error
    #[error("HTTP request failed: {0:?}")]
    RequestFailed(Result<String, reqwest::Error>),

    /// Retryable error
    #[error("Decoding response as text failed: {0:?}")]
    DecodingText(reqwest::Error),

    /// Retryable error
    #[error("Decoding response as json failed: {0:?}")]
    DecodingJson(serde_json::Error, String),
}

#[derive(thiserror::Error, Debug)]
pub enum UiFieldsError {
    /// Retryable error
    #[error("HTTP request: {0}")]
    Request(reqwest::Error),

    /// Retryable error
    #[error("HTTP request failed: {0:?}")]
    RequestFailed(StatusCode, Result<String, reqwest::Error>),

    /// Retryable error
    #[error("Json request: {0}")]
    Json(reqwest::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum PrepareDetailedError {
    #[error("Prepare Error: {0}")]
    Prepare(PrepareError),

    #[error("UiFieldsError: {0}")]
    UiFields(UiFieldsError),
}

#[derive(Debug)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Enum))]
pub enum PrepareDetailedResponse {
    Success(PrepareDetailedResponseSuccess),
    Error(PrepareResponseError),
}

#[derive(Debug)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Enum))]
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
pub enum WaitForSuccessError {
    #[error("Prepare Error: {0}")]
    Prepare(PrepareError),

    #[error("StatusResponseError: {0:?}")]
    StatusResponseError(StatusResponseError),

    #[error("StatusResponsePending: {0:?}")]
    StatusResponsePending(StatusResponsePending),
}
