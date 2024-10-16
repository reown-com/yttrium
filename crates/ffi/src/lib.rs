#![allow(dead_code, improper_ctypes, clippy::unnecessary_cast)]
use self::account_client::FFIAccountClient;
use self::account_client_eip7702::FFI7702AccountClient;
use self::erc6492_client::Erc6492Client;

pub mod account_client;
pub mod account_client_eip7702;
pub mod config;
pub mod erc6492_client;
pub mod error;
pub mod log;

#[allow(non_camel_case_types)]
#[swift_bridge::bridge]
mod ffi {
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[swift_bridge(swift_repr = "struct")]
    pub struct FFITransaction {
        pub _to: String,
        pub _value: String,
        pub _data: String,
    }

    #[derive(Debug, Clone)]
    #[swift_bridge(swift_repr = "struct")]
    pub struct FFIEndpoint {
        pub api_key: String,
        pub base_url: String,
    }

    #[derive(Debug, Clone)]
    #[swift_bridge(swift_repr = "struct")]
    pub struct FFIEndpoints {
        pub rpc: FFIEndpoint,
        pub bundler: FFIEndpoint,
        pub paymaster: FFIEndpoint,
    }

    #[derive(Debug, Clone)]
    #[swift_bridge(swift_repr = "struct")]
    pub struct FFIConfig {
        pub endpoints: FFIEndpoints,
    }

    #[derive(Debug, Clone)]
    #[swift_bridge(swift_repr = "struct")]
    pub struct FFIAccountClientConfig {
        pub owner_address: String,
        pub chain_id: u64,
        pub config: FFIConfig,
        pub signer_type: String,
        pub safe: bool,
    }

    #[derive(Debug, Clone)]
    #[swift_bridge(swift_repr = "struct")]
    pub struct FFIPreparedSignature {
        pub hash: String,
    }

    #[derive(Debug, Clone)]
    #[swift_bridge(swift_repr = "struct")]
    pub struct FFIPreparedSendTransaction {
        pub hash: String,
        pub do_send_transaction_params: String,
    }

    #[derive(Debug, Clone)]
    #[swift_bridge(swift_repr = "struct")]
    #[derive(serde::Deserialize)]
    pub struct FFIOwnerSignature {
        pub owner: String,
        pub signature: String,
    }

    enum FFIStringResult {
        Ok(String),
        Err(String),
    }

    enum FFIError {
        Unknown(String),
    }

    extern "Rust" {
        type FFIAccountClient;

        #[swift_bridge(init)]
        fn new(config: FFIAccountClientConfig) -> FFIAccountClient;

        pub fn chain_id(&self) -> u64;

        pub async fn get_address(&self) -> Result<String, FFIError>;

        pub async fn prepare_sign_message(
            &self,
            _message_hash: String,
        ) -> Result<FFIPreparedSignature, FFIError>;

        pub async fn do_sign_message(
            &self,
            _signatures: Vec<String>,
        ) -> Result<String, FFIError>;

        pub async fn send_transactions(
            &self,
            _transactions: Vec<String>,
        ) -> Result<String, FFIError>;

        pub async fn prepare_send_transactions(
            &self,
            _transactions: Vec<String>,
        ) -> Result<FFIPreparedSendTransaction, FFIError>;

        pub async fn do_send_transaction(
            &self,
            _signatures: Vec<String>,
            _do_send_transaction_params: String,
        ) -> Result<String, FFIError>;

        pub fn sign_message_with_mnemonic(
            &self,
            message: String,
            mnemonic: String,
        ) -> Result<String, FFIError>;

        pub async fn wait_for_user_operation_receipt(
            &self,
            user_operation_hash: String,
        ) -> Result<String, FFIError>;
    }

    extern "Rust" {
        type FFI7702AccountClient;

        #[swift_bridge(init)]
        fn new(config: FFIAccountClientConfig) -> FFI7702AccountClient;

        pub async fn send_batch_transaction(
            &self,
            batch: String,
        ) -> Result<String, FFIError>;
    }

    extern "Swift" {
        pub type NativeSignerFFI;

        #[swift_bridge(init)]
        pub fn new(signer_id: String) -> NativeSignerFFI;

        pub fn sign(&self, message: String) -> FFIStringResult;
    }

    extern "Swift" {
        pub type PrivateKeySignerFFI;

        #[swift_bridge(init)]
        pub fn new(signer_id: String) -> PrivateKeySignerFFI;

        pub fn private_key(&self) -> FFIStringResult;
    }

    enum Erc6492Error {
        InvalidSignature(String),
        InvalidAddress(String),
        InvalidMessageHash(String),
        Verification(String),
    }

    extern "Rust" {
        type Erc6492Client;

        #[swift_bridge(init)]
        fn new(rpc_url: String) -> Erc6492Client;

        pub async fn verify_signature(
            &self,
            signature: String,
            address: String,
            message_hash: String,
        ) -> Result<bool, Erc6492Error>;
    }
}
