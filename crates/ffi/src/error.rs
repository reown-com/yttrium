use crate::ffi::FFIError;

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
