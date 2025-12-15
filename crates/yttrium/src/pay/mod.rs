progenitor::generate_api!(
    spec = "src/pay/openapi.json",
    interface = Builder,
    tags = Separate,
);

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum PayError {
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("API error: {0}")]
    Api(String),
    #[error("Timeout: polling exceeded maximum duration")]
    Timeout,
}

impl<T: std::fmt::Debug> From<progenitor_client::Error<T>> for PayError {
    fn from(e: progenitor_client::Error<T>) -> Self {
        Self::Api(format!("{:?}", e))
    }
}

const MAX_RETRIES: u32 = 3;
const INITIAL_BACKOFF_MS: u64 = 100;

fn is_server_error<T>(err: &progenitor_client::Error<T>) -> bool {
    matches!(err, progenitor_client::Error::ErrorResponse(resp) if resp.status().is_server_error())
}

async fn with_retry<T, E, F, Fut>(
    f: F,
) -> Result<T, progenitor_client::Error<E>>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, progenitor_client::Error<E>>>,
{
    let mut attempt = 0;
    loop {
        match f().await {
            Ok(v) => return Ok(v),
            Err(e) if is_server_error(&e) && attempt < MAX_RETRIES => {
                attempt += 1;
                let backoff = INITIAL_BACKOFF_MS * 2u64.pow(attempt - 1);
                tokio::time::sleep(std::time::Duration::from_millis(backoff))
                    .await;
            }
            Err(e) => return Err(e),
        }
    }
}

