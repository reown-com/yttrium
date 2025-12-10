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
pub struct GetPaymentResponse {}
