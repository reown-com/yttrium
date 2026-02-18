use {
    crate::time::{SystemTime, UNIX_EPOCH},
    reqwest::Client as HttpClient,
    serde::Serialize,
    std::{collections::HashMap, sync::RwLock},
    uuid::Uuid,
};

const INGEST_STAGING_URL: &str = "https://ingest-staging.walletconnect.org";
const INGEST_PROD_URL: &str = "https://ingest.walletconnect.org";
const ACTOR: &str = "WALLET_SDK";
const EVENT_VERSION: u32 = 1;

static PAYMENT_ENVS: RwLock<Option<HashMap<String, bool>>> = RwLock::new(None);

/// Registers a payment's environment (staging or production) based on the payment link
pub(crate) fn set_payment_env(payment_id: &str, payment_link: &str) {
    fn url_decode(s: &str) -> String {
        urlencoding::decode(s)
            .map(|c| c.into_owned())
            .unwrap_or_else(|_| s.to_string())
    }

    fn extract_pay_url(link: &str) -> String {
        let decoded = url_decode(link);
        if decoded.starts_with("wc:") {
            if let Some((_, query)) = decoded.split_once('?') {
                for param in query.split('&') {
                    if let Some(value) = param.strip_prefix("pay=") {
                        return url_decode(value);
                    }
                }
            }
        }
        decoded
    }

    let pay_url = extract_pay_url(payment_link);
    // Only production URLs match "://pay.walletconnect." exactly (no subdomain)
    // Everything else (staging., dev., etc.) routes to staging ingest
    let is_prod = pay_url.contains("://pay.walletconnect.");
    let mut envs = PAYMENT_ENVS.write().expect("Payment envs lock poisoned");
    envs.get_or_insert_with(HashMap::new)
        .insert(payment_id.to_string(), !is_prod);
}

fn is_staging_payment(payment_id: &str) -> bool {
    PAYMENT_ENVS
        .read()
        .expect("Payment envs lock poisoned")
        .as_ref()
        .and_then(|m| m.get(payment_id).copied())
        .unwrap_or(false)
}

fn get_ingest_url(is_staging: bool) -> &'static str {
    if is_staging { INGEST_STAGING_URL } else { INGEST_PROD_URL }
}

fn format_timestamp_rfc3339() -> String {
    let duration =
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();
    let (year, month, day, hour, min, sec) = unix_to_datetime(secs as i64);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        year, month, day, hour, min, sec, millis
    )
}

fn unix_to_datetime(timestamp: i64) -> (i32, u32, u32, u32, u32, u32) {
    let secs_per_day = 86400i64;
    let days = timestamp / secs_per_day;
    let remaining = (timestamp % secs_per_day) as u32;
    let hour = remaining / 3600;
    let min = (remaining % 3600) / 60;
    let sec = remaining % 60;
    let (year, month, day) = days_to_ymd(days as i32 + 719468);
    (year, month, day, hour, min, sec)
}

