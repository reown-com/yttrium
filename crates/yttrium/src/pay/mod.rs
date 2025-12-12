progenitor::generate_api!(
    spec = "src/pay/openapi.json",
    interface = Builder,
    tags = Separate,
    derives = [Debug, Clone, PartialEq, uniffi::Record],
);

pub use types::{Amount, CreatePayment, CreatePaymentResponse};

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum PayError {
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("API error: {0}")]
    Api(String),
}

impl<T: std::fmt::Debug> From<progenitor_client::Error<T>> for PayError {
    fn from(e: progenitor_client::Error<T>) -> Self {
        Self::Api(format!("{:?}", e))
    }
}

#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct WalletConnectPay {
    client: Client,
}

#[cfg_attr(feature = "uniffi", uniffi::export(async_runtime = "tokio"))]
impl WalletConnectPay {
    #[cfg_attr(feature = "uniffi", uniffi::constructor)]
    pub fn new(base_url: &str) -> Self {
        let client = Client::new(base_url);
        Self { client }
    }

    pub async fn create_payment(
        &self,
        reference_id: String,
        amount_unit: String,
        amount_value: String,
    ) -> Result<CreatePaymentResponse, PayError> {
        let body = CreatePayment {
            reference_id,
            amount: Amount { unit: amount_unit, value: amount_value },
        };
        let response = self.client.create_payment().body(body).send().await?;
        Ok(response.into_inner())
    }
}
