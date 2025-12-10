use pay_api::{
    bodies::{
        confirm_payment::{
            ConfirmPaymentParams, ConfirmPaymentResponse, ConfirmResult,
        },
        get_payment::{GetPaymentParams, GetPaymentResponse},
    },
    endpoints,
    envelope::{ErrorResponse, GatewayRequest, GatewayResponse},
};

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum PayError {
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("API error: {code} - {message}")]
    Api { code: String, message: String },
}

impl From<reqwest::Error> for PayError {
    fn from(e: reqwest::Error) -> Self {
        Self::Http(e.to_string())
    }
}

impl From<ErrorResponse> for PayError {
    fn from(e: ErrorResponse) -> Self {
        Self::Api { code: e.code, message: e.message }
    }
}

#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct WalletConnectPay {
    http_client: reqwest::Client,
    base_url: String,
}

#[cfg_attr(feature = "uniffi", uniffi::export(async_runtime = "tokio"))]
impl WalletConnectPay {
    #[cfg_attr(feature = "uniffi", uniffi::constructor)]
    pub fn new(base_url: String) -> Self {
        Self { http_client: reqwest::Client::new(), base_url }
    }

    pub async fn get_payment(
        &self,
        payment_id: String,
        accounts: Vec<String>,
    ) -> Result<GetPaymentResponse, PayError> {
        let request = GatewayRequest::GetPayment(GetPaymentParams {
            payment_id,
            accounts,
        });
        let response = self
            .http_client
            .post(format!("{}{}", self.base_url, endpoints::GATEWAY))
            .json(&request)
            .send()
            .await?
            .json::<GatewayResponse<GetPaymentResponse>>()
            .await?;
        match response {
            GatewayResponse::Success { data } => Ok(data),
            GatewayResponse::Error { error } => Err(error.into()),
        }
    }

    pub async fn confirm_payment(
        &self,
        payment_id: String,
        option_id: String,
        results: Vec<ConfirmResult>,
    ) -> Result<ConfirmPaymentResponse, PayError> {
        let request = GatewayRequest::ConfirmPayment(ConfirmPaymentParams {
            payment_id,
            option_id,
            results,
        });
        let response = self
            .http_client
            .post(format!("{}{}", self.base_url, endpoints::GATEWAY))
            .json(&request)
            .send()
            .await?
            .json::<GatewayResponse<ConfirmPaymentResponse>>()
            .await?;
        match response {
            GatewayResponse::Success { data } => Ok(data),
            GatewayResponse::Error { error } => Err(error.into()),
        }
    }
}