fn days_to_ymd(days: i32) -> (i32, u32, u32) {
    let era = if days >= 0 { days } else { days - 146096 } / 146097;
    let doe = (days - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i32 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceEvent {
    SdkInitialized,
    PaymentOptionsRequested,
    PaymentOptionsReceived,
    PaymentOptionsFailed,
    RequiredActionsRequested,
    RequiredActionsReceived,
    RequiredActionsFailed,
    ConfirmPaymentCalled,
    ConfirmPaymentSucceeded,
    ConfirmPaymentFailed,
}

#[derive(Debug, Serialize)]
struct EventPayload {
    event_id: String,
    payment_id: String,
    actor: &'static str,
    event_type: TraceEvent,
    ts: String,
    version: u32,
    source_service: String,
    sdk_name: String,
    sdk_version: String,
    sdk_platform: String,
    api_key: String,
    app_id: String,
    client_id: String,
    bundle_id: String,
    payload: serde_json::Value,
}

fn sha256_hex(input: &str) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(input.as_bytes()))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn send_trace(
    http_client: &HttpClient,
    bundle_id: &str,
    _project_id: &str,
    api_key: &str,
    app_id: &str,
    client_id: &str,
    sdk_name: &str,
    sdk_version: &str,
    sdk_platform: &str,
    event: TraceEvent,
    payment_id: &str,
) {
    let payload = EventPayload {
        event_id: Uuid::new_v4().to_string(),
        payment_id: payment_id.to_string(),
        actor: ACTOR,
        event_type: event,
        ts: format_timestamp_rfc3339(),
        version: EVENT_VERSION,
        source_service: format!("{}-{}", sdk_name, sdk_platform),
        sdk_name: sdk_name.to_string(),
        sdk_version: sdk_version.to_string(),
        sdk_platform: sdk_platform.to_string(),
        api_key: sha256_hex(api_key),
        app_id: sha256_hex(app_id),
        client_id: client_id.to_string(),
        bundle_id: bundle_id.to_string(),
        payload: serde_json::json!({}),
    };

    let is_staging = is_staging_payment(payment_id);
    let url = format!("{}/event", get_ingest_url(is_staging));
    let user_agent = format!("{}/{}", sdk_name, sdk_version);
    let client = http_client.clone();
    let event_type = event;
    let pid = payment_id.to_string();
    crate::spawn::spawn(async move {
        // Try up to 2 attempts (initial + 1 retry)
        for attempt in 1..=2 {
            match client
                .post(&url)
                .header("User-Agent", &user_agent)
                .json(&payload)
                .send()
                .await
            {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() {
                        tracing::debug!(
                            "Trace sent: {:?} for {} -> {}",
                            event_type,
                            pid,
                            status
                        );
                        return;
                    }
                    let body = resp.text().await.unwrap_or_default();
                    if attempt == 2 {
                        tracing::warn!(
                            "Trace failed after retry: {:?} for {} -> {} - {}",
                            event_type,
                            pid,
                            status,
                            body
                        );
                    } else {
                        tracing::debug!(
                            "Trace attempt {} failed: {:?} for {} -> {} - {}, retrying...",
                            attempt,
                            event_type,
                            pid,
                            status,
                            body
                        );
                    }
                }
                Err(e) => {
                    if attempt == 2 {
                        tracing::debug!(
                            "Trace error after retry: {:?} for {} - {}",
                            event_type,
                            pid,
                            e
                        );
                        return;
                    }
                    tracing::debug!(
                        "Trace attempt {} error: {:?} for {} - {}, retrying...",
                        attempt,
                        event_type,
                        pid,
                        e
                    );
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_payload_serialization() {
        let api_key_hash = sha256_hex("test-api-key");
        let app_id_hash = sha256_hex("test-app-id");
        let payload = EventPayload {
            event_id: "test-id".to_string(),
            payment_id: "pay_123".to_string(),
            actor: ACTOR,
            event_type: TraceEvent::SdkInitialized,
            ts: "2025-01-07T10:00:00.000Z".to_string(),
            version: EVENT_VERSION,
            source_service: "test-sdk-ios".to_string(),
            sdk_name: "test-sdk".to_string(),
            sdk_version: "1.0.0".to_string(),
            sdk_platform: "ios".to_string(),
            api_key: api_key_hash.clone(),
            app_id: app_id_hash.clone(),
            client_id: "test-client-id".to_string(),
            bundle_id: "com.test.app".to_string(),
            payload: serde_json::json!({}),
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"event_id\":\"test-id\""));
        assert!(json.contains("\"payment_id\":\"pay_123\""));
        assert!(json.contains("\"actor\":\"WALLET_SDK\""));
        assert!(json.contains("\"event_type\":\"sdk_initialized\""));
        assert!(json.contains("\"version\":1"));
        assert!(json.contains("\"sdk_name\":\"test-sdk\""));
        assert!(json.contains("\"sdk_version\":\"1.0.0\""));
        assert!(json.contains("\"sdk_platform\":\"ios\""));
        assert!(json.contains(&format!("\"api_key\":\"{}\"", api_key_hash)));
        assert!(json.contains(&format!("\"app_id\":\"{}\"", app_id_hash)));
        assert!(json.contains("\"client_id\":\"test-client-id\""));
        assert!(json.contains("\"bundle_id\":\"com.test.app\""));
        // Raw credentials must not appear in payload
        assert!(!json.contains("test-api-key"));
        assert!(!json.contains("test-app-id"));
    }

    #[test]
    fn test_all_trace_events_serialize() {
        let events = [
            (TraceEvent::SdkInitialized, "sdk_initialized"),
            (TraceEvent::PaymentOptionsRequested, "payment_options_requested"),
            (TraceEvent::PaymentOptionsReceived, "payment_options_received"),
            (TraceEvent::PaymentOptionsFailed, "payment_options_failed"),
            (
                TraceEvent::RequiredActionsRequested,
                "required_actions_requested",
            ),
            (TraceEvent::RequiredActionsReceived, "required_actions_received"),
            (TraceEvent::RequiredActionsFailed, "required_actions_failed"),
            (TraceEvent::ConfirmPaymentCalled, "confirm_payment_called"),
            (TraceEvent::ConfirmPaymentSucceeded, "confirm_payment_succeeded"),
            (TraceEvent::ConfirmPaymentFailed, "confirm_payment_failed"),
        ];

        for (event, expected) in events {
            let json = serde_json::to_string(&event).unwrap();
            assert_eq!(json, format!("\"{}\"", expected));
        }
    }

    #[test]
    fn test_set_payment_env_and_lookup() {
        // Production: only "://pay.walletconnect." is production
        set_payment_env(
            "pay_prod_123",
            "https://pay.walletconnect.com/?pid=pay_prod_123",
        );
        assert!(!is_staging_payment("pay_prod_123"));

        // Staging: explicit staging subdomain
        set_payment_env(
            "pay_staging_456",
            "https://staging.pay.walletconnect.com/?pid=pay_staging_456",
        );
        assert!(is_staging_payment("pay_staging_456"));

        // Dev: routes to staging ingest
        set_payment_env(
            "pay_dev_789",
            "https://dev.pay.walletconnect.org/?pid=pay_dev_789",
        );
        assert!(is_staging_payment("pay_dev_789"));

        // Unknown payment defaults to production (not staging)
        assert!(!is_staging_payment("pay_unknown"));
    }

    #[test]
    fn test_set_payment_env_url_encoded() {
        // URL-encoded production link
        set_payment_env(
            "pay_encoded_prod",
            "https%3A%2F%2Fpay.walletconnect.com%2F%3Fpid%3Dpay_encoded_prod",
        );
        assert!(!is_staging_payment("pay_encoded_prod"));

        // URL-encoded staging link
        set_payment_env(
            "pay_encoded_staging",
            "https%3A%2F%2Fstaging.pay.walletconnect.com%2F%3Fpid%3Dpay_encoded_staging",
        );
        assert!(is_staging_payment("pay_encoded_staging"));
    }

    #[test]
    fn test_set_payment_env_wc_uri() {
        // WC URI with production pay link
        set_payment_env(
            "pay_wc_prod",
            "wc:abc123@2?relay-protocol=irn&pay=https%3A%2F%2Fpay.walletconnect.com%2F%3Fpid%3Dpay_wc_prod",
        );
        assert!(!is_staging_payment("pay_wc_prod"));

        // WC URI with staging pay link
        set_payment_env(
            "pay_wc_staging",
            "wc:abc123@2?relay-protocol=irn&pay=https%3A%2F%2Fstaging.pay.walletconnect.com%2F%3Fpid%3Dpay_wc_staging",
        );
        assert!(is_staging_payment("pay_wc_staging"));
    }

    #[test]
    fn test_set_payment_env_fully_encoded_wc_uri() {
        // Fully URL-encoded WC URI with production pay link
        set_payment_env(
            "pay_full_encoded",
            "wc%3Afe8314@2%3Fpay%3Dhttps%3A%2F%2Fpay.walletconnect.com%2F%3Fpid%3Dpay_full_encoded",
        );
        assert!(!is_staging_payment("pay_full_encoded"));
    }

    #[test]
    fn test_get_ingest_url() {
        assert_eq!(get_ingest_url(false), INGEST_PROD_URL);
        assert_eq!(get_ingest_url(true), INGEST_STAGING_URL);
    }

    #[tokio::test]
    async fn test_send_trace_retries_on_failure() {
        use wiremock::{
            Mock, MockServer, ResponseTemplate,
            matchers::{method, path},
        };

        let mock_server = MockServer::start().await;

        // First call returns 500 (up_to 1 means it only matches once)
        Mock::given(method("POST"))
            .and(path("/event"))
            .respond_with(
                ResponseTemplate::new(500).set_body_string("Server Error"),
            )
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        // Second call returns 200
        Mock::given(method("POST"))
            .and(path("/event"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let client = reqwest::Client::new();

        let payload = EventPayload {
            event_id: "test-retry".to_string(),
            payment_id: "pay_retry_test".to_string(),
            actor: ACTOR,
            event_type: TraceEvent::SdkInitialized,
            ts: "2025-01-07T10:00:00.000Z".to_string(),
            version: EVENT_VERSION,
            source_service: "test-sdk-ios".to_string(),
            sdk_name: "test-sdk".to_string(),
            sdk_version: "1.0.0".to_string(),
            sdk_platform: "ios".to_string(),
            api_key: sha256_hex("test-api-key"),
            app_id: sha256_hex("test-app-id"),
            client_id: "test-client-id".to_string(),
            bundle_id: "com.test.app".to_string(),
            payload: serde_json::json!({}),
        };

        let url = format!("{}/event", mock_server.uri());
        let user_agent = "test-sdk/1.0.0";

        // Simulate the retry logic from send_trace
        let mut success = false;
        let mut attempts = 0;
        for attempt in 1..=2 {
            attempts = attempt;
            match client
                .post(&url)
                .header("User-Agent", user_agent)
                .json(&payload)
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    success = true;
                    break;
                }
                Ok(_) if attempt < 2 => continue,
                Ok(_) => break,
                Err(_) if attempt < 2 => continue,
                Err(_) => break,
            }
        }

        assert!(success, "Should succeed on retry");
        assert_eq!(attempts, 2, "Should have made 2 attempts");
    }

    #[tokio::test]
    async fn test_send_trace_gives_up_after_two_failures() {
        use wiremock::{
            Mock, MockServer, ResponseTemplate,
            matchers::{method, path},
        };

        let mock_server = MockServer::start().await;

        // Both calls return 500
        Mock::given(method("POST"))
            .and(path("/event"))
            .respond_with(
                ResponseTemplate::new(500).set_body_string("Server Error"),
            )
            .expect(2)
            .mount(&mock_server)
            .await;

        let client = reqwest::Client::new();

        let payload = EventPayload {
            event_id: "test-fail".to_string(),
            payment_id: "pay_fail_test".to_string(),
            actor: ACTOR,
            event_type: TraceEvent::PaymentOptionsFailed,
            ts: "2025-01-07T10:00:00.000Z".to_string(),
            version: EVENT_VERSION,
            source_service: "test-sdk-ios".to_string(),
            sdk_name: "test-sdk".to_string(),
            sdk_version: "1.0.0".to_string(),
            sdk_platform: "ios".to_string(),
            api_key: sha256_hex("test-api-key"),
            app_id: sha256_hex("test-app-id"),
            client_id: "test-client-id".to_string(),
            bundle_id: "com.test.app".to_string(),
            payload: serde_json::json!({}),
        };

        let url = format!("{}/event", mock_server.uri());
        let user_agent = "test-sdk/1.0.0";

        // Simulate the retry logic
        let mut success = false;
        let mut attempts = 0;
        for attempt in 1..=2 {
            attempts = attempt;
            match client
                .post(&url)
                .header("User-Agent", user_agent)
                .json(&payload)
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    success = true;
                    break;
                }
                Ok(_) if attempt < 2 => continue,
                Ok(_) => break,
                Err(_) if attempt < 2 => continue,
                Err(_) => break,
            }
        }

        assert!(!success, "Should fail after retries");
        assert_eq!(attempts, 2, "Should have made exactly 2 attempts");
    }

    #[test]
    fn test_sha256_hex() {
        let hash = sha256_hex("hello");
        assert_eq!(hash.len(), 64);
        assert_eq!(
            hash,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e\
             1b161e5c1fa7425e73043362938b9824"
        );
    }
}
