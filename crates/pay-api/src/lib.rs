use serde::{Deserialize, Serialize};

pub mod methods {
    pub const CREATE_PAYMENT: &str = "createPayment";
    pub const GET_PAYMENT: &str = "getPayment";
    pub const BUILD_PAYMENT: &str = "buildPayment";
    pub const CONFIRM_PAYMENT: &str = "confirmPayment";
}

pub mod currencies {
    pub const USD: &str = "iso4217/USD";
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreatePaymentParams {
    pub amount: String,       // amount as a string in decimal (must support u256 for ERC-20)
    pub currency: String,     // CAIP-19 or ISO 4217
    pub reference_id: String, // 255 chars max custom reference ID
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreatePaymentResponse {
    pub payment_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetPaymentParams {
    pub payment_id: String,
    pub accounts: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetPaymentResponse {
    // payment_id
    // status
    // options? // only when accounts is passed to GetPaymentParams
    // actions?
    // pollInMs?
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BuildPaymentParams {
    pub option_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BuildPaymentResponse {}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmPaymentParams {
    pub payment_id: String,
    pub option_id: String,
    pub results: Vec<ConfirmResult>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmResult {
    #[serde(rename = "type")]
    pub result_type: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmPaymentResponse {}
