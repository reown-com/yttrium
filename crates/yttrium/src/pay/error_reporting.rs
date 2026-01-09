use {
    reqwest::Client as HttpClient,
    serde::Serialize,
    std::{
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    },
    uuid::Uuid,
};

const PULSE_BASE_URL: &str = "https://pulse.walletconnect.org/e";
const MAX_TRACE_LENGTH: usize = 500;
const MIN_REPORT_INTERVAL_MS: u64 = 1000; // Rate limit: max 1 report per second

static LAST_REPORT_MS: AtomicU64 = AtomicU64::new(0);

/// URL-encode a string for use in query parameters
fn url_encode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                c.to_string()
            }
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

fn build_pulse_url(project_id: &str, sdk_version: &str) -> String {
    format!(
        "{}?projectId={}&st=pay_sdk&sv={}",
        PULSE_BASE_URL,
        url_encode(project_id),
        url_encode(sdk_version)
    )
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorEvent {
    event_id: String,
    bundle_id: String,
    timestamp: u64,
    props: ErrorProps,
}

#[derive(Debug, Serialize)]
struct ErrorProps {
    event: &'static str,
    #[serde(rename = "type")]
    error_type: String,
    properties: ErrorProperties,
}

#[derive(Debug, Serialize)]
struct ErrorProperties {
    topic: String,
    trace: String,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn report_error(
    http_client: &HttpClient,
    bundle_id: &str,
    project_id: &str,
    sdk_name: &str,
    sdk_version: &str,
    error_type: &str,
    payment_id: &str,
    trace: &str,
) {
    // Rate limiting: skip if reported too recently
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let last = LAST_REPORT_MS.load(Ordering::Relaxed);
    if now_ms.saturating_sub(last) < MIN_REPORT_INTERVAL_MS {
        return;
    }
    LAST_REPORT_MS.store(now_ms, Ordering::Relaxed);

    // Truncate trace to limit data exposure
    let sanitized_trace = if trace.len() > MAX_TRACE_LENGTH {
        format!("{}...[truncated]", &trace[..MAX_TRACE_LENGTH])
    } else {
        trace.to_string()
    };

    let event = ErrorEvent {
        event_id: Uuid::new_v4().to_string(),
        bundle_id: bundle_id.to_string(),
        timestamp: now_ms,
        props: ErrorProps {
            event: "error",
            error_type: error_type.to_string(),
            properties: ErrorProperties {
                topic: payment_id.to_string(),
                trace: sanitized_trace,
            },
        },
    };

    let url = build_pulse_url(project_id, sdk_version);
    let user_agent = format!("{}/{}", sdk_name, sdk_version);
    let client = http_client.clone();
    crate::spawn::spawn(async move {
        if let Err(e) = client
            .post(&url)
            .header("User-Agent", user_agent)
            .json(&event)
            .send()
            .await
        {
            tracing::debug!("Error reporting failed: {}", e);
        }
    });
}

/// Trait for error types that provide a stable error type name for telemetry.
/// This avoids parsing Debug output which is fragile and can break on renames.
pub trait HasErrorType {
    fn error_type(&self) -> &'static str;
}

/// Get the error type name from an error that implements HasErrorType.
pub(crate) fn error_type_name<E: HasErrorType>(error: &E) -> &'static str {
    error.error_type()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_encode() {
        assert_eq!(url_encode("simple"), "simple");
        assert_eq!(url_encode("with space"), "with%20space");
        assert_eq!(url_encode("a&b=c"), "a%26b%3Dc");
        assert_eq!(url_encode("test?query"), "test%3Fquery");
    }

    #[test]
    fn test_build_pulse_url() {
        let url = build_pulse_url("123", "rust-0.1.0");
        assert_eq!(
            url,
            "https://pulse.walletconnect.org/e?projectId=123&st=pay_sdk&sv=rust-0.1.0"
        );
    }

    #[test]
    fn test_build_pulse_url_with_special_chars() {
        let url = build_pulse_url("id&evil=1", "v1.0 beta");
        assert!(url.contains("projectId=id%26evil%3D1"));
        assert!(url.contains("sv=v1.0%20beta"));
    }

    #[test]
    fn test_error_type_name_with_trait() {
        #[allow(dead_code)]
        enum TestError {
            NotFound(String),
            Timeout,
            Custom { code: u32 },
        }

        impl HasErrorType for TestError {
            fn error_type(&self) -> &'static str {
                match self {
                    Self::NotFound(_) => "NotFound",
                    Self::Timeout => "Timeout",
                    Self::Custom { .. } => "Custom",
                }
            }
        }

        assert_eq!(
            error_type_name(&TestError::NotFound("test".to_string())),
            "NotFound"
        );
        assert_eq!(error_type_name(&TestError::Timeout), "Timeout");
        assert_eq!(error_type_name(&TestError::Custom { code: 1 }), "Custom");
    }

    #[test]
    fn test_error_event_serialization() {
        let event = ErrorEvent {
            event_id: "test-id".to_string(),
            bundle_id: "com.example.app".to_string(),
            timestamp: 1234567890000,
            props: ErrorProps {
                event: "error",
                error_type: "PaymentNotFound".to_string(),
                properties: ErrorProperties {
                    topic: "pay_123".to_string(),
                    trace: "Error: Payment not found".to_string(),
                },
            },
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"eventId\":\"test-id\""));
        assert!(json.contains("\"bundleId\":\"com.example.app\""));
        assert!(json.contains("\"event\":\"error\""));
        assert!(json.contains("\"type\":\"PaymentNotFound\""));
        assert!(json.contains("\"topic\":\"pay_123\""));
        assert!(json.contains("\"trace\":\"Error: Payment not found\""));
    }

    /// Run with: PROJECT_ID=your_project_id cargo +nightly test -p yttrium --features=pay test_real_error_event -- --ignored --nocapture
    #[tokio::test]
    #[ignore]
    async fn test_real_error_event() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let project_id = std::env::var("PROJECT_ID")
            .expect("PROJECT_ID environment variable must be set");
        let sdk_name = "pay_sdk";
        let sdk_version = "rust-0.1.0";
        let bundle_id = "com.test.yttrium";

        let event = ErrorEvent {
            event_id: Uuid::new_v4().to_string(),
            bundle_id: bundle_id.to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            props: ErrorProps {
                event: "error",
                error_type: "TestError".to_string(),
                properties: ErrorProperties {
                    topic: "pay_sdk_integration_test".to_string(),
                    trace: "This is a test error from yttrium pay SDK integration test".to_string(),
                },
            },
        };

        let url = build_pulse_url(&project_id, sdk_version);
        println!("Sending to URL: {}", url);
        println!(
            "Event JSON: {}",
            serde_json::to_string_pretty(&event).unwrap()
        );

        let client = HttpClient::builder()
            .user_agent(format!("{}/{}", sdk_name, sdk_version))
            .build()
            .unwrap();
        let response = client.post(&url).json(&event).send().await;

        assert!(
            response.is_ok(),
            "Failed to send error event: {:?}",
            response.err()
        );
        let resp = response.unwrap();
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        assert!(
            status.is_success(),
            "Error event request failed with status: {}, body: {}",
            status,
            body
        );
        println!("Success! Response: {}", body);
    }
}
