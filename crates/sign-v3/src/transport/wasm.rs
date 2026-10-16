use {
    super::{
        HttpClient, HttpError, HttpRequest, HttpResponse, StreamResponse,
        parse_retry_after,
    },
    futures::StreamExt,
    js_sys::{Reflect, Uint8Array},
    wasm_bindgen::{JsCast, JsValue},
    wasm_bindgen_futures::JsFuture,
    web_sys::{AbortController, Headers, Request, RequestInit, Response},
};

/// The browser's `fetch`, streaming the poll body through its `ReadableStream`.
#[derive(Default)]
pub struct FetchClient;

fn net(e: JsValue) -> HttpError {
    HttpError::Network(format!("{e:?}"))
}

async fn fetch(
    req: &HttpRequest,
    abort: &AbortController,
) -> Result<Response, HttpError> {
    let headers = Headers::new().map_err(net)?;
    for (k, v) in &req.headers {
        headers.set(k, v).map_err(net)?;
    }
    let init = RequestInit::new();
    init.set_method(req.method);
    init.set_signal(Some(&abort.signal()));
    if let Some(body) = &req.body {
        headers.set("content-type", "application/json").map_err(net)?;
        init.set_body(&Uint8Array::from(body.as_slice()).into());
    }
    init.set_headers(&headers.into());
    let request =
        Request::new_with_str_and_init(&req.url, &init).map_err(net)?;
    let global = js_sys::global();
    let fetch_fn: js_sys::Function = Reflect::get(&global, &"fetch".into())
        .map_err(net)?
        .dyn_into()
        .map_err(net)?;
    let promise: js_sys::Promise = fetch_fn
        .call1(&global, &request)
        .map_err(net)?
        .dyn_into()
        .map_err(net)?;
    if let Some(ms) = req.timeout_ms {
        let abort = abort.clone();
        wasm_bindgen_futures::spawn_local(async move {
            crate::rt::sleep_ms(ms).await;
            abort.abort();
        });
    }
    JsFuture::from(promise).await.map_err(net)?.dyn_into().map_err(net)
}

fn retry_after(res: &Response) -> Option<u64> {
    parse_retry_after(
        res.headers().get("retry-after").ok().flatten().as_deref(),
    )
}

/// Aborts the fetch when the poll is dropped, so a kicked poll frees its connection at once.
struct AbortOnDrop(AbortController);
impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        self.0.abort();
    }
}

#[async_trait::async_trait(?Send)]
impl HttpClient for FetchClient {
    async fn request(
        &self,
        req: HttpRequest,
    ) -> Result<HttpResponse, HttpError> {
        let abort = AbortController::new().map_err(net)?;
        let res = fetch(&req, &abort).await?;
        let status = res.status();
        let retry_after_s = retry_after(&res);
        let buf = JsFuture::from(res.array_buffer().map_err(net)?)
            .await
            .map_err(net)?;
        Ok(HttpResponse {
            status,
            retry_after_s,
            body: Uint8Array::new(&buf).to_vec(),
        })
    }

    async fn stream(
        &self,
        req: HttpRequest,
    ) -> Result<StreamResponse, HttpError> {
        let abort = AbortController::new().map_err(net)?;
        let res = fetch(&req, &abort).await?;
        let status = res.status();
        let retry_after_s = retry_after(&res);
        let guard = AbortOnDrop(abort);
        let body = match res.body() {
            Some(b) => {
                wasm_streams::ReadableStream::from_raw(b.unchecked_into())
                    .into_stream()
            }
            None => {
                return Ok(StreamResponse {
                    status,
                    retry_after_s,
                    body: Box::pin(futures::stream::empty()),
                });
            }
        };
        let stream = body.map(move |chunk| {
            let _keep = &guard;
            chunk.map(|v| Uint8Array::new(&v).to_vec()).map_err(net)
        });
        Ok(StreamResponse { status, retry_after_s, body: Box::pin(stream) })
    }
}
