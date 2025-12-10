use {
    crate::{bodies, methods},
    serde::{Deserialize, Serialize},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", tag = "method", content = "params")]
pub enum GatewayRequest {
    CreatePayment(bodies::create_payment::CreatePayment),
    GetPaymentStatus(bodies::get_payment_status::GetPaymentStatusParams),
    GetPayment(bodies::get_payment::GetPaymentParams),
    BuildPaymentRequest,
    ConfirmPayment(bodies::confirm_payment::ConfirmPaymentParams),
}

impl GatewayRequest {
    pub fn method(&self) -> &str {
        match self {
            GatewayRequest::CreatePayment(_) => methods::CREATE_PAYMENT,
            GatewayRequest::GetPaymentStatus(_) => methods::GET_PAYMENT_STATUS,
            GatewayRequest::BuildPaymentRequest => {
                methods::BUILD_PAYMENT_REQUEST
            }
            GatewayRequest::GetPayment(_) => methods::GET_PAYMENT,
            GatewayRequest::ConfirmPayment(_) => methods::CONFIRM_PAYMENT,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", tag = "status")]
pub enum GatewayResponse<T> {
    Success { data: T },
    Error { error: ErrorResponse },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}
