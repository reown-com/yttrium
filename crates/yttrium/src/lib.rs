#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();
#[cfg(feature = "uniffi")]
pub mod uniffi_compat;

pub mod account_client;
pub mod blockchain_api;
#[cfg(not(target_arch = "wasm32"))]
pub mod bundler;
pub mod call;
pub mod chain;
pub mod chain_abstraction;
pub mod config;
pub mod eip7702;
pub mod entry_point;
pub mod erc20;
pub mod erc6492_client;
pub mod erc7579;
pub mod error;
pub mod gas_abstraction;
pub mod jsonrpc;
pub mod provider_pool;
pub mod smart_accounts;
pub mod test_helpers;
pub mod user_operation;
pub mod utils;

#[cfg(test)]
pub mod examples;
