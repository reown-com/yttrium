uniffi::setup_scaffolding!();

// Force import of this crate to ensure that the code is actually generated
#[allow(unused_imports)]
#[allow(clippy::single_component_path_imports)]
use yttrium;
#[cfg(feature = "account_client")]
use {
    alloy::sol_types::SolStruct,
    yttrium::account_client::AccountClient as YAccountClient,
    yttrium::{
        call::send::safe_test::{
            self, DoSendTransactionParams, OwnerSignature,
            PreparedSendTransaction,
        },
        config::Config,
        smart_accounts::account_address::AccountAddress as FfiAccountAddress,
        smart_accounts::safe::{SignOutputEnum, SignStep3Params},
    },
};
use {
    alloy::{
        hex,
        primitives::{
            ruint::aliases::U256, Address as FFIAddress, Bytes as FFIBytes,
            PrimitiveSignature as FFIPrimitiveSignature, Uint, U128 as FFIU128,
            U256 as FFIU256, U64 as FFIU64,
        },
    },
    yttrium::chain_abstraction::client::ExecuteError,
};
#[cfg(feature = "chain_abstraction_client")]
use {
    alloy::{
        network::Ethereum,
        providers::{Provider, ReqwestProvider},
        sol,
        sol_types::SolCall,
    },
    relay_rpc::domain::ProjectId,
    std::time::Duration,
    yttrium::call::Call,
    yttrium::chain_abstraction::client::ExecuteDetails,
    yttrium::chain_abstraction::{
        api::{
            prepare::{PrepareResponse, PrepareResponseAvailable},
            status::{StatusResponse, StatusResponseCompleted},
        },
        client::Client,
        currency::Currency,
        ui_fields::UiFields,
    },
};
// extern crate yttrium; // This might work too, but I haven't tested

sol! {
    pragma solidity ^0.8.0;
    function transfer(address recipient, uint256 amount) external returns (bool);
}

uniffi::custom_type!(FFIAddress, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| obj.to_string(),
});

uniffi::custom_type!(FFIPrimitiveSignature, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| format!("0x{}", hex::encode(obj.as_bytes())),
});

fn uint_to_hex<const BITS: usize, const LIMBS: usize>(
    obj: Uint<BITS, LIMBS>,
) -> String {
    format!("0x{obj:x}")
}

uniffi::custom_type!(FFIU64, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| uint_to_hex(obj),
});

uniffi::custom_type!(FFIU128, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| uint_to_hex(obj),
});

uniffi::custom_type!(FFIU256, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| uint_to_hex(obj),
});

uniffi::custom_type!(FFIBytes, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| obj.to_string(),
});

#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Record)]
pub struct Eip1559Estimation {
    /// The base fee per gas.
    pub max_fee_per_gas: FFIU128,
    /// The max priority fee per gas.
    pub max_priority_fee_per_gas: FFIU128,
}

impl From<alloy::providers::utils::Eip1559Estimation> for Eip1559Estimation {
    fn from(source: alloy::providers::utils::Eip1559Estimation) -> Self {
        Self {
            max_fee_per_gas: FFIU128::from(source.max_fee_per_gas),
            max_priority_fee_per_gas: FFIU128::from(
                source.max_priority_fee_per_gas,
            ),
        }
    }
}

#[derive(uniffi::Record)]
pub struct FfiPreparedSignature {
    pub message_hash: String,
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum FFIError {
    #[error("General {0}")]
    General(String),
}

#[cfg(feature = "account_client")]
#[derive(uniffi::Object)]
pub struct FFIAccountClient {
    pub owner_address: FfiAccountAddress,
    pub chain_id: u64,
    account_client: YAccountClient,
}

#[cfg(feature = "chain_abstraction_client")]
#[derive(uniffi::Object)]
pub struct ChainAbstractionClient {
    pub project_id: String,
    client: Client,
}

#[cfg(feature = "chain_abstraction_client")]
#[uniffi::export(async_runtime = "tokio")]
impl ChainAbstractionClient {
    #[uniffi::constructor]
    pub fn new(project_id: String) -> Self {
        let client = Client::new(ProjectId::from(project_id.clone()));
        Self { project_id, client }
    }

