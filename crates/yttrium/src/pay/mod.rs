progenitor::generate_api!(
    spec = "src/pay/openapi.json",
    interface = Builder,
    tags = Separate,
    derives = [PartialEq],
);

mod error_reporting;
mod observability;

#[cfg(feature = "uniffi")]
pub mod json;

#[cfg(feature = "uniffi")]
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
pub enum PayError {
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
                Self::Http(format!("Network error: {}", err))
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

impl error_reporting::HasErrorType for PayError {
    fn error_type(&self) -> &'static str {
        match self {
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
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Unsupported RPC method: {0}")]
    UnsupportedMethod(String),
}

impl error_reporting::HasErrorType for ConfirmPaymentError {
    fn error_type(&self) -> &'static str {
        match self {
            Self::PaymentNotFound(_) => "PaymentNotFound",
            Self::PaymentExpired(_) => "PaymentExpired",
            Self::InvalidOption(_) => "InvalidOption",
            Self::InvalidSignature(_) => "InvalidSignature",
            Self::RouteExpired(_) => "RouteExpired",
            Self::Http(_) => "Http",
            Self::InternalError(_) => "InternalError",
            Self::UnsupportedMethod(_) => "UnsupportedMethod",
        }
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
    use rand::Rng;
    let mut attempt = 0;
    loop {
        match f().await {
            Ok(v) => return Ok(v),
            Err(e) if is_server_error(&e) && attempt < MAX_RETRIES => {
                attempt += 1;
                let base_backoff = INITIAL_BACKOFF_MS
                    .saturating_mul(2u64.saturating_pow(attempt - 1));
                let jitter = rand::thread_rng().gen_range(0..=base_backoff / 2);
                let backoff = base_backoff + jitter;
                tokio::time::sleep(std::time::Duration::from_millis(backoff))
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
    pub project_id: String,
    pub api_key: String,
    pub sdk_name: String,
    pub sdk_version: String,
    pub sdk_platform: String,
    pub bundle_id: String,
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
pub struct ConfirmPaymentResultResponse {
    pub status: PaymentStatus,
    pub is_final: bool,
    pub poll_in_ms: Option<i64>,
}

impl From<types::ConfirmPaymentResponse> for ConfirmPaymentResultResponse {
    fn from(r: types::ConfirmPaymentResponse) -> Self {
        Self {
            status: r.status.into(),
            is_final: r.is_final,
            poll_in_ms: r.poll_in_ms,
        }
    }
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
}

impl From<types::CollectDataFieldType> for CollectDataFieldType {
    fn from(t: types::CollectDataFieldType) -> Self {
        match t {
            types::CollectDataFieldType::Text => CollectDataFieldType::Text,
            types::CollectDataFieldType::Date => CollectDataFieldType::Date,
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
}

impl From<types::CollectData> for CollectDataAction {
    fn from(c: types::CollectData) -> Self {
        Self { fields: c.fields.into_iter().map(Into::into).collect() }
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
    pub network_name: Option<String>,
}

impl From<types::AmountDisplay> for AmountDisplay {
    fn from(d: types::AmountDisplay) -> Self {
        Self {
            asset_symbol: d.asset_symbol,
            asset_name: d.asset_name,
            decimals: d.decimals,
            icon_url: d.icon_url,
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
    pub amount: PayAmount,
    pub eta_s: i64,
    pub actions: Vec<Action>,
}

impl From<types::PaymentOption> for PaymentOption {
    fn from(o: types::PaymentOption) -> Self {
        Self {
            id: o.id,
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

use std::sync::{OnceLock, RwLock};

/// Applies common SDK config headers to any progenitor-generated request builder
macro_rules! with_sdk_config {
    ($builder:expr, $config:expr) => {
        $builder
            .api_key(&$config.api_key)
            .sdk_name(&$config.sdk_name)
            .sdk_version(&$config.sdk_version)
            .sdk_platform(&$config.sdk_platform)
    };
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
        self.client.get_or_init(|| Client::new(&self.config.base_url))
    }

    fn error_http_client(&self) -> &reqwest::Client {
        self.error_http_client.get_or_init(|| {
            reqwest::Client::builder()
                .user_agent(format!(
                    "{}/{}",
                    self.config.sdk_name, self.config.sdk_version
                ))
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .unwrap_or_else(|e| {
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
    pub fn new(config: SdkConfig) -> Self {
        let client_id = config
            .client_id
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        Self {
            client: OnceLock::new(),
            config,
            cached_options: RwLock::new(Vec::new()),
            error_http_client: OnceLock::new(),
            initialized_event_sent: OnceLock::new(),
            client_id,
        }
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
                &self.config
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
        let mut cache = self
            .cached_options
            .write()
            .expect("Cache lock poisoned - indicates a bug");
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
            let cache = self
                .cached_options
                .read()
                .expect("Cache lock poisoned - indicates a bug");
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
                        let err =
                            GetPaymentRequestError::FetchError(e.to_string());
                        self.report_error(&err, &payment_id);
                        self.send_trace(
                            observability::TraceEvent::RequiredActionsFailed,
                            &payment_id,
                        );
                        err
                    })?;
                let mut cache = self
                    .cached_options
                    .write()
                    .expect("Cache lock poisoned - indicates a bug");
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
            &self.config
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
                err
            })?;
        let mut result: ConfirmPaymentResultResponse =
            response.into_inner().into();
        pay_debug!(
            "confirm_payment: initial status={:?}, is_final={}",
            result.status,
            result.is_final
        );
        while !result.is_final {
            let poll_ms = result.poll_in_ms.unwrap_or(1000);
            pay_debug!("confirm_payment: polling in {}ms", poll_ms);
            tokio::time::sleep(std::time::Duration::from_millis(
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
                    err
                })?;
            result = ConfirmPaymentResultResponse {
                status: status.status.into(),
                is_final: status.is_final,
                poll_in_ms: status.poll_in_ms,
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
}

// Private methods (not exported via uniffi)
impl WalletConnectPay {
    fn report_error<E: std::fmt::Debug + error_reporting::HasErrorType>(
        &self,
        error: &E,
        payment_id: &str,
    ) {
        error_reporting::report_error(
            self.error_http_client(),
            &self.config.bundle_id,
            &self.config.project_id,
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
            &self.config.project_id,
            &self.config.api_key,
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
                        .map_err(|e| {
                            GetPaymentRequestError::FetchError(e.to_string())
                        })?;
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
            with_sdk_config!(self.client().fetch_handler(), &self.config)
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
            &self.config
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

fn extract_payment_id(
    payment_link: &str,
) -> Result<String, GetPaymentOptionsError> {
    fn url_decode(s: &str) -> String {
        urlencoding::decode(s)
            .map(|c| c.into_owned())
            .unwrap_or_else(|_| s.to_string())
    }

    fn extract_pid_from_query(query: &str) -> Option<String> {
        query.split('&').find_map(|param| {
            param
                .strip_prefix("pid=")
                .filter(|id| !id.is_empty())
                .map(String::from)
        })
    }

    fn extract_pid_from_link(link: &str) -> Option<String> {
        if let Some((_, query)) = link.split_once('?') {
            if let Some(id) = extract_pid_from_query(query) {
                return Some(id);
            }
        }
        let last_segment = link.rsplit('/').next().unwrap_or("");
        if !last_segment.is_empty()
            && !last_segment.contains('?')
            && !last_segment.contains('%')
        {
            return Some(last_segment.to_string());
        }
        None
    }

    fn extract_pay_param_value(query: &str) -> Option<String> {
        for param in query.split('&') {
            if let Some(value) = param.strip_prefix("pay=") {
                return Some(url_decode(value));
            }
        }
        None
    }

    fn try_extract_from_wc_uri(uri: &str) -> Option<String> {
        let (_, query) = uri.split_once('?')?;
        let pay_link = extract_pay_param_value(query)?;
        extract_pid_from_link(&pay_link)
    }

    let decoded = url_decode(payment_link);

    if decoded.starts_with("wc:") {
        if let Some(id) = try_extract_from_wc_uri(&decoded) {
            return Ok(id);
        }
    }

    if payment_link.starts_with("wc:") {
        if let Some(id) = try_extract_from_wc_uri(payment_link) {
            return Ok(id);
        }
    }

    if let Some(id) = extract_pid_from_link(&decoded) {
        return Ok(id);
    }

    if let Some(id) = extract_pid_from_link(payment_link) {
        return Ok(id);
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
        other => GetPaymentOptionsError::Http(other.to_string()),
    }
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
        other => ConfirmPaymentError::Http(other.to_string()),
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
            project_id: "test-project-id".to_string(),
            api_key: "test-api-key".to_string(),
            sdk_name: "test-sdk".to_string(),
            sdk_version: "1.0.0".to_string(),
            sdk_platform: "test".to_string(),
            bundle_id: "com.test.app".to_string(),
            client_id: None,
        }
    }

    #[tokio::test]
    async fn test_get_payment_options_success() {
        let mock_server = MockServer::start().await;

        let mock_response = serde_json::json!({
            "options": [
                {
                    "id": "opt_1",
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
                    "actions": []
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

        let client = WalletConnectPay::new(test_config(mock_server.uri()));
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

        let client = WalletConnectPay::new(test_config(mock_server.uri()));
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

        let client = WalletConnectPay::new(test_config(mock_server.uri()));
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
        assert_eq!(
            extract_payment_id("https://pay.walletconnect.com/pay_123")
                .unwrap(),
            "pay_123"
        );
        assert_eq!(extract_payment_id("pay_456").unwrap(), "pay_456");
        assert_eq!(
            extract_payment_id("https://example.com/path/to/pay_789").unwrap(),
            "pay_789"
        );
        assert!(extract_payment_id("").is_err());
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

        let client = WalletConnectPay::new(test_config(mock_server.uri()));
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

        let client = WalletConnectPay::new(test_config(mock_server.uri()));
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

        let client = WalletConnectPay::new(test_config(mock_server.uri()));
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

        let client = WalletConnectPay::new(test_config(mock_server.uri()));
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

        let client = WalletConnectPay::new(test_config(mock_server.uri()));
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
        let client = WalletConnectPay::new(test_config(mock_server.uri()));
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

        let client = WalletConnectPay::new(test_config(mock_server.uri()));
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
            project_id: "my-custom-project-id".to_string(),
            api_key: "my-custom-api-key".to_string(),
            sdk_name: "my-app".to_string(),
            sdk_version: "2.5.0".to_string(),
            sdk_platform: "ios".to_string(),
            bundle_id: "com.custom.app".to_string(),
            client_id: Some("custom-client-id".to_string()),
        };

        Mock::given(method("POST"))
            .and(path("/v1/gateway/payment/pay_custom/confirm"))
            .and(header("Api-Key", "my-custom-api-key"))
            .and(header("Sdk-Name", "my-app"))
            .and(header("Sdk-Version", "2.5.0"))
            .and(header("Sdk-Platform", "ios"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(&mock_response),
            )
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = WalletConnectPay::new(custom_config);
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
    async fn test_collect_data_response() {
        let mock_server = MockServer::start().await;
        let mock_response = serde_json::json!({
            "options": [],
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
                ]
            }
        });
        Mock::given(method("POST"))
            .and(path("/v1/gateway/payment/pay_123/options"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(&mock_response),
            )
            .mount(&mock_server)
            .await;

        let client = WalletConnectPay::new(test_config(mock_server.uri()));
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

        let client = WalletConnectPay::new(test_config(mock_server.uri()));
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
}
