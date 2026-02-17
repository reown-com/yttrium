use super::{
    CollectDataFieldResult, ConfigError, ConfirmPaymentError,
    GetPaymentOptionsError, GetPaymentRequestError, SdkConfig,
    WalletConnectPay,
};

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
#[cfg_attr(feature = "wasm", derive(derive_jserror::JsError))]
pub enum PayJsonError {
    // JSON errors
    #[error("JSON parse error: {0}")]
    JsonParse(String),
    #[error("JSON serialize error: {0}")]
    JsonSerialize(String),
    // Config errors
    #[error("Missing authentication: {0}")]
    MissingAuth(String),
    // Connectivity errors
    #[error("No network connection: {0}")]
    NoConnection(String),
    #[error("Request timed out: {0}")]
    RequestTimeout(String),
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    // Shared domain errors
    #[error("Payment not found: {0}")]
    PaymentNotFound(String),
    #[error("Payment expired: {0}")]
    PaymentExpired(String),
    #[error("Invalid account: {0}")]
    InvalidAccount(String),
    #[error("Option not found: {0}")]
    OptionNotFound(String),
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("Internal error: {0}")]
    InternalError(String),
    // GetPaymentOptions specific errors
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("Payment not ready: {0}")]
    PaymentNotReady(String),
    #[error("Compliance failed: {0}")]
    ComplianceFailed(String),
    // GetPaymentRequest specific errors
    #[error("Fetch error: {0}")]
    FetchError(String),
    // ConfirmPayment specific errors
    #[error("Invalid option: {0}")]
    InvalidOption(String),
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    #[error("Route expired: {0}")]
    RouteExpired(String),
    #[error("Unsupported method: {0}")]
    UnsupportedMethod(String),
    #[error("Polling timeout: {0}")]
    PollingTimeout(String),
}

impl From<ConfigError> for PayJsonError {
    fn from(e: ConfigError) -> Self {
        match e {
            ConfigError::MissingAuth(msg) => PayJsonError::MissingAuth(msg),
        }
    }
}

impl From<GetPaymentOptionsError> for PayJsonError {
    fn from(e: GetPaymentOptionsError) -> Self {
        match e {
            GetPaymentOptionsError::NoConnection(msg) => {
                Self::NoConnection(msg)
            }
            GetPaymentOptionsError::RequestTimeout(msg) => {
                Self::RequestTimeout(msg)
            }
            GetPaymentOptionsError::ConnectionFailed(msg) => {
                Self::ConnectionFailed(msg)
            }
            GetPaymentOptionsError::PaymentNotFound(msg) => {
                Self::PaymentNotFound(msg)
            }
            GetPaymentOptionsError::PaymentExpired(msg) => {
                Self::PaymentExpired(msg)
            }
            GetPaymentOptionsError::InvalidAccount(msg) => {
                Self::InvalidAccount(msg)
            }
            GetPaymentOptionsError::OptionNotFound(msg) => {
                Self::OptionNotFound(msg)
            }
            GetPaymentOptionsError::Http(msg) => Self::Http(msg),
            GetPaymentOptionsError::InternalError(msg) => {
                Self::InternalError(msg)
            }
            GetPaymentOptionsError::InvalidRequest(msg) => {
                Self::InvalidRequest(msg)
            }
            GetPaymentOptionsError::PaymentNotReady(msg) => {
                Self::PaymentNotReady(msg)
            }
            GetPaymentOptionsError::ComplianceFailed(msg) => {
                Self::ComplianceFailed(msg)
            }
        }
    }
}

