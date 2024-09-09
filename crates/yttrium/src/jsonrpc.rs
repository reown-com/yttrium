use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

pub type Response<T> = Result<Option<T>, ErrorPayload<T>>;

#[derive(Debug, Deserialize, Error)]
pub struct ErrorPayload<T> {
    pub message: String,
    pub data: Option<T>,
    pub code: Option<i64>,
}

impl<T> fmt::Display for ErrorPayload<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ErrorPayload {{ message: {} }}", self.message)
    }
}

#[derive(Debug, Deserialize)]
pub struct JSONRPCResponse<T> {
    pub jsonrpc: String,
    pub id: u64,
    pub result: Option<T>,
    pub error: Option<ErrorPayload<T>>,
}

impl<T> Into<Response<T>> for JSONRPCResponse<T> {
    fn into(self) -> Response<T> {
        if let Some(result) = self.result {
            return Ok(Some(result));
        }

        if let Some(error) = self.error {
            return Err(error);
        }

        Ok(None)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Request<T> {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestWithEmptyParams {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
}