    pub async fn prepare(
        &self,
        chain_id: String,
        from: FFIAddress,
        call: Call,
    ) -> Result<PrepareResponse, FFIError> {
        self.client
            .prepare(chain_id, from, call)
            .await
            .map_err(|e| FFIError::General(e.to_string()))
    }

    pub async fn get_ui_fields(
        &self,
        route_response: PrepareResponseAvailable,
        currency: Currency,
    ) -> Result<UiFields, FFIError> {
        self.client
            .get_ui_fields(route_response, currency)
            .await
            .map_err(|e| FFIError::General(e.to_string()))
    }

    pub async fn status(
        &self,
        orchestration_id: String,
    ) -> Result<StatusResponse, FFIError> {
        self.client
            .status(orchestration_id)
            .await
            .map_err(|e| FFIError::General(e.to_string()))
    }

    pub async fn wait_for_success_with_timeout(
        &self,
        orchestration_id: String,
        check_in: u64,
        timeout: u64,
    ) -> Result<StatusResponseCompleted, FFIError> {
        self.client
            .wait_for_success_with_timeout(
                orchestration_id,
                Duration::from_secs(check_in),
                Duration::from_secs(timeout),
            )
            .await
            .map_err(|e| FFIError::General(e.to_string()))
    }

    pub async fn execute(
        &self,
        ui_fields: UiFields,
        route_txn_sigs: Vec<FFIPrimitiveSignature>,
        initial_txn_sig: FFIPrimitiveSignature,
    ) -> Result<ExecuteDetails, ExecuteError> {
        self.client.execute(ui_fields, route_txn_sigs, initial_txn_sig).await
    }

    pub async fn estimate_fees(
        &self,
        chain_id: String,
    ) -> Result<Eip1559Estimation, FFIError> {
        let url = format!(
            "https://rpc.walletconnect.com/v1?chainId={chain_id}&projectId={}",
            self.project_id
        )
        .parse()
        .expect("Invalid RPC URL");
        let provider = ReqwestProvider::<Ethereum>::new_http(url);
        provider
            .estimate_eip1559_fees(None)
            .await
            .map(Into::into)
            .map_err(|e| FFIError::General(e.to_string()))
    }

    pub fn prepare_erc20_transfer_call(
        &self,
        erc20_address: FFIAddress,
        to: FFIAddress,
        amount: U256,
    ) -> Call {
        let encoded_data = transferCall::new((to, amount)).abi_encode();

        Call {
            to: erc20_address,
            value: U256::ZERO,
            input: encoded_data.into(),
        }
    }

    pub async fn erc20_token_balance(
        &self,
        chain_id: &str,
        token: FFIAddress,
        owner: FFIAddress,
    ) -> Result<FFIU256, FFIError> {
        self.client
            .erc20_token_balance(chain_id, token, owner)
            .await
            .map_err(|e| FFIError::General(e.to_string()))
    }
}

#[cfg(feature = "account_client")]
#[uniffi::export(async_runtime = "tokio")]
impl FFIAccountClient {
    #[uniffi::constructor]
    pub fn new(
        owner: FfiAccountAddress,
        chain_id: u64,
        config: Config,
    ) -> Self {
        let account_client = YAccountClient::new(owner, chain_id, config);
        Self { owner_address: owner, chain_id, account_client }
    }

    pub fn chain_id(&self) -> u64 {
        self.chain_id
    }

    pub async fn get_address(&self) -> Result<String, FFIError> {
        self.account_client
            .get_address()
            .await
            .map(|address| address.to_string())
            .map_err(|e| FFIError::General(e.to_string()))
    }

    pub fn prepare_sign_message(
        &self,
        message_hash: String,
    ) -> FfiPreparedSignature {
        let res = self
            .account_client
            .prepare_sign_message(message_hash.parse().unwrap());
        let hash = res.safe_message.eip712_signing_hash(&res.domain);
        FfiPreparedSignature { message_hash: hash.to_string() }
    }

    pub async fn do_sign_message(
        &self,
        signatures: Vec<safe_test::OwnerSignature>,
    ) -> Result<SignOutputEnum, FFIError> {
        self.account_client
            .do_sign_message(signatures)
            .await
            .map_err(|e| FFIError::General(e.to_string()))
    }

