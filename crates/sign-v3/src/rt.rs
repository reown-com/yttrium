//! The runtime differences between native (tokio) and WASM (the browser event loop).

use std::future::Future;

#[cfg(not(target_arch = "wasm32"))]
pub trait MaybeSend: Send {}
#[cfg(not(target_arch = "wasm32"))]
impl<T: Send> MaybeSend for T {}
#[cfg(target_arch = "wasm32")]
pub trait MaybeSend {}
#[cfg(target_arch = "wasm32")]
impl<T> MaybeSend for T {}

#[cfg(not(target_arch = "wasm32"))]
pub fn spawn<F: Future<Output = ()> + Send + 'static>(f: F) {
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        handle.spawn(f);
    } else {
        std::thread::spawn(move || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("tokio runtime")
                .block_on(f)
        });
    }
}

#[cfg(target_arch = "wasm32")]
pub fn spawn<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}

pub async fn sleep_ms(ms: u64) {
    #[cfg(not(target_arch = "wasm32"))]
    tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        let p = js_sys::Promise::new(&mut |resolve, _| {
            let g = js_sys::global();
            if let Ok(f) = js_sys::Reflect::get(&g, &"setTimeout".into()) {
                let f: js_sys::Function = f.unchecked_into();
                let _ = f.call2(&g, &resolve, &(ms as f64).into());
            }
        });
        let _ = wasm_bindgen_futures::JsFuture::from(p).await;
    }
}

pub fn now_ms() -> u64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_millis() as u64
    }
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now() as u64
    }
}

/// Jitter in [0.5, 1.0) for backoff.
pub fn jitter() -> f64 {
    let b: [u8; 2] = crate::protocol::crypto::random_bytes();
    0.5 + (u16::from_le_bytes(b) as f64 / 65536.0) / 2.0
}

/// Boxed futures at dispatch boundaries keep each async state machine out of its caller's code.
#[cfg(not(target_arch = "wasm32"))]
pub type BoxFut<'a, T> = std::pin::Pin<Box<dyn Future<Output = T> + Send + 'a>>;
#[cfg(target_arch = "wasm32")]
pub type BoxFut<'a, T> = std::pin::Pin<Box<dyn Future<Output = T> + 'a>>;

pub fn boxed<'a, T>(
    f: impl Future<Output = T> + MaybeSend + 'a,
) -> BoxFut<'a, T> {
    Box::pin(f)
}
