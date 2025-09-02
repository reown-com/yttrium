#[derive(Debug, thiserror::Error, Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Error))]
#[error("Sign request error: {0}")]
pub enum RequestError {
    #[error("Internal: {0}")]
    Internal(String),

    #[error("Offline")]
    Offline,

    #[error("Invalid auth")]
    InvalidAuth,

    /// An error that shouldn't happen (e.g. JSON serializing constant values)
    #[error("Should never happen: {0}")]
    ShouldNeverHappen(String),

    /// An error that shouldn't happen because the relay should be behaving as expected
    #[error("Server misbehaved: {0}")]
    ServerMisbehaved(String),

    #[error("Cleanup")]
    Cleanup,
}

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Error))]
#[error("Sign next error: {0}")]
pub enum NextError {
    #[error("Internal: {0}")]
    Internal(String),
}

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Error))]
#[error("Sign pair error: {0}")]
pub enum PairError {
    #[error("Request error: {0}")]
    Request(RequestError),

    #[error("Internal: {0}")]
    Internal(String),

    #[error("Should never happen: {0}")]
    ShouldNeverHappen(String),
}

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Error))]
#[error("Sign approve error: {0}")]
pub enum ApproveError {
    #[error("Request error: {0}")]
    Request(RequestError),

    #[error("Internal: {0}")]
    Internal(String),

    #[error("Should never happen: {0}")]
    ShouldNeverHappen(String),
}

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Error))]
#[error("Sign reject error: {0}")]
pub enum RejectError {
    #[error("Request error: {0}")]
    Request(RequestError),

    #[error("Internal: {0}")]
    Internal(String),

    #[error("Should never happen: {0}")]
    ShouldNeverHappen(String),
}

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Error))]
#[error("Sign respond error: {0}")]
pub enum RespondError {
    #[error("Session not found")]
    SessionNotFound,

    #[error("Request: {0}")]
    Request(RequestError),

    #[error("Should never happen: {0}")]
    ShouldNeverHappen(String),
}

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Error))]
#[error("Sign disconnect error: {0}")]
pub enum DisconnectError {
    #[error("Should never happen: {0}")]
    ShouldNeverHappen(String),

    #[error("Request error: {0}")]
    Request(RequestError),
}

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Error))]
#[error("Sign connect error: {0}")]
pub enum ConnectError {
    #[error("Request error: {0}")]
    Request(RequestError),

    #[error("Internal: {0}")]
    Internal(String),

    #[error("Should never happen: {0}")]
    ShouldNeverHappen(String),
}

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Error))]
#[error("Sign update error: {0}")]
pub enum UpdateError {
    #[error("Session not found")]
    SessionNotFound,

    #[error("Request: {0}")]
    Request(RequestError),

    #[error("Internal: {0}")]
    Internal(String),

    #[error("Should never happen: {0}")]
    ShouldNeverHappen(String),
}
