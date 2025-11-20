use {
    serde::{Deserialize, Serialize},
    url::Url,
};

#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct WalletPayClient {
    client: reqwest::Client,
    blockchain_api_base_url: Url,
}

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum WalletPayError {
    #[error("NoWalletPayRequestFound: {0}")]
    NoWalletPayRequestFound(String),
    #[error("ParseError: {0}")]
    ParseError(String),
    #[error("BapiError: {0}")]
    BapiError(String),
    #[error("InvalidIndex: {0}")]
    InvalidIndex(String),
}

// Types for wallet pay structures

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct AcceptedPayment {
    pub recipient: String,
    pub asset: String,
    pub amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct WalletPay {
    pub version: String,
    pub accepted_payments: Vec<AcceptedPayment>,
    pub expiry: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct PaymentOption {
    pub asset: String,      // CAIP-19
    pub amount: String,     // hex string
    pub recipient: String,  // chain-specific addr
    pub types: Vec<String>, // e.g. "erc20-transfer", "erc3009-authorization", ...
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct WalletPayDisplayItem {
    pub asset: String,
    pub amount: String,
    pub chain: Option<String>,
    pub symbol: Option<String>, // for UI only
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct DisplayData {
    pub payment_options: Vec<WalletPayDisplayItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct ParsedData {
    pub asset_name: String,
    pub amount: String,
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct WalletPayDisplayItemWithParsed {
    pub asset: String,
    pub amount: String,
    pub chain: Option<String>,
    pub symbol: Option<String>,
    pub parsed_data: ParsedData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct DisplayDataWithParsed {
    pub payment_options: Vec<WalletPayDisplayItemWithParsed>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct WalletPayAction {
    pub method: String, // "transaction" | "712TypedData"
    pub data: String,   // JSON string
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct TransferConfirmation {
    #[serde(rename = "type")]
    pub type_: String, // "erc20-transfer" | "erc3009-authorization"
    pub hash: String,
    pub data: String, // JSON string
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct WalletPayResponseResultV1 {
    pub transfer_confirmation: TransferConfirmation,
}

#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct WalletPayRequest {
    client: WalletPayClient,
    wallet_pay: WalletPay,
    payment_options: Vec<PaymentOption>,
}

// BAPI request/response types (mocked for now)

#[derive(Debug, Serialize)]
struct BapiDisplayDataRequest {
    accepted_payments: Vec<AcceptedPayment>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BapiDisplayDataResponse {
    payment_options: Vec<WalletPayDisplayItemWithParsed>,
}

#[derive(Debug, Serialize)]
struct BapiActionRequest {
    payment_option: PaymentOption,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BapiActionResponse {
    method: String,
    data: serde_json::Value,
    hash: String,
}

#[derive(Debug, Serialize)]
struct BapiFinalizeRequest {
    payment_option: PaymentOption,
    signature: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BapiFinalizeResponse {
    transfer_confirmation: BapiTransferConfirmation,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BapiTransferConfirmation {
    #[serde(rename = "type")]
    pub type_: String,
    pub hash: String,
    pub data: serde_json::Value,
}

impl WalletPayClient {
    #[cfg_attr(feature = "uniffi", uniffi::constructor)]
    pub fn new(
        blockchain_api_base_url: Option<String>,
    ) -> Result<Self, WalletPayError> {
        let url = if let Some(url_str) = blockchain_api_base_url {
            url_str.parse().map_err(|e| {
                WalletPayError::ParseError(format!(
                    "Invalid BAPI base URL: {e}"
                ))
            })?
        } else {
            // Default mock endpoint - TODO: replace with actual BAPI URL
            "https://rpc.walletconnect.org/v1/wallet-pay".parse().map_err(|e| {
                WalletPayError::ParseError(format!(
                    "Failed to parse default URL: {e}"
                ))
            })?
        };

        Ok(Self {
            client: reqwest::Client::new(),
            blockchain_api_base_url: url,
        })
    }
}

// Internal BAPI methods - not exported via uniffi
impl WalletPayClient {
    pub(crate) async fn get_display_data_from_bapi(
        &self,
        accepted_payments: &[AcceptedPayment],
    ) -> Result<BapiDisplayDataResponse, WalletPayError> {
        let url =
            self.blockchain_api_base_url.join("display-data").map_err(|e| {
                WalletPayError::BapiError(format!("Invalid URL: {e}"))
            })?;

        let request = BapiDisplayDataRequest {
            accepted_payments: accepted_payments.to_vec(),
        };

        // TODO: Replace with actual BAPI call when endpoint is ready
        // For now, return mocked response
        let url_string = url.to_string();
        let response =
            self.client.post(url).json(&request).send().await.map_err(|e| {
                WalletPayError::BapiError(format!("Request failed: {e}"))
            })?;

        if !response.status().is_success() {
            return Err(WalletPayError::BapiError(format!(
                "BAPI returned error status: {} on url {}",
                response.status(),
                url_string
            )));
        }

        // Mock response for now
        let mock_response = BapiDisplayDataResponse {
            payment_options: accepted_payments
                .iter()
                .map(|payment| {
                    // Extract chain from asset (e.g., "eip155:1/erc20:0x..." -> "eip155:1")
                    let chain =
                        payment.asset.split('/').next().map(|s| s.to_string());

                    // Extract symbol from asset (mock for now)
                    let symbol = if payment.asset.contains("erc20") {
                        Some("USDC".to_string())
                    } else if payment.asset.contains("slip44:501") {
                        Some("SOL".to_string())
                    } else {
                        None
                    };

                    WalletPayDisplayItemWithParsed {
                        asset: payment.asset.clone(),
                        amount: payment.amount.clone(),
                        chain: chain.clone(),
                        symbol: symbol.clone(),
                        parsed_data: ParsedData {
                            asset_name: symbol
                                .unwrap_or_else(|| "Unknown".to_string()),
                            amount: format!(
                                "{}",
                                u64::from_str_radix(
                                    payment.amount.trim_start_matches("0x"),
                                    16
                                )
                                .unwrap_or(0)
                            ),
                            chain: chain
                                .unwrap_or_else(|| "Unknown".to_string()),
                        },
                    }
                })
                .collect(),
        };

        Ok(mock_response)
    }

    pub(crate) async fn get_action_from_bapi(
        &self,
        payment_option: &PaymentOption,
    ) -> Result<BapiActionResponse, WalletPayError> {
        let url = self.blockchain_api_base_url.join("action").map_err(|e| {
            WalletPayError::BapiError(format!("Invalid URL: {e}"))
        })?;

        let request =
            BapiActionRequest { payment_option: payment_option.clone() };

        // TODO: Replace with actual BAPI call when endpoint is ready
        // For now, return mocked response
        let url_string = url.to_string();
        let response =
            self.client.post(url).json(&request).send().await.map_err(|e| {
                WalletPayError::BapiError(format!("Request failed: {e}"))
            })?;

        if !response.status().is_success() {
            return Err(WalletPayError::BapiError(format!(
                "BAPI returned error status: {} on url {}",
                response.status(),
                url_string
            )));
        }

        // Mock response for now
        let mock_response = BapiActionResponse {
            method: "transaction".to_string(),
            data: serde_json::json!({
                "from": "0x0000000000000000000000000000000000000000",
                "to": payment_option.recipient,
                "value": payment_option.amount,
            }),
            hash: format!("0x{}", hex::encode([0u8; 32])),
        };

        Ok(mock_response)
    }

    pub(crate) async fn finalize_with_bapi(
        &self,
        payment_option: &PaymentOption,
        signature: &str,
    ) -> Result<BapiFinalizeResponse, WalletPayError> {
        let url =
            self.blockchain_api_base_url.join("finalize").map_err(|e| {
                WalletPayError::BapiError(format!("Invalid URL: {e}"))
            })?;

        let request = BapiFinalizeRequest {
            payment_option: payment_option.clone(),
            signature: signature.to_string(),
        };

        // TODO: Replace with actual BAPI call when endpoint is ready
        // For now, return mocked response
        let url_string = url.to_string();
        let response =
            self.client.post(url).json(&request).send().await.map_err(|e| {
                WalletPayError::BapiError(format!("Request failed: {e}"))
            })?;

        if !response.status().is_success() {
            return Err(WalletPayError::BapiError(format!(
                "BAPI returned error status: {} on url {}",
                response.status(),
                url_string
            )));
        }

        // Determine type based on payment option types
        let transfer_type = if payment_option
            .types
            .iter()
            .any(|t| t == "erc3009-authorization")
        {
            "erc3009-authorization"
        } else {
            "erc20-transfer"
        };

        // Mock response for now
        let data_json = serde_json::json!({
            "from": "0x0000000000000000000000000000000000000000",
            "to": payment_option.recipient,
            "value": payment_option.amount,
        });

        let mock_response = BapiFinalizeResponse {
            transfer_confirmation: BapiTransferConfirmation {
                type_: transfer_type.to_string(),
                hash: format!("0x{}", hex::encode([0u8; 32])),
                data: data_json,
            },
        };

        Ok(mock_response)
    }
}

#[cfg_attr(feature = "uniffi", uniffi::export(async_runtime = "tokio"))]
impl WalletPayRequest {
    // Parse raw (proposal or RPC) into UI-ready choices
    pub async fn get_display_data(
        &self,
    ) -> Result<DisplayData, WalletPayError> {
        let bapi_response = self
            .client
            .get_display_data_from_bapi(&self.wallet_pay.accepted_payments)
            .await?;

        let display_items: Vec<WalletPayDisplayItem> = bapi_response
            .payment_options
            .iter()
            .map(|item| WalletPayDisplayItem {
                asset: item.asset.clone(),
                amount: item.amount.clone(),
                chain: item.chain.clone(),
                symbol: item.symbol.clone(),
            })
            .collect();

        Ok(DisplayData { payment_options: display_items })
    }

    // Build signable action for selected option index
    pub async fn get_payment_action(
        &self,
        option_index: u64,
    ) -> Result<WalletPayAction, WalletPayError> {
        let option_index_usize = option_index as usize;
        let payment_option =
            self.payment_options.get(option_index_usize).ok_or_else(|| {
                WalletPayError::InvalidIndex(format!(
                    "Index {} out of range ({} options available)",
                    option_index,
                    self.payment_options.len()
                ))
            })?;

        self.get_action_from_payment_option(payment_option).await
    }

    // Build signable action for selected payment option (raw data)
    pub async fn get_action_from_payment_option(
        &self,
        payment_option: &PaymentOption,
    ) -> Result<WalletPayAction, WalletPayError> {
        let bapi_response =
            self.client.get_action_from_bapi(payment_option).await?;

        Ok(WalletPayAction {
            method: bapi_response.method,
            data: serde_json::to_string(&bapi_response.data).map_err(|e| {
                WalletPayError::ParseError(format!(
                    "Failed to serialize data: {e}"
                ))
            })?,
            hash: bapi_response.hash,
        })
    }

    // Execute/broadcast OR build authorization proof and
    // return CAIP-358-compliant result
    pub async fn finalize(
        &self,
        option_index: u64,
        signature: String,
    ) -> Result<WalletPayResponseResultV1, WalletPayError> {
        let option_index_usize = option_index as usize;
        let payment_option =
            self.payment_options.get(option_index_usize).ok_or_else(|| {
                WalletPayError::InvalidIndex(format!(
                    "Index {} out of range ({} options available)",
                    option_index,
                    self.payment_options.len()
                ))
            })?;

        let bapi_response =
            self.client.finalize_with_bapi(payment_option, &signature).await?;

        let transfer_confirmation = TransferConfirmation {
            type_: bapi_response.transfer_confirmation.type_,
            hash: bapi_response.transfer_confirmation.hash,
            data: serde_json::to_string(
                &bapi_response.transfer_confirmation.data,
            )
            .map_err(|e| {
                WalletPayError::ParseError(format!(
                    "Failed to serialize data: {e}"
                ))
            })?,
        };

        Ok(WalletPayResponseResultV1 { transfer_confirmation })
    }
}

// Factory that accepts raw session proposal OR wallet_pay RPC payload
#[cfg_attr(feature = "uniffi", uniffi::export)]
pub fn create_wallet_pay_request(
    raw_data: String, // JSON string
    blockchain_api_base_url: Option<String>,
) -> Result<WalletPayRequest, WalletPayError> {
    let client = WalletPayClient::new(blockchain_api_base_url)?;

    // Parse JSON string
    let raw_data_value: serde_json::Value = serde_json::from_str(&raw_data)
        .map_err(|e| {
            WalletPayError::ParseError(format!("Failed to parse JSON: {e}"))
        })?;

    // Try to extract walletPay from different possible structures
    let wallet_pay: WalletPay = if let Some(wallet_pay_value) =
        raw_data_value.get("walletPay")
    {
        serde_json::from_value(wallet_pay_value.clone()).map_err(|e| {
            WalletPayError::ParseError(format!(
                "Failed to parse walletPay: {e}"
            ))
        })?
    } else if let Some(params) = raw_data_value.get("params") {
        // Try to get walletPay from params (RPC structure)
        if let Some(wallet_pay_value) = params.get("walletPay") {
            serde_json::from_value(wallet_pay_value.clone()).map_err(|e| {
                WalletPayError::ParseError(format!(
                    "Failed to parse walletPay from params: {e}"
                ))
            })?
        } else {
            return Err(WalletPayError::NoWalletPayRequestFound(
                "No walletPay field found in raw data".to_string(),
            ));
        }
    } else {
        return Err(WalletPayError::NoWalletPayRequestFound(
            "No walletPay field found in raw data".to_string(),
        ));
    };

    // Convert accepted_payments to payment_options
    // For now, we'll infer types from the asset format
    let payment_options: Vec<PaymentOption> = wallet_pay
        .accepted_payments
        .iter()
        .map(|payment| {
            let types = if payment.asset.contains("erc20") {
                vec!["erc20-transfer".to_string()]
            } else if payment.asset.contains("erc3009")
                || payment.asset.contains("erc2612")
            {
                vec!["erc3009-authorization".to_string()]
            } else {
                vec!["transfer".to_string()]
            };

            PaymentOption {
                asset: payment.asset.clone(),
                amount: payment.amount.clone(),
                recipient: payment.recipient.clone(),
                types,
            }
        })
        .collect();

    Ok(WalletPayRequest { client, wallet_pay, payment_options })
}

// ## Summary

// Created a new `WalletPayClient` module in `wallet_pay.rs` following the `Erc6492Client` pattern. Implemented:

// ### Main Components:

// 1. **WalletPayClient** — Client for BAPI calls with configurable base URL
// 2. **WalletPayRequest** — Main request handler with methods:
//    - `get_display_data()` — Returns UI-ready payment options
//    - `get_payment_action(option_index)` — Builds signable action for selected option
//    - `get_action_from_payment_option(payment_option)` — Builds signable action from raw payment option
//    - `finalize(option_index, signature)` — Finalizes payment and returns CAIP-358 result

// 3. **Factory Function** — `create_wallet_pay_request(raw_data, blockchain_api_base_url)` — Parses raw session proposal or wallet_pay RPC payload

// ### Features:

// - ✅ Uniffi-compatible types (using `String` instead of `serde_json::Value` for FFI)
// - ✅ Error handling with `WalletPayError`
// - ✅ Mocked BAPI responses (ready to replace with real endpoints)
// - ✅ Supports parsing from both session proposal and RPC payload structures
// - ✅ All types properly serialized/deserialized

// ### Notes:

// - BAPI endpoints are currently mocked — replace the TODO sections in `get_display_data_from_bapi`, `get_action_from_bapi`, and `finalize_with_bapi` with actual API calls when ready
// - The default BAPI URL is a placeholder — update when the real endpoint is available
// - Index parameters use `u64` for uniffi compatibility (converted to `usize` internally)

// The module is ready to use and follows the same patterns as `Erc6492Client`. All code compiles and passes linting.

// ## Why `blockchain_api_base_url` is required

// The function creates a `WalletPayClient` that needs the BAPI base URL to make HTTP requests. Methods like `get_display_data()`, `get_payment_action()`, and `finalize()` call BAPI endpoints, so the client must know where to send requests.

// ## Why it returns `client`, `wallet_pay`, and `payment_options`

// The `WalletPayRequest` struct stores these three fields, and each is used by its methods:

// 1. **`client` (WalletPayClient)**: Used to make BAPI calls:
//    - `get_display_data()` → `client.get_display_data_from_bapi(&self.wallet_pay.accepted_payments)`
//    - `get_action_from_payment_option()` → `client.get_action_from_bapi(payment_option)`
//    - `finalize()` → `client.finalize_with_bapi(payment_option, &signature)`

// 2. **`wallet_pay` (WalletPay)**: Contains the original parsed wallet pay data (version, accepted_payments, expiry). Used in `get_display_data()` to pass `&self.wallet_pay.accepted_payments` to the BAPI.

// 3. **`payment_options` (Vec<PaymentOption>)**: A converted/enriched version of `accepted_payments` with inferred types (e.g., "erc20-transfer", "erc3009-authorization"). Used by:
//    - `get_payment_action(option_index)` — looks up the option by index
//    - `finalize(option_index, signature)` — looks up the option by index

// The function is a factory that:
// - Creates a client configured with the BAPI URL
// - Parses raw JSON to extract wallet pay data
// - Converts `accepted_payments` into `payment_options` with inferred types
// - Bundles them into a `WalletPayRequest` ready for the wallet pay flow

// This keeps the parsed data and client together so the request object can handle the full flow without re-parsing or re-creating the client.
