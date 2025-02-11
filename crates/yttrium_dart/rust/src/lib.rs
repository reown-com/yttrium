mod frb_generated; /* AUTO INJECTED BY flutter_rust_bridge. This line may not be accurate, and you can change it according to your needs. */

use alloy::primitives::{
    ruint::aliases::U256, Address as FFIAddress, Bytes as FFIBytes, Uint,
    U128 as FFIU128, U256 as FFIU256, U64 as FFIU64,
};
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
        smart_accounts::account_address::AccountAddress as YAccountAddress,
        smart_accounts::safe::{SignOutputEnum, SignStep3Params},
    },
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
use std::str::FromStr;
// extern crate yttrium; // This might work too, but I haven't tested

sol! {
    pragma solidity ^0.8.0;
    function transfer(address recipient, uint256 amount) external returns (bool);
}

// #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
// pub struct FFIAddress(pub String);
// // Convert `Address` -> `FFIAddress` (String)
// impl From<Address> for FFIAddress {
//     fn from(addr: Address) -> Self {
//         FFIAddress(addr.to_string()) // Convert to hex string
//     }
// }
// // Convert `FFIAddress` (String) -> `Address`
// impl TryFrom<FFIAddress> for Address {
//     type Error = <Address as FromStr>::Err; // Correct error type

//     fn try_from(ffi_addr: FFIAddress) -> Result<Self, Self::Error> {
//         Address::from_str(&ffi_addr.0)
//     }
// }
// fn uint_to_hex<const BITS: usize, const LIMBS: usize>(
//     obj: Uint<BITS, LIMBS>,
// ) -> String {
//     format!("0x{obj:x}")
// }

// #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
// pub struct FFIU64(pub String);
// // Convert `U64` → `FFIU64`
// impl From<U64> for FFIU64 {
//     fn from(value: U64) -> Self {
//         FFIU64(value.to_string())
//     }
// }
// // Convert `FFIU64` → `U64`
// impl TryFrom<FFIU64> for U64 {
//     type Error = alloy::primitives::ruint::ParseError;

//     fn try_from(value: FFIU64) -> Result<Self, Self::Error> {
//         U64::from_str(&value.0)
//     }
// }

// #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
// pub struct FFIU128(pub String);
// // Convert `U128` → `FFIU128`
// impl From<U128> for FFIU128 {
//     fn from(value: U128) -> Self {
//         FFIU128(value.to_string())
//     }
// }
// // Convert `FFIU128` → `U128`
// impl TryFrom<FFIU128> for U128 {
//     type Error = alloy::primitives::ruint::ParseError;

//     fn try_from(value: FFIU128) -> Result<Self, Self::Error> {
//         U128::from_str(&value.0)
//     }
// }

// #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
// pub struct FFIU256(pub String);
// // Convert `U256` → `FFIU256`
// impl From<U256> for FFIU256 {
//     fn from(value: U256) -> Self {
//         FFIU256(value.to_string())
//     }
// }
// // Convert `FFIU256` → `U256`
// impl TryFrom<FFIU256> for U256 {
//     type Error = alloy::primitives::ruint::ParseError;

//     fn try_from(value: FFIU256) -> Result<Self, Self::Error> {
//         U256::from_str(&value.0)
//     }
// }

// uniffi::custom_type!(FFIBytes, String, {
//     try_lift: |val| Ok(val.parse()?),
//     lower: |obj| obj.to_string(),
// });

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
            value: call.value.try_into().unwrap_or(0),
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

        let to = Address::from_str(&ffi_call.to).expect("Failed to parse address");
        let value = U256::from(ffi_call.value);
        let input = Bytes::from(ffi_call.input);

        Ok(Call { to, value, input })
    }
}

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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PreparedSignature {
    pub message_hash: String,
}

#[derive(Debug, thiserror::Error)]
pub enum FFIError {
    #[error("General {0}")]
    General(String),
}

#[cfg(feature = "account_client")]
pub struct FFIAccountClient {
    pub owner_address: YAccountAddress,
    pub chain_id: u64,
    account_client: YAccountClient,
}

#[cfg(feature = "chain_abstraction_client")]
pub struct ChainAbstractionClient {
    pub project_id: String,
    client: Client,
}

#[cfg(feature = "chain_abstraction_client")]
impl ChainAbstractionClient {
    pub fn new(project_id: String) -> Self {
        let client = Client::new(ProjectId::from(project_id.clone()));
        Self { project_id, client }
    }

