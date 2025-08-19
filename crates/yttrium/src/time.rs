pub use std::time::Duration;
#[cfg(target_arch = "wasm32")]
pub use wasmtimer::{
    std::{Instant, SystemTime, UNIX_EPOCH},
    tokio::{sleep, timeout, Sleep},
};
#[cfg(not(target_arch = "wasm32"))]
pub use {
    std::time::{Instant, SystemTime, UNIX_EPOCH},
    tokio::time::{sleep, timeout, Sleep},
};

pub fn durable_sleep(duration: Duration) -> tokio::sync::mpsc::Receiver<()> {
    let (tx, rx) = tokio::sync::mpsc::channel(1);
    crate::spawn::spawn(async move {
        crate::time::sleep(duration).await;
        let _ = tx.send(()).await.ok();
    });
    rx
}
