use {
    pay_api::{
        methods, ConfirmPaymentParams, ConfirmPaymentResponse, ConfirmResult,
        GetPaymentParams, GetPaymentResponse,
    },
    serde::{Deserialize, Serialize},
};

#[derive(Debug, thiserror::Error)]
pub enum PayError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("API error: {code} - {message}")]
    Api { code: String, message: String },
}

#[derive(Debug, Serialize)]
struct ApiRequest<P> {
    method: String,
    params: P,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "status", rename_all = "lowercase")]
enum ApiResponse<T> {
    Success { data: T },
    Error { error: ApiError },
}

#[derive(Debug, Deserialize)]
struct ApiError {
    code: String,
    message: String,
}

impl From<ApiError> for PayError {
    fn from(e: ApiError) -> Self {
        Self::Api { code: e.code, message: e.message }
    }
}

pub struct WalletConnectPay {
    http_client: reqwest::Client,
    base_url: String,
}

impl WalletConnectPay {
    pub fn new(base_url: String) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            base_url,
        }
    }

    pub async fn get_payment(
        &self,
        payment_id: String,
        accounts: Vec<String>,
    ) -> Result<GetPaymentResponse, PayError> {
        let request = ApiRequest {
            method: methods::GET_PAYMENT.to_owned(),
            params: GetPaymentParams { payment_id, accounts },
        };
        let response = self
            .http_client
            .post(format!("{}/v1/gateway", self.base_url))
            .json(&request)
            .send()
            .await?
            .json::<ApiResponse<GetPaymentResponse>>()
            .await?;
        match response {
            ApiResponse::Success { data } => Ok(data),
            ApiResponse::Error { error } => Err(error.into()),
        }
    }

    pub async fn confirm_payment(
        &self,
        payment_id: String,
        option_id: String,
        results: Vec<ConfirmResult>,
    ) -> Result<ConfirmPaymentResponse, PayError> {
        let request = ApiRequest {
            method: methods::CONFIRM_PAYMENT.to_owned(),
            params: ConfirmPaymentParams { payment_id, option_id, results },
        };
        let response = self
            .http_client
            .post(format!("{}/v1/gateway", self.base_url))
            .json(&request)
            .send()
            .await?
            .json::<ApiResponse<ConfirmPaymentResponse>>()
            .await?;
        match response {
            ApiResponse::Success { data } => Ok(data),
            ApiResponse::Error { error } => Err(error.into()),
        }
    }
}
