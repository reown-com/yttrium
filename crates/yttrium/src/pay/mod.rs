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

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum GetPaymentOptionsError {
    #[error("Payment expired: {0}")]
    PaymentExpired(String),
    #[error("Payment not found: {0}")]
    PaymentNotFound(String),
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("Option not found: {0}")]
    OptionNotFound(String),
    #[error("Payment not ready: {0}")]
    PaymentNotReady(String),
    #[error("Invalid account: {0}")]
    InvalidAccount(String),
    #[error("Compliance failed: {0}")]
    ComplianceFailed(String),
    #[error("HTTP error: {0}")]
    Http(String),
}

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum GetPaymentRequestError {
    #[error("Option not found: {0}")]
    OptionNotFound(String),
    #[error("Payment not found: {0}")]
    PaymentNotFound(String),
    #[error("Invalid account: {0}")]
    InvalidAccount(String),
    #[error("HTTP error: {0}")]
    Http(String),
}

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum ConfirmPaymentError {
    #[error("Payment not found: {0}")]
    PaymentNotFound(String),
    #[error("Payment expired: {0}")]
    PaymentExpired(String),
    #[error("Invalid option: {0}")]
    InvalidOption(String),
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    #[error("Route expired: {0}")]
    RouteExpired(String),
    #[error("Collect data required but not provided")]
    CollectDataRequired,
    #[error("HTTP error: {0}")]
    Http(String),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum PaymentStatus {
    Completed,
    Failed,
    Expired,
}

impl From<types::ConfirmPaymentResponseStatus> for PaymentStatus {
    fn from(s: types::ConfirmPaymentResponseStatus) -> Self {
        match s {
            types::ConfirmPaymentResponseStatus::Completed => {
                PaymentStatus::Completed
            }
            types::ConfirmPaymentResponseStatus::Failed => {
                PaymentStatus::Failed
            }
            types::ConfirmPaymentResponseStatus::Expired => {
                PaymentStatus::Expired
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PaymentResult {
    pub rpc_method: String,
    pub rpc_result: String,
    pub chain_id: String,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct InformationCapture {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub extra: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct ConfirmPaymentResponse {
    pub payment_id: String,
    pub status: PaymentStatus,
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
pub struct PayAmount {
    pub unit: String,
    pub value: String,
}

impl From<types::Amount> for PayAmount {
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
    pub amount: PayAmount,
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
    pub amount: PayAmount,
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
    pub network_short: String,
    pub decimals: i32,
    pub icon_url: String,
}

impl From<types::PaymentOptionDisplay> for PaymentOptionDisplay {
    fn from(d: types::PaymentOptionDisplay) -> Self {
        Self {
            asset_symbol: d.asset_symbol,
            asset_name: d.asset_name,
            network_name: d.network_name,
            network_short: d.network_short,
            decimals: d.decimals.try_into().unwrap_or_default(),
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
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct CollectDataSchemaField {
    pub name: String,
    pub field_type: String,
    pub required: bool,
}

impl From<types::CollectDataSchemaField> for CollectDataSchemaField {
    fn from(f: types::CollectDataSchemaField) -> Self {
        Self { name: f.name, field_type: f.type_, required: f.required }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct CollectDataSchema {
    pub fields: Vec<CollectDataSchemaField>,
}

impl From<types::CollectDataSchema> for CollectDataSchema {
    fn from(s: types::CollectDataSchema) -> Self {
        Self { fields: s.fields.into_iter().map(Into::into).collect() }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct CollectDataAction {
    pub schema: CollectDataSchema,
}

impl From<types::CollectDataAction> for CollectDataAction {
    fn from(a: types::CollectDataAction) -> Self {
        Self { schema: a.schema.into() }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum RequiredAction {
    WalletRpc { data: WalletRpcAction },
    CollectData { data: CollectDataAction },
}

impl From<types::RequiredAction> for RequiredAction {
    fn from(a: types::RequiredAction) -> Self {
        match a {
            types::RequiredAction::WalletRpc(data) => {
                RequiredAction::WalletRpc { data: data.into() }
            }
            types::RequiredAction::CollectData(data) => {
                RequiredAction::CollectData { data: data.into() }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PaymentRequest {
    pub payment_id: String,
    pub option_id: String,
    pub action: RequiredAction,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PaymentOption {
    pub payment_id: String,
    pub option_id: String,
    pub unit: String,
    pub value: String,
    pub display: PaymentOptionDisplay,
}

impl From<types::PaymentOption> for PaymentOption {
    fn from(o: types::PaymentOption) -> Self {
        Self {
            payment_id: o.payment_id,
            option_id: o.id,
            unit: o.unit,
            value: o.value,
            display: o.display.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct DataCaptureField {
    pub label: String,
    pub required: bool,
}

impl From<types::DataCaptureField> for DataCaptureField {
    fn from(f: types::DataCaptureField) -> Self {
        Self { label: f.label, required: f.required }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct DataCaptureRequirements {
    pub required: bool,
    pub fields: Vec<DataCaptureField>,
}

impl From<types::DataCaptureRequirements> for DataCaptureRequirements {
    fn from(r: types::DataCaptureRequirements) -> Self {
        Self {
            required: r.required,
            fields: r.fields.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PaymentSession {
    pub payment_id: String,
    pub amount: PayAmount,
    pub options: Vec<PaymentOption>,
    pub data_capture_requirements: Option<DataCaptureRequirements>,
}

impl From<types::GetPaymentOptionsResponse> for PaymentSession {
    fn from(r: types::GetPaymentOptionsResponse) -> Self {
        Self {
            payment_id: r.payment_id,
            amount: r.amount.into(),
            options: r.options.into_iter().map(Into::into).collect(),
            data_capture_requirements: r
                .data_capture_requirements
                .map(Into::into),
        }
    }
}

// ==================== Client ====================

use std::sync::RwLock;

#[derive(Debug, Clone)]
struct CachedPaymentOption {
    payment_id: String,
    option_id: String,
    required_actions: Vec<RequiredAction>,
}

#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct WalletConnectPay {
    client: Client,
    config: SdkConfig,
    cached_options: RwLock<Vec<CachedPaymentOption>>,
}

#[cfg_attr(feature = "uniffi", uniffi::export(async_runtime = "tokio"))]
impl WalletConnectPay {
    #[cfg_attr(feature = "uniffi", uniffi::constructor)]
    pub fn new(base_url: String, config: SdkConfig) -> Self {
        let client = Client::new(&base_url);
        Self { client, config, cached_options: RwLock::new(Vec::new()) }
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

    /// Get payment options for given accounts (returns PaymentSession for UI)
    /// Also caches the options for use by get_required_payment_actions
    pub async fn get_payment_options(
        &self,
        payment_link: String,
        accounts: Vec<String>,
    ) -> Result<PaymentSession, GetPaymentOptionsError> {
        let payment_id = extract_payment_id(&payment_link);
        let body = types::GetPaymentOptionsRequest { accounts, refresh: None };
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
        .await
        .map_err(map_payment_options_error)?;

        let api_response = response.into_inner();

        // Cache the options with their required actions
        {
            let cached: Vec<CachedPaymentOption> = api_response
                .options
                .iter()
                .map(|o| CachedPaymentOption {
                    payment_id: o.payment_id.clone(),
                    option_id: o.id.clone(),
                    required_actions: o
                        .required_actions
                        .iter()
                        .cloned()
                        .map(Into::into)
                        .collect(),
                })
                .collect();
            if let Ok(mut cache) = self.cached_options.write() {
                *cache = cached;
            }
        }

        Ok(api_response.into())
    }

    /// Get required payment actions for a selected option
    /// Returns the list of actions from the cached PaymentSession (must call get_payment_options first)
    pub fn get_required_payment_actions(
        &self,
        option_id: String,
        payment_id: String,
        _account: String,
    ) -> Result<Vec<PaymentRequest>, GetPaymentRequestError> {
        let cache = self.cached_options.read().map_err(|_| {
            GetPaymentRequestError::OptionNotFound(
                "Failed to read cache".to_string(),
            )
        })?;

        // TODO: Call buildPaymentRequest endpoint when option is not found in cache
        let cached_option = cache
            .iter()
            .find(|o| o.option_id == option_id && o.payment_id == payment_id)
            .ok_or_else(|| {
                GetPaymentRequestError::OptionNotFound(format!(
                    "Option {} not found in cache. Call get_payment_options first.",
                    option_id
                ))
            })?;

        let requests: Vec<PaymentRequest> = cached_option
            .required_actions
            .iter()
            .cloned()
            .map(|action| PaymentRequest {
                payment_id: payment_id.clone(),
                option_id: option_id.clone(),
                action,
            })
            .collect();

        Ok(requests)
    }

    /// Confirm a payment with wallet RPC results
    /// Throws CollectDataRequired if collect-data action was required but kycData is not provided
    pub async fn confirm_payment(
        &self,
        request: PaymentRequest,
        result: PaymentResult,
        kyc_data: Option<InformationCapture>,
    ) -> Result<ConfirmPaymentResponse, ConfirmPaymentError> {
        // Check if collect-data was required (scoped to drop lock before await)
        {
            let cache = self.cached_options.read().map_err(|_| {
                ConfirmPaymentError::InvalidOption(
                    "Failed to read cache".to_string(),
                )
            })?;

            let cached_option = cache.iter().find(|o| {
                o.option_id == request.option_id
                    && o.payment_id == request.payment_id
            });

            if let Some(option) = cached_option {
                let requires_collect_data = option
                    .required_actions
                    .iter()
                    .any(|a| matches!(a, RequiredAction::CollectData { .. }));
                if requires_collect_data && kyc_data.is_none() {
                    return Err(ConfirmPaymentError::CollectDataRequired);
                }
            }
        }

        let kyc = kyc_data.map(|k| types::KycData {
            first_name: k.first_name,
            last_name: k.last_name,
            extra: k.extra,
        });

        let body = types::ConfirmPaymentRequest {
            option_id: request.option_id,
            result: types::PaymentResultBody {
                rpc_method: result.rpc_method,
                rpc_result: result.rpc_result,
                chain_id: result.chain_id,
            },
            kyc_data: kyc,
        };

        let response = with_retry(|| async {
            self.client
                .confirm_payment_handler()
                .api_key(&self.config.api_key)
                .sdk_name(&self.config.sdk_name)
                .sdk_version(&self.config.sdk_version)
                .sdk_platform(&self.config.sdk_platform)
                .id(&request.payment_id)
                .body(body.clone())
                .send()
                .await
        })
        .await
        .map_err(map_confirm_payment_error)?;

        let resp = response.into_inner();
        Ok(ConfirmPaymentResponse {
            payment_id: resp.payment_id,
            status: resp.status.into(),
        })
    }
}

fn extract_payment_id(payment_link: &str) -> String {
    payment_link.rsplit('/').next().unwrap_or(payment_link).to_string()
}

fn map_payment_options_error<T: std::fmt::Debug>(
    e: progenitor_client::Error<T>,
) -> GetPaymentOptionsError {
    let msg = format!("{:?}", e);
    let status = match &e {
        progenitor_client::Error::ErrorResponse(resp) => {
            Some(resp.status().as_u16())
        }
        progenitor_client::Error::UnexpectedResponse(resp) => {
            Some(resp.status().as_u16())
        }
        _ => None,
    };
    match status {
        Some(404) => GetPaymentOptionsError::PaymentNotFound(msg),
        Some(400) => GetPaymentOptionsError::InvalidRequest(msg),
        Some(410) => GetPaymentOptionsError::PaymentExpired(msg),
        Some(422) => GetPaymentOptionsError::InvalidAccount(msg),
        Some(451) => GetPaymentOptionsError::ComplianceFailed(msg),
        _ => GetPaymentOptionsError::Http(msg),
    }
}

fn map_confirm_payment_error<T: std::fmt::Debug>(
    e: progenitor_client::Error<T>,
) -> ConfirmPaymentError {
    let msg = format!("{:?}", e);
    let status = match &e {
        progenitor_client::Error::ErrorResponse(resp) => {
            Some(resp.status().as_u16())
        }
        progenitor_client::Error::UnexpectedResponse(resp) => {
            Some(resp.status().as_u16())
        }
        _ => None,
    };
    match status {
        Some(404) => ConfirmPaymentError::PaymentNotFound(msg),
        Some(410) => ConfirmPaymentError::PaymentExpired(msg),
        Some(400) => ConfirmPaymentError::InvalidOption(msg),
        Some(422) => ConfirmPaymentError::InvalidSignature(msg),
        Some(409) => ConfirmPaymentError::RouteExpired(msg),
        _ => ConfirmPaymentError::Http(msg),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    fn test_config() -> SdkConfig {
        SdkConfig {
            api_key: "test-api-key".to_string(),
            sdk_name: "test-sdk".to_string(),
            sdk_version: "1.0.0".to_string(),
            sdk_platform: "test".to_string(),
        }
    }

    #[tokio::test]
    async fn test_get_payment_options_success() {
        let mock_server = MockServer::start().await;

        // TODO: Replace with actual JSON response from backend
        let mock_response = serde_json::json!({
            "paymentId": "pay_123",
            "status": "requires_action",
            "amount": {
                "unit": "iso4217/USD",
                "value": "1000"
            },
            "expiresAt": 1718236800,
            "options": [
                {
                    "paymentId": "pay_123",
                    "id": "opt_1",
                    "unit": "caip19/eip155:8453/erc20:0xUSDC",
                    "value": "1000000",
                    "display": {
                        "assetSymbol": "USDC",
                        "assetName": "USD Coin",
                        "networkName": "Base",
                        "networkShort": "BASE",
                        "decimals": 6,
                        "iconUrl": "https://example.com/usdc.png"
                    },
                    "etaSeconds": 5,
                    "requiredActions": []
                }
            ],
            "dataCaptureRequirements": null
        });

        Mock::given(method("POST"))
            .and(path("/v1/gateway/payment/pay_123/options"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(&mock_response),
            )
            .mount(&mock_server)
            .await;

        let client = WalletConnectPay::new(mock_server.uri(), test_config());
        let result = client
            .get_payment_options(
                "https://pay.walletconnect.com/pay_123".to_string(),
                vec!["eip155:8453:0x123".to_string()],
            )
            .await;

        assert!(result.is_ok());
        let session = result.unwrap();
        assert_eq!(session.payment_id, "pay_123");
        assert_eq!(session.amount.unit, "iso4217/USD");
        assert_eq!(session.amount.value, "1000");
        assert_eq!(session.options.len(), 1);
        assert_eq!(session.options[0].option_id, "opt_1");
        assert_eq!(session.options[0].display.network_short, "BASE");
        assert!(session.data_capture_requirements.is_none());
    }

    #[tokio::test]
    async fn test_get_payment_options_not_found() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/gateway/payment/pay_notfound/options"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        let client = WalletConnectPay::new(mock_server.uri(), test_config());
        let result = client
            .get_payment_options("pay_notfound".to_string(), vec![
                "eip155:8453:0x123".to_string(),
            ])
            .await;

        assert!(matches!(
            result,
            Err(GetPaymentOptionsError::PaymentNotFound(_))
        ));
    }

    #[tokio::test]
    async fn test_get_payment_options_expired() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/gateway/payment/pay_expired/options"))
            .respond_with(ResponseTemplate::new(410))
            .mount(&mock_server)
            .await;

        let client = WalletConnectPay::new(mock_server.uri(), test_config());
        let result = client
            .get_payment_options("pay_expired".to_string(), vec![
                "eip155:8453:0x123".to_string(),
            ])
            .await;

        assert!(matches!(
            result,
            Err(GetPaymentOptionsError::PaymentExpired(_))
        ));
    }

    #[tokio::test]
    async fn test_extract_payment_id() {
        assert_eq!(
            extract_payment_id("https://pay.walletconnect.com/pay_123"),
            "pay_123"
        );
        assert_eq!(extract_payment_id("pay_456"), "pay_456");
        assert_eq!(
            extract_payment_id("https://example.com/path/to/pay_789"),
            "pay_789"
        );
    }

    #[tokio::test]
    async fn test_get_required_payment_actions_success() {
        let mock_server = MockServer::start().await;

        let mock_response = serde_json::json!({
            "paymentId": "pay_123",
            "status": "requires_action",
            "amount": { "unit": "iso4217/USD", "value": "1000" },
            "expiresAt": 1718236800,
            "options": [
                {
                    "paymentId": "pay_123",
                    "id": "opt_1",
                    "unit": "caip19/eip155:8453/erc20:0xUSDC",
                    "value": "1000000",
                    "display": {
                        "assetSymbol": "USDC",
                        "assetName": "USD Coin",
                        "networkName": "Base",
                        "networkShort": "BASE",
                        "decimals": 6,
                        "iconUrl": "https://example.com/usdc.png"
                    },
                    "etaSeconds": 5,
                    "requiredActions": [
                        {
                            "type": "collect-data",
                            "data": {
                                "schema": {
                                    "fields": [
                                        { "name": "firstName", "type": "string", "required": true },
                                        { "name": "dob", "type": "date", "required": true }
                                    ]
                                }
                            }
                        },
                        {
                            "type": "wallet_rpc",
                            "data": {
                                "chainId": "eip155:8453",
                                "method": "eth_signTypedData_v4",
                                "params": ["0xabc", "{\"typed\":\"data\"}"]
                            }
                        }
                    ]
                }
            ],
            "dataCaptureRequirements": null
        });

        Mock::given(method("POST"))
            .and(path("/v1/gateway/payment/pay_123/options"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(&mock_response),
            )
            .mount(&mock_server)
            .await;

        let client = WalletConnectPay::new(mock_server.uri(), test_config());

        // First, call get_payment_options to populate the cache
        let session = client
            .get_payment_options("pay_123".to_string(), vec![
                "eip155:8453:0x123".to_string(),
            ])
            .await
            .unwrap();
        assert_eq!(session.options.len(), 1);

        // Now get_required_payment_actions reads from cache (sync)
        let result = client.get_required_payment_actions(
            "opt_1".to_string(),
            "pay_123".to_string(),
            "eip155:8453:0x123".to_string(),
        );

        assert!(result.is_ok());
        let actions = result.unwrap();
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].payment_id, "pay_123");
        assert_eq!(actions[0].option_id, "opt_1");
        // First action is collect-data (matches JSON order)
        assert!(matches!(
            actions[0].action,
            RequiredAction::CollectData { .. }
        ));
        // Second action is wallet_rpc
        assert!(matches!(actions[1].action, RequiredAction::WalletRpc { .. }));

        if let RequiredAction::CollectData { data } = &actions[0].action {
            assert_eq!(data.schema.fields.len(), 2);
            assert_eq!(data.schema.fields[0].name, "firstName");
            assert!(data.schema.fields[0].required);
        }

        if let RequiredAction::WalletRpc { data } = &actions[1].action {
            assert_eq!(data.chain_id, "eip155:8453");
            assert_eq!(data.method, "eth_signTypedData_v4");
        }
    }

    #[tokio::test]
    async fn test_get_required_payment_actions_option_not_found() {
        let mock_server = MockServer::start().await;

        let mock_response = serde_json::json!({
            "paymentId": "pay_123",
            "status": "requires_action",
            "amount": { "unit": "iso4217/USD", "value": "1000" },
            "expiresAt": 1718236800,
            "options": [],
            "dataCaptureRequirements": null
        });

        Mock::given(method("POST"))
            .and(path("/v1/gateway/payment/pay_123/options"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(&mock_response),
            )
            .mount(&mock_server)
            .await;

        let client = WalletConnectPay::new(mock_server.uri(), test_config());

        // First, call get_payment_options to populate the cache (empty options)
        client
            .get_payment_options("pay_123".to_string(), vec![
                "eip155:8453:0x123".to_string(),
            ])
            .await
            .unwrap();

        // Now try to get actions for non-existent option
        let result = client.get_required_payment_actions(
            "opt_nonexistent".to_string(),
            "pay_123".to_string(),
            "eip155:8453:0x123".to_string(),
        );

        assert!(matches!(
            result,
            Err(GetPaymentRequestError::OptionNotFound(_))
        ));
    }

    #[test]
    fn test_get_required_payment_actions_without_cache() {
        let client = WalletConnectPay::new(
            "https://example.com".to_string(),
            test_config(),
        );

        // Without calling get_payment_options first, should fail
        let result = client.get_required_payment_actions(
            "opt_1".to_string(),
            "pay_123".to_string(),
            "eip155:8453:0x123".to_string(),
        );

        assert!(matches!(
            result,
            Err(GetPaymentRequestError::OptionNotFound(_))
        ));
    }

    #[tokio::test]
    async fn test_confirm_payment_success() {
        let mock_server = MockServer::start().await;

        // Mock get_payment_options to populate cache
        let options_response = serde_json::json!({
            "paymentId": "pay_123",
            "status": "requires_action",
            "amount": { "unit": "iso4217/USD", "value": "1000" },
            "expiresAt": 1718236800,
            "options": [{
                "paymentId": "pay_123",
                "id": "opt_1",
                "unit": "caip19/eip155:8453/erc20:0xUSDC",
                "value": "1000000",
                "display": {
                    "assetSymbol": "USDC",
                    "assetName": "USD Coin",
                    "networkName": "Base",
                    "networkShort": "BASE",
                    "decimals": 6,
                    "iconUrl": "https://example.com/usdc.png"
                },
                "etaSeconds": 5,
                "requiredActions": [{
                    "type": "wallet_rpc",
                    "data": {
                        "chainId": "eip155:8453",
                        "method": "eth_signTypedData_v4",
                        "params": ["0xabc"]
                    }
                }]
            }],
            "dataCaptureRequirements": null
        });

        Mock::given(method("POST"))
            .and(path("/v1/gateway/payment/pay_123/options"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(&options_response),
            )
            .mount(&mock_server)
            .await;

        // Mock confirm payment
        let confirm_response = serde_json::json!({
            "paymentId": "pay_123",
            "status": "completed"
        });

        Mock::given(method("POST"))
            .and(path("/v1/gateway/payment/pay_123/confirm"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(&confirm_response),
            )
            .mount(&mock_server)
            .await;

        let client = WalletConnectPay::new(mock_server.uri(), test_config());

        // Populate cache
        client
            .get_payment_options("pay_123".to_string(), vec![
                "eip155:8453:0x123".to_string(),
            ])
            .await
            .unwrap();

        let request = PaymentRequest {
            payment_id: "pay_123".to_string(),
            option_id: "opt_1".to_string(),
            action: RequiredAction::WalletRpc {
                data: WalletRpcAction {
                    chain_id: "eip155:8453".to_string(),
                    method: "eth_signTypedData_v4".to_string(),
                    params: vec!["0xabc".to_string()],
                },
            },
        };

        let result = PaymentResult {
            rpc_method: "eth_signTypedData_v4".to_string(),
            rpc_result: "0xsignature".to_string(),
            chain_id: "eip155:8453".to_string(),
        };

        let response = client.confirm_payment(request, result, None).await;

        assert!(response.is_ok());
        let resp = response.unwrap();
        assert_eq!(resp.payment_id, "pay_123");
        assert_eq!(resp.status, PaymentStatus::Completed);
    }

    #[tokio::test]
    async fn test_confirm_payment_collect_data_required() {
        let mock_server = MockServer::start().await;

        // Mock with collect-data required
        let options_response = serde_json::json!({
            "paymentId": "pay_123",
            "status": "requires_action",
            "amount": { "unit": "iso4217/USD", "value": "1000" },
            "expiresAt": 1718236800,
            "options": [{
                "paymentId": "pay_123",
                "id": "opt_1",
                "unit": "caip19/eip155:8453/erc20:0xUSDC",
                "value": "1000000",
                "display": {
                    "assetSymbol": "USDC",
                    "assetName": "USD Coin",
                    "networkName": "Base",
                    "networkShort": "BASE",
                    "decimals": 6,
                    "iconUrl": "https://example.com/usdc.png"
                },
                "etaSeconds": 5,
                "requiredActions": [{
                    "type": "collect-data",
                    "data": {
                        "schema": {
                            "fields": [
                                { "name": "firstName", "type": "string", "required": true }
                            ]
                        }
                    }
                }]
            }],
            "dataCaptureRequirements": null
        });

        Mock::given(method("POST"))
            .and(path("/v1/gateway/payment/pay_123/options"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(&options_response),
            )
            .mount(&mock_server)
            .await;

        let client = WalletConnectPay::new(mock_server.uri(), test_config());

        // Populate cache
        client
            .get_payment_options("pay_123".to_string(), vec![
                "eip155:8453:0x123".to_string(),
            ])
            .await
            .unwrap();

        let request = PaymentRequest {
            payment_id: "pay_123".to_string(),
            option_id: "opt_1".to_string(),
            action: RequiredAction::CollectData {
                data: CollectDataAction {
                    schema: CollectDataSchema {
                        fields: vec![CollectDataSchemaField {
                            name: "firstName".to_string(),
                            field_type: "string".to_string(),
                            required: true,
                        }],
                    },
                },
            },
        };

        let result = PaymentResult {
            rpc_method: "eth_signTypedData_v4".to_string(),
            rpc_result: "0xsignature".to_string(),
            chain_id: "eip155:8453".to_string(),
        };

        // Without KYC data, should fail
        let response = client.confirm_payment(request, result, None).await;
        assert!(matches!(
            response,
            Err(ConfirmPaymentError::CollectDataRequired)
        ));
    }
}
