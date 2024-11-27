use reqwest::StatusCode;

use super::api::status::{StatusResponseError, StatusResponsePending};

#[derive(thiserror::Error, Debug)]
pub enum RouteError {
    /// Retryable error
    #[error("HTTP request: {0}")]
    Request(reqwest::Error),

    /// Retryable error
    #[error("HTTP request failed: {0:?}")]
    RequestFailed(Result<String, reqwest::Error>),
}

#[derive(thiserror::Error, Debug)]
pub enum WaitForSuccessError {
    #[error("Route Error: {0}")]
    RouteError(RouteError),

    #[error("StatusResponseError: {0:?}")]
    StatusResponseError(StatusResponseError),

    #[error("StatusResponsePending: {0:?}")]
    StatusResponsePending(StatusResponsePending),
}

#[derive(thiserror::Error, Debug)]
pub enum RouteUiFieldsError {
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
