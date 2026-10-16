use {
    super::{
        HttpClient, HttpError, HttpRequest, HttpResponse, StreamResponse,
        parse_retry_after,
    },
    futures::StreamExt,
    std::time::Duration,
};

/// reqwest: native-tls on iOS, rustls on Android (crate features).
pub struct ReqwestClient {
    client: reqwest::Client,
}

impl Default for ReqwestClient {
    fn default() -> Self {
        Self {
            client: reqwest::Client::builder().build().expect("reqwest client"),
        }
    }
}

impl ReqwestClient {
    fn build(&self, req: &HttpRequest) -> reqwest::RequestBuilder {
        let method =
            reqwest::Method::from_bytes(req.method.as_bytes()).expect("method");
        let mut b = self.client.request(method, &req.url);
        for (k, v) in &req.headers {
            b = b.header(k, v);
        }
        if let Some(body) = &req.body {
            b = b.header("content-type", "application/json").body(body.clone());
        }
        if let Some(t) = req.timeout_ms {
            b = b.timeout(Duration::from_millis(t));
        }
        b
    }
}

fn retry_after(h: &reqwest::header::HeaderMap) -> Option<u64> {
    parse_retry_after(h.get("retry-after").and_then(|v| v.to_str().ok()))
}

#[async_trait::async_trait]
impl HttpClient for ReqwestClient {
    async fn request(
        &self,
        req: HttpRequest,
    ) -> Result<HttpResponse, HttpError> {
        let res = self
            .build(&req)
            .send()
            .await
            .map_err(|e| HttpError::Network(e.to_string()))?;
        let status = res.status().as_u16();
        let retry_after_s = retry_after(res.headers());
        let body = res
            .bytes()
            .await
            .map_err(|e| HttpError::Network(e.to_string()))?
            .to_vec();
        Ok(HttpResponse { status, retry_after_s, body })
    }

    async fn stream(
        &self,
        req: HttpRequest,
    ) -> Result<StreamResponse, HttpError> {
        let res = self
            .build(&req)
            .send()
            .await
            .map_err(|e| HttpError::Network(e.to_string()))?;
        let status = res.status().as_u16();
        let retry_after_s = retry_after(res.headers());
        let body = res.bytes_stream().map(|c| {
            c.map(|b| b.to_vec()).map_err(|e| HttpError::Network(e.to_string()))
        });
        Ok(StreamResponse { status, retry_after_s, body: Box::pin(body) })
    }
}
