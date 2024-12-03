use crate::ffi::{FFIError, FFIRouteError, FFIWaitForSuccessError};

impl std::fmt::Display for FFIError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FFIError::Unknown(message) => {
                write!(f, "Unknown error: {}", message)
            }
        }
    }
}

impl std::fmt::Debug for FFIError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FFIError::Unknown(message) => {
                write!(f, "Unknown error: {}", message)
            }
        }
    }
}

impl std::error::Error for FFIError {}

// Implement std::fmt::Display for FFIRouteError
impl std::fmt::Display for FFIRouteError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FFIRouteError::Request(message) => {
                write!(f, "Request error: {}", message)
            }
            FFIRouteError::RequestFailed(message) => {
                write!(f, "Request failed: {}", message)
            }
        }
    }
}

// Implement std::fmt::Debug for FFIRouteError
impl std::fmt::Debug for FFIRouteError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FFIRouteError::Request(message) => {
                write!(f, "Request error: {}", message)
            }
            FFIRouteError::RequestFailed(message) => {
                write!(f, "Request failed: {}", message)
            }
        }
    }
}

// Implement std::error::Error for FFIRouteError
impl std::error::Error for FFIRouteError {}

impl std::fmt::Display for FFIWaitForSuccessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FFIWaitForSuccessError::StatusResponseError(msg) => {
                write!(f, "StatusResponseError: {}", msg)
            }
            FFIWaitForSuccessError::StatusResponsePending(msg) => {
                write!(f, "StatusResponsePending: {}", msg)
            }
            FFIWaitForSuccessError::RouteError(err) => {
                write!(f, "RouteError: {}", err)
            }
            FFIWaitForSuccessError::Unknown(msg) => {
                write!(f, "Unknown error: {}", msg)
            }
        }
    }
}

impl std::fmt::Debug for FFIWaitForSuccessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FFIWaitForSuccessError::StatusResponseError(msg) => {
                write!(f, "StatusResponseError({:?})", msg)
            }
            FFIWaitForSuccessError::StatusResponsePending(msg) => {
                write!(f, "StatusResponsePending({:?})", msg)
            }
            FFIWaitForSuccessError::RouteError(err) => {
                write!(f, "RouteError({:?})", err)
            }
            FFIWaitForSuccessError::Unknown(msg) => {
                write!(f, "Unknown({:?})", msg)
            }
        }
    }
}

impl std::error::Error for FFIWaitForSuccessError {}