    pub async fn prepare(
        &self,
        chain_id: String,
        from: String,
        call: FFICall,
    ) -> Result<PrepareResponse, FFIError> {
        // let ffi_from = Address::try_from(from).map_err(|e| Error::General(e.to_string()))?;
        // let ffi_from = Address::from_str(&from.0).map_err(|e| Error::General(e.to_string()))?;
        type Address = alloy::primitives::Address;
        let ffi_from = Address::from_str(&from).expect("Failed to parse address");
        let ffi_call = Call::try_from(call)?;

        self.client
            .prepare(chain_id, ffi_from, ffi_call)
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
        erc20_address: String,
        to: String,
        amount: u128,
    ) -> FFICall {
        type Address = alloy::primitives::Address;
        type U256 = alloy::primitives::U256;
        // let ffi_erc20_address = Address::from_str(&erc20_address).unwrap_or_else(|_| Address::ZERO);
        let ffi_erc20_address = Address::from_str(&erc20_address).expect("Failed to parse address");
        // let ffi_to = Address::from_str(&to).unwrap_or_else(|_| Address::ZERO);
        let ffi_to = Address::from_str(&to).expect("Failed to parse address");
        let ffi_amount = U256::from(amount);
    
        let encoded_data = transferCall::new((ffi_to, ffi_amount)).abi_encode().into();
    
        Call {
            to: ffi_erc20_address,
            value: U256::ZERO,
            input: encoded_data,
        }.into()
    }

    pub async fn erc20_token_balance(
        &self,
        chain_id: &str,
        token: String,
        owner: String,
    ) -> Result<String, FFIError> {
        type Address = alloy::primitives::Address;
        // let ffi_token = Address::try_from(token).map_err(|e| FFIError::General(e.to_string()))?;
        let ffi_token = Address::from_str(&token).expect("Failed to parse address");
        // let ffi_owner = Address::try_from(owner).map_err(|e| FFIError::General(e.to_string()))?;
        let ffi_owner = Address::from_str(&owner).expect("Failed to parse address");

        self.client
            .erc20_token_balance(chain_id, ffi_token, ffi_owner)
            .await
            .map(|balance| balance.to_string())
            .map_err(|e| FFIError::General(e.to_string()))
    }
}

#[cfg(feature = "account_client")]
impl FFIAccountClient {
    pub fn new(owner: YAccountAddress, chain_id: u64, config: Config) -> Self {
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
    ) -> PreparedSignature {
        let res = self
            .account_client
            .prepare_sign_message(message_hash.parse().unwrap());
        let hash = res.safe_message.eip712_signing_hash(&res.domain);
        PreparedSignature { message_hash: hash.to_string() }
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
    ) -> Result<Bytes, FFIError> {
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
                    FFIError::General(format!("Parsing user_operation_hash: {e}"))
                })?,
            )
            .await
            .iter()
            .map(serde_json::to_string)
            .collect::<Result<String, serde_json::FFIError>>()
            .map_err(|e| FFIError::General(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use {
        // super::*,
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
        // Simulate sending the data to Dart (convert U128 values to strings)
        let max_fee_per_gas = estimate.max_fee_per_gas.to_string();
        let max_priority_fee_per_gas =
            estimate.max_priority_fee_per_gas.to_string();

        println!("Max fee per gas: {max_fee_per_gas}, Max priority fee per gas: {max_priority_fee_per_gas}");
    }

    #[test]
    fn test_address_lower() {
        use alloy::hex;

        let addr = address!("abababababababababababababababababababab");

        // Convert address to hex string
        let addr_hex = format!("0x{}", hex::encode(addr.as_slice()));
        assert_eq!(
            addr_hex,
            format!("0xABaBaBaBABabABabAbAbABAbABabababaBaBABaB")
        );
    }

    #[test]
    fn test_u64_lower() {
        let num = 1234567890;

        // Convert number to hex string
        let num_hex = format!("0x{:x}", num);
        assert_eq!(num_hex, "0x499602d2");
    }

    #[test]
    fn test_u128_lower() {
        let num = 1234567890;

        // Convert number to hex string
        let num_hex = format!("0x{:x}", num);
        assert_eq!(num_hex, "0x499602d2");
    }

    #[test]
    fn test_u256_lower() {
        let num = 1234567890;

        let num_hex = format!("0x{:x}", num);
        assert_eq!(num_hex, "0x499602d2");
    }

    #[test]
    fn test_bytes_lower() {
        use alloy::hex;
        let ffi_u64 = bytes!("aabbccdd");

        // Convert byte data to hex string
        let byte_hex = format!("0x{}", hex::encode(ffi_u64));
        assert_eq!(byte_hex, "0xaabbccdd");
    }
}
