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
    pub amount: String,
    pub currency: String,
    pub reference_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetPaymentParams {
    pub payment_id: String,
    pub accounts: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
#[serde(rename_all = "camelCase")]
pub struct ConfirmResult {
    #[serde(rename = "type")]
    pub result_type: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmPaymentResponse {}
