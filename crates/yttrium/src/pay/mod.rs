progenitor::generate_api!(
    spec = "src/pay/openapi.json",
    interface = Builder,
    tags = Separate,
    derives = [PartialEq],
);

mod error_reporting;
mod observability;

#[cfg(any(feature = "uniffi", feature = "wasm"))]
pub mod json;

#[cfg(any(feature = "uniffi", feature = "wasm"))]
pub use json::{PayJsonError, WalletConnectPayJson};

// Logging helpers - use tracing which routes to registered logger
macro_rules! pay_debug {
    ($($arg:tt)*) => {
        tracing::debug!($($arg)*)
    };
}

macro_rules! pay_error {
    ($($arg:tt)*) => {
        tracing::error!($($arg)*)
    };
}

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum ConfigError {
    #[error("Missing authentication: {0}")]
    MissingAuth(String),
}

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum PayError {
    #[error("No network connection: {0}")]
    NoConnection(String),
    #[error("Request timed out: {0}")]
    RequestTimeout(String),
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("API error: {0}")]
    Api(String),
    #[error("Timeout: polling exceeded maximum duration")]
    Timeout,
}

impl From<progenitor_client::Error<types::ErrorResponse>> for PayError {
    fn from(e: progenitor_client::Error<types::ErrorResponse>) -> Self {
        match e {
            progenitor_client::Error::ErrorResponse(resp) => {
                let status = resp.status().as_u16();
                Self::Api(format!("{}: {}", status, resp.into_inner().message))
            }
            progenitor_client::Error::CommunicationError(err) => {
                map_reqwest_error_to_pay_error(&err)
            }
            progenitor_client::Error::InvalidRequest(msg) => Self::Api(msg),
            progenitor_client::Error::InvalidResponsePayload(_, err) => {
                Self::Api(format!("Invalid response: {}", err))
            }
            progenitor_client::Error::UnexpectedResponse(resp) => Self::Api(
                format!("{}: Unexpected response", resp.status().as_u16()),
            ),
            other => Self::Api(other.to_string()),
        }
    }
}

