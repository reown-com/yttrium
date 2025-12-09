use serde::{Deserialize, Serialize};

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
pub struct ConfirmPaymentResponse {
    pub payment_id: String,
    pub status: String,
}
