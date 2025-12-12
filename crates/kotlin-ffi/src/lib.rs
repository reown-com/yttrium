uniffi::setup_scaffolding!();

// Force import of this crate to ensure that the code is actually generated
#[cfg(any(
    feature = "chain_abstraction_client",
    feature = "account_client"
))]
use alloy::primitives::Bytes as FFIBytes;
#[allow(unused_imports)]
#[allow(clippy::single_component_path_imports)]
use yttrium;
#[cfg(feature = "pay")]
use pay_api;
#[cfg(any(
    feature = "chain_abstraction_client",
    feature = "account_client"
))]
use yttrium::call::Call;
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
#[cfg(feature = "chain_abstraction_client")]
use {
    alloy::{
        hex,
        primitives::{
            ruint::aliases::U256, Address as FFIAddress,
            Signature as FFIPrimitiveSignature, Uint, U128 as FFIU128,
            U256 as FFIU256, U64 as FFIU64,
        },
        sol,
    },
    yttrium::{
        chain_abstraction::{
            error::PrepareDetailedResponse, ui_fields::RouteSig,
        },
        pulse::PulseMetadata,
        wallet_service_api::{GetAssetsParams, GetAssetsResult},
    },
};
#[cfg(feature = "chain_abstraction_client")]
use {
    alloy::{providers::Provider, sol_types::SolCall},
    relay_rpc::domain::ProjectId,
    std::time::Duration,
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

#[cfg(feature = "chain_abstraction_client")]
sol! {
    pragma solidity ^0.8.0;
    function transfer(address recipient, uint256 amount) external returns (bool);
}

#[cfg(feature = "chain_abstraction_client")]
uniffi::custom_type!(FFIAddress, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| obj.to_string(),
});

#[cfg(feature = "chain_abstraction_client")]
uniffi::custom_type!(FFIPrimitiveSignature, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| format!("0x{}", hex::encode(obj.as_bytes())),
});

#[cfg(feature = "chain_abstraction_client")]
fn uint_to_hex<const BITS: usize, const LIMBS: usize>(
    obj: Uint<BITS, LIMBS>,
) -> String {
    format!("0x{obj:x}")
}

#[cfg(feature = "chain_abstraction_client")]
uniffi::custom_type!(FFIU64, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| uint_to_hex(obj),
});

#[cfg(feature = "chain_abstraction_client")]
uniffi::custom_type!(FFIU128, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| uint_to_hex(obj),
});

#[cfg(feature = "chain_abstraction_client")]
uniffi::custom_type!(FFIU256, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| uint_to_hex(obj),
});

#[cfg(any(feature = "chain_abstraction_client", feature = "account_client"))]
uniffi::custom_type!(FFIBytes, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| obj.to_string(),
});

#[cfg(feature = "chain_abstraction_client")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Record)]
pub struct Eip1559Estimation {
    /// The base fee per gas.
    pub max_fee_per_gas: FFIU128,
    /// The max priority fee per gas.
    pub max_priority_fee_per_gas: FFIU128,
}

