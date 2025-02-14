#[cfg(feature = "chain_abstraction_client")]
use {
    alloy::{
        // network::Ethereum,
        primitives::PrimitiveSignature as PrimitiveSignatureOrig,
        providers::{Provider,utils::Eip1559Estimation as Eip1559EstimationOrig},
        // sol,
        // sol_types::SolCall,
    },
    // super::api::prepare::PrepareResponse,
    crate::call::Call as CallOrig,
    crate::chain_abstraction::{
        api::{
            prepare::PrepareResponseAvailable,
            status::{StatusResponse, StatusResponseCompleted},
        },
        client::{Client,ExecuteDetails},
        currency::Currency,
        error::PrepareDetailedResponse,
        local_fee_acc::LocalAmountAcc as LocalAmountAccOrig,
        pulse::PulseMetadata as PulseMetadataOrig,
        ui_fields::UiFields as UiFieldsOrig,
    },
    relay_rpc::domain::ProjectId,
    std::str::FromStr,
    std::time::Duration,
};

// sol! {
//     pragma solidity ^0.8.0;
//     function transfer(address recipient, uint256 amount) external returns (bool);
// }

#[derive(Debug, thiserror::Error)]
pub enum ErrorCompat {
    #[error("General {0}")]
    General(String),
}

#[cfg(feature = "chain_abstraction_client")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PrimitiveSignatureCompat {
    pub y_parity: bool,
    pub r: String,
    pub s: String,
}

impl From<PrimitiveSignatureCompat> for PrimitiveSignatureOrig {

    fn from(ffi_signature: PrimitiveSignatureCompat) -> Self {
        type U256 = alloy::primitives::U256;
        PrimitiveSignatureOrig::new(
            U256::from_str(&ffi_signature.r).unwrap(),
            U256::from_str(&ffi_signature.s).unwrap(),
            ffi_signature.y_parity,
        )
    }
}

// ----------------

#[cfg(feature = "chain_abstraction_client")]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PulseMetadataCompat {
    // web
    pub url: Option<String>,
    // iOS
    pub bundle_id: Option<String>,
    // Android
    // FIXME this param is not used yet
    pub package_name: Option<String>,
    pub sdk_version: String,
    pub sdk_platform: String,
}

impl TryFrom<PulseMetadataCompat> for PulseMetadataOrig {
    type Error = ErrorCompat;

    fn try_from(ffi_pulse_metadata: PulseMetadataCompat) -> Result<Self, Self::Error> {
        let url = ffi_pulse_metadata.url.and_then(|s| url::Url::parse(&s).ok()); // Convert only if parsing succeeds
        let bundle_id = ffi_pulse_metadata.bundle_id.clone();
        let package_name = ffi_pulse_metadata.package_name.clone();
        let sdk_version = ffi_pulse_metadata.sdk_version.clone();
        let sdk_platform = ffi_pulse_metadata.sdk_platform.clone();

        Ok(PulseMetadataOrig { url, bundle_id, package_name, sdk_version, sdk_platform })
    }
}

// -----------------

#[cfg(feature = "chain_abstraction_client")]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CallCompat {
    pub to: String,     // Convert Address to String
    pub value: u128,    // Convert U256 to String
    pub input: Vec<u8>, // Convert Bytes to Vec<u8>
}

// Convert `CallCompat` → `FFICall`
impl From<CallOrig> for CallCompat {
    fn from(call: CallOrig) -> Self {
        CallCompat {
            to: call.to.to_string(),
            value: call.value.try_into().unwrap(),
            input: call.input.0.to_vec(),
        }
    }
}

// Convert `FFICall` → `CallCompat`
impl TryFrom<CallCompat> for CallOrig {
    type Error = ErrorCompat;

    fn try_from(ffi_call: CallCompat) -> Result<Self, Self::Error> {
        type Address = alloy::primitives::Address;
        type U256 = alloy::primitives::U256;
        type Bytes = alloy::primitives::Bytes;

        let to = Address::from_str(&ffi_call.to).unwrap();
        let value = U256::from(ffi_call.value);
        let input = Bytes::from(ffi_call.input);

        Ok(CallOrig { to, value, input })
    }
}

// -----------------

#[cfg(feature = "chain_abstraction_client")]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Eip1559EstimationCompat {
    /// The base fee per gas as a String.
    pub max_fee_per_gas: String,
    /// The max priority fee per gas as a String.
    pub max_priority_fee_per_gas: String,
}

impl From<Eip1559EstimationOrig> for Eip1559EstimationCompat {
    fn from(source: Eip1559EstimationOrig) -> Self {
        Self {
            max_fee_per_gas: source.max_fee_per_gas.to_string(),
            max_priority_fee_per_gas: source
                .max_priority_fee_per_gas
                .to_string(),
        }
    }
}

