use {
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
    // Only production URLs match "://pay.walletconnect." exactly (no subdomain)
    // Everything else (staging., dev., etc.) routes to staging ingest
    let is_prod = payment_link.contains("://pay.walletconnect.");
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
    payload: serde_json::Value,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn send_trace(
    http_client: &HttpClient,
    _bundle_id: &str,
    _project_id: &str,
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
        ts: chrono::Utc::now()
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        version: EVENT_VERSION,
        source_service: format!("{}-{}", sdk_name, sdk_platform),
        sdk_name: sdk_name.to_string(),
        sdk_version: sdk_version.to_string(),
        sdk_platform: sdk_platform.to_string(),
        payload: serde_json::json!({}),
    };

    let is_staging = is_staging_payment(payment_id);
    let url = format!("{}/event", get_ingest_url(is_staging));
    let user_agent = format!("{}/{}", sdk_name, sdk_version);
    let client = http_client.clone();
    let event_type = event;
    let pid = payment_id.to_string();
    crate::spawn::spawn(async move {
        match client
            .post(&url)
            .header("User-Agent", user_agent)
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
                } else {
                    let body = resp.text().await.unwrap_or_default();
                    tracing::warn!(
                        "Trace failed: {:?} for {} -> {} - {}",
                        event_type,
                        pid,
                        status,
                        body
                    );
                }
            }
            Err(e) => {
                tracing::debug!(
                    "Trace error: {:?} for {} - {}",
                    event_type,
                    pid,
                    e
                );
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_payload_serialization() {
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
            payload: serde_json::json!({}),
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"event_id\":\"test-id\""));
        assert!(json.contains("\"payment_id\":\"pay_123\""));
        assert!(json.contains("\"actor\":\"WALLET_SDK\""));
        assert!(json.contains("\"event_type\":\"sdk_initialized\""));
        assert!(json.contains("\"ts\":\"2025-01-07T10:00:00.000Z\""));
        assert!(json.contains("\"version\":1"));
        assert!(json.contains("\"sdk_name\":\"test-sdk\""));
        assert!(json.contains("\"sdk_version\":\"1.0.0\""));
        assert!(json.contains("\"sdk_platform\":\"ios\""));
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
    fn test_get_ingest_url() {
        assert_eq!(get_ingest_url(false), INGEST_PROD_URL);
        assert_eq!(get_ingest_url(true), INGEST_STAGING_URL);
    }
}
