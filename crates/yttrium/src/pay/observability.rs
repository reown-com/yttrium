use {
    reqwest::Client as HttpClient,
    serde::Serialize,
    std::time::{SystemTime, UNIX_EPOCH},
    uuid::Uuid,
};

const PULSE_BASE_URL: &str = "https://pulse.walletconnect.org/e";

fn build_pulse_url(project_id: &str, sdk_version: &str) -> String {
    format!(
        "{}?projectId={}&st=pay_sdk&sv={}",
        PULSE_BASE_URL, project_id, sdk_version
    )
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceEvent {
    SdkInitialized,
    PaymentOptionsRequested,
    PaymentOptionsReceived,
    PaymentOptionsFailed,
    ConfirmPaymentCalled,
    ConfirmPaymentSucceeded,
    ConfirmPaymentFailed,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TracePayload {
    event_id: String,
    bundle_id: String,
    timestamp: u64,
    props: TraceProps,
}

#[derive(Debug, Serialize)]
struct TraceProps {
    event: &'static str,
    #[serde(rename = "type")]
    trace_type: TraceEvent,
    properties: TraceProperties,
}

#[derive(Debug, Serialize)]
struct TraceProperties {
    topic: String,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn send_trace(
    http_client: &HttpClient,
    bundle_id: &str,
    project_id: &str,
    sdk_name: &str,
    sdk_version: &str,
    event: TraceEvent,
    topic: &str,
) {
    let payload = TracePayload {
        event_id: Uuid::new_v4().to_string(),
        bundle_id: bundle_id.to_string(),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0),
        props: TraceProps {
            event: "trace",
            trace_type: event,
            properties: TraceProperties { topic: topic.to_string() },
        },
    };

    let url = build_pulse_url(project_id, sdk_version);
    let user_agent = format!("{}/{}", sdk_name, sdk_version);
    let client = http_client.clone();
    crate::spawn::spawn(async move {
        let _ = client
            .post(&url)
            .header("User-Agent", user_agent)
            .json(&payload)
            .send()
            .await;
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_pulse_url() {
        let url = build_pulse_url("123", "rust-0.1.0");
        assert_eq!(
            url,
            "https://pulse.walletconnect.org/e?projectId=123&st=pay_sdk&sv=rust-0.1.0"
        );
    }

    #[test]
    fn test_trace_event_serialization() {
        let payload = TracePayload {
            event_id: "test-id".to_string(),
            bundle_id: "com.example.app".to_string(),
            timestamp: 1234567890000,
            props: TraceProps {
                event: "trace",
                trace_type: TraceEvent::SdkInitialized,
                properties: TraceProperties { topic: "test_topic".to_string() },
            },
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"eventId\":\"test-id\""));
        assert!(json.contains("\"bundleId\":\"com.example.app\""));
        assert!(json.contains("\"event\":\"trace\""));
        assert!(json.contains("\"type\":\"sdk_initialized\""));
        assert!(json.contains("\"topic\":\"test_topic\""));
    }

    #[test]
    fn test_all_trace_events_serialize() {
        let events = [
            (TraceEvent::SdkInitialized, "sdk_initialized"),
            (TraceEvent::PaymentOptionsRequested, "payment_options_requested"),
            (TraceEvent::PaymentOptionsReceived, "payment_options_received"),
            (TraceEvent::PaymentOptionsFailed, "payment_options_failed"),
            (TraceEvent::ConfirmPaymentCalled, "confirm_payment_called"),
            (TraceEvent::ConfirmPaymentSucceeded, "confirm_payment_succeeded"),
            (TraceEvent::ConfirmPaymentFailed, "confirm_payment_failed"),
        ];

        for (event, expected) in events {
            let json = serde_json::to_string(&event).unwrap();
            assert_eq!(json, format!("\"{}\"", expected));
        }
    }
}
