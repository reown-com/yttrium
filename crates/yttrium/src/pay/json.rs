use super::{CollectDataFieldResult, SdkConfig, WalletConnectPay};

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum PayJsonError {
    #[error("JSON parse error: {0}")]
    JsonParse(String),
    #[error("JSON serialize error: {0}")]
    JsonSerialize(String),
    #[error("Payment options error: {0}")]
    PaymentOptions(String),
    #[error("Payment request error: {0}")]
    PaymentRequest(String),
    #[error("Confirm payment error: {0}")]
    ConfirmPayment(String),
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
        Ok(Self { client: WalletConnectPay::new(config) })
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
            .map_err(|e| PayJsonError::PaymentOptions(e.to_string()))?;
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
            .map_err(|e| PayJsonError::PaymentRequest(e.to_string()))?;
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
            .map_err(|e| PayJsonError::ConfirmPayment(e.to_string()))?;
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
            api_key: "test-api-key".to_string(),
            sdk_name: "test-sdk".to_string(),
            sdk_version: "1.0.0".to_string(),
            sdk_platform: "test".to_string(),
        }
    }

    fn test_config_json(base_url: &str) -> String {
        format!(
            r#"{{"baseUrl":"{}","apiKey":"test-api-key","sdkName":"test-sdk","sdkVersion":"1.0.0","sdkPlatform":"test"}}"#,
            base_url
        )
    }

    #[tokio::test]
    async fn test_json_get_payment_options_success() {
        let mock_server = MockServer::start().await;

        let mock_response = serde_json::json!({
            "options": [{
                "id": "opt_json_1",
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

        let request_json = r#"{"paymentLink": "https://pay.example.com/pay_json_123", "accounts": ["eip155:8453:0xabc"]}"#;
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
}