fn map_reqwest_error_to_pay_error(err: &reqwest::Error) -> PayError {
    let msg = err.to_string();
    let friendly = USER_FRIENDLY_NETWORK_ERROR_RETRY.to_string();
    #[cfg(not(target_arch = "wasm32"))]
    if err.is_connect() {
        let lower = msg.to_lowercase();
        if lower.contains("connection refused")
            || lower.contains("actively refused")
        {
            return PayError::ConnectionFailed(friendly);
        } else {
            return PayError::NoConnection(friendly);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    if err.is_timeout() {
        return PayError::RequestTimeout(friendly);
    }
    if looks_like_network_error(&msg) {
        return PayError::NoConnection(friendly);
    }
    PayError::Http(msg)
}

impl error_reporting::HasErrorType for PayError {
    fn error_type(&self) -> &'static str {
        match self {
            Self::NoConnection(_) => "NoConnection",
            Self::RequestTimeout(_) => "RequestTimeout",
            Self::ConnectionFailed(_) => "ConnectionFailed",
            Self::Http(_) => "Http",
            Self::Api(_) => "Api",
            Self::Timeout => "Timeout",
        }
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
    #[error("No network connection: {0}")]
    NoConnection(String),
    #[error("Request timed out: {0}")]
    RequestTimeout(String),
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl error_reporting::HasErrorType for GetPaymentOptionsError {
    fn error_type(&self) -> &'static str {
        match self {
            Self::PaymentExpired(_) => "PaymentExpired",
            Self::PaymentNotFound(_) => "PaymentNotFound",
            Self::InvalidRequest(_) => "InvalidRequest",
            Self::OptionNotFound(_) => "OptionNotFound",
            Self::PaymentNotReady(_) => "PaymentNotReady",
            Self::InvalidAccount(_) => "InvalidAccount",
            Self::ComplianceFailed(_) => "ComplianceFailed",
            Self::NoConnection(_) => "NoConnection",
            Self::RequestTimeout(_) => "RequestTimeout",
            Self::ConnectionFailed(_) => "ConnectionFailed",
            Self::Http(_) => "Http",
            Self::InternalError(_) => "InternalError",
        }
    }
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
    #[error("No network connection: {0}")]
    NoConnection(String),
    #[error("Request timed out: {0}")]
    RequestTimeout(String),
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("Fetch error: {0}")]
    FetchError(String),
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl error_reporting::HasErrorType for GetPaymentRequestError {
    fn error_type(&self) -> &'static str {
        match self {
            Self::OptionNotFound(_) => "OptionNotFound",
            Self::PaymentNotFound(_) => "PaymentNotFound",
            Self::InvalidAccount(_) => "InvalidAccount",
            Self::NoConnection(_) => "NoConnection",
            Self::RequestTimeout(_) => "RequestTimeout",
            Self::ConnectionFailed(_) => "ConnectionFailed",
            Self::Http(_) => "Http",
            Self::FetchError(_) => "FetchError",
            Self::InternalError(_) => "InternalError",
        }
    }
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
    #[error("No network connection: {0}")]
    NoConnection(String),
    #[error("Request timed out: {0}")]
    RequestTimeout(String),
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Unsupported RPC method: {0}")]
    UnsupportedMethod(String),
    #[error("Polling timeout: {0}")]
    PollingTimeout(String),
}

impl error_reporting::HasErrorType for ConfirmPaymentError {
    fn error_type(&self) -> &'static str {
        match self {
            Self::PaymentNotFound(_) => "PaymentNotFound",
            Self::PaymentExpired(_) => "PaymentExpired",
            Self::InvalidOption(_) => "InvalidOption",
            Self::InvalidSignature(_) => "InvalidSignature",
            Self::RouteExpired(_) => "RouteExpired",
            Self::NoConnection(_) => "NoConnection",
            Self::RequestTimeout(_) => "RequestTimeout",
            Self::ConnectionFailed(_) => "ConnectionFailed",
            Self::Http(_) => "Http",
            Self::InternalError(_) => "InternalError",
            Self::UnsupportedMethod(_) => "UnsupportedMethod",
            Self::PollingTimeout(_) => "PollingTimeout",
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum GetPaymentStatusError {
    #[error("Payment not found: {0}")]
    PaymentNotFound(String),
    #[error("No network connection: {0}")]
    NoConnection(String),
    #[error("Request timed out: {0}")]
    RequestTimeout(String),
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("HTTP error: {0}")]
    Http(String),
}

impl error_reporting::HasErrorType for GetPaymentStatusError {
    fn error_type(&self) -> &'static str {
        match self {
            Self::PaymentNotFound(_) => "PaymentNotFound",
            Self::NoConnection(_) => "NoConnection",
            Self::RequestTimeout(_) => "RequestTimeout",
            Self::ConnectionFailed(_) => "ConnectionFailed",
            Self::Http(_) => "Http",
        }
    }
}

const MAX_RETRIES: u32 = 3;
const INITIAL_BACKOFF_MS: u64 = 100;
const API_CONNECT_TIMEOUT_SECS: u64 = 10;
const API_REQUEST_TIMEOUT_SECS: u64 = 30;
const MAX_POLLING_DURATION_SECS: u64 = 300;

fn looks_like_network_error(msg: &str) -> bool {
    let lower = msg.to_lowercase();
    lower.contains("dns error")
        || lower.contains("failed to lookup")
        || lower.contains("name or service not known")
        || lower.contains("no such host")
        || lower.contains("connection refused")
        || lower.contains("actively refused")
        || lower.contains("network is unreachable")
        || lower.contains("network is down")
        || lower.contains("no route to host")
        || lower.contains("error sending request")
        || lower.contains("connection reset")
        || lower.contains("connection closed")
        || lower.contains("connection aborted")
        || lower.contains("broken pipe")
        || lower.contains("software caused connection abort")
        || lower.contains("socket is not connected")
        || lower.contains("operation timed out")
        || lower.contains("timed out")
}

fn is_retryable_error<T>(err: &progenitor_client::Error<T>) -> bool {
    match err {
        progenitor_client::Error::ErrorResponse(resp) => {
            resp.status().is_server_error()
        }
        #[cfg(not(target_arch = "wasm32"))]
        progenitor_client::Error::CommunicationError(reqwest_err) => {
            reqwest_err.is_connect() || reqwest_err.is_timeout()
        }
        #[cfg(target_arch = "wasm32")]
        progenitor_client::Error::CommunicationError(_) => true,
        _ => false,
    }
}

async fn with_retry<T, E, F, Fut>(
    f: F,
) -> Result<T, progenitor_client::Error<E>>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, progenitor_client::Error<E>>>,
    E: std::fmt::Debug,
{
    use rand::Rng;
    let mut attempt = 0;
    loop {
        match f().await {
            Ok(v) => return Ok(v),
            Err(e) if is_retryable_error(&e) && attempt < MAX_RETRIES => {
                attempt += 1;
                let base_backoff = INITIAL_BACKOFF_MS
                    .saturating_mul(2u64.saturating_pow(attempt - 1));
                let jitter = rand::thread_rng().gen_range(0..=base_backoff / 2);
                let backoff = base_backoff + jitter;
                pay_debug!(
                    "Retry attempt {}/{} after {}ms (error: {})",
                    attempt,
                    MAX_RETRIES,
                    backoff,
                    e
                );
                crate::time::sleep(crate::time::Duration::from_millis(backoff))
                    .await;
            }
            Err(e) => return Err(e),
        }
    }
}

// ==================== Types ====================

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct SdkConfig {
    pub base_url: String,
    pub project_id: Option<String>,
    pub sdk_name: String,
    pub sdk_version: String,
    pub sdk_platform: String,
    pub bundle_id: String,
    pub api_key: Option<String>,
    pub app_id: Option<String>,
    pub client_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    RequiresAction,
    Processing,
    Succeeded,
    Failed,
    Expired,
}

impl From<types::PaymentStatus> for PaymentStatus {
    fn from(s: types::PaymentStatus) -> Self {
        match s {
            types::PaymentStatus::RequiresAction => {
                PaymentStatus::RequiresAction
            }
            types::PaymentStatus::Processing => PaymentStatus::Processing,
            types::PaymentStatus::Succeeded => PaymentStatus::Succeeded,
            types::PaymentStatus::Failed => PaymentStatus::Failed,
            types::PaymentStatus::Expired => PaymentStatus::Expired,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct PaymentResultInfo {
    pub tx_id: String,
    pub option_amount: PayAmount,
}

impl From<types::PaymentInformation> for PaymentResultInfo {
    fn from(i: types::PaymentInformation) -> Self {
        Self { tx_id: i.tx_id, option_amount: i.option_amount.into() }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct ConfirmPaymentResultResponse {
    pub status: PaymentStatus,
    pub is_final: bool,
    pub poll_in_ms: Option<i64>,
    pub info: Option<PaymentResultInfo>,
}

impl From<types::ConfirmPaymentResponse> for ConfirmPaymentResultResponse {
    fn from(r: types::ConfirmPaymentResponse) -> Self {
        Self {
            status: r.status.into(),
            is_final: r.is_final,
            poll_in_ms: r.poll_in_ms,
            info: r.info.map(Into::into),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct PaymentStatusResponse {
    pub payment_id: String,
    pub status: PaymentStatus,
    pub is_final: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct WalletRpcAction {
    pub chain_id: String,
    pub method: String,
    pub params: String,
}

impl From<types::WalletRpcAction> for WalletRpcAction {
    fn from(a: types::WalletRpcAction) -> Self {
        Self {
            chain_id: a.chain_id,
            method: a.method.to_string(),
            params: serde_json::to_string(&a.params).unwrap_or_else(|e| {
                pay_error!("Failed to serialize WalletRpcAction params: {}", e);
                "[]".to_string()
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[serde(rename_all = "snake_case")]
pub enum CollectDataFieldType {
    Text,
    Date,
    Checkbox,
}

impl From<types::CollectDataFieldType> for CollectDataFieldType {
    fn from(t: types::CollectDataFieldType) -> Self {
        match t {
            types::CollectDataFieldType::Text => CollectDataFieldType::Text,
            types::CollectDataFieldType::Date => CollectDataFieldType::Date,
            types::CollectDataFieldType::Checkbox => {
                CollectDataFieldType::Checkbox
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct CollectDataField {
    pub id: String,
    pub name: String,
    pub required: bool,
    pub field_type: CollectDataFieldType,
}

impl From<types::CollectDataField> for CollectDataField {
    fn from(f: types::CollectDataField) -> Self {
        Self {
            id: f.id,
            name: f.name,
            required: f.required,
            field_type: f.type_.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct CollectDataAction {
    pub fields: Vec<CollectDataField>,
    pub url: Option<String>,
    pub schema: Option<String>,
}

impl From<types::CollectData> for CollectDataAction {
    fn from(c: types::CollectData) -> Self {
        Self {
            fields: c.fields.into_iter().map(Into::into).collect(),
            url: c.url,
            schema: c.schema.and_then(|s| serde_json::to_string(&s).ok()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct Action {
    pub wallet_rpc: WalletRpcAction,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct CollectDataFieldResult {
    pub id: String,
    pub value: String,
}

impl From<CollectDataFieldResult> for types::CollectDataFieldResult {
    fn from(f: CollectDataFieldResult) -> Self {
        types::CollectDataFieldResult { id: f.id, value: f.value }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct AmountDisplay {
    pub asset_symbol: String,
    pub asset_name: String,
    pub decimals: i64,
    pub icon_url: Option<String>,
    pub network_icon_url: Option<String>,
    pub network_name: Option<String>,
}

impl From<types::AmountDisplay> for AmountDisplay {
    fn from(d: types::AmountDisplay) -> Self {
        Self {
            asset_symbol: d.asset_symbol,
            asset_name: d.asset_name,
            decimals: d.decimals,
            icon_url: d.icon_url,
            network_icon_url: d.network_icon_url,
            network_name: d.network_name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct PayAmount {
    pub unit: String,
    pub value: String,
    pub display: AmountDisplay,
}

impl From<types::Amount> for PayAmount {
    fn from(a: types::Amount) -> Self {
        Self { unit: a.unit, value: a.value, display: a.display.into() }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct PaymentOption {
    pub id: String,
    pub account: String,
    pub amount: PayAmount,
    pub eta_s: i64,
    pub actions: Vec<Action>,
    pub collect_data: Option<CollectDataAction>,
}

impl From<types::PaymentOption> for PaymentOption {
    fn from(o: types::PaymentOption) -> Self {
        Self {
            id: o.id,
            account: o.account,
            amount: o.amount.into(),
            eta_s: o.eta_s,
            actions: o
                .actions
                .into_iter()
                .filter_map(|a| match a {
                    types::Action::WalletRpc(data) => {
                        Some(Action { wallet_rpc: data.into() })
                    }
                    types::Action::Build(_) => None,
                })
                .collect(),
            collect_data: o.collect_data.map(Into::into),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct MerchantInfo {
    pub name: String,
    pub icon_url: Option<String>,
}

impl From<types::MerchantInfo> for MerchantInfo {
    fn from(m: types::MerchantInfo) -> Self {
        Self { name: m.name, icon_url: m.icon_url }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct BuyerInfo {
    pub account_caip10: String,
    pub account_provider_name: String,
    pub account_provider_icon: Option<String>,
}

impl From<types::BuyerInfo> for BuyerInfo {
    fn from(b: types::BuyerInfo) -> Self {
        Self {
            account_caip10: b.account_caip10,
            account_provider_name: b.account_provider_name,
            account_provider_icon: b.account_provider_icon,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct PaymentInfo {
    pub status: PaymentStatus,
    pub amount: PayAmount,
    pub expires_at: i64,
    pub merchant: MerchantInfo,
    pub buyer: Option<BuyerInfo>,
}

impl From<types::GetPaymentResponse> for PaymentInfo {
    fn from(r: types::GetPaymentResponse) -> Self {
        Self {
            status: r.status.into(),
            amount: r.amount.into(),
            expires_at: r.expires_at,
            merchant: r.merchant.into(),
            buyer: r.buyer.map(Into::into),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct PaymentOptionsResponse {
    pub payment_id: String,
    pub info: Option<PaymentInfo>,
    pub options: Vec<PaymentOption>,
    pub collect_data: Option<CollectDataAction>,
}

// ==================== Client ====================

use {parking_lot::RwLock, std::sync::OnceLock, url::Url};

/// Applies common SDK config headers to any progenitor-generated request builder.
/// Auth header logic:
/// - If app_id is present: send App-Id + Client-Id headers (api_key ignored)
/// - If only api_key is present: send Api-Key header
macro_rules! with_sdk_config {
    ($builder:expr, $config:expr, $client_id:expr) => {{
        let mut builder = $builder
            .sdk_name(&$config.sdk_name)
            .sdk_version(&$config.sdk_version)
            .sdk_platform(&$config.sdk_platform);
        if let Some(ref app_id) = $config.app_id {
            // app_id takes precedence - use App-Id + Client-Id headers
            builder = builder.app_id(app_id);
            builder = builder.client_id($client_id);
        } else if let Some(ref api_key) = $config.api_key {
            // Only use Api-Key if app_id is not present
            builder = builder.api_key(api_key);
        }
        builder
    }};
}

#[derive(Debug, Clone)]
struct CachedPaymentOption {
    option_id: String,
    actions: Vec<types::Action>,
}

#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct WalletConnectPay {
    /// Lazily initialized API client (requires Tokio runtime)
    client: OnceLock<Client>,
    config: SdkConfig,
    cached_options: RwLock<Vec<CachedPaymentOption>>,
    /// Lazily initialized HTTP client for error reporting (requires Tokio runtime)
    error_http_client: OnceLock<reqwest::Client>,
    /// Tracks if SdkInitialized event was sent (done on first API call, not constructor)
    initialized_event_sent: OnceLock<()>,
    /// Resolved client_id (from config or auto-generated UUID)
    client_id: String,
}

impl WalletConnectPay {
    fn client(&self) -> &Client {
        self.client.get_or_init(|| {
            #[cfg(not(target_arch = "wasm32"))]
            let http = reqwest::Client::builder()
                .connect_timeout(std::time::Duration::from_secs(
                    API_CONNECT_TIMEOUT_SECS,
                ))
                .timeout(std::time::Duration::from_secs(
                    API_REQUEST_TIMEOUT_SECS,
                ))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new());
            #[cfg(target_arch = "wasm32")]
            let http = reqwest::Client::new();
            Client::new_with_client(&self.config.base_url, http)
        })
    }

    fn error_http_client(&self) -> &reqwest::Client {
        self.error_http_client.get_or_init(|| {
            let builder = reqwest::Client::builder().user_agent(format!(
                "{}/{}",
                self.config.sdk_name, self.config.sdk_version
            ));
            #[cfg(not(target_arch = "wasm32"))]
            let builder = builder.timeout(std::time::Duration::from_secs(5));
            builder.build().unwrap_or_else(|e| {
                tracing::warn!("Failed to build error HTTP client: {}", e);
                reqwest::Client::new()
            })
        })
    }

    /// Send the SdkInitialized trace event once on first API call.
    /// This is done lazily because the constructor cannot create HTTP clients
    /// (reqwest requires a Tokio runtime, which isn't available when UniFFI
    /// calls the constructor from Android's non-Tokio thread).
    fn send_initialized_event_once(&self, payment_id: &str) {
        self.initialized_event_sent.get_or_init(|| {
            self.send_trace(
                observability::TraceEvent::SdkInitialized,
                payment_id,
            );
        });
    }
}

#[cfg_attr(feature = "uniffi", uniffi::export(async_runtime = "tokio"))]
impl WalletConnectPay {
    #[cfg_attr(feature = "uniffi", uniffi::constructor)]
    pub fn new(config: SdkConfig) -> Result<Self, ConfigError> {
        // Validate: at least one of api_key or app_id must be provided
        // - app_id only: use App-Id header + app_id for error reporting
        // - api_key + app_id: use Api-Key header + app_id for error reporting
        // - api_key only: use Api-Key header
        let has_api_key = config.api_key.is_some();
        let has_app_id = config.app_id.is_some();
        if !has_api_key && !has_app_id {
            return Err(ConfigError::MissingAuth(
                "provide `api_key` and/or `app_id`".to_string(),
            ));
        }

        let client_id = config
            .client_id
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        Ok(Self {
            client: OnceLock::new(),
            config,
            cached_options: RwLock::new(Vec::new()),
            error_http_client: OnceLock::new(),
            initialized_event_sent: OnceLock::new(),
            client_id,
        })
    }

    /// Get payment options for given accounts
    /// Also caches the options for use by get_actions
    pub async fn get_payment_options(
        &self,
        payment_link: String,
        accounts: Vec<String>,
        include_payment_info: bool,
    ) -> Result<PaymentOptionsResponse, GetPaymentOptionsError> {
        pay_debug!(
            "get_payment_options: payment_link={}, accounts={:?}, include_payment_info={}",
            payment_link,
            accounts,
            include_payment_info
        );
        let payment_id = extract_payment_id(&payment_link)?;
        self.send_initialized_event_once(&payment_id);

        // Register payment environment for observability routing
        observability::set_payment_env(&payment_id, &payment_link);

        self.send_trace(
            observability::TraceEvent::PaymentOptionsRequested,
            &payment_id,
        );

        let body = types::GetPaymentOptionsRequest {
            accounts: accounts.clone(),
            refresh: None,
        };
        let response = with_retry(|| async {
            with_sdk_config!(
                self.client().gateway_get_payment_options(),
                &self.config,
                &self.client_id
            )
            .id(&payment_id)
            .include_payment_info(include_payment_info)
            .body(body.clone())
            .send()
            .await
        })
        .await
        .map_err(|e| {
            pay_error!("get_payment_options: {:?}", e);
            let err = map_payment_options_error(e);
            self.report_error(&err, &payment_id);
            self.send_trace(
                observability::TraceEvent::PaymentOptionsFailed,
                &payment_id,
            );
            err
        })?;

        let api_response = response.into_inner();
        pay_debug!(
            "get_payment_options: success, {} options",
            api_response.options.len()
        );

        // Cache the options with their raw actions
        let cached: Vec<CachedPaymentOption> = api_response
            .options
            .iter()
            .map(|o| CachedPaymentOption {
                option_id: o.id.clone(),
                actions: o.actions.clone(),
            })
            .collect();
        let mut cache = self.cached_options.write();
        *cache = cached;

        self.send_trace(
            observability::TraceEvent::PaymentOptionsReceived,
            &payment_id,
        );
        Ok(PaymentOptionsResponse {
            payment_id,
            info: api_response.info.map(Into::into),
            options: api_response.options.into_iter().map(Into::into).collect(),
            collect_data: api_response.collect_data.map(Into::into),
        })
    }

    /// Get required payment actions for a selected option
    /// Returns cached actions if available, otherwise calls fetch to get them
    /// Build action types are automatically resolved by calling the fetch endpoint
    pub async fn get_required_payment_actions(
        &self,
        payment_id: String,
        option_id: String,
    ) -> Result<Vec<Action>, GetPaymentRequestError> {
        pay_debug!(
            "get_required_payment_actions: payment_id={}, option_id={}",
            payment_id,
            option_id
        );
        self.send_initialized_event_once(&payment_id);
        self.send_trace(
            observability::TraceEvent::RequiredActionsRequested,
            &payment_id,
        );

        let raw_actions = {
            let cache = self.cached_options.read();
            cache
                .iter()
                .find(|o| o.option_id == option_id)
                .map(|o| o.actions.clone())
        };
        let raw_actions = match raw_actions {
            Some(actions) if !actions.is_empty() => {
                pay_debug!(
                    "get_required_payment_actions: using cached actions"
                );
                actions
            }
            _ => {
                pay_debug!("get_required_payment_actions: fetching actions");
                let fetched = self
                    .fetch(&payment_id, &option_id, String::new())
                    .await
                    .map_err(|e| {
                        pay_error!(
                            "get_required_payment_actions fetch: {:?}",
                            e
                        );
                        let err = map_pay_error_to_request_error(e);
                        self.report_error(&err, &payment_id);
                        self.send_trace(
                            observability::TraceEvent::RequiredActionsFailed,
                            &payment_id,
                        );
                        err
                    })?;
                let mut cache = self.cached_options.write();
                if let Some(cached) =
                    cache.iter_mut().find(|o| o.option_id == option_id)
                {
                    cached.actions = fetched.clone();
                } else {
                    cache.push(CachedPaymentOption {
                        option_id: option_id.clone(),
                        actions: fetched.clone(),
                    });
                }
                fetched
            }
        };
        let result =
            self.resolve_actions(&payment_id, &option_id, raw_actions).await;
        match &result {
            Ok(actions) => {
                pay_debug!(
                    "get_required_payment_actions: success, {} actions",
                    actions.len()
                );
                self.send_trace(
                    observability::TraceEvent::RequiredActionsReceived,
                    &payment_id,
                );
            }
            Err(e) => {
                pay_error!("get_required_payment_actions: {:?}", e);
                self.report_error(e, &payment_id);
                self.send_trace(
                    observability::TraceEvent::RequiredActionsFailed,
                    &payment_id,
                );
            }
        }
        result
    }

    /// Confirm a payment with wallet RPC signatures
    /// Polls for final status if the initial response is not final
    pub async fn confirm_payment(
        &self,
        payment_id: String,
        option_id: String,
        signatures: Vec<String>,
        collected_data: Option<Vec<CollectDataFieldResult>>,
        max_poll_ms: Option<i64>,
    ) -> Result<ConfirmPaymentResultResponse, ConfirmPaymentError> {
        pay_debug!(
            "confirm_payment: payment_id={}, option_id={}, signatures_count={}",
            payment_id,
            option_id,
            signatures.len()
        );
        self.send_initialized_event_once(&payment_id);
        self.send_trace(
            observability::TraceEvent::ConfirmPaymentCalled,
            &payment_id,
        );

        let api_results: Vec<types::ConfirmPaymentResult> = signatures
            .into_iter()
            .map(|sig| {
                types::ConfirmPaymentResult::WalletRpc(vec![
                    serde_json::Value::String(sig),
                ])
            })
            .collect();
        let api_collected_data =
            collected_data.map(|fields| types::CollectDataResult {
                fields: fields.into_iter().map(Into::into).collect(),
            });
        let body = types::ConfirmPaymentRequest {
            option_id,
            results: api_results,
            collected_data: api_collected_data,
        };
        let mut req = with_sdk_config!(
            self.client().confirm_payment_handler(),
            &self.config,
            &self.client_id
        )
        .id(&payment_id)
        .body(body.clone());
        if let Some(ms) = max_poll_ms {
            req = req.max_poll_ms(ms);
        }

        let response = with_retry(|| async { req.clone().send().await })
            .await
            .map_err(|e| {
                pay_error!("confirm_payment: {:?}", e);
                let err = map_confirm_payment_error(e);
                self.report_error(&err, &payment_id);
                self.send_trace(
                    observability::TraceEvent::ConfirmPaymentFailed,
                    &payment_id,
                );
                if is_network_error(&err) {
                    make_user_friendly_error(err)
                } else {
                    err
                }
            })?;
        let mut result: ConfirmPaymentResultResponse =
            response.into_inner().into();
        pay_debug!(
            "confirm_payment: initial status={:?}, is_final={}",
            result.status,
            result.is_final
        );
        let poll_timeout = max_poll_ms
            .filter(|&ms| ms > 0)
            .map(|ms| crate::time::Duration::from_millis(ms as u64))
            .unwrap_or(crate::time::Duration::from_secs(
                MAX_POLLING_DURATION_SECS,
            ));
        let poll_start = crate::time::Instant::now();
        while !result.is_final {
            if poll_start.elapsed() >= poll_timeout {
                let msg =
                    format!("polling exceeded {}ms", poll_timeout.as_millis());
                pay_error!("confirm_payment: {}", msg);
                let err = ConfirmPaymentError::PollingTimeout(msg);
                self.report_error(&err, &payment_id);
                self.send_trace(
                    observability::TraceEvent::ConfirmPaymentFailed,
                    &payment_id,
                );
                return Err(err);
            }
            let poll_ms = result.poll_in_ms.unwrap_or(1000);
            pay_debug!("confirm_payment: polling in {}ms", poll_ms);
            crate::time::sleep(crate::time::Duration::from_millis(
                poll_ms as u64,
            ))
            .await;
            let status = self
                .get_gateway_payment_status(payment_id.clone(), max_poll_ms)
                .await
                .map_err(|e| {
                    pay_error!("confirm_payment poll: {:?}", e);
                    let err = ConfirmPaymentError::Http(e.to_string());
                    self.report_error(&err, &payment_id);
                    self.send_trace(
                        observability::TraceEvent::ConfirmPaymentFailed,
                        &payment_id,
                    );
                    if is_network_error(&err) {
                        make_user_friendly_error(err)
                    } else {
                        err
                    }
                })?;
            result = ConfirmPaymentResultResponse {
                status: status.status.into(),
                is_final: status.is_final,
                poll_in_ms: status.poll_in_ms,
                info: status.info.map(Into::into),
            };
            pay_debug!(
                "confirm_payment: polled status={:?}, is_final={}",
                result.status,
                result.is_final
            );
        }
        pay_debug!(
            "confirm_payment: complete, final status={:?}",
            result.status
        );
        self.send_trace(
            observability::TraceEvent::ConfirmPaymentSucceeded,
            &payment_id,
        );
        Ok(result)
    }

    /// Get the current status of a payment
    /// Use this to check status after a network error during confirm_payment
    pub async fn get_payment_status(
        &self,
        payment_id: String,
    ) -> Result<PaymentStatusResponse, GetPaymentStatusError> {
        pay_debug!("get_payment_status: payment_id={}", payment_id);
        self.send_initialized_event_once(&payment_id);

        let result = self
            .get_gateway_payment_status(payment_id.clone(), None)
            .await
            .map_err(|e| {
                pay_error!("get_payment_status: {:?}", e);
                let err = map_pay_error_to_status_error(e);
                self.report_error(&err, &payment_id);
                err
            })?;

        pay_debug!(
            "get_payment_status: status={:?}, is_final={}",
            result.status,
            result.is_final
        );
        Ok(PaymentStatusResponse {
            payment_id,
            status: result.status.into(),
            is_final: result.is_final,
        })
    }
}

// Private methods (not exported via uniffi)
impl WalletConnectPay {
    fn report_error<E: std::fmt::Debug + error_reporting::HasErrorType>(
        &self,
        error: &E,
        payment_id: &str,
    ) {
        // Skip error reporting if project_id is not configured
        let Some(ref project_id) = self.config.project_id else {
            return;
        };
        error_reporting::report_error(
            self.error_http_client(),
            &self.config.bundle_id,
            project_id,
            &self.config.sdk_name,
            &self.config.sdk_version,
            error_reporting::error_type_name(error),
            payment_id,
            &format!("{:?}", error),
        );
    }

    fn send_trace(&self, event: observability::TraceEvent, payment_id: &str) {
        observability::send_trace(
            self.error_http_client(),
            &self.config.bundle_id,
            self.config.project_id.as_deref().unwrap_or(""),
            self.config.api_key.as_deref().unwrap_or(""),
            self.config.app_id.as_deref().unwrap_or(""),
            &self.client_id,
            &self.config.sdk_name,
            &self.config.sdk_version,
            &self.config.sdk_platform,
            event,
            payment_id,
        );
    }

    async fn resolve_actions(
        &self,
        payment_id: &str,
        option_id: &str,
        actions: Vec<types::Action>,
    ) -> Result<Vec<Action>, GetPaymentRequestError> {
        let mut result = Vec::new();
        for action in actions {
            match action {
                types::Action::WalletRpc(data) => {
                    result.push(Action { wallet_rpc: data.into() });
                }
                types::Action::Build(build) => {
                    let resolved = self
                        .fetch(payment_id, option_id, build.data)
                        .await
                        .map_err(map_pay_error_to_request_error)?;
                    for resolved_action in resolved {
                        if let types::Action::WalletRpc(data) = resolved_action
                        {
                            result.push(Action { wallet_rpc: data.into() });
                        }
                    }
                }
            }
        }
        Ok(result)
    }

    async fn fetch(
        &self,
        payment_id: &str,
        option_id: &str,
        data: String,
    ) -> Result<Vec<types::Action>, PayError> {
        let body =
            types::FetchRequest { option_id: option_id.to_string(), data };
        let response = with_retry(|| async {
            with_sdk_config!(
                self.client().fetch_handler(),
                &self.config,
                &self.client_id
            )
            .id(payment_id)
            .body(body.clone())
            .send()
            .await
        })
        .await?;
        Ok(response.into_inner().actions)
    }

    async fn get_gateway_payment_status(
        &self,
        payment_id: String,
        max_poll_ms: Option<i64>,
    ) -> Result<types::GetPaymentStatusResponse, PayError> {
        let mut req = with_sdk_config!(
            self.client().gateway_get_payment_status(),
            &self.config,
            &self.client_id
        )
        .id(&payment_id);
        if let Some(ms) = max_poll_ms {
            req = req.max_poll_ms(ms);
        }
        let response =
            with_retry(|| async { req.clone().send().await }).await?;
        Ok(response.into_inner())
    }
}

fn is_network_error(err: &ConfirmPaymentError) -> bool {
    match err {
        ConfirmPaymentError::NoConnection(_)
        | ConfirmPaymentError::RequestTimeout(_)
        | ConfirmPaymentError::ConnectionFailed(_) => true,
        // Also check Http errors for network-like patterns
        // (reqwest sometimes categorizes network errors as generic Http)
        ConfirmPaymentError::Http(msg) => looks_like_network_error(msg),
        _ => false,
    }
}

const USER_FRIENDLY_NETWORK_ERROR: &str =
    "No internet connection. Check your payment status with the Merchant";

const USER_FRIENDLY_NETWORK_ERROR_RETRY: &str =
    "No internet connection. Please check your connection and try again";

fn make_user_friendly_error(err: ConfirmPaymentError) -> ConfirmPaymentError {
    match err {
        ConfirmPaymentError::NoConnection(_) => {
            ConfirmPaymentError::NoConnection(
                USER_FRIENDLY_NETWORK_ERROR.into(),
            )
        }
        ConfirmPaymentError::RequestTimeout(_) => {
            ConfirmPaymentError::RequestTimeout(
                USER_FRIENDLY_NETWORK_ERROR.into(),
            )
        }
        ConfirmPaymentError::ConnectionFailed(_) => {
            ConfirmPaymentError::ConnectionFailed(
                USER_FRIENDLY_NETWORK_ERROR.into(),
            )
        }
        // Also handle Http errors that look like network issues
        ConfirmPaymentError::Http(ref msg) if looks_like_network_error(msg) => {
            ConfirmPaymentError::NoConnection(
                USER_FRIENDLY_NETWORK_ERROR.into(),
            )
        }
        other => other,
    }
}

fn extract_payment_id(
    payment_link: &str,
) -> Result<String, GetPaymentOptionsError> {
    const WC_PAY_HOST: &str = "pay.walletconnect.com";

    fn is_wc_pay_host(host: Option<&str>) -> bool {
        host.is_some_and(|h| {
            h == WC_PAY_HOST || h.ends_with(".pay.walletconnect.com")
        })
    }

    fn url_decode(s: &str) -> String {
        urlencoding::decode(s)
            .map(|c| c.into_owned())
            .unwrap_or_else(|_| s.to_string())
    }

    fn get_query_param(url: &Url, key: &str) -> Option<String> {
        url.query_pairs()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.into_owned())
            .filter(|v| !v.is_empty())
    }

    fn get_path(url: &Url) -> Option<String> {
        let path = url.path().trim_start_matches('/');
        if path.is_empty() { None } else { Some(path.to_string()) }
    }

    fn extract_from_wc_pay_url(url: &Url) -> Option<String> {
        get_query_param(url, "pid").or_else(|| get_path(url))
    }

    fn process_url(url_str: &str) -> Option<String> {
        let url = Url::parse(url_str).ok()?;

        match url.scheme() {
            "wc" => {
                let pay_url_str = get_query_param(&url, "pay")?;
                let pay_url = Url::parse(&pay_url_str).ok()?;
                if is_wc_pay_host(pay_url.host_str()) {
                    extract_from_wc_pay_url(&pay_url)
                } else {
                    Some(urlencoding::encode(&pay_url_str).into_owned())
                }
            }
            "http" | "https" => {
                if is_wc_pay_host(url.host_str()) {
                    extract_from_wc_pay_url(&url)
                } else {
                    Some(urlencoding::encode(url_str).into_owned())
                }
            }
            _ => None,
        }
    }

    if payment_link.is_empty() {
        return Err(GetPaymentOptionsError::InvalidRequest(
            "unsupported payment link format: ''".to_string(),
        ));
    }

    // Try decoded version first, then original
    let decoded = url_decode(payment_link);
    if let Some(id) = process_url(&decoded) {
        return Ok(id);
    }
    if let Some(id) = process_url(payment_link) {
        return Ok(id);
    }

    // Bare payment ID (no URL scheme)
    if !payment_link.contains('/') {
        return Ok(payment_link.to_string());
    }

    Err(GetPaymentOptionsError::InvalidRequest(format!(
        "unsupported payment link format: '{}'",
        payment_link
    )))
}

fn map_payment_options_error(
    e: progenitor_client::Error<types::ErrorResponse>,
) -> GetPaymentOptionsError {
    match e {
        progenitor_client::Error::ErrorResponse(resp) => {
            let status = resp.status().as_u16();
            let msg = format!("{}: {}", status, resp.into_inner().message);
            match status {
                404 => GetPaymentOptionsError::PaymentNotFound(msg),
                400 => GetPaymentOptionsError::InvalidRequest(msg),
                410 => GetPaymentOptionsError::PaymentExpired(msg),
                422 => GetPaymentOptionsError::InvalidAccount(msg),
                451 => GetPaymentOptionsError::ComplianceFailed(msg),
                _ => GetPaymentOptionsError::Http(msg),
            }
        }
        progenitor_client::Error::UnexpectedResponse(resp) => {
            let status = resp.status().as_u16();
            let msg = format!("{}: Unexpected response", status);
            match status {
                404 => GetPaymentOptionsError::PaymentNotFound(msg),
                400 => GetPaymentOptionsError::InvalidRequest(msg),
                410 => GetPaymentOptionsError::PaymentExpired(msg),
                422 => GetPaymentOptionsError::InvalidAccount(msg),
                451 => GetPaymentOptionsError::ComplianceFailed(msg),
                _ => GetPaymentOptionsError::Http(msg),
            }
        }
        progenitor_client::Error::CommunicationError(err) => {
            map_reqwest_error_to_payment_options_error(&err)
        }
        other => GetPaymentOptionsError::Http(other.to_string()),
    }
}

fn map_reqwest_error_to_payment_options_error(
    err: &reqwest::Error,
) -> GetPaymentOptionsError {
    let msg = err.to_string();
    let friendly = USER_FRIENDLY_NETWORK_ERROR_RETRY.to_string();
    #[cfg(not(target_arch = "wasm32"))]
    if err.is_connect() {
        let lower = msg.to_lowercase();
        if lower.contains("connection refused")
            || lower.contains("actively refused")
        {
            return GetPaymentOptionsError::ConnectionFailed(friendly);
        } else {
            return GetPaymentOptionsError::NoConnection(friendly);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    if err.is_timeout() {
        return GetPaymentOptionsError::RequestTimeout(friendly);
    }
    if looks_like_network_error(&msg) {
        return GetPaymentOptionsError::NoConnection(friendly);
    }
    GetPaymentOptionsError::Http(msg)
}

fn map_confirm_payment_error(
    e: progenitor_client::Error<types::ErrorResponse>,
) -> ConfirmPaymentError {
    match e {
        progenitor_client::Error::ErrorResponse(resp) => {
            let status = resp.status().as_u16();
            let msg = format!("{}: {}", status, resp.into_inner().message);
            match status {
                404 => ConfirmPaymentError::PaymentNotFound(msg),
                410 => ConfirmPaymentError::PaymentExpired(msg),
                400 => ConfirmPaymentError::InvalidOption(msg),
                422 => ConfirmPaymentError::InvalidSignature(msg),
                409 => ConfirmPaymentError::RouteExpired(msg),
                _ => ConfirmPaymentError::Http(msg),
            }
        }
        progenitor_client::Error::UnexpectedResponse(resp) => {
            let status = resp.status().as_u16();
            let msg = format!("{}: Unexpected response", status);
            match status {
                404 => ConfirmPaymentError::PaymentNotFound(msg),
                410 => ConfirmPaymentError::PaymentExpired(msg),
                400 => ConfirmPaymentError::InvalidOption(msg),
                422 => ConfirmPaymentError::InvalidSignature(msg),
                409 => ConfirmPaymentError::RouteExpired(msg),
                _ => ConfirmPaymentError::Http(msg),
            }
        }
        progenitor_client::Error::CommunicationError(err) => {
            map_reqwest_error_to_confirm_payment_error(&err)
        }
        other => ConfirmPaymentError::Http(other.to_string()),
    }
}

fn map_reqwest_error_to_confirm_payment_error(
    err: &reqwest::Error,
) -> ConfirmPaymentError {
    let msg = err.to_string();
    let friendly = USER_FRIENDLY_NETWORK_ERROR.to_string();
    #[cfg(not(target_arch = "wasm32"))]
    if err.is_connect() {
        let lower = msg.to_lowercase();
        if lower.contains("connection refused")
            || lower.contains("actively refused")
        {
            return ConfirmPaymentError::ConnectionFailed(friendly);
        } else {
            return ConfirmPaymentError::NoConnection(friendly);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    if err.is_timeout() {
        return ConfirmPaymentError::RequestTimeout(friendly);
    }
    if looks_like_network_error(&msg) {
        return ConfirmPaymentError::NoConnection(friendly);
    }
    ConfirmPaymentError::Http(msg)
}

fn map_pay_error_to_request_error(e: PayError) -> GetPaymentRequestError {
    match e {
        PayError::NoConnection(msg) => {
            GetPaymentRequestError::NoConnection(msg)
        }
        PayError::RequestTimeout(msg) => {
            GetPaymentRequestError::RequestTimeout(msg)
        }
        PayError::ConnectionFailed(msg) => {
            GetPaymentRequestError::ConnectionFailed(msg)
        }
        PayError::Http(msg) => GetPaymentRequestError::Http(msg),
        PayError::Api(msg) => GetPaymentRequestError::FetchError(msg),
        PayError::Timeout => {
            GetPaymentRequestError::FetchError("Timeout".to_string())
        }
    }
}

fn map_pay_error_to_status_error(e: PayError) -> GetPaymentStatusError {
    match e {
        PayError::NoConnection(msg) => GetPaymentStatusError::NoConnection(msg),
        PayError::RequestTimeout(msg) => {
            GetPaymentStatusError::RequestTimeout(msg)
        }
        PayError::ConnectionFailed(msg) => {
            GetPaymentStatusError::ConnectionFailed(msg)
        }
        PayError::Http(msg) => GetPaymentStatusError::Http(msg),
        PayError::Api(msg) => {
            if msg.starts_with("404:") {
                GetPaymentStatusError::PaymentNotFound(msg)
            } else {
                GetPaymentStatusError::Http(msg)
            }
        }
        PayError::Timeout => {
            GetPaymentStatusError::RequestTimeout("Request timed out".into())
        }
    }
}

#[cfg(test)]
#[cfg(feature = "test_pay_api")]
mod e2e_tests;

#[cfg(test)]
mod tests {
    use {
        super::*,
        wiremock::{
            Mock, MockServer, ResponseTemplate,
            matchers::{header, method, path},
        },
    };

    fn test_config(base_url: String) -> SdkConfig {
        SdkConfig {
            base_url,
            project_id: Some("test-project-id".to_string()),
            sdk_name: "test-sdk".to_string(),
            sdk_version: "1.0.0".to_string(),
            sdk_platform: "test".to_string(),
            bundle_id: "com.test.app".to_string(),
            api_key: Some("test-api-key".to_string()),
            app_id: None,
            client_id: None,
        }
    }

    #[test]
    fn test_config_missing_auth() {
        let config = SdkConfig {
            base_url: "http://example.com".to_string(),
            project_id: Some("test".to_string()),
            sdk_name: "test".to_string(),
            sdk_version: "1.0.0".to_string(),
            sdk_platform: "test".to_string(),
            bundle_id: "com.test".to_string(),
            api_key: None,
            app_id: None,
            client_id: None,
        };
        let result = WalletConnectPay::new(config);
        assert!(matches!(result, Err(ConfigError::MissingAuth(_))));
    }

    #[test]
    fn test_config_valid_both_api_key_and_app_id() {
        // Both api_key and app_id can be provided:
        // - api_key is used for API auth headers
        // - app_id is used for error reporting
        let config = SdkConfig {
            base_url: "http://example.com".to_string(),
            project_id: Some("test".to_string()),
            sdk_name: "test".to_string(),
            sdk_version: "1.0.0".to_string(),
            sdk_platform: "test".to_string(),
            bundle_id: "com.test".to_string(),
            api_key: Some("key".to_string()),
            app_id: Some("app".to_string()),
            client_id: None,
        };
        assert!(WalletConnectPay::new(config).is_ok());
    }

    #[test]
    fn test_config_valid_api_key_auth() {
        let config = SdkConfig {
            base_url: "http://example.com".to_string(),
            project_id: Some("test".to_string()),
            sdk_name: "test".to_string(),
            sdk_version: "1.0.0".to_string(),
            sdk_platform: "test".to_string(),
            bundle_id: "com.test".to_string(),
            api_key: Some("key".to_string()),
            app_id: None,
            client_id: None,
        };
        assert!(WalletConnectPay::new(config).is_ok());
    }

    #[test]
    fn test_config_valid_app_id_auth() {
        let config = SdkConfig {
            base_url: "http://example.com".to_string(),
            project_id: Some("test".to_string()),
            sdk_name: "test".to_string(),
            sdk_version: "1.0.0".to_string(),
            sdk_platform: "test".to_string(),
            bundle_id: "com.test".to_string(),
            api_key: None,
            app_id: Some("app".to_string()),
            client_id: Some("client".to_string()),
        };
        assert!(WalletConnectPay::new(config).is_ok());
    }

    #[test]
    fn test_config_empty_client_id_generates_uuid() {
        let config = SdkConfig {
            base_url: "http://example.com".to_string(),
            project_id: Some("test".to_string()),
            sdk_name: "test".to_string(),
            sdk_version: "1.0.0".to_string(),
            sdk_platform: "test".to_string(),
            bundle_id: "com.test".to_string(),
            api_key: Some("key".to_string()),
            app_id: None,
            client_id: Some("".to_string()),
        };
        let client = WalletConnectPay::new(config).unwrap();
        // Empty string should be treated as None, generating a UUID
        assert!(!client.client_id.is_empty());
        // Should be a valid UUID format (36 chars with hyphens)
        assert_eq!(client.client_id.len(), 36);
    }

    #[tokio::test]
    async fn test_get_payment_options_success() {
        let mock_server = MockServer::start().await;

        let mock_response = serde_json::json!({
            "options": [
                {
                    "id": "opt_1",
                    "account": "eip155:8453:0x123",
                    "amount": {
                        "unit": "caip19/eip155:8453/erc20:0xUSDC",
                        "value": "1000000",
                        "display": {
                            "assetSymbol": "USDC",
                            "assetName": "USD Coin",
                            "decimals": 6,
                            "iconUrl": "https://example.com/usdc.png",
                            "networkName": "Base"
                        }
                    },
                    "etaS": 5,
                    "actions": [],
                    "collectData": {
                        "fields": [
                            {
                                "type": "text",
                                "id": "fullName",
                                "name": "Full Name",
                                "required": true
                            }
                        ],
                        "url": "https://data-collection.example.com/ic/pay_123",
                        "schema": {"type": "object"}
                    }
                }
            ]
        });

        Mock::given(method("POST"))
            .and(path("/v1/gateway/payment/pay_123/options"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(&mock_response),
            )
            .mount(&mock_server)
            .await;

        let client =
            WalletConnectPay::new(test_config(mock_server.uri())).unwrap();
        let result = client
            .get_payment_options(
                "https://pay.walletconnect.com/pay_123".to_string(),
                vec!["eip155:8453:0x123".to_string()],
                false,
            )
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.options.len(), 1);
        assert_eq!(response.options[0].id, "opt_1");
        assert_eq!(
            response.options[0].amount.unit,
            "caip19/eip155:8453/erc20:0xUSDC"
        );
        assert_eq!(
            response.options[0].amount.display.network_name,
            Some("Base".to_string())
        );
        let opt_cd =
            response.options[0].collect_data.as_ref().expect("collect_data");
        assert_eq!(opt_cd.fields.len(), 1);
        assert_eq!(opt_cd.fields[0].id, "fullName");
        assert_eq!(opt_cd.fields[0].field_type, CollectDataFieldType::Text);
        assert_eq!(
            opt_cd.url,
            Some("https://data-collection.example.com/ic/pay_123".to_string())
        );
        assert!(opt_cd.schema.is_some());
    }

    #[tokio::test]
    async fn test_get_payment_options_not_found() {
        let mock_server = MockServer::start().await;
        let error_response = serde_json::json!({
            "code": "payment_not_found",
            "message": "Payment not found"
        });
        Mock::given(method("POST"))
            .and(path("/v1/gateway/payment/pay_notfound/options"))
            .respond_with(
                ResponseTemplate::new(404).set_body_json(&error_response),
            )
            .mount(&mock_server)
            .await;

        let client =
            WalletConnectPay::new(test_config(mock_server.uri())).unwrap();
        let result = client
            .get_payment_options(
                "pay_notfound".to_string(),
                vec!["eip155:8453:0x123".to_string()],
                false,
            )
            .await;

        assert!(matches!(
            result,
            Err(GetPaymentOptionsError::PaymentNotFound(_))
        ));
    }

    #[tokio::test]
    async fn test_get_payment_options_expired() {
        let mock_server = MockServer::start().await;
        let error_response = serde_json::json!({
            "code": "invalid_state",
            "message": "Payment expired"
        });
        Mock::given(method("POST"))
            .and(path("/v1/gateway/payment/pay_expired/options"))
            .respond_with(
                ResponseTemplate::new(410).set_body_json(&error_response),
            )
            .mount(&mock_server)
            .await;

        let client =
            WalletConnectPay::new(test_config(mock_server.uri())).unwrap();
        let result = client
            .get_payment_options(
                "pay_expired".to_string(),
                vec!["eip155:8453:0x123".to_string()],
                false,
            )
            .await;

        assert!(matches!(
            result,
            Err(GetPaymentOptionsError::PaymentExpired(_))
        ));
    }

    #[tokio::test]
    async fn test_extract_payment_id() {
        // pay.walletconnect.com with path segment
        assert_eq!(
            extract_payment_id("https://pay.walletconnect.com/pay_123")
                .unwrap(),
            "pay_123"
        );
        // pay.walletconnect.com with full path (multiple segments)
        assert_eq!(
            extract_payment_id("http://pay.walletconnect.com/abc/123").unwrap(),
            "abc/123"
        );
        // Bare payment ID
        assert_eq!(extract_payment_id("pay_456").unwrap(), "pay_456");
        // Non-WC URL should be URL-encoded as the payment ID
        assert_eq!(
            extract_payment_id("https://example.com/path/to/pay_789").unwrap(),
            "https%3A%2F%2Fexample.com%2Fpath%2Fto%2Fpay_789"
        );
        assert!(extract_payment_id("").is_err());
    }

    #[tokio::test]
    async fn test_extract_payment_id_short_url() {
        // pay.wct.me/123 should be URL-encoded as the payment ID
        assert_eq!(
            extract_payment_id("https://pay.wct.me/123").unwrap(),
            "https%3A%2F%2Fpay.wct.me%2F123"
        );
        assert_eq!(
            extract_payment_id("https://pay.wct.me/abc_xyz").unwrap(),
            "https%3A%2F%2Fpay.wct.me%2Fabc_xyz"
        );
    }

    #[tokio::test]
    async fn test_extract_payment_id_wc_uri_with_short_url() {
        // wc: URI with pay.wct.me URL should URL-encode that URL
        assert_eq!(
            extract_payment_id(
                "wc:abc123@2?relay-protocol=irn&symKey=xyz&\
                 pay=https%3A%2F%2Fpay.wct.me%2F123"
            )
            .unwrap(),
            "https%3A%2F%2Fpay.wct.me%2F123"
        );
    }

    #[tokio::test]
    async fn test_extract_payment_id_pid_takes_precedence_over_path() {
        // pid query param should be preferred over path segment
        assert_eq!(
            extract_payment_id(
                "https://pay.walletconnect.com/path_id?pid=query_id"
            )
            .unwrap(),
            "query_id"
        );
    }

    #[tokio::test]
    async fn test_extract_payment_id_wc_pay_trailing_slash() {
        // Trailing slash with no path segment but has pid
        assert_eq!(
            extract_payment_id("https://pay.walletconnect.com/?pid=pay_123")
                .unwrap(),
            "pay_123"
        );
    }

    #[tokio::test]
    async fn test_extract_payment_id_wc_pay_other_query_params() {
        // Query params without pid should fall back to path
        assert_eq!(
            extract_payment_id("https://pay.walletconnect.com/pay_123?foo=bar")
                .unwrap(),
            "pay_123"
        );
    }

    #[tokio::test]
    async fn test_extract_payment_id_http_urls() {
        // HTTP (non-HTTPS) should also work
        assert_eq!(
            extract_payment_id("http://pay.walletconnect.com/pay_123").unwrap(),
            "pay_123"
        );
        assert_eq!(
            extract_payment_id("http://pay.wct.me/123").unwrap(),
            "http%3A%2F%2Fpay.wct.me%2F123"
        );
    }

    #[tokio::test]
    async fn test_extract_payment_id_with_pid_query_param() {
        assert_eq!(
            extract_payment_id("https://pay.walletconnect.com/?pid=pay_95a2ecc101KEYG1NH580DF2J04MBNRWE7V").unwrap(),
            "pay_95a2ecc101KEYG1NH580DF2J04MBNRWE7V"
        );
    }

    #[tokio::test]
    async fn test_extract_payment_id_url_encoded() {
        assert_eq!(
            extract_payment_id("https%3A%2F%2Fpay.walletconnect.com%2F%3Fpid%3Dpay_95a2ecc101KEYG1NH580DF2J04MBNRWE7V").unwrap(),
            "pay_95a2ecc101KEYG1NH580DF2J04MBNRWE7V"
        );
    }

    #[tokio::test]
    async fn test_extract_payment_id_wc_uri_with_pay_param() {
        assert_eq!(
            extract_payment_id("wc:abc123@2?relay-protocol=irn&symKey=xyz&pay=https%3A%2F%2Fpay.walletconnect.com%2F%3Fpid%3Dpay_03a2ecc101KEVQWPKPJ3TP47E1PBKSSV5Y").unwrap(),
            "pay_03a2ecc101KEVQWPKPJ3TP47E1PBKSSV5Y"
        );
    }

    #[tokio::test]
    async fn test_extract_payment_id_wc_uri_with_staging_pay_url() {
        assert_eq!(
            extract_payment_id("wc:c4ef1cc525b890c9c46ed40656e9048452d1d3eafe7c77ac963255cf372bcc51@2?expiryTimestamp=1768825874&relay-protocol=irn&symKey=b0487747501cd883edf2e08f33d802cd42d77631a3c9633c18b001507cfcbdc9&pay=https%3A%2F%2Fstaging.pay.walletconnect.com%2F%3Fpid%3Dpay_95a2ecc101KFB3B5W45JBD6CH1KJPCW9T1").unwrap(),
            "pay_95a2ecc101KFB3B5W45JBD6CH1KJPCW9T1"
        );
    }

    #[tokio::test]
    async fn test_extract_payment_id_subdomain_direct_urls() {
        assert_eq!(
            extract_payment_id(
                "https://staging.pay.walletconnect.com/?pid=pay_123"
            )
            .unwrap(),
            "pay_123"
        );
        assert_eq!(
            extract_payment_id(
                "https://dev.pay.walletconnect.com/?pid=pay_456"
            )
            .unwrap(),
            "pay_456"
        );
    }

    #[tokio::test]
    async fn test_extract_payment_id_rejects_non_subdomain_urls() {
        // Domains that don't end with .pay.walletconnect.com should be URL-encoded
        assert_eq!(
            extract_payment_id(
                "https://fakepay.walletconnect.com/?pid=pay_123"
            )
            .unwrap(),
            "https%3A%2F%2Ffakepay.walletconnect.com%2F%3Fpid%3Dpay_123"
        );
        assert_eq!(
            extract_payment_id(
                "https://evil.pay.walletconnect.com.attacker.com/?pid=pay_123"
            )
            .unwrap(),
            "https%3A%2F%2Fevil.pay.walletconnect.com.attacker.com%2F%3Fpid%3Dpay_123"
        );
    }

    #[tokio::test]
    async fn test_extract_payment_id_fully_encoded_wc_uri() {
        assert_eq!(
            extract_payment_id("wc%3Afe8314404caac9b487daca1bb3ba4076f1ef0a52794c6117bd944ed17c5b32a7%402%3FexpiryTimestamp%3D1768460398%26relay-protocol%3Dirn%26symKey%3Db614f88cbae08c993154c9c4c23cd538ca17600d0ee7666d52fb8494f6c98b70%26pay%3Dhttps%3A%2F%2Fpay.walletconnect.com%2F%3Fpid%3Dpay_95a2ecc101KEYG1NH580DF2J04MBNRWE7V").unwrap(),
            "pay_95a2ecc101KEYG1NH580DF2J04MBNRWE7V"
        );
    }

    #[tokio::test]
    async fn test_get_required_payment_actions_success() {
        let mock_server = MockServer::start().await;

        let mock_response = serde_json::json!({
            "options": [
                {
                    "id": "opt_1",
                    "account": "eip155:8453:0x123",
                    "amount": {
                        "unit": "caip19/eip155:8453/erc20:0xUSDC",
                        "value": "1000000",
                        "display": {
                            "assetSymbol": "USDC",
                            "assetName": "USD Coin",
                            "decimals": 6,
                            "iconUrl": "https://example.com/usdc.png",
                            "networkName": "Base"
                        }
                    },
                    "etaS": 5,
                    "actions": [
                        {
                            "type": "walletRpc",
                            "data": {
                                "chain_id": "eip155:8453",
                                "method": "eth_signTypedData_v4",
                                "params": ["0xabc", "{\"typed\":\"data\"}"]
                            }
                        }
                    ]
                }
            ]
        });

        Mock::given(method("POST"))
            .and(path("/v1/gateway/payment/pay_123/options"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(&mock_response),
            )
            .mount(&mock_server)
            .await;

        let client =
            WalletConnectPay::new(test_config(mock_server.uri())).unwrap();
        let response = client
            .get_payment_options(
                "pay_123".to_string(),
                vec!["eip155:8453:0x123".to_string()],
                false,
            )
            .await
            .unwrap();
        assert_eq!(response.options.len(), 1);

        let actions = client
            .get_required_payment_actions(
                "pay_123".to_string(),
                "opt_1".to_string(),
            )
            .await
            .unwrap();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].wallet_rpc.chain_id, "eip155:8453");
        assert_eq!(actions[0].wallet_rpc.method, "eth_signTypedData_v4");
    }

    #[tokio::test]
    async fn test_get_required_payment_actions_resolves_build() {
        let mock_server = MockServer::start().await;

        let mock_response = serde_json::json!({
            "options": [
                {
                    "id": "opt_1",
                    "account": "eip155:8453:0x123",
                    "amount": {
                        "unit": "caip19/eip155:8453/erc20:0xUSDC",
                        "value": "1000000",
                        "display": {
                            "assetSymbol": "USDC",
                            "assetName": "USD Coin",
                            "decimals": 6,
                            "iconUrl": "https://example.com/usdc.png",
                            "networkName": "Base"
                        }
                    },
                    "etaS": 5,
                    "actions": [
                        {
                            "type": "build",
                            "data": { "data": "some_data" }
                        }
                    ]
                }
            ]
        });

        let fetch_response = serde_json::json!({
            "actions": [{
                "type": "walletRpc",
                "data": {
                    "chain_id": "eip155:8453",
                    "method": "eth_signTypedData_v4",
                    "params": ["0xresolved", {"resolved": "data"}]
                }
            }]
        });

        Mock::given(method("POST"))
            .and(path("/v1/gateway/payment/pay_123/options"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(&mock_response),
            )
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/v1/gateway/payment/pay_123/fetch"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(&fetch_response),
            )
            .mount(&mock_server)
            .await;

        let client =
            WalletConnectPay::new(test_config(mock_server.uri())).unwrap();
        client
            .get_payment_options(
                "pay_123".to_string(),
                vec!["eip155:8453:0x123".to_string()],
                false,
            )
            .await
            .unwrap();

        let actions = client
            .get_required_payment_actions(
                "pay_123".to_string(),
                "opt_1".to_string(),
            )
            .await
            .unwrap();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].wallet_rpc.chain_id, "eip155:8453");
        assert!(actions[0].wallet_rpc.params.contains("resolved"));
    }

    #[tokio::test]
    async fn test_get_required_payment_actions_fetches_when_not_cached() {
        let mock_server = MockServer::start().await;

        let fetch_response = serde_json::json!({
            "actions": [{
                "type": "walletRpc",
                "data": {
                    "chain_id": "eip155:1",
                    "method": "eth_signTypedData_v4",
                    "params": ["0x123", {"types": {}}]
                }
            }]
        });

        Mock::given(method("POST"))
            .and(path("/v1/gateway/payment/pay_456/fetch"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(&fetch_response),
            )
            .mount(&mock_server)
            .await;

        let client =
            WalletConnectPay::new(test_config(mock_server.uri())).unwrap();
        // Call without populating cache first - should call fetch
        let actions = client
            .get_required_payment_actions(
                "pay_456".to_string(),
                "opt_new".to_string(),
            )
            .await
            .unwrap();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].wallet_rpc.chain_id, "eip155:1");
    }

    #[tokio::test]
    async fn test_get_required_payment_actions_fetch_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/gateway/payment/pay_789/fetch"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let client =
            WalletConnectPay::new(test_config(mock_server.uri())).unwrap();
        let result = client
            .get_required_payment_actions(
                "pay_789".to_string(),
                "opt_missing".to_string(),
            )
            .await;
        assert!(matches!(result, Err(GetPaymentRequestError::FetchError(_))));
    }

    #[tokio::test]
    async fn test_confirm_payment_success() {
        let mock_server = MockServer::start().await;

        let confirm_response = serde_json::json!({
            "status": "succeeded",
            "isFinal": true,
            "pollInMs": null
        });

        Mock::given(method("POST"))
            .and(path("/v1/gateway/payment/pay_123/confirm"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(&confirm_response),
            )
            .mount(&mock_server)
            .await;

        let client =
            WalletConnectPay::new(test_config(mock_server.uri())).unwrap();
        let response = client
            .confirm_payment(
                "pay_123".to_string(),
                "opt_1".to_string(),
                vec!["0x123".to_string()],
                None,
                None,
            )
            .await;
        assert!(response.is_ok());
        let resp = response.unwrap();
        assert_eq!(resp.status, PaymentStatus::Succeeded);
        assert!(resp.is_final);
    }

    #[tokio::test]
    async fn test_confirm_payment_polls_until_final() {
        let mock_server = MockServer::start().await;
        let confirm_response = serde_json::json!({
            "status": "processing",
            "isFinal": false,
            "pollInMs": 10
        });
        let status_response = serde_json::json!({
            "status": "succeeded",
            "isFinal": true
        });
        Mock::given(method("POST"))
            .and(path("/v1/gateway/payment/pay_123/confirm"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(&confirm_response),
            )
            .mount(&mock_server)
            .await;
        Mock::given(method("GET"))
            .and(path("/v1/gateway/payment/pay_123/status"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(&status_response),
            )
            .mount(&mock_server)
            .await;
        let client =
            WalletConnectPay::new(test_config(mock_server.uri())).unwrap();
        let response = client
            .confirm_payment(
                "pay_123".to_string(),
                "opt_1".to_string(),
                vec!["0x123".to_string()],
                None,
                Some(5000),
            )
            .await;
        assert!(response.is_ok());
        let resp = response.unwrap();
        assert_eq!(resp.status, PaymentStatus::Succeeded);
        assert!(resp.is_final);
    }

    #[tokio::test]
    async fn test_sdk_headers_are_set() {
        let mock_server = MockServer::start().await;

        let mock_response = serde_json::json!({
            "options": []
        });

        Mock::given(method("POST"))
            .and(path("/v1/gateway/payment/pay_headers/options"))
            .and(header("Api-Key", "test-api-key"))
            .and(header("Sdk-Name", "test-sdk"))
            .and(header("Sdk-Version", "1.0.0"))
            .and(header("Sdk-Platform", "test"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(&mock_response),
            )
            .expect(1)
            .mount(&mock_server)
            .await;

        let client =
            WalletConnectPay::new(test_config(mock_server.uri())).unwrap();
        let result = client
            .get_payment_options(
                "pay_headers".to_string(),
                vec!["eip155:1:0x123".to_string()],
                false,
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_custom_sdk_config_headers() {
        let mock_server = MockServer::start().await;

        let mock_response = serde_json::json!({
            "status": "succeeded",
            "isFinal": true,
            "pollInMs": null
        });

        let custom_config = SdkConfig {
            base_url: mock_server.uri(),
            project_id: Some("my-custom-project-id".to_string()),
            sdk_name: "my-app".to_string(),
            sdk_version: "2.5.0".to_string(),
            sdk_platform: "ios".to_string(),
            bundle_id: "com.custom.app".to_string(),
            api_key: None,
            app_id: Some("custom-app-id".to_string()),
            client_id: Some("custom-client-id".to_string()),
        };

        Mock::given(method("POST"))
            .and(path("/v1/gateway/payment/pay_custom/confirm"))
            .and(header("App-Id", "custom-app-id"))
            .and(header("Client-Id", "custom-client-id"))
            .and(header("Sdk-Name", "my-app"))
            .and(header("Sdk-Version", "2.5.0"))
            .and(header("Sdk-Platform", "ios"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(&mock_response),
            )
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = WalletConnectPay::new(custom_config).unwrap();
        let result = client
            .confirm_payment(
                "pay_custom".to_string(),
                "opt_1".to_string(),
                vec!["0x123".to_string()],
                None,
                None,
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_app_id_takes_precedence_over_api_key() {
        // When both api_key and app_id are provided, only App-Id + Client-Id
        // headers are sent (api_key is ignored for headers)
        let mock_server = MockServer::start().await;

        let mock_response = serde_json::json!({
            "status": "succeeded",
            "isFinal": true,
            "pollInMs": null
        });

        let config = SdkConfig {
            base_url: mock_server.uri(),
            project_id: Some("test-project".to_string()),
            sdk_name: "test-sdk".to_string(),
            sdk_version: "1.0.0".to_string(),
            sdk_platform: "test".to_string(),
            bundle_id: "com.test.app".to_string(),
            api_key: Some("my-api-key".to_string()),
            app_id: Some("my-app-id".to_string()),
            client_id: Some("my-client-id".to_string()),
        };

        // Expect App-Id + Client-Id headers only (NOT Api-Key)
        Mock::given(method("POST"))
            .and(path("/v1/gateway/payment/pay_test/confirm"))
            .and(header("App-Id", "my-app-id"))
            .and(header("Client-Id", "my-client-id"))
            .and(header("Sdk-Name", "test-sdk"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(&mock_response),
            )
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = WalletConnectPay::new(config).unwrap();
        let result = client
            .confirm_payment(
                "pay_test".to_string(),
                "opt_1".to_string(),
                vec!["0x123".to_string()],
                None,
                None,
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_collect_data_response() {
        let mock_server = MockServer::start().await;
        let mock_response = serde_json::json!({
            "options": [
                {
                    "id": "opt_1",
                    "account": "eip155:8453:0x123",
                    "amount": {
                        "unit": "caip19/eip155:8453/erc20:0xUSDC",
                        "value": "1000000",
                        "display": {
                            "assetSymbol": "USDC",
                            "assetName": "USD Coin",
                            "decimals": 6
                        }
                    },
                    "etaS": 5,
                    "actions": []
                }
            ],
            "collectData": {
                "fields": [
                    {
                        "type": "text",
                        "id": "firstName",
                        "name": "First Name",
                        "required": true
                    },
                    {
                        "type": "date",
                        "id": "dob",
                        "name": "Date of Birth",
                        "required": false
                    }
                ],
                "url": "https://data-collection.example.com/ic/pay_123",
                "schema": {
                    "type": "object",
                    "properties": {
                        "firstName": {"type": "string"}
                    }
                }
            }
        });
        Mock::given(method("POST"))
            .and(path("/v1/gateway/payment/pay_123/options"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(&mock_response),
            )
            .mount(&mock_server)
            .await;

        let client =
            WalletConnectPay::new(test_config(mock_server.uri())).unwrap();
        let response = client
            .get_payment_options(
                "pay_123".to_string(),
                vec!["eip155:8453:0x123".to_string()],
                false,
            )
            .await
            .unwrap();

        let data = response.collect_data.expect("Expected collect_data");
        assert_eq!(data.fields.len(), 2);
        assert_eq!(data.fields[0].id, "firstName");
        assert_eq!(data.fields[0].field_type, CollectDataFieldType::Text);
        assert!(data.fields[0].required);
        assert_eq!(data.fields[1].id, "dob");
        assert_eq!(data.fields[1].field_type, CollectDataFieldType::Date);
        assert!(!data.fields[1].required);
        assert_eq!(
            data.url,
            Some("https://data-collection.example.com/ic/pay_123".to_string())
        );
        assert!(data.schema.is_some());
        assert!(response.options[0].collect_data.is_none());
    }

    #[tokio::test]
    async fn test_initialized_event_sent_once_with_first_payment_id() {
        let mock_server = MockServer::start().await;
        let mock_response = serde_json::json!({
            "options": []
        });
        Mock::given(method("POST"))
            .and(path("/v1/gateway/payment/pay_first/options"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(&mock_response),
            )
            .mount(&mock_server)
            .await;
        Mock::given(method("POST"))
            .and(path("/v1/gateway/payment/pay_second/options"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(&mock_response),
            )
            .mount(&mock_server)
            .await;

        let client =
            WalletConnectPay::new(test_config(mock_server.uri())).unwrap();
        assert!(client.initialized_event_sent.get().is_none());

        client
            .get_payment_options(
                "pay_first".to_string(),
                vec!["eip155:1:0x123".to_string()],
                false,
            )
            .await
            .unwrap();
        assert!(client.initialized_event_sent.get().is_some());

        client
            .get_payment_options(
                "pay_second".to_string(),
                vec!["eip155:1:0x123".to_string()],
                false,
            )
            .await
            .unwrap();
        assert!(client.initialized_event_sent.get().is_some());
    }

    #[tokio::test]
    async fn test_get_payment_options_connection_error() {
        // Use a port that's definitely not listening
        // This tests that connection errors are properly categorized
        let client = WalletConnectPay::new(test_config(
            "http://127.0.0.1:54321".to_string(),
        ))
        .unwrap();

        let result = client
            .get_payment_options(
                "pay_123".to_string(),
                vec!["eip155:8453:0x123".to_string()],
                false,
            )
            .await;

        // Connection errors to closed ports map to NoConnection or
        // ConnectionFailed depending on the OS error message
        assert!(
            matches!(
                result,
                Err(GetPaymentOptionsError::NoConnection(_))
                    | Err(GetPaymentOptionsError::ConnectionFailed(_))
            ),
            "Expected NoConnection or ConnectionFailed, got {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_get_payment_options_no_connection_dns_failure() {
        // Use a hostname that will fail DNS resolution
        // .invalid is a reserved TLD that should never resolve
        let client = WalletConnectPay::new(test_config(
            "http://nonexistent.invalid:8080".to_string(),
        ))
        .unwrap();

        let result = client
            .get_payment_options(
                "pay_123".to_string(),
                vec!["eip155:8453:0x123".to_string()],
                false,
            )
            .await;

        // DNS failure should map to NoConnection
        assert!(
            matches!(result, Err(GetPaymentOptionsError::NoConnection(_))),
            "Expected NoConnection, got {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_get_required_payment_actions_connection_error() {
        // Test that connection errors are properly categorized
        // by pointing to a non-listening port
        let client = WalletConnectPay::new(test_config(
            "http://127.0.0.1:54322".to_string(),
        ))
        .unwrap();

        let result = client
            .get_required_payment_actions(
                "pay_123".to_string(),
                "opt_1".to_string(),
            )
            .await;

        // Connection errors map to NoConnection or ConnectionFailed
        assert!(
            matches!(
                result,
                Err(GetPaymentRequestError::NoConnection(_))
                    | Err(GetPaymentRequestError::ConnectionFailed(_))
            ),
            "Expected NoConnection or ConnectionFailed, got {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_confirm_payment_connection_error() {
        let client = WalletConnectPay::new(test_config(
            "http://127.0.0.1:54323".to_string(),
        ))
        .unwrap();

        let result = client
            .confirm_payment(
                "pay_123".to_string(),
                "opt_1".to_string(),
                vec![],
                None,
                Some(5000),
            )
            .await;

        // Connection errors map to NoConnection or ConnectionFailed
        assert!(
            matches!(
                result,
                Err(ConfirmPaymentError::NoConnection(_))
                    | Err(ConfirmPaymentError::ConnectionFailed(_))
            ),
            "Expected NoConnection or ConnectionFailed, got {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_confirm_payment_no_connection_dns_failure() {
        let client = WalletConnectPay::new(test_config(
            "http://nonexistent.invalid:8080".to_string(),
        ))
        .unwrap();

        let result = client
            .confirm_payment(
                "pay_123".to_string(),
                "opt_1".to_string(),
                vec![],
                None,
                Some(5000),
            )
            .await;

        // DNS failure should map to NoConnection
        assert!(
            matches!(result, Err(ConfirmPaymentError::NoConnection(_))),
            "Expected NoConnection, got {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_get_payment_status_success() {
        let mock_server = MockServer::start().await;

        let status_response = serde_json::json!({
            "status": "succeeded",
            "isFinal": true
        });

        Mock::given(method("GET"))
            .and(path("/v1/gateway/payment/pay_status_123/status"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(&status_response),
            )
            .mount(&mock_server)
            .await;

        let client =
            WalletConnectPay::new(test_config(mock_server.uri())).unwrap();
        let result =
            client.get_payment_status("pay_status_123".to_string()).await;

        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.payment_id, "pay_status_123");
        assert_eq!(resp.status, PaymentStatus::Succeeded);
        assert!(resp.is_final);
    }

    #[tokio::test]
    async fn test_get_payment_status_not_found() {
        let mock_server = MockServer::start().await;

        let error_response = serde_json::json!({
            "code": "payment_not_found",
            "message": "Payment not found"
        });

        Mock::given(method("GET"))
            .and(path("/v1/gateway/payment/pay_notfound/status"))
            .respond_with(
                ResponseTemplate::new(404).set_body_json(&error_response),
            )
            .mount(&mock_server)
            .await;

        let client =
            WalletConnectPay::new(test_config(mock_server.uri())).unwrap();
        let result =
            client.get_payment_status("pay_notfound".to_string()).await;

        assert!(
            matches!(result, Err(GetPaymentStatusError::PaymentNotFound(_))),
            "Expected PaymentNotFound, got {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_get_payment_status_connection_error() {
        let client = WalletConnectPay::new(test_config(
            "http://127.0.0.1:54324".to_string(),
        ))
        .unwrap();

        let result = client.get_payment_status("pay_123".to_string()).await;

        assert!(
            matches!(
                result,
                Err(GetPaymentStatusError::NoConnection(_))
                    | Err(GetPaymentStatusError::ConnectionFailed(_))
            ),
            "Expected NoConnection or ConnectionFailed, got {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_confirm_payment_returns_user_friendly_error_message() {
        let client = WalletConnectPay::new(test_config(
            "http://nonexistent.invalid:8080".to_string(),
        ))
        .unwrap();

        let result = client
            .confirm_payment(
                "pay_123".to_string(),
                "opt_1".to_string(),
                vec![],
                None,
                Some(100),
            )
            .await;

        // Network errors should contain user-friendly message
        match result {
            Err(ConfirmPaymentError::NoConnection(msg)) => {
                assert!(
                    msg.contains("No internet connection"),
                    "Expected user-friendly message, got: {}",
                    msg
                );
            }
            Err(ConfirmPaymentError::ConnectionFailed(msg)) => {
                assert!(
                    msg.contains("No internet connection"),
                    "Expected user-friendly message, got: {}",
                    msg
                );
            }
            other => {
                panic!("Expected NoConnection or ConnectionFailed: {:?}", other)
            }
        }
    }

    #[tokio::test]
    async fn test_confirm_payment_auto_recovers_on_network_return() {
        let mock_server = MockServer::start().await;

        let confirm_response = serde_json::json!({
            "status": "processing",
            "isFinal": false,
            "pollInMs": 10
        });
        let status_response = serde_json::json!({
            "status": "succeeded",
            "isFinal": true
        });

        Mock::given(method("POST"))
            .and(path("/v1/gateway/payment/pay_recover/confirm"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(&confirm_response),
            )
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/v1/gateway/payment/pay_recover/status"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(&status_response),
            )
            .mount(&mock_server)
            .await;

        let client =
            WalletConnectPay::new(test_config(mock_server.uri())).unwrap();
        let result = client
            .confirm_payment(
                "pay_recover".to_string(),
                "opt_1".to_string(),
                vec!["0x123".to_string()],
                None,
                Some(5000),
            )
            .await;

        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.status, PaymentStatus::Succeeded);
        assert!(resp.is_final);
    }

    #[tokio::test]
    async fn test_confirm_payment_polling_timeout() {
        let mock_server = MockServer::start().await;
        let confirm_response = serde_json::json!({
            "status": "processing",
            "isFinal": false,
            "pollInMs": 10
        });
        let status_response = serde_json::json!({
            "status": "processing",
            "isFinal": false,
            "pollInMs": 10
        });
        Mock::given(method("POST"))
            .and(path("/v1/gateway/payment/pay_timeout/confirm"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(&confirm_response),
            )
            .mount(&mock_server)
            .await;
        Mock::given(method("GET"))
            .and(path("/v1/gateway/payment/pay_timeout/status"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(&status_response),
            )
            .mount(&mock_server)
            .await;
        let client =
            WalletConnectPay::new(test_config(mock_server.uri())).unwrap();
        let result = client
            .confirm_payment(
                "pay_timeout".to_string(),
                "opt_1".to_string(),
                vec!["0x123".to_string()],
                None,
                Some(100),
            )
            .await;
        assert!(
            matches!(result, Err(ConfirmPaymentError::PollingTimeout(_))),
            "Expected PollingTimeout, got {:?}",
            result
        );
    }

    #[test]
    fn test_looks_like_network_error_detects_patterns() {
        assert!(looks_like_network_error(
            "error sending request for url (https://example.com)"
        ));
        assert!(looks_like_network_error("Connection reset by peer"));
        assert!(looks_like_network_error("operation timed out"));
        assert!(looks_like_network_error("Network is unreachable"));
        assert!(!looks_like_network_error("404: Not found"));
        assert!(!looks_like_network_error("500: Internal server error"));
        assert!(!looks_like_network_error("Invalid JSON response"));
    }

    #[test]
    fn test_is_network_error_detects_http_with_network_patterns() {
        let err = ConfirmPaymentError::Http(
            "error sending request for url (https://example.com)".into(),
        );
        assert!(is_network_error(&err));

        let err = ConfirmPaymentError::Http("404: Not found".into());
        assert!(!is_network_error(&err));

        let err = ConfirmPaymentError::NoConnection("test".into());
        assert!(is_network_error(&err));
        let err = ConfirmPaymentError::RequestTimeout("test".into());
        assert!(is_network_error(&err));
        let err = ConfirmPaymentError::ConnectionFailed("test".into());
        assert!(is_network_error(&err));
    }

    #[test]
    fn test_make_user_friendly_error_handles_http_network_errors() {
        let err = ConfirmPaymentError::Http(
            "error sending request for url (https://example.com)".into(),
        );
        let friendly = make_user_friendly_error(err);
        match friendly {
            ConfirmPaymentError::NoConnection(msg) => {
                assert!(msg.contains("No internet connection"));
            }
            other => {
                panic!("Expected NoConnection, got {:?}", other)
            }
        }

        let err = ConfirmPaymentError::Http("404: Not found".into());
        let friendly = make_user_friendly_error(err);
        match friendly {
            ConfirmPaymentError::Http(msg) => {
                assert_eq!(msg, "404: Not found");
            }
            other => {
                panic!("Expected Http, got {:?}", other)
            }
        }
    }
}
