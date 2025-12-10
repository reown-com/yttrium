#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();

use serde::{Deserialize, Serialize};

pub mod methods {
    pub const CREATE_PAYMENT: &str = "createPayment";
    pub const GET_PAYMENT: &str = "getPayment";
    pub const CONFIRM_PAYMENT: &str = "confirmPayment";
}

pub mod currencies {
    pub const USD: &str = "iso4217/USD";
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreatePayment {
    pub amount: String,       // amount as a string in decimal (must support u256 for ERC-20)
    pub currency: String,     // CAIP-19 or ISO 4217
    pub reference_id: String, // 255 chars max custom reference ID
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetPaymentParams {
    pub payment_id: String,
    pub accounts: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct GetPaymentResponse {}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmPaymentParams {
    pub payment_id: String,
    pub option_id: String,
    pub results: Vec<ConfirmResult>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct ConfirmResult {
    #[serde(rename = "type")]
    pub result_type: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct ConfirmPaymentResponse {}
