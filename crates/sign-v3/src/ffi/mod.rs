//! Bindings: the JSON facade (`init`, `call`, `subscribe`) shared by every binding, uniffi for
//! Swift and Kotlin, wasm-bindgen for the web.

pub mod json;
#[cfg(feature = "uniffi")]
pub mod uniffi_api;
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub mod wasm;
