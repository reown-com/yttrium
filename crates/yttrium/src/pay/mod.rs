progenitor::generate_api!(
    spec = "src/pay/openapi.json",
    interface = Builder,
    tags = Separate,
    derives = [PartialEq],
);

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
    pub api_key: String,
    pub sdk_name: String,
    pub sdk_version: String,
    pub sdk_platform: String,
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
pub struct ConfirmPaymentResult {
    pub status: PaymentStatus,
    pub is_final: bool,
    pub poll_in_ms: Option<i64>,
}

impl From<types::ConfirmPaymentResponse> for ConfirmPaymentResult {
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
pub struct BuildAction {
    pub data: String,
}

impl From<types::Build> for BuildAction {
    fn from(b: types::Build) -> Self {
        Self { data: serde_json::to_string(&b.data).unwrap_or_default() }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[serde(rename_all = "camelCase", tag = "type", content = "data")]
pub enum RequiredAction {
    WalletRpc(WalletRpcAction),
    Build(BuildAction),
}

impl From<types::RequiredAction> for RequiredAction {
    fn from(a: types::RequiredAction) -> Self {
        match a {
            types::RequiredAction::WalletRpc(data) => {
                RequiredAction::WalletRpc(data.into())
            }
            types::RequiredAction::Build(data) => {
                RequiredAction::Build(data.into())
            }
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
    pub eta_seconds: i64,
    pub required_actions: Vec<RequiredAction>,
}

impl From<types::PaymentOption> for PaymentOption {
    fn from(o: types::PaymentOption) -> Self {
        Self {
            id: o.id,
            amount: o.amount.into(),
            eta_seconds: o.eta_seconds,
            required_actions: o
                .required_actions
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct PaymentOptionsResponse {
    pub options: Vec<PaymentOption>,
}

impl From<types::GetPaymentOptionsResponse> for PaymentOptionsResponse {
    fn from(r: types::GetPaymentOptionsResponse) -> Self {
        Self { options: r.options.into_iter().map(Into::into).collect() }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct FetchResponse {
    pub required_actions: Vec<RequiredAction>,
}

impl From<types::FetchResponse> for FetchResponse {
    fn from(r: types::FetchResponse) -> Self {
        Self {
            required_actions: r
                .required_actions
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
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
    pub fn new(config: SdkConfig) -> Self {
        let client = Client::new(&config.base_url);
        Self { client, config, cached_options: RwLock::new(Vec::new()) }
    }

    /// Get payment options for given accounts
    /// Also caches the options for use by get_required_payment_actions
    pub async fn get_payment_options(
        &self,
        payment_link: String,
        accounts: Vec<String>,
    ) -> Result<PaymentOptionsResponse, GetPaymentOptionsError> {
        let payment_id = extract_payment_id(&payment_link);
        let body = types::GetPaymentOptionsRequest { accounts, refresh: None };
        let response = with_retry(|| async {
            with_sdk_config!(
                self.client.gateway_get_payment_options(),
                &self.config
            )
            .id(&payment_id)
            .body(body.clone())
            .send()
            .await
        })
        .await
        .map_err(map_payment_options_error)?;

        let api_response = response.into_inner();

        // Cache the options with their required actions
        let cached: Vec<CachedPaymentOption> = api_response
            .options
            .iter()
            .map(|o| CachedPaymentOption {
                option_id: o.id.clone(),
                required_actions: o
                    .required_actions
                    .iter()
                    .cloned()
                    .map(Into::into)
                    .collect(),
            })
            .collect();
        let mut cache = self.cached_options.write().map_err(|e| {
            GetPaymentOptionsError::InternalError(format!(
                "Cache lock poisoned: {}",
                e
            ))
        })?;
        *cache = cached;

        Ok(api_response.into())
    }

    /// Get required payment actions for a selected option
    /// Returns cached actions if available, otherwise calls fetch to get them
    pub async fn get_required_payment_actions(
        &self,
        payment_id: String,
        option_id: String,
    ) -> Result<Vec<RequiredAction>, GetPaymentRequestError> {
        {
            let cache = self.cached_options.read().map_err(|e| {
                GetPaymentRequestError::InternalError(format!(
                    "Cache lock poisoned: {}",
                    e
                ))
            })?;
            if let Some(cached_option) =
                cache.iter().find(|o| o.option_id == option_id)
            {
                return Ok(cached_option.required_actions.clone());
            }
        }
        let fetch_response = self
            .fetch(payment_id, option_id.clone(), "{}".to_string())
            .await
            .map_err(|e| GetPaymentRequestError::FetchError(e.to_string()))?;
        let mut cache = self.cached_options.write().map_err(|e| {
            GetPaymentRequestError::InternalError(format!(
                "Cache lock poisoned: {}",
                e
            ))
        })?;
        cache.push(CachedPaymentOption {
            option_id,
            required_actions: fetch_response.required_actions.clone(),
        });
        Ok(fetch_response.required_actions)
    }

    /// Confirm a payment
    /// Polls for final status if the initial response is not final
    pub async fn confirm_payment(
        &self,
        payment_id: String,
        max_poll_ms: Option<i64>,
    ) -> Result<ConfirmPaymentResult, ConfirmPaymentError> {
        let body = types::ConfirmPaymentRequest(serde_json::Map::new());
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
            .map_err(map_confirm_payment_error)?;
        let mut result: ConfirmPaymentResult = response.into_inner().into();
        while !result.is_final {
            let poll_ms = result.poll_in_ms.unwrap_or(1000);
            tokio::time::sleep(std::time::Duration::from_millis(
                poll_ms as u64,
            ))
            .await;
            let status = self
                .get_gateway_payment_status(payment_id.clone(), max_poll_ms)
                .await
                .map_err(|e| ConfirmPaymentError::Http(e.to_string()))?;
            result = ConfirmPaymentResult {
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
    async fn fetch(
        &self,
        payment_id: String,
        option_id: String,
        data: String,
    ) -> Result<FetchResponse, PayError> {
        let data_value: serde_json::Value =
            serde_json::from_str(&data).unwrap_or(serde_json::Value::Null);
        let body = types::FetchRequest { option_id, data: data_value };
        let response = with_retry(|| async {
            with_sdk_config!(self.client.fetch_handler(), &self.config)
                .id(&payment_id)
                .body(body.clone())
                .send()
                .await
        })
        .await?;
        Ok(response.into_inner().into())
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
            api_key: "test-api-key".to_string(),
            sdk_name: "test-sdk".to_string(),
            sdk_version: "1.0.0".to_string(),
            sdk_platform: "test".to_string(),
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
                    "etaSeconds": 5,
                    "requiredActions": []
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
                    "etaSeconds": 5,
                    "requiredActions": [
                        {
                            "type": "build",
                            "data": { "data": {} }
                        },
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
            )
            .await
            .unwrap();
        assert_eq!(response.options.len(), 1);

        // Test with cached option
        let actions = client
            .get_required_payment_actions(
                "pay_123".to_string(),
                "opt_1".to_string(),
            )
            .await
            .unwrap();
        assert_eq!(actions.len(), 2);
        assert!(matches!(actions[0], RequiredAction::Build(_)));
        assert!(matches!(actions[1], RequiredAction::WalletRpc(_)));
        if let RequiredAction::WalletRpc(data) = &actions[1] {
            assert_eq!(data.chain_id, "eip155:8453");
            assert_eq!(data.method, "eth_signTypedData_v4");
        }
    }

    #[tokio::test]
    async fn test_get_required_payment_actions_fetches_when_not_cached() {
        let mock_server = MockServer::start().await;

        let fetch_response = serde_json::json!({
            "requiredActions": [{
                "type": "walletRpc",
                "data": {
                    "chainId": "eip155:1",
                    "method": "eth_sendTransaction",
                    "params": ["0x123"]
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
        assert!(matches!(actions[0], RequiredAction::WalletRpc(_)));
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
        let response =
            client.confirm_payment("pay_123".to_string(), None).await;

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
        let response =
            client.confirm_payment("pay_123".to_string(), Some(5000)).await;

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
            api_key: "my-custom-api-key".to_string(),
            sdk_name: "my-app".to_string(),
            sdk_version: "2.5.0".to_string(),
            sdk_platform: "ios".to_string(),
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
        let result =
            client.confirm_payment("pay_custom".to_string(), None).await;

        assert!(result.is_ok());
    }
}