// -----------------

#[cfg(feature = "chain_abstraction_client")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FeeCompat {
    pub fungible_amount: String,
    pub fungible_decimals: u8,
    pub fungible_price: String,
    pub fungible_price_decimals: u8,
}

#[cfg(feature = "chain_abstraction_client")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LocalAmountAccCompat {
    pub fees: Vec<FeeCompat>,
}

impl From<LocalAmountAccOrig> for LocalAmountAccCompat {
    fn from(original: LocalAmountAccOrig) -> Self {
        Self { fees: original.get_fees_compat() }
    }
}

// -----------------

// #[cfg(feature = "chain_abstraction_client")]
// // #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
// pub struct UiFieldsCompat {
//     // FIXME
//     pub route_response: PrepareResponseAvailable,
//     pub route: Vec<crate::chain_abstraction::ui_fields::TxnDetails>,
//     pub local_route_total: crate::chain_abstraction::amount::Amount,
//     pub bridge: Vec<crate::chain_abstraction::ui_fields::TransactionFee>,
//     pub local_bridge_total: crate::chain_abstraction::amount::Amount,
//     pub initial: crate::chain_abstraction::ui_fields::TxnDetails,
//     pub local_total: crate::chain_abstraction::amount::Amount,
// }

// impl From<UiFieldsOrig> for UiFieldsCompat {
//     // FIXME
//     fn from(original: UiFieldsOrig) -> Self {
//         Self {
//             route_response: original.route_response,
//             route: original.route,
//             local_route_total: original.local_route_total,
//             bridge: original.bridge,
//             local_bridge_total: original.local_bridge_total,
//             initial: original.initial,
//             local_total: original.local_total,
//         }
//     }
// }

// impl TryFrom<UiFieldsCompat> for UiFieldsOrig {
//     type Error = ErrorCompat;

//     fn try_from(ffi_ui_fields: UiFieldsCompat) -> Result<Self, Self::Error> {
//         Ok(UiFieldsOrig {
//             // FIXME
//             route_response: ffi_ui_fields.route_response,
//             route: ffi_ui_fields.route,
//             local_route_total: ffi_ui_fields.local_route_total,
//             bridge: ffi_ui_fields.bridge,
//             local_bridge_total: ffi_ui_fields.local_bridge_total,
//             initial: ffi_ui_fields.initial,
//             local_total: ffi_ui_fields.local_total,
//         })
//     }
// }

// -----------------

#[cfg(feature = "chain_abstraction_client")]
pub struct ChainAbstractionClient {
    // project_id: String,
    client: Client,
}

#[cfg(feature = "chain_abstraction_client")]
impl ChainAbstractionClient {

    pub fn new(project_id: String, pulse_metadata: PulseMetadataCompat) -> Self {
        let pulse_metadata_orig = PulseMetadataOrig::try_from(pulse_metadata).unwrap();
        let project_id = ProjectId::from(project_id.clone());
        let client = Client::new(project_id, pulse_metadata_orig);
        Self { client }
    }

    // pub async fn prepare(
    //     &self,
    //     chain_id: String,
    //     from: String,
    //     call: FFICall,
    // ) -> Result<PrepareResponse, ErrorCompat> {
    //     type Address = alloy::primitives::Address;
    //     let ffi_from = Address::from_str(&from).unwrap();
    //     let ffi_call = CallCompat::try_from(call)?;

    //     self.client
    //         .prepare(chain_id, ffi_from, ffi_call)
    //         .await
    //         .map_err(|e| ErrorCompat::General(e.to_string()))
    // }

    // FIXME
    // pub async fn get_ui_fields(
    //     &self,
    //     route_response: PrepareResponseAvailable,
    //     currency: Currency,
    // ) -> Result<UiFieldsCompat, ErrorCompat> {

    //     self.client
    //         .get_ui_fields(route_response, currency)
    //         .await
    //         .map(Into::into)
    //         .map_err(|e| ErrorCompat::General(e.to_string()))
    // }

    pub async fn prepare_detailed(
        &self,
        chain_id: String,
        from: String,
        call: CallCompat,
        local_currency: Currency,
    ) -> Result<PrepareDetailedResponse, ErrorCompat> {
        type Address = alloy::primitives::Address;
        let ffi_from = Address::from_str(&from).unwrap();
        let call_orig = CallOrig::try_from(call)?;

        self.client
            .prepare_detailed(chain_id, ffi_from, call_orig, local_currency)
            .await
            .map_err(|e| ErrorCompat::General(e.to_string()))
    }

