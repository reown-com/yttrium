use {
    pay_api::{methods, GetPaymentParams, GetPaymentResponse},
    relay_rpc::domain::ProjectId,
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
    _project_id: ProjectId,
    http_client: reqwest::Client,
    base_url: String,
}

impl WalletConnectPay {
    pub fn new(project_id: ProjectId, base_url: String) -> Self {
        Self {
            _project_id: project_id,
            http_client: reqwest::Client::new(),
            base_url,
        }
    }

    pub async fn get_payment(
        &self,
        payment_id: String,
        accounts: Option<Vec<String>>,
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
}
