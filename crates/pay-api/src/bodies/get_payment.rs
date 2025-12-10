use super::create_payment::Amount;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct GetPaymentParams {
    pub payment_id: String,
    pub accounts: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct GetPaymentResponse {
    pub payment_id: String,
    pub status: String,
    pub amount: Amount,
    /// Payment options computed based on provided accounts. Empty until options computation is implemented.
    #[serde(default)]
    pub options: Vec<PaymentOption>,
}

/// A payment option representing a way the user can pay
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct PaymentOption {
    pub option_id: String,
    pub chain_id: String,
    pub token: TokenInfo,
    pub amount: String,
    pub fee: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct TokenInfo {
    pub address: String,
    pub symbol: String,
    pub decimals: u8,
}