    pub async fn status(
        &self,
        orchestration_id: String,
    ) -> Result<StatusResponse, ErrorCompat> {
        self.client
            .status(orchestration_id)
            .await
            .map_err(|e| ErrorCompat::General(e.to_string()))
    }

    pub async fn wait_for_success_with_timeout(
        &self,
        orchestration_id: String,
        check_in: u64,
        timeout: u64,
    ) -> Result<StatusResponseCompleted, ErrorCompat> {
        self.client
            .wait_for_success_with_timeout(
                orchestration_id,
                Duration::from_secs(check_in),
                Duration::from_secs(timeout),
            )
            .await
            .map_err(|e| ErrorCompat::General(e.to_string()))
    }

    // FIXME
    // pub async fn execute(
    //     &self,
    //     ui_fields: UiFieldsCompat,
    //     route_txn_sigs: Vec<PrimitiveSignatureCompat>,
    //     initial_txn_sig: PrimitiveSignatureCompat,
    // ) -> Result<ExecuteDetails, ErrorCompat> {

    //     let ui_fields_orig = UiFieldsOrig::try_from(ui_fields).unwrap();
    //     let route_txn_sigs_orig = route_txn_sigs
    //     .into_iter()
    //     .map(|sig| PrimitiveSignatureOrig::from(sig))
    //     .collect();

    //     let ffi_initial_txn_sig_orig = PrimitiveSignatureOrig::from(initial_txn_sig);

    //     self.client
    //         .execute(ui_fields_orig, route_txn_sigs_orig, ffi_initial_txn_sig_orig)
    //         .await
    //         // TODO wanted to return ExecuteError directly here, but can't because Swift keeps the UniFFI lifer private to the yttrium crate and not available to kotlin-ffi crate
    //         // This will be fixed when we merge these crates
    //         .map_err(|e| ErrorCompat::General(e.to_string()))
    // }

    pub async fn estimate_fees(
        &self,
        chain_id: String,
    ) -> Result<Eip1559EstimationCompat, ErrorCompat> {

        self.client
            .provider_pool
            .get_provider(&chain_id)
            .await
            .estimate_eip1559_fees(None)
            .await
            .map(Into::into)
            .map_err(|e| ErrorCompat::General(e.to_string()))
    }

    // pub async fn estimate_fees(
    //     &self,
    //     chain_id: String,
    // ) -> Result<Eip1559EstimationCompat, ErrorCompat> {
    //     let url = format!("https://rpc.walletconnect.org/v1?chainId={chain_id}&projectId={}", self.project_id)
    //     .parse()
    //     .expect("Invalid RPC URL");
    
    //     let provider = RootProvider::<Ethereum>::new_http(url);
    //     provider
    //         .estimate_eip1559_fees(None)
    //         .await
    //         .map(Into::into)
    //         .map_err(|e| ErrorCompat::General(e.to_string()))
    // }

    // pub fn prepare_erc20_transfer_call(
    //     &self,
    //     erc20_address: String,
    //     to: String,
    //     amount: u128,
    // ) -> FFICall {
    //     type Address = alloy::primitives::Address;
    //     type U256 = alloy::primitives::U256;
    //     // let ffi_erc20_address = Address::from_str(&erc20_address).unwrap_or_else(|_| Address::ZERO);
    //     let ffi_erc20_address = Address::from_str(&erc20_address).unwrap();
    //     // let ffi_to = Address::from_str(&to).unwrap_or_else(|_| Address::ZERO);
    //     let ffi_to = Address::from_str(&to).unwrap();
    //     let ffi_amount = U256::from(amount);

    //     let encoded_data = transferCall::new((ffi_to, ffi_amount)).abi_encode().into();

    //     CallCompat {
    //         to: ffi_erc20_address,
    //         value: U256::ZERO,
    //         input: encoded_data,
    //     }.into()
    // }

    pub async fn erc20_token_balance(
        &self,
        chain_id: &str,
        token: String,
        owner: String,
    ) -> Result<String, ErrorCompat> {
        type Address = alloy::primitives::Address;
        // let ffi_token = Address::try_from(token).map_err(|e| ErrorCompat::General(e.to_string()))?;
        let ffi_token = Address::from_str(&token).unwrap();
        // let ffi_owner = Address::try_from(owner).map_err(|e| ErrorCompat::General(e.to_string()))?;
        let ffi_owner = Address::from_str(&owner).unwrap();

        self.client
            .erc20_token_balance(chain_id, ffi_token, ffi_owner)
            .await
            .map(|balance| balance.to_string())
            .map_err(|e| ErrorCompat::General(e.to_string()))
    }
}
