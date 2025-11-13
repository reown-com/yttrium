#[cfg(target_arch = "wasm32")]
pub mod create;
pub mod validate;

pub const VERIFY_SERVER_URL: &str = "https://verify.walletconnect.org";
