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
