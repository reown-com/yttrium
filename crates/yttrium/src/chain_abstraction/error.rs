use {
    super::{
        api::{
            prepare::{PrepareResponseError, PrepareResponseNotRequired},
            status::{StatusResponseError, StatusResponsePending},
        },
        ui_fields::UiFields,
    },
    reqwest::StatusCode,
    serde::{Deserialize, Serialize},
};

#[derive(thiserror::Error, Debug)]
pub enum PrepareError {
    /// Retryable error
    #[error("HTTP request: {0}")]
    Request(reqwest::Error),

    /// Retryable error
    #[error("HTTP request failed: {0}")]
    RequestFailed(String),
    #[error("HTTP request text failed: {0}")]
    RequestFailedText(reqwest::Error),

    /// Retryable error
    #[error("Decoding response as text failed: {0}")]
    DecodingText(reqwest::Error),

    /// Retryable error
    #[error("Decoding response as json failed: {0}")]
    DecodingJson(serde_json::Error, String),
}

#[cfg(feature = "wasm")]
impl From<PrepareError> for wasm_bindgen::prelude::JsValue {
    fn from(error: PrepareError) -> Self {
        wasm_bindgen::prelude::JsValue::from_str(&error.to_string())
    }
}

#[derive(thiserror::Error, Debug)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum StatusError {
    /// Retryable error
    #[error("HTTP request: {0}")]
    Request(reqwest::Error),

    /// Retryable error
    #[error("HTTP request failed: {0}")]
    RequestFailed(String),
    #[error("HTTP request text failed: {0}")]
    RequestFailedText(reqwest::Error),

    /// Retryable error
    #[error("Decoding response as text failed: {0}")]
    DecodingText(reqwest::Error),

    /// Retryable error
    #[error("Decoding response as json failed: {0}")]
    DecodingJson(serde_json::Error, String),
}

#[cfg(feature = "wasm")]
impl From<StatusError> for wasm_bindgen::prelude::JsValue {
    fn from(error: StatusError) -> Self {
        wasm_bindgen::prelude::JsValue::from_str(&error.to_string())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum UiFieldsError {
    /// Retryable error
    #[error("HTTP request: {0}")]
    Request(reqwest::Error),

    /// Retryable error
    #[error("HTTP request failed: {0}")]
    RequestFailed(StatusCode, Result<String, reqwest::Error>),

    /// Retryable error
    #[error("Json request: {0}")]
    Json(reqwest::Error),
}

#[cfg(feature = "wasm")]
impl From<UiFieldsError> for wasm_bindgen::prelude::JsValue {
    fn from(error: UiFieldsError) -> Self {
        wasm_bindgen::prelude::JsValue::from_str(&error.to_string())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum PrepareDetailedError {
    #[error("Prepare Error: {0}")]
    Prepare(PrepareError),

    #[error("UiFieldsError: {0}")]
    UiFields(UiFieldsError),
}

#[cfg(feature = "wasm")]
impl From<PrepareDetailedError> for wasm_bindgen::prelude::JsValue {
    fn from(error: PrepareDetailedError) -> Self {
        wasm_bindgen::prelude::JsValue::from_str(&error.to_string())
    }
}

// TODO this response type shouldn't be in `error` module
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Enum))]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub enum PrepareDetailedResponse {
    Success(PrepareDetailedResponseSuccess),
    Error(PrepareResponseError),
}

#[derive(Debug, Serialize, Deserialize)]
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
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum WaitForSuccessError {
    #[error("Status: {0}")]
    Status(StatusError),

    #[error("StatusResponseError: {0:?}")]
    StatusResponseError(StatusResponseError),

    #[error("StatusResponsePending: {0:?}")]
    StatusResponsePending(StatusResponsePending),
}

#[cfg(feature = "wasm")]
impl From<WaitForSuccessError> for wasm_bindgen::prelude::JsValue {
    fn from(error: WaitForSuccessError) -> Self {
        wasm_bindgen::prelude::JsValue::from_str(&error.to_string())
    }
}
