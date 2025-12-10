#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();

pub mod bodies;
pub mod envelope;
pub mod errors;

#[cfg(test)]
mod tests;

pub mod methods {
    pub const CREATE_PAYMENT: &str = "createPayment";
    pub const GET_PAYMENT_STATUS: &str = "getPaymentStatus";
    pub const GET_PAYMENT: &str = "getPayment";
    pub const BUILD_PAYMENT_REQUEST: &str = "buildPaymentRequest";
    pub const CONFIRM_PAYMENT: &str = "confirmPayment";
}

pub mod currencies {
    pub const USD: &str = "iso4217/USD";
}

pub mod payment_states {
    pub const REQUIRES_ACTION: &str = "requires_action";
    pub const PROCESSING: &str = "processing";
    pub const SUCCEEDED: &str = "succeeded";
    pub const FAILED: &str = "failed";
    pub const EXPIRED: &str = "expired";

    pub const ALL: [&str; 5] =
        [REQUIRES_ACTION, PROCESSING, SUCCEEDED, FAILED, EXPIRED];
}

pub mod headers {
    pub const API_KEY: &str = "x-api-key";
    pub const IDEMPOTENCY_KEY: &str = "idempotency-key";
}

pub mod endpoints {
    pub const GATEWAY: &str = "/v1/gateway";
}
