use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Object))]
pub struct CreatePayment {
    pub amount: String,
    pub currency: String,
    pub reference_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Object))]
pub struct CreatePaymentResponse {
    pub payment_id: String,
}
