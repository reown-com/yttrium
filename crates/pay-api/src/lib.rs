#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();

pub mod bodies;
pub mod envelope;
pub mod errors;

#[cfg(test)]
mod tests;

pub mod methods {
    pub const CREATE_PAYMENT: &str = "createPayment";
    pub const GET_PAYMENT: &str = "getPayment";
    pub const BUILD_PAYMENT_REQUEST: &str = "buildPaymentRequest";
    pub const CONFIRM_PAYMENT: &str = "confirmPayment";
}

pub mod currencies {
    pub const USD: &str = "iso4217/USD";
}

pub mod payment_states {
    pub const REQUIRES_ACTION: &str = "requires_action";
}

pub mod headers {
    pub const API_KEY: &str = "x-api-key";
    pub const IDEMPOTENCY_KEY: &str = "idempotency-key";
}
