use {
    super::{InitialTransaction, Transaction},
    crate::chain_abstraction::amount::Amount,
    alloy::primitives::{utils::Unit, Address, U256},
    relay_rpc::domain::ProjectId,
    serde::{Deserialize, Serialize},
};

pub const ROUTE_ENDPOINT_PATH: &str = "/v1/ca/orchestrator/route";

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RouteQueryParams {
    pub project_id: ProjectId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteRequest {
    pub transaction: InitialTransaction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub funding_from: Vec<FundingMetadata>,
    pub initial_transaction: InitialTransactionMetadata,
    pub check_in: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
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
#[serde(rename_all = "camelCase")]
pub struct FundingMetadata {
    pub chain_id: String,
    pub token_contract: Address,
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
#[serde(rename_all = "camelCase")]
pub struct RouteResponseAvailable {
    pub orchestration_id: String,
    pub initial_transaction: Transaction,
    pub transactions: Vec<Transaction>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
#[serde(rename_all = "camelCase")]
pub struct RouteResponseNotRequired {
    pub initial_transaction: Transaction,
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Enum))]
#[serde(untagged)]
pub enum RouteResponseSuccess {
    Available(RouteResponseAvailable),
    NotRequired(RouteResponseNotRequired),
}

impl RouteResponseSuccess {
    pub fn into_option(self) -> Option<RouteResponseAvailable> {
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
pub struct RouteResponseError {
    pub error: BridgingError,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Enum))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BridgingError {
    NoRoutesAvailable,
    InsufficientFunds,
    InsufficientGasFunds,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Enum))]
#[serde(untagged)]
pub enum RouteResponse {
    Success(RouteResponseSuccess),
    Error(RouteResponseError),
}

impl RouteResponse {
    pub fn into_result(
        self,
    ) -> Result<RouteResponseSuccess, RouteResponseError> {
        match self {
            Self::Success(success) => Ok(success),
            Self::Error(error) => Err(error),
        }
    }
}
