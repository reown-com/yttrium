#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;
use {
    super::Transaction,
    crate::{call::Call, chain_abstraction::amount::Amount},
    alloy::primitives::{utils::Unit, Address, U256},
    core::fmt,
    relay_rpc::domain::ProjectId,
    serde::{Deserialize, Serialize},
    std::str::FromStr,
};
#[cfg(feature = "solana")]
use {
    crate::chain_abstraction::solana,
    solana_sdk::transaction::VersionedTransaction,
};

pub const ROUTE_ENDPOINT_PATH: &str = "/v2/ca/orchestrator/route";

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RouteQueryParams {
    pub project_id: ProjectId,
    pub session_id: Option<String>,
    #[serde(rename = "st")]
    pub sdk_type: Option<String>,
    #[serde(rename = "sv")]
    pub sdk_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrepareRequest {
    pub transaction: PrepareRequestTransaction,
    /// List of CAIP-10 accounts
    #[serde(default)]
    pub accounts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrepareRequestTransaction {
    pub chain_id: String,
    pub from: Address,
    #[serde(flatten)]
    pub calls: CallOrCalls,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CallOrCalls {
    Call {
        #[serde(flatten)]
        call: Call,
    },
    // Don't use this yet until Blockchain API upgrades
    Calls {
        calls: Vec<Call>,
    },
}

impl CallOrCalls {
    pub fn into_calls(self) -> Vec<Call> {
        match self {
            Self::Call { call } => vec![call],
            Self::Calls { calls } => calls,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Eip155OrSolanaAddress {
    #[cfg(feature = "eip155")]
    Eip155(Address),
    #[cfg(feature = "solana")]
    Solana(crate::chain_abstraction::solana::SolanaPubkey),
}

impl fmt::Display for Eip155OrSolanaAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "eip155")]
            Self::Eip155(address) => address.fmt(f),
            #[cfg(feature = "solana")]
            Self::Solana(address) => address.fmt(f),
        }
    }
}

impl Eip155OrSolanaAddress {
    #[cfg(feature = "solana")]
    pub fn as_solana(
        &self,
    ) -> Option<&crate::chain_abstraction::solana::SolanaPubkey> {
        match self {
            Self::Solana(address) => Some(address),
            Self::Eip155(_) => None,
        }
    }

    #[cfg(feature = "eip155")]
    pub fn as_eip155(&self) -> Option<&Address> {
        match self {
            Self::Eip155(address) => Some(address),
            #[cfg(feature = "solana")]
            Self::Solana(_) => None,
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum Eip155OrSolanaAddressParseError {
    #[error("Invalid EIP-155 address: {0}")]
    #[cfg(feature = "eip155")]
    Eip155(alloy::hex::FromHexError),

    #[error("Invalid Solana address: {0}")]
    #[cfg(feature = "solana")]
    Solana(solana_sdk::pubkey::ParsePubkeyError),
}

impl FromStr for Eip155OrSolanaAddress {
    type Err = Eip155OrSolanaAddressParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("0x") {
            Ok(Self::Eip155(
                Address::from_str(s)
                    .map_err(Eip155OrSolanaAddressParseError::Eip155)?,
            ))
        } else {
            #[cfg(feature = "solana")]
            {
                Ok(Self::Solana(
                    crate::chain_abstraction::solana::SolanaPubkey::from_str(s)
                        .map_err(Eip155OrSolanaAddressParseError::Solana)?,
                ))
            }
            #[cfg(not(feature = "solana"))]
            {
                panic!("solana feature is not enabled");
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub funding_from: Vec<FundingMetadata>,
    pub initial_transaction: InitialTransactionMetadata,
    /// The number of milliseconds to delay before calling `/status` after getting successful transaction receipts from all sent transactions.
    /// Not switching to Duration yet because Kotlin maps this to a native `duration` type but this requires API version 26 but we support 23.
    /// https://reown-inc.slack.com/archives/C07HQ8RCGD8/p1738740204879269
    pub check_in: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub struct InitialTransactionMetadata {
    pub transfer_to: Address,
    pub amount: U256,
    pub token_contract: Address,
    pub symbol: String,
    pub decimals: u8,
}

impl InitialTransactionMetadata {
    pub fn to_amount(&self) -> Amount {
        Amount::new(
            self.symbol.clone(),
            self.amount,
            Unit::new(self.decimals).unwrap(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub struct FundingMetadata {
    // TODO consolidate into a single CAIP-10 account ID
    pub chain_id: String,
    pub token_contract: Eip155OrSolanaAddress,
    pub symbol: String,

    // The amount that was sourced (includes the bridging fee)
    pub amount: U256,

    // The amount taken by the bridge as a fee
    pub bridging_fee: U256,

    // #[serde(
    //     deserialize_with = "crate::utils::deserialize_unit",
    //     serialize_with = "crate::utils::serialize_unit"
    // )]
    pub decimals: u8,
}

// TODO remove default when Blockchain API is updated to provide this
// fn default_unit() -> Unit {
//     Unit::new(6).unwrap()
// }

impl FundingMetadata {
    pub fn to_amount(&self) -> Amount {
        Amount::new(
            self.symbol.clone(),
            self.amount,
            Unit::new(self.decimals).unwrap(),
        )
    }

    pub fn to_bridging_fee_amount(&self) -> Amount {
        Amount::new(
            self.symbol.clone(),
            self.bridging_fee,
            Unit::new(self.decimals).unwrap(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub struct PrepareResponseAvailable {
    pub orchestration_id: String,
    pub initial_transaction: Transaction,
    pub transactions: Vec<Transactions>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Enum))]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub enum Transactions {
    #[cfg(feature = "eip155")]
    Eip155(Vec<Transaction>),
    #[cfg(feature = "solana")]
    Solana(Vec<SolanaTransaction>),
}

impl Transactions {
    #[cfg(feature = "eip155")]
    pub fn into_eip155(self) -> Option<Vec<Transaction>> {
        match self {
            Self::Eip155(txns) => Some(txns),
            #[cfg(feature = "solana")]
            Self::Solana(_) => None,
        }
    }

    #[cfg(feature = "eip155")]
    pub fn as_eip155(&self) -> Option<&Vec<Transaction>> {
        match self {
            Self::Eip155(txns) => Some(txns),
            #[cfg(feature = "solana")]
            Self::Solana(_) => None,
        }
    }

    #[cfg(feature = "solana")]
    pub fn into_solana(self) -> Option<Vec<SolanaTransaction>> {
        match self {
            Self::Solana(txns) => Some(txns),
            Self::Eip155(_) => None,
        }
    }

    #[cfg(feature = "solana")]
    pub fn as_solana(&self) -> Option<&Vec<SolanaTransaction>> {
        match self {
            Self::Solana(txns) => Some(txns),
            Self::Eip155(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "solana")]
pub struct SolanaTransaction {
    pub chain_id: String,
    pub from: solana::SolanaPubkey,
    pub transaction: VersionedTransaction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub struct PrepareResponseNotRequired {
    pub initial_transaction: Transaction,
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Enum))]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(untagged)]
pub enum PrepareResponseSuccess {
    Available(PrepareResponseAvailable),
    NotRequired(PrepareResponseNotRequired),
}

impl PrepareResponseSuccess {
    pub fn into_option(self) -> Option<PrepareResponseAvailable> {
        match self {
            Self::Available(a) => Some(a),
            Self::NotRequired(_) => None,
        }
    }
}

/// Bridging check error response that should be returned as a normal HTTP 200
/// response
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub struct PrepareResponseError {
    pub error: BridgingError,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Enum))]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BridgingError {
    NoRoutesAvailable,
    InsufficientFunds,
    InsufficientGasFunds,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Enum))]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(untagged)]
pub enum PrepareResponse {
    Success(PrepareResponseSuccess),
    Error(PrepareResponseError),
}

impl PrepareResponse {
    pub fn into_result(
        self,
    ) -> Result<PrepareResponseSuccess, PrepareResponseError> {
        match self {
            Self::Success(success) => Ok(success),
            Self::Error(error) => Err(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use {super::*, alloy::primitives::Bytes};

    #[test]
    fn deserializes_current_request_body() {
        let chain_id = "eip155:1";
        let from = Address::ZERO;
        let to = Address::ZERO;
        let value = U256::from(0);
        let input = Bytes::new();
        let json = serde_json::json!({
            "transaction": {
                "chainId": chain_id,
                "from": from,
                "to": to,
                "value": value,
                "input": input,
            }
        });
        let result = serde_json::from_value::<PrepareRequest>(json).unwrap();
        assert_eq!(result.transaction.chain_id, chain_id);
        assert_eq!(result.transaction.from, from);
        assert!(matches!(result.transaction.calls, CallOrCalls::Call { .. }));
        let calls = result.transaction.calls.into_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].to, to);
        assert_eq!(calls[0].value, value);
        assert_eq!(calls[0].input, input);
        assert_eq!(result.accounts, Vec::<String>::new());
    }

    #[test]
    fn deserializes_accounts() {
        let chain_id = "eip155:1";
        let from = Address::ZERO;
        let to = Address::ZERO;
        let value = U256::from(0);
        let input = Bytes::new();
        let json = serde_json::json!({
            "transaction": {
                "chainId": chain_id,
                "from": from,
                "to": to,
                "value": value,
                "input": input,
            },
            "accounts": [
                "eip155:1:0x0000000000000000000000000000000000000000",
                "eip155:1:0x0000000000000000000000000000000000000001",
            ]
        });
        let result = serde_json::from_value::<PrepareRequest>(json).unwrap();
        assert_eq!(result.transaction.chain_id, chain_id);
        assert_eq!(result.transaction.from, from);
        assert_eq!(
            result.accounts,
            vec![
                "eip155:1:0x0000000000000000000000000000000000000000",
                "eip155:1:0x0000000000000000000000000000000000000001",
            ]
        );
        assert!(matches!(result.transaction.calls, CallOrCalls::Call { .. }));
        let calls = result.transaction.calls.into_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].to, to);
        assert_eq!(calls[0].value, value);
        assert_eq!(calls[0].input, input);
    }

    #[test]
    fn deserializes_single_call() {
        let chain_id = "eip155:1";
        let from = Address::ZERO;
        let to = Address::ZERO;
        let value = U256::from(0);
        let input = Bytes::new();
        let json = serde_json::json!({
            "transaction": {
                "chainId": chain_id,
                "from": from,
                "calls": [{
                    "to": to,
                    "value": value,
                    "input": input,
                }]
            }
        });
        let result = serde_json::from_value::<PrepareRequest>(json).unwrap();
        assert_eq!(result.transaction.chain_id, chain_id);
        assert_eq!(result.transaction.from, from);
        assert!(matches!(result.transaction.calls, CallOrCalls::Calls { .. }));
        let calls = result.transaction.calls.into_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].to, to);
        assert_eq!(calls[0].value, value);
        assert_eq!(calls[0].input, input);
    }

    #[test]
    fn deserializes_multiple_calls() {
        let chain_id = "eip155:1";
        let from = Address::ZERO;
        let to = Address::ZERO;
        let value = U256::from(0);
        let input = Bytes::new();
        let json = serde_json::json!({
            "transaction": {
                "chainId": chain_id,
                "from": from,
                "calls": [{
                    "to": to,
                    "value": value,
                    "input": input,
                }, {
                    "to": to,
                    "value": value,
                    "input": input,
                }]
            }
        });
        let result = serde_json::from_value::<PrepareRequest>(json).unwrap();
        assert_eq!(result.transaction.chain_id, chain_id);
        assert_eq!(result.transaction.from, from);
        assert!(matches!(result.transaction.calls, CallOrCalls::Calls { .. }));
        let calls = result.transaction.calls.into_calls();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].to, to);
        assert_eq!(calls[0].value, value);
        assert_eq!(calls[0].input, input);
    }

    #[test]
    fn deserializes_unknown_bridging_error() {
        let json = serde_json::json!({
            "error": "NEW_ERROR_TYPE",
            "reason": "Some new error type we don't know about"
        });
        let result = serde_json::from_value::<PrepareResponseError>(json).unwrap();
        assert!(matches!(result.error, BridgingError::Unknown));
        assert_eq!(result.reason, "Some new error type we don't know about");
    }

    #[test]
    fn serializes_unknown_bridging_error() {
        let error = PrepareResponseError {
            error: BridgingError::Unknown,
            reason: "Test reason".to_string(),
        };
        let json = serde_json::to_value(&error).unwrap();
        assert_eq!(json["error"], "UNKNOWN");
        assert_eq!(json["reason"], "Test reason");
    }
}
