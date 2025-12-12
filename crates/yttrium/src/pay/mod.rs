progenitor::generate_api!(
    spec = "src/pay/openapi.json",
    interface = Builder,
    tags = Separate,
    derives = [Debug, Clone, PartialEq],
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

// FFI wrappers for progenitor-generated types
#[derive(Debug, Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct AmountFfi {
    pub unit: String,
    pub value: String,
}

impl From<Amount> for AmountFfi {
    fn from(a: Amount) -> Self {
        Self { unit: a.unit, value: a.value }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct CreatePaymentResponseFfi {
    pub payment_id: String,
    pub status: String,
    pub amount: AmountFfi,
    pub expires_at: i64,
    pub poll_in_ms: i64,
    pub gateway_url: String,
}

impl From<CreatePaymentResponse> for CreatePaymentResponseFfi {
    fn from(r: CreatePaymentResponse) -> Self {
        Self {
            payment_id: r.payment_id,
            status: r.status,
            amount: r.amount.into(),
            expires_at: r.expires_at,
            poll_in_ms: r.poll_in_ms,
            gateway_url: r.gateway_url,
        }
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
    ) -> Result<CreatePaymentResponseFfi, PayError> {
        let body = CreatePayment {
            reference_id,
            amount: Amount { unit: amount_unit, value: amount_value },
        };
        let response = self.client.create_payment().body(body).send().await?;
        Ok(response.into_inner().into())
    }
}
