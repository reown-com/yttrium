use serde::{Deserialize, Serialize};

pub mod methods {
    pub const GET_PAYMENT: &str = "getPayment";
}

pub mod currencies {
    pub const USD: &str = "iso4217/USD";
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetPaymentParams {
    pub payment_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accounts: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetPaymentResponse {}
