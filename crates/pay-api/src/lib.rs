use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreatePayment {
    pub amount: String, // amount as a string in decimal (must support u256 for ERC-20)
    pub currency: String, // CAIP-19 or ISO 4217
    pub reference_id: String, // 255 chars max custom reference ID
}

pub mod methods {
    pub const CREATE_PAYMENT: &str = "createPayment";
}

pub mod currencies {
    pub const USD: &str = "iso4217/USD";
}