// ==================== UniFFI-compatible types ====================

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct SdkConfig {
    pub api_key: String,
    pub sdk_name: String,
    pub sdk_version: String,
    pub sdk_platform: String,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct Amount {
    pub unit: String,
    pub value: String,
}

impl From<types::Amount> for Amount {
    fn from(a: types::Amount) -> Self {
        Self { unit: a.unit, value: a.value }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct CreatePaymentResponse {
    pub payment_id: String,
    pub status: String,
    pub expires_at: i64,
    pub is_final: bool,
    pub gateway_url: String,
    pub poll_in_ms: Option<i64>,
}

impl From<types::CreatePaymentResponse> for CreatePaymentResponse {
    fn from(r: types::CreatePaymentResponse) -> Self {
        Self {
            payment_id: r.payment_id,
            status: r.status,
            expires_at: r.expires_at,
            is_final: r.is_final,
            gateway_url: r.gateway_url,
            poll_in_ms: r.poll_in_ms,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct GetPaymentResponse {
    pub status: String,
    pub amount: Amount,
    pub expires_at: i64,
}

impl From<types::GetPaymentResponse> for GetPaymentResponse {
    fn from(r: types::GetPaymentResponse) -> Self {
        Self {
            status: r.status,
            amount: r.amount.into(),
            expires_at: r.expires_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct GetPaymentStatusResponse {
    pub status: String,
    pub amount: Amount,
    pub is_final: bool,
    pub poll_in_ms: Option<i64>,
}

impl From<types::GetPaymentStatusResponse> for GetPaymentStatusResponse {
    fn from(r: types::GetPaymentStatusResponse) -> Self {
        Self {
            status: r.status,
            amount: r.amount.into(),
            is_final: r.is_final,
            poll_in_ms: r.poll_in_ms,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PaymentOptionDisplay {
    pub asset_symbol: String,
    pub asset_name: String,
    pub network_name: String,
    pub decimals: i64,
    pub icon_url: String,
}

impl From<types::PaymentOptionDisplay> for PaymentOptionDisplay {
    fn from(d: types::PaymentOptionDisplay) -> Self {
        Self {
            asset_symbol: d.asset_symbol,
            asset_name: d.asset_name,
            network_name: d.network_name,
            decimals: d.decimals,
            icon_url: d.icon_url,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct WalletRpcAction {
    pub chain_id: String,
    pub method: String,
    pub params: Vec<String>,
}

impl From<types::WalletRpcAction> for WalletRpcAction {
    fn from(a: types::WalletRpcAction) -> Self {
        Self { chain_id: a.chain_id, method: a.method, params: a.params }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum RequiredAction {
    WalletRpc { data: WalletRpcAction },
}

impl From<types::RequiredAction> for RequiredAction {
    fn from(a: types::RequiredAction) -> Self {
        match a {
            types::RequiredAction::WalletRpc(data) => {
                RequiredAction::WalletRpc { data: data.into() }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PaymentOption {
    pub id: String,
    pub unit: String,
    pub value: String,
    pub display: PaymentOptionDisplay,
    pub eta_seconds: i64,
    pub required_actions: Vec<RequiredAction>,
}

impl From<types::PaymentOption> for PaymentOption {
    fn from(o: types::PaymentOption) -> Self {
        Self {
            id: o.id,
            unit: o.unit,
            value: o.value,
            display: o.display.into(),
            eta_seconds: o.eta_seconds,
            required_actions: o
                .required_actions
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct GetPaymentOptionsResponse {
    pub status: String,
    pub amount: Amount,
    pub expires_at: i64,
    pub options: Vec<PaymentOption>,
}

impl From<types::GetPaymentOptionsResponse> for GetPaymentOptionsResponse {
    fn from(r: types::GetPaymentOptionsResponse) -> Self {
        Self {
            status: r.status,
            amount: r.amount.into(),
            expires_at: r.expires_at,
            options: r.options.into_iter().map(Into::into).collect(),
        }
    }
}

// ==================== Client ====================

#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct WalletConnectPay {
    client: Client,
    config: SdkConfig,
}

#[cfg_attr(feature = "uniffi", uniffi::export(async_runtime = "tokio"))]
impl WalletConnectPay {
    #[cfg_attr(feature = "uniffi", uniffi::constructor)]
    pub fn new(base_url: String, config: SdkConfig) -> Self {
        let client = Client::new(&base_url);
        Self { client, config }
    }

    /// Simple test method to verify bindings are loaded correctly
    pub fn sum(&self, a: i64, b: i64) -> i64 {
        a + b
    }

    // ==================== Merchant API ====================

    /// Create a new payment
    pub async fn create_payment(
        &self,
        merchant_id: String,
        reference_id: String,
        amount_unit: String,
        amount_value: String,
    ) -> Result<CreatePaymentResponse, PayError> {
        let body = types::CreatePayment {
            reference_id,
            amount: types::Amount { unit: amount_unit, value: amount_value },
        };
        let response = with_retry(|| async {
            self.client
                .create_payment_handler()
                .api_key(&self.config.api_key)
                .merchant_id(&merchant_id)
                .sdk_name(&self.config.sdk_name)
                .sdk_version(&self.config.sdk_version)
                .sdk_platform(&self.config.sdk_platform)
                .body(body.clone())
                .send()
                .await
        })
        .await?;
        Ok(response.into_inner().into())
    }

    /// Get payment status (for polling)
    pub async fn get_payment_status(
        &self,
        merchant_id: String,
        payment_id: String,
    ) -> Result<GetPaymentStatusResponse, PayError> {
        let response = with_retry(|| async {
            self.client
                .get_payment_status_handler()
                .api_key(&self.config.api_key)
                .merchant_id(&merchant_id)
                .sdk_name(&self.config.sdk_name)
                .sdk_version(&self.config.sdk_version)
                .sdk_platform(&self.config.sdk_platform)
                .id(&payment_id)
                .send()
                .await
        })
        .await?;
        Ok(response.into_inner().into())
    }

    /// Poll for payment status until it reaches a final state or timeout
    pub async fn poll_payment_status(
        &self,
        merchant_id: String,
        payment_id: String,
        timeout_ms: u64,
    ) -> Result<GetPaymentStatusResponse, PayError> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);
        loop {
            let status = self
                .get_payment_status(merchant_id.clone(), payment_id.clone())
                .await?;
            if status.is_final {
                return Ok(status);
            }
            if start.elapsed() >= timeout {
                return Err(PayError::Timeout);
            }
            let poll_delay = status.poll_in_ms.unwrap_or(1000);
            let remaining = timeout.saturating_sub(start.elapsed());
            let delay = std::time::Duration::from_millis(poll_delay as u64)
                .min(remaining);
            if delay.is_zero() {
                return Err(PayError::Timeout);
            }
            tokio::time::sleep(delay).await;
        }
    }

    // ==================== Gateway API ====================

    /// Get basic payment information
    pub async fn get_payment(
        &self,
        payment_id: String,
    ) -> Result<GetPaymentResponse, PayError> {
        let response = with_retry(|| async {
            self.client
                .get_payment_handler()
                .api_key(&self.config.api_key)
                .sdk_name(&self.config.sdk_name)
                .sdk_version(&self.config.sdk_version)
                .sdk_platform(&self.config.sdk_platform)
                .id(&payment_id)
                .send()
                .await
        })
        .await?;
        Ok(response.into_inner().into())
    }

    /// Get payment options for given accounts
    pub async fn get_payment_options(
        &self,
        payment_id: String,
        accounts: Vec<String>,
        refresh: Option<Vec<String>>,
    ) -> Result<GetPaymentOptionsResponse, PayError> {
        let body = types::GetPaymentOptionsRequest { accounts, refresh };
        let response = with_retry(|| async {
            self.client
                .get_payment_options_handler()
                .api_key(&self.config.api_key)
                .sdk_name(&self.config.sdk_name)
                .sdk_version(&self.config.sdk_version)
                .sdk_platform(&self.config.sdk_platform)
                .id(&payment_id)
                .body(body.clone())
                .send()
                .await
        })
        .await?;
        Ok(response.into_inner().into())
    }
}
