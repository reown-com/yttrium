#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();
#[cfg(feature = "uniffi")]
pub mod uniffi_compat;

pub mod account_client;
#[cfg(not(target_arch = "wasm32"))]
pub mod bundler;
pub mod chain;
pub mod chain_abstraction;
pub mod config;
pub mod eip7702;
pub mod entry_point;
pub mod erc20;
// pub mod erc6492_client; // disabled while we resolve Swift build issues
pub mod erc7579;
pub mod error;
pub mod jsonrpc;
pub mod private_key_service;
pub mod sign_service;
pub mod signer;
pub mod smart_accounts;
pub mod test_helpers;
pub mod transaction;
pub mod user_operation;

#[cfg(test)]
pub mod examples;
