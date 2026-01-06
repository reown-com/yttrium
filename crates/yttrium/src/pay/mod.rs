progenitor::generate_api!(
    spec = "src/pay/openapi.json",
    interface = Builder,
    tags = Separate,
    derives = [PartialEq],
);

mod error_reporting;

#[cfg(feature = "uniffi")]
pub mod json;

#[cfg(feature = "uniffi")]
pub use json::{PayJsonError, WalletConnectPayJson};

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
    #[error("Internal error: {0}")]
    InternalError(String),
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
                tracing::error!(
                    "Failed to serialize WalletRpcAction params: {}",
                    e
                );
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
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[serde(rename_all = "camelCase", tag = "type", content = "data")]
pub enum Action {
    WalletRpc(WalletRpcAction),
    CollectData(CollectDataAction),
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

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct CollectDataResultData {
    pub fields: Vec<CollectDataFieldResult>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct WalletRpcResultData {
    pub method: String,
    pub data: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[serde(rename_all = "camelCase", tag = "type", content = "data")]
pub enum ConfirmPaymentResultItem {
    WalletRpc(WalletRpcResultData),
    CollectData(CollectDataResultData),
}

fn try_into_confirm_result(
    r: ConfirmPaymentResultItem,
) -> Result<types::ConfirmPaymentResult, ConfirmPaymentError> {
    match r {
        ConfirmPaymentResultItem::WalletRpc(data) => {
            let result = match data.method.as_str() {
                "eth_signTypedData_v4" => {
                    types::WalletRpcResult::EthSignTypedDataV4(data.data)
                }
                _ => {
                    return Err(ConfirmPaymentError::UnsupportedMethod(
                        data.method,
                    ));
                }
            };
            Ok(types::ConfirmPaymentResult::WalletRpc(result))
        }
        ConfirmPaymentResultItem::CollectData(data) => {
            Ok(types::ConfirmPaymentResult::CollectData(
                types::CollectDataResult {
                    fields: data.fields.into_iter().map(Into::into).collect(),
                },
            ))
        }
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
                        Some(Action::WalletRpc(data.into()))
                    }
                    types::Action::CollectData(data) => {
                        Some(Action::CollectData(data.into()))
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
}

// ==================== Client ====================

use std::sync::RwLock;

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
    client: Client,
    config: SdkConfig,
    cached_options: RwLock<Vec<CachedPaymentOption>>,
    error_http_client: reqwest::Client,
}

#[cfg_attr(feature = "uniffi", uniffi::export(async_runtime = "tokio"))]
impl WalletConnectPay {
    #[cfg_attr(feature = "uniffi", uniffi::constructor)]
    pub fn new(config: SdkConfig) -> Self {
        let client = Client::new(&config.base_url);
        let error_http_client = reqwest::Client::builder()
            .user_agent(format!("{}/{}", config.sdk_name, config.sdk_version))
            .build()
            .unwrap_or_default();
        Self {
            client,
            config,
            cached_options: RwLock::new(Vec::new()),
            error_http_client,
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
        let payment_id = extract_payment_id(&payment_link)?;
        let body = types::GetPaymentOptionsRequest { accounts, refresh: None };
        let response = with_retry(|| async {
            with_sdk_config!(
                self.client.gateway_get_payment_options(),
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
            let err = map_payment_options_error(e);
            self.report_error(&err, &payment_id);
            err
        })?;

        let api_response = response.into_inner();

        // Cache the options with their raw actions
        let cached: Vec<CachedPaymentOption> = api_response
            .options
            .iter()
            .map(|o| CachedPaymentOption {
                option_id: o.id.clone(),
                actions: o.actions.clone(),
            })
            .collect();
        let mut cache = self.cached_options.write().map_err(|e| {
            let err = GetPaymentOptionsError::InternalError(format!(
                "Cache lock poisoned: {}",
                e
            ));
            self.report_error(&err, &payment_id);
            err
        })?;
        *cache = cached;

        Ok(PaymentOptionsResponse {
            payment_id,
            info: api_response.info.map(Into::into),
            options: api_response.options.into_iter().map(Into::into).collect(),
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
        let raw_actions = {
            let cache = self.cached_options.read().map_err(|e| {
                let err = GetPaymentRequestError::InternalError(format!(
                    "Cache lock poisoned: {}",
                    e
                ));
                self.report_error(&err, &payment_id);
                err
            })?;
            cache
                .iter()
                .find(|o| o.option_id == option_id)
                .map(|o| o.actions.clone())
        };
        let raw_actions = match raw_actions {
            Some(actions) if !actions.is_empty() => actions,
            _ => {
                let fetched = self
                    .fetch(&payment_id, &option_id, String::new())
                    .await
                    .map_err(|e| {
                        let err =
                            GetPaymentRequestError::FetchError(e.to_string());
                        self.report_error(&err, &payment_id);
                        err
                    })?;
                let mut cache = self.cached_options.write().map_err(|e| {
                    let err = GetPaymentRequestError::InternalError(format!(
                        "Cache lock poisoned: {}",
                        e
                    ));
                    self.report_error(&err, &payment_id);
                    err
                })?;
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
        self.resolve_actions(&payment_id, &option_id, raw_actions)
            .await
            .map_err(|e| {
                self.report_error(&e, &payment_id);
                e
            })
    }

    /// Confirm a payment
    /// Polls for final status if the initial response is not final
    pub async fn confirm_payment(
        &self,
        payment_id: String,
        option_id: String,
        results: Vec<ConfirmPaymentResultItem>,
        max_poll_ms: Option<i64>,
    ) -> Result<ConfirmPaymentResultResponse, ConfirmPaymentError> {
        let api_results: Vec<types::ConfirmPaymentResult> = results
            .into_iter()
            .map(try_into_confirm_result)
            .collect::<Result<_, _>>()
            .map_err(|e| {
                self.report_error(&e, &payment_id);
                e
            })?;
        let body =
            types::ConfirmPaymentRequest { option_id, results: api_results };
        let mut req = with_sdk_config!(
            self.client.confirm_payment_handler(),
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
                let err = map_confirm_payment_error(e);
                self.report_error(&err, &payment_id);
                err
            })?;
        let mut result: ConfirmPaymentResultResponse =
            response.into_inner().into();
        while !result.is_final {
            let poll_ms = result.poll_in_ms.unwrap_or(1000);
            tokio::time::sleep(std::time::Duration::from_millis(
                poll_ms as u64,
            ))
            .await;
            let status = self
                .get_gateway_payment_status(payment_id.clone(), max_poll_ms)
                .await
                .map_err(|e| {
                    let err = ConfirmPaymentError::Http(e.to_string());
                    self.report_error(&err, &payment_id);
                    err
                })?;
            result = ConfirmPaymentResultResponse {
                status: status.status.into(),
                is_final: status.is_final,
                poll_in_ms: status.poll_in_ms,
            };
        }
        Ok(result)
    }
}

// Private methods (not exported via uniffi)
impl WalletConnectPay {
    fn report_error<E: std::fmt::Debug>(&self, error: &E, topic: &str) {
        error_reporting::report_error(
            &self.error_http_client,
            &self.config.bundle_id,
            &self.config.project_id,
            &self.config.sdk_name,
            &self.config.sdk_version,
            &error_reporting::error_type_name(error),
            topic,
            &format!("{:?}", error),
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
                    result.push(Action::WalletRpc(data.into()));
                }
                types::Action::CollectData(data) => {
                    result.push(Action::CollectData(data.into()));
                }
                types::Action::Build(build) => {
                    let resolved = self
                        .fetch(payment_id, option_id, build.data)
                        .await
                        .map_err(|e| {
                            GetPaymentRequestError::FetchError(e.to_string())
                        })?;
                    for resolved_action in resolved {
                        match resolved_action {
                            types::Action::WalletRpc(data) => {
                                result.push(Action::WalletRpc(data.into()));
                            }
                            types::Action::CollectData(data) => {
                                result.push(Action::CollectData(data.into()));
                            }
                            types::Action::Build(_) => {}
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
            with_sdk_config!(self.client.fetch_handler(), &self.config)
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
            self.client.gateway_get_payment_status(),
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
    let id = if let Some(query) = payment_link.split('?').nth(1) {
        query
            .split('&')
            .find_map(|param| param.strip_prefix("pid="))
            .unwrap_or("")
    } else {
        payment_link.rsplit('/').next().unwrap_or("")
    };
    if id.is_empty() {
        return Err(GetPaymentOptionsError::InvalidRequest(
            "payment_link cannot be empty".to_string(),
        ));
    }
    Ok(id.to_string())
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

        Mock::given(method("POST"))
            .and(path("/v1/gateway/payment/pay_notfound/options"))
            .respond_with(ResponseTemplate::new(404))
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

        Mock::given(method("POST"))
            .and(path("/v1/gateway/payment/pay_expired/options"))
            .respond_with(ResponseTemplate::new(410))
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
                                "chainId": "eip155:8453",
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
        let Action::WalletRpc(data) = &actions[0] else {
            panic!("Expected WalletRpc action");
        };
        assert_eq!(data.chain_id, "eip155:8453");
        assert_eq!(data.method, "eth_signTypedData_v4");
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
                    "chainId": "eip155:8453",
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
        let Action::WalletRpc(data) = &actions[0] else {
            panic!("Expected WalletRpc action");
        };
        assert_eq!(data.chain_id, "eip155:8453");
        assert!(data.params.contains("resolved"));
    }

    #[tokio::test]
    async fn test_get_required_payment_actions_fetches_when_not_cached() {
        let mock_server = MockServer::start().await;

        let fetch_response = serde_json::json!({
            "actions": [{
                "type": "walletRpc",
                "data": {
                    "chainId": "eip155:1",
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
        assert!(matches!(actions[0], Action::WalletRpc(_)));
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
        let results =
            vec![ConfirmPaymentResultItem::WalletRpc(WalletRpcResultData {
                method: "eth_signTypedData_v4".to_string(),
                data: vec!["0x123".to_string()],
            })];
        let response = client
            .confirm_payment(
                "pay_123".to_string(),
                "opt_1".to_string(),
                results,
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
        let results =
            vec![ConfirmPaymentResultItem::WalletRpc(WalletRpcResultData {
                method: "eth_signTypedData_v4".to_string(),
                data: vec!["0x123".to_string()],
            })];
        let response = client
            .confirm_payment(
                "pay_123".to_string(),
                "opt_1".to_string(),
                results,
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
        let results =
            vec![ConfirmPaymentResultItem::WalletRpc(WalletRpcResultData {
                method: "eth_signTypedData_v4".to_string(),
                data: vec!["0x123".to_string()],
            })];
        let result = client
            .confirm_payment(
                "pay_custom".to_string(),
                "opt_1".to_string(),
                results,
                None,
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_collect_data_action() {
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
                            "decimals": 6
                        }
                    },
                    "etaS": 5,
                    "actions": [
                        {
                            "type": "collectData",
                            "data": {
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
        assert_eq!(response.options[0].actions.len(), 1);

        let Action::CollectData(data) = &response.options[0].actions[0] else {
            panic!("Expected CollectData action");
        };
        assert_eq!(data.fields.len(), 2);
        assert_eq!(data.fields[0].id, "firstName");
        assert_eq!(data.fields[0].field_type, CollectDataFieldType::Text);
        assert!(data.fields[0].required);
        assert_eq!(data.fields[1].id, "dob");
        assert_eq!(data.fields[1].field_type, CollectDataFieldType::Date);
        assert!(!data.fields[1].required);
    }
}
