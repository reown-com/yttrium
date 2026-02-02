use {
    super::{
        CollectDataFieldResult, PaymentStatus, SdkConfig, WalletConnectPay,
    },
    alloy::{
        dyn_abi::TypedData,
        primitives::Address,
        signers::{SignerSync, local::PrivateKeySigner},
    },
    reqwest::Client as HttpClient,
    serde::{Deserialize, Serialize},
    serial_test::serial,
    std::str::FromStr,
};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct CreatePaymentRequest {
    reference_id: String,
    amount: PaymentAmount,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct PaymentAmount {
    unit: String,
    value: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct CreatePaymentResponse {
    payment_id: String,
    status: String,
    gateway_url: String,
    expires_at: u64,
    poll_in_ms: u64,
    is_final: bool,
}

const POS_API_BASE_URL: &str = "https://api.pay.walletconnect.com";

fn get_merchant_api_key() -> String {
    std::env::var("MERCHANT_API_KEY")
        .expect("MERCHANT_API_KEY environment variable must be set")
}

fn get_merchant_id() -> String {
    std::env::var("MERCHANT_ID")
        .unwrap_or_else(|_| "gancho-test-collectdata".to_string())
}

fn get_expected_test_address() -> Option<Address> {
    std::env::var("TEST_WALLET_ADDRESS")
        .ok()
        .and_then(|addr| Address::from_str(&addr).ok())
}
const CHAIN_BASE: &str = "eip155:8453";
const CHAIN_POLYGON: &str = "eip155:137";

#[derive(Debug, thiserror::Error)]
enum PayTestError {
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(&'static str),
    #[error("Invalid private key: {0}")]
    InvalidPrivateKey(String),
    #[error("Address mismatch: expected {expected}, got {actual}")]
    AddressMismatch { expected: String, actual: String },
    #[error("POS API error: {0}")]
    PosApi(String),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Signing error: {0}")]
    Signing(String),
}

struct PosApiClient {
    http_client: HttpClient,
}

impl PosApiClient {
    fn new() -> Self {
        Self {
            http_client: HttpClient::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| HttpClient::new()),
        }
    }

    async fn create_payment(
        &self,
        amount_cents: u64,
    ) -> Result<CreatePaymentResponse, PayTestError> {
        let reference_id =
            format!("t-{}", &uuid::Uuid::new_v4().to_string()[..32]);
        let body = CreatePaymentRequest {
            reference_id,
            amount: PaymentAmount {
                unit: "iso4217/USD".to_string(),
                value: amount_cents.to_string(),
            },
        };

        let response = self
            .http_client
            .post(format!("{}/v1/merchant/payment", POS_API_BASE_URL))
            .header("Api-Key", get_merchant_api_key())
            .header("Merchant-Id", get_merchant_id())
            .header("Sdk-Name", "yttrium-e2e-test")
            .header("Sdk-Version", env!("CARGO_PKG_VERSION"))
            .header("Sdk-Platform", "rust-tests")
            .header("Idempotency-Key", uuid::Uuid::new_v4().to_string())
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(PayTestError::PosApi(format!("{}: {}", status, body)));
        }

        response
            .json::<CreatePaymentResponse>()
            .await
            .map_err(PayTestError::from)
    }
}

struct TestWallet {
    signer: PrivateKeySigner,
    address: Address,
}

impl TestWallet {
    fn from_env() -> Result<Self, PayTestError> {
        let private_key =
            std::env::var("TEST_WALLET_PRIVATE_KEY").map_err(|_| {
                PayTestError::MissingEnvVar("TEST_WALLET_PRIVATE_KEY")
            })?;

        let key = private_key.strip_prefix("0x").unwrap_or(&private_key);
        let signer = PrivateKeySigner::from_str(key)
            .map_err(|e| PayTestError::InvalidPrivateKey(e.to_string()))?;

        let address = signer.address();

        // Optionally validate address if TEST_WALLET_ADDRESS env var is set
        if let Some(expected) = get_expected_test_address() {
            if address != expected {
                return Err(PayTestError::AddressMismatch {
                    expected: expected.to_string(),
                    actual: address.to_string(),
                });
            }
        }

        Ok(Self { signer, address })
    }

    fn caip10_account(&self, chain_id: &str) -> String {
        format!("{}:{}", chain_id, self.address)
    }

    fn sign_typed_data_v4(
        &self,
        params_json: &str,
    ) -> Result<String, PayTestError> {
        let params: Vec<serde_json::Value> = serde_json::from_str(params_json)?;

        // params[1] is the typed data (either as string or object)
        let typed_data_value = params.get(1).ok_or_else(|| {
            PayTestError::Signing("params[1] missing".to_string())
        })?;

        let typed_data_str = if typed_data_value.is_string() {
            typed_data_value.as_str().unwrap().to_string()
        } else {
            serde_json::to_string(typed_data_value)?
        };

        let typed_data: TypedData = serde_json::from_str(&typed_data_str)
            .map_err(|e| {
                PayTestError::Signing(format!("Invalid typed data: {}", e))
            })?;

        let hash = typed_data.eip712_signing_hash().map_err(|e| {
            PayTestError::Signing(format!("EIP-712 hash error: {}", e))
        })?;

        let signature = self
            .signer
            .sign_hash_sync(&hash)
            .map_err(|e| PayTestError::Signing(e.to_string()))?;

        Ok(format!("0x{}", hex::encode(signature.as_bytes())))
    }
}

fn test_sdk_config() -> SdkConfig {
    SdkConfig {
        base_url: POS_API_BASE_URL.to_string(),
        project_id: std::env::var("REOWN_PROJECT_ID").ok(),
        sdk_name: "yttrium-e2e-test".to_string(),
        sdk_version: env!("CARGO_PKG_VERSION").to_string(),
        sdk_platform: "rust-tests".to_string(),
        bundle_id: "com.yttrium.e2e.tests".to_string(),
        api_key: Some(get_merchant_api_key()),
        app_id: None,
        client_id: None,
    }
}

#[tokio::test]
#[serial(pay)]
async fn e2e_payment_options_only() {
    let wallet = TestWallet::from_env().expect("Failed to load test wallet");
    println!("Test wallet address: {}", wallet.address);

    let pos_client = PosApiClient::new();
    let payment =
        pos_client.create_payment(1).await.expect("Failed to create payment");
    println!(
        "Created payment: {} (status: {})",
        payment.payment_id, payment.status
    );
    println!("Gateway URL: {}", payment.gateway_url);

    let pay_client = WalletConnectPay::new(test_sdk_config()).unwrap();
    let accounts = vec![wallet.caip10_account(CHAIN_BASE)];

    let response = pay_client
        .get_payment_options(payment.gateway_url, accounts, true)
        .await
        .expect("Failed to get payment options");

    assert!(!response.payment_id.is_empty());
    println!(
        "Payment {} has {} options",
        response.payment_id,
        response.options.len()
    );

    if let Some(info) = &response.info {
        println!(
            "Amount: {} {}",
            info.amount.value, info.amount.display.asset_symbol
        );
        println!("Merchant: {}", info.merchant.name);
    }

    assert!(
        !response.options.is_empty(),
        "Expected at least one payment option"
    );
}

#[tokio::test]
#[serial(pay)]
async fn e2e_payment_happy_path() {
    let wallet = TestWallet::from_env().expect("Failed to load test wallet");
    println!("Test wallet address: {}", wallet.address);

    // Step 1: Create payment via POS API ($0.01 = 1 cent)
    let pos_client = PosApiClient::new();
    let payment =
        pos_client.create_payment(1).await.expect("Failed to create payment");
    println!(
        "Created payment: {} (status: {})",
        payment.payment_id, payment.status
    );
    println!("Gateway URL: {}", payment.gateway_url);

    // Step 2: Get payment options
    let pay_client = WalletConnectPay::new(test_sdk_config()).unwrap();
    let accounts = vec![
        wallet.caip10_account(CHAIN_BASE),
        wallet.caip10_account(CHAIN_POLYGON),
    ];

    let options_response = pay_client
        .get_payment_options(payment.gateway_url.clone(), accounts, true)
        .await
        .expect("Failed to get payment options");

    println!("Got {} payment options", options_response.options.len());
    assert!(
        !options_response.options.is_empty(),
        "Expected at least one payment option"
    );

    if let Some(info) = &options_response.info {
        println!(
            "Payment amount: {} {}",
            info.amount.value, info.amount.display.asset_symbol
        );
        println!("Merchant: {}", info.merchant.name);
    }

    // Step 3: Select first available option (prefer Base chain)
    let selected_option = options_response
        .options
        .iter()
        .find(|opt| {
            opt.actions.iter().any(|a| a.wallet_rpc.chain_id == CHAIN_BASE)
        })
        .or_else(|| options_response.options.first())
        .expect("No payment option available");

    println!(
        "Selected option: {} ({} actions)",
        selected_option.id,
        selected_option.actions.len()
    );

    // Step 4: Get required payment actions
    let actions = pay_client
        .get_required_payment_actions(
            options_response.payment_id.clone(),
            selected_option.id.clone(),
        )
        .await
        .expect("Failed to get required actions");

    println!("Got {} required actions", actions.len());

    // Step 5: Sign each action
    let mut signatures = Vec::new();
    for action in &actions {
        let rpc = &action.wallet_rpc;
        println!("Action: chain={}, method={}", rpc.chain_id, rpc.method);

        match rpc.method.as_str() {
            "eth_signTypedData_v4" => {
                let signature = wallet
                    .sign_typed_data_v4(&rpc.params)
                    .expect("Failed to sign typed data");
                println!("Signed: {}...", &signature[..20]);
                signatures.push(signature);
            }
            method => {
                panic!("Unsupported RPC method: {}", method);
            }
        }
    }

    // Step 6: Handle collect data if required
    let collected_data: Option<Vec<CollectDataFieldResult>> =
        options_response.collect_data.as_ref().map(|cd| {
            cd.fields
                .iter()
                .map(|field| {
                    use super::CollectDataFieldType;
                    let value = match field.id.as_str() {
                        "firstName" => "Test".to_string(),
                        "lastName" => "User".to_string(),
                        "email" => "test@example.com".to_string(),
                        _ => match field.field_type {
                            CollectDataFieldType::Date => {
                                "1990-01-15".to_string()
                            }
                            CollectDataFieldType::Text => {
                                "test-value".to_string()
                            }
                            CollectDataFieldType::Checkbox => {
                                "true".to_string()
                            }
                        },
                    };
                    CollectDataFieldResult { id: field.id.clone(), value }
                })
                .collect()
        });

    // Step 7: Confirm payment with signatures
    let result = pay_client
        .confirm_payment(
            options_response.payment_id.clone(),
            selected_option.id.clone(),
            signatures,
            collected_data,
            Some(30000), // 30 second max poll
        )
        .await
        .expect("Failed to confirm payment");

    println!(
        "Payment result: status={:?}, is_final={}",
        result.status, result.is_final
    );

    // Step 8: Verify final status
    assert!(result.is_final, "Expected final status");
    match result.status {
        PaymentStatus::Succeeded => {
            println!("Payment succeeded!");
        }
        PaymentStatus::Failed => {
            println!(
                "Payment failed - check if test wallet has sufficient USDC on Base"
            );
        }
        status => {
            panic!("Unexpected payment status: {:?}", status);
        }
    }
}
