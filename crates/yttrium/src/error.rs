use std::{error::Error, fmt};

#[derive(Eq, Hash, PartialEq, Debug, Clone, Default, PartialOrd, Ord)]
pub struct YttriumError {
    pub message: String,
}

impl fmt::Display for YttriumError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for YttriumError {
    fn description(&self) -> &str {
        &self.message
    }
}

#[cfg(any(feature = "account_client", feature = "chain_abstraction_client"))]
impl From<alloy::signers::Error> for YttriumError {
    fn from(e: alloy::signers::Error) -> Self {
        YttriumError { message: e.to_string() }
    }
}