    pub async fn finalize_sign_message(
        &self,
        signatures: Vec<safe_test::OwnerSignature>,
        sign_step_3_params: SignStep3Params,
    ) -> Result<FFIBytes, FFIError> {
        self.account_client
            .finalize_sign_message(signatures, sign_step_3_params)
            .await
            .map_err(|e| FFIError::General(e.to_string()))
    }

    pub async fn prepare_send_transactions(
        &self,
        transactions: Vec<Call>,
    ) -> Result<PreparedSendTransaction, FFIError> {
        self.account_client
            .prepare_send_transactions(transactions)
            .await
            .map_err(|e| FFIError::General(e.to_string()))
    }

    pub async fn do_send_transactions(
        &self,
        signatures: Vec<OwnerSignature>,
        do_send_transaction_params: DoSendTransactionParams,
    ) -> Result<String, FFIError> {
        Ok(self
            .account_client
            .do_send_transactions(signatures, do_send_transaction_params)
            .await
            .map_err(|e| FFIError::General(e.to_string()))?
            .to_string())
    }

    pub async fn wait_for_user_operation_receipt(
        &self,
        user_operation_hash: String,
    ) -> Result<String, FFIError> {
        self.account_client
            .wait_for_user_operation_receipt(
                user_operation_hash.parse().map_err(|e| {
                    FFIError::General(format!(
                        "Parsing user_operation_hash: {e}"
                    ))
                })?,
            )
            .await
            .iter()
            .map(serde_json::to_string)
            .collect::<Result<String, serde_json::Error>>()
            .map_err(|e| FFIError::General(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        alloy::{
            network::Ethereum,
            primitives::{address, bytes},
            providers::{Provider, ReqwestProvider},
        },
    };

    #[tokio::test]
    #[ignore = "run manually"]
    async fn estimate_fees() {
        let chain_id = "eip155:42161";
        let project_id = std::env::var("REOWN_PROJECT_ID").unwrap();
        let url = format!(
            "https://rpc.walletconnect.com/v1?chainId={chain_id}&projectId={project_id}")
        .parse()
        .expect("Invalid RPC URL");
        let provider = ReqwestProvider::<Ethereum>::new_http(url);

        let estimate = provider.estimate_eip1559_fees(None).await.unwrap();

        println!("estimate: {estimate:?}");
    }

    #[test]
    fn test_address_lower() {
        let ffi_u64 = address!("abababababababababababababababababababab");
        let u = ::uniffi::FfiConverter::<crate::UniFfiTag>::lower(ffi_u64);
        let s: String =
            ::uniffi::FfiConverter::<crate::UniFfiTag>::try_lift(u).unwrap();
        assert_eq!(s, format!("0xABaBaBaBABabABabAbAbABAbABabababaBaBABaB"));
    }

    #[test]
    fn test_u64_lower() {
        let num = 1234567890;
        let ffi_u64 = FFIU64::from(num);
        let u = ::uniffi::FfiConverter::<crate::UniFfiTag>::lower(ffi_u64);
        let s: String =
            ::uniffi::FfiConverter::<crate::UniFfiTag>::try_lift(u).unwrap();
        assert_eq!(s, format!("0x{num:x}"));
    }

    #[test]
    fn test_u128_lower() {
        let num = 1234567890;
        let ffi_u64 = FFIU128::from(num);
        let u = ::uniffi::FfiConverter::<crate::UniFfiTag>::lower(ffi_u64);
        let s: String =
            ::uniffi::FfiConverter::<crate::UniFfiTag>::try_lift(u).unwrap();
        assert_eq!(s, format!("0x{num:x}"));
    }

    #[test]
    fn test_u256_lower() {
        let num = 1234567890;
        let ffi_u64 = FFIU256::from(num);
        let u = ::uniffi::FfiConverter::<crate::UniFfiTag>::lower(ffi_u64);
        let s: String =
            ::uniffi::FfiConverter::<crate::UniFfiTag>::try_lift(u).unwrap();
        assert_eq!(s, format!("0x{num:x}"));
    }

    #[test]
    fn test_bytes_lower() {
        let ffi_u64 = bytes!("aabbccdd");
        let u = ::uniffi::FfiConverter::<crate::UniFfiTag>::lower(ffi_u64);
        let s: String =
            ::uniffi::FfiConverter::<crate::UniFfiTag>::try_lift(u).unwrap();
        assert_eq!(s, format!("0xaabbccdd"));
    }
}
