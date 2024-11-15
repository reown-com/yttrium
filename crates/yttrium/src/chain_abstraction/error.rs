#[derive(thiserror::Error, Debug)]
pub enum RouteError {
    /// Retryable error
    #[error("HTTP request: {0}")]
    Request(reqwest::Error),

    /// Retryable error
    #[error("HTTP request failed: {0:?}")]
    RequestFailed(Result<String, reqwest::Error>),
}
