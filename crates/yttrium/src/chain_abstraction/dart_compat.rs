// use super::error::PrepareDetailedResponse;

#[cfg(feature = "chain_abstraction_client")]
use {
    alloy::{
        network::Ethereum,
        providers::{Provider, ReqwestProvider},
        // sol,
        // sol_types::SolCall,
    },
    crate::call::Call,
    crate::chain_abstraction::{
        api::{
            prepare::PrepareResponseAvailable,
            status::{StatusResponse, StatusResponseCompleted},
        },
        client::Client,
        currency::Currency,
        ui_fields::UiFields,
    },
    relay_rpc::domain::ProjectId,
    std::time::Duration,
    std::str::FromStr,
    super::error::PrepareDetailedResponse,
};

// sol! {
//     pragma solidity ^0.8.0;
//     function transfer(address recipient, uint256 amount) external returns (bool);
// }

#[derive(Debug, thiserror::Error)]
pub enum FFIError {
    #[error("General {0}")]
    General(String),
}

// -----------------

#[cfg(feature = "chain_abstraction_client")]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FFICall {
    pub to: String,     // Convert Address to String
    pub value: u128,  // Convert U256 to String
    pub input: Vec<u8>, // Convert Bytes to Vec<u8>
}

// Convert `Call` → `FFICall`
impl From<Call> for FFICall {
    fn from(call: Call) -> Self {
        FFICall {
            to: call.to.to_string(),
            value: call.value.try_into().unwrap(),
            input: call.input.0.to_vec(),
        }
    }
}

// Convert `FFICall` → `Call`
impl TryFrom<FFICall> for Call {
    type Error = FFIError;

    fn try_from(ffi_call: FFICall) -> Result<Self, Self::Error> {
        type Address = alloy::primitives::Address;
        type U256 = alloy::primitives::U256;
        type Bytes = alloy::primitives::Bytes;

        let to = Address::from_str(&ffi_call.to).unwrap();
        let value = U256::from(ffi_call.value);
        let input = Bytes::from(ffi_call.input);

        Ok(Call { to, value, input })
    }
}

// -----------------

#[cfg(feature = "chain_abstraction_client")]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Eip1559Estimation {
    /// The base fee per gas as a String.
    pub max_fee_per_gas: String,
    /// The max priority fee per gas as a String.
    pub max_priority_fee_per_gas: String,
}

impl From<alloy::providers::utils::Eip1559Estimation> for Eip1559Estimation {
    fn from(source: alloy::providers::utils::Eip1559Estimation) -> Self {
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

impl From<crate::chain_abstraction::local_fee_acc::LocalAmountAcc> for LocalAmountAccCompat {
    fn from(original: crate::chain_abstraction::local_fee_acc::LocalAmountAcc) -> Self {
        Self {
            fees: original.get_fees_compat(),
        }
    }
}

// -----------------

#[cfg(feature = "chain_abstraction_client")]
pub struct ChainAbstractionClient {
    project_id: String,
    client: Client,
}

#[cfg(feature = "chain_abstraction_client")]
impl ChainAbstractionClient {
    
    pub fn new(project_id: String) -> Self {
        let client = Client::new(ProjectId::from(project_id.clone()));
        Self { project_id, client }
    }

    pub async fn prepare_detailed(
        &self,
        chain_id: String,
        from: String,
        call: FFICall,
        local_currency: Currency,
    ) -> Result<PrepareDetailedResponse, FFIError> {
        // let ffi_from = Address::try_from(from).map_err(|e| Error::General(e.to_string()))?;
        // let ffi_from = Address::from_str(&from.0).map_err(|e| Error::General(e.to_string()))?;
        type Address = alloy::primitives::Address;
        let ffi_from = Address::from_str(&from).unwrap();
        let ffi_call = Call::try_from(call)?;

        self.client
            .prepare_detailed(chain_id, ffi_from, ffi_call, local_currency)
            .await
            .map_err(|e| FFIError::General(e.to_string()))
    }

    // execute

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
    
    //     Call {
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
    ) -> Result<String, FFIError> {
        type Address = alloy::primitives::Address;
        // let ffi_token = Address::try_from(token).map_err(|e| FFIError::General(e.to_string()))?;
        let ffi_token = Address::from_str(&token).unwrap();
        // let ffi_owner = Address::try_from(owner).map_err(|e| FFIError::General(e.to_string()))?;
        let ffi_owner = Address::from_str(&owner).unwrap();

        self.client
            .erc20_token_balance(chain_id, ffi_token, ffi_owner)
            .await
            .map(|balance| balance.to_string())
            .map_err(|e| FFIError::General(e.to_string()))
    }
}
