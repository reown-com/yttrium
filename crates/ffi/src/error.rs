use crate::ffi::{FFIRouteError, FFIError};

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