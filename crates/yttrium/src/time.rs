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

// sleep() returns a future that can only be awaited once. If you use select! it will drop the sleep and cannot be used again
// This function transforms the sleep into a receiver that can be awaited multiple times
// TODO consider if we need this, can we use `&mut sleep` instead? Seems non-WASM requires pinning it. https://reown-inc.slack.com/archives/C06J58UUSTW/p1755699372241649
pub type DurableSleep = tokio::sync::mpsc::Receiver<()>;
pub fn durable_sleep(duration: Duration) -> DurableSleep {
    let (tx, rx) = tokio::sync::mpsc::channel(1);
    crate::spawn::spawn(async move {
        crate::time::sleep(duration).await;
        let _ = tx.send(()).await.ok();
    });
    rx
}
