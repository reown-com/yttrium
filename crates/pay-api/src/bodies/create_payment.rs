use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct Amount {
    pub unit: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Object))]
pub struct CreatePayment {
    pub reference_id: String,
    pub amount: Amount,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Object))]
pub struct CreatePaymentResponse {
    pub payment_id: String,
    pub status: String,
    pub amount: Amount,
    pub expires_at: u64,
    pub poll_in_ms: u64,
}
