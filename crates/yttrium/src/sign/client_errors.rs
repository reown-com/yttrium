use crate::sign::storage::StorageError;

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
    #[error("Get public key: {0}")]
    GetPublicKey(String),

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
    #[error("Storage: {0}")]
    Storage(StorageError),

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
    #[error("Storage: {0}")]
    Storage(StorageError),

    #[error("Should never happen: {0}")]
    ShouldNeverHappen(String),

    #[error("Request error: {0}")]
    Request(RequestError),
}

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Error))]
#[error("Sign emit error: {0}")]
pub enum EmitError {
    #[error("Storage: {0}")]
    Storage(StorageError),

    #[error("Session not found")]
    SessionNotFound,

    #[error("Request: {0}")]
    Request(RequestError),

    #[error("Should never happen: {0}")]
    ShouldNeverHappen(String),
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
#[error("Sign extend error: {0}")]
pub enum ExtendError {
    #[error("Storage: {0}")]
    Storage(StorageError),

    #[error("Session not found")]
    SessionNotFound,

    #[error("Invalid expiry value")]
    InvalidExpiry,

    #[error("Request: {0}")]
    Request(RequestError),

    #[error("Should never happen: {0}")]
    ShouldNeverHappen(String),
}

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Error))]
#[error("Sign update error: {0}")]
pub enum UpdateError {
    #[error("Storage: {0}")]
    Storage(StorageError),

    #[error("Session not found")]
    SessionNotFound,

    #[error("Unauthorized: not controller")]
    Unauthorized,

    #[error("Request: {0}")]
    Request(RequestError),

    #[error("Internal: {0}")]
    Internal(String),

    #[error("Should never happen: {0}")]
    ShouldNeverHappen(String),
}
