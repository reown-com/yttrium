#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();
#[cfg(feature = "uniffi")]
pub mod uniffi_compat;

#[cfg(feature = "wasm")]
pub mod wasm_compat;

#[cfg(feature = "account_client")]
pub mod account_client;
pub mod blockchain_api;
pub mod bundler;
pub mod call;
pub mod chain;
#[cfg(feature = "chain_abstraction_client")]
pub mod chain_abstraction;
pub mod config;
pub mod eip7702;
pub mod entry_point;
pub mod erc20;
#[cfg(feature = "erc6492_client")]
pub mod erc6492_client;
pub mod erc7579;
pub mod error;
pub mod jsonrpc;
pub mod provider_pool;
pub mod serde;
pub mod smart_accounts;
pub mod test_helpers;
pub mod time;
#[cfg(feature = "transaction_sponsorship_client")]
pub mod transaction_sponsorship;
pub mod user_operation;
pub mod utils;

#[cfg(test)]
pub mod examples;
