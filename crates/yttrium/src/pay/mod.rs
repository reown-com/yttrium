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

// JSON string wrapper client for Flutter/React Native
#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct WalletConnectPayJson {
    client: WalletConnectPay,
}

#[cfg_attr(feature = "uniffi", uniffi::export(async_runtime = "tokio"))]
impl WalletConnectPayJson {
    #[cfg_attr(feature = "uniffi", uniffi::constructor)]
    pub fn new(base_url: &str) -> Self {
        Self { client: WalletConnectPay::new(base_url) }
    }

    /// Create a payment. Takes a JSON string and returns a JSON string.
    /// 
    /// Input JSON format:
    /// ```json
    /// {
    ///   "referenceId": "string",
    ///   "amount": {
    ///     "unit": "string",
    ///     "value": "string"
    ///   }
    /// }
    /// ```
    /// 
    /// Returns JSON string with the payment response or error.
    pub async fn create_payment(&self, request_json: String) -> Result<String, PayError> {
        let request: CreatePayment = serde_json::from_str(&request_json)
            .map_err(|e| PayError::Api(format!("Invalid JSON: {}", e)))?;
        
        let response = self.client.create_payment(
            request.reference_id,
            request.amount.unit,
            request.amount.value,
        ).await?;
        
        serde_json::to_string(&response)
            .map_err(|e| PayError::Api(format!("Failed to serialize response: {}", e)))
    }
}
