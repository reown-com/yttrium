//! HTTP for calls 1-6 and the streamed poll, behind a trait with a native and a WASM implementation.

use {futures::Stream, std::pin::Pin};

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub mod native;
pub mod relay;
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub mod wasm;

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: &'static str,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
    /// Whole-request timeout for calls 1-6; `None` for the poll, which watches liveness itself.
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub retry_after_s: Option<u64>,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum HttpError {
    #[error("network: {0}")]
    Network(String),
}

#[cfg(not(target_arch = "wasm32"))]
pub type ByteStream =
    Pin<Box<dyn Stream<Item = Result<Vec<u8>, HttpError>> + Send>>;
#[cfg(target_arch = "wasm32")]
pub type ByteStream = Pin<Box<dyn Stream<Item = Result<Vec<u8>, HttpError>>>>;

pub struct StreamResponse {
    pub status: u16,
    pub retry_after_s: Option<u64>,
    pub body: ByteStream,
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait::async_trait]
pub trait HttpClient: Send + Sync {
    async fn request(
        &self,
        req: HttpRequest,
    ) -> Result<HttpResponse, HttpError>;
    async fn stream(
        &self,
        req: HttpRequest,
    ) -> Result<StreamResponse, HttpError>;
}

#[cfg(target_arch = "wasm32")]
#[async_trait::async_trait(?Send)]
pub trait HttpClient {
    async fn request(
        &self,
        req: HttpRequest,
    ) -> Result<HttpResponse, HttpError>;
    async fn stream(
        &self,
        req: HttpRequest,
    ) -> Result<StreamResponse, HttpError>;
}

pub fn parse_retry_after(v: Option<&str>) -> Option<u64> {
    v.and_then(|s| s.trim().parse().ok())
}
