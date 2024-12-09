#![allow(dead_code, improper_ctypes, clippy::unnecessary_cast)]

use self::{
    account_client::FFIAccountClient, chain_abstraction_client::FFIChainClient,
    erc6492_client::Erc6492Client,
};

pub mod account_client;
pub mod chain_abstraction_client;
pub mod config;
pub mod erc6492_client;
pub mod error;
pub mod log;

#[allow(non_camel_case_types)]
#[swift_bridge::bridge]
mod ffi {

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[swift_bridge(swift_repr = "struct")]
    pub struct FFIEip1559Estimation {
        pub maxFeePerGas: String,
        pub maxPriorityFeePerGas: String,
    }

    pub enum FFIStatusResponse {
        Pending(String),   // JSON string of StatusResponsePending
        Completed(String), // JSON string of StatusResponseCompleted
        Error(String),     // JSON string of StatusResponseError
    }

    pub enum FFIRouteResponseSuccess {
        Available(String),   // JSON string of RouteResponseAvailable
        NotRequired(String), // JSON string of RouteResponseNotRequired
    }

    pub enum FFIRouteResponse {
        Success(FFIRouteResponseSuccess),
        Error(String), // JSON string of RouteResponseError
    }

    pub enum FFIRouteError {
        Request(String),
        RequestFailed(String),
        DecodingText(String),
        DecodingJson(String, String),
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[swift_bridge(swift_repr = "struct")]
    pub struct FFIEthTransaction {
        pub from: String,
        pub to: String,
        pub value: String,
        pub gas: String,
        pub gas_price: String,
        pub data: String,
        pub nonce: String,
        pub max_fee_per_gas: String,
        pub max_priority_fee_per_gas: String,
        pub chain_id: String,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[swift_bridge(swift_repr = "struct")]
    pub struct FFITransaction {
        pub _to: String,
        pub _value: String,
        pub _data: String,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[swift_bridge(swift_repr = "struct")]
    pub struct FFIPreparedSign {
        pub signature: String,
        pub hash: String,
        pub sign_step_3_params: String,
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
        ) -> Result<FFIPreparedSign, FFIError>;

        pub async fn finalize_sign_message(
            &self,
            signatures: Vec<String>,
            sign_step_3_params: String,
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

    extern "Rust" {
        type FFIChainClient;

        #[swift_bridge(init)]
        fn new(project_id: String) -> FFIChainClient;

        pub async fn route(
            &self,
            transaction: FFIEthTransaction,
        ) -> Result<FFIRouteResponse, FFIRouteError>;

        pub async fn status(
            &self,
            orchestration_id: String,
        ) -> Result<FFIStatusResponse, FFIRouteError>;

        pub async fn estimate_fees(
            &self,
            chain_id: String,
        ) -> Result<FFIEip1559Estimation, FFIError>;
    }
}