#[cfg(feature = "chain_abstraction_client")]
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
    #[error("Prepare: {0}")]
    Prepare(String),

    #[error("GetUiFields: {0}")]
    GetUiFields(String),

    #[error("PrepareDetailed: {0}")]
    PrepareDetailed(String),

    #[error("Status: {0}")]
    Status(String),

    #[error("WaitForSuccessWithTimeout: {0}")]
    WaitForSuccessWithTimeout(String),

    #[error("Execute: {0}")]
    Execute(String),

    #[error("EstimateFees: {0}")]
    EstimateFees(String),

    #[error("Erc20TokenBalance: {0}")]
    Erc20TokenBalance(String),

    #[error("GetWalletAssets: {0}")]
    GetWalletAssets(String),
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
    pub fn new(project_id: String, pulse_metadata: PulseMetadata) -> Self {
        let client =
            Client::new(ProjectId::from(project_id.clone()), pulse_metadata);
        Self { project_id, client }
    }

    #[uniffi::constructor]
    pub fn new_with_blockchain_api_url(
        project_id: String,
        pulse_metadata: PulseMetadata,
        blockchain_api_url: String,
    ) -> Self {
        let client = Client::with_blockchain_api_url(
            ProjectId::from(project_id.clone()),
            pulse_metadata,
            blockchain_api_url.parse().unwrap(),
        );
        Self { project_id, client }
    }

    pub async fn prepare(
        &self,
        chain_id: String,
        from: FFIAddress,
        call: Call,
        use_lifi: bool,
    ) -> Result<PrepareResponse, FFIError> {
        self.client
            .prepare(chain_id, from, call, vec![], use_lifi)
            .await
            .map_err(|e| FFIError::Prepare(e.to_string()))
    }

    pub async fn get_ui_fields(
        &self,
        route_response: PrepareResponseAvailable,
        local_currency: Currency,
    ) -> Result<UiFields, FFIError> {
        self.client
            .get_ui_fields(route_response, local_currency)
            .await
            .map_err(|e| FFIError::GetUiFields(e.to_string()))
    }

    pub async fn prepare_detailed(
        &self,
        chain_id: String,
        from: FFIAddress,
        call: Call,
        accounts: Vec<String>,
        local_currency: Currency,
        // TODO use this to e.g. modify priority fee
        // _speed: String,
        use_lifi: bool,
    ) -> Result<PrepareDetailedResponse, FFIError> {
        self.client
            .prepare_detailed(
                chain_id,
                from,
                call,
                accounts,
                local_currency,
                use_lifi,
            )
            .await
            .map_err(|e| FFIError::PrepareDetailed(e.to_string()))
    }

    pub async fn status(
        &self,
        orchestration_id: String,
    ) -> Result<StatusResponse, FFIError> {
        self.client
            .status(orchestration_id)
            .await
            .map_err(|e| FFIError::Status(e.to_string()))
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
            .map_err(|e| FFIError::WaitForSuccessWithTimeout(e.to_string()))
    }

    pub async fn execute(
        &self,
        ui_fields: UiFields,
        route_txn_sigs: Vec<RouteSig>,
        initial_txn_sig: FFIPrimitiveSignature,
    ) -> Result<ExecuteDetails, FFIError> {
        self.client
            .execute(ui_fields, route_txn_sigs, initial_txn_sig)
            .await
            // TODO wanted to return ExecuteError directly here, but can't because Swift keeps the UniFFI lifer private to the yttrium crate and not available to kotlin-ffi crate
            // This will be fixed when we merge these crates
            .map_err(|e| FFIError::Execute(e.to_string()))
    }

    pub async fn estimate_fees(
        &self,
        chain_id: String,
    ) -> Result<Eip1559Estimation, FFIError> {
        self.client
            .provider_pool
            .get_provider(&chain_id)
            .await
            .estimate_eip1559_fees()
            .await
            .map(Into::into)
            .map_err(|e| FFIError::EstimateFees(e.to_string()))
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
            .map_err(|e| FFIError::Erc20TokenBalance(e.to_string()))
    }

    pub async fn get_wallet_assets(
        &self,
        params: GetAssetsParams,
    ) -> Result<GetAssetsResult, FFIError> {
        self.client
            .get_wallet_assets(params)
            .await
            .map_err(|e| FFIError::GetWalletAssets(e.to_string()))
    }
}

// Free-function helpers required by utils bindings; export them from this crate so the
// checksum symbols are present in the final cdylib regardless of upstream crate layout.
#[cfg(feature = "chain_abstraction_client")]
#[uniffi::export]
fn funding_metadata_to_amount(
    value: yttrium::chain_abstraction::api::prepare::FundingMetadata,
) -> yttrium::chain_abstraction::amount::Amount {
    value.to_amount()
}

#[cfg(feature = "chain_abstraction_client")]
#[uniffi::export]
fn funding_metadata_to_bridging_fee_amount(
    value: yttrium::chain_abstraction::api::prepare::FundingMetadata,
) -> yttrium::chain_abstraction::amount::Amount {
    value.to_bridging_fee_amount()
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
            .map_err(|e| FFIError::Prepare(e.to_string()))
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
            .map_err(|e| FFIError::Prepare(e.to_string()))
    }

    pub async fn finalize_sign_message(
        &self,
        signatures: Vec<safe_test::OwnerSignature>,
        sign_step_3_params: SignStep3Params,
    ) -> Result<FFIBytes, FFIError> {
        self.account_client
            .finalize_sign_message(signatures, sign_step_3_params)
            .await
            .map_err(|e| FFIError::Prepare(e.to_string()))
    }

    pub async fn prepare_send_transactions(
        &self,
        transactions: Vec<Call>,
    ) -> Result<PreparedSendTransaction, FFIError> {
        self.account_client
            .prepare_send_transactions(transactions)
            .await
            .map_err(|e| FFIError::Prepare(e.to_string()))
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
            .map_err(|e| FFIError::Prepare(e.to_string()))?
            .to_string())
    }

    pub async fn wait_for_user_operation_receipt(
        &self,
        user_operation_hash: String,
    ) -> Result<String, FFIError> {
        self.account_client
            .wait_for_user_operation_receipt(
                user_operation_hash.parse().map_err(|e| {
                    FFIError::Prepare(format!(
                        "Parsing user_operation_hash: {e}"
                    ))
                })?,
            )
            .await
            .iter()
            .map(serde_json::to_string)
            .collect::<Result<String, serde_json::Error>>()
            .map_err(|e| FFIError::Prepare(e.to_string()))
    }
}

#[cfg(test)]
#[cfg(feature = "chain_abstraction_client")]
mod tests {
    use {
        super::*,
        alloy::{
            primitives::{address, bytes},
            providers::{Provider, ProviderBuilder},
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
        let provider = ProviderBuilder::new().connect_http(url);

        let estimate = provider.estimate_eip1559_fees().await.unwrap();

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
