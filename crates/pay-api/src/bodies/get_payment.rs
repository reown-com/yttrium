use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetPaymentParams {
    pub payment_id: String,
    pub accounts: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct GetPaymentResponse {
    pub payment_id: String,
    pub status: String,
    pub amount: PaymentAmount,
    pub options: Vec<PaymentOption>,
    pub poll_in_ms: u64,
    pub expires_at: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct PaymentAmount {
    pub unit: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct PaymentOption {
    pub id: String,
    pub unit: String,
    pub value: String,
    pub display: PaymentOptionDisplay,
    pub eta_seconds: u64,
    pub required_actions: Vec<RequiredAction>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct PaymentOptionDisplay {
    pub asset_symbol: String,
    pub asset_name: String,
    pub network_name: String,
    pub network_short: String,
    pub decimals: u8,
    pub icon_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[serde(rename_all = "camelCase", tag = "type", content = "data")]
pub enum RequiredAction {
    WalletRpc(WalletRpcAction),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct WalletRpcAction {
    pub chain_id: String,
    pub method: String,
    pub params: Vec<String>,
}