impl From<GetPaymentRequestError> for PayJsonError {
    fn from(e: GetPaymentRequestError) -> Self {
        match e {
            GetPaymentRequestError::NoConnection(msg) => {
                Self::NoConnection(msg)
            }
            GetPaymentRequestError::RequestTimeout(msg) => {
                Self::RequestTimeout(msg)
            }
            GetPaymentRequestError::ConnectionFailed(msg) => {
                Self::ConnectionFailed(msg)
            }
            GetPaymentRequestError::PaymentNotFound(msg) => {
                Self::PaymentNotFound(msg)
            }
            GetPaymentRequestError::InvalidAccount(msg) => {
                Self::InvalidAccount(msg)
            }
            GetPaymentRequestError::OptionNotFound(msg) => {
                Self::OptionNotFound(msg)
            }
            GetPaymentRequestError::Http(msg) => Self::Http(msg),
            GetPaymentRequestError::InternalError(msg) => {
                Self::InternalError(msg)
            }
            GetPaymentRequestError::FetchError(msg) => Self::FetchError(msg),
        }
    }
}

impl From<ConfirmPaymentError> for PayJsonError {
    fn from(e: ConfirmPaymentError) -> Self {
        match e {
            ConfirmPaymentError::NoConnection(msg) => Self::NoConnection(msg),
            ConfirmPaymentError::RequestTimeout(msg) => {
                Self::RequestTimeout(msg)
            }
            ConfirmPaymentError::ConnectionFailed(msg) => {
                Self::ConnectionFailed(msg)
            }
            ConfirmPaymentError::PaymentNotFound(msg) => {
                Self::PaymentNotFound(msg)
            }
            ConfirmPaymentError::PaymentExpired(msg) => {
                Self::PaymentExpired(msg)
            }
            ConfirmPaymentError::Http(msg) => Self::Http(msg),
            ConfirmPaymentError::InternalError(msg) => Self::InternalError(msg),
            ConfirmPaymentError::InvalidOption(msg) => Self::InvalidOption(msg),
            ConfirmPaymentError::InvalidSignature(msg) => {
                Self::InvalidSignature(msg)
            }
            ConfirmPaymentError::RouteExpired(msg) => Self::RouteExpired(msg),
            ConfirmPaymentError::UnsupportedMethod(msg) => {
                Self::UnsupportedMethod(msg)
            }
            ConfirmPaymentError::PollingTimeout(msg) => {
                Self::PollingTimeout(msg)
            }
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetPaymentOptionsRequestJson {
    payment_link: String,
    accounts: Vec<String>,
    #[serde(default)]
    include_payment_info: bool,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetRequiredPaymentActionsRequestJson {
    payment_id: String,
    option_id: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct CollectDataFieldResultJson {
    id: String,
    value: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConfirmPaymentJsonRequestJson {
    payment_id: String,
    option_id: String,
    signatures: Vec<String>,
    #[serde(default)]
    collected_data: Option<Vec<CollectDataFieldResultJson>>,
    max_poll_ms: Option<i64>,
}

/// JSON wrapper for WalletConnectPay client
/// Accepts JSON strings as input, deserializes, calls underlying methods, and returns JSON strings
#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct WalletConnectPayJson {
    client: WalletConnectPay,
}

#[cfg_attr(feature = "uniffi", uniffi::export(async_runtime = "tokio"))]
impl WalletConnectPayJson {
    #[cfg_attr(feature = "uniffi", uniffi::constructor)]
    pub fn new(sdk_config: String) -> Result<Self, PayJsonError> {
        let config: SdkConfig = serde_json::from_str(&sdk_config)
            .map_err(|e| PayJsonError::JsonParse(e.to_string()))?;
        Ok(Self { client: WalletConnectPay::new(config)? })
    }

    /// Get payment options for a payment link
    /// Input JSON: { "paymentLink": "string", "accounts": ["string"], "includePaymentInfo": bool? }
    /// Returns JSON PaymentOptionsResponse or error
    pub async fn get_payment_options(
        &self,
        request_json: String,
    ) -> Result<String, PayJsonError> {
        let req: GetPaymentOptionsRequestJson =
            serde_json::from_str(&request_json)
                .map_err(|e| PayJsonError::JsonParse(e.to_string()))?;
        if req.payment_link.is_empty() {
            return Err(PayJsonError::JsonParse(
                "payment_link cannot be empty".to_string(),
            ));
        }
        if req.accounts.is_empty() {
            return Err(PayJsonError::JsonParse(
                "accounts cannot be empty".to_string(),
            ));
        }
        let result = self
            .client
            .get_payment_options(
                req.payment_link,
                req.accounts,
                req.include_payment_info,
            )
            .await
            .map_err(PayJsonError::from)?;
        serde_json::to_string(&result)
            .map_err(|e| PayJsonError::JsonSerialize(e.to_string()))
    }

    /// Get required payment actions for a selected option
    /// Input JSON: { "paymentId": "string", "optionId": "string" }
    /// Returns JSON array of Action or error
    pub async fn get_required_payment_actions(
        &self,
        request_json: String,
    ) -> Result<String, PayJsonError> {
        let req: GetRequiredPaymentActionsRequestJson =
            serde_json::from_str(&request_json)
                .map_err(|e| PayJsonError::JsonParse(e.to_string()))?;
        if req.payment_id.is_empty() {
            return Err(PayJsonError::JsonParse(
                "payment_id cannot be empty".to_string(),
            ));
        }
        if req.option_id.is_empty() {
            return Err(PayJsonError::JsonParse(
                "option_id cannot be empty".to_string(),
            ));
        }
        let result = self
            .client
            .get_required_payment_actions(req.payment_id, req.option_id)
            .await
            .map_err(PayJsonError::from)?;
        serde_json::to_string(&result)
            .map_err(|e| PayJsonError::JsonSerialize(e.to_string()))
    }

    /// Confirm a payment
    /// Input JSON: { "paymentId": "string", "optionId": "string", "signatures": ["string"], "collectedData": [{"id": "string", "value": "string"}]?, "maxPollMs": number? }
    /// Returns JSON ConfirmPaymentResponse or error
    pub async fn confirm_payment(
        &self,
        request_json: String,
    ) -> Result<String, PayJsonError> {
        let req: ConfirmPaymentJsonRequestJson =
            serde_json::from_str(&request_json)
                .map_err(|e| PayJsonError::JsonParse(e.to_string()))?;
        if req.payment_id.is_empty() {
            return Err(PayJsonError::JsonParse(
                "payment_id cannot be empty".to_string(),
            ));
        }
        if req.option_id.is_empty() {
            return Err(PayJsonError::JsonParse(
                "option_id cannot be empty".to_string(),
            ));
        }
        let collected_data = req.collected_data.map(|fields| {
            fields
                .into_iter()
                .map(|f| CollectDataFieldResult { id: f.id, value: f.value })
                .collect()
        });
        let result = self
            .client
            .confirm_payment(
                req.payment_id,
                req.option_id,
                req.signatures,
                collected_data,
                req.max_poll_ms,
            )
            .await
            .map_err(PayJsonError::from)?;
        serde_json::to_string(&result)
            .map_err(|e| PayJsonError::JsonSerialize(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::pay::SdkConfig,
        wiremock::{
            Mock, MockServer, ResponseTemplate,
            matchers::{method, path},
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

    fn test_config_json(base_url: &str) -> String {
        format!(
            r#"{{"baseUrl":"{}","projectId":"test-project-id","sdkName":"test-sdk","sdkVersion":"1.0.0","sdkPlatform":"test","bundleId":"com.test.app","apiKey":"test-api-key","appId":null,"clientId":null}}"#,
            base_url
        )
    }

    #[tokio::test]
    async fn test_json_get_payment_options_success() {
        let mock_server = MockServer::start().await;

        let mock_response = serde_json::json!({
            "options": [{
                "id": "opt_json_1",
                "account": "eip155:8453:0xabc",
                "amount": {
                    "unit": "caip19/eip155:8453/erc20:0xUSDC",
                    "value": "500000",
                    "display": {
                        "assetSymbol": "USDC",
                        "assetName": "USD Coin",
                        "decimals": 6,
                        "iconUrl": "https://example.com/usdc.png",
                        "networkName": "Base"
                    }
                },
                "etaS": 5,
                "actions": [{
                    "type": "walletRpc",
                    "data": {
                        "chain_id": "eip155:8453",
                        "method": "eth_signTypedData_v4",
                        "params": ["0x123", {"types": {}}]
                    }
                }]
            }]
        });

        Mock::given(method("POST"))
            .and(path("/v1/gateway/payment/pay_json_123/options"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(&mock_response),
            )
            .mount(&mock_server)
            .await;

        let client =
            WalletConnectPayJson::new(test_config_json(&mock_server.uri()))
                .unwrap();

        let request_json = r#"{"paymentLink": "https://pay.walletconnect.com/pay_json_123", "accounts": ["eip155:8453:0xabc"]}"#;
        let result = client.get_payment_options(request_json.to_string()).await;

        assert!(result.is_ok());
        let response_json = result.unwrap();
        let parsed: serde_json::Value =
            serde_json::from_str(&response_json).unwrap();
        assert_eq!(parsed["options"][0]["id"], "opt_json_1");
        assert_eq!(parsed["options"][0]["amount"]["value"], "500000");
        assert_eq!(parsed["options"][0]["etaS"], 5);
        let actions = parsed["options"][0]["actions"].clone();
        assert_eq!(actions.as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_json_get_payment_options_invalid_json() {
        let mock_server = MockServer::start().await;
        let client =
            WalletConnectPayJson::new(test_config_json(&mock_server.uri()))
                .unwrap();

        let result =
            client.get_payment_options("not valid json".to_string()).await;

        assert!(matches!(result, Err(PayJsonError::JsonParse(_))));
    }

    #[tokio::test]
    async fn test_json_get_required_payment_actions_success() {
        let mock_server = MockServer::start().await;

        let mock_response = serde_json::json!({
            "options": [{
                "id": "opt_json_2",
                "account": "eip155:1:0x123",
                "amount": {
                    "unit": "caip19/eip155:1/erc20:0xDAI",
                    "value": "100000000000000000000",
                    "display": {
                        "assetSymbol": "DAI",
                        "assetName": "Dai Stablecoin",
                        "decimals": 18,
                        "iconUrl": "https://example.com/dai.png",
                        "networkName": "Ethereum"
                    }
                },
                "etaS": 10,
                "actions": [{
                    "type": "walletRpc",
                    "data": {
                        "chain_id": "eip155:1",
                        "method": "eth_signTypedData_v4",
                        "params": ["0xwallet", {"types": {}}]
                    }
                }]
            }]
        });

        Mock::given(method("POST"))
            .and(path("/v1/gateway/payment/pay_json_456/options"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(&mock_response),
            )
            .mount(&mock_server)
            .await;

        let client =
            WalletConnectPayJson::new(test_config_json(&mock_server.uri()))
                .unwrap();

        let options_req = r#"{"paymentLink": "pay_json_456", "accounts": ["eip155:1:0x123"]}"#;
        client.get_payment_options(options_req.to_string()).await.unwrap();

        let actions_req =
            r#"{"paymentId": "pay_json_456", "optionId": "opt_json_2"}"#;
        let response_json = client
            .get_required_payment_actions(actions_req.to_string())
            .await
            .unwrap();
        let parsed: serde_json::Value =
            serde_json::from_str(&response_json).unwrap();
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 1);
        assert_eq!(parsed[0]["walletRpc"]["chainId"], "eip155:1");
        assert_eq!(parsed[0]["walletRpc"]["method"], "eth_signTypedData_v4");
        let params_str = parsed[0]["walletRpc"]["params"].as_str().unwrap();
        let params: serde_json::Value =
            serde_json::from_str(params_str).unwrap();
        assert_eq!(params[0], "0xwallet");
    }

    #[tokio::test]
    async fn test_json_confirm_payment_success() {
        let mock_server = MockServer::start().await;

        let confirm_response = serde_json::json!({
            "status": "succeeded",
            "isFinal": true,
            "pollInMs": null
        });

        Mock::given(method("POST"))
            .and(path("/v1/gateway/payment/pay_json_789/confirm"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(&confirm_response),
            )
            .mount(&mock_server)
            .await;

        let client =
            WalletConnectPayJson::new(test_config_json(&mock_server.uri()))
                .unwrap();

        let confirm_req = r#"{"paymentId": "pay_json_789", "optionId": "opt_1", "signatures": ["0x123"]}"#;
        let result = client.confirm_payment(confirm_req.to_string()).await;

        assert!(result.is_ok(), "Expected Ok but got: {:?}", result);
        let response_json = result.unwrap();
        let parsed: serde_json::Value =
            serde_json::from_str(&response_json).unwrap();
        assert_eq!(parsed["status"], "succeeded");
        assert_eq!(parsed["isFinal"], true);
        assert!(parsed["pollInMs"].is_null());
    }

    #[test]
    fn test_json_config_mapping() {
        let base_url = "https://api.example.com";
        let config_json = test_config_json(base_url);
        let expected_config = test_config(base_url.to_string());
        let parsed_config: SdkConfig =
            serde_json::from_str(&config_json).unwrap();
        assert_eq!(parsed_config, expected_config);
    }

    #[tokio::test]
    async fn test_json_get_payment_options_empty_payment_link() {
        let mock_server = MockServer::start().await;
        let client =
            WalletConnectPayJson::new(test_config_json(&mock_server.uri()))
                .unwrap();
        let request_json =
            r#"{"paymentLink": "", "accounts": ["eip155:1:0x123"]}"#;
        let result = client.get_payment_options(request_json.to_string()).await;
        assert!(matches!(result, Err(PayJsonError::JsonParse(_))));
    }

    #[tokio::test]
    async fn test_json_get_payment_options_empty_accounts() {
        let mock_server = MockServer::start().await;
        let client =
            WalletConnectPayJson::new(test_config_json(&mock_server.uri()))
                .unwrap();
        let request_json = r#"{"paymentLink": "pay_123", "accounts": []}"#;
        let result = client.get_payment_options(request_json.to_string()).await;
        assert!(matches!(result, Err(PayJsonError::JsonParse(_))));
    }

    #[tokio::test]
    async fn test_json_confirm_payment_empty_payment_id() {
        let mock_server = MockServer::start().await;
        let client =
            WalletConnectPayJson::new(test_config_json(&mock_server.uri()))
                .unwrap();
        let request_json = r#"{"paymentId": "", "optionId": "opt_1", "results": [], "maxPollMs": null}"#;
        let result = client.confirm_payment(request_json.to_string()).await;
        assert!(matches!(result, Err(PayJsonError::JsonParse(_))));
    }

    #[tokio::test]
    async fn test_json_confirm_payment_negative_poll_ms() {
        let mock_server = MockServer::start().await;
        let client =
            WalletConnectPayJson::new(test_config_json(&mock_server.uri()))
                .unwrap();
        // Invalid JSON: "results" should be "signatures"
        let request_json = r#"{"paymentId": "pay_123", "optionId": "opt_1", "results": [], "maxPollMs": -1000}"#;
        let result = client.confirm_payment(request_json.to_string()).await;
        assert!(matches!(result, Err(PayJsonError::JsonParse(_))));
    }

    #[tokio::test]
    async fn test_json_get_required_payment_actions_empty_payment_id() {
        let mock_server = MockServer::start().await;
        let client =
            WalletConnectPayJson::new(test_config_json(&mock_server.uri()))
                .unwrap();
        let request_json = r#"{"paymentId": "", "optionId": "opt_1"}"#;
        let result =
            client.get_required_payment_actions(request_json.to_string()).await;
        assert!(matches!(result, Err(PayJsonError::JsonParse(_))));
    }

    #[tokio::test]
    async fn test_json_get_required_payment_actions_empty_option_id() {
        let mock_server = MockServer::start().await;
        let client =
            WalletConnectPayJson::new(test_config_json(&mock_server.uri()))
                .unwrap();
        let request_json = r#"{"paymentId": "pay_123", "optionId": ""}"#;
        let result =
            client.get_required_payment_actions(request_json.to_string()).await;
        assert!(matches!(result, Err(PayJsonError::JsonParse(_))));
    }
}
