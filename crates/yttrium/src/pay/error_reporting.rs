use {
    reqwest::Client as HttpClient,
    serde::Serialize,
    std::{
        panic::PanicHookInfo,
        sync::OnceLock,
        time::{SystemTime, UNIX_EPOCH},
    },
    uuid::Uuid,
};

const PULSE_BASE_URL: &str = "https://pulse.walletconnect.org/e";

static PANIC_CONFIG: OnceLock<PanicConfig> = OnceLock::new();

struct PanicConfig {
    bundle_id: String,
    project_id: String,
    sdk_name: String,
    sdk_version: String,
}

fn build_pulse_url(project_id: &str, sdk_name: &str, sdk_version: &str) -> String {
    format!(
        "{}?projectId={}&st={}&sv={}",
        PULSE_BASE_URL, project_id, sdk_name, sdk_version
    )
}

/// Install a global panic hook that reports panics to the pulse endpoint.
/// Should be called once during initialization (e.g., in WalletConnectPay::new).
/// Only the first call will have effect; subsequent calls are ignored.
pub fn install_panic_hook(
    bundle_id: String,
    project_id: String,
    sdk_name: String,
    sdk_version: String,
) {
    if PANIC_CONFIG
        .set(PanicConfig { bundle_id, project_id, sdk_name, sdk_version })
        .is_ok()
    {
        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            report_panic(panic_info);
            previous_hook(panic_info);
        }));
    }
}

fn report_panic(panic_info: &PanicHookInfo<'_>) {
    let Some(config) = PANIC_CONFIG.get() else {
        return;
    };

    let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
        s.clone()
    } else {
        "Unknown panic".to_string()
    };

    let location = panic_info
        .location()
        .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
        .unwrap_or_else(|| "unknown location".to_string());

    let backtrace = std::backtrace::Backtrace::capture().to_string();

    let trace = format!(
        "PANIC at {}\nMessage: {}\nBacktrace:\n{}",
        location, message, backtrace
    );

    let event = ErrorEvent {
        event_id: Uuid::new_v4().to_string(),
        bundle_id: config.bundle_id.clone(),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0),
        props: ErrorProps {
            event: "error",
            error_type: "Panic".to_string(),
            properties: ErrorProperties { topic: "pay_sdk".to_string(), trace },
        },
    };

    let url =
        build_pulse_url(&config.project_id, &config.sdk_name, &config.sdk_version);
    let user_agent = format!("{}/{}", config.sdk_name, config.sdk_version);

    // Use a new thread with its own tokio runtime since we're in a panic hook
    #[cfg(not(test))]
    std::thread::spawn(move || {
        if let Ok(rt) =
            tokio::runtime::Builder::new_current_thread().enable_all().build()
        {
            rt.block_on(async {
                let client = HttpClient::new();
                let _ = client
                    .post(&url)
                    .header("User-Agent", user_agent)
                    .json(&event)
                    .send()
                    .await;
            });
        }
    });

    #[cfg(test)]
    {
        let _ = event; // Don't send in tests
    }
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

pub(crate) fn report_error(
    http_client: &HttpClient,
    bundle_id: &str,
    project_id: &str,
    sdk_name: &str,
    sdk_version: &str,
    error_type: &str,
    topic: &str,
    trace: &str,
) {
    let event = ErrorEvent {
        event_id: Uuid::new_v4().to_string(),
        bundle_id: bundle_id.to_string(),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0),
        props: ErrorProps {
            event: "error",
            error_type: error_type.to_string(),
            properties: ErrorProperties {
                topic: topic.to_string(),
                trace: trace.to_string(),
            },
        },
    };

    let url = build_pulse_url(project_id, sdk_name, sdk_version);
    let user_agent = format!("{}/{}", sdk_name, sdk_version);
    let client = http_client.clone();
    let fut = async move {
        match client
            .post(&url)
            .header("User-Agent", user_agent)
            .json(&event)
            .send()
            .await
        {
            Ok(response) => {
                if !response.status().is_success() {
                    tracing::debug!(
                        "Pay error reporting failed: {}",
                        response.status()
                    );
                }
            }
            Err(e) => {
                tracing::debug!("Pay error reporting failed: {}", e);
            }
        }
    };

    #[cfg(not(test))]
    crate::spawn::spawn(fut);

    #[cfg(test)]
    {
        let _ = fut; // Don't send in tests
    }
}

/// Helper to get error type name from an error enum
pub(crate) fn error_type_name<E: std::fmt::Debug>(error: &E) -> String {
    let debug = format!("{:?}", error);
    debug.split('(').next().unwrap_or("Unknown").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_pulse_url() {
        let url = build_pulse_url(
            "123",
            "pay_sdk",
            "rust-0.1.0",
        );
        assert_eq!(
            url,
            "https://pulse.walletconnect.org/e?projectId=123&st=pay_sdk&sv=rust-0.1.0"
        );
    }

    #[test]
    fn test_error_type_name() {
        #[derive(Debug)]
        enum TestError {
            NotFound(String),
            InvalidRequest(String),
        }

        assert_eq!(
            error_type_name(&TestError::NotFound("test".to_string())),
            "NotFound"
        );
        assert_eq!(
            error_type_name(&TestError::InvalidRequest("msg".to_string())),
            "InvalidRequest"
        );
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

    #[tokio::test]
    #[ignore] // Run with: PROJECT_ID=your_project_id cargo +nightly test -p yttrium --features=pay test_real_error_event -- --ignored --nocapture
    async fn test_real_error_event() {
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
                event: "ERROR",
                error_type: "TestError".to_string(),
                properties: ErrorProperties {
                    topic: "pay_sdk_integration_test".to_string(),
                    trace: "This is a test error from yttrium pay SDK integration test".to_string(),
                },
            },
        };

        let url = build_pulse_url(&project_id, sdk_name, sdk_version);
        println!("Sending to URL: {}", url);
        println!("Event JSON: {}", serde_json::to_string_pretty(&event).unwrap());

        let client = HttpClient::builder()
            .user_agent(format!("{}/{}", sdk_name, sdk_version))
            .build()
            .unwrap();
        let response = client
            .post(&url)
            .header("Origin", "https://test.walletconnect.com")
            .header("x-sdk-type", sdk_name)
            .header("x-sdk-version", sdk_version)
            .header("x-bundle-id", bundle_id)
            .json(&event)
            .send()
            .await;

        assert!(response.is_ok(), "Failed to send error event: {:?}", response.err());
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
