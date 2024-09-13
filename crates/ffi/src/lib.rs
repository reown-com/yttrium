use self::account_client::FFIAccountClient;
use self::account_client_eip7702::FFI7702AccountClient;
use swift_bridge;

pub mod account_client;
pub mod account_client_eip7702;
pub mod config;
pub mod error;
pub mod log;

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

        pub async fn send_transaction(
            &self,
            _transaction: FFITransaction,
        ) -> Result<String, FFIError>;

        pub fn sign_message_with_mnemonic(
            &self,
            message: String,
            mnemonic: String,
        ) -> Result<String, FFIError>;

        pub async fn wait_for_user_operation_receipt(
            &self,
            user_operation_hash: String
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
        type NativeSignerFFI;

        #[swift_bridge(init)]
        fn new(signer_id: String) -> NativeSignerFFI;

        fn sign(&self, message: String) -> FFIStringResult;
    }

    extern "Swift" {
        type PrivateKeySignerFFI;

        #[swift_bridge(init)]
        fn new(signer_id: String) -> PrivateKeySignerFFI;

        fn private_key(&self) -> FFIStringResult;
    }
}
